SHELL = bash
all: build

run: build
	cargo run

build:
	cargo build --all

release:
	cargo build --all --release

run_release: release
	cargo run --release

watch:
	cargo watch -w ecs_engine/src -w ecs_game/src -x 'build --all'

tags:
	rusty-tags vi

fmt:
	find ecs_engine/src ecs_game/src ecs_runner/src -type f -exec rustfmt {} +

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
