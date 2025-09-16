use ipnet::{Ipv4Net, Ipv6Net};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ASNInfo {
    pub asn: u32,
    pub handle: String,
    pub description: String,
    pub subnets: Subnets,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subnets {
    pub ipv4: Vec<Ipv4Net>,
    pub ipv6: Vec<Ipv6Net>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_asn_info_serialization() {
        let asn_info = ASNInfo {
            asn: 1234,
            handle: "FORTUM".to_string(),
            description: "Fortum".to_string(),
            subnets: Subnets {
                ipv4: vec![
                    "132.171.0.0/16".parse().unwrap(),
                    "137.96.0.0/16".parse().unwrap(),
                    "193.110.32.0/21".parse().unwrap(),
                ],
                ipv6: vec!["2405:1800::/32".parse().unwrap()],
            },
        };

        let json = serde_json::to_value(&asn_info).unwrap();
        assert_eq!(
            json,
            json!({
                "asn": 1234,
                "handle": "FORTUM",
                "description": "Fortum",
                "subnets": {
                    "ipv4": [
                        "132.171.0.0/16",
                        "137.96.0.0/16",
                        "193.110.32.0/21"
                    ],
                    "ipv6": [
                        "2405:1800::/32"
                    ]
                }
            })
        );
    }
}
