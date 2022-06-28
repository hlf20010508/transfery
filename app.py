import os
import threading
import flask
import OSS_minio
import mysql_db
import config as myconfig

thread_lock=threading.Lock()

config = myconfig.load()

db= mysql_db.db()
table=config['table']

client = OSS_minio.init()

host_minio = os.path.join(config['host_minio'], config['bucket'])

app = flask.Flask(__name__)


def query_items(start, amount):
    return db.query('select * from %s order by id desc LIMIT %d, %d' % (table, start, amount))


def push_item(item):
    print(item)
    db.table_insert(table, item)

def update_item(content,time):
    db.query('update %s set showTime=1 where content="%s" and time="%s"'%(table,content,time))

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
    client.upload_stream(file_name, f, size)  # 流式上传
    print("uploaded")
    thread_lock.release()
    return flask.jsonify({"success": True,"fileName":file_name})


@app.route('/get/download', methods=['GET'])
def download():
    thread_lock.acquire()
    file_name = flask.request.args['fileName']
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
    db.query('delete from %s where content="%s" and time="%s"' % (table, item["content"],item['time']))
    if item['type'] == 'file':
        client.remove(item['fileName'])
    thread_lock.release()
    return flask.jsonify({"success": True})
