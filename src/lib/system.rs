use crate::lib::network::Network;
use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub struct System<'a> {
    pub name: String,
    pub networks: Vec<Rc<Network>>,
    pub base_box: String,
}
