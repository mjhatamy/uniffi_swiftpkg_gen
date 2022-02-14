const IOS_TARGETS: [&str; 2] = ["x86_64-apple-ios", "aarch64-apple-ios"];
const IOS_SIM_TARGETS: [&str; 2] = ["aarch64-apple-ios-sim", "x86_64-apple-ios"];
const MACOS_TARGETS: [&str; 2]= ["aarch64-apple-darwin", "x86_64-apple-darwin"];

use std::process::Command;

pub(crate) struct XCGenerator {
    default_shell: String,
    //host_arch: String
}

impl XCGenerator {
    #[allow(unused)]
    pub(crate) fn new() -> Self {
        let res = Command::new("sh")
            .args(["-c", "echo $SHELL"])
            .output()
            .expect("Unable to get default shell. Check your system if any shell is installed");
        let default_shell = String::from_utf8(res.stdout)
            .unwrap()
            .lines()
            .map(|f| f.to_string()).last()
            .unwrap_or_else(|| "".to_string());

        // let res = Command::new("sh")
        //     .args(["-c", "uname -m"])
        //     .output()
        //     .expect("Unable to get system architecture. Check your system if any shell is installed and 'uname -m' is working");
        // let host_arch = String::from_utf8(res.stdout)
        //     .unwrap()
        //     .lines()
        //     .map(|f| f.to_string()).last()
        //     .unwrap_or("".to_string());
        let inst = XCGenerator {
            default_shell,
            //host_arch
        };
        inst.init();
        inst
    }

    #[allow(unused)]
    fn init(&self) {
        let not_installed_targets = self.get_not_installed_targets();
        for target in not_installed_targets {
            self.install_rustup_target(target);
        }
    }

    #[allow(unused)]
    fn get_not_installed_targets(&self) -> Vec<String> {
        let res = Command::new(&self.default_shell)
            .args(["-c", "rustup target list --installed"])
            .output()
            .expect("failed to execute process");
        let installed_archs = String::from_utf8(res.stdout)
            .unwrap()
            .lines()
            .map(|f| f.to_string())
            .collect::<Vec<String>>();
        IOS_TARGETS.into_iter()
            .chain(IOS_SIM_TARGETS.into_iter())
            .chain(MACOS_TARGETS.into_iter())
            .filter(|f| !installed_archs.contains(&f.to_string()))
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
    }

    #[allow(unused)]
    fn install_rustup_target(&self, target: String) {
        Command::new(&self.default_shell)
            .args(["-c", format!("rustup target add {}", target).as_str()])
            .output()
            .unwrap_or_else(|_| panic!("Failed to install rust target: {}. Check your connection to internet ", target));
    }
}

#[cfg(test)]
mod tests {
    use super::XCGenerator;
    #[test]
    fn get_rust_up_targets_for() {
        let xc = XCGenerator::new();
        //xc.get_rust_up_targets_for();
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
