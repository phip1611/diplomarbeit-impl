
all: regular musl

regular:
	rustup target add x86_64-unknown-linux-gnu
	cargo build --target x86_64-unknown-linux-gnu

musl:
	rustup target add x86_64-unknown-linux-musl
	# Rust links this statically; can also be verified by using `ldd` command
	cargo build --target x86_64-unknown-linux-musl

.PHONY: clean

clean:
	cargo clean
