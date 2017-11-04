import argparse

from wikidataintegrator import wdi_login

from oparl_to_wikidata import OParlToWikidata
from cache import IdiomaticFileCache
from import_properties import get_properties_mapping_cached
from server_iterator import ServerIterator


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--oparl-endpoint", default="http://localhost:8080/oparl/v1.0")
    parser.add_argument("--wikibase-server", default="mediawiki.local")
    parser.add_argument("--base-url-template", default="http://{}/api.php")
    parser.add_argument("--schemadir", default="/home/konsti/oparl/schema")
    args = parser.parse_args()

    oparl_client = ServerIterator(url=args.oparl_endpoint,
                                  cache=IdiomaticFileCache("/home/konsti/cache-idiomatic/url-to-json"))

    login = wdi_login.WDLogin(
        user='Konsti@bot', pwd='citsdvh4ct69bqepeiblc8p5njnrq26j',
        server=args.wikibase_server, base_url_template=args.base_url_template)

    mapping = get_properties_mapping_cached(args.schemadir, login, args.wikibase_server, args.base_url_template)

    oparl_to_wd = IdiomaticFileCache("/home/konsti/cache-idiomatic/url-to-wd-item-id")
    importer = OParlToWikidata(oparl_client, login, mapping, oparl_to_wd, args.wikibase_server, args.base_url_template)
    importer.index_all()


if __name__ == '__main__':
    main()
