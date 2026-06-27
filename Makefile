

all: fmt wasm native

release: fmt wasm-release native-release

fmt:
	cargo fmt

wasm:
	cargo build --target wasm32-unknown-unknown
	wasm-bindgen target/wasm32-unknown-unknown/debug/mikumikutactics.wasm --out-dir web/pkg --target web
	cp -r assets web/assets
	cp index.html web/index.html

native:
	cargo build

wasm-release:
	cargo build --target wasm32-unknown-unknown --release
	wasm-bindgen target/wasm32-unknown-unknown/release/mikumikutactics.wasm --out-dir web/pkg --target web
	cp -r assets web/assets
	cp index.html web/index.html

native-release:
	cargo build --release	

clean:
	rm -rf target/ web/

doc:
	cargo doc
	cargo doc --target wasm32-unknown-unknown	