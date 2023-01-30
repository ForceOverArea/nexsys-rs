use nexsys::{parsing::{conditionals, conversions}, units::unit_data};

#[test]
fn test_conditional_parser() {

    let my_code = r#"
    if[a < b] {
        b - a
    } else {
        a - b
    }"#;

    let res = conditionals(my_code).unwrap().replace(['\n', ' '], "");
    assert_eq!("if(a,4.0,b,b-a,a-b)=0", res);
}

#[should_panic]
#[test]
fn test_comparison_op_parser() {

    let my_code = r#"
    if[a =< b] {
        b - a
    } else {
        a - b
    }"#;

    let res = conditionals(my_code).unwrap().replace(['\n', ' '], "");
    assert_eq!("if(a,4.0,b,b-a,a-b)=0", res);
}

#[test]
fn test_nested_conditional_formatting() {
    let my_code = r#"
    if[a < b] {
        b - a = 1
    } else {
        if [a == b] {
            b = a
        } else {
            a - b = 1
        }
    }"#;

    let res = conditionals(my_code).unwrap().replace(['\n', ' '], "");
    assert_eq!("if(a,4.0,b,b-a-(1),if(a,1.0,b,b-(a),a-b-(1)))=0", res);
}

#[test]
fn test_conversion_parser() {
    let my_sys = "[in->cm]\n[in/s->cm/s]\n[gpm->m^3/s]";
    let formatted = conversions(my_sys).unwrap();

    assert_eq!(formatted.as_str(), "2.54\n2.54\n0.0000630902")
}

#[test]
fn test_unit_data() {

    let ud = unit_data();
    println!("{}",ud["VELOCITY"]["in/s"]);
    println!("{}",ud["VELOCITY"]["cm/s"]);
    println!("{}",ud["VOLUME"]["m^3"]);
    println!("{}",ud["PRESSURE"]["N/m^2"]);
    println!("{}",ud["AREA"]["in^2"]);
    println!("{}",ud["VOLUMETRIC FLOW"]["m^3/s"]);
    println!("{}",ud["VOLUMETRIC FLOW"]["gpm"]);
    println!("{}",ud["VELOCITY"]["mph"]);
}