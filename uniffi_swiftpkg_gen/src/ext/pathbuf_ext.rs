use std::path::{ PathBuf };
use super::build_file_structure_trait::*;
use std::fs::{ create_dir, File };
use std::io::Write;

impl BuildFileStructure for PathBuf {
    fn create_dir_if_not_exist(&self, message: &str) {
        if !self.exists() {
            create_dir(self)
                .unwrap_or_else(|_| panic!("{}.\nPath: {:?}\n", message, self));
        }
    }

    fn write_file(self, content: String, overwrite: bool) -> Result<(), std::io::Error> {
        if self.exists() && !overwrite {
            Ok(())
        } else {
            let file = File::create(self.as_path());
            match file {
                Ok(mut f) => f.write_all(content.as_bytes()),
                Err(e) => Err(e)
            }
        }
    }
}