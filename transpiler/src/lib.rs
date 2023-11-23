use std::collections::HashMap;

use circuit::MyCircuit;
use rhai::{Engine, EvalAltResult};
use system::SimplifiedConstraitSystem;
use transpiler::transpile;
use util::get_known_value;

use crate::engine::EngineExt;
use once_cell::sync::Lazy;

pub mod circuit;
pub mod engine;
pub mod system;
pub mod transpiler;
pub mod util;

static mut CONTEXT: SimplifiedConstraitSystem = SimplifiedConstraitSystem {
    // ..Default::default()
    signals: Vec::new(),
    columns: Vec::new(),
    regions: Vec::new(),
    gates: Vec::new(),
    inputs: Lazy::new(|| HashMap::new()),
    cells: Lazy::new(|| HashMap::new()),
    instance_count: 0,
};

pub fn try_run(code: String) -> Result<String, Box<EvalAltResult>> {
    unsafe {
        CONTEXT = SimplifiedConstraitSystem {
            // ..Default::default()
            signals: Vec::new(),
            columns: Vec::new(),
            regions: Vec::new(),
            gates: Vec::new(),
            inputs: Lazy::new(|| HashMap::new()),
            cells: Lazy::new(|| HashMap::new()),
            instance_count: 0,
        };
    }

    let mut engine = Engine::new();

    engine.register_plonk_script();

    let script = transpile(code);
    let mut file = std::fs::File::create("debug.rhai").unwrap();
    std::io::Write::write_all(&mut file, script.as_bytes()).unwrap();

    // println!("{}", script);
    if let Err(error) = engine.run(script.as_str()) {
        println!("Script Error: {:#?}", error);
        return Err(error);
    }

    let d = unsafe { format!("{:#?}", CONTEXT) };
    let mut file = std::fs::File::create("context.rust").unwrap();
    std::io::Write::write_all(&mut file, d.as_bytes()).unwrap();

    let k = unsafe { CONTEXT.inputs.get("k").or(Some(&"8".to_string())) }
        .unwrap()
        .parse::<u32>()
        .unwrap();

    let public_input = unsafe { CONTEXT.signals.clone() }
        .into_iter()
        .map(|x| match x.value {
            Some(x) => get_known_value(x.clone()).expect(format!("Decoding failed: {x}").as_str()),
            None => panic!("No value for signal [{}]", x.name),
        })
        .collect();

    let ret = run_prover(k, public_input);

    Ok(ret)
}

fn run_prover(k: u32, public_input: Vec<halo2_proofs::pasta::Fp>) -> String {
    let circuit = MyCircuit {
        _marker: std::marker::PhantomData,
    };

    let presult = halo2_proofs::dev::MockProver::run(k, &circuit, vec![public_input.clone()]);

    if let Ok(prover) = presult {
        let d = format!("{:#?}", prover);
        let mut file = std::fs::File::create("visualization.rust").unwrap();
        std::io::Write::write_all(&mut file, d.as_bytes()).unwrap();

        prover.assert_satisfied();
        format!("{:#?}", prover)
    } else {
        format!("{:?}", presult.err())
    }
}
