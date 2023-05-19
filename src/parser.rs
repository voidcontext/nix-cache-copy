#[derive(Debug, PartialEq)]
pub enum Line {
    Info(String),
    Copied(String),
    Built(String),
}

impl Line {
    pub fn parse(s: String) -> Line {
        if s.starts_with("copying path") {
            Line::Copied(s)
        } else if s.starts_with("building") {
            Line::Built(s)
        } else {
            Line::Info(s)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parser::Line;

    #[test]
    fn parse_should_parse_artbitrary_line_as_info() {
        let string = String::from("these 26 derivations will be built:");
        assert_eq!(Line::parse(string.clone()), Line::Info(string));
    }

    #[test]
    fn parse_should_parse_copied_store_paths() {
        let string = String::from("copying path '/nix/store/vnwdak3n1w2jjil119j65k8mw1z23p84-glibc-2.35-224' from 'https://cache.nixos.org'...");
        assert_eq!(Line::parse(string.clone()), Line::Copied(string));
    }

    #[test]
    fn parse_should_parse_built_derivations() {
        let string = String::from(
            "building '/nix/store/kwd8mkkl1sv3n5z9jf8447gr9g299pmp-nix-cache-copy-0.1.0.drv'...",
        );
        assert_eq!(Line::parse(string.clone()), Line::Built(string));
    }
}
