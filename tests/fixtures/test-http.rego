package fixtures

import rego.v1

# Test that automatic ser/der is working fine for request and response
get_json := http.send({"url": sprintf("%s/json", [input.base_url]), "method": "get"})
get_yaml := http.send({"url": sprintf("%s/yaml", [input.base_url]), "method": "get"})
post_json := http.send({"url": sprintf("%s/post", [input.base_url]), "method": "post", "body": {"key": "value"}})

# Test a connection error doesn't error out the whole policy when using raise_error=false
get_no_conn := http.send({"url": "https://cahbe8ang5umaiwavai1shuchiehae7u.com", "method": "get", "raise_error": false})

# Test automatic redirection
get_redirect := http.send({"url": sprintf("%s/redirect", [input.base_url]), "method": "get"})
get_redirect_follow := http.send({"url": sprintf("%s/redirect", [input.base_url]), "method": "get", "enable_redirect": true})
