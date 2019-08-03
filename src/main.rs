#![allow(dead_code)]
#![allow(unused_variables)]
#[allow(unused_imports)]

mod once;
mod combinatorics;
mod fp_vector;
mod matrix;
mod algebra;
mod adem_algebra;
mod milnor_algebra;
mod module;
mod module_homomorphism;
mod finite_dimensional_module;
mod free_module;
mod free_module_homomorphism;
mod chain_complex;
mod resolution;

#[cfg(test)]
extern crate rand;
extern crate spin;

#[macro_use]
extern crate lazy_static;

use serde_json::value::Value;

use crate::algebra::Algebra;
use crate::adem_algebra::AdemAlgebra;
use crate::milnor_algebra::MilnorAlgebra;
use crate::finite_dimensional_module::FiniteDimensionalModule;
use crate::chain_complex::ChainComplexConcentratedInDegreeZero;
use crate::resolution::Resolution;

use std::error::Error;

#[allow(unreachable_code)]
#[allow(non_snake_case)]
#[allow(unused_mut)]
fn main() {
    let args : Vec<_> = std::env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        std::process::exit(1);
    });

    match run(config) {
        Ok(string) => println!("{}", string),
        Err(e) => { eprintln!("Application error: {}", e); std::process::exit(1); }
    }
}

#[allow(non_snake_case)]
fn run(config : Config) -> Result<String, Box<Error>> {
    let contents = std::fs::read_to_string(format!("static/modules/{}.json", config.module_name))?;
    let mut json : Value = serde_json::from_str(&contents)?;
    let p = json["p"].as_u64().unwrap() as u32;
    let max_degree = config.max_degree;

    // You need a box in order to allow for different possible types implementing the same
    // trait
    let A : Box<Algebra>;
    match config.algebra_name.as_ref() {
        "adem" => A = Box::new(AdemAlgebra::new(p, p != 2, false, max_degree)),
        "milnor" => A = Box::new(MilnorAlgebra::new(p)),
        _ => { println!("Invalid algebra"); return Err(Box::new(InvalidAlgebraError { name : config.algebra_name })); }
    };

    A.compute_basis(max_degree);
    let M = FiniteDimensionalModule::from_json(&*A, &config.algebra_name, &mut json);
    let CC = ChainComplexConcentratedInDegreeZero::new(&M);
    let res = Resolution::new(&CC, max_degree, None, None);
    res.resolve_through_degree(max_degree);
    Ok(res.graded_dimension_string())
}

struct Config {
    module_name : String,
    algebra_name : String,
    max_degree : i32
}

impl Config {
    fn new(args: &[String]) -> Result<Self, String> {
        if args.len() < 4 {
            return Err("Not enough arguments".to_string());
        }
        let module_name = args[1].clone();
        let algebra_name = args[2].clone();
        let max_deg_result : Result<i32,_> = args[3].parse();

        if let Err(error) = max_deg_result {
            return Err(format!("{} in argument max_degree.", error));
        }
        let max_degree = max_deg_result.unwrap();
        Ok(Self { module_name, algebra_name, max_degree })
    }
}

#[derive(Debug)]
struct InvalidAlgebraError {
    name : String
}

impl std::fmt::Display for InvalidAlgebraError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid algebra: {}", &self.name)
    }
}

impl Error for InvalidAlgebraError {
    fn description(&self) -> &str {
        "Invalid algebra supplied"
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    #[test]
    fn milnor_vs_adem() {
        compare("S_2", 30);
        compare("C2", 30);
        compare("Joker", 30);
        compare("RP4", 30);
        compare("Csigma", 30);
        compare("S_3", 30);
        compare("Calpha", 30);
        compare("C3", 60);
    }

    fn compare(module_name : &str, max_degree : i32) {
        let a = Config {
            module_name : String::from(module_name),
            max_degree,
            algebra_name : String::from("adem")
        };
        let m = Config {
            module_name : String::from(module_name),
            max_degree,
            algebra_name : String::from("milnor")
        };

        match (run(a), run(m)) {
            (Err(e), _)    => panic!("Failed to read file: {}", e),
            (_, Err(e))    => panic!("Failed to read file: {}", e),
            (Ok(x), Ok(y)) => assert_eq!(x, y)
        }
    }
}
