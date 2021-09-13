
all: static-hello-world

.PHONY: static-hello-world

static-hello-world:
	# https://www.gnu.org/software/make/manual/html_node/MAKE-Variable.html
	cd static-hello-world && $(MAKE)
