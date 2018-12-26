use crate::lib::network::Network;

use toml::Value;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct System {
    pub name: String,
    pub networks: Vec<Rc<Network>>,
    pub base_box: String,
}

impl System {
    pub fn from_toml(system_toml: &Value, scenario_networks: &Vec<Rc<Network>>) -> Result<System, std::boxed::Box<std::error::Error>> {
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

                let system_networks: Result<Vec<Rc<Network>>, std::boxed::Box<std::error::Error>> =
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

                            Ok(Rc::clone(scenario_networks
                                .iter()
                                .find(|&network| network.name == network_name)
                                .ok_or(format!(
                                    r#"System "{}" is configured to use network "{}" but no network with that name could be found"#,
                                    system.name, network_name 
                                ))?))
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
    }
}

#[cfg(test)]
mod tests {
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
            *Scenario::from_toml(&input).unwrap_err().description(),
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
            *Scenario::from_toml(&input).unwrap_err().description(),
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
            *Scenario::from_toml(&input).unwrap_err().description(),
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
            *Scenario::from_toml(&input).unwrap_err().description(),
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
            *Scenario::from_toml(&input).unwrap_err().description(),
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
            *Scenario::from_toml(&input).unwrap_err().description(),
            "Could not parse networks for system: Test System".to_string()
        );
        Ok(())
    }
}