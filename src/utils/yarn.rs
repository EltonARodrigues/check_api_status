use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type ReqHash = HashMap<String, String>;

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub enum ConfigMethod {
    GET,
    POST,
    // DELETE,
    // PUT,
    // PATH,
}

impl ToString for ConfigMethod {
    fn to_string(&self) -> String {
        match self {
            ConfigMethod::GET => String::from("GET"),
            ConfigMethod::POST => String::from("POST"),
        }
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct Request {
    pub url: String,
    pub headers: Option<ReqHash>,
    pub method: ConfigMethod,
    pub body: Option<ReqHash>,
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct Depends {
    pub name: String,
    pub header_fields: Vec<String>,
    pub body_fields: Vec<String>,
    pub request: Request,
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct Api {
    pub name: String,
    pub depends_on: Option<Depends>,
    pub request: Request,
    pub expected_status: u16,
    pub cron_expression: String,
    pub system_notify: bool,
    pub notify_type: String,
    // pub one_time_notify: bool,
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct ApisConfig {
    pub requests: HashMap<String, Api>,
}
