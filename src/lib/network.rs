use ipnet::Ipv4Net;
use toml::Value;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct Network {
    pub name: String,
    pub network_type: NetworkType,
    pub subnet: Ipv4Net,
}

#[derive(Debug, PartialEq)]
pub enum NetworkType {
    Public,
    Internal,
}

impl Network {
    pub fn from_toml(network_toml: &Value) -> Result<Rc<Network>, std::boxed::Box<std::error::Error>> {
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
                network_name
            ))?
            .as_str()
            .ok_or(format!(
                "Could not read subnet as string for network: {}",
                network_name
            ))?
            .parse()
            .map_err(|_| {
                format!(
                    "Could not parse subnet as a valid CIDR range for network: {}",
                    network_name
                )
            })?;

        Ok(Rc::new(Network {
            name: network_name,
            network_type: network_type?,
            subnet: subnet,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_network_without_name_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            type = "Internal"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Network::from_toml(&input).unwrap_err().description(),
            "Could not read network name".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_network_with_name_that_is_not_a_string_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            name = 42
            type = "Internal"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Network::from_toml(&input).unwrap_err().description(),
            "Could not read network name as a string".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_network_without_type_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            name = "TestNet"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Network::from_toml(&input).unwrap_err().description(),
            "Could not read network type for network: TestNet".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_network_with_type_that_is_not_a_string_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            name = "TestNet"
            type = 42
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Network::from_toml(&input).unwrap_err().description(),
            "Could not read network type as a string for network: TestNet".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_network_with_type_that_is_not_valid_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            name = "TestNet"
            type = "NotValid"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Network::from_toml(&input).unwrap_err().description(),
            "Could not parse network type as a valid type for network: TestNet. Valid types are: Public, Internal".to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_network_with_invalid_subnet_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.1"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Network::from_toml(&input).unwrap_err().description(),
            "Could not parse subnet as a valid CIDR range for network: TestNet".to_string()
        );
        Ok(())
    }
}
