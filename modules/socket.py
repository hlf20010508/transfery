# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from modules.client import socketio


@socketio.on("connect")
def connect(sid, environ, auth):
    print('client %s connected' % sid)


@socketio.on("disconnect")
def disconnect(sid):
    print('client %s disconnected' % sid)