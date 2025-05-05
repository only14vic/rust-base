TESTS += tests-app-macros, \

.PHONY: app-macros-perf
app-macros-perf:
	cargo build -p app-macros --tests $(CARGO_ARGS)
	perf record -F99 --call-graph dwarf \
		"$(shell find target -type f -executable -path */release/deps/app_macros_test*)"
	perf report
