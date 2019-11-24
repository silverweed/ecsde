all: build

run:
	LD_LIBRARY_PATH=ecs_game/target/debug cargo run

build:
	cargo build

release:
	cargo build --release

run_release:
	LD_LIBRARY_PATH=ecs_game/target/release cargo run --release

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
