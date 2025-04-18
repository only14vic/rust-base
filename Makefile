-include .env
export

SHELL = sh
.DEFAULT_GOAL = flags

ifndef VERBOSE
.SILENT:
endif

make = make --no-print-directory
CARGO_ARGS = --release
RUSTFLAGS = -Ctarget-cpu=native \
			-Clinker-plugin-lto \
			-Clink-arg=-fuse-ld=lld \
			-Clink-arg=-lc

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

CARGO_ARGS += $(args)

ALL =

-include app-*/Makefile

all:
	echo $(ALL) | sed 's/[,\ ]\+$$//g' | sed 's/\s*,\+\s*/\n/g' | xargs -I '{}' sh -c "$(make) {}"

.PHONY: flags
flags:
	@echo "---=== MAKE FLAGS ===---"
	@echo TARGET: $(CARGO_BUILD_TARGET)
	@echo CARGO_ARGS: $(CARGO_ARGS)
	@echo RUSTFLAGS: $(RUSTFLAGS)
	@echo ALL: $(ALL)
	@echo "------------------------"
