.PHONY: build test lint localnet clean

build:
	anchor build

test:
	anchor test

lint:
	cargo clippy --all-targets -- -D warnings

localnet:
	solana-test-validator --reset --quiet & \
	sleep 5 && \
	anchor deploy && \
	pkill -f solana-test-validator

clean:
	cargo clean
	rm -rf .anchor
	rm -rf target
	rm -rf node_modules
	rm -rf Cargo.lock

install:
	yarn install