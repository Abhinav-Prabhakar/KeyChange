.PHONY: run test app dmg icon clean

run:
	cargo run

test:
	cargo test

app:
	scripts/bundle.sh

dmg:
	scripts/dmg.sh

icon:
	scripts/make-icon.sh

clean:
	cargo clean
	rm -rf dist
