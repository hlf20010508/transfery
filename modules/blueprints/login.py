# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from sanic import Blueprint
from sanic.response import json
from cryptography.hazmat.primitives.serialization import load_pem_private_key, load_pem_public_key
from cryptography.hazmat.backends import default_backend
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import ec
from base64 import b64encode
from modules.env import USERNAME, PASSWORD
from modules.sql import query_auth_key

login_bp = Blueprint("login")
private_key = None
public_key = None

def load_pem_key(private_key_str, public_key_str):
    pem_private_key = private_key_str.encode('utf-8')
    pem_public_key = public_key_str.encode('utf-8')

    private_key = load_pem_private_key(
        pem_private_key,
        password=None,
        backend=default_backend()
    )

    public_key = load_pem_public_key(
        pem_public_key,
        backend=default_backend()
    )

    return private_key, public_key


@login_bp.listener('before_server_start')
async def setup_bp(app, loop):
    global private_key, public_key
    key = await query_auth_key()
    private_key, public_key = load_pem_key(key['privateKey'], key['publicKey'])


@login_bp.route('/auth', methods=['POST'])
async def auth(request):
    print('received auth request')
    
    username = request.json['username']
    password = request.json['password']
    remember_me = request.json['rememberMe']
    fingerprint = request.json['fingerprint'].encode()

    if username == USERNAME and password == PASSWORD:
        if remember_me:
            signature_bytes = private_key.sign(
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
                # 不能通过脚本获取cookie
                httponly=True,

            )

            return response
        else:
            return json({"success": True})
    else:
        return json({"success": False})