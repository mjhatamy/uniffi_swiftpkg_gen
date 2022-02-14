use std::process::{ Output };
use super::*;

impl CommandOutputTrait for Output {
    fn utf8_string(&self) -> Vec<String> {
        String::from_utf8(
            self.stdout.to_vec()).unwrap()
            .lines()
            .map(|f| f.to_string())
            .collect()
    }

    // fn stream_out(&self) -> Vec<String> {
    //     let stdout = self.as_mut().unwrap();
    //     let stdout_reader = BufReader::new(stdout);
    //     let stdout_lines = stdout_reader.lines();
    //     // String::from_utf8(
    //     //     self.stdout.to_vec()).unwrap()
    //     //     .lines()
    //     //     .map(|f| f.to_string())
    //     //     .collect()
    // }


}