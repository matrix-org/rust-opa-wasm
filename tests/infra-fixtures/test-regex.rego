package test

is_valid_true := regex.is_valid(".*")

is_valid_false := regex.is_valid("*")

find_n_all := regex.find_n("[oa]+", "foo bar", -1)

find_n_few := regex.find_n("[oa]+", "foo bar", 1)

find_n_none := regex.find_n("[oa]+", "foo bar", 0)

split_1 := regex.split("/", "foo//bar/baz")

split_2 := regex.split("/", "")

split_3 := regex.split("/", "foo-bar-baz")

globs_match_true := regex.globs_match("a.a.", ".b.b")

globs_match_false := regex.globs_match("[a-z]+", "[0-9]*")

template_match_true := regex.template_match("/users/id-{[0-9]{1,4}}/update", "/users/id-123/update", "{", "}")

template_match_false := regex.template_match("/users/id-{[0-9]{1,4}}/update", "/users/id-123123/update", "{", "}")

replace_all := regex.replace("abc 123 abcdefg", "[abc]+", "XXX")

replace_empty := regex.replace("", "abc", "XXX")

match_true := regex.match(".*", "foobar")

match_false := regex.match("[0-9]+", "foobar")

submatch_all := regex.find_all_string_submatch_n(
	"([a-z]+)/([a-z]+)",
	"home/user ~ home/root ~ home/admin",
	-1,
)

submatch_some := regex.find_all_string_submatch_n(
	"([a-z]+)/([a-z]+)",
	"home/user ~ home/root ~ home/admin",
	1,
)

submatch_none := regex.find_all_string_submatch_n(
	"([a-z]+)/([a-z]+)",
	"home/user ~ home/root ~ home/admin",
	0,
)
