# -*- coding: utf-8 -*-
# A Convenient Temporary Message and File transfer Project
# (C) 2022 L-ING <hlf01@icloud.com>
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <https://www.gnu.org/licenses/>.

from miniopy_async import Minio
from miniopy_async.error import S3Error, InvalidResponseError
import config as myconfig

config = myconfig.load()
port = config['host_minio'].split(':')[1]
host = '127.0.0.1:%s' % port if config['local_minio'] else config['host_minio']
username = config['username_minio']
password = config['password_minio']
bucket = config['bucket']
secure = config['secure_minio']


class Client:
    # Create a client with the MinIO server playground, its access key
    # and secret key.
    def __init__(self, host=host, username=username, password=password, bucket=bucket, secure=secure):
        self.client = Minio(
            host,
            access_key=username,
            secret_key=password,
            secure=secure
        )
        #for printing
        self.host = host
        self.bucket = bucket

    def init(self):
        try:
            if not self.client.bucket_exists(self.bucket):
                self.client.make_bucket(self.bucket)
        except InvalidResponseError as err:
            print(err)

    async def upload(self, remote_path, local_path,):
        try:
            await self.client.fput_object(
                self.bucket, remote_path, local_path)
            print(
                "file is successfully uploaded as \n object %s to bucket %s." % (
                    remote_path, self.bucket)
            )
            address = 'http://'+self.host+'/'+self.bucket+'/'+remote_path
            print(address)
            return address
        except S3Error as exc:
            print("error occurred.", exc)

    async def remove(self, remote_path):
        try:
            await self.client.remove_object(self.bucket, remote_path)
            print("%s is successfully removed from bucket %s" %
                  (remote_path, self.bucket))
        except S3Error as exc:
            print("error occurred.", exc)

    async def remove_all(self):
        try:
            items = await self.list()
            for item in items:
                await self.remove(item['name'])
            print("all objects are removed from bucket %s successfully" %
                  self.bucket)
        except S3Error as exc:
            print("error occurred.", exc)

    async def list(self):
        try:
            obj_list = await self.client.list_objects(self.bucket, recursive=True)
            obj_list = [{'name': obj.object_name, 'size': obj.size,
                         'last_modified': obj.last_modified} for obj in obj_list]
            print('objects list:',obj_list)
            return obj_list
        except S3Error as exc:
            print("error occurred.", exc)

    async def get_download_url(self, remote_path, change_host=None):
        try:
            url = await self.client.presigned_get_object(
                self.bucket,
                remote_path,
                change_host=change_host
            )
            print("successfully created download url %s for %s from bucket %s" %
                  (url, remote_path, self.bucket))
            return url
        except S3Error as exc:
            print("error occurred.", exc)
