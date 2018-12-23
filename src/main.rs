mod lib;

use toml::Value;

use clap::{App, AppSettings, Arg, SubCommand};

use std::fs;
use std::fs::File;
use std::path::Path;

use crate::lib::scenario::parse_scenario;

fn main() -> Result<(), std::boxed::Box<std::error::Error>> {
    let arg_matches = App::new("Lab Builder")
        .settings(&vec![AppSettings::SubcommandRequired])
        .version("0.1")
        .author("Daniel Murphy <danhatesnumbers@gmail.com>")
        .subcommand(
            SubCommand::with_name("plan")
                .about("plan Vagrantfile from Scenario")
                .arg(
                    Arg::with_name("scenario")
                        .short("s")
                        .required(true)
                        .takes_value(true)
                        .value_name("SCENARIO_PATH")
                        .help("path to Scenario to plan in TOML format"),
                ),
        )
        .get_matches();

    if let Some(plan) = arg_matches.subcommand_matches("plan") {
        let scenario_path = Path::new(plan.value_of("scenario").unwrap());

        let scenario_toml = fs::read_to_string(scenario_path)?.parse::<Value>()?;
        println!("{:?}", parse_scenario(&scenario_toml));
    };

    Ok(())
}
