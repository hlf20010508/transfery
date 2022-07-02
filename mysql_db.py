import pymysql
from ezmysql import ConnectionSync
import config as myconfig

config = myconfig.load()
port = int(config['host_mysql'].split(':')[1])
host = '127.0.0.1' if config['local_mysql'] else config['host_mysql'].split(':')[
    0]
username = config['username_mysql']
password = config['password_mysql']
database = config['database']


def db():
    # create connection
    return ConnectionSync(
        host,
        database,
        username,
        password,
        port
    )


def init():
    conn = pymysql.connect(host=host, user=username,
                           password=password, port=port, charset='utf8mb4')
    cursor = conn.cursor()

    sql = "create datebase if not exists %s" % database
    cursor.execute(sql)

    sql = '''create table if not exists %s(
    id int primary key auto_increment,
    content text not null,
    fileName text,
    type varchar(5) not null,
    showTime int not null,
    time bigint not null
    )
    '''
    cursor.execute(sql)

    conn.close()
