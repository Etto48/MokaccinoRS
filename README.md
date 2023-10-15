# Mokaccino

[![Rust-Linux](https://github.com/Etto48/MokaccinoRS/actions/workflows/rust-linux.yml/badge.svg)](https://github.com/Etto48/MokaccinoRS/actions/workflows/rust-linux.yml)
[![Rust-MacOS](https://github.com/Etto48/MokaccinoRS/actions/workflows/rust-macos.yml/badge.svg)](https://github.com/Etto48/MokaccinoRS/actions/workflows/rust-macos.yml)
[![Rust-Windwos](https://github.com/Etto48/MokaccinoRS/actions/workflows/rust-windows.yml/badge.svg)](https://github.com/Etto48/MokaccinoRS/actions/workflows/rust-windows.yml)

Mokaccino is P2P chat and VoIP application with ecdsa authentication, ecdhe key exchange and aes-256-gcm encryption.

## Build

```bash
cargo build --release
```

### Dependencies

You will probably need to install only [perl](https://www.perl.org/) if you already have cargo set up, and if you are on linux you will also need alsa (libasound2-dev) installed.

- [Rust](https://www.rust-lang.org/) compiler
- All the depndencies to build [cpal](https://docs.rs/cpal/latest/cpal/)
  - libasound2-dev
- All the dependencies to build [openssl-rust](https://docs.rs/openssl/latest/openssl/)
  - C compiler (gcc, clang, msvc, ...)
  - [perl](https://www.perl.org/)
  - make (gmake, nmake, ...)
