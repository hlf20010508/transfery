import OSS_minio
import flask
import mysql_db

# db=mysql_db.db()

client=OSS_minio.init()

app = flask.Flask(__name__)

@app.route('/', methods=['POST', 'GET'])
def index():
    if flask.request.method == 'POST':
        f = flask.request.files.get('file')
        if f:
            print('uploading ...')
            client.upload_stream(f.filename,f) #流式上传
            return flask.redirect(flask.url_for('index'))
        else:
            return flask.redirect(flask.url_for('download_stream'))
    return flask.render_template('index.html')

@app.route('/download')
def download_stream():
    file_name='app.py'
    print('downloading ...')
    file_stream=client.download_stream(file_name) #流式下载
    return flask.send_file(file_stream,download_name=file_name,as_attachment=True)