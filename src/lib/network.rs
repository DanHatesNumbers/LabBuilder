use ipnet::{Ipv4AddrRange, Ipv4Net};
use std::rc::Rc;
use toml::Value;

#[derive(Debug, PartialEq)]
pub struct Network {
    pub name: String,
    pub network_type: NetworkType,
    pub subnet: Option<Ipv4Net>,
    available_hosts: Option<Ipv4AddrRange>,
}

#[derive(Debug, PartialEq)]
pub enum NetworkType {
    Public,
    Internal,
}

impl Network {
    pub fn from_toml(
        network_toml: &Value,
    ) -> Result<Rc<Network>, std::boxed::Box<std::error::Error>> {
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
        }?;

        let mut subnet: Option<Ipv4Net> = None;
        let mut available_hosts: Option<Ipv4AddrRange> = None;

        if network_type == NetworkType::Internal {
            subnet = Some(
                network_toml
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
                    })?,
            );

            available_hosts = Some(subnet.unwrap().hosts());
        }

        Ok(Rc::new(Network {
            name: network_name,
            network_type: network_type,
            subnet: subnet,
            available_hosts: available_hosts,
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

    #[test]
    fn parsing_network_with_subnet_too_small_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.1/31"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Network::from_toml(&input).unwrap_err().description(),
            r#"Network "TestNet" configured with a subnet smaller than /30. Networks smaller than /30 can't have multiple hosts."#.to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_network_with_non_rfc1918_compliant_subnet_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            name = "TestNet"
            type = "Internal"
            subnet = "1.1.1.1/8"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Network::from_toml(&input).unwrap_err().description(),
            r#"Subnet configured for network "TestNet" is not RFC 1918 compliant. Subnets must be in valid alocation for private networks."#.to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_network_with_subnet_start_in_rfc1918_space_and_end_outside_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            name = "TestNet"
            type = "Internal"
            subnet = "192.168.0.0/15"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Network::from_toml(&input).unwrap_err().description(),
            r#"Subnet configured for network "TestNet" starts in RFC1918 space but extends beyond RFC1918 space."#.to_string()
        );
        Ok(())
    }

    #[test]
    fn parsing_public_network_should_not_configure_subnet_or_available_hosts(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            name = "TestNet"
            type = "Public"
            subnet = "192.168.0.0/24"
            "#
        .parse::<Value>()?;

        let result = Network::from_toml(&input)?;

        assert_eq!(result.subnet.is_none(), true);
        assert_eq!(result.available_hosts.is_none(), true);

        Ok(())
    }
}
