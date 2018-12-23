use ipnet::Ipv4Net;

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
