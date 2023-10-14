# Mokaccino

Mokaccino is P2P chat and VoIP application with ecdsa authentication, ecdhe key exchange and aes-256-gcm encryption.

## Build

```bash
cargo build --release
```

### Dependencies

You will probably need to install only [perl](https://www.perl.org/) if you already have cargo set up.

- [Rust](https://www.rust-lang.org/) compiler
- All the dependencies to build [openssl-rust](https://docs.rs/openssl/latest/openssl/)
  - C compiler (gcc, clang, msvc, ...)
  - [perl](https://www.perl.org/)
  - make (gmake, nmake, ...)
