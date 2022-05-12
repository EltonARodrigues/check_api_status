mod request;
mod utils;

use request::{get_depends_result, request_api};

use utils::notify::send_notify;
use utils::yarn::{Api, ApisConfig, ReqHash};

use clap::Arg;
use clap::Command;
use job_scheduler::{Job, JobScheduler};
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use std::{fs, time::Duration};

#[tokio::main]
async fn verify_api(api: &Api) {
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

                        let value_converted: String = serde_json::from_str(&field_value).unwrap();

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
            println!("{0}: OK -> {1}", api.name, e);
            if api.system_notify == true {
                send_notify(
                    api.name.as_str(),
                    "dialog-error",
                    "Request failed",
                )
                .unwrap();
            }
            return ();
        }
    };

    println!("received {0}: spected {1}", status, api.expected_status);
    if status == api.expected_status {
        println!("{0}: OK -> {1}", api.name, status);
        if api.notify_type != "ERROR" && api.system_notify == true {
            let msg = format!("Expected: {} Received: {} - Everything is OK", status, api.expected_status);
            send_notify(
                api.name.as_str(),
                "dialog-information",
                &msg,
            )
            .unwrap();
        }
    } else {
        println!("{0}: ERROR -> {1}", api.name, status);
        if api.system_notify == true {
            let msg = format!("Expected: {} Received: {} - ERROR", status, api.expected_status);
            send_notify(
                api.name.as_str(),
                "dialog-error",
                &msg,
            )
            .unwrap();
        }
    }
}

fn start_monitor(configs: &HashMap<String, Api>) {
    // println!("{:?}", configs);
    let mut sched = JobScheduler::new();

    for config in configs {
        println!("Start thread to monitor endpoint: {:?}", config.0);

        sched.add(Job::new(
            config.1.cron_expression.parse().unwrap(),
            move || {
                println!("Verifing {0} ...", config.0);
                verify_api(&config.1);
            },
        ));
    }

    loop {
        sched.tick();

        std::thread::sleep(Duration::from_millis(500));
    }
}

fn load_config(config_path: &str) -> Result<ApisConfig, serde_yaml::Error> {
    println!("Searching for {}", config_path);

    let config_str_content =
        fs::read_to_string(config_path).expect("Something went wrong reading the file");

    let deserialized = serde_yaml::from_str::<ApisConfig>(&config_str_content);

    deserialized
}

fn main() -> std::io::Result<()> {
    println!("Start Application");

    let matches = Command::new("My Test Program")
        .version("0.1.0")
        .author("Elton de Andrade Rodrigues <xxxxxxxxxxxx@xx>")
        .about("Verify API status")
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .takes_value(true)
                .help("config file with APIs"),
        )
        .get_matches();

    let config_path = matches
        .value_of("file")
        .unwrap_or_else(|| panic!("File not set"));

    let config_object = load_config(config_path);

    match config_object {
        Ok(configs) => start_monitor(&configs.requests),
        Err(e) => println!("error parsing: {:?}", e),
    }

    Ok(())
}
