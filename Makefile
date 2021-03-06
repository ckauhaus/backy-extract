DESTDIR := dist

all: release

release: target/release/backy-extract target/release/backy-fuse

target/release/backy-%: Cargo.toml src/*.rs src/*/*.rs
	cargo build --release --features fuse_driver

VERSION := $(shell cargo read-manifest | jq .version -r)
PV = backy-extract-$(VERSION)

dist: release
	install -D target/release/backy-extract -t tmp/$(PV)/bin
	install -D target/release/backy-fuse -t tmp/$(PV)/bin
	install -D -m 0644 README.md ChangeLog -t tmp/$(PV)/share/doc
	install -d tmp/$(PV)/share/man/man1 dist
	cd man && for f in *.1.rst; do \
	  rst2man $$f | sed s"/@version@/$(VERSION)/" > ../tmp/$(PV)/share/man/man1/$${f%%.rst}; \
	done
	tar czf dist/$(PV).tar.gz -C tmp $(PV)
	rm -r tmp

clean:
	cargo clean
	rm -rf tmp dist

.PHONY: release dist clean
