# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

import pymysql
import asyncio
from cryptography.hazmat.primitives.asymmetric import ec
from cryptography.hazmat.primitives import serialization
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
    MYSQL_TABLE_MESSAGE,
    MYSQL_TABLE_AUTH
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
            isPrivate tinyint not null,
            type varchar(5) not null,
            fileName text,
            isComplete tinyint
        )
    ''' % MYSQL_TABLE_MESSAGE
    cursor.execute(sql)

    sql = '''
        create table if not exists %s(
            id int primary key auto_increment,
            privateKey text not null,
            publicKey text not null
        )
    ''' % MYSQL_TABLE_AUTH
    cursor.execute(sql)

    if not is_key_exist(cursor):
        private_key, public_key = gen_key()
        sql = '''
            insert into %s (privateKey, publicKey)
            select "%s", "%s"
            where not exists (select 1 from auth)
        ''' % (MYSQL_TABLE_AUTH, private_key, public_key)
        cursor.execute(sql)
        conn.commit()

    conn.close()


def is_key_exist(cursor):
    sql = "select count(*) from auth"
    cursor.execute(sql)

    result = cursor.fetchone()[0]
    
    if result > 0:
        return True
    else:
        return False


def gen_key():
    private_key = ec.generate_private_key(ec.SECP256R1()) 
    public_key = private_key.public_key()

    # 序列化ECC私钥为PEM格式
    pem_private_key = private_key.private_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PrivateFormat.PKCS8,
        encryption_algorithm=serialization.NoEncryption()  # 无密码保护
    )
    private_key_str = pem_private_key.decode('utf-8')

    # 序列化ECC公钥为PEM格式
    pem_public_key = public_key.public_bytes(
        encoding=serialization.Encoding.PEM,
        format=serialization.PublicFormat.SubjectPublicKeyInfo
    )
    public_key_str = pem_public_key.decode('utf-8')

    return private_key_str, public_key_str

if __name__ == '__main__':
    init()