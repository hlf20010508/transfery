from minio import Minio
from minio.error import S3Error
import config as myconfig
import minio_progress

config = myconfig.load()
port=config['host_minio'].split(':')[1]
host = '127.0.0.1:%s'%port if config['local_minio'] else config['host_minio']
username = config['username_minio']
password = config['password_minio']
bucket = config['bucket']

class Client:
    # Create a client with the MinIO server playground, its access key
    # and secret key.
    def __init__(self, host=host, username=username, password=password, bucket=bucket):
        self.host = host
        self.client = Minio(
            host,
            access_key=username,
            secret_key=password,
            secure=False
        )
        self.bucket = bucket

    def upload(self, remote_path, local_path):
        try:
            self.client.fput_object(
                self.bucket, remote_path, local_path,
            )
            print(
                "%s is successfully uploaded as \n object %s to bucket %s." % (
                    local_path, remote_path, self.bucket)
            )
            address = 'http://'+self.host+'/'+self.bucket+'/'+remote_path
            print(address)
            return address
        except S3Error as exc:
            print("error occurred.", exc)

    def upload_stream(self, remote_path, data, size, progress=None):
        try:
            self.client.put_object(
                self.bucket, remote_path, data, int(size), progress=minio_progress.Progress())
            print(
                "file is successfully uploaded as \n object %s to bucket %s." % (
                    remote_path, self.bucket)
            )
            address = 'http://'+self.host+'/'+self.bucket+'/'+remote_path
            print(address)
            return address
        except S3Error as exc:
            print("error occurred.", exc)

    def download(self, remote_path, local_path):
        try:
            self.client.fget_object(self.bucket, remote_path, local_path)
            print("object %s is successfully downloaded to \n %s from bucket %s." % (
                remote_path, local_path, self.bucket))
        except S3Error as exc:
            print("error occurred.", exc)

    def download_stream(self, remote_path):
        try:
            response = self.client.get_object(self.bucket, remote_path)
            return response
        except S3Error as exc:
            print("error occurred.", exc)

    def remove(self, remote_path):
        try:
            self.client.remove_object(self.bucket, remote_path)
            print("%s is successfully removed from bucket %s" %
                  (remote_path, self.bucket))
        except S3Error as exc:
            print("error occurred.", exc)

    def list(self):
        try:
            obj_list = self.client.list_objects(self.bucket, recursive=True)
            obj_list = [[obj.object_name, obj.size, obj.last_modified]
                        for obj in obj_list]
            print(obj_list)
            return obj_list
        except S3Error as exc:
            print("error occurred.", exc)

    def get_download_url(self, remote_path):
        try:
            url = self.client.presigned_get_object(
                self.bucket,
                remote_path,
            )
            print("successfully created download url %s for %s from bucket %s" %
                  (url, remote_path, self.bucket))
            return url
        except S3Error as exc:
            print("error occurred.", exc)

    def get_upload_url(self, remote_path):
        try:
            url = self.client.presigned_put_object(
                self.bucket,
                remote_path,
            )
            print("successfully created upload url %s for %s to bucket %s" %
                  (url, remote_path, self.bucket))
            return url
        except S3Error as exc:
            print("error occurred.", exc)
