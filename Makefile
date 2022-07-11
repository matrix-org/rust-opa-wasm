build-opa:
	ls tests/infra-fixtures/*.rego | xargs -I {} opa build {} -t wasm -e test -o {}.tar.gz
clean-opa:
	rm tests/infra-fixtures/*.tar.gz
