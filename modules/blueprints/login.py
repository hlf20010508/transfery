# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

import json
from sanic import Blueprint, response
from base64 import b64encode
from modules.env import USERNAME, PASSWORD, Secret
from modules.utils import check_login, get_current_timestamp, get_auth_value
import modules.sql as sql

login_bp = Blueprint("login")


@login_bp.route('/auth', methods=['POST'])
async def auth(request):
    print('received auth request')

    username = request.json['username']
    password = request.json['password']
    remember_me = request.json['rememberMe']
    fingerprint = request.json['fingerprint']
    browser = request.json['browser']

    if username == USERNAME and password == PASSWORD:
        max_age = 1000 * 60 * 5 # 5分钟
        if remember_me:
            max_age = 1000 * 3600 * 24 * 365 # 1年

        current_timestamp = get_current_timestamp()
        expiration_timestamp = current_timestamp + max_age
        
        certificate_raw = json.dumps({
            "fingerprint": fingerprint,
            "timestamp": expiration_timestamp
        })
        certificate_bytes = Secret.key.encrypt(certificate_raw.encode())
        certificate = b64encode(certificate_bytes).decode('utf-8')

        await sql.insert_device({
            "fingerprint": fingerprint,
            "browser": browser,
            "lastUseTimestamp": current_timestamp,
            "expirationTimestamp": expiration_timestamp
        })

        return response.json({
            "success": True,
            "certificate": certificate
        })
    else:
        return response.json({"success": False})


@login_bp.route('/login', methods=['GET'])
async def login(request):
    print('received login request')

    if check_login(request):
        authorization = request.headers.get("Authorization")
        fingerprint = get_auth_value(authorization, 'fingerprint')
        await sql.update_device({
            'fingerprint': fingerprint,
            'lastUseTimestamp': get_current_timestamp()
        })

        return response.json({"success": True})
    else:
        return response.json({"success": False})


@login_bp.route('/device', methods=['GET'])
async def device(request):
    if not check_login(request):
        return response.json({"success": False})

    print('received device request')

    device = await sql.query_device()

    return response.json({
        "success": True,
        "device": device
    })


@login_bp.route('/deviceSignOut', methods=['GET'])
async def device_sign_out(request):
    if not check_login(request):
        return response.json({"success": False})

    print('received device sign out request')

    fingerprint = request.args['fingerprint'][0]
    await sql.remove_device(fingerprint)

    return response.json({"success": True})