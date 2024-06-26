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
- 支持私密消息
- 提供消息发送和接收的API
- 支持Minio和MySQL，也可本地存储

## API
- `/push_text`
    - 功能：发送文本
    - 协议：GET，POST
    - 参数：
        - `content` 文本内容
        - `token` 授权凭证
    - 响应：无
- `/latest_text`
    - 功能：接收最新文本
    - 协议：GET
    - 参数
        - `token` 授权凭证
    - 响应：文本内容

`token`需要登录后在管理员菜单的`授权`处生成。

## 运行环境
运行Transfery，你需要
- <a href="https://github.com/minio/minio">Minio</a>，作为对象存储服务
- MySQL，作为数据库
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
    ports:
      - xxxx:8080
    # 如果使用minio和mysql则可以不设置对应volume
    volumes:
      - /path/to/your/db.sqlite:/db.sqlite
      - /path/to/your/uploaded:/uploaded
    command: ^
      --username xxxx
      --password xxxx
      # --item-per-page 15 # 每次最多向服务器请求的消息数量，默认为15
      # --minio
      # --minio-endpoint https://example.com:9000
      # --minio-username xxxx
      # --minio-password xxxx
      # --minio-bucket xxxx
      # --mysql
      # --mysql-endpoint example.com:3306
      # --mysql-username xxxx
      # --mysql-password xxxx
      # --mysql-database xxxx
```

部署
```sh
sudo docker compose up -d
```