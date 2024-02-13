# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

import base64
from time import time
from modules.env import Secret


def rename(old_filename, timestamp):
    temp = old_filename.split('.')
    temp[0] += '_'+str(timestamp)[:-3]
    temp = '.'.join(temp)
    temp = temp.split()
    temp = '_'.join(temp)
    return temp


def getFromPostJson(request, key):
    return None if not key in request.json else request.json[key]


def get_current_timestamp():
    return int(time())


def verify_certification(certification_str, fingerprint):
    certification = base64.b64decode(certification_str)

    try:
        certification_raw = Secret.key.decrypt(certification).decode()
        cert_fingerprint, cert_timestamp = certification_raw.split(', ')

        if cert_fingerprint == fingerprint and int(cert_timestamp) > get_current_timestamp():
            return True
        else:
            return False
    except Exception as e:
        print('certification error:', e)
        return False


def authorization_to_dict(authorization):
    result = {}
    if authorization:
        auth_str_list = authorization.split(', ')
        for auth_str in auth_str_list:
            auth = auth_str.split(': ')
            if len(auth) == 2:
                result[auth[0]] = auth[1]
    return result


def get_fingerprint(authorization):
    authorization = authorization_to_dict(authorization)
    if 'fingerprint' in authorization:
        return authorization['fingerprint']
    else:
        return ''


def check_login(request):
    authorization = request.headers.get("Authorization")
    fingerprint = ''
    if authorization:
        fingerprint = get_fingerprint(authorization)

    is_login = False
    if (certification := request.cookies.get('certification')) and fingerprint:
        is_login = verify_certification(certification, fingerprint)

    return is_login