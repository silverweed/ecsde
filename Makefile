all: build

build:
	cargo build --features gfx_sdl

test:
	cargo test --features gfx_sdl -- --test-threads=1
