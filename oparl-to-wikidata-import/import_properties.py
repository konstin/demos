import argparse
import json

import os
from wikidataintegrator import wdi_core, wdi_login


def create_properties_mamping(mapping, schemadir, login, server, base_url_template):
    for schemafile in os.listdir(schemadir):
        with open(os.path.join(schemadir, schemafile)) as f:
            schema = json.load(f)

        for propname, prop_options in schema["properties"].items():
            if propname in mapping.keys():
                continue

            wd_type = "String"
            if prop_options["type"] == "string" and "format" in prop_options.keys():
                if prop_options["format"] == "url":
                    wd_type = "URL"
                elif prop_options["format"] == "date-time":
                    wd_type = "time"
            # Every object has an explicit url as value for type, but in wikidata this more properly represented as url
            if propname == "type":
                wd_type = "URL"

            wd_property = wdi_core.WDItemEngine(item_name=propname, domain='dummy', data=[], server=server,
                                                base_url_template=base_url_template)
            wd_property.set_label(propname)
            try:
                property_id = wd_property.write(login, entity_type=u'property', property_datatype=wd_type)
            except Exception as err:
                # Quick'n'dirty getting existing properties
                property_id = err.wd_error_msg["error"]["messages"][0]["parameters"][2].split("|")[1][:-2]
            print(property_id, wd_type)
            mapping[propname] = {"property": property_id, "type": wd_type}

    return mapping


def get_properties_mapping_cached(schemadir, login, server, base_url_template):
    if os.path.isfile('mapping.json'):
        with open('mapping.json') as f:
            mapping = json.load(f)
    else:
        print("Could not read existing mmapping, using new one")
        mapping = {}

    mapping = create_properties_mamping(mapping, schemadir, login, server, base_url_template)
    with open('mapping.json', "w") as f:
        json.dump(mapping, f, indent=4)

    return mapping


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--schemadir", default="/home/konsti/oparl/schema")
    parser.add_argument("--wikibase-server", default="mediawiki.local")
    parser.add_argument("--base-url-template", default="http://{}/api.php")
    args = parser.parse_args()
    schemadir = args.schemadir

    login = wdi_login.WDLogin(
        user='Konsti@bot', pwd='citsdvh4ct69bqepeiblc8p5njnrq26j',
        server=args.wikibase_server, base_url_template=args.base_url_template)

    print(get_properties_mapping_cached(schemadir, login, args.wikibase_server, args.base_url_template))

if __name__ == '__main__':
    main()