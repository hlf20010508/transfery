# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from time import time
from sanic import Blueprint, response
import modules.sql as sql
from modules.client import socketio

method_bp = Blueprint("method")


def verify_token(token):
    return True


@method_bp.route('/method/push_text', methods=['GET', 'POST'])
async def push_text(request):
    print('received push text request')

    if request.method == 'GET':
        content = request.args['content'][0]
        is_private = request.args['isPrivate'][0]
        token = request.args['token'][0]
    else:
        content = request.json['content']
        is_private = request.json['isPrivate']
        token = request.json['token']

    if verify_token(token) or not is_private:
        if content:
            item = {
                "content": content.strip(),
                "timestamp": int(time()) * 1000,
                "isPrivate": is_private,
                "type": "text",
            }
            
            item["id"] = await sql.insert_message(item)

            await socketio.emit('newItem', item)
            print('text pushed')

            return response.json({"success": True})
        else:
            print('no content')

            return response.json({"success": False})



@method_bp.route('/method/latest_text', methods=['GET'])
async def latest_text(request):
    print('received get latest text request')

    token = request.args['token'][0]

    item = (await sql.query_items(start=0, amount=1, access_private=verify_token(token)))[0]

    if item:
        return response.json({
            "success": True,
            "content": item['content']
        })
    else:
        return response.json({"success": False})