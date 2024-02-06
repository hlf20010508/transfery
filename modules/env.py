# :project: transfery
# :author: L-ING
# :copyright: (C) 2022 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

import os
import pymysql
import asyncio
from miniopy_async import Minio

PORT = int(os.environ.get("PORT", 8080))
ITEM_PER_PAGE =  int(os.environ.get('ITEM_PER_PAGE', 15))
MINIO_HOST =  os.environ['MINIO_HOST']
MINIO_USERNAME =  os.environ['MINIO_USERNAME']
MINIO_PASSWORD =  os.environ['MINIO_PASSWORD']
MINIO_BUCKET =  os.environ['MINIO_BUCKET']
MYSQL_HOST =  os.environ['MYSQL_HOST']
MYSQL_USERNAME =  os.environ['MYSQL_USERNAME']
MYSQL_PASSWORD =  os.environ['MYSQL_PASSWORD']
MYSQL_DATABASE =  os.environ['MYSQL_DATABASE']
MYSQL_TABLE =  os.environ['MYSQL_TABLE']


def init():
    print('Initializing minio...')
    init_minio(
        host=MINIO_HOST,
        username=MINIO_USERNAME,
        password=MINIO_PASSWORD,
        bucket=MINIO_BUCKET
    )
    print('Minio initialized.')

    print('Initializing mysql...')
    host, port = MYSQL_HOST.split(':')
    port = int(port)
    init_mysql(
        host=host,
        port=port,
        username=MYSQL_USERNAME,
        password=MYSQL_PASSWORD,
        database=MYSQL_DATABASE,
        table=MYSQL_TABLE
    )
    print('Mysql initialized.')
    
    print('All initialization completed.')


def init_minio(host, username, password, bucket):
    # create bucket if not exists
    client = Minio(
        host,
        access_key=username,
        secret_key=password,
        secure=False
    )
    async def main():
        if not await client.bucket_exists(bucket):
            await client.make_bucket(bucket)
    asyncio.run(main())


def init_mysql(host, port, username, password, database, table):
    # create database and table if not exists
    conn = pymysql.connect(
        host=host,
        user=username,
        password=password,
        port=port,
        charset='utf8mb4'
    )

    cursor = conn.cursor()
    sql = "create database if not exists %s" % database
    cursor.execute(sql)

    sql = "use %s" % database
    cursor.execute(sql)

    sql = '''
        create table if not exists %s(
            id int primary key auto_increment,
            content text not null,
            fileName text,
            type varchar(5) not null,
            showTime int not null,
            time bigint not null
        )
    ''' % table
    cursor.execute(sql)

    conn.close()


if __name__ == '__main__':
    init()
