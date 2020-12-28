check:
	SKIP_WASM_BUILD=1 cargo check

test:
	SKIP_WASM_BUILD=1 cargo test --all

run:
	SKIP_WASM_BUILD= cargo run --release -- --tmp --dev

build:
	WASM_BUILD_TOOLCHAIN=nightly-2020-10-05 cargo build --release

purge:
	SKIP_WASM_BUILD= cargo run -- purge-chain --dev -y

toolchain:
	./scripts/init.sh

restart: purge run

init: toolchain build