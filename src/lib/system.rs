use crate::lib::network::{Network, NetworkType};

use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::rc::Rc;
use toml::Value;

#[derive(Debug, PartialEq)]
pub struct System {
    pub name: String,
    pub networks: Vec<Rc<Network>>,
    network_names: Vec<String>,
    pub base_box: String,
    pub leased_network_addresses: HashMap<String, Vec<Ipv4Addr>>,
}

impl System {
    pub fn from_toml(system_toml: &Value) -> Result<System, std::boxed::Box<std::error::Error>> {
        let mut system = System {
            name: "".into(),
            networks: Vec::new(),
            network_names: Vec::new(),
            base_box: "".into(),
            leased_network_addresses: HashMap::new(),
        };

        system.name = system_toml
            .get("name")
            .ok_or("Could not read name of system")?
            .as_str()
            .ok_or("Could not read name of system as a string")?
            .into();

        let network_names: Result<Vec<String>, std::boxed::Box<std::error::Error>> = system_toml
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
                Ok(network_name.to_string())
            })
            .collect();

        system.network_names.append(&mut network_names?);

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

    pub fn configure_networking(
        &mut self,
        scenario_networks: &Vec<Rc<Network>>,
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let system_networks: Result<Vec<Rc<Network>>, std::boxed::Box<std::error::Error>> =
            self.network_names.iter()
                .map(|network_name|
                    Ok(Rc::clone(scenario_networks
                        .iter()
                        .find(|&network| &network.name == network_name)
                        .ok_or(format!(
                            r#"System "{}" is configured to use network "{}" but no network with that name could be found"#,
                            self.name, network_name
                        ))?))
                )
                .collect();

        self.networks.append(&mut system_networks?);

        let internal_nets: Vec<Rc<Network>> = self
            .networks
            .iter()
            .cloned()
            .filter(|net| net.network_type == NetworkType::Internal)
            .collect();

        for net in internal_nets.into_iter() {
            let leased_addr = net.get_address_lease()
                .ok_or(format!(r#"Subnet for network "{}" does not have enough available addresses for all systems configured to use it."#, net.name.to_string()))?;

            self.leased_network_addresses
                .entry((&net.name).to_string())
                .and_modify(|e| {
                    e.push(leased_addr);
                })
                .or_insert_with(|| {
                    return vec![leased_addr];
                });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::lib::scenario::Scenario;

    #[test]
    fn parsing_system_without_name_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            networks = ["TestNet"]
            base_box = "Debian"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *System::from_toml(&input).unwrap_err().description(),
            "Could not read name of system".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_system_with_name_that_is_not_a_string_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            name = 42
            networks = ["TestNet"]
            base_box = "Debian"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *System::from_toml(&input).unwrap_err().description(),
            "Could not read name of system as a string".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_system_without_base_box_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            name = "Test System"
            networks = ["TestNet"]
            "#
        .parse::<Value>()?;

        assert_eq!(
            *System::from_toml(&input).unwrap_err().description(),
            "Could not read base_box for system: Test System".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_system_with_base_box_that_is_not_a_string_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            name = "Test System"
            networks = ["TestNet"]
            base_box = 42
            "#
        .parse::<Value>()?;

        assert_eq!(
            *System::from_toml(&input).unwrap_err().description(),
            "Could not read base_box as a string for system: Test System".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_system_without_networks_array_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            name = "Test System"
            base_box = "Debian"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *System::from_toml(&input).unwrap_err().description(),
            "Could not read networks for system: Test System".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_system_with_networks_array_containing_something_other_than_strings_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            name = "Test System"
            networks = [42]
            base_box = "Debian"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *System::from_toml(&input).unwrap_err().description(),
            "Could not parse networks for system: Test System".to_string()
        );
        Ok(())
    }

    #[test]
    fn configuring_networking_with_1_public_network_should_not_lease_address(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let scenario_toml = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            name = "Test system"
            base_box = "Debian"
            networks = ["TestNet"]
            [[networks]]
            name = "TestNet"
            type = "Public"
        "#
        .parse::<Value>()?;

        let mut scenario = Scenario::from_toml(&scenario_toml)?;
        scenario.systems[0].configure_networking(&scenario.networks)?;

        let leased_addresses = &scenario.systems[0].leased_network_addresses;
        assert_eq!(leased_addresses.is_empty(), true);
        Ok(())
    }

    #[test]
    fn configuring_networking_with_1_internal_network_should_work(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let scenario_toml = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            name = "Test system"
            base_box = "Debian"
            networks = ["TestNet"]
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.1/24"
        "#
        .parse::<Value>()?;

        let mut scenario = Scenario::from_toml(&scenario_toml)?;
        scenario.systems[0].configure_networking(&scenario.networks)?;

        let leased_addresses = &scenario.systems[0].leased_network_addresses;
        assert_eq!(leased_addresses.len(), 1);
        assert_eq!(leased_addresses.contains_key("TestNet"), true);
        assert_eq!(
            scenario.networks[0]
                .subnet
                .unwrap()
                .contains(&leased_addresses["TestNet"][0]),
            true
        );
        Ok(())
    }

    #[test]
    fn configuring_networking_with_2_NICs_in_same_internal_network_should_work(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let scenario_toml = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            name = "Test system"
            base_box = "Debian"
            networks = ["TestNet", "TestNet"]
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.1/24"
        "#
        .parse::<Value>()?;

        let mut scenario = Scenario::from_toml(&scenario_toml)?;
        scenario.systems[0].configure_networking(&scenario.networks)?;

        let leased_addresses = &scenario.systems[0].leased_network_addresses;
        assert_eq!(leased_addresses.len(), 1);
        assert_eq!(leased_addresses.contains_key("TestNet"), true);
        assert_eq!(leased_addresses["TestNet"].len(), 2);
        assert_eq!(
            scenario.networks[0]
                .subnet
                .unwrap()
                .contains(&leased_addresses["TestNet"][0]),
            true
        );
        assert_eq!(
            scenario.networks[0]
                .subnet
                .unwrap()
                .contains(&leased_addresses["TestNet"][1]),
            true
        );
        assert_eq!(
            leased_addresses["TestNet"][0] == leased_addresses["TestNet"][1],
            false
        );
        Ok(())
    }

    #[test]
    fn configuring_networking_with_2_different_internal_networks_should_work(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let scenario_toml = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            name = "Test system"
            base_box = "Debian"
            networks = ["TestNet", "OtherNet"]
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.1/24"
            [[networks]]
            name = "OtherNet"
            type = "Internal"
            subnet = "192.168.1.1/24"
        "#
        .parse::<Value>()?;

        let mut scenario = Scenario::from_toml(&scenario_toml)?;
        scenario.systems[0].configure_networking(&scenario.networks)?;

        let leased_addresses = &scenario.systems[0].leased_network_addresses;
        assert_eq!(leased_addresses.len(), 2);
        assert_eq!(leased_addresses.contains_key("TestNet"), true);
        assert_eq!(leased_addresses.contains_key("OtherNet"), true);
        assert_eq!(leased_addresses["TestNet"].len(), 1);
        assert_eq!(leased_addresses["OtherNet"].len(), 1);
        assert_eq!(
            scenario.networks[0]
                .subnet
                .unwrap()
                .contains(&leased_addresses["TestNet"][0]),
            true
        );
        assert_eq!(
            scenario.networks[1]
                .subnet
                .unwrap()
                .contains(&leased_addresses["OtherNet"][0]),
            true
        );

        Ok(())
    }

    #[test]
    fn configuring_networking_for_2_systems_in_same_internal_network_should_work(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let scenario_toml = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            name = "Test system"
            base_box = "Debian"
            networks = ["TestNet"]
            [[systems]]
            name = "Test system 2"
            base_box = "Debian"
            networks = ["TestNet"]
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.1/24"
        "#
        .parse::<Value>()?;

        let mut scenario = Scenario::from_toml(&scenario_toml)?;
        scenario.systems[0].configure_networking(&scenario.networks)?;
        scenario.systems[1].configure_networking(&scenario.networks)?;

        let mut leased_addresses = Vec::new();
        leased_addresses.push(&scenario.systems[0].leased_network_addresses);
        leased_addresses.push(&scenario.systems[1].leased_network_addresses);

        for x in leased_addresses.iter() {
            assert_eq!(x.len(), 1);
            assert_eq!(x.contains_key("TestNet"), true);
            assert_eq!(x["TestNet"].len(), 1);
            assert_eq!(
                scenario.networks[0]
                    .subnet
                    .unwrap()
                    .contains(&x["TestNet"][0]),
                true
            );
        }

        assert_eq!(
            leased_addresses[0]["TestNet"][0] == leased_addresses[1]["TestNet"][0],
            false
        );

        Ok(())
    }

    #[test]
    fn configuring_networking_for_2_systems_in_subnet_that_is_too_small_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let scenario_toml = r#"
            [scenario]
            name = "Test scenario"
            [[systems]]
            name = "Test system"
            base_box = "Debian"
            networks = ["TestNet"]
            [[systems]]
            name = "Test system 2"
            base_box = "Debian"
            networks = ["TestNet"]
            [[systems]]
            name = "Test system 3"
            base_box = "Debian"
            networks = ["TestNet"]
            [[networks]]
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.1/30"
        "#
        .parse::<Value>()?;

        let mut scenario = Scenario::from_toml(&scenario_toml)?;
        scenario.systems[0].configure_networking(&scenario.networks)?;
        scenario.systems[1].configure_networking(&scenario.networks)?;
        let result = scenario.systems[2].configure_networking(&scenario.networks);

        assert_eq!(
            result.unwrap_err().description(),
            r#"Subnet for network "TestNet" does not have enough available addresses for all systems configured to use it."#.to_string()
        );

        Ok(())
    }
}
