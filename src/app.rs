use job_scheduler::{Job, JobScheduler};
use reqwest::header::{HeaderMap, HeaderName};
use serde_yaml::Error;
use std::{collections::HashMap, str::FromStr, thread, time::Duration};
use utils::yarn::{Api, ApisConfig, ReqHash};

use serde_json::Result;

use crate::{
    request::{get_depends_result, request_api},
    utils,
};

pub enum CurrentScreen {
    Main,
    Editing,
    Exiting,
}

pub enum CurrentlyEditing {
    Key,
    Value,
}

pub struct App<'a> {
    // pub configs: HashMap<std::string::String, Api>,
    pub configs: &'a ApisConfig,
    pub apis_infos: Vec<HashMap<std::string::String, std::string::String>>,
}

impl App<'_> {
    pub fn new(configs: &ApisConfig) -> App {
        App {
            configs,
            apis_infos: Vec::<HashMap<std::string::String, std::string::String>>::new(),
        }
    }

    pub fn format_api_infos(&mut self) {
        for config in self.configs.requests.iter() {
            let mut api = HashMap::new();
            api.insert("name".to_string(), config.1.name.to_string());
            api.insert("url".to_string(), config.1.request.url.to_string());
            api.insert("method".to_string(), config.1.request.method.to_string());
            api.insert("status".to_string(), "OK".to_string()); // TODO
            self.apis_infos.push(api)
        }
    }

    #[tokio::main]
    pub async fn verify_api(&mut self, api: &Api) {
        let mut headers_map = HeaderMap::new();

        // println!("{:?}", api.depends_on);

        let fields_required = match &api.depends_on {
            Some(depends) => get_depends_result(&depends, api.system_notify).await,
            None => HashMap::new(),
        };

        // println!("headers main: {:?}", api.request.headers);

        for header in &api.request.headers {
            for (key, value) in header {
                if fields_required.len() > 0 {
                    for results in &fields_required["depends_headers"] {
                        for (field_name, field_value) in results {
                            let field = &format!("{{{}}}", field_name);

                            if let Some(_) = value.find(field) {
                                let replace_value = value.replace(field, &field_value);
                                headers_map.insert(
                                    HeaderName::from_str(&key).unwrap(),
                                    replace_value.parse().unwrap(),
                                );
                            } else if headers_map.get(key) == None {
                                headers_map.insert(
                                    HeaderName::from_str(&key).unwrap(),
                                    value.parse().unwrap(),
                                );
                            }
                        }
                    }
                } else {
                    headers_map.insert(HeaderName::from_str(&key).unwrap(), value.parse().unwrap());
                }
            }
        }

        let body = if fields_required.len() > 0 {
            let mut custom_body = ReqHash::new();

            for body in &api.request.body {
                for (key, value) in body {
                    for results in &fields_required["depends_body"] {
                        for (field_name, field_value) in results {
                            let field = &format!("{{{}}}", field_name);

                            let value_converted: String =
                                serde_json::from_str(&field_value).unwrap();

                            if let Some(_) = value.find(field) {
                                let replace_value = value.replace(field, &value_converted);

                                custom_body.insert(key.to_owned(), replace_value);
                            } else if body.get(key) == None {
                                custom_body.insert(key.to_string(), value.to_string());
                            }
                        }
                    }
                }
            }
            Some(custom_body)
        } else {
            api.request.body.to_owned()
        };

        let response = request_api(&api.request, headers_map, &body).await;

        let status = match response {
            Ok(resp) => resp.status().as_u16(),
            Err(e) => {
                return ();
            }
        };

        println!("received {0}: spected {1}", status, api.expected_status);
        let mut api_details = HashMap::new();

        api_details.insert("name".to_string(), api.name.to_string());
        api_details.insert("url".to_string(), api.request.url.to_string());
        api_details.insert("method".to_string(), api.request.method.to_string());
        // self.apis_infos.push(api_details);

        if status == api.expected_status {
            if api.notify_type != "ERROR" {
                api_details.insert("status".to_string(), "OK".to_string()); // TODO
            }
        } else {
            api_details.insert("status".to_string(), "ERROR".to_string()); // TODO
        }
        self.apis_infos.push(api_details);
    }

    // pub async fn start_monitor(&mut self) {
    //     let mut sched: JobScheduler<'_> = JobScheduler::new();

    //     for config in &self.configs {
    //         // self.verify_api(&config.1);
    //         sched.add(Job::new(
    //             config.1.cron_expression.parse().unwrap(),
    //             move || {
    //                 let aaa = config.1.clone();
    //                 self.verify_api(&aaa);
    //             },
    //         ));
    //     }

    //     loop {
    //         sched.tick();

    //         std::thread::sleep(Duration::from_millis(1000));
    //     }
    // }
}
