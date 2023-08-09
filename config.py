# :project: transfery
# :author: L-ING
# :copyright: (C) 2022 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

import os
import json
import pymysql
import asyncio
from miniopy_async import Minio

def init():
    host_minio = input('Host name or ip address of minio server (eg: https://example.com:9000): ')
    username_minio = input('Username: ')
    password_minio = input('Password: ')
    bucket = input('Bucket name: ')
    host_mysql = input(
        'Host name or ip address of mysql server (eg: example.com:3306) : ')
    username_mysql = input('Username: ')
    password_mysql = input('Password: ')
    database = input('Database name: ')
    table = input('Table name: ')

    config = {
        'cache_path': 'cache',
        'item_per_page': 15,
        'host_minio': host_minio,
        'username_minio': username_minio,
        'password_minio': password_minio,
        'bucket': bucket,
        'host_mysql': host_mysql,
        'username_mysql': username_mysql,
        'password_mysql': password_mysql,
        'database': database,
        'table': table,
    }

    config_file = open('config.json', 'w')
    json.dump(config, config_file)
    config_file.close()
    print('Configuration completed')

    print('Initializing minio...')
    init_minio(host_minio, username_minio, password_minio, bucket)
    print('Minio initialized')

    print('Initializing mysql...')
    port = int(host_mysql.split(':')[1])
    host = host_mysql.split(':')[0]
    init_mysql(host, username_mysql,
               password_mysql, port, database, table)
    print('Mysql initialized')
    
    print('All initialization completed')


def load():
    try:
        config_file = open('config.json', 'r')
        config = json.load(config_file)
        config_file.close()
        return config
    except:
        try:
            config = {
                'host_mysql': os.environ['host_mysql'],
                'username_mysql': os.environ['username_mysql'],
                'password_mysql': os.environ['password_mysql'],
                'database': os.environ['database'],
                'table': os.environ['table'],
                'host_minio': os.environ['host_minio'],
                'username_minio': os.environ['username_minio'],
                'password_minio': os.environ['password_minio'],
                'bucket': os.environ['bucket'],
                'cache_path': os.environ.get('cache_path', 'cache'),
                'item_per_page': int(os.environ.get('item_per_page', 15)),
            }
            return config
        except:
            print('Configuration not found, run config.py first')
            print('python config.py')
            exit()


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
    loop = asyncio.get_event_loop()
    loop.run_until_complete(main())
    loop.close()


def init_mysql(host, user, password, port, database, table):
    # create database and table if not exists
    conn = pymysql.connect(host=host, user=user,
                           password=password, port=port, charset='utf8mb4')
    cursor = conn.cursor()
    sql = "create database if not exists %s" % database
    cursor.execute(sql)

    sql = "use %s" % database
    cursor.execute(sql)

    sql = '''create table if not exists %s(
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
