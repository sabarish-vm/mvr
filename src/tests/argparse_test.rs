#[cfg(test)]
use super::*;
#[test]
#[should_panic]
fn test_empty_set() {
    {
        let args = vec!["-m", "(.*)", "$1.bak", ""];
        let parser = build_parser();
        parser.try_get_matches_from(args).unwrap();
    }
    {
        let args = vec!["-m", "(.*)", "$1.bak", " "];
        let parser = build_parser();
        parser.try_get_matches_from(args).unwrap();
    }
}
