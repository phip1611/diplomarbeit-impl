
all: c go rust zig

.PHONY: c go rust zig clean

c:
	cd C && $(MAKE)

go:
	cd Go && $(MAKE)

rust:
	cd Rust && $(MAKE)

zig:
	cd Zig && $(MAKE)

clean:
	cd C && $(MAKE) clean
	cd Go && $(MAKE) clean
	cd Rust && $(MAKE) clean
	cd Zig && $(MAKE) clean
