/// Normalizes user input into a local `.dock` domain.
pub fn normalize_domain(input: &str) -> String {
    let value = input.trim().to_lowercase();
    let value = value
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_start_matches("www.");
    let value = value
        .trim_end_matches('/')
        .trim_end_matches(".com")
        .trim_end_matches(".net")
        .trim_end_matches(".local")
        .trim_end_matches(".dock");

    format!("{value}.dock")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_domain_adds_dock_suffix() {
        assert_eq!(normalize_domain("myapp"), "myapp.dock");
        assert_eq!(normalize_domain("myapp.dock"), "myapp.dock");
        assert_eq!(normalize_domain("http://myapp.com"), "myapp.dock");
        assert_eq!(normalize_domain("www.myapp.net/"), "myapp.dock");
    }
}
