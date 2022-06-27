import os
import time
from queue import Empty
import flask
from flask_socketio import SocketIO
import OSS_minio
import minio_progress
import mysql_db
import config as myconfig



config = myconfig.load()

db, table = mysql_db.db()

client = OSS_minio.init()

upload_progress_thread=None

host_minio = os.path.join(config['host_minio'], config['bucket'])

app = flask.Flask(__name__)
socketio = SocketIO(app)


def query_items(start, amount):
    return db.query('select * from %s order by time desc LIMIT %d, %d' % (table, start, amount))


def push_item(item):
    db.table_insert(table, item)


@app.route('/', methods=['GET'])
def index():
    return flask.render_template('index.html')


@app.route('/get/page', methods=['GET'])
def page():
    item_per_page = config['item_per_page']
    page_id = int(flask.request.args['page'])
    start = page_id*item_per_page
    result = query_items(start, item_per_page)
    return flask.jsonify({'messages': result})

@socketio.on('connect', namespace='/socket/connect')
def socket_connect():
    print('socket connected')
    socketio.start_background_task(target=socket_thread)

def socket_thread():
    global upload_progress_thread
    interval=upload_progress_thread.interval
    initial_time = time.time()
    displayed_time = 0
    percentage=0
    while percentage!=100:
        try:
                # display every interval secs
                task = upload_progress_thread.percentage_queue.get(timeout=interval)
        except Empty:
            elapsed_time = time.time() - initial_time
            if elapsed_time > displayed_time:
                displayed_time = elapsed_time
            continue
        percentage = task
        socketio.emit('socket_response', {'percentage': percentage}, namespace='/socket/connect')
        socketio.sleep(1)

@socketio.on('disconnect', namespace='/socket/disconnect')
def socket_disconnect():
    print('socket disconnected')

@app.route('/post/upload', methods=['POST'])
def upload():
    f = flask.request.files.get('file')
    size=flask.request.form.get('size')
    print(size)
    print('uploading ...')
    # global upload_progress_thread
    # upload_progress_thread=minio_progress.Progress()
    # upload_progress_thread.start()
    client.upload_stream(f.filename, f,size)  # 流式上传
    # upload_progress_thread=None
    return flask.jsonify({"success": True})


@app.route('/post/download', methods=['POST'])
def download():
    item = flask.request.get_json(silent=True)
    file_name = item['content']
    print('downloading ...')
    file_stream = client.download_stream(file_name)  # 流式下载
    return flask.send_file(file_stream,download_name=file_name, as_attachment=True)


@app.route('/post/message', methods=['POST'])
def send():
    item = flask.request.get_json(silent=True)
    push_item(item)
    return flask.jsonify({"success": True})


@app.route('/post/remove', methods=['POST'])
def remove():
    item = flask.request.get_json(silent=True)
    if item['change']:
        db.table_update(table, {"showTime": True}, "time", item['change'])
    db.query('delete from %s where time="%s"' % (table, item['time']))
    if item['type']=='file':
        try:
            client.remove(item['content'])
        except:
            print('no such file in bucket')
    return flask.jsonify({"success": True})
