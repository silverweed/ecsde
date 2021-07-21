SHELL = bash
all: build

c:
	cargo check

r: run

run: build
	cargo run

build:
	cargo build --all

rel:
	cargo build --all --release

run_rel: rel
	cargo run --release

w:
	cargo watch -w inle -w ecs_game/src -x 'build --all'

tags:
	rusty-tags vi

fmt:
	cargo fmt

test:
	cargo test

clippy:
	cargo clippy

link:
	@exec &>/dev/null; \
	pushd target/debug && ln -s ../../cfg && ln -s ../../assets; \
	pushd deps && ln -s ../../../cfg && ln -s ../../../assets && ln -s ../../../test_resources; \
	popd

link_release:
	@exec &>/dev/null; \
	pushd target/release && ln -s ../../cfg && ln -s ../../assets; \
	pushd deps && ln -s ../../../cfg && ln -s ../../../assets && ln -s ../../../test_resources; \
	popd
