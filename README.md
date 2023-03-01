# transfery

> A Convenient Temporary Message and File transfer Project

## Languages
- <a href="https://github.com/hlf20010508/transfery/blob/master/README.md">English</a>
- <a href="https://github.com/hlf20010508/transfery/blob/master/README/README.zh_CN.md">简体中文</a>

## Catalogue
- [Interface](#interface)
- [Functionality](#functionality)
- [Significance](#significance)
- [Environment](#environment)
- [Note](#note)
- [Running](#running)

<span id="interface"></span>

## Interface
Web Browser  
<img width="1082" alt="image" src="https://user-images.githubusercontent.com/76218469/176151027-c40cc300-7c22-42c5-9da8-2984066a8b99.png">  
<img width="1082" alt="image" src="https://user-images.githubusercontent.com/76218469/176161205-42b4f732-569d-4cd4-876e-6d0ae31c5f84.png">  
<img width="1082" alt="image" src="https://user-images.githubusercontent.com/76218469/176152167-abc40d1f-26d6-4b19-8438-519ff1c774a1.png">

ios WebApp (Add to Home Screen)  
<div align=center>
<img width="300" alt="image" src="https://user-images.githubusercontent.com/76218469/176231809-af30a998-f494-479e-8355-2a4c5b5f18dd.PNG"> <img width="300" alt="image" src="https://user-images.githubusercontent.com/76218469/176233010-944534ff-9db5-4935-9e71-44a28af17b28.PNG"> <img width="300" alt="image" src="https://user-images.githubusercontent.com/76218469/176233015-457633fd-bb43-4a7f-b021-4a3e85ef02b2.PNG"> <img width="300" alt="image" src="https://user-images.githubusercontent.com/76218469/176664354-69f6b382-44ef-4592-aa34-57333a22240f.PNG">
</div>

<span id="functionality"></span>

## Functionality
- Send messages
- Send files, multiple files uploads and multipart uploads supported
- Full-duplex instant messaging
- Asynchronous framework, messaging while uploading supported
- Scales page elastically when using soft keyboard on phone
- Delete history messages

<span id="significance"></span>

## Significance
Consider a scenario:  
You're in class, and this class requires you to report on stage.  
However, although you have brought a computer, but forget to bring a USB flash drive, there is no way to copy the PPT to the classroom computer.  
The computer in the classroom is relatively new, and there is no communication software installed, and it's not inconvenient for you to install it directly.  
At this time, the reliable way maybe is to send the PPT to yourself by e-mail and then log in to your mailbox to download it.  
Or send the PPT to your network disk, and then log in to the website of the network disk to download it.  
However, these require login, which may let people observe your operations for a long time.

Let's take another scenario:  
You have three computers, one is a Windows, one is a Mac, and one is a Linux.  
You'll have to send a lot of small files and text messages between these three computers.  
If it's between a phone and a computer, using communication software is a good solution.  
However, now there are three computers with three different operating systems. Maybe one of the system can't install the communication software you want, or it's impossible to log in to three computers at the same time.

It's inevitable that you need to operate other people's or public computers. Or maybe you have many different computer platforms. It's very troublesome to transfer small files and send text in this situation.

Transfery's significance, is to transfer small temperary files and share text messages, without login, and no limit on the number of devices.

<span id="environment"></span>

## Environment
To run Transfery, you need:
- <a href="https://github.com/minio/minio.git">Minio</a>, as an object storage server
- MySQL, as a database server
- Sanic, as a back end 
- A server, to enjoy it anytime

<span id="dependencies"></span>

### Dependencies
- [Pipfile](https://github.com/hlf20010508/transfery/blob/master/Pipfile)
- [Pipfile.lock](https://github.com/hlf20010508/transfery/blob/master/Pipfile.lock)

<span id="note"></span>

## Note
- Because no password setting, please do not share you address of your Transfery server on the Internet.
- What you should do is just install Minio and MySQL, and make sure they can run well. Bucket, database and table will be automatically initialized by running config.py.
- This project is just a backend, if you want to modify frontend, please go to <a href="https://github.com/hlf20010508/transfery-vue.git">transfery-vue</a>.
- If you put transfery and transfery-vue under the same directory, running "npm run build" in transfery-vue will use webpack to generate html and js files and automatically import them to transfery.

<span id="running"></span>

## Running Through Docker
Create configuration file
```sh
vim .env
```

Write your configuration(eg.)
```sh
# local port
LOCAL_PORT=5020
# path to restore cache (default: cache)
cache_path=cache
# item per page (default: 15)
item_per_page=15
# ip address and port of minio server
host_minio=123.123.123.123:9000
# whether to use http(false) or https(true)
secure_minio=false
# whether minio is on the same machine with transfery, if true, the ip address of minio server will be 127.0.0.1
local_minio=false
# user name of minio
username_minio=user
# user password of minio
password_minio=12345678
# bucker on minio for transfery
bucket=transfer
# ip address and port of mysql server
host_mysql=123.123.123.123:3306
# same as minio
local_mysql=false
# user name of mysql
username_mysql=root
# user password of mysql
password_mysql=12345678
# database on mysql for transfery
database=transfery
# table name in database for transfery
table=main
```

Installation
```sh
# install docker-compose
pip install docker-compose
# launch
docker-compose up -d
```

## Docker Build
```sh
docker-compose -f docker-compose-build.yml up
```

## Running Directly
``` bash
# install pipenv
pip install pipenv

# use pipenv to install dependencies
pipenv sync

# run configuration setting
pipenv run python config.py

# run service
pipenv run python sanic run.app

# provide host and port
# if you run it on online server, make sure the host is 0.0.0.0
pipenv run python sanic run.app -H 0.0.0.0 -p 8080
```

<span id="background&boot"></span>

### Background Running and Boots up
```bash
# edit transfery@.service
vim transfery@.service

# edit ExecStart in line 6 and WorkingDirectory in line 11, refering the given example.

# copy transfery@.service to /etc/systemd/system
sudo cp transfery@.service /etc/systemd/system

# launch service, USERNAME is the name of current user of os
sudo systemctl start transfery@USERNAME

# check status
sudo systemctl status transfery@USERNAME

# boots up
sudo systemctl enable transfery@USERNAME

# restart service
sudo systemctl restart transfery@USERNAME

# close service
sudo systemctl stop transfery@USERNAME

# cancel boots up
sudo systemctl disable transfery@USERNAME
```
