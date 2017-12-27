#!/usr/bin/env python3
"""
Imports the contents of an oparl api into wikidata so we can run sparql queries on it

Setup:
 - get pipenv (`pip install pipenv`)
 - `pipenv install`
 - `pipenv shell
 - Start a wikibase + sparql stack following https://github.com/wmde/wikibase-docker/blob/master/README-compose.md
 - Create an account: http://localhost:8181/wiki/Special:CreateAccount
 - Create a bot and a passsword: http://localhost:8181/wiki/Special:BotPasswords

Usage:
 - Run ./main.py --help` to list the available options
 - Run `./main.py` with your options
"""

import argparse
import hashlib
import json
import os
from collections import defaultdict
from datetime import timezone, datetime
from typing import Union

import dateutil.parser
import requests
from wikidataintegrator import wdi_login, wdi_core
from wikidataintegrator.wdi_core import WDString, WDUrl, WDTime, WDItemEngine, WDItemID


class Wikiparl:
    def __init__(self, oparl_schema_location, login, server, base_url_template, cachedir):
        self.base_url_template = base_url_template
        self.server = server
        self.login = login
        self.schema = self.load_schema(oparl_schema_location)
        self.type_mapping = {}
        self.id_mapping = self.load_id_mapping()
        self.list_mapping = self.load_list_mapping()
        self.missing_links = defaultdict(list)
        self.suffix = ""
        self.cachedir = cachedir

    def load_schema(self, oparl_schema_location):
        mapping = {}
        for filename in os.listdir(oparl_schema_location):
            with open(os.path.join(oparl_schema_location, filename)) as fp:
                contents = json.load(fp)
            mapping["https://schema.oparl.org/1.0/" + contents["title"]] = contents
        return mapping

    def yield_list(self, url):
        next = url
        while next:
            page = self.load_json(next)
            next = page["pagination"].get("next")
            for elem in page["data"]:
                yield elem

    def load_json(self, url):
        """ Idiomatically cached requests """
        cachefile = os.path.join(self.cachedir, hashlib.sha224(url.encode()).hexdigest())
        if os.path.isfile(cachefile):
            with open(cachefile) as fp:
                return json.load(fp)
        else:
            response = requests.get(url).json()
            with open(cachefile, "w") as fp:
                json.dump(response, fp, indent=4)
            return response

    def prepare_and_push(self, oparl_object):
        oparl_object = {k: v for k, v in oparl_object.items() if ":" not in k}
        keys = []
        for key, value in oparl_object.items():
            if isinstance(value, dict):
                keys.append(key)
            if isinstance(value, list) and (len(value) == 0 or isinstance(value[0], dict)):
                keys.append(key)
        embedded = []
        for key in keys:
            if isinstance(oparl_object[key], dict):
                embedded.append(oparl_object.pop(key))
            elif isinstance(oparl_object[key], list):
                for i in oparl_object.pop(key):
                    embedded.append(i)
            else:
                raise Exception(oparl_object["id"] + " " + key)
        self.push_elem(oparl_object)
        for i in embedded:
            i = {k: v for k, v in i.items() if ":" not in k}
            self.push_elem(i)

    def push_elem(self, oparl_object):
        print("PROESSING", oparl_object["id"])
        oparl_id = oparl_object["id"]

        if oparl_id in self.id_mapping.keys():
            wd_item_id = self.id_mapping.get(oparl_id)
            item_name = None
            domain = ""
        else:
            wd_item_id = ""
            item_name = oparl_id
            domain = None

        claims = self.get_claims(oparl_object)
        wd_item = WDItemEngine(wd_item_id=wd_item_id, item_name=item_name, domain=domain,
                               data=claims, server=self.server, base_url_template=self.base_url_template)
        wd_item.set_label(oparl_id)
        returned = wd_item.write(self.login)
        self.id_mapping[oparl_id] = returned

        if item_name:
            print("CREATED", "http://{}/index.php?title=Item:{}".format(self.server, returned))
        else:
            print("UPDATED", "http://{}/index.php?title=Item:{}".format(self.server, returned))

    def get_claims(self, oparl_object):
        claims = []
        for key, value in oparl_object.items():
            if not value:
                continue

            if not type(value) == list:
                value_list = [value]
            else:
                value_list = value

            for claim_value in value_list:
                # Skip additional properties that are not in the spec
                if key not in self.type_mapping:
                    continue
                if self.type_mapping[key]["type"] == WDItemID.DTYPE and claim_value not in self.id_mapping:
                    self.missing_links[oparl_object["id"]].append((key, claim_value))
                    continue
                claim = self.create_single_claim(claim_value, self.type_mapping[key]["type"],
                                                 self.type_mapping[key]["property"])
                if claim:
                    claims.append(claim)

        return claims

    def create_single_claim(self, value, wd_type, prop_nr) -> Union[WDString, WDItemID, None, WDUrl, WDTime]:
        if wd_type == WDString.DTYPE:
            # Strip away everything wikidata doesn't like
            bad_chars = "\n\t\r„“"
            value = str(value)
            for bad_char in bad_chars:
                value = value.replace(bad_char, "")
            value = value[:390].strip()
            return WDString(value, prop_nr)
        if wd_type == WDItemID.DTYPE:
            if not self.id_mapping.get(value):
                # Catch external lists
                data = self.load_json(value)
                if isinstance(data.get("data"), list):
                    return WDUrl(str(value), self.type_mapping["externalList"]["property"])
            return WDItemID(self.id_mapping[value], prop_nr)
        elif wd_type == WDUrl.DTYPE:
            if str(value) == "" or str(value)[0] == "<":
                print("WARN: Skipping this:")
                return None
            return WDUrl(str(value), prop_nr)
        elif wd_type == WDTime.DTYPE:
            if value == "-0001-11-30T00:00:00+01:00":
                return WDTime("+0001-01-01T00:00:00Z", prop_nr, precision=11)
            value = dateutil.parser.parse(value).strftime("+%Y-%m-%dT00:00:00Z")
            return WDTime(value, prop_nr, precision=11)
        else:
            print("SKIPPING", wd_type)
            return None

    def run(self, entrypoint):
        self.first_pass(entrypoint)

        self.save_id_mapping()

        self.second_pass()

        self.save_id_mapping()

    def first_pass(self, entrypoint):
        body_lists = ["paper", "organization", "person", "meeting"]
        system = self.load_json(entrypoint)
        self.prepare_and_push(system)
        for body in self.yield_list(system["body"]):
            self.prepare_and_push(body)
            for list_name in body_lists:
                if body[list_name] in self.list_mapping:
                    print("SKIPPING {}".format(body[list_name]))
                    continue
                for oparl_object in self.yield_list(body[list_name]):
                    self.prepare_and_push(oparl_object)
                self.save_id_mapping()
                now = datetime.now(timezone.utc).astimezone()
                self.list_mapping[body[list_name]] = now.replace(microsecond=0).isoformat()
                self.save_list_mapping()

    def second_pass(self):
        for oparl_object, values in self.missing_links.items():
            print("ADDMISSING", oparl_object)
            wd_item_id = self.id_mapping.get(oparl_object)
            claims = []
            for (key, value) in values:
                claim = self.create_single_claim(value, self.type_mapping[key]["type"],
                                                 self.type_mapping[key]["property"])
                claims.append(claim)
                print("CLAIM", claim)
            wd_item = WDItemEngine(wd_item_id=wd_item_id, item_name=None, domain="",
                                   data=claims, server=self.server, base_url_template=self.base_url_template)
            wd_item.write(self.login)

    def load_id_mapping(self):
        if os.path.isfile("id-mapping.json"):
            with open("id-mapping.json") as fp:
                return json.load(fp)
        else:
            return {}

    def save_id_mapping(self):
        with open("id-mapping.json", "w") as fp:
            json.dump(self.id_mapping, fp, indent=4)

    def load_type_mapping(self):
        if os.path.isfile("type-mapping.json"):
            with open("type-mapping.json") as fp:
                self.type_mapping = json.load(fp)
        else:
            print("No mapping found, creating a new one")
            self.create_properties_mapping()

    def load_list_mapping(self):
        if os.path.isfile("list-mapping.json"):
            with open("list-mapping.json") as fp:
                return json.load(fp)
        else:
            print("No mapping found, creating a new one")
            return {}

    def save_list_mapping(self):
        with open("list-mapping.json", "w") as fp:
            json.dump(self.id_mapping, fp, indent=4)

    def create_properties_mapping(self):
        self.add_property("externalList", WDUrl.DTYPE)

        for oparl_type in self.schema.values():
            for propname, prop_options in oparl_type["properties"].items():
                if propname in self.type_mapping.keys():
                    continue

                wd_type = WDString.DTYPE
                if prop_options["type"] == "string" and "format" in prop_options.keys():
                    if prop_options["format"] == "url":
                        if "references" in prop_options.keys():
                            if prop_options["references"] == "externalList":
                                continue
                            wd_type = WDItemID.DTYPE
                        else:
                            wd_type = WDUrl.DTYPE
                    elif prop_options["format"] == "date-time" and prop_options["format"] == "date":
                        wd_type = WDTime.DTYPE
                # Every object has an explicit url as value for type, but in wikidata this more properly represented
                # as url
                if propname == "type":
                    wd_type = WDUrl.DTYPE

                self.add_property(propname, wd_type)
        with open("type-mapping.json", "w") as fp:
            json.dump(self.type_mapping, fp, indent=4)

    def add_property(self, propname, wd_type):
        wd_property = wdi_core.WDItemEngine(item_name=propname + self.suffix, domain='dummy', data=[],
                                            server=self.server, base_url_template=self.base_url_template)
        wd_property.set_label(propname + self.suffix)
        try:
            property_id = wd_property.write(self.login, entity_type=u'property', property_datatype=wd_type)
        except Exception as err:
            # Quick'n'dirty getting existing properties
            property_id = err.wd_error_msg["error"]["messages"][0]["parameters"][2].split("|")[1][:-2]
        print("PROPERTY", property_id, propname, wd_type)
        self.type_mapping[propname] = {"property": property_id, "type": wd_type}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--entrypoint", default="http://localhost:8080/oparl/v1.0/")
    parser.add_argument("--wikibase-server", default="mediawiki.local")
    parser.add_argument("--base-url-template", default="http://{}/api.php")
    parser.add_argument("--oparl_schema_location", default="/home/konsti/oparl/schema")
    # Wikibase wants a password, wikibase gets a password. I don't care if that password is in git
    parser.add_argument("--cachedir", default="./cache")
    parser.add_argument("--user", default="Bot1@Bot1")
    parser.add_argument("--password", default="kft004cr9toivjesd7obnqm16mpr3596")
    args = parser.parse_args()

    os.makedirs(args.cachedir, exist_ok=True)

    login = wdi_login.WDLogin(
        user=args.user, pwd=args.password,
        server=args.wikibase_server, base_url_template=args.base_url_template)

    loader = Wikiparl(args.oparl_schema_location, login, args.wikibase_server, args.base_url_template, args.cachedir)
    loader.load_type_mapping()
    loader.run(args.entrypoint)


if __name__ == '__main__':
    main()
