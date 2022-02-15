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

        self.package_swift_uri()
            .write_file(self.build_package_swift_file_contents(), true)
            .expect("Generating Swift Package file failed");
        self.tests_swift_file_uri()
            .write_file(self.build_tests_swift_file_contents(), false)
            .expect("Generating Swift Package Tests file failed");

        self.package_readme_md().write_file(self.build_readme_md_content(), false);

        self.git_ignore_file().write_file(self.build_git_ignore_file(), false);

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

        // Copy XC Framework
        let status = self.command.args_stream([
            format!("rm -rf {1}/{2}; cp -R {0} {1}/",
                    self.framework_item.xc_frameworks_uri.to_str().unwrap(),
                    self.swift_package_dir().to_str().unwrap(),
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

    fn git_ignore_file(&self) -> PathBuf {
        self.swift_package_dir().join(".gitignore")
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
        content.push_str(format!("{}name: \"{}\",\n", String::build_whitespaces(1),
                                 self.framework_item.swift_package_name).as_str());
        content.push_str(format!("{}platforms: [\n",
                                 String::build_whitespaces(1)).as_str());
        content.push_str(format!("{}.iOS(.v13),\n",
                                 String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}.macOS(SupportedPlatform.MacOSVersion.v10_10)\n",
                                 String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}],\n",
                                 String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}products: [\n",
                                 String::build_whitespaces(1)).as_str());
        content.push_str(format!("{}.library(\n",
                                 String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}name: \"{}\",\n",
                                 String::build_whitespaces(3),
                                 self.framework_item.swift_package_name).as_str());
        content.push_str(format!("{}targets: [\"{}\"]\n",
                                 String::build_whitespaces(3),
                                 self.framework_item.swift_package_name).as_str());
        content.push_str(format!("{})\n", String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}],\n", String::build_whitespaces(1)).as_str());
        content.push_str(format!("{}dependencies: [\n",
                                 String::build_whitespaces(1)).as_str());
        content.push_str(format!("{}// Dependencies declare other packages that this package depends on.\n",
                                 String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}// .package(url: /* package url */, from: \"1.0.0\"),\n",
                                 String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}],\n", String::build_whitespaces(1)).as_str());
        content.push_str(format!("{}targets: [\n",
                                 String::build_whitespaces(1)).as_str());

        content.push_str(format!("{}.systemLibrary(name: \"sys_lib_{}\", path: \"./{}\", pkgConfig: nil, providers: []),\n",
                                 String::build_whitespaces(2),
                                 self.framework_item.swift_package_name.to_lowercase(),
                                 diff_paths(&self.headers_dir(), &self.swift_package_dir() )
                                     .expect("Unable to get headers directory differential path.").to_str()
                                     .expect("Unable to unwrap string"),
        ).as_str());

        content.push_str(format!("{}.binaryTarget(name: \"lib_{}_xc\", path: \"./{}\"),\n",
                                 String::build_whitespaces(2),
                                 self.framework_item.swift_package_name.to_lowercase(),
                                 // diff_paths(&self.resources_dir(), &self.swift_package_dir())
                                 //     .expect("Unable to get headers directory differential path.").to_str()
                                 //     .expect("Unable to unwrap string"),
            self.framework_item.xc_frameworks_uri.file_name().unwrap().to_str().unwrap()
        ).as_str());

        content.push_str(format!("{}.target(\n",
                                 String::build_whitespaces(2)).as_str());

        content.push_str(format!("{}name: \"{}\",\n",
                                 String::build_whitespaces(3),
                                 self.framework_item.swift_package_name).as_str());

        content.push_str(format!("{}dependencies: [\n",
                                 String::build_whitespaces(3)).as_str());

        // Define system library here
        content.push_str(format!("{}.target(name: \"sys_lib_{}\", condition: .when(platforms: [.iOS, .macOS])),\n",
                                 String::build_whitespaces(4),
                                 self.framework_item.swift_package_name.to_lowercase()).as_str());

        // Define binaryTarget dependency
        content.push_str(format!("{}.target(name: \"lib_{}_xc\")\n",
                                 String::build_whitespaces(4),
                                 self.framework_item.swift_package_name.to_lowercase()).as_str());

        content.push_str(format!("{}],\n", String::build_whitespaces(3)).as_str());

        content.push_str(format!("{}cxxSettings: [.headerSearchPath(\"Headers\")]\n",
                                 String::build_whitespaces(3)).as_str());

        content.push_str(format!("{}),\n", String::build_whitespaces(2)).as_str());

        content.push_str(format!("{}.testTarget(\n",
                                 String::build_whitespaces(2)).as_str());
        content.push_str(format!("{}name: \"{}Tests\",\n",
                                 String::build_whitespaces(3),
                                 self.framework_item.swift_package_name).as_str());
        content.push_str(format!("{}dependencies: [\"{}\"]\n",
                                 String::build_whitespaces(3),
                                 self.framework_item.swift_package_name).as_str());

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

    fn build_git_ignore_file(&self) -> String {
        let mut content = String::new();
        content.push_str(".DS_Store\n");
        content.push_str("**/.DS_Store\n");
        content.push_str("*.xcscmblueprint\n");
        content.push_str("# c.f. http://www.westwind.com/reference/os-x/invisibles.html\n");
        content.push_str(".Trashes\n");
        content.push_str("# c.f. http://www.westwind.com/reference/os-x/invisibles.html\n");
        content.push_str("*.swp\n");
        content.push_str(r#"#

*.swiftpm
.swiftpm
**/.swiftpm

# *.lock - this is used and abused by many editors for many different things.
#    For the main ones I use (e.g. Eclipse), it should be excluded
#    from source-control, but YMMV.
#   (lock files are usually local-only file-synchronization on the local FS that should NOT go in git)
# c.f. the "OPTIONAL" section at bottom though, for tool-specific variations!
#
# In particular, if you're using CocoaPods, you'll want to comment-out this line:
*.lock
#
# profile - REMOVED temporarily (on double-checking, I can't find it in OS X docs?)
#profile

####
# Xcode temporary files that should never be committed
#
# NB: NIB/XIB files still exist even on Storyboard projects, so we want this...

*~.nib


####
# Xcode build files -
#
# NB: slash on the end, so we only remove the FOLDER, not any files that were badly named "DerivedData"

DerivedData/

# NB: slash on the end, so we only remove the FOLDER, not any files that were badly named "build"

build/

#####
# Xcode private settings (window sizes, bookmarks, breakpoints, custom executables, smart groups)
#
# This is complicated:
#
# SOMETIMES you need to put this file in version control.
# Apple designed it poorly - if you use "custom executables", they are
#  saved in this file.
# 99% of projects do NOT use those, so they do NOT want to version control this file.
#  ..but if you're in the 1%, comment out the line "*.pbxuser"

# .pbxuser: http://lists.apple.com/archives/xcode-users/2004/Jan/msg00193.html

*.pbxuser

# .mode1v3: http://lists.apple.com/archives/xcode-users/2007/Oct/msg00465.html

*.mode1v3

# .mode2v3: http://lists.apple.com/archives/xcode-users/2007/Oct/msg00465.html

*.mode2v3

# .perspectivev3: http://stackoverflow.com/questions/5223297/xcode-projects-what-is-a-perspectivev3-file

*.perspectivev3

#    NB: also, whitelist the default ones, some projects need to use these
!default.pbxuser
!default.mode1v3
!default.mode2v3
!default.perspectivev3

####
# Xcode 4 - semi-personal settings
#
# Apple Shared data that Apple put in the wrong folder
# c.f. http://stackoverflow.com/a/19260712/153422
#     FROM ANSWER: Apple says "don't ignore it"
#     FROM COMMENTS: Apple is wrong; Apple code is too buggy to trust; there are no known negative side-effects to ignoring Apple's unofficial advice and instead doing the thing that actively fixes bugs in Xcode
# Up to you, but ... current advice: ignore it.
*.xccheckout

#
#
# OPTION 1: ---------------------------------
#     throw away ALL personal settings (including custom schemes!
#     - unless they are "shared")
# As per build/ and DerivedData/, this ought to have a trailing slash
#
# NB: this is exclusive with OPTION 2 below
xcuserdata/

# OPTION 2: ---------------------------------
#     get rid of ALL personal settings, but KEEP SOME OF THEM
#     - NB: you must manually uncomment the bits you want to keep
#
# NB: this *requires* git v1.8.2 or above; you may need to upgrade to latest OS X,
#    or manually install git over the top of the OS X version
# NB: this is exclusive with OPTION 1 above
#
#xcuserdata/**/*

#     (requires option 2 above): Personal Schemes
#
#!xcuserdata/**/xcschemes/*

####
# Xcode 4 workspaces - more detailed
#
# Workspaces are important! They are a core feature of Xcode - don't exclude them :)
#
# Workspace layout is quite spammy. For reference:
#
# /(root)/
#   /(project-name).xcodeproj/
#     project.pbxproj
#     /project.xcworkspace/
#       contents.xcworkspacedata
#       /xcuserdata/
#         /(your name)/xcuserdatad/
#           UserInterfaceState.xcuserstate
#     /xcshareddata/
#       /xcschemes/
#         (shared scheme name).xcscheme
#     /xcuserdata/
#       /(your name)/xcuserdatad/
#         (private scheme).xcscheme
#         xcschememanagement.plist
#
#

####
# Xcode 4 - Deprecated classes
#
# Allegedly, if you manually "deprecate" your classes, they get moved here.
#
# We're using source-control, so this is a "feature" that we do not want!

*.moved-aside

####
# OPTIONAL: Some well-known tools that people use side-by-side with Xcode / iOS development
#
# NB: I'd rather not include these here, but gitignore's design is weak and doesn't allow
#     modular gitignore: you have to put EVERYTHING in one file.
#
# COCOAPODS:
#
# c.f. http://guides.cocoapods.org/using/using-cocoapods.html#what-is-a-podfilelock
# c.f. http://guides.cocoapods.org/using/using-cocoapods.html#should-i-ignore-the-pods-directory-in-source-control
#
#!Podfile.lock
#
# RUBY:
#
# c.f. http://yehudakatz.com/2010/12/16/clarifying-the-roles-of-the-gemspec-and-gemfile/
#
#!Gemfile.lock
#
# IDEA:
#
# c.f. https://www.jetbrains.com/objc/help/managing-projects-under-version-control.html?search=workspace.xml
#
#.idea/workspace.xml
#
# TEXTMATE:
#
# -- UNVERIFIED: c.f. http://stackoverflow.com/a/50283/153422
#
#tm_build_errors

####
# UNKNOWN: recommended by others, but I can't discover what these files are
#
*/*.xcframework
**/*.xcframework
*.xcframework
*.a
**/*.a
"#);
        content
    }
}