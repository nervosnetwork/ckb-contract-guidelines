TARGET := riscv64-unknown-linux-gnu
CC := $(TARGET)-gcc
LD := $(TARGET)-gcc
OBJCOPY := $(TARGET)-objcopy
CFLAGS := -fPIC -O3 -fno-builtin-printf -fno-builtin-memcmp -nostdinc -nostdlib -nostartfiles -fvisibility=hidden -fdata-sections -ffunction-sections -I deps/ckb-c-stdlib -I deps/ckb-c-stdlib/libc -I deps/ckb-c-stdlib/molecule -Wall -Werror -Wno-nonnull -Wno-nonnull-compare -Wno-unused-function -g
LDFLAGS := -Wl,-static -fdata-sections -ffunction-sections -Wl,--gc-sections

ENVIRONMENT := debug

SIMULATOR_CC := gcc
SIMULATOR_CLANG := clang
SIMULATOR_LIB := deps/simulator/target/release/libckb_x64_simulator.a
SIMULATOR_CFLAGS := -fno-builtin-printf -fno-builtin-memcmp -I deps/ckb-c-stdlib -I deps/ckb-c-stdlib/molecule -Wall -Werror -Wno-nonnull -Wno-unused-function -g -DCKB_STDLIB_NO_SYSCALL_IMPL -DBLAKE2_REF_C
SIMULATOR_COVERAGE_CFLAGS := -fprofile-arcs -ftest-coverage -Wno-nonnull-compare
SIMULATOR_UNDEFINED_CFLAGS := -fsanitize=undefined -fsanitize=implicit-conversion -fsanitize=local-bounds -fsanitize=unsigned-integer-overflow -fsanitize=nullability
SIMULATOR_ADDRESS_CFLAGS := -fsanitize=address
SIMULATOR_LDFLAGS := -lpthread -ldl

# docker pull nervos/ckb-riscv-gnu-toolchain:gnu-bionic-20191012
BUILDER_DOCKER := nervos/ckb-riscv-gnu-toolchain@sha256:aae8a3f79705f67d505d1f1d5ddc694a4fd537ed1c7e9622420a470d59ba2ec3

all: build/$(ENVIRONMENT)/simple_udt build/$(ENVIRONMENT)/bin_sample build/$(ENVIRONMENT)/lib_sample

all-via-docker:
	docker run --rm -v `pwd`:/code ${BUILDER_DOCKER} bash -c "cd /code && make"

simulators: build/$(ENVIRONMENT)/simple_udt_sim build/$(ENVIRONMENT)/bin_sample_sim build/$(ENVIRONMENT)/lib_sample_sim.so

test: all simulators
	cd tests && cargo test
	ASAN_OPTIONS=detect_odr_violation=0 scripts/run_sim_tests.sh $(ENVIRONMENT)

coverage: test
	mkdir -p build/coverage
	gcovr -r . -e deps --html --html-details -o build/coverage/coverage.html -s

build/$(ENVIRONMENT)/simple_udt_sim: simple_udt.c ${SIMULATOR_LIB}
	mkdir -p build/$(ENVIRONMENT)
	$(SIMULATOR_CC) $(SIMULATOR_CFLAGS) $(SIMULATOR_COVERAGE_CFLAGS) $(SIMULATOR_LDFLAGS) -o $@ $^
	$(SIMULATOR_CLANG) $(SIMULATOR_CFLAGS) $(SIMULATOR_UNDEFINED_CFLAGS) $(SIMULATOR_LDFLAGS) -o $@.ubsan $^
	$(SIMULATOR_CLANG) $(SIMULATOR_CFLAGS) $(SIMULATOR_ADDRESS_CFLAGS) $(SIMULATOR_LDFLAGS) -o $@.asan $^

build/$(ENVIRONMENT)/bin_sample_sim: bin_sample.c ${SIMULATOR_LIB}
	mkdir -p build/$(ENVIRONMENT)
	$(SIMULATOR_CC) $(SIMULATOR_CFLAGS) $(SIMULATOR_LDFLAGS) -o $@ $^
	$(SIMULATOR_CLANG) $(SIMULATOR_CFLAGS) $(SIMULATOR_UNDEFINED_CFLAGS) $(SIMULATOR_LDFLAGS) -o $@.ubsan $^
	$(SIMULATOR_CLANG) $(SIMULATOR_CFLAGS) $(SIMULATOR_ADDRESS_CFLAGS) $(SIMULATOR_LDFLAGS) -o $@.asan $^

build/$(ENVIRONMENT)/lib_sample_sim.so: lib_sample.c ${SIMULATOR_LIB}
	mkdir -p build/$(ENVIRONMENT)
	$(SIMULATOR_CC) $(SIMULATOR_CFLAGS) $(SIMULATOR_LDFLAGS) -shared -fPIC -o $@ $^
	$(SIMULATOR_CLANG) $(SIMULATOR_CFLAGS) $(SIMULATOR_UNDEFINED_CFLAGS) $(SIMULATOR_LDFLAGS) -shared -fPIC -o $@.ubsan $^
	$(SIMULATOR_CLANG) $(SIMULATOR_CFLAGS) $(SIMULATOR_ADDRESS_CFLAGS) $(SIMULATOR_LDFLAGS) -shared -fPIC -o $@.asan $^

build/$(ENVIRONMENT)/simple_udt: simple_udt.c
	mkdir -p build/$(ENVIRONMENT)
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $<
	$(OBJCOPY) --strip-debug --strip-all $@ $@.strip

build/$(ENVIRONMENT)/bin_sample: bin_sample.c
	mkdir -p build/$(ENVIRONMENT)
	$(CC) $(CFLAGS) $(LDFLAGS) -o $@ $<
	$(OBJCOPY) --strip-debug --strip-all $@ $@.strip

build/$(ENVIRONMENT)/lib_sample: lib_sample.c
	mkdir -p build/$(ENVIRONMENT)
	$(CC) $(CFLAGS) $(LDFLAGS) -shared -o $@ $<
	$(OBJCOPY) --strip-debug --strip-all $@ $@.strip

${SIMULATOR_LIB}:
	cd deps/simulator && cargo build --release

clean:
	rm -rf build/$(ENVIRONMENT) build/coverage
	rm -rf *.gcov *.gcda *.gcno
	cd deps/simulator && cargo clean
	cd tests && cargo clean

dist: clean all simulators

fmt:
	clang-format -i -style=Google $(wildcard *.h *.c)
	cd tests; cargo fmt --all
	git diff --exit-code

.PHONY: all all-via-docker simulators clean dist fmt test coverage
