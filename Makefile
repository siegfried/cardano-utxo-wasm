.PHONY: clean

clean:
	rm -rf pkg

build: clean
	wasm-pack build --release

publish: pkg
	npm publish pkg/
