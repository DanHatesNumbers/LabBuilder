use crate::lib::network::{Network, NetworkType};
use crate::lib::system::System;

use ipnet::Ipv4Net;
use toml::Value;

use std::collections::HashMap;
use std::rc::Rc;
#[allow(unused_imports)]
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Scenario {
    pub name: String,
    pub systems: Vec<System>,
    pub networks: Vec<Rc<Network>>,
}

impl Scenario {
    pub fn from_toml(
        scenario_toml: &Value,
    ) -> Result<Scenario, std::boxed::Box<std::error::Error>> {
        let mut scenario = Scenario {
            name: "".into(),
            networks: Vec::new(),
            systems: Vec::new(),
        };

        scenario.name = scenario_toml
            .get("scenario")
            .ok_or("Could not get scenario from configuration")?
            .get("name")
            .ok_or("Could not read name of scenario from configuration")?
            .as_str()
            .ok_or("Could not read name of scenario as a string")?
            .into();

        let networks: Result<Vec<Rc<Network>>, std::boxed::Box<std::error::Error>> = scenario_toml
            .get("networks")
            .ok_or("Could not read networks from configuration")?
            .as_array()
            .ok_or("Could not read networks from configuration")?
            .into_iter()
            .map(Network::from_toml)
            .collect();

        scenario.networks.append(&mut networks?);

        scenario.are_network_names_unique()?;

        let systems: Result<Vec<System>, std::boxed::Box<std::error::Error>> = scenario_toml
            .get("systems")
            .ok_or("Could not get systems from configuration")?
            .as_array()
            .ok_or("Could not get systems from configuration")?
            .into_iter()
            .map(|system_toml| System::from_toml(&system_toml, &scenario.networks))
            .collect();

        scenario.systems.append(&mut systems?);

        Ok(scenario)
    }

    fn are_network_names_unique(&self) -> Result<(), String> {
        let names = self.networks.iter().fold(HashMap::new(), |mut acc, x| {
            acc.entry((x.name).to_string())
                .and_modify(|e| *e += 1)
                .or_insert(1);

            acc
        });

        let dup_names: Vec<&String> = names
            .iter()
            .filter_map(|n| match n.1 {
                1 => None,
                _ => Some(n.0),
            })
            .collect();

        let first_dupe_name = dup_names.iter().nth(0);

        match first_dupe_name {
            Some(name) => Err(format!(
                r#"Multiple networks parsed with name "{}". Network names must be unique."#,
                name
            )),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_toml_without_scenario_block_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [Notscenario]
            [[systems]]
            name = "Test System"
            networks = ["TestNet"]
            base_box = "Debian"
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::from_toml(&input).unwrap_err().description(),
            "Could not get scenario from configuration".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_toml_without_scenario_name_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [scenario]
            [[systems]]
            name = "Test System"
            networks = ["TestNet"]
            base_box = "Debian"
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::from_toml(&input).unwrap_err().description(),
            "Could not read name of scenario from configuration".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_toml_with_scenario_name_that_is_not_a_string_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [scenario]
            name = 42
            [[systems]]
            name = "Test System"
            networks = ["TestNet"]
            base_box = "Debian"
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::from_toml(&input).unwrap_err().description(),
            "Could not read name of scenario as a string".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_toml_without_systems_array_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [scenario]
            name = "Test scenario"
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::from_toml(&input).unwrap_err().description(),
            "Could not get systems from configuration".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_toml_without_networks_array_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            name = "Test System"
            networks = ["TestNet"]
            base_box = "Debian"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::from_toml(&input).unwrap_err().description(),
            "Could not read networks from configuration".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_system_with_networks_array_containing_non_existant_network_name_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            name = "Test System"
            networks = ["OtherNet"]
            base_box = "Debian"
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::from_toml(&input).unwrap_err().description(),
            r#"System "Test System" is configured to use network "OtherNet" but no network with that name could be found"#.to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_scenario_with_duplicate_network_names_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            name = "Test System"
            networks = ["TestNet"]
            base_box = "Debian"
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.0/24"
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.1.0/24"
        "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::from_toml(&input).unwrap_err().description(),
            r#"Multiple networks parsed with name "TestNet". Network names must be unique."#
                .to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_single_system_and_network_scenario_works(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [scenario]
            name = "Test scenario"
            
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
            type = "Internal"
            subnet = "192.168.0.1/24"
                
        "#
        .parse::<Value>()?;

        let scenario = Scenario::from_toml(&input)?;

        assert_eq!(scenario.name, "Test scenario");
        assert_eq!(scenario.networks.len(), 1);
        assert_eq!(scenario.systems.len(), 2);

        assert_eq!(scenario.networks[0].name, "LAN");
        assert_eq!(scenario.networks[0].network_type, NetworkType::Internal);
        assert_eq!(
            scenario.networks[0].subnet.unwrap(),
            Ipv4Net::from_str("192.168.0.1/24").unwrap()
        );

        assert_eq!(scenario.systems[0].name, "Desktop");
        assert_eq!(scenario.systems[0].networks[0], scenario.networks[0]);
        assert_eq!(scenario.systems[0].base_box, "Windows 10");

        assert_eq!(scenario.systems[1].name, "Server");
        assert_eq!(scenario.systems[1].networks[0], scenario.networks[0]);
        assert_eq!(scenario.systems[1].base_box, "Debian");

        Ok(())
    }

}
