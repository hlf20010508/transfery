import flask
import time
import OSS_minio
import mysql_db
import config as myconfig
import os

config = myconfig.load()

db, table = mysql_db.db()

client = OSS_minio.init()

host_minio = os.path.join(config['host_minio'], config['bucket'])

app = flask.Flask(__name__)


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


@app.route('/post/files', methods=['POST'])
def upload():
    f = flask.request.files.get('file')
    print('uploading ...')
    client.upload_stream(f.filename, f)  # 流式上传

    time_parse = int(round(time.time() * 1000))
    return {
        "time": time_parse,
        "url": os.path.join(host_minio, f.filename)
    }


@app.route('/post/message', methods=['POST'])
def send():
    item = flask.request.get_json(silent=True)
    push_item(item)
    return {
        "sucess": True
    }
