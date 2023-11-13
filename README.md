# transfery

> 便捷的临时消息文件传输项目

## 目录
- [项目部分界面展示](#interface)
- [项目功能](#functionality)
- [项目所需环境](#environment)
- [注意](#note)
- [项目运行](#running)

<span id="interface"></span>

## 项目部分界面展示
浏览器样式  
<img width="1082" alt="image" src="https://user-images.githubusercontent.com/76218469/176151027-c40cc300-7c22-42c5-9da8-2984066a8b99.png">  
<img width="1082" alt="image" src="https://user-images.githubusercontent.com/76218469/176161205-42b4f732-569d-4cd4-876e-6d0ae31c5f84.png">  
<img width="1082" alt="image" src="https://user-images.githubusercontent.com/76218469/176152167-abc40d1f-26d6-4b19-8438-519ff1c774a1.png">

ios WebApp（增加到主屏幕）  
<div align=center>
<img width="300" alt="image" src="https://user-images.githubusercontent.com/76218469/176231809-af30a998-f494-479e-8355-2a4c5b5f18dd.PNG"> <img width="300" alt="image" src="https://user-images.githubusercontent.com/76218469/176233010-944534ff-9db5-4935-9e71-44a28af17b28.PNG"> <img width="300" alt="image" src="https://user-images.githubusercontent.com/76218469/176233015-457633fd-bb43-4a7f-b021-4a3e85ef02b2.PNG"> <img width="300" alt="image" src="https://user-images.githubusercontent.com/76218469/176664354-69f6b382-44ef-4592-aa34-57333a22240f.PNG">
</div>

<span id="functionality"></span>

## 项目功能
- 发送文字消息
- 传输文件，支持多文件，支持分片上传
- 全双工即时通信
- 异步框架，支持边上传边发送消息
- 手机使用屏幕键盘时页面弹性缩放
- 删除历史记录
- 支持通过GET或POST进行文本发送与接收等请求

使用`/method/push_text`发送文本，支持GET和POST，参数为content  
使用`/method/latest_text`接收最新文本，支持GET  
使用`/method/remove_latest_text`删除最新文本，支持GET  
使用`/method/remove_all`删除所有记录，支持GET

<span id="environment"></span>

## 项目所需环境
运行Transfery，你需要
- <a href="https://github.com/minio/minio.git">Minio</a>，作为对象存储服务
- MySQL，作为数据库
- Sanic，作为后端服务
- 一台服务器，以便随时随地使用

<span id="dependencies"></span>

### 依赖
- [Pipfile](https://github.com/hlf20010508/transfery/blob/master/Pipfile)
- [Pipfile.lock](https://github.com/hlf20010508/transfery/blob/master/Pipfile.lock)

<span id="note"></span>

## 注意
- 由于没有设置密码，因此请不要将Transfery的服务网址分享到网络上，以免被恶意上传。
- 仅需安装好Minio和Mysql并能正常连接即可,config.py会自动在Minio中创建bucket以及在MySQL中创建数据库和表。
- 本项目仅为后端，如需自定义前端界面，请前往<a href="https://github.com/hlf20010508/transfery-vue.git">transfery-vue</a>。
- 若将transfery与transfery-vue放在同级目录下，在transfery-vue中使用“npm run build”会自动将webpack打包好的html和js文件导入transfery。

<span id="running"></span>

## 通过Docker部署
### 若未初始化过Mysql和MinIO，需要先初始化
先前往 [直接运行](#direct_launch) 参考初始化命令来进行初始化

### 若已初始化过MySql和MinIO，调试时可以直接使用.env文件导入环境
创建配置文件
```sh
vim .env
```

输入配置（例子）
```sh
# 缓存存储路径 (默认值: cache)
cache_path=cache
# 每页项目个数 (默认值: 15)
item_per_page=15
# minio服务器地址和端口号
host_minio=https://123.123.123.123:9000
# minio用户名
username_minio=user
# minio用户密码
password_minio=12345678
# minio bucket名
bucket=transfer
# mysql服务器地址和端口号
host_mysql=123.123.123.123:3306
# mysql用户名
username_mysql=root
# mysql用户密码
password_mysql=12345678
# mysql 数据库名
database=transfery
# mysql 表名
table=main
```

安装
```sh
# 安装docker-compose
pip install docker-compose
# 部署
docker-compose up -d
```

## Docker构建
```sh
docker-compose -f docker-compose-build.yml up
```

<span id="direct_launch"></span>
## 直接运行
``` bash
# 安装pipenv
pip install pipenv

# 使用pipenv安装依赖
pipenv sync

# 初始化配置
pipenv run python config.py

# 运行服务
pipenv run python sanic run.app

# 自定义host和port，运行在服务器上必须使用0.0.0.0，否则无法访问
pipenv run python sanic run.app -H 0.0.0.0 -p 5020
```

<span id="background&boot"></span>

### 后台运行与开机自启
```bash
# 编辑transfery@.service
vim transfery@.service

# 参照已有命令更改第6行ExecStart和第11行WorkingDirectory

# 将transfery@.service复制到/etc/systemd/system
sudo cp transfery@.service /etc/systemd/system

# 启动服务 USERNAME是本机用户名，下同
sudo systemctl start transfery@USERNAME

# 查看状态
sudo systemctl status transfery@USERNAME

# 开机自启
sudo systemctl enable transfery@USERNAME

# 重启服务
sudo systemctl restart transfery@USERNAME

# 关闭服务
sudo systemctl stop transfery@USERNAME

# 取消开机自启
sudo systemctl disable transfery@USERNAME
```
