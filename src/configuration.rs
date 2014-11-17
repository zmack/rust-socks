use std::io::File;
use std::io::BufferedReader;
use std::io::net::ip::{IpAddr, Ipv4Addr, SocketAddr};

pub struct Configuration {
    pub whitelisted_ips: Vec<IpAddr>,
    pub listen_ip: String,
    pub listen_port: u16,
}

impl Configuration {
    pub fn new(filename: Path) -> Configuration {
        let file = match File::open(&filename) {
            Ok(f) => f,
            _ => return Configuration {
                whitelisted_ips: Vec::new(),
                listen_ip: "127.0.0.1".to_string(),
                listen_port: 1080
            }
        };

        return Configuration {
            whitelisted_ips: Configuration::get_whitelisted_ips(file),
            listen_ip: "127.0.0.1".to_string(),
            listen_port: 1080
        };
    }

    fn get_whitelisted_ips(file: File) -> Vec<IpAddr> {
        let mut ip_vec:Vec<IpAddr> = Vec::new();
        let mut reader = BufferedReader::new(file);
        for line in reader.lines() {
            match line {
                Ok(s) => {
                    let ip = match from_str(s.as_slice().trim()) {
                        Some(res) => res,
                        _ => break
                    };
                    ip_vec.push(ip)
                },
                _ => break
            }
        }
        println!("Ip_vec -> {}", ip_vec);

        return ip_vec;
    }
}

