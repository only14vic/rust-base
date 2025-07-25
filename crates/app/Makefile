ALL += build-app, \
		build-app no_std=1, \
		build-app no_std=1 static=1, \

TESTS += build-app, \
		build-app no_std=1, \
		examples-app-c, \
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
	cargo build -p app --bins $(CARGO_ARGS)
else
	cargo rustc -p app --lib --crate-type=staticlib $(CARGO_ARGS)
	cargo build -p app --bins $(CARGO_ARGS)
endif

.PHONY: examples-app
examples-app: flags
	cargo run -p app --example app-example1 $(CARGO_ARGS)

.PHONY: tests-app
tests-app: flags
	cargo test -p app --tests $(CARGO_ARGS) -- --nocapture --color always

.PHONY: examples-app-c
examples-app-c: flags objs
	$(MAKE_CC) $(MAKE_CFLAGS) \
		-lapp \
		-o $(TARGET_DIR)/app-example-c \
		crates/app/examples/app-example.c
	$(TARGET_DIR)/app-example-c

.PHONY: perf-app-c
perf-app-c:
	perf record -F99 --call-graph dwarf \
		"$(TARGET_DIR)/app-example-c" --log-level=info
	perf report
