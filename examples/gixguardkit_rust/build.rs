use uniffi_swiftpkg_gen::*;

fn main() {
    uniffi_build::generate_scaffolding("./src/gix_guard.udl")
        .unwrap();


    Builder::new().generate();
}