use std::collections::HashMap;
use utils::yarn::{Api, ApisConfig, ReqHash};

use serde_json::Result;

use crate::utils;

pub enum CurrentScreen {
    Main,
    Editing,
    Exiting,
}

pub enum CurrentlyEditing {
    Key,
    Value,
}

pub struct App {
    pub configs: HashMap<String, Api>,
    pub apis_infos: HashMap<String, Api>

}

impl App {
    pub fn new(configs: &HashMap<String, Api>) -> App {
        App {
            configs: configs.clone(),
        }
    }

    pub fn format_api_infos(&mut self) {
        let apis = Vec::<HashMap<&str, &str>>::new();
        for config in self.configs {
            let mut api = HashMap::new();
            api.insert("name",config.name);
            api.insert("url",config.request.url);
            api.insert("method",config.request.method);
            api.insert("status","OK"); // TODO
            apis.push(api)

        }
    }
}