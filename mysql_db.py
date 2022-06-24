from ezmysql import ConnectionSync
import config as myconfig


def db():
    config=myconfig.load()
    host = config['host_mysql']
    username = config['username_mysql']
    password = config['password_mysql']
    database = config['database']
    table = config['table']

    # create connection
    return ConnectionSync(
        host,
        database,
        username,
        password,
    ), table
