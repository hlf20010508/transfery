# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

import base64
from time import time
import json
from sanic.request.types import Request
from modules.env import Secret


def rename(old_filename, timestamp):
    temp = old_filename.split(".")
    temp[0] += "_" + str(timestamp)[:-3]
    temp = ".".join(temp)
    temp = temp.split()
    temp = "_".join(temp)
    return temp


def getFromPostJson(request, key):
    return None if not key in request.json else request.json[key]


def get_current_timestamp():
    return int(time()) * 1000


def verify_certificate(certificate_str, fingerprint):
    try:
        if not certificate_str:
            return False

        certificate_bytes = base64.b64decode(certificate_str)
        certificate_raw = Secret.key.decrypt(certificate_bytes).decode()
        certificate = json.loads(certificate_raw)

        if (
            certificate["fingerprint"] == fingerprint
            and certificate["timestamp"] > get_current_timestamp()
        ):
            return True
        else:
            return False
    except Exception as e:
        print("certificate error:", e)
        return False


def get_auth_value(authorization_str, key):
    try:
        authorization = json.loads(authorization_str)

        if key in authorization:
            return authorization[key]
        else:
            return ""
    except:
        return ""


def check_login(auth):
    if isinstance(auth, Request):
        authorization = auth.headers.get("Authorization")
    elif isinstance(auth, dict):
        authorization = auth["authorization"]

    if authorization:
        fingerprint = get_auth_value(authorization, "fingerprint")
        certificate = get_auth_value(authorization, "certificate")

        return verify_certificate(certificate, fingerprint)
    else:
        return False
