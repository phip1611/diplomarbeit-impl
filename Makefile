# Builds my runtime environment, the microkernel, and relevant userland components (Apps to run).
# It copies the relevant files to the "build" directory.
#
# Usage:
#   "$ make -j 8"
#   "$ make run" (starts QEMU)

BUILD_DIR=build
# needs absolute paths!
MUSL_BUILD_DIR=$(PWD)/$(BUILD_DIR)/.musl
MUSL_GCC_DIR=$(MUSL_BUILD_DIR)/bin

PARALLEL_CARGO_RUSTUP_HACK=$(BUILD_DIR)/.cargo_rustup_check

# Use the same target dir for all Rust targets. Cargo is actually intended
# to be used like this and it should not bring up any problems.
# Bound to environment variable. Cargo checks this ENV var by default.
# See https://doc.rust-lang.org/cargo/reference/environment-variables.html
export CARGO_TARGET_DIR=$(PWD)/target

.PHONY: all check clean libc_musl microkernel run run_nogui runtime_environment roottask static_foreign_apps userland_tarball

# "make" builds everything
# userland tarball itself depends on "runtime_environment static_foreign_apps"
all: microkernel roottask userland_tarball

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
	$(MAKE) -j $(shell numproc) || exit 1; \
	cp "src/hypervisor.elf32" "../../$(BUILD_DIR)/hedron.elf32"

# All artifacts of the Runtime Environment
runtime_environment: | $(BUILD_DIR)
	cd "runtime-environment" && $(MAKE) || exit 1
	cp "$(CARGO_TARGET_DIR)/x86_64-unknown-hedron/release/roottask-bin" "$(BUILD_DIR)"
	cp "$(CARGO_TARGET_DIR)/x86_64-unknown-hedron/release/native-hello-world-rust-bin" "$(BUILD_DIR)"

# Foreign Apps and Hybrid Foreign Apps in several languages (C, Rust).
# It depends on the runtime_environment target to prevent the concurrent installation of
# additional rustup targets/toolchains. Otherwise, this may break the build.
static_foreign_apps: | $(BUILD_DIR) libc_musl runtime_environment
	# bind environment var MUSL_GCC_DIR
	cd "static-foreign-apps" && MUSL_GCC_DIR="$(MUSL_GCC_DIR)" $(MAKE)
	find "static-foreign-apps/build/" -type f -exec cp "{}" "$(BUILD_DIR)" \;

# Installs musl locally in the directory but doesn't require to have it installed
# in the system. Executable is in "./libc-musl/obj/musl-gcc"
libc_musl:
    # Install it locally with absolute paths. This is important so that during runtime
    # the musl compiler finds all header files etc.
	cd "libc-musl" && ./configure "--prefix=$(MUSL_BUILD_DIR)"
	cd "libc-musl" && $(MAKE) -j $(shell numproc)
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
	.build_helpers/run_qemu_gui.sh

run_nogui:
	.build_helpers/run_qemu_nogui.sh

# Helps the user to check if all relevant stuff is installed on the system
# and the project is ready to build.
check:
	.build_helpers/check_tooling.sh
	.build_helpers/check_repo.sh
	.build_helpers/check_machine.sh

# Prepares the files for the network-boot. This is special to my
# local setup on my developer machine, where remote computers are
# connected via LAN and load files via TFTP from my laptop.
networkboot:
# TODO

clean:
	rm -rf $(BUILD_DIR) $(CARGO_TARGET_DIR)
	cd "runtime-environment" && $(MAKE) clean
	cd "static-foreign-apps" && $(MAKE) clean
	cd "thesis-hedron-fork/build" && $(MAKE) clean
