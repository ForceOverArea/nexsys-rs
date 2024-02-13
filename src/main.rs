use std::{env, process};
use std::fs::{read_to_string, write};
use geqslib::shunting::Token;
use nexsys::{solve_with_preprocessors, parsing::{conditionals, conversions, consts}};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 || args[1] == *"help" || args[1] == *"-h" {
        println!(
r#"Nexsys - Solve nonlinear systems using Newton's method

USAGE:
nxc <FILEPATH> <OPTIONS>

OPTIONS:
--margin, -tol <float>              The margin that the solver should hit before returning a solution
--max-iterations, -max <int>           The maximum number of iterations that the solver can take to converge
--output-file, -o                      Sends the results to a .txt file rather than printing them in the terminal
--verbose -v                           Prints compiled nexsys code in the terminal for debugging
"#);
        process::exit(0);
    }

    let system = match read_to_string(&args[1]) {
        Ok(o) => o,
        Err(_) => {
            println!("[nxc].....ERR: could not find the specified filepath");
            process::exit(1);
        }
    };

    let mut margin = 0.0;
    let mut limit = 1; 
    let mut output_file = false; // todo: make this produce different file types

    for i in 0..args.len() {
        if args[i] == *"--margin" || args[i] == *"-tol" {
            match args[i+1].parse::<f64>() {
                Ok(o) => {
                    println!("[nxc].....margin set to {o}");
                    margin = o;
                },
                Err(_) => {
                    println!("[nxc].....ERR: margin is not a valid float value");
                    process::exit(1);
                }
            }
        }
        if args[i] == *"--max-iterations" || args[i] == *"-max" {
            match args[i+1].parse::<usize>() {
                Ok(o) => {
                    println!("[nxc].....iteration limit set to: {o}");
                    limit = o;
                },
                Err(_) => {
                    println!("[nxc].....ERR: iteration limit is not a valid integer value");
                    process::exit(1);
                }
            }
        }
        if args[i] == *"--verbose" || args[i] == *"-v" {
            println!("[nxc].....Printing compiled code...");

            let mut preprocess = conversions(&system).unwrap();
            preprocess = consts(&preprocess).unwrap();
            preprocess = conditionals(&preprocess).unwrap();
            
            println!("\n{preprocess}\n");
        }
        if args[i] == *"--to-file" || args[i] == *"-o" {
            println!("[nxc].....Writing to file...");
            output_file = true;
        }
    }

    let (log, soln) = match solve_with_preprocessors(&system, margin, limit) {
        Ok(o) => o,
        Err(e) => {
            println!("[nxc].....ERR: nxc could not solve the system");
            println!("[nxc].....{e}");
            process::exit(1);
        }
    };

    let output = format!(
        "[->] Nexsys - {} results:\n\nSolution:\n+=======+\n{}\nProcedure:\n+========+\n{}\n",
        &args[1],
        soln.into_iter().map(|i| {
            if let Token::Var(v) = i.1
            {
                format!("{} = {}\n", i.0, f64::from(*v.borrow()))
            }
            else 
            {
                if let Token::Num(n) = i.1
                {
                    if i.0 != "pi" && i.0 != "e"
                    {
                        return format!("{} = {}\n", i.0, n);
                    }
                }
                String::default()
            }
        }).collect::<String>(),
        log.join("\n")
    );

    if output_file {
        match write(args[1].replace(".nxs", ".txt"), output) {
            Ok(_) => process::exit(0),
            Err(_) => {
                println!("[nxc].....ERR: nxc could not write to the output file");
                process::exit(1);
            }
        }
    } else {
        println!("{output}");
        process::exit(0);
    }
}
