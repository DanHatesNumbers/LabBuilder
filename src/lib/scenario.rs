use crate::lib::network::Network;
use crate::lib::system::System;

#[derive(Debug, PartialEq)]
pub struct Scenario<'a> {
    pub name: String,
    pub systems: Vec<System<'a>>,
    pub networks: Vec<&'a Network>,
}
