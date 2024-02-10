# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from time import time
from sanic import Blueprint
from sanic.response import json
import modules.sql as sql
from modules.client import socketio

method_bp = Blueprint("method")

@method_bp.route('/method/push_text', methods=['GET', 'POST'])
async def push_text(request):
    print('received push text request')

    if request.method == 'GET':
        content = request.args['content'][0]
    else:
        content = request.json['content']

    if content:
        item = {
            "content": content.strip(),
            "type": "text",
            "timestamp": int(time()) * 1000,
        }
        
        item["id"] = await sql.insert(item)

        await socketio.emit('newItem', item)
        print('text pushed')

        return json({"success": True})
    else:
        print('no content')

        return json({"success": False})


@method_bp.route('/method/latest_text', methods=['GET'])
async def latest_text(request):
    print('received get latest text request')

    item = (await sql.query_items(start=0, amount=1))[0]

    if item:
        return json({
            "success": True,
            "content": item['content']
        })
    else:
        return json({"success": False})