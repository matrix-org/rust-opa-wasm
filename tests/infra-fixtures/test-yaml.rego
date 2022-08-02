package test

is_valid := yaml.is_valid("--\nfoo: bar num: 1\n")

is_valid_1 := yaml.is_valid("foo: bar\nnum: 1")

marshal := yaml.marshal({"foo": "bar", "num": 1})

unmarshal := yaml.unmarshal("foo: bar\nnum: 1\nnum2: 2")