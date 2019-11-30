SHELL = bash
all: build

run:
	pushd ecs_game && cargo build && popd && cargo run

build:
	pushd ecs_game && cargo build && popd && cargo build

release:
	pushd ecs_game && cargo build --release && popd && cargo build --release

run_release:
	pushd ecs_game && cargo build --release && popd && cargo run --release

test:
	cargo test

clippy:
	cargo clippy

link:
	@exec &>/dev/null; \
	pushd target/debug && ln -s ../../cfg && ln -s ../../assets; \
	pushd deps && ln -s ../../cfg && ln -s ../../assets && ln -s ../../test_resources; \
	popd

link_release:
	@exec &>/dev/null; \
	pushd target/release && ln -s ../../cfg && ln -s ../../assets; \
	pushd deps && ln -s ../../cfg && ln -s ../../assets && ln -s ../../test_resources; \
	popd
