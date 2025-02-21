build-opa:
	ls tests/fixtures/*.rego | xargs -I {} opa build {} -t wasm -e fixtures -o {}.tar.gz
clean-opa:
	rm tests/fixtures/*.tar.gz
