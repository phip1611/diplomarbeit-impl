runtime_environment:
	cd "ws" && ./build-cargo-fake-workspace.sh || exit 1

.PHONY: clean

clean:
	# Probably pointless because I use a shared target dir that gets
    # already cleaned by the main make clean command.
	./ws_cargo clean
