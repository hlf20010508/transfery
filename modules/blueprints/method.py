# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from time import time
from sanic import Blueprint
from sanic.response import json
import modules.sql as sql
import modules.socket as socket

method_bp = Blueprint("method")

@method_bp.route('/method/push_text', methods=['GET', 'POST'])
async def push_text(request):
    print('received push text request')

    if request.method == 'GET':
        content = request.args['content'][0]
    else:
        content = request.json['content']

    if content:
        newItem = {
            "content": content.strip(),
            "type": "text",
            "timestamp": int(time()) * 1000,
        }
        
        await socket.push_item(None, newItem)
        print('text pushed')

        return json({"success": True})
    else:
        print('no content')

        return json({"success": False})


@method_bp.route('/method/latest_text', methods=['GET'])
async def latest_text(request):
    print('received get latest text request')

    item = await sql.query_latest_text()

    if item:
        return json({
            "success": True,
            "content": item['content']
        })
    else:
        return json({"success": False})