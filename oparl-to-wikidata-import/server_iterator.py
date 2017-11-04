import hashlib

import gi
import requests

gi.require_version('OParl', '0.2')

from gi.repository import OParl
from gi.repository import GLib
from collections import deque


class ServerIterator:
    """
    Util Class for yielding all objects of a server while using a cache
    """

    def __init__(self, url, cache):
        self.url = url
        self.cache = cache
        self.client = OParl.Client()
        self.client.set_strict(False)
        self.client.connect('resolve_url', self.resolve_url)
        self.seen = []

    def resolve_url(self, client, url, status):
        if url is None:  # This is from objects liboparl failed to resolve!
            print("None")
            return None
        if not self.cache.has(url):
            r = requests.get(url)
            r.raise_for_status()
            self.cache.set(url, r.text)
            return r.text
        else:
            text = self.cache.get(url)
            return text

    def validate(self):
        system = self.client.open(self.url)
        yield self.validate_object(system)

        bodies = system.get_body()

        for body in bodies:
            yield self.validate_object(body)
            neighbors = deque(self.get_unseen_neighbors(body))
            while len(neighbors) > 0:
                neighbor = neighbors.popleft()
                if not neighbor:
                    print("FAILURE")
                    continue

                value = self.validate_object(neighbor)
                if value:
                    yield value

                additional_neighbors = self.get_unseen_neighbors(body)
                if len(additional_neighbors) > 0:
                    neighbors.extend(additional_neighbors)

    def get_all(self):
        self.seen = []
        system = self.client.open(self.url)
        yield self.validate_object(system)

        bodies = system.get_body()
        neighbors = deque(bodies)
        while len(neighbors) > 0:
            neighbor = neighbors.popleft()
            value = self.validate_object(neighbor)
            neighbors.extend(self.get_unseen_neighbors(neighbor))
            if value:
                yield value

        print("Finished")

    def validate_object(self, object):
        """ Validate a single object """
        hash = self.get_object_hash(object)
        if not hash or hash in self.seen:
            return None
        else:
            self.seen.append(hash)

        return object

    def get_unseen_neighbors(self, object):
        unseen_neighbors = []

        if object is None or object.get_id() is None:
            print("FAILURE get_unseen_neighbors")
            return []

        try:
            object_neighbors = object.get_neighbors()
        except GLib.Error:
            print("FAILURE: ", object.get_id())
            object_neighbors = []

        for neighbor in object_neighbors:
            hash = self.get_object_hash(neighbor)
            if hash not in self.seen:
                unseen_neighbors.append(neighbor)

        return unseen_neighbors

    def get_object_hash(self, object):
        """ Compute the hash with which the an object is tracked by the validator """
        if object.get_id() is None:
            # TODO: track invalid object id
            print("FAIL: ", object)
            return None
        return self.sha1_hexdigest(object.get_id().encode('ascii'))

    def sha1_hexdigest(self, string):
        string = str(string).encode('utf_8')

        return hashlib.sha1(string).hexdigest()
