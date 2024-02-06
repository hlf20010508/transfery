# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from sanic import Blueprint
from sanic.response import json
from modules.client import storage

download_bp = Blueprint("download")

@download_bp.route('/downloadUrl', methods=['GET'])
async def download_url(request):
    print('received download request')

    file_name = request.args['fileName'][0]
    url = await storage.get_download_url(file_name)
    print('url pushed')
    
    return json({"success": True, "url": url})