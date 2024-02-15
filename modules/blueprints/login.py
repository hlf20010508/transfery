# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

import json
from sanic import Blueprint, response
from base64 import b64encode
from modules.env import USERNAME, PASSWORD, Secret
from modules.utils import check_login, get_current_timestamp

login_bp = Blueprint("login")


@login_bp.route('/auth', methods=['POST'])
async def auth(request):
    print('received auth request')

    username = request.json['username']
    password = request.json['password']
    remember_me = request.json['rememberMe']
    fingerprint = request.json['fingerprint']

    if username == USERNAME and password == PASSWORD:
        max_age = 300 # 5分钟
        if remember_me:
            max_age = 3600 * 24 * 365 # 1年

        certificate_raw = json.dumps({
            "fingerprint": fingerprint,
            "timestamp": get_current_timestamp() + max_age
        })
        certificate_bytes = Secret.key.encrypt(certificate_raw.encode())
        certificate = b64encode(certificate_bytes).decode('utf-8')

        return response.json({
            "success": True,
            "certificate": certificate
        })
    else:
        return response.json({"success": False})


@login_bp.route('/login', methods=['GET'])
async def login(request):
    print('received login request')

    return response.json({"success": check_login(request)})