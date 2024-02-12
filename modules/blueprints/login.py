# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from sanic import Blueprint
from sanic.response import json
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import ec
from base64 import b64encode
from modules.env import USERNAME, PASSWORD, Secret
from modules.utils import check_login

login_bp = Blueprint("login")


@login_bp.route('/auth', methods=['POST'])
async def auth(request):
    print('received auth request')

    username = request.json['username']
    password = request.json['password']
    remember_me = request.json['rememberMe']
    fingerprint = request.json['fingerprint'].encode()

    if username == USERNAME and password == PASSWORD:
        if remember_me:
            signature_bytes = Secret.private_key.sign(
                fingerprint,
                ec.ECDSA(hashes.SHA256())
            )
            signature = b64encode(signature_bytes).decode('utf-8')

            response = json({"success": True})
            response.add_cookie(
                key="signature",
                value=signature,
                # 设置cookie有效期为1年
                max_age=3600 * 24 * 365,
            )

            return response
        else:
            return json({"success": True})
    else:
        return json({"success": False})


@login_bp.route('/login', methods=['GET'])
async def login(request):
    print('received login request')

    return json({"success": check_login(request)})