TESTS += examples-app-async, \
		 tests-app-async

app-async-test_%:
	APP_ENV=test cargo test -p app-async --test test_$* $(CARGO_ARGS) -- --nocapture

app-async%example:
	APP_ENV=test cargo run -p app-async --example $@ $(CARGO_ARGS)

.PHONY: app-async-perf
app-async-perf:
	perf record -F99 --call-graph dwarf \
		"$(shell find target -type f -executable -path "*/release/examples/app-async-example1" | head -n1)"
	perf report
