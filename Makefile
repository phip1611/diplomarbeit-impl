# Builds my runtime environment, the microkernel, and relevant userland components (Apps to run).

BUILD_DIR="./build"
# needs absolute paths!
MUSL_BUILD_DIR="$(PWD)/$(BUILD_DIR)/.musl"
MUSL_GCC="$(MUSL_BUILD_DIR)/bin/musl-gcc"

# "make" builds everything
all: microkernel runtime_environment static_foreign_apps userland_tarball

$(BUILD_DIR):
	mkdir -p $@

# Hedron Microhypervisor/Microkernel
microkernel: | $(BUILD_DIR)
    # To execute all in the same sub process I need to
    # concat multiple commands with ";" and remove newlines (\)
    # https://stackoverflow.com/questions/1789594/how-do-i-write-the-cd-command-in-a-makefile
	cd "./thesis-hedron-fork"; \
	mkdir -p "build"; \
	cd "build"; \
	cmake -DBUILD_TESTING=OFF -DCMAKE_BUILD_TYPE=Release ..; \
	$(MAKE) \
	cp "src/hypervisor.elf32" "../../$(BUILD_DIR)/hedron.elf32"

# All artifacts of the Runtime Environment
runtime_environment:| $(BUILD_DIR)
	cd "runtime-environment" && $(MAKE)
	cp "runtime-environment/ws/roottask-bin/target/x86_64-unknown-hedron/release/roottask-bin" "$(BUILD_DIR)"
	cp "runtime-environment/ws/helloworld-bin/target/x86_64-unknown-hedron/release/helloworld-bin" "$(BUILD_DIR)"
# TODO file server
# cp "runtime-environment/ws/fileserver-bin/target/x86_64-unknown-hedron/release/fileserver-bin" "$(BUILD_DIR)"

# Foreign Apps and Hybrid Foreign Apps in several languages (C, Rust).
static_foreign_apps:| $(BUILD_DIR) libc_musl
	# bind environment var MUSL_GCC
	cd "static-foreign-apps" && MUSL_GCC="$(MUSL_GCC)" $(MAKE)
	find "static-foreign-apps/build/" -type f -exec cp "{}" "$(BUILD_DIR)" \;

# Installs musl locally in the directory but doesn't require to have it installed
# in the system. Executable is in "./libc-musl/obj/musl-gcc"
libc_musl:
    # Install it locally with absolute paths. This is important so that during runtime
    # the musl compiler finds all header files etc.
	cd "libc-musl" && ./configure "--prefix=$(MUSL_BUILD_DIR)"
	cd "libc-musl" && $(MAKE)
	mkdir -p "$(BUILD_DIR)/.musl"
	cd "libc-musl" && $(MAKE) install
	echo "Installed musl to: $(MUSL_BUILD_DIR)"

# Creates a tarball with the whole userland the roottask should bootstrap.
# Currently this only works because each expected file is hard-coded into
# the roottask. Basically this contains all relevant files from
userland_tarball: | runtime_environment static_foreign_apps
	# shell script, because I don't know how
	# to nicely solve it in Makefile
	.build_helpers/build_tarball.sh

# Starts QEMU with Hedron and my runtime environment
run: | all
	.build_helpers/run_qemu.sh

# Prepares the files for the network-boot. This is special to my
# local setup on my developer machine, where remote computers are
# connected via LAN and load files via TFTP from my laptop.
networkboot:

.PHONY: clean

clean:

