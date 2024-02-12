# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

import base64
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.hazmat.primitives.serialization import load_pem_private_key, load_pem_public_key
from cryptography.hazmat.backends import default_backend
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


def verify_signature(signature_str, fingerprint):
    signature = base64.b64decode(signature_str)
    fingerprint = fingerprint.encode()

    try:
        Secret.public_key.verify(
            signature,
            fingerprint,
            ec.ECDSA(hashes.SHA256())
        )
        return True
    except Exception as e:
        print('signature error:', e)
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
    if (signature := request.cookies.get('signature')) and fingerprint:
        is_login = verify_signature(signature, fingerprint)
    
    return is_login