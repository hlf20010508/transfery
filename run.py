# :project: transfery
# :author: L-ING
# :copyright: (C) 2022 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from sanic import Sanic
from sanic_jinja2 import SanicJinja2
from modules.blueprints import (
    message_bp,
    upload_bp,
    download_bp,
    method_bp
)
from modules.client import socketio
from modules.env import PORT
import modules.socket

app = Sanic(__name__)
template = SanicJinja2(app, pkg_name='run')
app.static('/static', './static/')
# the max size of file can be uploaded, 10GB
app.config.REQUEST_MAX_SIZE = 10*1024*1024*1024

app.blueprint(message_bp)
app.blueprint(upload_bp)
app.blueprint(download_bp)
app.blueprint(method_bp)

socketio.attach(app)

@app.route('/')
async def index(request):
    return template.render('index.html', request)

if __name__ == "__main__":
    app.run(
        host='0.0.0.0',
        port=PORT
    )