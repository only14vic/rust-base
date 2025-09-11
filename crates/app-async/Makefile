TESTS += app-async-examples,

.PHONY: app-async-examples
app-async-examples:
	cargo run -p app-async --example app-async-example $(CARGO_ARGS)

app-async-tests:
	cargo test -p app-async --tests $(CARGO_ARGS) -- --nocapture

app-async-test_%:
	cargo test -p app-async --test test_$* $(CARGO_ARGS) -- --nocapture

app-async%example:
	cargo run -p app-async --example $@ $(CARGO_ARGS)

.PHONY: app-async-perf
app-async-perf:
	perf record -F99 --call-graph dwarf \
		"$(shell find target -type f -executable -path "*/release/examples/app-async-example1" | head -n1)"
	perf report
