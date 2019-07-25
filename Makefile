all: build

run: 
	cargo run --features use-sfml

build: 
	cargo build --features use-sfml

test: 
	cargo test --features use-sfml

link: 
	@exec &>/dev/null; \
	pushd target/debug && ln -s ../../cfg && ln -s ../../assets; \
	pushd deps && ln -s ../../../cfg && ln -s ../../../assets && ln -s ../../../test_resources; \
	popd 
