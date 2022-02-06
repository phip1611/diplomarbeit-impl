# Builds my runtime environment, the microkernel, and relevant userland components (Apps to run).
#
# "$ make -j 8"
# "$ make run" (starts QEMU)

BUILD_DIR=build
# needs absolute paths!
MUSL_BUILD_DIR=$(PWD)/$(BUILD_DIR)/.musl
MUSL_GCC_DIR=$(MUSL_BUILD_DIR)/bin

PARALLEL_CARGO_RUSTUP_HACK=$(BUILD_DIR)/.cargo_rustup_check

# "make" builds everything
# userland tarball itself depends on "runtime_environment static_foreign_apps"
all: microkernel roottask userland_tarball

$(BUILD_DIR):
	mkdir -p $@

check_tooling:
	.build_helpers/check_tooling.sh

# Hedron Microhypervisor/Microkernel
microkernel: | $(BUILD_DIR) check_tooling
    # To execute all in the same sub process I need to
    # concat multiple commands with ";" and remove newlines (\)
    # https://stackoverflow.com/questions/1789594/how-do-i-write-the-cd-command-in-a-makefile
	cd "./thesis-hedron-fork"; \
	mkdir -p "build"; \
	cd "build"; \
	cmake -DBUILD_TESTING=OFF -DCMAKE_BUILD_TYPE=Release ..; \
	$(MAKE); \
	cp "src/hypervisor.elf32" "../../$(BUILD_DIR)/hedron.elf32"

# All artifacts of the Runtime Environment
runtime_environment: | $(BUILD_DIR) cargo_rustup_check check_tooling
	cd "runtime-environment" && $(MAKE)
	cp "runtime-environment/ws/roottask-bin/target/x86_64-unknown-hedron/release/roottask-bin" "$(BUILD_DIR)"
	cp "runtime-environment/ws/helloworld-bin/target/x86_64-unknown-hedron/release/helloworld-bin" "$(BUILD_DIR)"
# TODO file server
# cp "runtime-environment/ws/fileserver-bin/target/x86_64-unknown-hedron/release/fileserver-bin" "$(BUILD_DIR)"

# Foreign Apps and Hybrid Foreign Apps in several languages (C, Rust).
static_foreign_apps: | $(BUILD_DIR) libc_musl cargo_rustup_check check_tooling
	# bind environment var MUSL_GCC_DIR
	cd "static-foreign-apps" && MUSL_GCC_DIR="$(MUSL_GCC_DIR)" $(MAKE)
	find "static-foreign-apps/build/" -type f -exec cp "{}" "$(BUILD_DIR)" \;

# Installs musl locally in the directory but doesn't require to have it installed
# in the system. Executable is in "./libc-musl/obj/musl-gcc"
libc_musl: | check_tooling
    # Install it locally with absolute paths. This is important so that during runtime
    # the musl compiler finds all header files etc.
	cd "libc-musl" && ./configure "--prefix=$(MUSL_BUILD_DIR)"
	cd "libc-musl" && $(MAKE)
	mkdir -p "$(BUILD_DIR)/.musl"
	cd "libc-musl" && $(MAKE) install
	echo "Installed musl to: $(MUSL_BUILD_DIR)"

roottask: | runtime_environment

# Creates a tarball with the whole userland the roottask should bootstrap.
# Currently this only works because each expected file is hard-coded into
# the roottask. Basically this contains all relevant files from
userland_tarball: | runtime_environment static_foreign_apps
	# shell script, because I don't know how
	# to nicely solve it in Makefile
	.build_helpers/build_tarball.sh

# Starts QEMU with Hedron and my runtime environment
# Doesn't depend on "all", because usage is intended to be: "make -j 8 && make run"
run:
	.build_helpers/run_qemu.sh

# Hack for build stability when using more than one job ($ make -j n). If multiple Rust builds (i.e. of runtime
# environment and static foreign apps) start in parallel, they might use "rustup" simultaneously to install
# new toolchains. This doesn't work as rustup only wants to be used by one component at a time.
cargo_rustup_check: $(PARALLEL_CARGO_RUSTUP_HACK)

# The marker file in "./build" enables that I can make sure cargo/rustup downloads all relevant
# additional targets/toolchains before the parallel build starts. Otherwise, multiple cargo
# instances will use rustup to install desired targets, which leads to build failures. Rustup
# can't cope with t hat.
$(PARALLEL_CARGO_RUSTUP_HACK):
	cd "runtime-environment/ws/libhedron/" && cargo check
	cd "static-foreign-apps/Rust/" && cargo check
	touch $@

# Prepares the files for the network-boot. This is special to my
# local setup on my developer machine, where remote computers are
# connected via LAN and load files via TFTP from my laptop.
networkboot:
# TODO

.PHONY: clean

clean:
	rm -rf $(BUILD_DIR)
	cd "runtime-environment" && $(MAKE) clean
	cd "static-foreign-apps" && $(MAKE) clean
	cd "thesis-hedron-fork/build" && $(MAKE) clean
