ALL += build-app, \
		build-app no_std=1, \

TESTS += tests-app, \
		examples-app, \
		build-app, \
		run-app args=config, \
		build-app no_std=1, \
		run-app no_std=1, \
		run-app no_std=1 args=config, \
		build-app no_std=1 static=1, \
		run-app no_std=1 static=1, \
		app-example.c, \

.PHONY: install-app
install-app: clean build-app strip
	mkdir -p bin lib
	find bin lib -type f -delete
	find $(TARGET_DIR) -maxdepth 1 -type f -executable ! -name "*.so" -exec install -D {} $(DESTDIR)/bin/ \;
	find $(TARGET_DIR) -maxdepth 1 -type f -executable -name "*.so" -exec install -D {} $(DESTDIR)/lib/ \;

app-test_%:
	APP_ENV=test cargo test -p app --test test_$* $(CARGO_ARGS) -- --nocapture

app%example: flags
	APP_ENV=test cargo run -p app --example $@ $(CARGO_ARGS)

app%example.c:
	$(MAKE_CC) $(MAKE_CFLAGS) -lapp \
	-I./include \
	-o $(TARGET_DIR)/app$*example-c \
	crates/app/examples/$@
	$(TARGET_DIR)/app$*example-c

.PHONY: perf-app
perf-app:
	perf record -F99 --call-graph dwarf \
		"$(TARGET_DIR)/app" $(args)
	perf report
