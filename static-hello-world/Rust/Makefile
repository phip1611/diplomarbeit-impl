
all: regular musl
	rustup target add x86_64-unknown-linux-gnu
	rustup target add x86_64-unknown-linux-musl

regular:
	cargo build --target x86_64-unknown-linux-gnu

musl:
	# Rust links this statically; can also be verified by using `ldd` command
	cargo build --target x86_64-unknown-linux-musl

.PHONY: clean

clean:
	cargo clean
