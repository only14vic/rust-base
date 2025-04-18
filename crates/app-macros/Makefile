ALL += tests-macros, \

.PHONY: tests-macros
tests-macros: flags
	cargo test -p app-macros $(CARGO_ARGS) -- --nocapture --color always

.PHONY: perf-macros
perf-macros:
	cargo build -p app-macros --tests $(CARGO_ARGS)
	perf record -F99 --call-graph dwarf \
		"$(shell find target -type f -executable -path */release/deps/app_macros_test*)"
	perf report
