# transfery
便捷的临时消息文件传输服务

前端 [transfery-vue](https://github.com/hlf20010508/transfery-vue)
## 功能
- 发送文字消息
- 传输文件，支持多文件，使用分片上传，可暂停
- 全双工即时通信，所有客户端同步消息和上传进度
- 异步框架，支持边上传边发送消息
- 适配移动端键盘
- 删除消息和文件

- `/method/push_text`发送文本，支持GET和POST，参数为content
- `/method/latest_text`接收最新文本，支持GET

## 展示
<p><img src="https://github.com/hlf20010508/transfery-vue/assets/76218469/f809d6d7-9fc4-4329-bc20-71480918dba8" width="400"></p>
<p><img src="https://github.com/hlf20010508/transfery-vue/assets/76218469/e77a9ca7-1cc2-4939-9a47-343a6e448a0b" width="400"></p>
<p><img src="https://github.com/hlf20010508/transfery-vue/assets/76218469/40fe27ef-78fb-4c53-b4e6-69e32690f432" width="400"></p>

## 运行环境
运行Transfery，你需要
- <a href="https://github.com/minio/minio">Minio</a>，作为对象存储服务
- MySQL，作为数据库
- Sanic，作为后端服务
- 一台服务器，以便随时随地使用

## 通过Docker部署
Docker compose 配置
```yml
version: '3.8'
services:
  transfery:
    container_name: transfery
    image: hlf01/transfery
    restart: always
    environment:
      - PORT=xxxx
      - MINIO_HOST=https://example.com:9000
      - MINIO_USERNAME=xxxx
      - MINIO_PASSWORD=xxxx
      - MINIO_BUCKET=xxxx
      - MYSQL_HOST=example.com:3306
      - MYSQL_USERNAME=xxxx
      - MYSQL_PASSWORD=xxxx
      - MYSQL_DATABASE=xxxx
    network_mode: host
```

部署
```sh
sudo docker compose up -d
```