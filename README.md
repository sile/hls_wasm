hls_wasm
=========

Build
------

```console
$ rustup target add wasm32-unknown-unknown --toolchain nightly
$ cargo +nightly build --target=wasm32-unknown-unknown --release

$ firefox examples/player.html
```
