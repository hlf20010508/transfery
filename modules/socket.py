# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from modules.client import socketio

connection_number = 0

@socketio.on("connect")
async def connect(sid, environ, auth):
    global connection_number
    connection_number += 1
    print('client %s connected, connection number %d' % (sid, connection_number))

    await socketio.emit("connectionNumber", connection_number)


@socketio.on("disconnect")
async def disconnect(sid):
    global connection_number
    connection_number -= 1
    print('client %s disconnected, connection number %d' % (sid, connection_number))

    await socketio.emit("connectionNumber", connection_number)

@socketio.on("progress")
async def progress(sid, data):
    await socketio.emit('progress', data, skip_sid=sid)