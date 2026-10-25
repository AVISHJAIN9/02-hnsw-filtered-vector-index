.PHONY: build test bench
build:
	cd backend && cargo build
test:
	cd backend && cargo test
bench:
	cd backend && cargo bench
