mod conditionals;
// mod duplicate; TODO: need to polish this up.

use geqslib::shunting::{eval_str, ContextHashMap, ContextLike};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;
use crate::{units::{convert, const_data}, errors::ConstFormatError};

pub use conditionals::*;

const LEGAL_VAR_PATTERN: &str = r"[a-z][a-z0-9_]*";
const LEGAL_NUM_PATTERN: &str = r"-? ?[0-9]+\.?[0-9]*";

/// Replaces `"@N"` and `"@V"` literals with the nexsys-legal number and variable patterns, respectively.
fn nexsys_regex(pattern: &str) -> Regex
{
    Regex::new(&pattern
        .replace("@N", LEGAL_NUM_PATTERN)
        .replace("@V", LEGAL_VAR_PATTERN)
    ).unwrap()
}

/// Identifies and returns guess values found in a Nexsys-legal string.
pub fn guess_values(text: &str) -> (String, HashMap<String, f64>)
{
    lazy_static!
    {
        static ref RE: Regex = nexsys_regex(r"(?i)guess +(@N) +for +(@V)");
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
        static ref RE: Regex = nexsys_regex(r"(?i)keep +(@V) +on +\[ *(@N), *(@N) *\]");
    }
    let mut res = (text.to_owned(), HashMap::new());
    let domains = RE.captures_iter(text);

    for d in domains 
    {
        res.0 = res.0.replace(d.get(0).unwrap().as_str(), "");
        res.1.insert(
            d.get(1).unwrap().as_str().to_owned(),
            [d.get(2).unwrap().as_str().parse()
                .expect("failed to parse first number in domain declaration"),
            d.get(3).unwrap().as_str().parse()
                .expect("failed to parse second number in domain declaration")]
        );
    }
    res
}

/// Identifies and removes comments found in a Nexsys-legal string.
pub fn comments(text: &str) -> String 
{
    lazy_static! 
    {
        static ref RE: Regex = nexsys_regex(r"//[^\n]*");
    }
    let mut output = text.to_string();

    for f in RE.find_iter(text).map(|i| i.as_str()) 
    {
        output = output.replace(f, "");
    }

    output
}

/// Identifies and replaces any unit conversion tokens in a Nexsys-legal string.
pub fn conversions(text: &str) -> anyhow::Result<String> {
    lazy_static! 
    {
        static ref RE: Regex = nexsys_regex(r"(?i)\[[a-z0-9_^/-]+->[a-z0-9_^/-]+\]");
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
pub fn consts(text: &str) -> anyhow::Result<String> 
{
    lazy_static! 
    {
        static ref RE: Regex = nexsys_regex(r"(?i)#[a-z_]+");
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
            return Err(ConstFormatError.into())
        }
    }
    Ok(output)
}

pub fn const_values(text: &str) -> anyhow::Result<(String, HashMap<String, f64>)>
{
    lazy_static!
    {
        static ref RE: Regex = nexsys_regex(r"(?i)const +(@V) *= *(@N)");
    }
    let mut res = (text.to_owned(), HashMap::new());
    let const_vals = RE.captures_iter(text);

    for c in const_vals
    {
        res.0 = res.0.replace(c.get(0).unwrap().as_str(), "");
        res.1.insert(
            c.get(1).unwrap().as_str().to_owned(), 
            eval_str(c.get(2).unwrap().as_str())?
        );
    }
    Ok(res)
}

/// Wraps most functions in `nexsys::parsing`, returning either an error that 
/// prevents the code from being solvable or the intermediate language representation
/// of the `.nxs`-formatted code.
/// 
/// This also mutates the given `ctx` and `declared` arguments, adding any found constant or 
pub fn compile(code: &str, ctx: &mut ContextHashMap, declared: &mut HashMap<String, [f64; 3]>) -> anyhow::Result<String> 
{    
    let sys_domains: HashMap<String, [f64; 2]>;
    let sys_guesses: HashMap<String, f64>;
    let sys_consts:  HashMap<String, f64>;
    
    let mut nil = comments(code); 

    // Copy-paste all common engineering constants (this happens first so users can rename constants)
    nil = consts(&nil)?;

    // Copy-paste any unit conversions (this happens second so they can be used in const definitions)
    nil = conversions(&nil)?;

    // Set all constants used in the solution
    (nil, sys_consts) = const_values(&nil)?;
    for (var, val) in sys_consts
    {
        ctx.add_const_to_ctx(&var, val);
    }

    // Set all domains for variables in the solution
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

    // Set all initial guesses for variables in the solution
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

    // Format all conditional statements. (this happens last since most information is needed in order to evaluate the expression)
    nil = conditionals(&nil)?;

    Ok(nil)
}