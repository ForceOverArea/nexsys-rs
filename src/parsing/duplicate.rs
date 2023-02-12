use lazy_static::lazy_static;
use regex::Regex;

/// Duplicates a block of code, expanding it from one
/// block of equations to as many as the user specifies.
pub fn duplications(code:&str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(
r#"(?ms)^[ \t]duplicate (?i)[a-z][a-z0-9_]*(?-i) +[0-9]+, *[0-9]+ *:$
.*
^[ \t]end$"#
        ).unwrap();
    }

    for dupl in RE.find_iter(code).map(|i| i.as_str()) {
        let groups = dupl
            .split(':')
            .collect::<Vec<&str>>();

        let var = groups[0]
            .trim_start()
            .strip_prefix("duplicate ")
            .unwrap() // This is acceptable because `duplicate ` must be present to match regex
            .split(' ')
            .collect::<Vec<&str>>()[0];

        //now for each int in the range, add in the block with indices subbed in    
    }

    todo!()
}