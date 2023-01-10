package test

object_1 := object.union_n([{"a": 1}, {"b": 2}, {"a": 3}])

object_2 := object.union_n([{"a": 1}, {"b": 2}, {"a": 3, "b": 1}])

object_override_by_string := object.union_n([{"a": 1}, {"b": 2}, {"a": "3"}])
