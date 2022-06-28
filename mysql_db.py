from ezmysql import ConnectionSync
import config as myconfig


def db():
    config=myconfig.load()
    port=int(config['host_mysql'].split(':')[1])
    host = '127.0.0.1' if config['local_mysql'] else config['host_mysql'].split(':')[0]
    username = config['username_mysql']
    password = config['password_mysql']
    database = config['database']

    # create connection
    return ConnectionSync(
        host,
        database,
        username,
        password,
        port
    )
