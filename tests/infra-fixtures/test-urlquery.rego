package test

encode := urlquery.encode("?foo=1&bar=test")

encode_object := urlquery.encode_object({"foo":"1","bar":"test", "arr": ["foo", "bar"], "obj": {"obj1", "obj2"}})

decode := urlquery.decode("%3Ffoo%3D1%26bar%3Dtest")

decode_object := urlquery.decode_object("arr=foo&arr=bar&bar=test&foo=1&obj=obj1&obj=obj2")