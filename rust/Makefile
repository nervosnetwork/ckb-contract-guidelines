ENVIRONMENT := debug

all: build/$(ENVIRONMENT)/nft-validator

simulators:
	cargo build -p natives
	mkdir -p build/$(ENVIRONMENT)
	cp target/$(ENVIRONMENT)/nft-validator-sim build/$(ENVIRONMENT)/nft-validator-sim

test: all simulators
	cargo test -p tests
	scripts/run_sim_tests.sh $(ENVIRONMENT)

clean:
	cargo clean
	rm -rf build/$(ENVIRONMENT)

build/$(ENVIRONMENT)/nft-validator:
	capsule build

.PHONY: all simulators test clean
