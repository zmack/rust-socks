use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::str::FromStr;
use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub struct User {
    username: String,
    password_hash: String
}

pub struct Configuration {
    pub whitelisted_ips: Vec<IpAddr>,
    pub user_accounts: Vec<User>,
    pub listen_ip: String,
    pub listen_port: u16,
}

impl Configuration {
    pub fn new() -> Configuration {
        let configuration = match Configuration::parse_configuration() {
            Ok(f) => f,
            _ => Configuration {
                whitelisted_ips: Vec::new(),
                user_accounts: Vec::new(),
                listen_ip: "127.0.0.1".to_string(),
                listen_port: 1080
            }
        };

        return configuration
    }

    fn parse_configuration() -> Result<Configuration, String> {
        let whitelist = Configuration::get_whitelisted_ips(&Path::new("whitelisted_ips.conf"));
        let accounts = Configuration::get_accounts(&Path::new("accounts.conf"));

        Ok(Configuration {
            whitelisted_ips: whitelist,
            user_accounts: accounts,
            listen_ip: "127.0.0.1".to_string(),
            listen_port: 1080
        })

    }

    fn get_whitelisted_ips(path: &Path) -> Vec<IpAddr> {
        let mut ip_vec:Vec<IpAddr> = Vec::new();
        let file = match File::open(path) {
            Ok(f) => f,
            _ => return Vec::new()
        };

        let mut reader = BufReader::new(file);

        for line in reader.lines() {
            match IpAddr::from_str(line.unwrap().trim()) {
                Ok(res) => ip_vec.push(res),
                _ => break
            };
        }

        ip_vec
    }

    fn get_accounts(path: &Path) -> Vec<User> {
        let mut accounts:Vec<User> = Vec::new();
        let file = match File::open(path) {
            Ok(f) => f,
            _ => return Vec::new()
        };

        let mut reader = BufReader::new(file);

        for line in reader.lines() {
            let creds:Vec<String> = line.unwrap().split(':').map(|x| { x.to_string() }).collect();
            accounts.push(User{ username: creds[0].clone(), password_hash: creds[1].clone() });
        }

        accounts
    }
}
