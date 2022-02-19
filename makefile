#Run library test while suppressing the warnings
test:
	RUSTFLAGS=-Awarnings cargo test --lib -- --nocapture
test-broker:
	RUSTFLAGS=-Awarnings cargo test --lib -- broker::tests --nocapture
test-services:
	RUSTFLAGS=-Awarnings cargo test --lib -- services::tests --nocapture
.PHONY: test test-broker test-services