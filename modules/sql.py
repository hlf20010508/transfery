# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from modules.client import database
from modules.env import MYSQL_TABLE_MESSAGE

async def query_items(start, amount):
    return await database.query(
        'select * from %s order by timestamp desc, id desc LIMIT %d, %d' % (
            MYSQL_TABLE_MESSAGE,
            start,
            amount
        )
    )


async def query_items_after(id):
    return await database.query(
        'select * from %s where id > %s' % (
            MYSQL_TABLE_MESSAGE,
            id
        )
    )


async def insert(item):
    # id
    return await database.table_insert(MYSQL_TABLE_MESSAGE, item)


async def update_complete(id):
    await database.table_update(MYSQL_TABLE_MESSAGE, {"isComplete": True}, "id", id)


async def remove_item(id):
    await database.query(
        'delete from %s where id="%d"' % (
            MYSQL_TABLE_MESSAGE,
            id
        )
    )


async def remove_all_items():
    await database.query('delete from %s' % MYSQL_TABLE_MESSAGE)
