mod conditionals;

use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, error::Error};
use std::fs::read_to_string;
use crate::{solve, Variable, units::{convert, const_data}, errors::ConstFormatError};

pub use conditionals::*;

/// Removes a list of characters from a given `String`.
/// 
/// User be warned: under the hood this is done by 
/// repeatedly calling `.replace()`, which might not be 
/// desirable.
/// # Example
/// ```
/// use nexsys::cleanup;
/// 
/// let mut my_string = "Hello,_World!".to_string();
/// 
/// my_string = cleanup!(my_string, "_", ",", "!");
/// 
/// assert_eq!("HelloWorld".to_string(), my_string)
/// ```
#[macro_export]
macro_rules! cleanup {
    ( $i:expr, $( $ch:tt ),* ) => {{
        let mut out = $i;
        $(out = out.replace($ch, "");)*
        out
    }};
}

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
                i.as_str().split(' ').collect::<Vec<&str>>()
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
                i.as_str().split(' ').collect::<Vec<&str>>()
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
        .replace(['[', ']'], "")
        .split(", ")
        .map(|i| i.parse().unwrap())
        .collect::<Vec<f64>>();

        res.insert(v, [bounds[0], bounds[1]]);
    }
    res
}

/// Identifies and returns imports found in a Nexsys-legal string.
fn _imports(text: &str, tolerance: Option<f64>, max_iterations: Option<usize>, allow_nonconvergence: bool) -> HashMap<String, Variable> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?i)use [[a-z_\.-]+] ?-> ?.*").unwrap();
    }
    let statements = RE.find_iter(text);
    let mut imports = HashMap::new();

    for stmnt in statements {

        let raw_stmnt: Vec<&str> = stmnt.as_str().split(']').collect();

        let contents = read_to_string( raw_stmnt[0].replace('[', "") ).expect("failed to locate file");
        
        let clean_vars = cleanup!(raw_stmnt[1].to_string(), "->", ' ');
        let vars: Vec<&str> = clean_vars.split(',').collect();

        let soln = solve(&contents, tolerance, max_iterations, allow_nonconvergence)
        .expect("failed to solve imported system").0
        .into_iter()
        .filter(move |i| vars.contains(&i.0.as_str()));

        imports.extend(soln);
    }
    imports
}

/// Identifies and replaces include statements with external code
fn _includes(text: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?i)#include \[[a-z_\.-]+\]").unwrap();
    } 
    let mut output = text.to_string();

    for f in RE.find_iter(text).map(|i| i.as_str()) {
        let fp = cleanup!(f.to_string(), "#include [", ']');
        let code = read_to_string(fp).expect("failed to read file in #include statement");
        output = output.replace(f, code.as_str());
    }
    output
}

/// Identifies and removes comments found in a Nexsys-legal string.
pub fn comments(text: &str) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r#"(?i)".*?""#).unwrap();
    }
    let mut output = text.to_string();

    for f in RE.find_iter(text).map(|i| i.as_str()) {
        output = output.replace(f, "");
    }

    output
}

/// Identifies and replaces any unit conversion tokens in a Nexsys-legal string.
pub fn conversions(text: &str) -> Result<String, Box<dyn Error>> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?i)\[[a-z0-9_^/-]+->[a-z0-9_^/-]+\]").unwrap();
    }

    let mut output = text.to_string();

    let res: Vec<&str> = RE.find_iter(text).map(|i| i.as_str()).collect();

    for m in res {

        let pre = m.replace(['[', ']'], "");
        
        let args: Vec<&str> = pre.split("->").collect();
        
        output = output.replace(m, 
            &format!("{}", convert(args[0], args[1])? )
        );
    }

    Ok(output)
}

/// Identifies and replaces any constants in a Nexsys-legal string.
pub fn consts(text: &str) -> Result<String, Box<dyn Error>> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"(?i)#[a-z_]+").unwrap();
        static ref CONSTS: HashMap<String, f64> = const_data();
    }

    let mut output = text.to_string();

    for m in RE.find_iter(text).map(|i| i.as_str()) {
        if let Some(c) = CONSTS.get(&m.to_string()) {
            output = output.replace(m, &c.to_string());
        } else {
            return Err(Box::new(ConstFormatError))
        }
    }
    Ok(output)
}

/// Wraps most functions in `nexsys::parsing`, returning either an error that 
/// prevents the code from being solvable or the intermediate language representation
/// of the `.nxs`-formatted code
pub fn compile(code: &str) -> Result<String, Box<dyn Error>> {
    
    let mut nil = comments(code); 
    
    nil = conversions(&nil)?;
    
    nil = consts(&nil)?;
    
    conditionals(&nil)
}