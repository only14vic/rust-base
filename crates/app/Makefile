ALL += build-app, \
	run-app, \
	run-app no_std=1, \
	run-app no_std=1 static=1, \
	examples-app, \
	tests-app, \

.PHONY: run-app
run-app: flags
	cargo run -p app --bin app $(CARGO_ARGS)

.PHONY: build-app
build-app: flags
ifeq ($(static),)
	cargo rustc -p app --lib --crate-type=cdylib $(CARGO_ARGS)
else
	cargo rustc -p app --lib --crate-type=staticlib $(CARGO_ARGS)
endif

.PHONY: examples-app
examples-app: flags
	cargo run -p app --example app-example1 $(CARGO_ARGS)

.PHONY: tests-app
tests-app: flags
	cargo test -p app --tests $(CARGO_ARGS)
