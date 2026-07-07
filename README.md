This project is by no mean finished, so it's riddled with bugs and is missing testing.

This is a small hobby project, a recreation of a couple of classic games on a 64x64 screen. I have a habit of making a game engine in every language I touch and this time I wanted to make something that would run on something cheap and small (ESP32-S3). This is a third rewrite of the engine, although with each rewrite I'm adding more stuff. First the engine was made in JS, but it would be too slow, so I rewrote the thing onto C++ which I grew to hate so now it's a Rust project and I'm loving it this way. It's been a great playground for me to learn Rust, thus I use AI only to have something about Rust explained or to fix bulks of warnings.

It's running at my house in my modified IKEA coffee table. I mounted into it two 32x64 HUB75 screens, five arcade buttons and two arcade joysticks.

[[picture here]]



It can be run either on a desktop or on an esp32, so it has two entry points.

To run on a desktop (tested only on Mac):
```
cd desktop_main
cargo run
```

To run on esp32:
```
cd esp32_main
RUSTFLAGS="-A warnings" cargo espflash flash --release --monitor --bin esp32_main
```

Setting `STOLIK_DEBUG` env var to `1` will enable debug features (drawing colliders).






```
cargo espflash flash --release
```

```
RUSTFLAGS="-A warnings" cargo espflash flash --release --monitor --bin esp32_main
```

```
RUSTFLAGS="-A warnings" cargo build --release --bin esp32_main
```
