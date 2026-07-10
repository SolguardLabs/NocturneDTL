use nocturne_dtl::codec;
use nocturne_dtl::scenario::{demo_scenario, run_scenario, run_scenario_json};
use std::env;
use std::fs;
use std::io::{self, Read};

fn read_stdin() -> io::Result<String> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Ok(input)
}

fn main() {
    let args = env::args().skip(1).collect::<Vec<_>>();
    let result = if args.iter().any(|arg| arg == "--demo") {
        run_scenario(demo_scenario()).and_then(|output| codec::to_json(&output))
    } else if let Some(path) = args.iter().find(|arg| !arg.starts_with("--")) {
        fs::read_to_string(path)
            .map_err(|err| nocturne_dtl::NocturneError::Scenario(err.to_string()))
            .and_then(|input| run_scenario_json(&input))
    } else {
        read_stdin()
            .map_err(|err| nocturne_dtl::NocturneError::Scenario(err.to_string()))
            .and_then(|input| run_scenario_json(&input))
    };

    match result {
        Ok(output) => println!("{}", output),
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    }
}
