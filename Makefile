build:
		cargo build -p orion-client

bundle: build
		cp ./target/wasm32-wasi/debug/orion-client.wasm www/orion-client.wasm
