import json
from ezmysql import ConnectionSync

def db():
    try:
        config_file = open('config.json', 'r')
    except:
        print('未找到数据库配置文件，请先运行config.py')
        print('python config.py')
        exit()
    config = json.load(config_file)
    config_file.close()
    host = config['host']
    database = config['database']
    user = config['user']
    password = config['password']

    # create connection
    return ConnectionSync(
        host,
        database,
        user,
        password,
    )
