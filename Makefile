# Builds my runtime environment, the microkernel, and relevant userland components (Apps to run).

BUILD_ARTIFACT_DIR="./build"

all: microkernel runtime_environment static_foreign_apps userland_tarball

$(BUILD_ARTIFACT_DIR):
	mkdir -p $@

# Hedron Microhypervisor/Microkernel
microkernel: | $(BUILD_ARTIFACT_DIR)
    # To execute all in the same sub process I need to
    # concat multiple commands with ";" and remove newlines (\)
    # https://stackoverflow.com/questions/1789594/how-do-i-write-the-cd-command-in-a-makefile
	cd "./thesis-hedron-fork"; \
	mkdir -p "build"; \
	cd "build"; \
	cmake -DBUILD_TESTING=OFF -DCMAKE_BUILD_TYPE=Release ..; \
	$(MAKE) -j $(shell nproc); \
	cp "src/hypervisor.elf32" "../../$(BUILD_ARTIFACT_DIR)/hedron.elf32"

# All artifacts of the Runtime Environment
runtime_environment:| $(BUILD_ARTIFACT_DIR)
	cd "runtime-environment" && $(MAKE)
	cp "runtime-environment/ws/roottask-bin/target/x86_64-unknown-hedron/release/roottask-bin" "$(BUILD_ARTIFACT_DIR)"
	cp "runtime-environment/ws/helloworld-bin/target/x86_64-unknown-hedron/release/helloworld-bin" "$(BUILD_ARTIFACT_DIR)"
# TODO file server
# cp "runtime-environment/ws/fileserver-bin/target/x86_64-unknown-hedron/release/fileserver-bin" "$(BUILD_ARTIFACT_DIR)"

# Foreign Apps and Hybrid Foreign Apps in several languages (C, Rust).
static_foreign_apps:| $(BUILD_ARTIFACT_DIR)
	cd "static-foreign-apps" && $(MAKE)
	find "static-foreign-apps/build/" -type f -exec cp "{}" "$(BUILD_ARTIFACT_DIR)" \;

# Creates a tarball with the whole userland the roottask should bootstrap.
# Currently this only works because each expected file is hard-coded into
# the roottask. Basically this contains all relevant files from
userland_tarball: | runtime_environment static_foreign_apps
	# shell script, because I don't know how
	# to nicely solve it in Makefile
	.build_helpers/build_tarball.sh

# Starts QEMU with Hedron and my runtime environment
run: | microkernel userland_tarball
	.build_helpers/run_qemu.sh

# Prepares the files for the network-boot. This is special to my
# local setup on my developer machine, where remote computers are
# connected via LAN and load files via TFTP from my laptop.
networkboot:

.PHONY: clean

clean:

