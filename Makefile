-include .env
export

SHELL = sh
.DEFAULT_GOAL = flags

ifndef VERBOSE
.SILENT:
endif

make = make --no-print-directory

CARGO_ARGS =
RUSTFLAGS = -Ctarget-cpu=native \
			-Clinker-plugin-lto \
			-Clink-arg=-fuse-ld=lld \
			-Clink-arg=-lc

ifeq ($(debug),)
	CARGO_ARGS += --release
endif

ifneq ($(static),)
	CARGO_BUILD_TARGET = x86_64-unknown-linux-musl
	RUSTFLAGS += -Ctarget-feature=+crt-static
else
	#CARGO_BUILD_TARGET = x86_64-unknown-linux-gnu
	#RUSTFLAGS +=
ifneq ($(dynamic),)
	RUSTFLAGS += -Cprefer-dynamic
endif
endif

ifneq ($(no_std),)
	RUSTFLAGS += -Cpanic=abort
	CARGO_ARGS += --no-default-features
endif

target_dir = $(shell cargo metadata --format-version 1|jq ".[\"target_directory\"]"|tr -d '"')/$(CARGO_BUILD_TARGET)/release

CARGO_ARGS += $(args)

ALL =
-include crates/*/Makefile
ALL += info

all: clean check
	echo $(ALL) | sed 's/[,\ ]\+$$//g' | sed 's/\s*,\+\s*/\n/g' | xargs -I '{}' sh -c "$(make) {}"

clean:
	find ./target \
		-path "./target/*" -name "*app*" -type f -executable -delete

check:
	$(eval RUSTFLAGS=)
	cargo check --workspace --no-default-features
	cargo check --workspace
	cargo clippy --no-deps
	rustup run nightly rustfmt --check crates/*/src/**/*.rs

.PHONY: info
info:
	find ./target -type f -executable \
		-path "*/release/*" -a ! -path "*/deps/*" -a -name "*app*" \
		-a -regextype sed ! -regex '.*-[a-f0-9]\{16\}.*' \
		-exec ls -sh {} \; -exec ldd {} \; -exec echo -e "------------------------" \;

.PHONY: strip
strip:
	find ./target -type f -executable \
		-path "*/release/*" -a ! -path "*/deps/*" -a -name "*app*" \
		-a -regextype sed ! -regex '.*-[a-f0-9]\{16\}.*' \
		-exec strip {} \;

.PHONY: flags
flags:
	@echo "---=== MAKE FLAGS ===---"
	@echo target_dir: $(target_dir)
	@echo TARGET: $(CARGO_BUILD_TARGET)
	@echo CARGO_ARGS: $(CARGO_ARGS)
	@echo RUSTFLAGS: $(RUSTFLAGS)
	@echo ALL: $(ALL)
	@echo "------------------------"
