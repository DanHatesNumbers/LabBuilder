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

    scenario.name = scenario_toml
        .get("scenario")
        .ok_or("Could not get Scenario from configuration")?
        .get("name")
        .ok_or("Could not read name of scenario from configuration")?
        .as_str()
        .ok_or("Could not read name of scenario as a string")?
        .into();

    Ok(scenario)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_toml_without_scenario_block_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"[NotScenario]"#.parse::<Value>()?;
        assert_eq!(
            *parse_scenario(&input).unwrap_err().description(),
            "Could not get Scenario from configuration".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_toml_without_scenario_name_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"[scenario]
            notname = "blah"
            "#
        .parse::<Value>()?;
        assert_eq!(
            *parse_scenario(&input).unwrap_err().description(),
            "Could not read name of scenario from configuration".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_toml_with_scenario_name_that_is_not_a_string_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"[scenario]
            name = 42
            "#
        .parse::<Value>()?;
        assert_eq!(
            *parse_scenario(&input).unwrap_err().description(),
            "Could not read name of scenario as a string".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_single_system_and_network_scenario_works(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [scenario]
            name = "Test Scenario"
            
            [[systems]]
            name = "Desktop"
            networks = ["LAN"]
            base_box = "Windows 10"

            [[systems]]
            name = "Server"
            networks = ["LAN"]
            base_box = "Debian"

            [[networks]]
            name = "LAN"
            network_type = "internal"
            subnet = "192.168.0.1/24"
                
        "#
        .parse::<Value>()?;

        let scenario = parse_scenario(&input)?;

        assert_eq!(scenario.name, "Test Scenario");
        assert_eq!(scenario.networks.len(), 1);
        assert_eq!(scenario.systems.len(), 2);

        assert_eq!(scenario.networks[0].name, "LAN");
        assert_eq!(scenario.networks[0].network_type, NetworkType::Internal);
        assert_eq!(
            scenario.networks[0].subnet,
            Ipv4Net::from_str("192.168.0.1/24").unwrap()
        );

        assert_eq!(scenario.systems[0].name, "Desktop");
        assert_eq!(scenario.systems[0].networks[0], scenario.networks[0]);
        assert_eq!(scenario.systems[0].base_box, "Windows 10");

        assert_eq!(scenario.systems[1].name, "Server");
        assert_eq!(scenario.systems[1].networks[1], scenario.networks[0]);
        assert_eq!(scenario.systems[1].base_box, "Debian");

        Ok(())
    }

}
