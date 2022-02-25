all: build_man

build_man:
	@pandoc --standalone --to man swhkd.md -o swhkd.1
	@pandoc --standalone --to man swhks.md -o swhks.1

.PHONY: build_man
