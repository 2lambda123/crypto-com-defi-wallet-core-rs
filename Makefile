
cpp_example = ./example/cpp-example

.PHONY: wasm android ios test clean cleanall mac_install cpp python-tests lint-fix lint-py wasm-tests wasm-ci-tests proto cpp-ci-tests cpp-tests

wasm:
	wasm-pack build --scope crypto-com bindings/wasm

android:
	./android_build.sh

ios:
	./ios_build.sh

test:
	cargo test

clean:
	rm -rf target bindings/android bindings/ios
	./clean.sh

cleanall:
	rm -rf target bindings/android bindings/ios
	rm -rf NDK
	./clean.sh


mac_install:
	cargo install uniffi_bindgen
	brew install ktlint
	brew install swiftformat

prepare_cpp:
	cargo build --package defi-wallet-core-cpp --release

cpp: prepare_cpp
	cp $(shell find ./target/release -name "libcxxbridge1.a") $(cpp_example)
	cp ./target/release/libdefi_wallet_core_cpp.* $(cpp_example)
	cp ./target/cxxbridge/rust/cxx.h $(cpp_example)
	cp ./target/cxxbridge/defi-wallet-core-cpp/src/*.h $(cpp_example)
	cp ./target/cxxbridge/defi-wallet-core-cpp/src/*.cc $(cpp_example)
	source ./scripts/.env && cd $(cpp_example) && make

cppx86_64:
	cargo build --release --target x86_64-apple-darwin
	cp ./target/x86_64-apple-darwin/release/libdefi_wallet_core_cpp.a $(cpp_example)
	cp ./target/cxxbridge/rust/cxx.h $(cpp_example)
	cp ./target/cxxbridge/defi-wallet-core-cpp/src/*.h $(cpp_example)
	cp ./target/cxxbridge/defi-wallet-core-cpp/src/*.cc $(cpp_example)
	cd $(cpp_example) && make x86_64


proto:
	cd proto-build && cargo run

python-tests:
	@nix-shell ./integration_tests/shell.nix --run scripts/python-tests

wasm-ci-tests:
	export WASM_BINDGEN_TEST_TIMEOUT=60
	@nix-shell ./integration_tests/shell.nix --run "scripts/start-all"
	cd bindings/wasm/ && wasm-pack test --chrome --headless
	@nix-shell ./integration_tests/shell.nix --run "scripts/stop-all"

wasm-tests:
	./scripts/wasm-tests

cpp-ci-tests:
	@nix-shell ./integration_tests/shell.nix --run "scripts/start-all"
	make cpp
	@nix-shell ./integration_tests/shell.nix --run "scripts/stop-all"

cpp-tests:
	./scripts/cpp-tests

lint-py:
	flake8 --show-source --count --statistics \
          --format="::error file=%(path)s,line=%(row)d,col=%(col)d::%(path)s:%(row)d:%(col)d: %(code)s %(text)s" \

lint-nix:
	find . -name "*.nix" ! -path './example/*' | xargs nixpkgs-fmt --check
