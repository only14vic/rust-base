ALL += build-base, \
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
	cargo rustc -p app-base --lib --crate-type=staticlib $(CARGO_ARGS) --verbose -- --print native-static-libs
endif

.PHONY: examples-base
examples-base: flags
	cargo run -p app-base --example app-base-example1 $(CARGO_ARGS)

.PHONY: examples-base-c
examples-base-c: flags
	cc -std=gnu18 -Os -g -pipe -march=native -flto=2 -fno-fat-lto-objects -fuse-linker-plugin \
		-fPIC -Wall -Wextra \
		-Wl,--gc-sections,-z,relro,-z,now,-rpath='$$ORIGIN',-rpath='$$ORIGIN/lib',-rpath='$$ORIGIN/../lib',-rpath='$(target_dir)' \
		-L$(target_dir) -lapp_base \
		-o $(target_dir)/app-base-example-c \
		crates/app-base/examples/app-base-example.c
	$(target_dir)/app-base-example-c

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
