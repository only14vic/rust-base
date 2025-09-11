ALL += build-app, \
		build-app no_std=1, \

TESTS += build-app, \
		run-config, \
		build-app no_std=1, \
		run-app no_std=1, \
		run-config no_std=1, \
		build-app no_std=1 static=1, \
		run-app no_std=1 static=1, \
		tests-app, \

.PHONY: install-app
install-app:
	$(RUST) cargo install --no-track --force --path=crates/app --root=.

.PHONY: run-app
run-app: flags
	cargo run -p app --bin $(APP_BIN) $(CARGO_ARGS)

.PHONY: run-config
run-config: flags
	cargo run -p app --bin $(APP_BIN) $(CARGO_ARGS) -- config

.PHONY: build-app
build-app: flags
ifneq ($(static),)
	#cargo rustc -p app --lib --crate-type=staticlib $(CARGO_ARGS)
	cargo build -p app --bins $(CARGO_ARGS)
else
	cargo rustc -p app --lib --crate-type=cdylib $(CARGO_ARGS)
	cargo build -p app --bins $(CARGO_ARGS)
endif

.PHONY: examples-app
examples-app:
	cargo test -p app --examples $(CARGO_ARGS) -- --nocapture

.PHONY: tests-app
tests-app:
	cargo test -p app --tests $(CARGO_ARGS) -- --nocapture --color always

app-test_%:
	cargo test -p app --test test_$* $(CARGO_ARGS) -- --nocapture

app-%example: flags
	cargo run -p app --example $@ $(CARGO_ARGS)

app-%example.c:
	$(MAKE_CC) $(MAKE_CFLAGS) -lapp \
	-o $(TARGET_DIR)/app-$*example \
	crates/app/examples/$@
	$(TARGET_DIR)/app-$*example

.PHONY: perf-app
perf-app:
	perf record -F99 --call-graph dwarf \
		"$(TARGET_DIR)/app" --log-level=info
	perf report
