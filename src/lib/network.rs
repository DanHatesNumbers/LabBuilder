use ipnet::{Ipv4AddrRange, Ipv4Net};
use std::cell::RefCell;
use std::collections::hash_set::HashSet;
use std::net::Ipv4Addr;
use std::rc::Rc;
use toml::Value;

#[derive(Debug, PartialEq)]
pub struct Network {
    pub name: String,
    pub network_type: NetworkType,
    pub subnet: Option<Ipv4Net>,
    available_hosts: Option<Ipv4AddrRange>,
    allocated_hosts: Option<RefCell<HashSet<Ipv4Addr>>>,
}

#[derive(Debug, PartialEq)]
pub enum NetworkType {
    Public,
    Internal,
}

impl Network {
    #[allow(clippy::useless_let_if_seq)]
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
            .ok_or_else(|| format!(
                "Could not read network type for network: {}",
                network_name
            ))?
            .as_str()
            .ok_or_else(|| format!(
                "Could not read network type as a string for network: {}",
                network_name
            ))?;

        let network_type = match network_type_unparsed {
            "Public" => Ok(NetworkType::Public),
            "Internal" => Ok(NetworkType::Internal),
            _ => Err(format!(
                "Could not parse network type as a valid type for network: {}. Valid types are: Public, Internal",
                network_name
            )),
        }?;

        let mut subnet: Option<Ipv4Net> = None;

        if network_type == NetworkType::Internal {
            subnet = Some(
                network_toml
                    .get("subnet")
                    .ok_or_else(|| format!(
                        "Could not read subnet for network: {}",
                        network_name
                    ))?
                    .as_str()
                    .ok_or_else(|| format!(
                        "Could not read subnet as string for network: {}",
                        network_name
                    ))?
                    .parse()
                    .map_err(|_| {
                        format!(
                            "Could not parse subnet as a valid CIDR range for network: {}",
                            network_name
                        )
                    })
                    .and_then(|subnet: Ipv4Net| 
                        match subnet.prefix_len() {
                            0...30 => Ok(subnet),
                            _ => Err(format!(r#"Network "{}" configured with a subnet smaller than /30. Networks smaller than /30 can't have multiple hosts."#, network_name))
                        }
                    )
                    .and_then(|subnet: Ipv4Net| {
                        let private_nets = vec![
                            "10.0.0.0/8".parse::<Ipv4Net>().unwrap(),
                            "172.16.0.0/12".parse::<Ipv4Net>().unwrap(),
                            "192.168.0.0/16".parse::<Ipv4Net>().unwrap(),
                        ];

                        let privacy_result = private_nets.iter()
                            .any(|&priv_net| priv_net.contains(&subnet));

                        if privacy_result {
                            Ok(subnet)
                        } else {
                            Err(format!(r#"Subnet configured for network "{}" is not RFC 1918 compliant. Subnets must be in valid allocation for private networks."#, network_name))
                        }
                    })?
            );
        } else {
            match network_toml.get("subnet") {
                None => Ok(()),
                Some(_) => Err(format!(r#"Network "{}" is configured as a Public network and has a subnet configured. Public networks can't have configured subnets."#, network_name)),
            }?
        }

        let available_hosts = match network_type {
            NetworkType::Internal => Some(subnet.unwrap().hosts()),
            NetworkType::Public => None,
        };
        let allocated_hosts: Option<RefCell<HashSet<Ipv4Addr>>> = match network_type {
            NetworkType::Internal => Some(RefCell::new(HashSet::new())),
            NetworkType::Public => None,
        };

        Ok(Rc::new(Network {
            name: network_name,
            network_type,
            subnet,
            available_hosts,
            allocated_hosts,
        }))
    }

    pub fn get_address_lease(&self) -> Option<Ipv4Addr> {
        if let Some(allocated_hosts) = &self.allocated_hosts {
            let leased_addr = self
                .available_hosts?
                .skip_while(|addr| allocated_hosts.borrow().contains(addr))
                .next();
            return match leased_addr {
                Some(addr) => {
                    allocated_hosts.borrow_mut().insert(addr);
                    return Some(addr);
                }
                None => None,
            };
        }
        None
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
            r#"Subnet configured for network "TestNet" is not RFC 1918 compliant. Subnets must be in valid allocation for private networks."#.to_string()
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
            r#"Subnet configured for network "TestNet" is not RFC 1918 compliant. Subnets must be in valid allocation for private networks."#.to_string() 
        );
        Ok(())
    }

    #[test]
    fn parsing_public_network_should_not_configure_subnet_or_available_hosts(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            name = "TestNet"
            type = "Public"
            "#
        .parse::<Value>()?;

        let result = Network::from_toml(&input)?;

        assert_eq!(result.subnet.is_none(), true);
        assert_eq!(result.available_hosts.is_none(), true);

        Ok(())
    }

    #[test]
    fn parsing_public_network_with_subnet_should_fail_with_msg(
    ) -> Result<(), std::boxed::Box<std::error::Error>> {
        let input = r#"
            name = "TestNet"
            type = "Public"
            subnet = "192.168.0.1/24"
            "#
        .parse::<Value>()?;

        assert_eq!(
            *Network::from_toml(&input).unwrap_err().description(),
            r#"Network "TestNet" is configured as a Public network and has a subnet configured. Public networks can't have configured subnets."#.to_string()
        );

        Ok(())
    }
}
