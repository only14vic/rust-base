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

CARGO_ARGS += $(args)

ALL =

-include crates/app-*/Makefile

all:
	echo $(ALL) | sed 's/[,\ ]\+$$//g' | sed 's/\s*,\+\s*/\n/g' | xargs -I '{}' sh -c "$(make) {}"

clean:
	find ./target \
		-a -path "./target/*" -a -name "app*" \
			-type f -executable -delete

.PHONY: info
info:
	find ./target -type f -executable -path "*/release/*" -name "app*" ! -regex '.*-[0-9].*' \
		-exec ls -sh {} \; -exec ldd {} \; -exec echo -e "------------------------" \;

.PHONY: flags
flags:
	@echo "---=== MAKE FLAGS ===---"
	@echo TARGET: $(CARGO_BUILD_TARGET)
	@echo CARGO_ARGS: $(CARGO_ARGS)
	@echo RUSTFLAGS: $(RUSTFLAGS)
	@echo ALL: $(ALL)
	@echo "------------------------"
