ALL += build-app, \
	   build-app no_std=1, \
	   examples-app-c, \
	   run-app, \
	   run-app no_std=1, \
	   run-app no_std=1 static=1, \
	   examples-app, \
	   tests-app, \

.PHONY: run-app
run-app: flags
	cargo run -p app --bin app $(CARGO_ARGS)

.PHONY: build-app
build-app: flags
ifeq ($(static),)
	cargo rustc -p app --lib --crate-type=cdylib $(CARGO_ARGS)
else
	cargo rustc -p app --lib --crate-type=staticlib $(CARGO_ARGS)
endif

.PHONY: examples-app
examples-app: flags
	cargo run -p app --example app-example1 $(CARGO_ARGS)

.PHONY: tests-app
tests-app: flags
	cargo test -p app --tests $(CARGO_ARGS)

.PHONY: examples-app-c
examples-app-c: flags
	cc -std=gnu18 -Os -g -pipe -march=native -flto=2 -fno-fat-lto-objects -fuse-linker-plugin \
		-fPIC -Wall -Wextra \
		-Wl,--gc-sections,-z,relro,-z,now,-rpath='$$ORIGIN',-rpath='$$ORIGIN/lib',-rpath='$$ORIGIN/../lib',-rpath='$(target_dir)' \
		-L$(target_dir) -lapp \
		-o $(target_dir)/app-example-c \
		crates/app/examples/app-example.c
	$(target_dir)/app-example-c

.PHONY: perf-app-c
perf-app-c:
	perf record -F99 --call-graph dwarf \
		"$(target_dir)/app-example-c" --log-level=info
	perf report
