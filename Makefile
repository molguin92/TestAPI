.PHONY: all

all:
	pandoc --standalone instructions.md -o instructions.html
	cargo build --release
