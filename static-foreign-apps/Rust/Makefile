
all: regular musl

# dynamically linked with glibc
regular:
	rustup target add x86_64-unknown-linux-gnu
	cargo build --target x86_64-unknown-linux-gnu
	cargo build --release --target x86_64-unknown-linux-musl

# statically linked with musl
musl:
	rustup target add x86_64-unknown-linux-musl
	# Rust links this statically; can also be verified by using `ldd` command
	cargo build --target x86_64-unknown-linux-musl
	cargo build --release --target x86_64-unknown-linux-musl

.PHONY: clean

clean:
	cargo clean
