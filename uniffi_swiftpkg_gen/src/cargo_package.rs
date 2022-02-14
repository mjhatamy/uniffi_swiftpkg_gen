use cargo_metadata::Target;
use colored::Colorize;
use convert_case::{Case, Casing};
use pathdiff::diff_paths;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::exit;

#[derive(Debug, std::cmp::Eq, std::cmp::PartialEq)]
pub(crate) struct CargoPackage {
    pub(crate) swift_package_name: String,
    pub(crate) swift_package_path: PathBuf,
    pub(crate) crate_name: String,
    pub(crate) target_name: String,
    pub(crate) lib_src_path: PathBuf,
    pub(crate) crate_manifest_path: PathBuf,
    pub(crate) crate_dir: PathBuf,
    pub(crate) cargo_relative_path_to_xcode_project: PathBuf,
    pub(crate) udl_absolute_files_path: Vec<(PathBuf, String)>,
    pub(crate) udl_relative_files_path: Vec<(PathBuf, String)>,
    pub(crate) base_bundle_identifier: String,
}

impl CargoPackage {
    pub(crate) fn new(
        crate_path: &Path,
        crate_name: Option<&String>,
        lib_name: Option<&String>,
        swift_package_name: Option<&String>,
        swift_package_path: &Path,
    ) -> Self {
        let (crate_package_name, crate_lib_name, lib_src_path) =
            CargoPackage::validate_cargo_and_return(crate_path,
                                                    crate_name,
                                                    lib_name);

        let cargo_base_dir = if crate_path.ends_with("Cargo.toml") {
            crate_path
                .parent()
                .expect("Cargo.toml file has no Parent folder !!!!!!!!")
                .to_path_buf()
        } else {
            crate_path.to_path_buf()
        };

        // Convert Xcode Project case to Pascal Format
        let xcode_framework_name = match swift_package_name {
            Some(name) => name.to_case(Case::Pascal),
            None => crate_lib_name.to_case(Case::Pascal),
        };

        // Check if Provided Path ends with Xcode Project name in Pascal format
        let xcode_framework_path = if swift_package_path.ends_with(&xcode_framework_name) {
            swift_package_path.to_path_buf()
        } else {
            swift_package_path.join(&xcode_framework_name)
        };
        let cargo_relative_path_to_xcode_project =
            diff_paths(&cargo_base_dir, &xcode_framework_path)
                .expect("Unable to get relative path to Cargo");

        let udl_files_path = CargoPackage::find_udl_files(crate_path);

        let mut udl_relative_files_path: Vec<(PathBuf, String)> = vec![];
        for (udl_path, filename) in &udl_files_path {
            udl_relative_files_path.push((
                diff_paths(udl_path, &xcode_framework_path).expect("Getting relative path failed."),
                filename.clone(),
            ));
        }
        //println!("udl_relative_files_path: {:?}", udl_relative_files_path);

        CargoPackage {
            swift_package_name: xcode_framework_name,
            swift_package_path: xcode_framework_path,
            crate_name: crate_package_name,
            target_name: crate_lib_name,
            lib_src_path,
            crate_manifest_path: crate_path.to_path_buf(),
            crate_dir: cargo_base_dir,
            cargo_relative_path_to_xcode_project,
            udl_absolute_files_path: udl_files_path,
            udl_relative_files_path,
            base_bundle_identifier: "com.example".to_string(),
        }
    }

    fn validate_cargo_and_return(
        crate_path: &Path,
        package_name: Option<&String>,
        lib_name: Option<&String>,
    ) -> (String, String, PathBuf) {
        let mut cargo_cmd = cargo_metadata::MetadataCommand::new();
        cargo_cmd.no_deps();
        cargo_cmd.manifest_path(crate_path);

        let mut cargo_metadata = match cargo_cmd.exec() {
            Ok(m) => m,
            Err(e) => {
                eprintln!(
                    "{} error: Can't parse cargo metadata in {:?} because: {}",
                    line!(), &crate_path, e
                );
                exit(1);
            }
        };

        let mut cargo_package = match package_name {
            Some(name) => {
                cargo_metadata.packages.retain(|f| f.name == *name);
                match cargo_metadata.packages.into_iter().next() {
                    Some(package) => package,
                    None => {
                        eprintln!(
                            "\nCheck your project Cargo.toml file or Specify a correct path.\n{} '{}' {} {:?}",
                            "Specified package name".red(),
                            name.bold(),
                            "not found in your Cargo.toml file at:".red(),
                            crate_path
                        );
                        exit(1)
                    }
                }
            }
            None => {
                match cargo_metadata.packages.into_iter().next() {
                    Some(package) => package,
                    None => {
                        eprintln!("{}\nCheck your project Cargo.toml file or Specify path to the correct \
                file or folder.\nPath:{:?}", "No Rust package found in your Cargo.toml file.".red(),
                                  crate_path);
                        exit(1);
                    }
                }
            }
        };


        let cargo_target = match lib_name {
            Some(ln) => {
                cargo_package.targets.retain(|f| f.name == *ln);
                if cargo_package.targets.is_empty() {
                    eprintln!(
                        "\n{} {} {}",
                        "No library named: ".red(),
                        ln.yellow().underline(),
                        "found in all targets.".red()
                    );
                    eprintln!("Check provided library name and try again.\n");
                    exit(1);
                }
                CargoPackage::target_is_valid(
                    cargo_package.targets,
                    &cargo_package.name,
                    crate_path,
                )
            }
            None => CargoPackage::target_is_valid(
                cargo_package.targets,
                &cargo_package.name,
                crate_path,
            ),
        };
        //println!("Detected package: {:?}", cargo_target);
        (
            cargo_package.name,
            cargo_target.name,
            PathBuf::from(cargo_target.src_path),
        )
    }

    fn target_is_valid(
        mut targets: Vec<Target>,
        package_name: &str,
        cargo_manifest_path: &Path,
    ) -> Target {
        targets.retain(|f| f.kind.iter().any(|x| (x == "staticlib" || x == "lib")));
        if targets.is_empty() {
            eprintln!("\nNo target of type: [\"lib\"] found in cargo package.name: '{}' in file:{:?} \n{}\n\n",
                      package_name.blue(), cargo_manifest_path, "Xcode framework project could only be created for Cargo 'library' targets.".red().bold());
            exit(1);
        }
        targets.retain(|f| f.crate_types.contains(&String::from("staticlib")));
        match targets.into_iter().next() {
            Some(some_target) => some_target,
            None => {
                eprintln!("No crate_type of type: [\"staticlib\"] found in cargo package.name: '{}' in file:{:?} \n{}\n\n",
                          package_name.blue(), cargo_manifest_path, ". Xcode framework project could only be created for Cargo library with crate_type containing 'staticlib'.".red().bold());
                exit(1);
            }
        }
    }

    fn find_udl_files(lib_src_path: &Path) -> Vec<(PathBuf, String)> {
        let expected_udl_files_path = if lib_src_path.ends_with("Cargo.toml") {
            lib_src_path
                .parent()
                .expect("Cargo.toml file has no Parent folder !!!!!!!!")
                .join("src")
        } else {
            lib_src_path.join("src")
        };

        let udl_files_path: Vec<(PathBuf, String)> =
            match std::fs::read_dir(expected_udl_files_path) {
                Ok(paths) => {
                    paths
                        .filter(|f| f.is_ok())
                        .map(|f| f.expect("Failed to convert Directory Entry to path").path())
                        .map(|f| {
                            match (f.file_name(), f.extension()) {
                                (Some(file_name), Some(ext)) => {
                                    if ext == OsStr::new("udl") {
                                        //println!("path: {:?}", f);
                                        let m = file_name
                                            .to_str()
                                            .expect("Unable to get file name from UDL file path")
                                            .to_string();
                                        (Some(f.to_path_buf()), Some(m))
                                    } else {
                                        (None, None)
                                    }
                                }
                                _ => (None, None),
                            }
                        })
                }
                Err(e) => {
                    eprintln!(
                        "Unable to open existing Xcode Project Directory at: {:?}\nError: {:?}",
                        lib_src_path, e
                    );
                    exit(1);
                }
            }
                .filter(|(a, _)| a.is_some())
                .map(|(a, b)| {
                    (
                        a.expect("UDL file paths must not be empty"),
                        b.expect("Unable to get file name from UDL file path"),
                    )
                })
                .collect::<Vec<(PathBuf, String)>>();
        if udl_files_path.is_empty() {
            eprintln!("\n{}\n", "No UDL files found.".red());
            eprintln!(
                "Refer to Rust UniFFI: {}\n\n",
                "https://mozilla.github.io/uniffi-rs/udl_file_spec.html"
                    .underline()
                    .bright_blue()
            );
            exit(1);
        }

        udl_files_path
    }
}
