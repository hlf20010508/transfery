# -*- coding: utf-8 -*-
# A Convenient Temporary Message and File transfer Project
# (C) 2022 L-ING <hlf01@icloud.com>
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
table = config['table']  # mysql table

# cache directory to store uploaded temporary files
cache_path = config['cache_path']

app = Sanic(__name__)
template = SanicJinja2(app, pkg_name='run')
app.static('/static', './static/')
# the max size of file can be uploaded, 10GB
app.config.REQUEST_MAX_SIZE = 10*1024*1024*1024

socketio = socketio.AsyncServer(async_mode='sanic')
socketio.attach(app)

client = OSS_minio.Client()

async def query_items(start, amount):
    _db = db()
    result = await _db.query('select * from %s order by time desc, id desc LIMIT %d, %d' % (table, start, amount))
    _db.close()
    return result


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
    temp = '.'.join(temp)
    temp = temp.split()
    temp = '_'.join(temp)
    return temp


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


@app.route('/post/upload', methods=['POST'])
async def upload(request):
    print('received upload request')
    if not os.path.exists(config['cache_path']):
        os.mkdir(config['cache_path'])
    file = request.files.get('file')
    time = request.form.get('time')

    file_name = rename(file.name, time)
    save_path = os.path.join(cache_path, file_name)

    # save to cache
    async with aiofiles.open(save_path, 'wb') as temp:
        await temp.write(file.body)
    temp.close()

    # upload to minio from cache
    await client.upload(file_name, save_path)
    os.remove(save_path)
    print("uploaded")
    return json({"success": True, "fileName": file_name})


@app.route('/get/download', methods=['GET'])
async def download(request):
    print('received download request')
    file_name = request.args['fileName'][0]

    if config['local_minio']:
        # if minio is local, change the host to create url properly,
        # or it will use 127.0.0.1 to create url
        protocal = 'https://' if config['secure_minio'] else 'http://'
        host = protocal + config['host_minio']
        url = await client.get_download_url(file_name, change_host=host)

    print('url pushed')
    return json({"success": True, "url": url})


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
    if item['change']:
        # if the item to be removed is showing time, it's next item
        # should show time
        update_item(item['change']['id'])
        print('changed next item')

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
