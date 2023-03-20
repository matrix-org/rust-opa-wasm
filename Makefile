#
# building and testing with extras:
# 
# extras, are a bunch of BIFs that does not exist in the Go impl. but do exist in this Rust SDK.
# because we're building to WASM, it's possible to have BIFs that were never implemented in Go.
# all is needed is a custom capabilities.json file while building to WASM with the `opa` CLI,
# and to then implmenet these BIFs in Rust, and map them correctly to the `extras` feature.
#
# Below are two ways to build AND test:
#
# 1. with extras: compile to wasm with opa-caps.json which includes the extra builtin defs.
# these builtin implementations exist in the Rust SDK but not in the native Go impl.
#
# 2. without extras: compile to wasm without any special additions. produces a WASM file
# that uses builtins that exists both in Rust SDK and Go impl. We must not build any `.rego` file that contain
# extras because the Go impl. will not know about them (there are no custom capabilities to describe)
#
# Below tasks make sure that:
# compiling and feature flagging works, and runs the correct set of tests for variant (1) and (2)
# excluding/including the 'extras' test and feature when required.
#

# for development, include everything: uses the custom caps file, will implicitly include the extras rego file, and feature (because 'all-features')
build-opa:
	ls tests/infra-fixtures/*.rego | xargs -I {} opa build {} -t wasm --capabilities opa-caps.json -e test -o {}.tar.gz
clean-opa:
	rm tests/infra-fixtures/*.tar.gz

# test both variants, rebuilding the rego files each time with and without custom capabilities
test:
	make test-core
	make test-extras

# uses the custom caps file, will implicitly include the extras rego file, and feature (because 'all-features')
test-extras:
	ls tests/infra-fixtures/*.rego | xargs -I {} opa build {} -t wasm --capabilities opa-caps.json -e test -o {}.tar.gz
	cargo test --all-features

# uses vanilla compilation with the Go impl, excludes the extras file (with grep), and selects
# features specifically that includes everything except 'extras'
test-core:
	ls tests/infra-fixtures/*.rego | grep -v extras | xargs -I {} opa build {} -t wasm -e test -o {}.tar.gz
	cargo test --features all-builtins,loader

# coverage includes 'extras' (--all-features)
test-cover:
	ls tests/infra-fixtures/*.rego | xargs -I {} opa build {} -t wasm --capabilities opa-caps.json -e test -o {}.tar.gz
	cargo test --all-features --no-fail-fast --tests
