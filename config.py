import json


def init():
    host_minio = input('请输入minio服务器完整域名或ip地址 (eg: http://example.com:9000)：')
    username_minio = input('请输入用户名：')
    password_minio = input('请输入密码：')
    bucket = input('请输入bucket名：')
    host_mysql = input(
        '请输入mysql服务器域名或ip地址，必须使用默认端口 (eg: example.com) ：')
    username_mysql = input('请输入用户名：')
    password_mysql = input('请输入密码：')
    database = input('请输入数据库名：')
    table = input('请输入表名：')
    item_per_page = input('请输入每次加载的项目条数：')

    config = {
        'host_minio': host_minio,
        'username_minio': username_minio,
        'password_minio': password_minio,
        'bucket': bucket,
        'host_mysql': host_mysql,
        'username_mysql': username_mysql,
        'password_mysql': password_mysql,
        'database': database,
        'table': table,
        'item_per_page': int(item_per_page)
    }
    config_file = open('config.json', 'w')
    json.dump(config, config_file)
    print('设置成功！')


def load():
    try:
        config_file = open('config.json', 'r')
    except:
        print('未找到数据库配置文件，请先运行config.py')
        print('python config.py')
        exit()
    config = json.load(config_file)
    config_file.close()
    return config


if __name__ == '__main__':
    init()
