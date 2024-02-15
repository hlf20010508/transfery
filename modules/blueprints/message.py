# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from sanic import Blueprint, response
from modules.env import ITEM_PER_PAGE
from modules.client import socketio, storage
from modules.utils import getFromPostJson, check_login
import modules.sql as sql

message_bp = Blueprint("message")

@message_bp.route('/page', methods=['GET'])
async def page(request):
    print('received new page request')

    start = int(request.args['size'][0])

    result = await sql.query_items(
        start=start,
        amount=ITEM_PER_PAGE,
        access_private=check_login(request)
    )
    print('new page pushed')

    return response.json({'messages': result})


@message_bp.route('/sync', methods=['GET'])
async def sync(request):
    print('received sync request')

    latest_id = int(request.args['latestId'][0])
    result = await sql.query_items_after(
        id=latest_id,
        access_private=check_login(request)
    )
    print('synced:', result)

    return response.json({'newItems': result})


@message_bp.route("/newItem", methods=["POST"])
async def new_item(request):
    item = {
        "content": request.json['content'],
        "timestamp": request.json['timestamp'],
        "isPrivate": request.json['isPrivate'],
        "type": request.json['type'],
        "fileName": getFromPostJson(request, 'fileName'),
        "isComplete": getFromPostJson(request, 'isComplete'),
    }

    sid = request.json['sid']

    print('received item: ', item)

    item['id'] = await sql.insert(item)
    print('pushed to db')

    await socketio.emit(
        'newItem',
        item,
        room=None if not item['isPrivate'] else "private",
        skip_sid=sid
    )
    print('broadcasted')

    return response.json({
        "success": True,
        "id": item['id']
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

    return response.json({"success": True})


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

    return response.json({"success": True})