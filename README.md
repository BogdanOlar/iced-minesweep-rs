# Iced minesweep-rs

A rust implementation of the popular game, using the [iced](https://github.com/iced-rs/iced) library.

![screenshot](.github/Screenshot.png)

## TODO

- [x] Linux
- [ ] Windows
- [ ] WASM
- [x] Config
- [x] High scores
- [x] Layout

## Build & run

### Desktop

Prerequisites: 
 - `cargo` and `rustc` (see [installation instructions](https://www.rust-lang.org/tools/install))
 - Some crates used by `iced` seem to need `cmake`, `g++`, and some font utilities. On Fedora these can be installed with 
    ```bash
    sudo dnf group install "C Development Tools and Libraries" "Development Tools"
    sudo dnf install fontconfig fontconfig-devel
    ```

Build and run:

```bash
git clone https://github.com/BogdanOlar/iced-minesweep-rs
cd iced-minesweep-rs/
cargo run --release
```

### Wasm

TODO:

## License

[MIT](./LICENSE)
