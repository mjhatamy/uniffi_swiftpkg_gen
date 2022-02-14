# uniffi_swiftpkg_gen

Generates swift package based on rust UniFFI.

## How to install

Add **uniffi_swiftpkg_gen** to your rust cargo.

```cargo
[build-dependencies]
uniffi_swiftpkg_gen = "0.1.2"
```

## Example:
```rust
use uniffi_swiftpkg_gen::*;

fn main() {
    // Rust uniffi package gen
    uniffi_build::generate_scaffolding("./src/gix_guard.udl")
        .unwrap();

    // Generates Xcode Swift package
    Builder::new().generate();
}
```