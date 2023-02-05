use nexsys::{parsing::{conditionals, conversions}, units::unit_data};

#[test]
fn test_conditional_parser() {

    let my_code = 
r#"
If you see this in the output you're in it deep
    if a < b:
    b - a
else:
    a - b
end"#;

    let res = conditionals(my_code).unwrap();
    println!("{res}");
    assert!(res.contains("if(a,4.0,b,b-a,a-b) = 0"));
}

#[test]
fn test_comparison_op_parser() {

    let my_code = 
r#"
If you see this in the output you're in it deep
    if a =< b:
    b - a = 0
else:
    a - b = 0
end"#;

    match conditionals(my_code){
        Err(e) => assert!(e.to_string() == "invalid comparison operator. valid operators are: <, >, <=, >=, ==, !="),
        _ => panic!()
    }
}

#[test]
fn test_nested_conditional_formatting() {
    let my_code = 
r#"
If you see this in the output you're in it deep

if a < b:
    b - a = 1
else:
    if a == b:
        b = a
    else:
        a - b = 1
    end
end
"#;

    let res = conditionals(my_code).unwrap();
    println!("{res}");
    assert!(res.contains("if(a,4.0,b,b-a-(1),if(a,1.0,b,b-(a),a-b-(1))) = 0"));
}

#[test]
fn test_conversion_parser() {
    let my_sys = "[in->cm]\n[in/s->cm/s]\n[gpm->m^3/s]";
    let formatted = conversions(my_sys).unwrap();

    assert_eq!(formatted.as_str(), "2.54\n2.54\n0.0000630902")
}

#[test]
fn test_unit_data() {

    let _ud = unit_data();
    // println!("{}",_ud["VELOCITY"]["in/s"]);
    // println!("{}",_ud["VELOCITY"]["cm/s"]);
    // println!("{}",_ud["VOLUME"]["m^3"]);
    // println!("{}",_ud["PRESSURE"]["N/m^2"]);
    // println!("{}",_ud["AREA"]["in^2"]);
    // println!("{}",_ud["VOLUMETRIC FLOW"]["m^3/s"]);
    // println!("{}",_ud["VOLUMETRIC FLOW"]["gpm"]);
    // println!("{}",_ud["VELOCITY"]["mph"]);
}