import json
import pymysql
from minio_async import Minio

def init():
    host_minio = input('请输入minio服务器域名或ip地址 (eg: example.com:9000) ：')
    secure_minio = True if input(
        '请选择minio服务器使用的协议： 0 http 1 https ')=='1' else False
    local_minio = True if input(
        'minio是否与transfery在同一个服务器？ 0 否 1 是 ') == '1' else False
    username_minio = input('请输入用户名：')
    password_minio = input('请输入密码：')
    bucket = input('请输入bucket名：')
    host_mysql = input(
        '请输入mysql服务器域名或ip地址 (eg: example.com:3306) ：')
    local_mysql = True if input(
        'mysql是否与transfery在同一个服务器？ 0 否 1 是 ') == '1' else False
    username_mysql = input('请输入用户名：')
    password_mysql = input('请输入密码：')
    database = input('请输入数据库名：')
    table = input('请输入表名：')
    item_per_page = input('请输入每次加载的项目条数 (eg: 15) ：')

    config = {
        'cache_path': 'cache',
        'host_minio': host_minio,
        'secure_minio': secure_minio,
        'local_minio': local_minio,
        'username_minio': username_minio,
        'password_minio': password_minio,
        'bucket': bucket,
        'host_mysql': host_mysql,
        'local_mysql': local_mysql,
        'username_mysql': username_mysql,
        'password_mysql': password_mysql,
        'database': database,
        'table': table,
        'item_per_page': int(item_per_page)
    }

    config_file = open('config.json', 'w')
    json.dump(config, config_file)
    config_file.close()
    print('设置成功！')

    print('正在初始化minio...')
    init_minio(host_minio, username_minio, password_minio, bucket)

    print('正在初始化mysql...')
    port = int(host_mysql.split(':')[1])
    host = '127.0.0.1' if local_mysql else host_mysql.split(':')[0]
    init_mysql(host, username_mysql,
               password_mysql, port, database, table)

    print('初始化完成')


def load():
    try:
        config_file = open('config.json', 'r')
    except:
        print('未找到\配置文件，请先运行config.py')
        print('python config.py')
        exit()
    config = json.load(config_file)
    config_file.close()
    return config


def init_minio(host, username, password, bucket):
    client = Minio(
        host,
        access_key=username,
        secret_key=password,
        secure=False
    )
    if not client.bucket_exists(bucket):
        client.make_bucket(bucket)


def init_mysql(host, user, password, port, database, table):
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
