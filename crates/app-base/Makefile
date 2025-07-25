ALL += build-base, \
		build-base no_std=1, \
		build-base static=1, \
		build-base static=1 no_std=1, \

TESTS += tests-base, \
		build-base, \
		build-base no_std=1, \
		build-base static=1, \
		build-base static=1 no_std=1, \
		examples-base-c, \
		examples-base, \
		examples-base no_std=1, \
		examples-base static=1, \
		examples-base static=1 no_std=1, \

.PHONY: build-base
build-base: flags
ifeq ($(static),)
	cargo rustc -p app-base --lib --crate-type=cdylib $(CARGO_ARGS)
else
	cargo rustc -p app-base --lib --crate-type=staticlib $(CARGO_ARGS)
endif

.PHONY: examples-base
examples-base: flags
	cargo run -p app-base --example app-base-example1 $(CARGO_ARGS)

.PHONY: examples-base-c
examples-base-c: flags objs
	$(MAKE_CC) $(MAKE_CFLAGS) \
		-lapp_base \
		-o $(TARGET_DIR)/app-base-example-c \
		crates/app-base/examples/app-base-example.c
	$(TARGET_DIR)/app-base-example-c

.PHONY: tests-base
tests-base: flags
	cargo test -p app-base --tests $(CARGO_ARGS) -- --nocapture

.PHONY: check-base
check-base:
	find target -type f -executable -path "*/release/examples/app-base-example1" \( \
			-exec echo -e "-----------------------\n" \; \
			-exec ls -sh {} \; -exec ldd {} \; \
			-exec valgrind --tool=memcheck --leak-check=full --show-leak-kinds=all --error-exitcode=1 {} \; \
		-o -quit \)

.PHONY: perf-base
perf-base:
	perf record -F99 --call-graph dwarf \
		"$(shell find target -type f -executable -path "*/release/examples/app-base-example1" | head -n1)"
	perf report
