TESTS += app-web-tests,

.PHONY: app-web-tests
app-web-tests: flags
	cargo test -p app-web --tests $(CARGO_ARGS) -- --nocapture --color always
