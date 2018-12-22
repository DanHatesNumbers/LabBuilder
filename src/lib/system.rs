use crate::lib::network::Network;

#[derive(Debug, PartialEq)]
pub struct System<'a> {
    pub name: String,
    pub networks: Vec<&'a Network>,
    pub base_box: String,
}
