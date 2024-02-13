use crate::{errors::ConditionFormatError, parsing::nexsys_regex};
use lazy_static::lazy_static;
use regex::Regex;

/// Evaluates if the first expression contains any of the later expressions
macro_rules! contains_any {
    ($s:expr, $ch1:expr, $( $ch:tt ),* ) => {{
        $s.contains($ch1) $( || $s.contains($ch) )*
    }};
}

/// Formats a "curly braces" `if` statement to a `conditional(...)` function call that will work in meval.
/// This function returns an `Err` if an invalid conditional operator is found in `cndl`.
pub (in crate) fn format_conditional(cndl: &str) -> anyhow::Result<String> {

    let mut args = cndl.replace("if ",  "if(")  // make start of function call
    .replace([' ', '\n'], "")   // strip whitespace
    .replace(':',   ",")        // delimit arguments
    .replace("else", "")        // (ditto)
    .replace("end", ")");       // close function call

    //if(a<b,a-b=1,b-a=1)
    // println!("SUBBED TOKENS: {}", args);

    if !(contains_any!(args, "==", "<=", ">=", "<", ">", "!=")) {
        return Err(ConditionFormatError::ConditionalSyntax.into())
    }

    // replace conditional sign with f64 code number
    if args.contains("==") {args = args.replace("==", ",1.0,");} 
    if args.contains("<=") {args = args.replace("<=", ",2.0,");} 
    if args.contains(">=") {args = args.replace(">=", ",3.0,");} 
    if args.contains('<') {

        if args.contains("=<") {
            return Err(ConditionFormatError::Comparator.into())
        }

        args = args.replace('<',  ",4.0,");
        
    } 
    if args.contains('>') {
        
        if args.contains("=>") {
            return Err(ConditionFormatError::Comparator.into())
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
pub fn conditionals(text: &str) -> anyhow::Result<String> {
    lazy_static!{
        static ref RE: Regex = nexsys_regex(            
r#"(?m)^[ \t]*if [^<>=]+[<>=]{1,2}[^<>=]+:$
^.*$
^[ \t]*else:$
^.*$
^[ \t]*end"#
        );
    }
    let mut output = text.to_string();
    
    loop {
        let tmp = output.to_string(); //FIXME: this looks stupid. Is there a better way to do it?
        let cdls: Vec<&str> = RE.find_iter(&tmp).map(|i| i.as_str()).collect();

        // println!("{cdls:#?}");
    
        for raw in &cdls {
    
            let mut rows = raw
                .split('\n')
                .map(|i| i.to_string())
                .collect::<Vec<String>>();
    
            // println!("{rows:#?}");
    
            for r in [1,3] {
                if rows[r].contains('=') {
                    let terms = rows[r].split('=').collect::<Vec<&str>>();
                    if terms[1].replace(' ',"") == 0.to_string() {
                        rows[r] = terms[0].to_string();
                    } else {
                        rows[r] = format!("{} - ({})", terms[0], terms[1]);
                    }
                }
            }
            
            let fmt_eqns = rows.join("\n");
    
            // println!("{}", fmt_eqns);
    
            let fmtd = &format_conditional(&fmt_eqns)?;
    
            output = output.replace(raw, fmtd);
        }

        if cdls.is_empty() { break } // keep going until there are no if statement matches left
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