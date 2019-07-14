all: build

run:
	cargo run --features use-sfml

build:
	cargo build --features use-sfml

test:
	cargo test --features use-sfml
