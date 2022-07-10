# transfery

> A Convenient Temporary Message and File transfer Project

<br/>

## Languages
- <a href="https://github.com/hlf20010508/transfery/blob/master/README.md">English</a>
- <a href="https://github.com/hlf20010508/transfery/blob/master/README/README.zh_CN.md">简体中文</a>

<br/>

## Catalogue
- <a href="#h1">Interface</a>
- <a href="#h2">Functionality</a>
- <a href="#h3">Significance</a>
- <a href="#h4">Environment</a>
  - <a href="#sh41">Dependencies</a>
- <a href="#h5">Note</a>
- <a href="#h6">Running</a>
  - <a href="#sh61">Background Running and Boots up</a>

<br/>

## Interface<span id="h1"></span>

Web Browser

<img width="1082" alt="image" src="https://user-images.githubusercontent.com/76218469/176151027-c40cc300-7c22-42c5-9da8-2984066a8b99.png">

<img width="1082" alt="image" src="https://user-images.githubusercontent.com/76218469/176161205-42b4f732-569d-4cd4-876e-6d0ae31c5f84.png">

<img width="1082" alt="image" src="https://user-images.githubusercontent.com/76218469/176152167-abc40d1f-26d6-4b19-8438-519ff1c774a1.png">

<br/>

ios WebApp（Add to Home Screen）

<div align=center>
<img width="300" alt="image" src="https://user-images.githubusercontent.com/76218469/176231809-af30a998-f494-479e-8355-2a4c5b5f18dd.PNG"> <img width="300" alt="image" src="https://user-images.githubusercontent.com/76218469/176233010-944534ff-9db5-4935-9e71-44a28af17b28.PNG"> <img width="300" alt="image" src="https://user-images.githubusercontent.com/76218469/176233015-457633fd-bb43-4a7f-b021-4a3e85ef02b2.PNG"> <img width="300" alt="image" src="https://user-images.githubusercontent.com/76218469/176664354-69f6b382-44ef-4592-aa34-57333a22240f.PNG">
</div>

<br/>

## Functionality<span id="h2"></span>

- Send messages
- Send files, multiple files uploads supported
- Full-duplex instant messaging
- Asynchronous framework, messaging while uploading supported
- Scales page elastically when using soft keyboard on phone
- Delete history messages

<br/>

## Significance<span id="h3"></span>

Consider a scenario:

You're in class, and this class requires you to report on stage.

However, although you have brought a computer, but forget to bring a USB flash drive, there is no way to copy the PPT to the classroom computer.

The computer in the classroom is relatively new, and there is no communication software installed, and it's not inconvenient for you to install it directly.

At this time, the reliable way maybe is to send the PPT to yourself by e-mail and then log in to your mailbox to download it.

Or send the PPT to your network disk, and then log in to the website of the network disk to download it.

However, these require login, which may let people observe your operations for a long time.

<br/>

Let's take another scenario:

You have three computers, one is a Windows, one is a Mac, and one is a Linux.

You'll have to send a lot of small files and text messages between these three computers.

If it's between a phone and a computer, using communication software is a good solution.

However, now there are three computers with three different operating systems. Maybe one of the system can't install the communication software you want, or it's impossible to log in to three computers at the same time.

<br/>

It's inevitable that you need to operate other people's or public computers. Or maybe you have many different computer platforms. It's very troublesome to transfer small files and send text in this situation.

<br/>

Transfery's significance, is to transfer small temperary files and share text messages, without login, and no limit on the number of devices.

<br/>

## Environment<span id="h4"></span>

To run Transfery, you need:
- <a href="https://github.com/minio/minio.git">Minio</a>, as an object storage server
- MySQL, as a database server
- Sanic, as a back end 
- A server, to enjoy it anytime

### Dependencies<span id="sh41"></span>

- python>3.6
- sanic 22.6.0
- python-socketio 5.6.0
- minio-async 1.1.0
- ezmysql 0.9.0

Minio-async is maintained by me.

You may not be able to find minio-async on pypi.

Minio-async on gitee <a href="https://gitee.com/hlf01/minio-async.git">https://gitee.com/hlf01/minio-async.git</a>

Minio-async on github <a href="https://github.com/hlf20010508/minio-async.git">https://github.com/hlf20010508/minio-async.git</a>

Minio-async in Pipfile dafaultly use gitee's repository. If you cannot visit the repository on gitee, please change the link to github in Pipfile, and run
```bash
pipenv install
```

<br/>

Refer to Pipfile.lock for more dependencies.

<br/>

### Note<span id="h5"></span>

- Because no password setting, please do not share you address of your Transfery server on the Internet.
- What you should do is just install Minio and MySQL, and make sure they can run well. Bucket, database and table will be automatically initialized by running config.py.
- This project is just a backend, if you want to modify frontend, please go to <a href="https://github.com/hlf20010508/transfery-vue.git">transfery-vue</a>.
- If you put transfery and transfery-vue under the same directory, running "npm run build" in transfery-vue will use webpack to generate html and js files and automatically import them to transfery.

<br/>

## Running<span id="h6"></span>

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
pipenv run python sanic run.app -H 0.0.0.0 -p 5000
```

<br/>

### Background Running and Boots up<span id="sh61"></span>

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
