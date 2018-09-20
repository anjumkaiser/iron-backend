#!/bin/sh

export PATH=~/.cargo/bin:$PATH

if [ -x .cargo/bin/rustup ]; then
	curl https://sh.rustup.rs -sSf | sh -s -- -y
	cargo --version || exit $?
fi

echo "Running tests ..."
cargo test || exit $?


echo "Running release build ..."
cargo build --release || exit $?

