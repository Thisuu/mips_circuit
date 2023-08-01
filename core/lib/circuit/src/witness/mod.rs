// Built-in deps
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, stdin};
use std::path::Path;
use zokrates_ast::ir;
// Workspace imports
use types::{BlockNumber};
use zokrates_ast::ir::{ProgEnum, Statement};
use zokrates_ast::typed::{ConcreteSignature, ConcreteType};
use zokrates_ast::typed::abi::Abi;
use zokrates_ast::typed::types::GTupleType;
use zokrates_field::Field;
// External imports
use serde_json::from_reader;

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

    let verbose = matches!(args.get(&"verbose".to_string()).unwrap().as_str(),"ture");
    let is_stdin = matches!(args.get(&"stdin".to_string()).unwrap().as_str(),"ture");
    let is_abi = matches!(args.get(&"abi".to_string()).unwrap().as_str(),"ture");

    if !is_stdin && is_abi {
        return Err("ABI input as inline argument is not supported. Please use `stdin`.".into());
    }

    let signature = match is_abi {
        true => {
            let path = Path::new(args.get(&"abi-spec".to_string()).unwrap());
            let file = File::open(path)
                .map_err(|why| format!("Could not open {}: {}", path.display(), why))?;
            let mut reader = BufReader::new(file);

            let abi: Abi = from_reader(&mut reader).map_err(|why| why.to_string())?;

            abi.signature()
        }
        false => ConcreteSignature::new()
            .inputs(vec![ConcreteType::FieldElement; ir_prog.arguments.len()])
            .output(ConcreteType::Tuple(GTupleType::new(
                vec![ConcreteType::FieldElement; ir_prog.return_count],
            ))),
    };

    use zokrates_abi::Inputs;

    // get arguments
    let arguments = match is_stdin {
        // take inline arguments
        false => {
            let arguments = args.get(&"arguments".to_string()).unwrap();
            arguments
                .split(' ')
                .map(|x| T::try_from_dec_str(x).map_err(|_| x.to_string()))
                .collect::<Result<Vec<_>, _>>()
                .map(Inputs::Raw)
        }
        // take stdin arguments
        true => {
            let mut stdin = stdin();
            let mut input = String::new();

            match is_abi {
                true => {
                    use zokrates_abi::parse_strict;

                    input = args.get(&"arguments".to_string()).clone().unwrap().to_string();
                    parse_strict(&input, signature.inputs)
                        .map(Inputs::Abi)
                        .map_err(|why| why.to_string())
                },
                false =>  match ir_prog.arguments.len() {
                    0 => Ok(Inputs::Raw(vec![])),
                    _ => match stdin.read_to_string(&mut input) {
                        Ok(_) => {
                            input.retain(|x| x != '\n');
                            input
                                .split(' ')
                                .map(|x| T::try_from_dec_str(x).map_err(|_| x.to_string()))
                                .collect::<Result<Vec<_>, _>>()
                                .map(Inputs::Raw)
                        }
                        Err(_) => Err(String::from("???")),
                    },
                }
            }
        }
    }
    .map_err(|e| format!("Could not parse argument: {}", e))?;

    // let interpreter = zokrates_interpreter::Interpreter::default();
    // let public_inputs = ir_prog.public_inputs();
    //
    // let witness = interpreter
    //     .execute_with_log_stream(
    //         &arguments.encode(),
    //         ir_prog.statements,
    //         &ir_prog.arguments,
    //         &ir_prog.solvers,
    //         &mut std::io::stdout(),
    //     )
    //     .map_err(|e| format!("Execution failed: {}", e))?;
    //
    // use zokrates_abi::Decode;
    //
    // let results_json_value: serde_json::Value =
    //     zokrates_abi::Value::decode(witness.return_values(), *signature.output).into_serde_json();
    //
    // if verbose {
    //     println!("\nWitness: \n{}\n", results_json_value);
    // }



    Ok(())
}