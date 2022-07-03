.PHONY: clean

clean:
  rm -rf pkg

build:
  wasm-pack build --release

publish:
  npm publish pkg
