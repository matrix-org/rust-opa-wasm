package test

default allow := false

allow {
	input.attributes.request.http.method == "GET"
	input.attributes.request.http.path == "/"
}

allow {
	input.attributes.request.http.headers.authorization == "Basic charlie"
}
