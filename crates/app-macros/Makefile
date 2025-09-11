TESTS += app-macros-tests, \

.PHONY: app-macros-tests
app-macros-tests: flags
	cargo test -p app-macros $(CARGO_ARGS) -- --nocapture --color always

.PHONY: app-macros-perf
app-macros-perf:
	cargo build -p app-macros --tests $(CARGO_ARGS)
	perf record -F99 --call-graph dwarf \
		"$(shell find target -type f -executable -path */release/deps/app_macros_test*)"
	perf report
