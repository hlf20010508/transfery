# :project: transfery
# :author: L-ING
# :copyright: (C) 2022 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

from miniopy_async import Minio
from miniopy_async.error import S3Error, InvalidResponseError
from miniopy_async.helpers import genheaders
from miniopy_async.datatypes import Part


class Storage:
    # Create a client with the MinIO server playground, its access key
    # and secret key.
    def __init__(self, host, username, password, bucket, secure):
        self.client = Minio(
            endpoint=host,
            access_key=username,
            secret_key=password,
            secure=secure
        )
        # for printing
        self.host = host
        self.bucket = bucket

    def init(self):
        try:
            if not self.client.bucket_exists(self.bucket):
                self.client.make_bucket(self.bucket)
        except InvalidResponseError as err:
            print(err)

    async def create_multipart_upload_id(self, remote_path):
        try:
            headers = genheaders(
                headers=None,
                sse=None,
                tags=None,
                retention=None,
                legal_hold=False
            )
            headers["Content-Type"] = "application/octet-stream"
            upload_id = await self.client._create_multipart_upload(self.bucket, remote_path, headers)
            return upload_id
        except S3Error as exc:
            print("error occurred.", exc)

    async def multipart_upload(self, remote_path, upload_id, part_data, part_number):
        try:
            etag = await self.client._upload_part(
                bucket_name=self.bucket,
                object_name=remote_path,
                data=part_data,
                headers=None,
                upload_id=upload_id,
                part_number=part_number
            )
            return etag
        except S3Error as exc:
            print("error occurred.", exc)

    async def complete_multipart_upload(self, remote_path, upload_id, parts):
        try:
            parts = [Part(part['partNumber'], part['etag']) for part in parts]
            await self.client._complete_multipart_upload(
                bucket_name=self.bucket,
                object_name=remote_path,
                upload_id=upload_id,
                parts=parts
            )
            print(
                "file is successfully uploaded as object %s to bucket %s." % (
                    remote_path,
                    self.bucket
                )
            )
            address = 'http://'+self.host+'/'+self.bucket+'/'+remote_path
            print(address)
        except S3Error as exc:
            print("error occurred.", exc)

    async def remove(self, remote_path):
        try:
            await self.client.remove_object(self.bucket, remote_path)
            print(
                "%s is successfully removed from bucket %s" % (
                    remote_path,
                    self.bucket
                )
            )
        except S3Error as exc:
            print("error occurred.", exc)

    async def remove_all(self):
        try:
            items = await self.list()
            for item in items:
                await self.remove(item['name'])
            print("all objects are removed from bucket %s successfully" % self.bucket)
        except S3Error as exc:
            print("error occurred.", exc)

    async def list(self):
        try:
            obj_list = await self.client.list_objects(self.bucket, recursive=True)
            obj_list = [
                {
                    'name': obj.object_name,
                    'size': obj.size,
                    'last_modified': obj.last_modified
                }
                for obj in obj_list
            ]
            print('objects list:', obj_list)
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
            print(
                "successfully created download url %s for %s from bucket %s" % (
                    url,
                    remote_path,
                    self.bucket
                )
            )
            return url
        except S3Error as exc:
            print("error occurred.", exc)
