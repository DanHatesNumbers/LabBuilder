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