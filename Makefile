# Space Compute — Quick Task Runner
.PHONY: all deploy-local verify test clean

all: verify

deploy-local:
	@bash scripts/deploy-local.sh

verify:
	@bash scripts/verify-local.sh

test:
	@cargo test --workspace

clean:
	@cargo clean
	@rm -rf .env.local
