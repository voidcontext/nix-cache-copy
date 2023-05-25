use regex::Regex;

use crate::{BinaryCache, DrvFile, StorePath};

#[derive(Debug, PartialEq)]
pub enum Line {
    Info(String),
    Copied(String, StorePath, BinaryCache),
    Built(String, DrvFile),
}

impl Line {
    pub fn parse(s: &str) -> Line {
        let copy_regex = Regex::new(r"^copying path '(.*?)' from '(.*)'...$").unwrap();
        let caps = copy_regex.captures(s);

        if let Some(caps) = caps {
            Line::Copied(
                s.to_string(),
                StorePath::from(String::from(caps.get(1).unwrap().as_str())),
                BinaryCache::from(String::from(caps.get(2).unwrap().as_str())),
            )
        } else {
            let build_regex = Regex::new(r"^building '(.*?)'...$").unwrap();
            let caps = build_regex.captures(s);

            if let Some(caps) = caps {
                Line::Built(
                    s.to_string(),
                    DrvFile::from(String::from(caps.get(1).unwrap().as_str())),
                )
            } else {
                Line::Info(s.to_string())
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{parser::Line, BinaryCache, DrvFile, StorePath};

    #[test]
    fn parse_should_parse_artbitrary_line_as_info() {
        let string = String::from("these 26 derivations will be built:");
        assert_eq!(Line::parse(&string), Line::Info(string));
    }

    #[test]
    fn parse_should_parse_copied_store_paths() {
        let string = String::from("copying path '/nix/store/vnwdak3n1w2jjil119j65k8mw1z23p84-glibc-2.35-224' from 'https://cache.nixos.org'...");
        assert_eq!(
            Line::parse(&string),
            Line::Copied(
                string,
                StorePath::from(String::from(
                    "/nix/store/vnwdak3n1w2jjil119j65k8mw1z23p84-glibc-2.35-224"
                )),
                BinaryCache::from(String::from("https://cache.nixos.org"))
            )
        );
    }

    #[test]
    fn parse_should_parse_built_derivations() {
        let string = String::from(
            "building '/nix/store/kwd8mkkl1sv3n5z9jf8447gr9g299pmp-nix-cache-copy-0.1.0.drv'...",
        );
        assert_eq!(
            Line::parse(&string),
            Line::Built(
                string,
                DrvFile::from(String::from(
                    "/nix/store/kwd8mkkl1sv3n5z9jf8447gr9g299pmp-nix-cache-copy-0.1.0.drv"
                ))
            )
        );
    }
}
