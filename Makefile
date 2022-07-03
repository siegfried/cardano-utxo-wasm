.PHONY: clean

clean:
	rm -rf pkg

build: clean
	wasm-pack build --release --target web

publish: pkg
	npm publish pkg/
