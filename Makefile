.PHONY: all build test clean run-node install-deps help

all: build

help:
	@echo "Polkadot Signal - Build Commands"
	@echo "================================"
	@echo "make build          - Build all components"
	@echo "make test           - Run all tests"
	@echo "make clean          - Clean build artifacts"
	@echo "make run-node       - Run development node"
	@echo "make install-deps   - Install dependencies"
	@echo "make build-client   - Build TypeScript client"
	@echo "make build-protocol - Build protocol library"
	@echo "make build-pallets  - Build Substrate pallets"
	@echo "make check          - Check code without building"
	@echo "make clippy         - Run clippy linter"
	@echo "make fmt            - Format code"
	@echo "make docs           - Generate documentation"

build:
	cargo build --release

build-debug:
	cargo build

build-pallets:
	cargo build -p pallet-signal-keys -p pallet-signal-groups -p pallet-message-queue

build-protocol:
	cargo build -p signal-protocol

build-p2p:
	cargo build -p p2p-messaging

build-storage:
	cargo build -p ipfs-storage

build-node:
	cargo build -p polkadot-signal-node --release

build-client:
	cd client && npm install && npm run build

test:
	cargo test --all

test-pallets:
	cargo test -p pallet-signal-keys -p pallet-signal-groups -p pallet-message-queue

test-protocol:
	cargo test -p signal-protocol

test-p2p:
	cargo test -p p2p-messaging

test-storage:
	cargo test -p ipfs-storage

check:
	cargo check --all

clippy:
	cargo clippy --all -- -D warnings

fmt:
	cargo fmt --all

clean:
	cargo clean
	cd client && rm -rf node_modules dist

run-node:
	cargo run --release -p polkadot-signal-node -- --dev --tmp

run-node-alice:
	cargo run --release -p polkadot-signal-node -- \
		--chain dev \
		--alice \
		--tmp \
		--port 30333 \
		--rpc-port 9944

run-node-bob:
	cargo run --release -p polkadot-signal-node -- \
		--chain dev \
		--bob \
		--tmp \
		--port 30334 \
		--rpc-port 9945 \
		--bootnodes /ip4/127.0.0.1/tcp/30333/p2p/ALICE_PEER_ID

install-deps:
	rustup update stable
	rustup target add wasm32-unknown-unknown --toolchain stable
	cd client && npm install

docs:
	cargo doc --no-deps --open

benchmark:
	cargo run --release -p polkadot-signal-node -- benchmark \
		--chain dev \
		--execution wasm \
		--wasm-execution compiled \
		--pallet pallet_signal_keys \
		--extrinsic '*' \
		--steps 20 \
		--repeat 10 \
		--output pallets/signal-keys/src/weights.rs

generate-weights:
	cargo run --release --features runtime-benchmarks -- benchmark \
		--chain dev \
		--execution wasm \
		--wasm-execution compiled \
		--pallet pallet_signal_keys \
		--extrinsic '*' \
		--steps 20 \
		--repeat 10 \
		--output pallets/signal-keys/src/weights.rs

ipfs-start:
	ipfs daemon --init

docker-build:
	docker build -t polkadot-signal:latest .

docker-run:
	docker run -p 9944:9944 -p 30333:30333 polkadot-signal:latest

wasm:
	cargo build --target wasm32-unknown-unknown --release

example-demo:
	cargo run --release -p polkadot-signal-examples --bin demo
