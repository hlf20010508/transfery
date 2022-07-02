from minio import Minio
from minio.error import S3Error, InvalidResponseError
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
        #打印
        self.host = host
        self.bucket = bucket

    def init(self):
        try:
            if not self.client.bucket_exists(self.bucket):
                self.client.make_bucket(self.bucket)
        except InvalidResponseError as err:
            print(err)

    async def upload(self, remote_path, data, size):
        try:
            await self.client.put_object(
                self.bucket, remote_path, data, int(size))
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

    async def get_download_url(self, remote_path):
        try:
            url = await self.client.presigned_get_object(
                self.bucket,
                remote_path,
            )
            print("successfully created download url %s for %s from bucket %s" %
                  (url, remote_path, self.bucket))
            return url
        except S3Error as exc:
            print("error occurred.", exc)
