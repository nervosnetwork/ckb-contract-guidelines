TARGET := riscv64-unknown-linux-gnu
CC := $(TARGET)-gcc
LD := $(TARGET)-gcc
OBJCOPY := $(TARGET)-objcopy
CFLAGS := -fPIC -O3 -fno-builtin-printf -fno-builtin-memcmp -nostdinc -nostdlib -nostartfiles -fvisibility=hidden -fdata-sections -ffunction-sections -I deps/ckb-c-stdlib -I deps/ckb-c-stdlib/libc -I deps/ckb-c-stdlib/molecule -Wall -Werror -Wno-nonnull -Wno-nonnull-compare -Wno-unused-function -g
LDFLAGS := -Wl,-static -fdata-sections -ffunction-sections -Wl,--gc-sections

SIMULATOR_CC := gcc
SIMULATOR_LIB := deps/simulator/target/release/libckb_x64_simulator.a
SIMULATOR_CFLAGS := -fno-builtin-printf -fno-builtin-memcmp -I deps/ckb-c-stdlib -I deps/ckb-c-stdlib/molecule -Wall -Werror -Wno-nonnull -Wno-nonnull-compare -Wno-unused-function -g -DCKB_STDLIB_NO_SYSCALL_IMPL
SIMULATOR_LDFLAGS := -lpthread -ldl

# docker pull nervos/ckb-riscv-gnu-toolchain:gnu-bionic-20191012
BUILDER_DOCKER := nervos/ckb-riscv-gnu-toolchain@sha256:aae8a3f79705f67d505d1f1d5ddc694a4fd537ed1c7e9622420a470d59ba2ec3

all: build/simple_udt

all-via-docker:
	docker run --rm -v `pwd`:/code ${BUILDER_DOCKER} bash -c "cd /code && make"

simulators: build/simple_udt_sim

build/simple_udt_sim: simple_udt.c ${SIMULATOR_LIB}
	$(SIMULATOR_CC) $(SIMULATOR_CFLAGS) $(SIMULATOR_LDFLAGS) -o $@ $^

build/simple_udt: simple_udt.c
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $<
	$(OBJCOPY) --only-keep-debug $@ $@.debug
	$(OBJCOPY) --strip-debug --strip-all $@

${SIMULATOR_LIB}:
	cd deps/simulator && cargo build --release

clean:
	rm -rf build/simple_udt build/simple_udt_sim
	cd deps/simulator && cargo clean

dist: clean all simulators

fmt:
	clang-format -i -style=Google $(wildcard *.h *.c)
	git diff --exit-code

.PHONY: all all-via-docker simulators clean dist fmt
