all: build

run: 
	cargo run --features use-sfml

build: 
	cargo build --features use-sfml

release: 
	cargo build --release --features use-sfml

run_release: 
	cargo run --release --features use-sfml

test: 
	cargo test --features use-sfml

clippy:
	cargo clippy --features use-sfml

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
