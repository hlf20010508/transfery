# :project: transfery
# :author: L-ING
# :copyright: (C) 2024 L-ING <hlf01@icloud.com>
# :license: MIT, see LICENSE for more details.

def rename(old_filename, time):
    temp = old_filename.split('.')
    temp[0] += '_'+str(time)[:-3]
    temp = '.'.join(temp)
    temp = temp.split()
    temp = '_'.join(temp)
    return temp

def getFromPostJson(request, key):
    return None if not key in request.json else request.json[key]