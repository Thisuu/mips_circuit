// Built-in deps
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use zokrates_ast::ir;
// Workspace imports
use types::{BlockNumber};
use zokrates_ast::ir::{ProgEnum, Statement};
use zokrates_field::Field;

pub fn compute_witness(compiled_circuit_path: String, args: &HashMap<String, String>, block_number: BlockNumber) -> Result<(), String> {
    let path = Path::new(args.get(&"input".to_string()).unwrap());
    let file = File::open(path).map_err(|why| format!("Could not open {}: {}", path.display(), why))?;

    let mut reader = BufReader::new(file);

    match ProgEnum::deserialize(&mut reader)? {
        ProgEnum::Bls12_381Program(p) => compute(p, args),
        ProgEnum::Bn128Program(p) => compute(p, args),
        ProgEnum::Bls12_377Program(p) => compute(p, args),
        ProgEnum::Bw6_761Program(p) => compute(p, args),
        ProgEnum::PallasProgram(p) => compute(p, args),
        ProgEnum::VestaProgram(p) => compute(p, args),
    }
}

fn compute<'a, T: Field, I: Iterator<Item=ir::Statement<'a, T>>>(
    ir_prog: ir::ProgIterator<'a, T, I>,
    args: &HashMap<String, String>,
) -> Result<(), String> {
    vlog::warn!("Computing witness...");

    Ok(())
}