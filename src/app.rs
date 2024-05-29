use crate::utils;

use std::convert::TryInto;

use utils::yarn::ApisConfig;


pub struct App {
    pub configs: ApisConfig,
    pub apis_infos: Vec<ListRequests>,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct ApiInformation {
    pub name: String,
    pub url: String,
    pub method: String,
    pub status: String,
}

#[derive(Debug)]
#[derive(Clone)]
pub struct ListRequests {
    pub(crate) id: usize,
    pub data: ApiInformation,
    pub interval: u64,
}

impl App {
    pub fn new(configs: ApisConfig) -> App {
        App {
            configs,
            apis_infos: Vec::<ListRequests>::new(),
        }
    }

    pub fn append_satus2(&mut self, result: Vec<ListRequests>) {
        self.apis_infos = result
    }

    pub fn format_api_infos(&mut self) {
        for (id, config) in self.configs.requests.iter().enumerate() {
            let api = ApiInformation {
                name:config.1.name.to_string(),
                url: config.1.request.url.to_string(),
                method: config.1.request.method.to_string(),
                status: "WAINTING".to_string(),
            };
            let new_request = ListRequests {
                id: id.try_into().unwrap(),
                data: api,
                interval: config.1.interval
            };
            self.apis_infos.push(new_request)
        }
    }

}
