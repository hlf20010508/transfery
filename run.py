# :project: transfery
# :author: L-ING
# :copyright: (C) 2022 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

import os
from time import time as std_time
from sanic import Sanic
from sanic.response import json
from sanic_jinja2 import SanicJinja2
import socketio
import OSS_minio
from mysql_db import db
import config as myconfig

config = myconfig.load()
table = config['table']  # mysql table

# cache directory to store uploaded temporary files
cache_path = config['cache_path']

app = Sanic(__name__)
template = SanicJinja2(app, pkg_name='run')
app.static('/static', './static/')
# the max size of file can be uploaded, 10GB
app.config.REQUEST_MAX_SIZE = 10*1024*1024*1024

socketio = socketio.AsyncServer(async_mode='sanic', cors_allowed_origins="*")
socketio.attach(app)

client = OSS_minio.Client()


async def query_items(start, amount):
    _db = db()
    result = await _db.query('select * from %s order by time desc, id desc LIMIT %d, %d' % (table, start, amount))
    _db.close()
    return result


async def query_latest_text():
    _db = db()
    result = await _db.query('select * from %s where type="text" order by time desc, id desc LIMIT 1' % table)
    _db.close()
    if result:
        return result[0]
    else:
        return None


async def sync_items(id):
    _db = db()
    result = await _db.query(
        'select * from %s where id > %s' % (table, id))
    _db.close()
    return result


async def push_item(item):
    _db = db()
    id = await _db.table_insert(table, item)
    _db.close()
    return id


async def remove_item(id):
    _db = db()
    await _db.query('delete from %s where id="%d"' %
                    (table, id))
    _db.close()


async def remove_all_items():
    _db = db()
    await _db.query('delete from %s' % table)
    _db.close()


def rename(old_filename, time):
    temp = old_filename.split('.')
    temp[0] += '_'+str(time)[:-3]
    temp = '.'.join(temp)
    temp = temp.split()
    temp = '_'.join(temp)
    return temp

def should_show_time(time1, time2):
    if abs(time1 - time2) > 1000 * 60:
        return True
    return False


@app.route('/')
async def index(request):
    # clear cache
    try:
        for i in os.listdir(config['cache_path']):
            os.remove(os.path.join(config['cache_path'], i))
        return template.render('index.html', request)
    except:
        return template.render('index.html', request)


@app.route('/get/page', methods=['GET'])
async def page(request):
    print('received new page request')
    item_per_page = config['item_per_page']
    start = int(request.args['size'][0])
    result = await query_items(start, item_per_page)
    print('new page pushed')
    return json({'messages': result})


@app.route('/get/sync', methods=['GET'])
async def sync(request):
    print('received sync request')
    last_id = int(request.args['lastId'][0])
    print('last_id:', last_id)
    result = await sync_items(last_id)
    print('items to be syncd: ', result)
    print('synced')
    return json({'newItems': result})


@app.route('/post/getUploadId', methods=['POST'])
async def get_upload_id(request):
    print('received get upload id request')
    content = request.json['content']
    time = request.json['time']
    file_name = rename(content, time)
    upload_id = await client.create_multipart_upload_id(file_name)
    print('upload id pushed')
    return json({"success": True, "uploadId": upload_id, "fileName": file_name})


@app.route('/post/uploadPart', methods=['POST'])
async def upload_part(request):
    file_part = request.files.get('filePart').body
    content = request.form.get('content')
    upload_id = request.form.get('uploadId')
    part_number = request.form.get('partNumber')
    etag = await client.multipart_upload(content, upload_id, file_part, part_number)
    return json({"success": True, "etag": etag})


@app.route('/post/completeUpload', methods=['POST'])
async def complete_upload(request):
    print('received complete upload request')
    content = request.json['content']
    upload_id = request.json['uploadId']
    parts = request.json['parts']
    await client.complete_multipart_upload(content, upload_id, parts)
    print('complete upload finished')
    return json({"success": True})


@app.route('/get/download', methods=['GET'])
async def download(request):
    print('received download request')
    file_name = request.args['fileName'][0]

    url = await client.get_download_url(file_name)
        
    print('url pushed')
    return json({"success": True, "url": url})


@app.route('/method/push_text', methods=['GET', 'POST'])
async def push_text(request):
    print('received push text request')
    if request.method == 'GET':
        content = request.args['content'][0]
    else:
        content = request.json['content']
    if content:
        time_now = int(std_time()) * 1000
        show_time = True
        item = await query_latest_text()
        if item:
            time_last = item['time']
            show_time = should_show_time(time_now, time_last)
        newItem = {
            "content": content,
            "type": "text",
            "time": time_now,
        }
        
        await pushItem(None, newItem)
        print('text pushed')
        return json({"success": True})
    else:
        print('no content')
        return json({"success": False})


@app.route('/method/latest_text', methods=['GET'])
async def latest_text(request):
    print('received get latest text request')
    item = await query_latest_text()
    if item:
        result = item['content'].strip()
        return json({
            "success": True,
            "content": result
        })
    else:
        return json({"success": False})


@app.route('/method/remove_latest_text', methods=['GET'])
async def remove_latest_text(request):
    print('received remove latest text request')
    item = await query_latest_text()
    if item:
        await remove(None, item)
        return json({"success": True})
    else:
        return json({"success": False})


@app.route('/method/remove_all', methods=['GET'])
async def remove_all(request):
    print('received get remove all request')
    await removeAll(None)
    return json({"success": True})


@socketio.event
async def pushItem(sid, item):
    print('received item: ', item)
    id = await push_item(item)
    print('pushed to db')
    item['id'] = id
    await socketio.emit('getNewItem', item, skip_sid=sid)
    print('broadcasted')
    return id, True


@socketio.event
async def remove(sid, item):
    print('received item to be removed: ', item)

    await remove_item(item["id"])
    print('removed item in db')

    if item['type'] == 'file':
        await client.remove(item['fileName'])
        print('removed item in oss')

    await socketio.emit('removeItem', item['id'], skip_sid=sid)
    print('broadcasted')
    return True


@socketio.event
async def removeAll(sid):
    print('received remove all item request')
    await remove_all_items()
    print('removed all items in db')
    await client.remove_all()
    print('all items removed in oss')
    await socketio.emit('removeAll', skip_sid=sid)
    return True


@socketio.event
def connect(sid, environ, auth):
    print('client %s connected' % sid)


@socketio.event
def disconnect(sid):
    print('client %s disconnected' % sid)
