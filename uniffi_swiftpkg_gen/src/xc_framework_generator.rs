use std::fmt::Formatter;
use std::path::{Path, PathBuf};
use std::process::{ exit };
use colored::Colorize;
use super::ext::*;
use super::command_builder::*;
use super::cargo_package::*;
use super::shared_constants::*;

const IOS_TARGETS: [&str; 1] = [ "aarch64-apple-ios" ];
const IOS_SIM_TARGETS: [&str; 2] = ["aarch64-apple-ios-sim", "x86_64-apple-ios"];
const MACOS_TARGETS: [&str; 2]= ["aarch64-apple-darwin", "x86_64-apple-darwin"];

#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum BuildType {
    Debug,
    Release
}

impl std::fmt::Display for BuildType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match *self {
            BuildType::Release => "release",
            BuildType::Debug => "debug",
        };
        f.write_str(value )
    }
}

#[derive(Debug)]
pub(crate) struct XCFrameworkItem {
    pub(crate) build_type: BuildType,
    pub(crate) headers_path: PathBuf,
    pub(crate) swift_files_path: PathBuf,
    pub(crate) xc_frameworks_uri: PathBuf,
    pub(crate) swift_package_build_path: PathBuf,
    pub(crate) swift_package_name: String,
}

struct XCFrameworkBuildItems {
    build_type: BuildType,
    ios_lib_path: String,
    ios_headers_path: String,
    ios_sim_lib_path: String,
    ios_sim_headers_path: String,
    osx_lib_path: String,
    osx_headers_path: String,
}

pub(crate) struct XCFrameworkBuilder {
    command: CommandBuilder,
    //host_arch: String,
    crate_package: CargoPackage
}

impl XCFrameworkBuilder {
    #[allow(unused)]
    pub(crate) fn new(crate_package: CargoPackage) -> Self {
        let command_builder =  CommandBuilder::new();

        // let host_arch = command_builder
        //     .args(["uname -m"])
        //     .expect("Unable to get system architecture. Check your system if any shell is installed and 'uname -m' is working")
        //     .utf8_string().first()
        //     .unwrap().clone();

        let inst = XCFrameworkBuilder {
            command: command_builder,
            //host_arch,
            crate_package
        };
        inst.init();
        inst
    }

    fn init(&self) {
        let not_installed_targets = self.get_not_installed_targets();
        for target in not_installed_targets {
            self.install_rustup_target(target);
        }
    }

    fn get_not_installed_targets(&self) -> Vec<String> {
        let installed_archs = self.command
            .args(["rustup target list --installed"])
            .expect("failed to execute process")
            .utf8_string();

        IOS_TARGETS.into_iter()
            .chain(IOS_SIM_TARGETS.into_iter())
            .chain(MACOS_TARGETS.into_iter())
            .filter(|f| !installed_archs.contains(&f.to_string()))
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
    }

    fn install_rustup_target(&self, target: String) {
        let status = self.command
            .args_stream([format!("rustup target add {}", target).as_str()]);
        if !status.success() {
            eprintln!("Failed to install rust target: {}. Check your connection to internet ", target);
            eprintln!("{}. status:{}", "execution failed. ".red(), status);
            exit(1);
        }
    }

    #[allow(unused)]
    pub(crate) fn build(&self) -> Vec<XCFrameworkItem> {
        let mut items: Vec<XCFrameworkItem> = vec![];
        let (headers_dir, swift_files_dir) = self.build_uniffi_bindgen();
        let build_items = self.build_targets(&headers_dir);
        for build_item in build_items {
            let xc_framework_uri = self.build_xc_framework(&build_item);
            let item = XCFrameworkItem {
                build_type: build_item.build_type,
                headers_path: Path::new(build_item.ios_headers_path.as_str()).to_path_buf(),
                xc_frameworks_uri: Path::new(xc_framework_uri.as_str()).to_path_buf(),
                swift_files_path: Path::new(swift_files_dir.as_str()).to_path_buf(),
                swift_package_build_path: self.crate_package.swift_package_path.clone(),
                swift_package_name: self.crate_package.swift_package_name.clone(),
            };
            items.push(item);
        }
        items
    }

    #[allow(unused)]
    fn build_xc_framework(&self, item: &XCFrameworkBuildItems) -> String {
        let out_dir = format!("{}/target/universal/xc",
                              self.crate_package.crate_dir.to_str().unwrap() );
        let libs_query = format!("-library {} -headers {} -library {} -headers {} -library {} -headers {}",
        item.ios_lib_path, item.ios_headers_path,
        item.ios_sim_lib_path, item.ios_sim_headers_path,
        item.osx_lib_path, item.osx_headers_path);
        let output_lib = format!("{}/{}{}.xcframework",
            out_dir,
                                 self.crate_package.swift_package_name,
                                 if item.build_type == BuildType::Debug { "_debug" } else { ""} );
        let main_query = format!("mkdir -p {0}; rm -rf {2}; xcodebuild -create-xcframework {1} -output {2}",
                                  out_dir,
                                 libs_query, output_lib);

        let status = self.command.args_stream([main_query]);
        if !status.success() {
            eprintln!("{}{}", "Building xc-framework failed. ".red(), status);
            exit(1);
        }
        output_lib
    }

    #[allow(unused)]
    fn build_targets(&self, headers_path: &str) -> Vec<XCFrameworkBuildItems> {
        let mut items: Vec<XCFrameworkBuildItems> = vec![];
        let build_types = [BuildType::Debug, BuildType::Release];

        for build_type in build_types {
            let ios_lib_files = self.build_lipo("ios", &build_type,
                                                IOS_TARGETS.to_vec().iter()
                                                    .map(|f| self.compile_for_target(f, build_type))
                                                    .collect::<Vec<String>>());
            let ios_sim_lib_files = self.build_lipo("ios_sim", &build_type,
                                                    IOS_SIM_TARGETS.to_vec().iter()
                                                        .map(|f| self.compile_for_target(f, build_type))
                                                        .collect::<Vec<String>>());
            let osx_lib_files = self.build_lipo("osx", &build_type,
                                                MACOS_TARGETS.to_vec().iter()
                                                    .map(|f| self.compile_for_target(f, build_type))
                                                    .collect::<Vec<String>>());
            let item = XCFrameworkBuildItems {
                build_type,
                ios_lib_path: ios_lib_files,
                ios_headers_path: headers_path.to_string(),
                ios_sim_lib_path: ios_sim_lib_files,
                ios_sim_headers_path: headers_path.to_string(),
                osx_lib_path: osx_lib_files,
                osx_headers_path: headers_path.to_string()
            };
            items.push(item);
        }
        items
    }

    #[allow(unused)]
    fn build_uniffi_bindgen(&self) -> (String, String) {
        let out_dir = format!("{}/target/universal/headers",
                                 self.crate_package.crate_dir.to_str().unwrap() );
        let swift_out_dir = format!("{}/target/universal/swift",
                              self.crate_package.crate_dir.to_str().unwrap() );

        for (path, _name) in &self.crate_package.udl_absolute_files_path {
            let status = self.command.args_stream([format!("$HOME/.cargo/bin/uniffi-bindgen generate {} --language swift --out-dir {}",
                                              path.to_str().unwrap(), out_dir ).as_str()]);
            if !status.success() {
                eprintln!("{}{}", "execution of uniffi_bindgen failed. ".red(), status);
                exit(1);
            }

            // rename *.modulemap to module.modulemap
            let status = self.command.args_stream([format!("mv {0}/*FFI.modulemap {0}/module.modulemap", out_dir)]);
            if !status.success() {
                eprintln!("{}{}", "Renaming module map failed. ".red(), status);
                exit(1);
            }
            // Move Swift packages to swift
            let status = self.command.args_stream([format!("mkdir -p {1}; mv {0}/*.swift {1}/", out_dir, swift_out_dir)]);
            if !status.success() {
                eprintln!("{}{}", "Moving swift files failed. ".red(), status);
                exit(1);
            }
        }
        (out_dir, swift_out_dir)
    }

    #[allow(unused)]
    fn build_lipo(&self, os: &str, build_type: &BuildType, targets_paths: Vec<String>) -> String {
        let target_dir = format!("{}/target/universal/{}/{}/",
                                 self.crate_package.crate_dir.to_str().unwrap(),
                                 build_type, os);
        let target_uri = format!("{}/target/universal/{}/{}/lib{}.a",
                           self.crate_package.crate_dir.to_str().unwrap(),
                                 build_type, os,
                           self.crate_package.target_name);

        let mut target_str = String::new();
        let mut count: usize = 1;
        for value in targets_paths.iter() {
            target_str.push_str(value);
            target_str.push(' '); // space in between
            count += 1;
        }
        // Build directory if required
        let status = self.command.args_stream([format!("mkdir -p {}", target_dir).as_str()]);
        if !status.success() {
            eprintln!("Failed to create universal base directory at: {}", target_dir);
            eprintln!("{}. status:{}", "execution failed. ".red(), status);
            exit(1);
        }

        if count == 0 {
            eprintln!("Failed to create Fat binary for OS: {} ", os.red());
            exit(1);
        } else if count == 1 {
            let status = self.command.args_stream([format!("mv {} {}", target_str, target_uri)]);
            if !status.success() {
                eprintln!("Unable to move built library from:\n{}\nto\n{}\n for OS: {} in mode: {}",
                        target_str, target_uri, os, build_type);
                eprintln!("{}. status:{}", "execution failed. ".red(), status);
                exit(1);
            }
        } else {
            let status = self.command.args_stream([format!("lipo -create -output  {} {}",
                                       target_uri, target_str)]);
            if !status.success() {
                eprintln!("Unable to move built library from:\n{}\nto\n{}\n for OS: {} in mode: {}",
                        target_str, target_uri, os, build_type);
                eprintln!("{}. status:{}", "execution failed. ".red(), status);
                exit(1);
            }
        }
        target_uri
    }

    #[allow(unused)]
    fn compile_for_target(&self, target: &str, build_types: BuildType) -> String {
        let target_dir = format!("{}/target",
                           self.crate_package.crate_dir.to_str().unwrap());

        // Skipping builder is important to eliminate the cyclic build process execution.
        let command_code = format!("{}=true $HOME/.cargo/bin/cargo build --locked -p {} --lib {} --target {} --target-dir {} --manifest-path {}",
                                   SKIP_UNIFFI_SWIFTPKG_GEN,
                                   self.crate_package.crate_name,
                                   if build_types == BuildType::Debug { "" } else { "--release" },
                                   target, target_dir, self.crate_package.crate_dir.join("Cargo.toml").to_str().unwrap()
        );
        let status = self.command.args_stream([command_code.as_str()]);
        if !status.success() {
            eprintln!("Failed to compile crate name: {}, lib_name: {}, target architecture: {}",
                             self.crate_package.crate_name,
                             self.crate_package.target_name, target);
            eprintln!("{}{}", "Argument failed with error: ".red(), status);
            exit(1);
        }

        let path = format!("{}/target/{}/{}/lib{}.a",
                self.crate_package.crate_dir.to_str().unwrap(),
                target, build_types, self.crate_package.target_name);
        path
    }
}

#[cfg(test)]
mod tests {
    use super::XCFrameworkBuilder;
    #[test]
    fn get_rust_up_targets_for() {
        let xc = XCFrameworkBuilder::new();
        //xc.get_rust_up_targets_for();
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}