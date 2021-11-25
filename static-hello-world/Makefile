
all: c go rust zig 

.PHONY: c go rust zig clean

builddir:
	rm -rf ./build
	mkdir -p ./build

c: | builddir
	cd C && $(MAKE)
	cp C/static_hello_world_musl ./build/linux_c_hello_world_musl.elf

go: | builddir
	cd Go && $(MAKE)

rust: | builddir
	cd Rust && $(MAKE)

zig: | builddir
	cd Zig && $(MAKE)
	cp Zig/hello_world ./build/linux_zig_hello_world.elf

clean:
	cd C && $(MAKE) clean
	cd Go && $(MAKE) clean
	cd Rust && $(MAKE) clean
	cd Zig && $(MAKE) clean
