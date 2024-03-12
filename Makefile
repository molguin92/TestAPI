.PHONY: all

all:
	pandoc --standalone instructions.md -o instructions.html
	cargo build --release

docker:
	docker build -t molguin92/testapi:latest -f Dockerfile .