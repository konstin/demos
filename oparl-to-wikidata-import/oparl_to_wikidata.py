from wikidataintegrator.wdi_core import WDBaseDataType, WDUrl, WDString, WDTime, WDItemEngine


class OParlToWikidata:
    def __init__(self, oparl_client, wikibase_login, mapping, oparl_to_wd, server, base_url_template):
        self.mapping = mapping
        self.wikibase_login = wikibase_login
        self.oparl_client = oparl_client
        self.url_to_item_id = oparl_to_wd
        self.server = server
        self.base_url_template = base_url_template

    def index_all(self):
        """
        In the first pass all the objects are created with their corresponding id and type are created, so the can be
        referenced to. In the second pass all the other attributes are created, which may reference to existing objects.
        """
        print("Minimal Pass")
        for i in self.oparl_client.get_all():
            pass #self.index_single_object(i, False)
        print("Full Pass")
        for i in self.oparl_client.get_all():
            self.index_single_object(i, True)
        print("Done")

    def wd_type_to_class(self, value, wd_type, prop_nr, debug_url="") -> WDBaseDataType:
        if wd_type == WDString.DTYPE:
            return WDString(str(value), prop_nr)
        elif wd_type == WDUrl.DTYPE:
            if str(value) == "" or str(value)[0] == "<":
                print("WARN: Skipping this:", debug_url)
                return None
            return WDUrl(str(value), prop_nr)
        elif wd_type == WDUrl.DTYPE:
            print("GO", value.format("+%Y-%m-%dT00:00:00Z"))
            return WDTime(value.format("+%Y-%m-%dT00:00:00Z"), prop_nr, precision=11)
        else:
            print(wd_type)
            assert False

    def get_claims(self, oparl_object, debug_url=""):
        claims = []
        for liboparl_property in oparl_object.list_properties():
            print(liboparl_property)
            value = oparl_object.get_property(liboparl_property.name)
            if value is None:
                continue

            # Map liboparl attribute names to oparl attribute names
            name = liboparl_property.name
            if "access" not in name and "download" not in name:
                name = name.replace("-url", "")
            if name == "fully-loaded" or name == "vendor-attributes":
                continue
            if name == "oparl-type":
                name = "type"
            normalized_title = name.replace("-", " ").title().replace(" ", "")
            normalized_title = normalized_title[0].lower() + normalized_title[1:]

            if not type(value) == list:
                value_list = [value]
            else:
                value_list = value

            for value in value_list:
                prop_nr = self.mapping[normalized_title]["property"]
                claim = self.wd_type_to_class(value, self.mapping[normalized_title]["type"], prop_nr, debug_url=debug_url)
                if claim:
                    claims.append(claim)

        return claims

    def index_single_object(self, oparl_object, full_pass):
        oparl_id = oparl_object.get_id()

        if full_pass:
            claims = self.get_claims(oparl_object, debug_url=oparl_id)
        else:
            id_claim = WDUrl(value=oparl_id, prop_nr=self.mapping["id"]["property"])
            type_claim = WDUrl(value=oparl_object.get_oparl_type(), prop_nr=self.mapping["type"]["property"])
            claims = [id_claim, type_claim]

        if self.url_to_item_id.has(oparl_id):
            wd_item_id = self.url_to_item_id.get(oparl_id)
            item_name = None
            domain = ""
        else:
            print("Creating new item")
            wd_item_id = ""
            item_name = oparl_id
            domain = None

        wd_item = WDItemEngine(wd_item_id=wd_item_id, item_name=item_name, domain=domain,
                                        data=claims, server=self.server, base_url_template=self.base_url_template)
        wd_item.set_label(oparl_id)
        returned = wd_item.write(self.wikibase_login)
        self.url_to_item_id.set(oparl_id, returned)

        print(oparl_id)
        print("http://{}/index.php?title=Item:{}".format(self.server, returned))
        print()