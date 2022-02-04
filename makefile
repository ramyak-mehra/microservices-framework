#Run library test while suppressing the warnings
test:
	RUSTFLAGS=-Awarnings cargo test --lib -- --nocapture
.PHONY: test