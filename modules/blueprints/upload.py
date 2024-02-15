# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from sanic import Blueprint, response
from modules.utils import rename
from modules.client import storage
from modules.sql import update_complete

upload_bp = Blueprint("upload")

@upload_bp.route('/fetchUploadId', methods=['POST'])
async def fetch_upload_id(request):
    print('received get upload id request')

    content = request.json['content']
    timestamp = request.json['timestamp']

    file_name = rename(content, timestamp)
    upload_id = await storage.create_multipart_upload_id(file_name)
    print('upload id pushed')

    return response.json({
        "success": True,
        "uploadId": upload_id,
        "fileName": file_name
    })


@upload_bp.route('/uploadPart', methods=['POST'])
async def upload_part(request):
    file_part = request.files.get('filePart').body
    file_name = request.form.get('fileName')
    upload_id = request.form.get('uploadId')
    part_number = request.form.get('partNumber')

    etag = await storage.multipart_upload(file_name, upload_id, file_part, part_number)

    return response.json({
        "success": True,
        "etag": etag
    })


@upload_bp.route('/completeUpload', methods=['POST'])
async def complete_upload(request):
    print('received complete upload request')

    id = request.json['id']
    file_name = request.json['fileName']
    upload_id = request.json['uploadId']
    parts = request.json['parts']

    await storage.complete_multipart_upload(file_name, upload_id, parts)
    print('complete upload finished')

    await update_complete(id)
    
    return response.json({"success": True})