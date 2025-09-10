ALL += app-base-build, \
		app-base-build no_std=1, \
		app-base-build static=1, \
		app-base-build static=1 no_std=1, \

TESTS += app-base-tests, \
		app-base-examples, \
		app-base-build, \
		app-base-build no_std=1, \
		app-base-build static=1, \
		app-base-build static=1 no_std=1, \
		app-base-examples.c, \
		app-base-module-examples.c, \
		app-base-examples, \
		app-base-examples no_std=1, \
		app-base-examples static=1, \
		app-base-examples static=1 no_std=1, \

.PHONY: app-base-build
app-base-build: flags
ifeq ($(static),)
	cargo rustc -p app-base --lib --crate-type=cdylib $(CARGO_ARGS)
else
	cargo rustc -p app-base --lib --crate-type=staticlib $(CARGO_ARGS)
endif

.PHONY: app-base-examples
app-base-examples: flags
	cargo test -p app-base --examples $(CARGO_ARGS)

.PHONY: app-base-tests
app-base-tests: flags
	cargo test -p app-base --tests $(CARGO_ARGS) -- --nocapture

app-base-test_%: flags
	cargo test -p app-base --test test_$* $(CARGO_ARGS) -- --nocapture

app-base%example: flags
	cargo run -p app-base --example $@ $(CARGO_ARGS)

app-base%example.c:
	$(MAKE_CC) $(MAKE_CFLAGS) -lapp_base \
	-o $(TARGET_DIR)/app-base$*example \
	crates/app-base/examples/$@
	$(TARGET_DIR)/app-base$*example

.PHONY: app-base-check
app-base-check:
	find target -type f -executable -path "*/release/examples/app-base-example" \( \
			-exec echo -e "-----------------------\n" \; \
			-exec ls -sh {} \; -exec ldd {} \; \
			-exec valgrind --tool=memcheck --leak-check=full --show-leak-kinds=all --error-exitcode=1 {} \; \
		-o -quit \)

.PHONY: app-base-perf
app-base-perf:
	perf record -F99 --call-graph dwarf \
		"$(shell find target -type f -executable -path "*/release/examples/app-base-example" | head -n1)"
	perf report
