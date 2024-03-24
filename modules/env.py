# :project: transfery
# :author: L-ING
# :copyright: (C) 2022 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

import os

AUTO_RELOAD = True if os.environ.get("AUTO_RELOAD", "false") == "true" else False
PORT = int(os.environ.get("PORT", 8080))
ITEM_PER_PAGE = int(os.environ.get("ITEM_PER_PAGE", 15))
USERNAME = os.environ["USERNAME"]
PASSWORD = os.environ["PASSWORD"]
MINIO_PROTOCOL, MINIO_HOST = os.environ["MINIO_HOST"].split("://")
MINIO_SECURE = True if MINIO_PROTOCOL == "https" else False
MINIO_USERNAME = os.environ["MINIO_USERNAME"]
MINIO_PASSWORD = os.environ["MINIO_PASSWORD"]
MINIO_BUCKET = os.environ["MINIO_BUCKET"]
MYSQL_HOST, MYSQL_PORT = os.environ["MYSQL_HOST"].split(":")
MYSQL_PORT = int(MYSQL_PORT)
MYSQL_USERNAME = os.environ["MYSQL_USERNAME"]
MYSQL_PASSWORD = os.environ["MYSQL_PASSWORD"]
MYSQL_DATABASE = os.environ["MYSQL_DATABASE"]
MYSQL_TABLE_MESSAGE = "message"
MYSQL_TABLE_AUTH = "auth"
MYSQL_TABLE_DEVICE = "device"


class Secret:
    key = None
