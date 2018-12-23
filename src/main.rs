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
        .ok_or("Could not get scenario from configuration")?
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
            *parse_scenario(&input).unwrap_err().description(),
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
            *parse_scenario(&input).unwrap_err().description(),
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
            *parse_scenario(&input).unwrap_err().description(),
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
            *parse_scenario(&input).unwrap_err().description(),
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
            *parse_scenario(&input).unwrap_err().description(),
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
            *parse_scenario(&input).unwrap_err().description(),
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
            *parse_scenario(&input).unwrap_err().description(),
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
            *parse_scenario(&input).unwrap_err().description(),
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
            *parse_scenario(&input).unwrap_err().description(),
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
            *parse_scenario(&input).unwrap_err().description(),
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
            *parse_scenario(&input).unwrap_err().description(),
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
            *parse_scenario(&input).unwrap_err().description(),
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
            *parse_scenario(&input).unwrap_err().description(),
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
            *parse_scenario(&input).unwrap_err().description(),
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
            *parse_scenario(&input).unwrap_err().description(),
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
            *parse_scenario(&input).unwrap_err().description(),
            "Could not parse network type as a valid type for network: TestNet. Valid types are: Public, Internal".to_string()
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
            *parse_scenario(&input).unwrap_err().description(),
            "System 'Test System' is configured to use network 'OtherNet' but no network with that name could be found".to_string()
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
            network_type = "internal"
            subnet = "192.168.0.1/24"
                
        "#
        .parse::<Value>()?;

        let scenario = parse_scenario(&input)?;

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
        assert_eq!(scenario.systems[0].networks[0], scenario.networks[0]);
        assert_eq!(scenario.systems[0].base_box, "Windows 10");

        assert_eq!(scenario.systems[1].name, "Server");
        assert_eq!(scenario.systems[1].networks[1], scenario.networks[0]);
        assert_eq!(scenario.systems[1].base_box, "Debian");

        Ok(())
    }

}
