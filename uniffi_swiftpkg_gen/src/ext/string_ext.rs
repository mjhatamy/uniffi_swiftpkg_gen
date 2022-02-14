use super::*;

impl StringExtension for String {
    fn build_whitespaces(tab_count: u32) -> Self {
        (0..tab_count*4).map(|_| " ").collect::<Self>()
    }
}