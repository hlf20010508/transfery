# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from sanic import Blueprint
from sanic.response import json
from modules.env import ITEM_PER_PAGE
from modules.client import socketio, storage
from modules.utils import getFromPostJson
import modules.sql as sql

message_bp = Blueprint("message")

@message_bp.route('/page', methods=['GET'])
async def page(request):
    print('received new page request')

    start = int(request.args['size'][0])
    result = await sql.query_items(start, ITEM_PER_PAGE)
    print('new page pushed')

    return json({'messages': result})


@message_bp.route('/sync', methods=['GET'])
async def sync(request):
    print('received sync request')

    last_id = int(request.args['lastId'][0])
    result = await sql.query_items_after(last_id)
    print('synced:', result)

    return json({'newItems': result})


@message_bp.route("/newItem", methods=["POST"])
async def new_item(request):
    item = {
        "content": request.json['content'],
        "type": request.json['type'],
        "time": request.json['time'],
        "fileName": getFromPostJson(request, 'fileName'),
    }

    sid = request.json['sid']

    print('received item: ', item)

    id = await sql.insert_item(item)
    print('pushed to db')
    item['id'] = id

    await socketio.emit('newItem', item, skip_sid=sid)
    print('broadcasted')

    return json({
        "success": True,
        "id": id
    })


@message_bp.route("/removeItem", methods=["POST"])
async def remove_item(request):
    item = {
        'id': request.json['id'],
        'type': request.json['type'],
        'fileName': getFromPostJson(request, 'fileName')
    }

    sid = request.json['sid']

    print('received item to be removed: ', item)

    await sql.remove_item(item["id"])
    print('removed item in db')

    if item['type'] == 'file':
        await storage.remove(item['fileName'])
        print('removed item in storage')

    await socketio.emit('removeItem', item['id'], skip_sid=sid)
    print('broadcasted')

    return json({"success": True})


@message_bp.route("/removeAll", methods=["GET"])
async def remove_all(request):
    print('received remove all item request')
    
    sid = request.args['sid'][0]

    await sql.remove_all_items()
    print('removed all items in db')

    await storage.remove_all()
    print('all items removed in storage')

    await socketio.emit('removeAll', skip_sid=sid)
    print('broadcasted')

    return json({"success": True})