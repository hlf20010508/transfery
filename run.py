# :project: transfery
# :author: L-ING
# :copyright: (C) 2022 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from sanic import Sanic
from sanic_jinja2 import SanicJinja2
from cryptography.fernet import Fernet
from modules.blueprints import message_bp, upload_bp, download_bp, method_bp, login_bp
from modules.client import socketio
from modules.env import PORT, AUTO_RELOAD, Secret
from modules.sql import query_auth_key
import modules.socket

app = Sanic(__name__)
template = SanicJinja2(app, pkg_name="run")
app.static("/static", "./static/")
# the max size of file can be uploaded, 10GB
app.config.REQUEST_MAX_SIZE = 10 * 1024 * 1024 * 1024

app.blueprint(message_bp)
app.blueprint(upload_bp)
app.blueprint(download_bp)
app.blueprint(method_bp)
app.blueprint(login_bp)

socketio.attach(app)


@app.route("/")
async def index(request):
    return template.render("index.html", request)


@app.before_server_start
async def setup(app, loop):
    key_str = await query_auth_key()
    Secret.key = Fernet(key_str)


if __name__ == "__main__":
    app.run(host="0.0.0.0", port=PORT, auto_reload=AUTO_RELOAD)
