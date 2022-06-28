import os
import threading
import flask
import OSS_minio
import mysql_db
import config as myconfig

thread_lock=threading.Lock()

config = myconfig.load()

table=config['table']

app = flask.Flask(__name__)


def query_items(start, amount):
    db= mysql_db.db()
    result=db.query('select * from %s order by id desc LIMIT %d, %d' % (table, start, amount))
    db.close()
    return result


def push_item(item):
    db= mysql_db.db()
    db.table_insert(table, item)
    db.close()

def update_item(content,time):
    db= mysql_db.db()
    db.query('update %s set showTime=1 where content="%s" and time="%s"'%(table,content,time))
    db.close()

def remove_item(content,time):
    db= mysql_db.db()
    db.query('delete from %s where content="%s" and time="%s"' % (table, content,time))
    db.close()

def rename(old_filename,time):
    temp=old_filename.split('.')
    temp[0]+='_'+str(time)[:-3]
    return '.'.join(temp)

@app.route('/', methods=['GET'])
def index():
    return flask.render_template('index.html')


@app.route('/get/page', methods=['GET'])
def page():
    thread_lock.acquire()
    item_per_page = config['item_per_page']
    page_id = int(flask.request.args['page'])
    start = page_id*item_per_page
    result = query_items(start, item_per_page)
    thread_lock.release()
    return flask.jsonify({'messages': result})


@app.route('/post/upload', methods=['POST'])
def upload():
    thread_lock.acquire()
    f = flask.request.files.get('file')
    size = flask.request.form.get('size')
    time = flask.request.form.get('time')
    print('uploading ...')
    file_name=rename(f.filename,time)
    client = OSS_minio.Client()
    client.upload_stream(file_name, f, size)  # 流式上传
    print("uploaded")
    thread_lock.release()
    return flask.jsonify({"success": True,"fileName":file_name})


@app.route('/get/download', methods=['GET'])
def download():
    thread_lock.acquire()
    file_name = flask.request.args['fileName']
    client=None
    if config['local_minio']:
        client = OSS_minio.Client(host=config['host_minio'])
    else:
        client=OSS_minio.Client()
    url=client.get_download_url(file_name)
    thread_lock.release()
    return flask.jsonify({"success": True,"url":url})


@app.route('/post/message', methods=['POST'])
def send():
    thread_lock.acquire()
    item = flask.request.get_json(silent=True)
    push_item(item)
    thread_lock.release()
    return flask.jsonify({"success": True})


@app.route('/post/remove', methods=['POST'])
def remove():
    thread_lock.acquire()
    item = flask.request.get_json(silent=True)
    if item['change']:
        update_item(table,item['change']['content'],item['change']['time'])
    remove_item(item["content"],item['time'])
    if item['type'] == 'file':
        client = OSS_minio.Client()
        client.remove(item['fileName'])
    thread_lock.release()
    return flask.jsonify({"success": True})
