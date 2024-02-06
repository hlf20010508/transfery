# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from sanic import Blueprint
from sanic.response import json
from modules.utils import rename
from modules.client import storage

upload_bp = Blueprint("upload")

@upload_bp.route('/fetchUploadId', methods=['POST'])
async def fetch_upload_id(request):
    print('received get upload id request')

    content = request.json['content']
    time = request.json['time']

    file_name = rename(content, time)
    upload_id = await storage.create_multipart_upload_id(file_name)
    print('upload id pushed')

    return json({
        "success": True,
        "uploadId": upload_id,
        "fileName": file_name
    })


@upload_bp.route('/uploadPart', methods=['POST'])
async def upload_part(request):
    file_part = request.files.get('filePart').body
    content = request.form.get('content')
    upload_id = request.form.get('uploadId')
    part_number = request.form.get('partNumber')

    etag = await storage.multipart_upload(content, upload_id, file_part, part_number)

    return json({
        "success": True,
        "etag": etag
    })


@upload_bp.route('/completeUpload', methods=['POST'])
async def complete_upload(request):
    print('received complete upload request')

    content = request.json['content']
    upload_id = request.json['uploadId']
    parts = request.json['parts']

    await storage.complete_multipart_upload(content, upload_id, parts)
    print('complete upload finished')
    
    return json({"success": True})