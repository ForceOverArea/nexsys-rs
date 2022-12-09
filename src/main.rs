use std::{env, process};
use std::fs::{read_to_string, write};
use nexsys::solve;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 || args[1] == format!("help") || args[1].contains(&format!("-h")) {
        println!(
r#"Nexsys - Solve nonlinear systems using Newton's method

USAGE:
nxc <FILEPATH> <OPTIONS>

OPTIONS:
--tolerance, --tol <float>              The tolerance that the solver should hit before returning a solution.
--max-iterations, --max <int>           The maximum number of iterations that the solver can take to converge.
--allow-nonconvergence, --ancv          Allows the solver to ignore results that do not converge.
--fast                                  Makes the solver use the newton-raphson method for single-unknown problems instead of the golden-section search.
"#
        );
        process::exit(0);
    }

    let system = match read_to_string(&args[1]) {
        Ok(o) => o,
        Err(_) => {
            println!("[nxc].....ERR: could not find the specified filepath");
            process::exit(1);
        }
    };

    let mut tolerance = None;
    let mut max_iterations = None; 
    let mut allow_nonconvergence = None;
    // let mut output_format = "md"; // todo: make this produce different file types

    for i in 0..args.len() {
        if &args[i] == &format!("--tolerance") || &args[i] == &format!("--tol") {
            match args[i+1].parse::<f64>() {
                Ok(o) => {
                    println!("[nxc].....tolerance set to {}", o);
                    tolerance = Some(o);
                },
                Err(_) => {
                    println!("[nxc].....ERR: tolerance is not a valid float value");
                    process::exit(1);
                }
            }
        }
        if &args[i] == &format!("--max-iterations") || &args[i] == &format!("--max") {
            match args[i+1].parse::<usize>() {
                Ok(o) => {
                    println!("[nxc].....iteration limit set to: {}", o);
                    max_iterations = Some(o);
                },
                Err(_) => {
                    println!("[nxc].....ERR: iteration limit is not a valid integer value");
                    process::exit(1);
                }
            }
        }
        if &args[i] == &format!("--allow-nonconvergence") || &args[i] == &format!("--ancv") {
            println!("[nxc].....nonconvergence is allowed");
            allow_nonconvergence = Some(true);
        }
    }

    let (soln, log) = match solve(&system, tolerance, max_iterations, allow_nonconvergence) {
        Ok(o) => o,
        Err(_) => {
            println!("[nxc].....ERR: nxc could not solve the system");
            process::exit(1);
        }
    };

    let output = format!(
        "# **[->] Nexsys** - *{}* results:\n\n**Solution:**\n\n{}\n___\n**Solution Procedure:**\n\n{}",
        &args[1],
        soln.into_iter().map(|i| format!("{} = {}\n\n", i.0, i.1.as_f64())).collect::<String>(),
        log.join("\n\n")
    );

    match write(&args[1].replace(".nxs", ".md"), output) {
        Ok(_) => process::exit(0),
        Err(_) => {
            println!("[nxc].....ERR: nxc could not write to the output file");
            process::exit(1);
        }
    }
}
