#!/usr/bin/env python3
import json
import os
import re

oparl_schema = "/home/konsti/oparl/schema"
outfile = "src/schema.rs"


def to_snake_case(name):
    """ https://stackoverflow.com/a/1176023/3549270 """
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', name)
    return re.sub('([a-z0-9])([A-Z])', r'\1_\2', s1).lower()


def json_schema_to_rust(property_):
    type_ = property_["type"]
    if type_ == "string":
        if property_.get("format") == "date-time":
            return "DateTime<FixedOffset>"
        elif property_.get("format") == "date":
            return "String"
        elif property_.get("format") == "url":
            if property_.get("references") == "externalList":
                return "ExternalListUrl<{}>".format(property_["items"]["schema"].split(".")[0])
            elif "references" in property_:
                return "OParlUrl<{}>".format(property_["references"])
        return "String"
    elif type_ == "boolean":
        return "bool"
    elif type_ == "object":
        if "schema" not in property_:
            return "JsonValue"
        return property_["schema"].split(".")[0]
    elif type_ == "integer":
        return "usize"
    elif type_ == "array":
        return "Vec<{}>".format(json_schema_to_rust(property_["items"]))
    else:
        print("Failed: ", property_)
        raise RuntimeError()


def main():
    outlines = [
        "use serde_json::Value as JsonValue;",
        "use urls::{OParlUrl, ExternalListUrl};",
        "use chrono::prelude::*;"
        "",
    ]
    for i in os.listdir(oparl_schema):
        with open(os.path.join(oparl_schema, i)) as f:
            schema = json.load(f)

        outlines.append("#[derive(Deserialize, Serialize)]")
        outlines.append('#[serde(rename_all = "camelCase")]')
        outlines.append("pub struct " + schema["title"] + "{")
        for name, property in schema["properties"].items():
            print(name)
            type_ = json_schema_to_rust(property)
            if name not in schema["required"]:
                type_ = "Option<{}>".format(type_)
            name = to_snake_case(name)
            if name == "type":
                outlines.append('    #[serde(rename = "type")]')
                name = "oparl_type"
            outlines.append("    pub {}: {},".format(name, type_))
        outlines.append("}")
        outlines.append("")
        with open(outfile, "w") as f:
            f.write("\n".join(outlines))


if __name__ == '__main__':
    main()
