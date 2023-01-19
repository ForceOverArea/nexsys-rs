use lazy_static::lazy_static;
use serde_json::{Value, from_str};
use std::{collections::HashMap, error::Error};

use crate::errors::UnitConversionError;

/// Handles generating additional units for various combinations of other units
fn generate_num_denom_units(mut data:HashMap<String, HashMap<String, f64>>, qty: &str, num: &str, denom: &str) -> HashMap<String, HashMap<String, f64>> {
    
    let mut temp = HashMap::new();
    
    for i in &data[&num.to_string()] {
    
        for j in &data[&denom.to_string()] {
    
            temp.insert(
                format!("{}/{}", i.0, j.0),
                *i.1 / *j.1
            );
        
        }
    
    }

    if let Some(d) = data.get_mut(qty) {
        d.extend(temp.into_iter());
    }

    data
}

/// Handles generating additional units for various combinations of other units
fn generate_fact_fact_units(mut data:HashMap<String, HashMap<String, f64>>, qty: &str, fc1: &str, fc2: &str) -> HashMap<String, HashMap<String, f64>> {

    let mut temp = HashMap::new();
    
    if fc1 == fc2 {
        for i in &data[&fc1.to_string()] {
            temp.insert( format!("{}^2", i.0), *i.1 * *i.1);
        }
    } else {
        for i in &data[&fc1.to_string()] {
            for j in &data[&fc2.to_string()] {
                let val = *i.1 * *j.1;
                temp.insert( format!("{}-{}", i.0, j.0), val);
                temp.insert( format!("{}-{}", j.0, i.0), val);
            }
        }
    }

    if let Some(d) = data.get_mut(qty) {
        d.extend(temp.into_iter());
    }

    data
}

/// Handles generating additional units for various combinations of other units
fn generate_volume_units(mut data:HashMap<String, HashMap<String, f64>>) -> HashMap<String, HashMap<String, f64>> {

    let mut temp = HashMap::new();
    
    for i in &data[&"LENGTH".to_string()] {
        temp.insert( format!("{}^3", i.0), *i.1 * *i.1);    
    }

    if let Some(d) = data.get_mut("VOLUME") {
        d.extend(temp.into_iter());
    }
    
    data
}

/// Returns the data contained in units.json as a `HashMap`, allowing for easier access to this data in Rust.
pub fn raw_unit_data() -> HashMap<String, HashMap<String, f64>> {

    let raw_text = include_str!("units.json");
    let err = "failed to parse json... is the formatting of 'units.json' correct?";
    
    // again I ask, lord forgive me for what I am about to do...
    let dejson: HashMap<String, HashMap<String, f64>> = from_str::<HashMap<&str, Value>>(raw_text).expect(err).into_iter()
    .map(
        |i| (
            i.0.to_string(), 
            i.1.as_object().expect(err).into_iter()
            .map(
                |j| (
                    j.0.to_string(), 
                    j.1.as_f64().expect(err)
                )
            ).collect()
        )
    ).collect();

    dejson
}

/// Generates a more complete set of unit conversion data by combining different units to create other common units.
pub fn unit_data() -> HashMap<String, HashMap<String, f64>> {
    let mut data = raw_unit_data();
    data.insert("SPRING FORCE".to_string(), HashMap::new());

    data = generate_fact_fact_units(data, "AREA",               "LENGTH",           "LENGTH");
    data = generate_fact_fact_units(data, "VISCOSITY-DYNAMIC",  "PRESSURE",         "TIME");
    data = generate_fact_fact_units(data, "ENERGY",             "FORCE",            "LENGTH"); // this is also torque units
    data =    generate_volume_units(data);


    data = generate_num_denom_units(data, "VELOCITY",           "LENGTH",           "TIME");
    data = generate_num_denom_units(data, "FREQUENCY",          "NON DIMENSIONAL",  "TIME");
    data = generate_num_denom_units(data, "VOLUMETRIC FLOW",    "VOLUME",           "TIME");
    data = generate_num_denom_units(data, "POWER",              "ENERGY",           "TIME");
    data = generate_num_denom_units(data, "PRESSURE",           "FORCE",            "AREA");
    data = generate_num_denom_units(data, "SPRING FORCE",       "FORCE",            "LENGTH");

    data
}

/// Returns a conversion factor between any unit in `unit_data()` for a given `fro` and `to` unit
pub fn convert(fro: &str, to: &str) -> Result<f64, Box<dyn Error>> {
    lazy_static! { // Make it such that we don't need to generate this list more than once on runtime
        static ref UD: HashMap<String, HashMap<String, f64>> = unit_data();
    }

    let cf: Vec<f64> = UD.iter()
    .filter(|&i| { 
        let qty = UD.get(i.0).unwrap().clone();
        qty.contains_key(fro) && qty.contains_key(to)
    }).map(|i| {
        i.1[fro] / i.1[to]
    }).collect();

    if cf.len() != 1 {
        return Err(Box::new(UnitConversionError))
    }

    Ok(cf[0])
}

/// Returns the data contained in consts.json as a `HashMap`, allowing for easier access to this data in Rust.
pub fn const_data() -> HashMap<String, f64> {

    let raw_text = include_str!("consts.json");
    let err = "failed to parse json... is the formatting of consts.json correct?";
    
    // thankfully this isn't as bad as reading units.json
    let dejson: HashMap<String, f64> = from_str::<HashMap<&str, Value>>(raw_text).expect(err).into_iter()
    .map(
        |i| {
            let err = format!("failed to parse json: {:#?}", i.1);
            let c1 = i.1.as_array().expect(&err);

            let err = format!("failed to parse json: {:#?}", c1[1]);
            let c2 = match c1[1].as_f64() {
                Some(c) => c,
                None    => c1[1].as_str().expect(&err).parse::<f64>().expect(&err)
            };

            (i.0.to_string(), c2)

    }).collect();

    dejson
}