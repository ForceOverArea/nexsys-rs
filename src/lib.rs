/// Provides data sets of common units and functions for converting between them.
pub mod units;
/// Provides tools for parsing text prior to passing to the equation solving engine.
pub mod parsing;
/// Different errors specific to Nexsys implementations of algorithms.
pub mod errors;

use std::collections::HashMap;

use geqslib::shunting::new_context;
pub use geqslib::*;
pub use gmatlib::*;
use parsing::compile;

use crate::shunting::{ContextHashMap, ContextLike};
use crate::system::{ConstrainResult, get_equation_unknowns, SystemBuilder};

/// Solves a single equation for a single unknown value, returning a `bool` indicating if the solution attempt was successful 
fn try_solve_single_unknown_eqn(eqn_pool: &mut Vec<String>, ctx: &mut ContextHashMap, declared: &mut HashMap<String, [f64; 3]>, log_step: &mut String, margin: f64, limit: usize) -> anyhow::Result<bool>
{
    for (i, equation) in eqn_pool.iter().enumerate()
    {
        let unknowns: Vec<&str> = get_equation_unknowns(&equation, ctx).collect();
        if unknowns.len() != 1
        {
            return Ok(false);
        }

        let var_info: [f64; 3];
        if declared.contains_key(unknowns[0])
        {
            var_info = declared[unknowns[0]];
        }
        else
        {
            var_info = [1.0, f64::NEG_INFINITY, f64::INFINITY];
        }

        let soln = solve_equation_with_context(equation, ctx, var_info[0], var_info[1], var_info[2], margin, limit)?;
        ctx.add_var_with_domain_to_ctx(&soln.0, soln.1, var_info[1], var_info[2]);
        *log_step = format!(
            "Var: {:#?} \nEquation: {}", 
            soln.0, equation
        );
        eqn_pool.remove(i);
        return Ok(true);
    }
    Ok(false)
}

fn try_solve_subsystem_of_equations(eqn_pool: &mut Vec<String>, ctx: &mut ContextHashMap, declared: &mut HashMap<String, [f64; 3]>, log_step: &mut String, margin: f64, limit: usize) -> anyhow::Result<bool>
{
    for equation in &*eqn_pool
    {
        let mut builder = SystemBuilder::new(equation, ctx.clone())?;
        let mut sub_pool = eqn_pool.iter()
            .filter(|&x| x != equation)
            .map(|x| x.to_owned())
            .collect::<Vec<String>>();
        let mut still_learning = true;

        while still_learning
        {
            // Build up a constrained system:
            still_learning = false;
            for (i, equation) in sub_pool.iter().enumerate()
            {
                match builder.try_constrain_with(equation)?
                {
                    ConstrainResult::WillConstrain => {
                        sub_pool.remove(i);
                        still_learning = true;
                        break;
                    },
                    ConstrainResult::WillOverConstrain => break,
                    _ => {}
                }
            }
        }
            
        *log_step = format!("{:#?}", builder);

        // Solve the found constrained system:
        if let Some(mut system) = builder.build_system()
        {
            for (var, var_info) in declared
            {
                system.specify_variable(var, var_info[0], var_info[1], var_info[2]);
            }

            let soln = system.solve(margin, limit)?;
            for (var, val) in soln 
            {
                ctx.add_var_to_ctx(&var, val);
            }
            eqn_pool.clear();
            eqn_pool.extend(sub_pool.into_iter());

            return Ok(true);
        }
    }
    Ok(false)
}

/// Solves a system of equations in plain-text format.
/// For more supported syntax, see `solve_with_preprocessors`
/// 
/// # Example
/// ```
/// ```
pub fn basic_solve(system: &str, ctx: &mut ContextHashMap, declared: &mut HashMap<String, [f64; 3]>, margin: f64, limit: usize) -> anyhow::Result<(Vec<String>, ContextHashMap)>
{
    let mut log = vec![];
    let mut eqn_pool = system.split('\n')
        .filter(|x| x.contains('='))
        .map(|x| x.to_owned())
        .collect();

    loop
    {
        let mut log_step = String::default();
        // Get less expensive solutions:
        if try_solve_single_unknown_eqn(&mut eqn_pool, ctx, declared, &mut log_step, margin, limit)?
        {
            log.push(log_step);
            continue;
        }

        // Dig in and solve a more expensive subsystem:
        else if try_solve_subsystem_of_equations(&mut eqn_pool, ctx, declared, &mut log_step, margin, limit)?
        {
            log.push(log_step);
            continue;
        }
        
        break;
    }
    Ok((log, ctx.clone()))
}

/// Solves a system of equations with additional syntax used to indicate unit conversions, 
pub fn solve_with_preprocessors(system: &str, margin: f64, limit: usize) -> anyhow::Result<(Vec<String>, ContextHashMap)>
{
    let mut ctx = new_context(); 
    let mut declared = HashMap::new();
    let compiled = compile(system, &mut ctx, &mut declared)?;

    basic_solve(&compiled, &mut ctx, &mut declared, margin, limit)
}