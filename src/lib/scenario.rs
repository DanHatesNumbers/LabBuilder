use crate::lib::network::{Network, NetworkType};
use crate::lib::system::System;
use ipnet::Ipv4Net;
use toml::Value;

use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Scenario<'a> {
    pub name: String,
    pub systems: Vec<System<'a>>,
    pub networks: Vec<Network>,
}

impl<'a> Scenario<'a> {
    pub fn parse_scenario(
        scenario_toml: &'a Value,
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

        let networks: Result<Vec<Network>, std::boxed::Box<std::error::Error>> = scenario_toml
            .get("networks")
            .ok_or("Could not read networks from configuration")?
            .as_array()
            .ok_or("Could not read networks from configuration")?
            .into_iter()
            .map(|network_toml| {
                let network_name = network_toml
                    .get("name")
                    .ok_or("Could not read network name")?
                    .as_str()
                    .ok_or("Could not read network name as a string")?
                    .into();

                let network_type_unparsed = network_toml
                    .get("type")
                    .ok_or(format!(
                        "Could not read network type for network: {}",
                        network_name
                    ))?
                    .as_str()
                    .ok_or(format!(
                        "Could not read network type as a string for network: {}",
                        network_name
                    ))?
                    .into();

                let network_type = match network_type_unparsed {
                    "Public" => Ok(NetworkType::Public),
                    "Internal" => Ok(NetworkType::Internal),
                    _ => Err(format!(
                        "Could not parse network type as a valid type for network: {}. Valid types are: Public, Internal",
                        network_name
                    )),
                };

                let subnet: Ipv4Net = network_toml
                    .get("subnet")
                    .ok_or(format!(
                        "Could not read subnet for network: {}",
                        network_name))?
                    .as_str()
                    .ok_or(format!(
                        "Could not read subnet as string for network: {}",
                        network_name))?
                    .parse()
                    .map_err(|_| format!(
                        "Could not parse subnet as a valid CIDR range for network: {}",
                        network_name))?;

                Ok(Network {
                    name: network_name,
                    network_type: network_type?,
                    subnet: subnet,
                })
            })
            .collect();

        scenario.networks.append(&mut networks?);

        let systems: Result<Vec<System>, std::boxed::Box<std::error::Error>> = scenario_toml
            .get("systems")
            .ok_or("Could not get systems from configuration")?
            .as_array()
            .ok_or("Could not get systems from configuration")?
            .into_iter()
            .map(|system_toml| {
                let mut system = System {
                    name: "".into(),
                    networks: Vec::new(),
                    base_box: "".into(),
                };

                system.name = system_toml
                    .get("name")
                    .ok_or("Could not read name of system")?
                    .as_str()
                    .ok_or("Could not read name of system as a string")?
                    .into();

                let system_networks: Result<Vec<&Network>, std::boxed::Box<std::error::Error>> =
                    system_toml
                        .get("networks")
                        .ok_or(format!(
                            "Could not read networks for system: {}",
                            system.name
                        ))?
                        .as_array()
                        .ok_or(format!(
                            "Could not read networks for system: {}",
                            system.name
                        ))?
                        .into_iter()
                        .map(|network_name_toml| {
                            let network_name = network_name_toml.as_str().ok_or(format!(
                                "Could not parse networks for system: {}",
                                system.name
                            ))?;

                            Ok(scenario
                                .networks
                                .iter()
                                .find(|&network| network.name == network_name)
                                .ok_or(format!(
                                    r#"System "{}" is configured to use network "{}" but no network with that name could be found"#,
                                    system.name, network_name 
                                ))?)
                        })
                        .collect();

                system.networks.append(&mut system_networks?);

                system.base_box = system_toml
                    .get("base_box")
                    .ok_or(format!(
                        "Could not read base_box for system: {}",
                        system.name
                    ))?
                    .as_str()
                    .ok_or(format!(
                        "Could not read base_box as a string for system: {}",
                        system.name
                    ))?
                    .into();

                Ok(system)
            })
            .collect();

        scenario.systems.append(&mut systems?);

        Ok(scenario)
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
            *Scenario::parse_scenario(&input).unwrap_err().description(),
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
            *Scenario::parse_scenario(&input).unwrap_err().description(),
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
            *Scenario::parse_scenario(&input).unwrap_err().description(),
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
            *Scenario::parse_scenario(&input).unwrap_err().description(),
            "Could not get systems from configuration".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_system_without_name_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            networks = ["TestNet"]
            base_box = "Debian"
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::parse_scenario(&input).unwrap_err().description(),
            "Could not read name of system".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_system_with_name_that_is_not_a_string_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            name = 42
            networks = ["TestNet"]
            base_box = "Debian"
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::parse_scenario(&input).unwrap_err().description(),
            "Could not read name of system as a string".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_system_without_base_box_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            name = "Test System"
            networks = ["TestNet"]
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::parse_scenario(&input).unwrap_err().description(),
            "Could not read base_box for system: Test System".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_system_with_base_box_that_is_not_a_string_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            name = "Test System"
            networks = ["TestNet"]
            base_box = 42
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::parse_scenario(&input).unwrap_err().description(),
            "Could not read base_box as a string for system: Test System".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_system_without_networks_array_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            name = "Test System"
            base_box = "Debian"
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::parse_scenario(&input).unwrap_err().description(),
            "Could not read networks for system: Test System".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_system_with_networks_array_containing_something_other_than_strings_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            name = "Test System"
            networks = [42]
            base_box = "Debian"
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::parse_scenario(&input).unwrap_err().description(),
            "Could not parse networks for system: Test System".to_string()
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
            *Scenario::parse_scenario(&input).unwrap_err().description(),
            "Could not read networks from configuration".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_network_without_name_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            name = "Test System"
            networks = ["TestNet"]
            base_box = "Debian"
            [[networks]]
            type = "Internal"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::parse_scenario(&input).unwrap_err().description(),
            "Could not read network name".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_network_with_name_that_is_not_a_string_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            name = "Test System"
            networks = ["TestNet"]
            base_box = "Debian"
            [[networks]]
            name = 42
            type = "Internal"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::parse_scenario(&input).unwrap_err().description(),
            "Could not read network name as a string".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_network_without_type_should_fail_with_msg(
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
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::parse_scenario(&input).unwrap_err().description(),
            "Could not read network type for network: TestNet".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_network_with_type_that_is_not_a_string_should_fail_with_msg(
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
            type = 42
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::parse_scenario(&input).unwrap_err().description(),
            "Could not read network type as a string for network: TestNet".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_network_with_type_that_is_not_valid_should_fail_with_msg(
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
            type = "NotValid"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::parse_scenario(&input).unwrap_err().description(),
            "Could not parse network type as a valid type for network: TestNet. Valid types are: Public, Internal".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_network_with_invalid_subnet_should_fail_with_msg(
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
            subnet = "192.168.0.1"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Scenario::parse_scenario(&input).unwrap_err().description(),
            "Could not parse subnet as a valid CIDR range for network: TestNet".to_string()
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
            *Scenario::parse_scenario(&input).unwrap_err().description(),
            r#"System "Test System" is configured to use network "OtherNet" but no network with that name could be found"#.to_string()
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

        let scenario = Scenario::parse_scenario(&input)?;

        assert_eq!(scenario.name, "Test scenario");
        assert_eq!(scenario.networks.len(), 1);
        assert_eq!(scenario.systems.len(), 2);

        assert_eq!(scenario.networks[0].name, "LAN");
        assert_eq!(scenario.networks[0].network_type, NetworkType::Internal);
        assert_eq!(
            scenario.networks[0].subnet,
            Ipv4Net::from_str("192.168.0.1/24").unwrap()
        );

        assert_eq!(scenario.systems[0].name, "Desktop");
        assert_eq!(scenario.systems[0].networks[0], &scenario.networks[0]);
        assert_eq!(scenario.systems[0].base_box, "Windows 10");

        assert_eq!(scenario.systems[1].name, "Server");
        assert_eq!(scenario.systems[1].networks[0], &scenario.networks[0]);
        assert_eq!(scenario.systems[1].base_box, "Debian");

        Ok(())
    }

}
