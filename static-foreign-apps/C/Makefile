CFLAGS=-O3 -march=broadwell -ggdb -static
MUSL_GCC=/usr/local/musl/bin/musl-gcc

all: static_hello_world static_matrix_mult static_dump_aux

static_hello_world: static_hello_world_libc static_hello_world_musl
static_matrix_mult: static_matrix_mult_libc static_matrix_mult_musl
static_dump_aux: static_dump_aux_libc static_dump_aux_musl

static_hello_world_libc: hello_world.c
	gcc $(CFLAGS) -o $@ $+

static_hello_world_musl: hello_world.c
	$(MUSL_GCC) $(CFLAGS) -o $@ $+

static_matrix_mult_libc: matrix_mult.c
	gcc $(CFLAGS) -o $@ $+

static_matrix_mult_musl: matrix_mult.c
	$(MUSL_GCC) $(CFLAGS) -o $@ $+

static_dump_aux_libc: dump_aux.c
	gcc $(CFLAGS) -o $@ $+

static_dump_aux_musl: dump_aux.c
	$(MUSL_GCC) $(CFLAGS) -o $@ $+


.PHONY: clean

clean:
	rm -f static_hello_world_libc
	rm -f static_hello_world_musl
	rm -f static_dump_aux_libc
	rm -f static_dump_aux_musl
	rm -f static_matrix_mult_libc
	rm -f static_matrix_mult_musl
