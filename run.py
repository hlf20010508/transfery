import os
import aiofiles
from sanic import Sanic
from sanic.response import json
from sanic_jinja2 import SanicJinja2
import socketio
import OSS_minio
from mysql_db import db
import config as myconfig

config = myconfig.load()
table = config['table']

cache_path=config['cache_path']

app = Sanic(__name__)
template = SanicJinja2(app,pkg_name='run')
app.static('/static', './static/')
app.config.REQUEST_MAX_SIZE = 10*1024*1024*1024

socketio = socketio.AsyncServer(async_mode='sanic')
socketio.attach(app)


async def query_items(start, amount):
    _db = db()
    result = await _db.query('select * from %s order by id desc LIMIT %d, %d' % (table, start, amount))
    _db.close()
    return result

async def sync_items(id):
    _db = db()
    result =await _db.query(
        'select * from %s where id > %s' % (table,id))
    _db.close()
    return result


async def push_item(item):
    _db = db()
    id = await _db.table_insert(table, item)
    _db.close()
    return id


async def update_item(id):
    _db = db()
    await _db.query('update %s set showTime=1 where id="%d"' %
              (table, id))
    _db.close()


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
    return '.'.join(temp)


@app.route('/')
async def index(request):
    # 清空缓存
    try:
        for i in os.listdir(config['cache_path']):
            os.remove(os.path.join(config['cache_path'],i))
        return template.render('index.html',request)
    except:
        return template.render('index.html',request)

    


@app.route('/get/page', methods=['GET'])
async def page(request):
    item_per_page = config['item_per_page']
    page_id = int(request.args['page'][0])
    start = page_id*item_per_page
    result = await query_items(start, item_per_page)
    return json({'messages': result})

    
@app.route('/get/sync', methods=['GET'])
async def sync(request):
    last_id = int(request.args['lastId'][0])
    result=await sync_items(last_id)
    return json({'newItems': result})


@app.route('/post/upload', methods=['POST'])
async def upload(request):
    print('received upload request')
    if not os.path.exists(config['cache_path']):
        os.mkdir(config['cache_path'])
    file = request.files.get('file')
    size=request.form.get('size')
    time = request.form.get('time')
    file_name = rename(file.name, time)
    save_path=os.path.join(cache_path,file_name)
    async with aiofiles.open(save_path, 'wb') as temp:
        await temp.write(file.body)
    temp.close()
    client = OSS_minio.Client()
    temp=open(save_path,'rb')
    await client.upload(file_name,temp,size)
    temp.close()
    for i in os.listdir(config['cache_path']):
        os.remove(os.path.join(config['cache_path'],i))
    print("uploaded")
    return json({"success": True, "fileName": file_name})


@app.route('/get/download', methods=['GET'])
async def download(request):
    print('received download request')
    file_name = request.args['fileName'][0]
    client = None
    if config['local_minio']:
        client = OSS_minio.Client(host=config['host_minio'])
    else:
        client = OSS_minio.Client()
    url = await client.get_download_url(file_name)
    print('url pushed')
    return json({"success": True, "url": url})


@socketio.event
async def pushItem(sid,item):
    print('received item: ', item)
    id = await push_item(item)
    print('pushed to db')
    item['id'] = id
    await socketio.emit('getNewItem', item, skip_sid=sid)
    print('broadcasted')
    return id, True


@socketio.event
async def remove(sid,item):
    print('received item to be removed: ', item)
    if item['change']:
        update_item(item['change']['id'])
        print('changed next item')
    await remove_item(item["id"])
    print('removed item in db')
    if item['type'] == 'file':
        client = OSS_minio.Client()
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
    client = OSS_minio.Client()
    await client.remove_all()
    print('all items removed in oss')
    await socketio.emit('removeAll', skip_sid=sid)
    return True
