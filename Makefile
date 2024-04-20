SHELL := /bin/bash
DIRECTORY_LIST := $(shell find . -name Cargo.toml -printf '%h\n' | sort -u)

# Runs cargo test on all sub-workspaces.
.PHONY: test
test: 
	for f in $(DIRECTORY_LIST); do \
		echo "Running cargo test in $$f"; \
  	pushd $$f > /dev/null;\
  	cargo test;\
  	popd > /dev/null;\
	done;