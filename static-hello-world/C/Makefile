SOURCE=hello_world.c
MUSL_GCC=/usr/local/musl/bin/musl-gcc

static-hello-world: static_hello_world_libc static_hello_world_musl

static_hello_world_libc: $(SOURCE)
	gcc -static -o $@ $+

static_hello_world_musl: $(SOURCE)
	$(MUSL_GCC) -static -o $@ $+

.PHONY: clean

clean:
	rm -f static_hello_world_libc
	rm -f static_hello_world_musl
