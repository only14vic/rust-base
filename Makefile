-include .env

SHELL = sh
.DEFAULT_GOAL = flags

ifndef VERBOSE
.SILENT:
endif

make = make --no-print-directory

ifdef env
ifneq ($(env),)
	APP_ENV = $(env)
endif
endif

DESTDIR = $(PWD)
CARGO_ARGS =
CLANG_VERSION = $(shell clang --version | grep -o "version [0-9]\+")
RUSTFLAGS = -Ctarget-cpu=native \
			-Clink-arg=-fuse-ld=lld

ifeq ($(CLANG_VERSION),version 20)
	RUSTFLAGS += -Clinker=clang \
				-Clinker-plugin-lto \
				-Clink-arg=-lc
endif

MAKE_CC = cc
MAKE_CFLAGS = -std=gnu18 -Wall -Wextra -L$(TARGET_DIR) -fPIC -Os -g -march=native -flto=2 -fno-fat-lto-objects -fuse-linker-plugin

ifeq ($(APP_DEBUG),1)
	debug = 1
endif
ifeq ($(debug),)
	CARGO_ARGS += --release
endif

ifneq ($(static),)
	CARGO_BUILD_TARGET = x86_64-unknown-linux-musl
	RUSTFLAGS += -Ctarget-feature=+crt-static
	MAKE_CFLAGS += -static -static-pie
else
	#CARGO_BUILD_TARGET = x86_64-unknown-linux-gnu
	MAKE_CFLAGS += -pipe -Wl,--gc-sections,-z,relro,-z,now,-rpath='$$ORIGIN',-rpath='$$ORIGIN/lib',-rpath='$$ORIGIN/../lib',-rpath='$(TARGET_DIR)'
ifneq ($(dynamic),)
	RUSTFLAGS += -Cprefer-dynamic \
				-Clink-args=-Wl,-rpath,$ORIGIN,-rpath,$ORIGIN/lib,-rpath,$ORIGIN/../,-rpath,$ORIGIN/../lib,-rpath,$ORIGIN/../../,-rpath,$ORIGIN/../../lib,-rpath,$ORIGIN/../../../,-rpath,$ORIGIN/../../../lib
endif
endif

ifneq ($(no_std),)
	RUSTFLAGS += -Cpanic=abort
	CARGO_ARGS += --no-default-features
endif
ifdef args
	CARGO_ARGS += -- $(args)
endif

TARGET_DIR = $(shell cargo metadata --format-version 1|jq ".[\"target_directory\"]"|tr -d '"')/$(CARGO_BUILD_TARGET)/release
MAKE_AOBJS = $(wildcard $(TARGET_DIR)/*.a)
MAKE_OBJS = $(MAKE_AOBJS:.a=.o)

ALL =
TESTS =

-include */Makefile
-include docker/*/Makefile
-include crates/*/Makefile

ALL += info
TESTS += info

#################

all: clean check
	echo $(ALL) | sed 's/[,\ ]\+$$//g' | sed 's/\s*,\+\s*/\n/g' | xargs -I '{}' sh -c "$(make) {}"

tests: clean check
	echo $(TESTS) | sed 's/[,\ ]\+$$//g' | sed 's/\s*,\+\s*/\n/g' | xargs -I '{}' sh -c "$(make) {}"

clean:
	find ./target \
		-path "./target/*" -name "*app*" -type f -executable -delete

check:
	$(eval RUSTFLAGS=)
	cargo check --workspace --no-default-features --exclude app
	cargo check --workspace
	cargo clippy --no-deps --fix --allow-dirty --allow-staged
	rustup run nightly rustfmt --check crates/*/src/**/*.rs

.PHONY: info
info:
	find ./target -type f \
		-path "*/release/*" -a ! -path "*/deps/*" -a ! -path "*/build/*"  \
		-a \( -executable -o -name "*.a" -o -name "*.so" \) \
		-a -regextype sed ! -regex '.*-[a-f0-9]\{16\}.*' \
		-exec ls -sh {} \; -exec ldd {} 2>/dev/null \; -exec echo -e "------------------------" \;

.PHONY: doc
doc:
	$(eval RUSTFLAGS=)
	cargo doc --no-deps --offline $(a)

.PHONY: strip
strip:
	find ./target -type f -executable \
		-path "*/release/*" -a ! -path "*/deps/*" -a -name "*app*" \
		-a -regextype sed ! -regex '.*-[a-f0-9]\{16\}.*' \
		-exec strip {} \;

.PHONY: flags
flags:
	@echo "---=== MAKE FLAGS ===---"
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

objs: $(MAKE_OBJS)

%.o: %.a
	ar rcs $@ $<

_confirm:
	read -r -p "Continue? [yes/No] " input; test 'yes' = $$input

#################

.PHONY: git-hooks-install
git-hooks-install:
	echo -e "#!/bin/sh \n test -t 1 && exec < /dev/tty \n eval make _git-\$$(basename "\$$0") \n" \
 | tee .git/hooks/pre-commit .git/hooks/post-commit .git/hooks/pre-push .git/hooks/post-checkout > /dev/null
	chmod a+x .git/hooks/*

_git-pre-commit:
	#git submodule foreach --recursive "APP_ENV= git commit || true"
	#git update-index --add $$(git submodule summary|grep '^*'|cut -d' ' -f2) || true

_git-post-commit:
	git diff-index --quiet --cached HEAD -- || git commit --amend --no-verify
	git status -s

_git-pre-push: check
	echo "Git status check:"
	git status -s
	test "$$(git status -s|wc -l)" = "0"
	#git submodule foreach --recursive git push

_git-post-checkout:
	test -f .gitmodules && grep -Po '(?<=\[submodule ).*(?=\])|(?<=branch = ).*' .gitmodules | paste -d ' ' - - |  xargs -n 2 sh -c 'git -C $$0 checkout -q $$1'

#################
