pub trait Optional<T> {
    fn as_option(&self) -> Option<T>;
}

impl Optional<String> for String {
    fn as_option(&self) -> Option<String> {
        match self.trim().is_empty() {
            true => None,
            false => Some(self.clone()),
        }
    }
}

pub fn is_eol(c: char) -> bool {
    c == '\n' || c == '\r'
}
