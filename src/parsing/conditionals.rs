use std::error::Error;

use crate::errors::ConditionFormatError;
use lazy_static::lazy_static;
use regex::Regex;

/// Evaluates if the first expression contains any of the later expressions
macro_rules! contains_any {
    ($s:expr, $ch1:expr, $( $ch:tt ),* ) => {{
        $s.contains($ch1) $( || $s.contains($ch) )*
    }};
}

/// The actual code that should be executed for evaluating an if statement via `meval`.
/// This function panics if it doesn't receive exactly 5 arguments.
pub (in crate) fn conditional(st: &[f64]) -> f64 {
    // Panic if the formatting is incorrect. This should not happen if the 
    // formatting code does its job properly.
    assert!(st.len() == 5); 

    let (a, op, b, res1, res2) = (st[0], st[1], st[2], st[3], st[4]);

    let decision = move |p: bool| -> f64 {
        if p {
            res1
        } else {
            res2
        }
    };
    
    if op == 1.0 {          // eq
        decision(a == b)

    } else if op == 2.0 {   // le
        decision(a <= b)

    } else if op == 3.0 {   // ge
        decision(a >= b)

    } else if op == 4.0 {   // lt
        decision(a < b)

    } else if op == 5.0 {   // gt
        decision(a > b)

    } else {                // ne
        decision(a != b)
    }
}

/// Formats a "curly braces" `if` statement to a `conditional(...)` function call that will work in meval.
/// This function returns an `Err` if an invalid conditional operator is found in `cndl`.
pub (in crate) fn format_conditional(cndl: &str) -> Result<String, Box<dyn Error>> {

    let mut args = cndl.replace([' ', '\n'], " ") // strip whitespace
    .replace("if[", "if(")  // make start of function call
    .replace("]{", ",")     // delimit arguments
    .replace("}else{", ",") // (ditto)
    .replace('}', ")");     // close function call

    //if(a<b,a-b=1,b-a=1)
    // println!("SUBBED TOKENS: {}", args);

    if !(contains_any!(args, "==", "<=", ">=", "<", ">", "!=")) {
        return Err(Box::new(ConditionFormatError::ConditionalSyntax))
    }

    // replace conditional sign with f64 code number
    if args.contains("==") {args = args.replace("==", ",1.0,");} 
    if args.contains("<=") {args = args.replace("<=", ",2.0,");} 
    if args.contains(">=") {args = args.replace(">=", ",3.0,");} 
    if args.contains('<') {

        if args.contains("=<") {
            return Err(Box::new(ConditionFormatError::Comparator))
        }

        args = args.replace('<',  ",4.0,");
        
    } 
    if args.contains('>') {
        
        if args.contains("=>") {
            return Err(Box::new(ConditionFormatError::Comparator))
        }

        args = args.replace('>',  ",5.0,");
                
    } 
    if args.contains("!=") {args = args.replace("!=", ",6.0,");}

    // println!("FINAL: {}", args);
    // Conditional statement reformatted as function call
    // This allows us to use `better` notation to call a function via meval
    Ok(args + " = 0")
}

/// Identifies and returns conditional statements found in a Nexsys-legal string.
pub fn conditionals(text: &str) -> Result<String, Box<dyn Error>> {
    lazy_static!{
        static ref RE: Regex = Regex::new(r"(?si)if ?\[ ?.* ?[<>=]+ ?.* ?\] ?\{.*\} ?else ?\{.*\}").unwrap();
        static ref CHK: Regex = Regex::new(r"").unwrap();
    }
    let mut output = text.to_string();
    let cdls: Vec<&str> = RE.find_iter(text).map(|i| i.as_str()).collect();

    for raw in cdls {

        // format equations contained in the conditional via .map(), then pass to format_conditional()
        let fmt_eqns = raw.split('\n').map(|line| {
            
            if line.contains('=') && !(contains_any!(line, "[", "]", "<", ">")) {
                
                let terms = line.split('=').collect::<Vec<&str>>();
                
                format!("{} - ({})", terms[0], terms[1])
            
            } else {
                
                line.to_string()
            
            }

        }).collect::<Vec<String>>().join("\n");

        // println!("{}", fmt_eqns);

        let fmtd = &format_conditional(&fmt_eqns)?;

        // if !CHK.is_match(fmtd) { // todo: make an error return if the formatted conditional statement is erroneous
        //     return Err(Box::new(ConditionFormatError::ConditionalSyntax))
        // }

        output = output.replace(raw, fmtd);
    }

    Ok(output)
}

/// Testing for non-public macros
#[cfg(test)]
mod test {

    /// Tests the `contains_any!` macro
    #[test]
    fn test_contains_any_macro() {
        assert_eq!(
            contains_any!("test_string", "a", "b", "c"), 
            false  
        );

        assert_eq!(
            !(contains_any!("test_string", "a", "b", "c")),
            true
        );

        assert_eq!(
            contains_any!("test_string", "t", "b", "c"),
            true
        );
    } 
    
    /// Additional testing for how the `contains_any!` macro works
    #[test]
    fn buggy_case() {
        if !(contains_any!("if(a<b,b-a-(1),if(a==b,b-(a),a-b-(1)))", "==", "<=", ">=", "<", ">", "!=")) {
            panic!()
        }
    }
    
}