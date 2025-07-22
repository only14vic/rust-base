ALL +=

.PHONY: tests-web
tests-web: flags
	cargo test -p app-web --tests $(CARGO_ARGS) -- --nocapture --color always

.PHONY: example-web-http-server
examples-web-http-server:
	cargo run -p app-web --example app-web-http-server-example $(CARGO_ARGS)
