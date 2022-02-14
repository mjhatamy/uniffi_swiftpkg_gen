
pub(crate) trait BuildFileStructure {
    fn create_dir_if_not_exist(&self, message: &str);
    fn write_file(self, content: String, overwrite: bool) -> Result<(), std::io::Error>;
}


pub(crate) trait StringExtension {
    fn build_whitespaces(count: u32) -> Self;
}

pub(crate) trait CommandOutputTrait {
    fn utf8_string(&self) -> Vec<String>;
}