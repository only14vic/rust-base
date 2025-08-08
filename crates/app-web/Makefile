TESTS += tests-web,

.PHONY: tests-web
tests-web: flags
	cargo test -p app-web --tests $(CARGO_ARGS) -- --nocapture --color always
