
all: static-hello-world

.PHONY: static-hello-world clean

static-hello-world:
	# https://www.gnu.org/software/make/manual/html_node/MAKE-Variable.html
	cd static-hello-world && $(MAKE)

clean:
	cd static-hello-world && $(MAKE) clean
