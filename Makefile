test:
	cargo test

bench:
	cargo bench

cov:
	docker run -it --rm --security-opt seccomp=unconfined --volume "$$(PWD):/volume" elmtai/docker-rust-kcov
