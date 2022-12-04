use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

/// Identifies and returns variables found in a Nexsys-legal string.
pub fn legal_variable(text: &str) -> Vec<String> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?i)[a-z](?:[a-z0-9_]+)?").unwrap();
    }
    let raw = RE.find_iter(text).map(|i| i.as_str()).collect::<Vec<&str>>();
    let mut res = vec![];

    for i in raw {
        let var = i.to_string();
        if !res.contains(&var) {
            res.push(var)
        }
    }
    res
}

/// Identifies and returns guess values found in a Nexsys-legal string.
pub fn guess_values(text: &str) -> HashMap<String, f64> {
    lazy_static!{
        static ref RE: Regex = Regex::new(r"(?i)guess -?[0-9]+ for [a-z](?:[a-z0-9_]+)?").unwrap();
    }
    let mut res = HashMap::new();
    let guesses = RE.find_iter(text)
        .map(
            |i| {
                i.as_str().split(" ").collect::<Vec<&str>>()
            }
        ).collect::<Vec<Vec<&str>>>();

    for g in guesses {
        res.insert(g[3].to_string(), g[1].parse().unwrap());
    }
    res
}

/// Identifies and returns domains found in a Nexsys-legal string.
pub fn domains(text: &str) -> HashMap<String, [f64; 2]> {
    lazy_static!{
        static ref RE: Regex = Regex::new(r"(?i)keep [a-z](?:[a-z0-9_]+)? on \[-?[0-9.]+, ?-?[0-9.]+\]").unwrap();
    }
    let mut res = HashMap::new();
    let domains = RE.find_iter(text)
        .map(
            |i| {
                i.as_str().split(" ").collect::<Vec<&str>>()
            }
        ).collect::<Vec<Vec<&str>>>();

    for d in domains {
        let terms = d;
        let v = terms[1].to_string();
        // println!("Found bounded variable: {}",v);

        let bound_string: String;
        if terms.len() == 4 {
            bound_string = terms[3].to_string();
        } else if terms.len() == 5 {
            bound_string = [terms[3], terms[4]].join(" ");
        } else {
            panic!("expected 4 or 5 terms in domain specification, found {}", terms.len())
        }
       
        let bounds = bound_string
        .replace("[","")
        .replace("]","")
        .split(", ")
        .map(|i| i.parse().unwrap())
        .collect::<Vec<f64>>();

        res.insert(v, [bounds[0], bounds[1]]);
    }
    res
}
