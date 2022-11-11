package test

quote_1 := glob.quote_meta("abc")

quote_2 := glob.quote_meta("foobar*")

match_1_true := glob.match("*.foo.bar", ["/"], "n.goo.bar.foo.bar")

match_2_false := glob.match("*.foo.bar", ["."], "n.goo.bar.foo.bar")
