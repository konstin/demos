import hashlib
import json
from urllib.parse import urlparse, parse_qs, urlencode

import os


class Cache:
    """ the cache's base key """
    def __init__(self):
        self.basekey = ""

        self.hits = 0
        self.misses = 0
        self.lookups = 0
        self.cachedir = ""

    def __init__(self, basekey=""):
        """
            Initialize a Cache instance

            Caches can preprend a basekey to every cached item.
            This makes it possible to use one cache provider (i.e. Redis)
            with multiple Cache instances
        """
        self.basekey = basekey

    def has(self, key):
        """ Check wether a key exists """
        return False

    def get(self, key):
        """ Get the contents of a key """
        return ""

    def set(self, key, value, ttl=0):
        """
            Set the contents of a key

            This allows to optionally set the time this cache item will be kept
        """
        pass

    def fullkey(self, key):
        """
            Get the full key name of a key

            This will preprend the key with the Cache instance's base key value
        """
        if len(self.basekey) > 0:
            return "{}:{}".format(self.basekey, key)
        else:
            return key


class IdiomaticFileCache(Cache):
    def __init__(self, cachedir):
        super().__init__()
        self.cachedir = cachedir
        os.makedirs(self.cachedir, exist_ok=True)

    def sha1_hexdigest(self, string):
        string = str(string).encode('utf_8')
        return hashlib.sha1(string).hexdigest()

    def set(self, key, value, ttl=0):
        filepath = os.path.join(self.cachedir, self.sha1_hexdigest(key))
        with open(filepath, "w") as f:
            f.write(value)

    def get(self, key):
        filepath = os.path.join(self.cachedir, self.sha1_hexdigest(key))
        with open(filepath) as f:
            return f.read()

    def has(self, key):
        filepath = os.path.join(self.cachedir, self.sha1_hexdigest(key))
        return os.path.isfile(filepath)


class FileCache(Cache):
    def __init__(self, basekey=""):
        Cache.__init__(self, basekey=basekey)
        self.cachedir = "/home/konsti/cache-rust"

    def url_to_path(self, url_raw):
        """
        Takes an url as string and returns ap path in the format <cachedir>/<scheme>:<host>[:<port>][/<path>].json

        :param url_raw:
        :return: the path to where the corresponding url is cached
        """
        url = urlparse(url_raw)
        url_options = url.params

        query = parse_qs(url.query)
        query.pop("modified_since", None)
        query.pop("modified_until", None)
        query.pop("created_since", None)
        query.pop("created_until", None)

        if query != {}:
            url_options += "?" + urlencode(query)
        if url.fragment != "":
            url_options += "#" + url.fragment

        return os.path.join(self.cachedir, url.scheme + ":" + url.netloc, url.path[1:] + url_options + ".json")

    def has(self, key):
        return True

    def get(self, key):
        print("KEY: ", key)
        print(key, self.url_to_path(key))
        with open(self.url_to_path(key)) as f:
            read = f.read()

        # Check if we got an external list
        if read[0] == "[":
            deserialized = json.loads(read)
            new = {
                "data": [],
                "pagination": {},
                "links": {}
            }
            for i in deserialized:
                new["data"].append(json.loads(self.get(i)))

            read = json.dumps(new)

        return read
