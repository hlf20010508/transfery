# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

import pymysql
import asyncio
from miniopy_async import Minio
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
    MYSQL_TABLE_MESSAGE
)


def init():
    print('Initializing minio...')
    init_minio()
    print('Minio initialized.')

    print('Initializing mysql...')
    init_mysql()
    print('Mysql initialized.')
    
    print('All initialization completed.')


def init_minio():
    # create bucket if not exists
    client = Minio(
        endpoint=MINIO_HOST,
        access_key=MINIO_USERNAME,
        secret_key=MINIO_PASSWORD,
        secure=MINIO_SECURE
    )
    async def main():
        if not await client.bucket_exists(MINIO_BUCKET):
            await client.make_bucket(MINIO_BUCKET)
    asyncio.run(main())


def init_mysql():
    # create database and table if not exists
    conn = pymysql.connect(
        host=MYSQL_HOST,
        user=MYSQL_USERNAME,
        password=MYSQL_PASSWORD,
        port=MYSQL_PORT,
        charset='utf8mb4'
    )

    cursor = conn.cursor()
    sql = "create database if not exists %s" % MYSQL_DATABASE
    cursor.execute(sql)

    sql = "use %s" % MYSQL_DATABASE
    cursor.execute(sql)

    sql = '''
        create table if not exists %s(
            id int primary key auto_increment,
            content text not null,
            timestamp bigint not null,
            type varchar(5) not null,
            fileName text,
            isComplete tinyint
        )
    ''' % MYSQL_TABLE_MESSAGE
    cursor.execute(sql)

    conn.close()


if __name__ == '__main__':
    init()