import json
host = input('请输入mysql服务器域名或ip地址：')
user = input('请输入用户名：')
password = input('请输入密码：')
database = input('请输入数据库名：')
config = {
    'host': host,
    'user': user,
    'password': password,
    'database': database,
}
config_file = open('config.json', 'w')
json.dump(config, config_file)
print('设置成功！')
