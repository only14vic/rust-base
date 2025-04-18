ALL += examples-base, \
		examples-base static=1, \
		examples-base no_std=1, \
		examples-base no_std=1 static=1, \
		check-base info-base, \

.PHONY: examples-base
examples-base: flags
	cargo run -p app-base --example app-base-example1 $(CARGO_ARGS)

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
