# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from ezmysql import ConnectionAsync
import socketio
from modules.storage import Storage
from modules.env import (
    MINIO_HOST,
    MINIO_USERNAME,
    MINIO_PASSWORD,
    MINIO_BUCKET,
    MYSQL_HOST,
    MYSQL_USERNAME,
    MYSQL_PASSWORD,
    MYSQL_DATABASE
)

minio_protocol, minio_host = MINIO_HOST.split('://')
minio_username = MINIO_USERNAME
minio_password = MINIO_PASSWORD
minio_bucket = MINIO_BUCKET
minio_secure = True if minio_protocol == 'https' else False

mysql_host, mysql_port = MYSQL_HOST.split(':')
mysql_port = int(mysql_port)

storage = Storage(
    host=minio_host,
    username=minio_username,
    password=minio_password,
    bucket=minio_bucket,
    secure=minio_secure
)

database = ConnectionAsync(
    host=mysql_host,
    database=MYSQL_DATABASE,
    user=MYSQL_USERNAME,
    password=MYSQL_PASSWORD,
    port=mysql_port
)

socketio = socketio.AsyncServer(async_mode='sanic', cors_allowed_origins="*")