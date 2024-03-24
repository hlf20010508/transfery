# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from ezmysql import ConnectionAsync
import socketio
from modules.storage import Storage
from modules.env import (
    MINIO_HOST,
    MINIO_SECURE,
    MINIO_USERNAME,
    MINIO_PASSWORD,
    MINIO_BUCKET,
    MYSQL_HOST,
    MYSQL_PORT,
    MYSQL_USERNAME,
    MYSQL_PASSWORD,
    MYSQL_DATABASE,
)

storage = Storage(
    host=MINIO_HOST,
    username=MINIO_USERNAME,
    password=MINIO_PASSWORD,
    bucket=MINIO_BUCKET,
    secure=MINIO_SECURE,
)

database = ConnectionAsync(
    host=MYSQL_HOST,
    database=MYSQL_DATABASE,
    user=MYSQL_USERNAME,
    password=MYSQL_PASSWORD,
    port=MYSQL_PORT,
)

socketio = socketio.AsyncServer(async_mode="sanic", cors_allowed_origins="*")
