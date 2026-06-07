```
cargo run -p desktop
```

```
cargo espflash flash --release
```

```
cargo espflash flash --release --monitor --bin esp32_main
```

```
RUSTFLAGS="-A warnings" cargo build --release --bin esp32_main
```
