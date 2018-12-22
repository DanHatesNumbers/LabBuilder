mod lib;

use crate::lib::network::{Network, NetworkType};
use crate::lib::scenario::Scenario;
use crate::lib::system::System;
use ipnet::Ipv4Net;

use toml::Value;

use std::str::FromStr;

fn main() -> Result<(), std::boxed::Box<std::error::Error>> {
    Ok(())
}

fn parse_scenario<'a>(
    scenario_toml: &'a Value,
) -> Result<Scenario, std::boxed::Box<std::error::Error>> {
    let mut scenario = Scenario {
        name: "".into(),
        networks: Vec::new(),
        systems: Vec::new(),
    };

    Ok(scenario)
}

}
