SHELL = bash
all: build

run:
	cargo run 

build:
	cargo build 

release:
	cargo build --release 

run_release:
	cargo run --release

test:
	cargo test --features 

test_release:
	cargo test --release 

clippy:
	cargo clippy 

link:
	@exec &>/dev/null; \
	pushd target/debug && ln -s ../../../cfg && ln -s ../../../assets; \
	pushd deps && ln -s ../../../../cfg && ln -s ../../../../assets && ln -s ../../../../test_resources; \
	popd

link_release:
	@exec &>/dev/null; \
	pushd target/release && ln -s ../../../cfg && ln -s ../../../assets; \
	pushd deps && ln -s ../../../../cfg && ln -s ../../../../assets && ln -s ../../../../test_resources; \
	popd
