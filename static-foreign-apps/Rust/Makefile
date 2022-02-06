
all: regular musl

# dynamically linked with glibc
regular:
	cargo build --target x86_64-unknown-linux-gnu
	cargo build --release --target x86_64-unknown-linux-gnu

# statically linked with musl
musl:
	# Rust links this statically; can also be verified by using `ldd` command
	cargo build --target x86_64-unknown-linux-musl
	cargo build --release --target x86_64-unknown-linux-musl

.PHONY: clean

clean:
	cargo clean
