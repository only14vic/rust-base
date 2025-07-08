ALL += examples-app, \
	   tests-app, \

.PHONY: examples-app
examples-app: flags
	cargo run -p app --example app-example1 $(CARGO_ARGS)

.PHONY: tests-app
tests-app: flags
	cargo test -p app --tests $(CARGO_ARGS)
