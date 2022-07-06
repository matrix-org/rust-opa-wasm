package test

policy[data] {
	data := {
		# these two must be equal during a single query, but different from one invocation to another
		"one": uuid.rfc4122("id"),
		"two": uuid.rfc4122("id"),
	}
}
