unexport CFLAGS

-include .env

ifneq ($(env_file),)
include $(env_file)
endif

# Fix env value with quotes
ifdef DATABASE_URL
DATABASE_URL:=$(subst ',,$(subst ",,$(DATABASE_URL)))
endif

SHELL = sh
.DEFAULT_GOAL = flags

.EXPORT_ALL_VARIABLES:

ifndef VERBOSE
.SILENT:
endif

make = make --no-print-directory
CARGO_ARGS =
RUSTFLAGS =
DESTDIR = $(PWD)

ifdef env
ifneq ($(env),)
	APP_ENV = $(env)
endif
endif

ifeq ($(APP_DEBUG),1)
	debug = 1
endif
ifeq ($(debug),)
	CARGO_ARGS += --release
endif

ifneq ($(static),)
	CARGO_BUILD_TARGET = x86_64-unknown-linux-musl
	export OPENSSL_DIR = /opt/openssl-static
endif

ifneq ($(aarch64),)
	CARGO_BUILD_TARGET = aarch64-unknown-linux-gnu
	RUSTFLAGS = -Clinker=aarch64-linux-gnu-gcc \
				-Clink-arg=-fuse-ld=lld
	export OPENSSL_DIR = /opt/openssl-aarch64
else
	RUSTFLAGS += -Ctarget-cpu=native \
				 -Clink-arg=-fuse-ld=lld
	CLANG_VERSION = $(shell clang --version | grep -o "version [0-9]\+")
ifeq ($(CLANG_VERSION),version 20)
	RUSTFLAGS += -Clinker=clang \
				-Clinker-plugin-lto \
				-Clink-arg=-lc
endif
endif

TARGET_DIR = $(shell cargo metadata --format-version 1|jq ".[\"target_directory\"]"|tr -d '"')/$(CARGO_BUILD_TARGET)

ifeq ($(debug),)
	TARGET_DIR := $(TARGET_DIR)/release
else
	TARGET_DIR := $(TARGET_DIR)/debug
endif

MAKE_CC = clang
MAKE_CFLAGS = -std=gnu23 -Wall -Wextra -L$(TARGET_DIR) -fPIC -Os -g -march=native -fno-fat-lto-objects -pipe \
			-Wl,--gc-sections,-z,relro,-z,now,-rpath='$$ORIGIN',-rpath='$$ORIGIN/lib',-rpath='$$ORIGIN/../lib',-rpath='$(TARGET_DIR)'

ifneq ($(static),)
	RUSTFLAGS += -Ctarget-feature=+crt-static
else
ifneq ($(dynamic),)
	RUSTFLAGS += -Cprefer-dynamic \
				-Clink-args=-Wl,-rpath,$ORIGIN,-rpath,$ORIGIN/lib,-rpath,$ORIGIN/../,-rpath,$ORIGIN/../lib,-rpath,$ORIGIN/../../,-rpath,$ORIGIN/../../lib,-rpath,$ORIGIN/../../../,-rpath,$ORIGIN/../../../lib
endif
endif

ifneq ($(no_std),)
	RUSTFLAGS += -Cpanic=abort
	CARGO_ARGS += --no-default-features
endif
ifneq ($(features),)
	CARGO_ARGS += --no-default-features --features "$(features)"
endif
ifdef args
	CARGO_ARGS += -- $(args)
endif

MAKE_OOBJS = $(wildcard $(TARGET_DIR)/*.o)
MAKE_AOBJS = $(MAKE_OOBJS:.o=.a)

ALL =
TESTS =

-include */Makefile
-include docker/*/Makefile
-include crates/*/Makefile

ALL += info
TESTS += info

#################

all: clean check
	# sh -c 'exit 255' sh - aborts process on first error
	echo $(ALL) | sed 's/[,\ ]\+$$//g' | sed 's/\s*,\+\s*/\n/g' | xargs -I '{}' sh -c '$(make) {} || exit 255' sh

tests: clean check
	echo $(TESTS) | sed 's/[,\ ]\+$$//g' | sed 's/\s*,\+\s*/\n/g' | xargs -I '{}' sh -c '$(make) {} || exit 255' sh

tests-%:
	APP_ENV=test cargo test -p $* --tests $(CARGO_ARGS) -- --nocapture

examples-%:
	APP_ENV=test cargo test -p $* --examples $(CARGO_ARGS)

build-%: flags
ifneq ($(static),)
	cargo rustc -p $* --lib --crate-type=staticlib $(CARGO_ARGS)
	cargo build -p $* --bins $(CARGO_ARGS)
else
	cargo rustc -p $* --lib --crate-type=cdylib $(CARGO_ARGS)
	cargo build -p $* --bins $(CARGO_ARGS)
endif

run-%: flags
	cargo run -p $* $(CARGO_ARGS)

clean:
	find target -maxdepth 3 -type f  -executable -regextype sed -regex ".*/\(release\|debug\)/[^\/]\+" -delete || true

check:
	cargo check --workspace --no-default-features --exclude app
	cargo check --workspace --all-features
	cargo clippy --workspace --no-deps --fix --allow-dirty --allow-staged
	find crates -type f -name "*.rs" | xargs rustup run nightly rustfmt --check

.PHONY: info
info:
	find ./target -maxdepth 3 -type f \
		-executable -regextype sed -regex ".*/\(release\|debug\)/[^\/]\+" \
		-exec ls -sh {} \; -exec ldd {} 2>/dev/null \; -exec echo -e "------------------------" \;

gdb_args = --readnow -iex "set auto-load safe-path /" -x .gdb_local \
		--directory "$$(ls -1d ~/.cargo/registry/src/* ~/.rustup/toolchains/*/lib/rustlib/src/rust/library | xargs echo | sed s/\ /:/g)"

.PHONY: gdb
gdb:
ifdef f
	rust-gdb $(gdb_args) --args $(f)
else
	rust-gdb $(gdb_args)
endif

.PHONY: doc
doc:
	$(eval RUSTFLAGS=)
	cargo doc --no-deps --offline $(a)

ifndef CARGO_WATCH_DELAY
CARGO_WATCH_DELAY=5
endif
CARGO_WATCH_ARGS = --delay $(CARGO_WATCH_DELAY) -B 1 --poll --why --no-gitignore -i bindings.rs \
		-w Cargo.toml -w .cargo -w crates -w assets -w .env -w config

.PHONY: watch
watch: flags
	echo CARGO_WATCH_ARGS: $(CARGO_WATCH_ARGS)
	cargo watch $(CARGO_WATCH_ARGS) -s ". ./.env; cargo run $(CARGO_ARGS)"

watch-%: flags
	echo CARGO_WATCH_ARGS: $(CARGO_WATCH_ARGS)
	cargo watch $(CARGO_WATCH_ARGS) -s ". ./.env; cargo run -p $* $(CARGO_ARGS)"

.PHONY: strip
strip:
	find ./target -maxdepth 3 -type f -executable \
		-executable -regextype sed -regex ".*/release/[^\/]\+" \
		-exec strip {} \; || true

.PHONY: flags
flags:
	@echo "---=== MAKE FLAGS ===---"
	@echo DEBUG: $(debug)
	@echo DESTDIR: $(DESTDIR)
	@echo TARGET_DIR: $(TARGET_DIR)
	@echo TARGET: $(CARGO_BUILD_TARGET)
	@echo CARGO_ARGS: $(CARGO_ARGS)
	@echo RUSTFLAGS: $(RUSTFLAGS)
	@echo MAKE_CFLAGS: $(MAKE_CFLAGS)
	@echo CLANG_VERSION: $(CLANG_VERSION)
	@echo TESTS: $(TESTS)
	@echo ALL: $(ALL)
	@echo "------------------------"

.PHONY: env
env:
	env|sort

objs: $(MAKE_AOBJS)

%.a: %.o
	ar rcs $@ $<

_confirm:
	read -r -p "Continue? [yes/No] " input; test 'yes' = $$input

#################

.PHONY: git-update
git-update:
	git submodule update --remote --rebase --recursive
	git submodule foreach git branch

.PHONY: git-hooks-install
git-hooks-install:
	echo -e "#!/bin/sh \n test -t 1 && exec < /dev/tty \n eval make _git-\$$(basename "\$$0") \n" | tee .git/hooks/pre-commit .git/hooks/post-commit .git/hooks/pre-push .git/hooks/post-checkout > /dev/null
	chmod a+x .git/hooks/*

_git-pre-commit:
	#git submodule foreach --recursive "APP_ENV= git commit || true"
	#git update-index --add $$(git submodule summary|grep '^*'|cut -d' ' -f2) || true

_git-post-commit:
	git diff-index --quiet --cached HEAD -- || git commit --amend --no-verify
	git status -s

_git-pre-push: tests
	echo "Git status check:"
	git status -s
	test "$$(git status -s|wc -l)" = "0"
	#git submodule foreach --recursive git push

_git-post-checkout:
	test -f .gitmodules && grep -Po '(?<=\[submodule ).*(?=\])|(?<=branch = ).*' .gitmodules | paste -d ' ' - - |  xargs -n 2 sh -c 'git -C $$0 checkout -q $$1'

#################
