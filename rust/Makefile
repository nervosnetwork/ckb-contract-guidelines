ENVIRONMENT := debug

all: build/$(ENVIRONMENT)/nft-validator

simulators:
	CARGO_INCREMENTAL=0 RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort" RUSTDOCFLAGS="-Cpanic=abort" cargo build -p natives
	mkdir -p build/$(ENVIRONMENT)
	cp target/$(ENVIRONMENT)/nft-validator-sim build/$(ENVIRONMENT)/nft-validator-sim

test: all simulators
	cargo test -p tests
	scripts/run_sim_tests.sh $(ENVIRONMENT)

coverage: test
	zip -0 build/$(ENVIRONMENT)/ccov.zip `find . \( -name "nft_validator_sim*.gc*" \) -print`
	grcov build/$(ENVIRONMENT)/ccov.zip -s . -t lcov --llvm --branch --ignore-not-existing --ignore "/*" -o build/$(ENVIRONMENT)/lcov.info
	genhtml -o build/$(ENVIRONMENT)/coverage/ --rc lcov_branch_coverage=1 --show-details --highlight --ignore-errors source --legend build/$(ENVIRONMENT)/lcov.info

clean:
	cargo clean
	rm -rf build/$(ENVIRONMENT)

build/$(ENVIRONMENT)/nft-validator:
	capsule build

.PHONY: all simulators test coverage clean
