# By default I only build C and Rust so that others that check out this repository
# can bootstrap everything fast without installing and setting up a ton of stuff.
all: c rust

all_extra: all go zig

.PHONY: c go rust zig clean

builddir:
	rm -rf ./build
	mkdir -p ./build

c: | builddir
	cd C && $(MAKE)
	cp C/static_hello_world_musl ./build/linux_c_hello_world_musl.elf
	cp C/static_dump_aux_musl ./build/linux_c_dump_aux_musl.elf
	cp C/static_matrix_mult_musl ./build/linux_c_matrix_mult_musl.elf

go: | builddir
	cd Go && $(MAKE)

rust: | builddir
	cd Rust && $(MAKE)
	cp Rust/target/x86_64-unknown-linux-musl/debug/hello_world           ./build/linux_rust_hello_world_musl--debug.elf
	cp Rust/target/x86_64-unknown-linux-musl/debug/hello_world_hybrid    ./build/linux_rust_hello_world_hybrid_musl--debug.elf
	cp Rust/target/x86_64-unknown-linux-musl/debug/hybrid_benchmark      ./build/linux_rust_hybrid_benchmark--debug.elf
	cp Rust/target/x86_64-unknown-linux-musl/release/hello_world         ./build/linux_rust_hello_world_musl--release.elf
	cp Rust/target/x86_64-unknown-linux-musl/release/hello_world_hybrid  ./build/linux_rust_hello_world_hybrid_musl--release.elf
	cp Rust/target/x86_64-unknown-linux-musl/release/hybrid_benchmark    ./build/linux_rust_hybrid_benchmark--release.elf

zig: | builddir
	cd Zig && $(MAKE)
	cp Zig/hello_world ./build/linux_zig_hello_world.elf

clean:
	rm -rf ./build
	cd C && $(MAKE) clean
	cd Go && $(MAKE) clean
	cd Rust && $(MAKE) clean
	cd Zig && $(MAKE) clean
