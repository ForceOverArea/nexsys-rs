mod conditionals;
mod duplicate;

use geqslib::shunting::ContextHashMap;
use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, error::Error};
use crate::{units::{convert, const_data}, errors::ConstFormatError};

pub use conditionals::*;
pub use duplicate::*;

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

/// Identifies and returns guess values found in a Nexsys-legal string.
pub fn guess_values(text: &str) -> (String, HashMap<String, f64>)
{
    lazy_static!
    {
        static ref RE: Regex = Regex::new(r"(?i)guess (-?[0-9]+\.?[0-9]*) for ([a-z][a-z0-9_]*)").unwrap();
    }
    let mut res = (text.to_owned(), HashMap::new());
    let guesses = RE.captures_iter(text);

    for g in guesses 
    {
        res.0 = res.0.replace(g.get(0).unwrap().as_str(), "");
        res.1.insert(
            g.get(1).unwrap().as_str().to_owned(), 
            g.get(2).unwrap().as_str().parse()
                .expect("failed to parse number in guess declaration")
        );
    }
    res
}

/// Identifies and returns domains found in a Nexsys-legal string.
pub fn domains(text: &str) -> (String, HashMap<String, [f64; 2]>)
{
    lazy_static!
    {
        static ref RE: Regex = Regex::new(r"(?i)keep ([a-z][a-z0-9_]*) on \[(-? ?[0-9]+\.?[0-9]*), ?(-? ?[0-9]+\.?[0-9]*)\]").unwrap();
    }
    let mut res = (text.to_owned(), HashMap::new());
    let domains = RE.captures_iter(text);

    for d in domains 
    {
        res.0.replace(d.get(0).unwrap().as_str(), "");
        res.1.insert(
            d.get(1).unwrap().as_str().to_owned(),
            [d.get(2).unwrap().as_str().parse()
                .expect("failed to parse number in domain declaration"),
            d.get(3).unwrap().as_str().parse()
                .expect("failed to parse number in domain declaration")]
        );
    }
    res
}

/// Identifies and removes comments found in a Nexsys-legal string.
pub fn comments(text: &str) -> String 
{
    lazy_static! 
    {
        static ref RE: Regex = Regex::new(r#"(?i)".*?""#).unwrap();
    }
    let mut output = text.to_string();

    for f in RE.find_iter(text).map(|i| i.as_str()) 
    {
        output = output.replace(f, "");
    }

    output
}

/// Identifies and replaces any unit conversion tokens in a Nexsys-legal string.
pub fn conversions(text: &str) -> Result<String, Box<dyn Error>> {
    lazy_static! 
    {
        static ref RE: Regex = Regex::new(r"(?i)\[[a-z0-9_^/-]+->[a-z0-9_^/-]+\]").unwrap();
    }

    let mut output = text.to_string();

    let res: Vec<&str> = RE.find_iter(text).map(|i| i.as_str()).collect();

    for m in res 
    {
        let pre = m.replace(['[', ']'], "");
        
        let args: Vec<&str> = pre.split("->").collect();
        
        output = output.replace(m, 
            &format!("{}", convert(args[0], args[1])? )
        );
    }

    Ok(output)
}

/// Identifies and replaces any constants in a Nexsys-legal string.
pub fn consts(text: &str) -> Result<String, Box<dyn Error>> 
{
    lazy_static! 
    {
        static ref RE: Regex = Regex::new(r"(?i)#[a-z_]+").unwrap();
        static ref CONSTS: HashMap<String, f64> = const_data();
    }

    let mut output = text.to_string();

    for m in RE.find_iter(text).map(|i| i.as_str()) 
    {
        if let Some(c) = CONSTS.get(&m.to_string()) 
        {
            output = output.replace(m, &c.to_string());
        } 
        else 
        {
            return Err(Box::new(ConstFormatError))
        }
    }
    Ok(output)
}

/// Wraps most functions in `nexsys::parsing`, returning either an error that 
/// prevents the code from being solvable or the intermediate language representation
/// of the `.nxs`-formatted code
pub fn compile(code: &str, ctx: &mut ContextHashMap, declared: &mut HashMap<String, [f64; 3]>) -> Result<String, Box<dyn Error>> {
    
    let sys_domains: HashMap<String, [f64; 2]>;
    let sys_guesses: HashMap<String, f64>;
    
    let mut nil = comments(code); 
    (nil, sys_domains) = domains(&nil);
    for (var, bounds) in sys_domains
    {
        if let Some(var_info) = declared.get_mut(&var)
        {
            var_info[1] = bounds[0];
            var_info[2] = bounds[1];
        }
        else 
        {
            declared.insert(var, [1.0, bounds[0], bounds[1]]);
        }
    }

    (nil, sys_guesses) = guess_values(&nil);
    for (var, guess) in sys_guesses
    {
        if let Some(var_info) = declared.get_mut(&var)
        {
            var_info[0] = guess;
        }
        else 
        {
            declared.insert(var, [guess, f64::NEG_INFINITY, f64::INFINITY]);
        }
    }

    nil = consts(&nil)?;

    nil = conversions(&nil)?;

    nil = conditionals(&nil)?;

    Ok(nil)
}