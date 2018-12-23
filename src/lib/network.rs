use ipnet::Ipv4Net;
use toml::Value;

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
    pub fn from_toml(network_toml: &Value) -> Result<Network, std::boxed::Box<std::error::Error>> {
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

        Ok(Network {
            name: network_name,
            network_type: network_type?,
            subnet: subnet,
        })
    }
}
