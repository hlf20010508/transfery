# transfery
> 便捷的临时消息文件传输项目

<br/>

## 目录
- <a href="#h1">项目部分界面展示</a>
- <a href="#h2">项目功能</a>
- <a href="#h3">项目意义</a>
- <a href="#h4">项目所需环境</a>
  - <a href="#sh41">项目所需环境</a>
- <a href="#h5">项目运行</a>
  - <a href="#sh51">后台运行与开机自启</a>

<br/>

## 项目部分界面展示<span id="h1"></span>
<img width="1082" alt="image" src="https://user-images.githubusercontent.com/76218469/176151027-c40cc300-7c22-42c5-9da8-2984066a8b99.png">
<img width="1082" alt="image" src="https://user-images.githubusercontent.com/76218469/176161205-42b4f732-569d-4cd4-876e-6d0ae31c5f84.png">
<img width="1082" alt="image" src="https://user-images.githubusercontent.com/76218469/176152167-abc40d1f-26d6-4b19-8438-519ff1c774a1.png">

<br/>

## 项目功能<span id="h2"></span>

- 发送文字消息

- 传输文件，支持多文件

- 删除历史记录

<br/>

## 项目意义<span id="h3"></span>

假设一个场景：

你正在上课，这堂课需要你上台汇报。

然而你虽然带了电脑，但忘记带上U盘，没有办法将PPT拷贝到教室的电脑里。

教室的电脑比较新，没有安装通讯软件，你又不便直接在上面安装。

此时比较靠谱的办法可能是将PPT通过电子邮件发送给自己，然后登录自己的邮箱进行下载。

或者将PPT发送到自己的网盘，然后登录网盘的网页版进行下载。

然而这些都需要进行登录操作，可能会让台下的人长时间观摩你的操作。

<br/>

再假设一个场景：

你拥有三台电脑，一台是Windows电脑，一台是Mac电脑，一台是Linux电脑。

你需要在这三台电脑之间发送大量的小文件，同时还要时不时传递文字信息。

如果是在手机和电脑上，你应该会直接使用通讯软件，这的确很方便。

然而现在是三台电脑，三种操作系统，可能有一个操作系统无法装你想要的通讯软件，也有可能通讯软件不能同时登录三台电脑

<br/>

生活中难免会遇到需要操作别人的或者公共的电脑时候，亦或者自己有很多不同平台的设备，此时传送小文件、发送文字是挺麻烦的一件事。

<br/>

Transfery的意义，就是传送小型的临时文件，共享剪贴板，而无需登录，无设备数量限制。

<br/>

## 项目所需环境<span id="h4"></span>

运行Transfery，你需要
- <a href="https://github.com/minio/minio.git">Minio</a>，作为对象存储服务
- MySQL，作为数据库
- Flask，作为后端服务
- 一台服务器，以便随时随地使用

<br/>

仅需安装好并能正常连接Minio和Mysql即可
config.py会自动在Minio中创建bucket并在MySQL中创建数据库和表

<br/>

### 注意<span id="sh41"></span>

由于没有设置密码，因此请不要将Transfery的服务网址分享到网络上，以免被恶意上传。

<br/>

本项目仅为后端，如需自定义前端界面，请前往<a href="https://github.com/hlf20010508/transfery-vue.git">transfery-vue</a>。

将transfery与transfery-vue放在同级目录下，进入transfery-vue项目文件夹，运行：
```bash
npm run build
```

会自动将webpack打包好的html和js文件导入transfery。

<br/>

## 项目运行<span id="h5"></span>

``` bash
# 安装pipenv
pip install pipenv

# 使用pipenv安装依赖
pipenv sync

# 运行配置脚本
pipenv run python config.py

# 运行服务
pipenv run python flask run

# 自定义host和port，运行在服务器上必须使用0.0.0.0，否则无法访问
pipenv run python flask run -h 0.0.0.0 -p 5000
```

<br/>

### 后台运行与开机自启<span id="sh51"></span>

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
