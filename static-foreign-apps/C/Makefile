CFLAGS=-O3 -march=broadwell -ggdb -static

# Comes as ENV var from main Make file
MUSL_GCC=$(MUSL_GCC_DIR)/musl-gcc

all: static_hello_world static_matrix_mult static_dump_aux

static_hello_world: static_hello_world_glibc static_hello_world_musl
static_matrix_mult: static_matrix_mult_glibc static_matrix_mult_musl
static_dump_aux: static_dump_aux_glibc static_dump_aux_musl

static_hello_world_glibc: hello_world.c
	gcc $(CFLAGS) -o $@ $+

static_hello_world_musl: hello_world.c
	$(MUSL_GCC) $(CFLAGS) -o $@ $+

static_matrix_mult_glibc: matrix_mult.c
	gcc $(CFLAGS) -o $@ $+

static_matrix_mult_musl: matrix_mult.c
	$(MUSL_GCC) $(CFLAGS) -o $@ $+

static_dump_aux_glibc: dump_aux.c
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
