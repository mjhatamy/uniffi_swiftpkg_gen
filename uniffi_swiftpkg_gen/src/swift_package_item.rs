use std::path::{Path, PathBuf};
use colored::Colorize;
use std::process::{exit};
use crate::ext::*;
use pathdiff::diff_paths;
use super::xc_framework_generator::*;
use super::command_builder::*;

#[derive(Debug)]
pub(crate) struct SwiftPackageItem {
    framework_item: XCFrameworkItem,
    command: CommandBuilder,
}

impl SwiftPackageItem {
    /// path: Path to the directory to create Swift Package
    pub(crate) fn new(framework_item: XCFrameworkItem) -> Self {
        let command =  CommandBuilder::new();
        SwiftPackageItem {
            framework_item,
            command
        }
    }

    #[allow(unused)]
    pub(crate) fn build(mut self) -> Self {
        self.swift_package_dir()
            .create_dir_if_not_exist("Unable to create Swift Package directory.");
        self.sources_base_dir()
            .create_dir_if_not_exist("Unable to create Swift Package Sources directory.");
        self.tests_base_dir()
            .create_dir_if_not_exist("Unable to create Swift Package Tests directory.");

        self.sources_package_dir()
            .create_dir_if_not_exist("Unable to create Swift Package Sources lib directory.");
        self.tests_package_dir()
            .create_dir_if_not_exist("Unable to create Swift Package Tests lib directory.");

        self.headers_dir()
            .create_dir_if_not_exist("Unable to create Swift Package Headers directory.");

        self.resources_dir()
            .create_dir_if_not_exist("Unable to create Swift Package Resources directory.");

        self.package_swift_uri().write_file(self.build_package_swift_file_contents(), true)
            .expect("Generating Swift Package file failed");
        self.tests_swift_file_uri().write_file(self.build_tests_swift_file_contents(), false)
            .expect("Generating Swift Package Tests file failed");

        self.package_readme_md().write_file(self.build_readme_md_content(), false);


        // copy Required files
        let status = self.command.args_stream([
            format!("cp -R {}/* {}/",
                    self.framework_item.headers_path.to_str().unwrap(),
                    self.headers_dir().to_str().unwrap())]);
        if !status.success() {
            eprintln!("{}{}", "Copying headers files failed. ".red(), status);
            exit(1);
        }

        let status = self.command.args_stream([
            format!("cp -R {}/* {}/",
                    self.framework_item.swift_files_path.to_str().unwrap(),
                    self.sources_package_dir().to_str().unwrap())]);
        if !status.success() {
            eprintln!("{}{}", "Copying swift files failed. ".red(), status);
            exit(1);
        }

        let status = self.command.args_stream([
            format!("rm -rf {1}/{2}; cp -R {} {}/",
                    self.framework_item.xc_frameworks_uri.to_str().unwrap(),
                    self.resources_dir().to_str().unwrap(),
                    self.framework_item.xc_frameworks_uri.file_name().unwrap().to_str().unwrap())]);
        if !status.success() {
            eprintln!("{}{}", "Copying xc framework files failed. ".red(), status);
            exit(1);
        }

        self
    }

    fn swift_package_dir(&self) -> PathBuf {
        Path::new(
            format!("{}{}", &self.framework_item.swift_package_build_path.to_str().unwrap(),
                    if self.framework_item.build_type == BuildType::Debug { "_debug" } else { "" })
                .as_str()
        ).to_path_buf()
    }

    #[allow(unused)]
    fn package_readme_md(&self) -> PathBuf {
        self.swift_package_dir().join("README.md")
    }

    #[allow(unused)]
    fn package_swift_uri(&self) -> PathBuf {
        self.swift_package_dir().join("Package.swift")
    }

    #[allow(unused)]
    fn tests_swift_file_uri(&self) -> PathBuf {
        self.tests_package_dir().join(format!("{}Tests.swift", self.framework_item.swift_package_name))
    }

    #[allow(unused)]
    fn headers_dir(&self) -> PathBuf {
        self.sources_package_dir()
            .join("Headers")
    }

    #[allow(unused)]
    fn resources_dir(&self) -> PathBuf {
        self.sources_package_dir()
            .join("Resources")
    }

    #[allow(unused)]
    fn sources_package_dir(&self) -> PathBuf {
        self.swift_package_dir()
            .join("Sources")
            .join(&self.framework_item.swift_package_name)
    }

    #[allow(unused)]
    fn tests_package_dir(&self) -> PathBuf {
        self.swift_package_dir()
            .join("Tests")
            .join(format!("{}Tests", &self.framework_item.swift_package_name))
    }

    #[allow(unused)]
    fn sources_base_dir(&self) -> PathBuf {
        self.swift_package_dir().join("Sources")
    }

    #[allow(unused)]
    fn tests_base_dir(&self) -> PathBuf {
        self.swift_package_dir().join("Tests")
    }

    #[allow(unused)]
    fn build_package_swift_file_contents(&self) -> String {
        let mut content = String::new();
        content.push_str("// swift-tools-version:5.5\n");
        content.push_str("// The swift-tools-version declares the minimum version of Swift required to build this package.\n");
        content.push_str(format!("// Swift Package: {}\n\n", self.framework_item.swift_package_name).as_str());
        content.push_str("import PackageDescription;\n\n");
        content.push_str("let package = Package(\n");
        content.push_str(format!("{}name: \"{}\",\n", String::build_whitespaces(1), self.framework_item.swift_package_name).as_str());
        content.push_str(format!("{}platforms: [\n", String::build_whitespaces(1)).as_str());
        content.push_str(format!("{}.iOS(.v13),\n", String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}.macOS(SupportedPlatform.MacOSVersion.v10_10)\n", String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}],\n", String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}products: [\n", String::build_whitespaces(1)).as_str());
        content.push_str(format!("{}.library(\n", String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}name: \"{}\",\n", String::build_whitespaces(3), self.framework_item.swift_package_name).as_str());
        content.push_str(format!("{}targets: [\"{}\"]\n", String::build_whitespaces(3), self.framework_item.swift_package_name).as_str());
        content.push_str(format!("{})\n", String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}],\n", String::build_whitespaces(1)).as_str());
        content.push_str(format!("{}dependencies: [\n", String::build_whitespaces(1)).as_str());
        content.push_str(format!("{}// Dependencies declare other packages that this package depends on.\n", String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}// .package(url: /* package url */, from: \"1.0.0\"),\n", String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}],\n", String::build_whitespaces(1)).as_str());
        content.push_str(format!("{}targets: [\n", String::build_whitespaces(1)).as_str());

        content.push_str(format!("{}.systemLibrary(name: \"{}\", path: \"./{}\", pkgConfig: nil, providers: []),\n",
                                 String::build_whitespaces(2),
                                 self.framework_item.swift_package_name.to_lowercase(),
                                 diff_paths(&self.headers_dir(), &self.swift_package_dir() )
                                     .expect("Unable to get headers directory differential path.").to_str()
                                     .expect("Unable to unwrap string"),
        ).as_str());

        content.push_str(format!("{}.binaryTarget(name: \"lib_{}_xc\", path: \"./{}/{}\"),\n",
                                 String::build_whitespaces(2),
                                 self.framework_item.swift_package_name.to_lowercase(),
                                 diff_paths(&self.resources_dir(), &self.swift_package_dir())
                                     .expect("Unable to get headers directory differential path.").to_str()
                                     .expect("Unable to unwrap string"),
            self.framework_item.xc_frameworks_uri.file_name().unwrap().to_str().unwrap()
        ).as_str());

        content.push_str(format!("{}.target(\n", String::build_whitespaces(2)).as_str());

        content.push_str(format!("{}name: \"{}\",\n",
                                 String::build_whitespaces(3),
                                 self.framework_item.swift_package_name).as_str());

        content.push_str(format!("{}dependencies: [\n", String::build_whitespaces(3)).as_str());

        // Define system library here
        content.push_str(format!("{}.target(name: \"{}\", condition: .when(platforms: [.iOS, .macOS])),\n",
                                 String::build_whitespaces(4),
                                 self.framework_item.swift_package_name.to_lowercase())
            .as_str());

        // Define binaryTarget dependency
        content.push_str(format!("{}.target(name: \"lib_{}_xc\")\n",
                                 String::build_whitespaces(4),
                                 self.framework_item.swift_package_name.to_lowercase())
            .as_str());

        content.push_str(format!("{}],\n", String::build_whitespaces(3)).as_str());

        content.push_str(format!("{}cxxSettings: [.headerSearchPath(\"Headers\")]\n", String::build_whitespaces(3)).as_str());

        content.push_str(format!("{}),\n", String::build_whitespaces(2)).as_str());

        content.push_str(format!("{}.testTarget(\n", String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}name: \"{}Tests\",\n",
                                 String::build_whitespaces(3),
                                 self.framework_item.swift_package_name
        ).as_str());
        content.push_str(format!("{}dependencies: [\"{}\"]\n",
                                 String::build_whitespaces(3),
                                 self.framework_item.swift_package_name
        ).as_str());

        content.push_str(format!("{}),\n", String::build_whitespaces(2)).as_str());

        content.push_str(format!("{}]\n", String::build_whitespaces(1)).as_str());
        content.push_str(")\n");

        content
    }

    #[allow(unused)]
    fn build_tests_swift_file_contents(&self) -> String {
        let mut content = String::new();
        content.push_str("import XCTest\n");
        content.push_str(format!("@testable import  {}\n\n", self.framework_item.swift_package_name).as_str());
        content.push_str(format!("final class {}Tests: XCTestCase {{ \n", self.framework_item.swift_package_name).as_str());

        content.push_str(format!("{}func testExample() throws {{ \n", String::build_whitespaces(1)).as_str());
        content.push_str(format!("{}// This is an example of a functional test case.\n", String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}// Use XCTAssert and related functions to verify your tests produce the correct\n", String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}// results.\n", String::build_whitespaces(2)).as_str());
        content.push_str(format!("{} }}\n", String::build_whitespaces(1)).as_str());
        content.push_str("}\n");
        content
    }

    #[allow(unused)]
    fn build_readme_md_content(&self) -> String {
        let mut content = String::new();
        content.push_str(format!("# {}\n\n", self.framework_item.swift_package_name).as_str());
        content.push_str("A description of this package.\n");
        content
    }
}