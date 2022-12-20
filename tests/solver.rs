mod tools;

use std::collections::HashMap;
use nexsys::algos::{BlockMgr, Equation, Variable};
use nexsys::solver::Nexsys;
use nexsys::solve;

#[test]
fn test_equation() {
    let my_eqn = "x = y + z + 2";
    let eqn = Equation::new(my_eqn);

    let vars = vec!["x".to_string(), "y".to_string(), "z".to_string()];
    let uks = vec!["y".to_string(), "z".to_string()];

    assert_eq!(eqn.vars(), vars);

    let ctx = HashMap::from([
        ("x".to_string(), Variable::new(1.0, None))
    ]);

    assert_eq!(eqn.unknowns(&ctx), uks);
}

#[test]
fn test_block_mgr() {
    let ctx = HashMap::from([("a".to_string(), Variable::new(0.0, None))]);
    let mut bkm = BlockMgr::new(&ctx);

    let my_eqns = vec![
        "2*x + 5*y + 2*z = -38 + a",
        "3*x - 2*y + 4*z = 17",
        "-6*x + y - 7*z = -12"
    ];

    for i in my_eqns.iter().map(
        |e| Equation::new(e)
    ) {
        bkm.add_item(&i);
    }

    println!("{:#?}", bkm);

    bkm.constrained().unwrap(); // This will panic if the test fails
}

#[should_panic]
#[test]
fn test_block_mgr_guard_clause() {
    let my_eqn = Equation::new("x^2");

    let ctx = HashMap::new();

    let mut bkm = BlockMgr::new(&ctx);

    bkm.add_item(&my_eqn);

    bkm.constrained().unwrap();
}

#[test]
fn test_solver_engine() {
    let my_sys = Nexsys::new(r#"
        a = 4
        b = a + 5
        x + y = b
        x - y = a"#, 
        
        1e-10, 300, false );

    let soln = match my_sys.solve() {
        Ok(o) => o,
        Err(e) => panic!("{}", e)  
    };


    println!("{}", soln.1.join("\n"));

    let x = "x".to_string();
    let y = "y".to_string();

    assert_thou!(soln.0[&x].as_f64(), 6.5);
    assert_thou!(soln.0[&y].as_f64(), 2.5);
}

#[test]
fn test_solver_w_conditional() {
    let my_code = r#"
    a = -4
    if [a < 0] {
        b = sqrt(-a)
    } else {
        b = sqrt(a)
    }
    "#;

    let (soln, _) = solve(my_code, None, None, false).unwrap();

    assert_thou!(soln["b"].as_f64(), 2.0);
}

#[test]
fn test_solver_w_conversions() {
    let my_code = r#"
    a = 2.54 * [cm->in]
    b = 12 * a * [in->ft]
    c = b * [ft->cm]
    "#;

    let (soln, _) = solve(my_code, Some(1E-10), None, false).unwrap();

    assert_thou!(soln["c"].as_f64(), 30.48);
}