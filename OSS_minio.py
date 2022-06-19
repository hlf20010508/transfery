from minio import Minio
from minio.error import S3Error


class OSS:
    # Create a client with the MinIO server playground, its access key
    # and secret key.
    def __init__(self, host, username, password, bucket):
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
    
    def upload_stream(self,remote_path,data):
        try:
            self.client.put_object(self.bucket,remote_path,data,-1,part_size=5*1024*1024)
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


def init():
    host = "124.223.224.49:9000"
    username = "hlf01"
    password = "1486922887"
    bucket = 'transfer'
    return OSS(host, username, password, bucket)
