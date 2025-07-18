ALL += examples-async,

.PHONY: examples-async
examples-async:
	cargo run -p app-async --example app-async-example1 $(CARGO_ARGS)

.PHONY: example-http-server
examples-http-server:
	cargo run -p app-async --example app-http-server-example $(CARGO_ARGS)

.PHONY: perf-async
perf-async:
	perf record -F99 --call-graph dwarf \
		"$(shell find target -type f -executable -path "*/release/examples/app-async-example1" | head -n1)"
	perf report
