# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from modules.client import database
from modules.env import MYSQL_TABLE_MESSAGE, MYSQL_TABLE_AUTH, MYSQL_TABLE_DEVICE
from modules.utils import get_current_timestamp

async def query_auth_key():
    return (await database.query('select secretKey from %s' % MYSQL_TABLE_AUTH))[0]['secretKey']


async def query_items(start, amount, access_private=False):
    sql = 'select * from %s ' % MYSQL_TABLE_MESSAGE

    if not access_private:
        sql += 'where isPrivate = false '

    sql += 'order by timestamp desc, id desc limit %d, %d' % (
        start,
        amount,
    )

    return await database.query(sql)


async def query_items_after(id, access_private=False):
    sql = 'select * from %s where id > %s' % (
        MYSQL_TABLE_MESSAGE,
        id
    )

    if not access_private:
        sql += ' and isPrivate = false'

    return await database.query(sql)


async def query_device():
    sql = 'select * from %s' % MYSQL_TABLE_DEVICE

    return await database.query(sql)


async def insert_message(item):
    # id
    return await database.table_insert(MYSQL_TABLE_MESSAGE, item)


async def insert_device(item):
    await database.table_insert(MYSQL_TABLE_DEVICE, item)


async def update_complete(id):
    await database.table_update(MYSQL_TABLE_MESSAGE, {"isComplete": True}, "id", id)


async def update_last_use_timestamp(fingerprint):
    await database.table_update(MYSQL_TABLE_DEVICE, {"lastUseTimestamp": get_current_timestamp()}, "fingerprint", fingerprint)


async def remove_item(id):
    await database.query(
        'delete from %s where id="%d"' % (
            MYSQL_TABLE_MESSAGE,
            id
        )
    )


async def remove_all_items():
    await database.query('delete from %s' % MYSQL_TABLE_MESSAGE)
