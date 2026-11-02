ROOT     := $(abspath $(dir $(lastword $(MAKEFILE_LIST))))
PROGRAMS := $(notdir $(wildcard $(ROOT)/programs/*))
CBS      ?= $(ROOT)/.bin/cargo-build-sbf
ARCH     ?= v3

MAKEFLAGS += --keep-going

build = $(CBS) --lto --manifest-path $(ROOT)/programs/$(2)/Cargo.toml --arch $(ARCH) $(3) \
	--sbf-out-dir $(ROOT)/out/$(1) -- --target-dir $(ROOT)/target/$(1)

.PHONY: build_sbf build_upstream test clean

build_sbf: $(PROGRAMS:%=sbf/%)

build_upstream: $(PROGRAMS:%=upstream/%)

sbf/%:
	$(call build,sbf,$*)

upstream/%:
	$(call build,upstream,$*,--toolchain upstream)

test:
	-@$(MAKE) --no-print-directory build_sbf build_upstream
	cargo run --quiet --manifest-path $(ROOT)/compare/Cargo.toml --target-dir $(ROOT)/target/compare

clean:
	rm -rf $(ROOT)/target $(ROOT)/out
