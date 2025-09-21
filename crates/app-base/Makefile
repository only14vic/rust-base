ALL += build-app-base, \
		build-app-base no_std=1, \
		build-app-base static=1, \
		build-app-base static=1 no_std=1, \

TESTS += tests-app-base, \
		examples-app-base, \
		build-app-base, \
		build-app-base no_std=1, \
		build-app-base static=1, \
		build-app-base static=1 no_std=1, \
		app-base-example.c, \
		app-base-module-example.c, \
		examples-app-base, \
		app-base-example, \
		app-base-example no_std=1, \
		app-base-example static=1, \
		app-base-example static=1 no_std=1, \

app-base-test_%:
	APP_ENV=test cargo test -p app-base --test test_$* $(CARGO_ARGS) -- --nocapture

app-base%example: flags
	APP_ENV=test cargo run -p app-base --example $@ $(CARGO_ARGS)

app-base%example.c:
	$(MAKE_CC) $(MAKE_CFLAGS) -lapp_base \
	-I./include \
	-o $(TARGET_DIR)/app-base$*example-c \
	crates/app-base/examples/$@
	$(TARGET_DIR)/app-base$*example-c

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
		"$(TARGET_DIR)/examples/app-base-example"
	perf report
