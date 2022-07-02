import threading
from flask import Flask, render_template, request, jsonify
from flask_socketio import SocketIO, emit
import OSS_minio
from mysql_db import db
import config as myconfig

thread_lock = threading.Lock()

config = myconfig.load()

table = config['table']

app = Flask(__name__)
app.config['SECRET_KEY'] = 'secret!'
socketio = SocketIO(app, cors_allowed_origins='*')


def query_items(start, amount):
    _db = db()
    result = _db.query(
        'select * from %s order by id desc LIMIT %d, %d' % (table, start, amount))
    _db.close()
    return result

def sync_items(id):
    _db = db()
    result = _db.query(
        'select * from %s where id > %s' % id)
    _db.close()
    return result


def push_item(item):
    _db = db()
    id = _db.table_insert(table, item)
    _db.close()
    return id


def update_item(id):
    _db = db()
    _db.query('update %s set showTime=1 where id="%d"' %
              (table, id))
    _db.close()


def remove_item(id):
    _db = db()
    _db.query('delete from %s where id="%d"' %
              (table, id))
    _db.close()


def remove_all_items():
    _db = db()
    _db.query('delete from %s' % table)
    _db.close()


def rename(old_filename, time):
    temp = old_filename.split('.')
    temp[0] += '_'+str(time)[:-3]
    return '.'.join(temp)


@app.route('/')
def index():
    return render_template('index.html')


@app.route('/get/page', methods=['GET'])
def page():
    thread_lock.acquire()
    item_per_page = config['item_per_page']
    page_id = int(request.args['page'])
    start = page_id*item_per_page
    result = query_items(start, item_per_page)
    thread_lock.release()
    return jsonify({'messages': result})

    
@app.route('/get/sync', methods=['GET'])
def page():
    thread_lock.acquire()
    last_id = int(request.args['lastId'])
    result=sync_items(last_id)
    thread_lock.release()
    return jsonify({'newItems': result})


@app.route('/post/upload', methods=['POST'])
def upload():
    thread_lock.acquire()
    print('received upload request')
    f = request.files.get('file')
    size = request.form.get('size')
    time = request.form.get('time')
    file_name = rename(f.filename, time)
    client = OSS_minio.Client()
    client.upload_stream(file_name, f, size)  # 流式上传
    print("uploaded")
    thread_lock.release()
    return jsonify({"success": True, "fileName": file_name})


@app.route('/get/download', methods=['GET'])
def download():
    thread_lock.acquire()
    print('received download request')
    file_name = request.args['fileName']
    client = None
    if config['local_minio']:
        client = OSS_minio.Client(host=config['host_minio'])
    else:
        client = OSS_minio.Client()
    url = client.get_download_url(file_name)
    print('url pushed')
    thread_lock.release()
    return jsonify({"success": True, "url": url})


@socketio.on('pushItem')
def handle_item(item):
    thread_lock.acquire()
    print('received item: ', item)
    id = push_item(item)
    print('pushed to db')
    item['id'] = id
    emit('getNewItem', item, broadcast=True, include_self=False)
    print('broadcasted')
    thread_lock.release()
    return id, True


@socketio.on('remove')
def remove(item):
    thread_lock.acquire()
    print('received item to be removed: ', item)
    if item['change']:
        update_item(item['change']['id'])
        print('changed next item')
    remove_item(item["id"])
    print('removed item in db')
    if item['type'] == 'file':
        client = OSS_minio.Client()
        client.remove(item['fileName'])
        print('removed item in oss')
    emit('removeItem', item['id'], broadcast=True, include_self=False)
    print('broadcasted')
    thread_lock.release()
    return True


@socketio.on('removeAll')
def remove_all():
    thread_lock.acquire()
    print('received remove all item request')
    remove_all_items()
    print('removed all items in db')
    client = OSS_minio.Client()
    client.remove_all()
    print('all items removed in oss')
    emit('removeAll', broadcast=True, include_self=False)
    thread_lock.release()
    return True
