use std::path::{Path, PathBuf};

mod cargo_package;
mod xcgenerator;
mod xc_framework_generator;
mod swift_package_item;
mod ext;
mod command_builder;
mod shared_constants;

use std::env;
use cargo_package::*;
use xc_framework_generator::XCFrameworkBuilder;
use shared_constants::*;
use crate::swift_package_item::SwiftPackageItem;

#[allow(unused)]
pub struct Builder {
    create_path: PathBuf,
    crate_package_name: String,
    xcode_framework_name: Option<String>,
    xcode_framework_path: PathBuf,
    crate_lib_name: Option<String>,
}

impl Builder {
    #[allow(clippy::new_without_default)]
    #[allow(unused)]
    pub fn new() -> Self {
        let package_name = env::var("CARGO_PKG_NAME").unwrap();
        let crate_dir = Path::new(env::var("CARGO_MANIFEST_DIR").unwrap().as_str()).to_path_buf();
        let create_path = crate_dir.join("Cargo.toml");
        Builder {
            create_path,
            crate_package_name: package_name,
            xcode_framework_name: None,
            xcode_framework_path: crate_dir.join("xcode"),
            crate_lib_name: None,
        }
    }

    #[allow(unused)]
    pub fn with_crate_lib_name(mut self, lib_name: String) -> Builder {
        self.crate_lib_name = Some(lib_name);
        self
    }

    #[allow(unused)]
    pub fn with_swift_package_name(mut self, package_name: String) -> Builder {
        self.xcode_framework_name = Some(package_name);
        self
    }

    #[allow(unused)]
    pub fn with_swift_package_build_path(mut self, build_path: String) -> Builder {
        self.xcode_framework_path = Path::new(build_path.as_str()).to_path_buf();
        self
    }

    #[allow(unused)]
    pub fn generate(mut self) {
        if env::var(SKIP_UNIFFI_SWIFTPKG_GEN).unwrap_or_else(|_| "false".to_string()).to_lowercase() == "true" {
            return;
        }

        let cargo_package = CargoPackage::new(
            self.create_path.as_path(),
            Some(&self.crate_package_name),
            self.crate_lib_name.as_ref(),
            self.xcode_framework_name.as_ref(),
            self.xcode_framework_path.as_path());

        // for udl_item in &cargo_package.udl_absolute_files_path {
        //     uniffi_build::generate_scaffolding(udl_item.0.to_str().unwrap())
        //         .unwrap();
        // }

        //println!("***************** cargo_package: {:?}", cargo_package);

        let xc = XCFrameworkBuilder::new(cargo_package);
        let frameworks = xc.build();

        for item in frameworks {
            let swift_package: SwiftPackageItem = SwiftPackageItem::new(item);
            //println!("swift_package: {:?}", swift_package);
            swift_package.build();
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
