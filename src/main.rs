extern crate job_scheduler;
extern crate serde;
extern crate serde_json;
extern crate yaml_rust;

#[macro_use]
extern crate serde_derive;

use crate::Value::Null;
use clap::Arg;
use clap::Command;
use job_scheduler::{Job, JobScheduler};
use notify_rust::Notification;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use reqwest::Method;
use reqwest::Response;
use reqwest::{Client, Url};
use serde_json::Value;
use std::collections::HashMap;
use std::str::FromStr;
use std::{fs, time::Duration};

pub type ReqHash = HashMap<String, String>;

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
enum ConfigMethod {
    GET,
    POST,
    // DELETE,
    // PUT,
    // PATH,
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
struct Request {
    url: String,
    headers: Option<ReqHash>,
    method: ConfigMethod,
    body: Option<ReqHash>,
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
struct Depends {
    name: String,
    header_fields: Vec<String>,
    body_fields: Vec<String>,
    request: Request,
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
struct Api {
    name: String,
    depends_on: Option<Depends>,
    request: Request,
    expected_status: u16,
    cron_expression: String,
    gnome_notify: bool,
    notify_type: String,
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct Data {
    requests: HashMap<String, Api>,
}

fn notify(name: &str, icon_type: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    Notification::new()
        .summary(name)
        .body(message)
        .icon(icon_type)
        .show()?;
    Ok(())
}

async fn api_config(
    request: &Request,
    headers: HeaderMap,
    body: &Option<ReqHash>,
) -> Result<reqwest::Response, Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;

    // Missing all methods
    let method: &'static [u8] = if request.method == ConfigMethod::POST {
        b"POST"
    } else {
        b"GET"
    };

    // TODO
    let resp: Response;

    resp = match body {
        Some(req_body) => {
            client
                .request(Method::from_bytes(method)?, Url::parse(&request.url)?)
                .headers(headers)
                .json(&req_body)
                .send()
                .await?
        }
        None => {
            client
                .request(Method::from_bytes(method)?, Url::parse(&request.url)?)
                .headers(headers)
                .send()
                .await?
        }
    };

    Ok(resp)
}

fn get_field<'a>(value: &'a Value, fields: Vec<&str>, size: usize, start: usize) -> &'a Value {
    let next_value = &value[fields[start]];

    if start == (size - 1) {
        return next_value;
    }

    return get_field(next_value, fields, size, start + 1);
}

async fn get_depends_result(depends: &Depends) -> HashMap<String, Vec<ReqHash>> {
    let mut headers_map = HeaderMap::new();

    for header in &depends.request.headers {
        for (key, value) in header {
            headers_map.insert(
                HeaderName::from_lowercase(key.as_bytes()).unwrap(),
                value.parse().unwrap(),
            );
        }
    }
    let response = api_config(&depends.request, headers_map, &depends.request.body).await;

    let resp_body = match response {
        Ok(r) => r.text().await,
        Err(err) => panic!("Error to request Api: {:?}", err),
    };

    let result: Value = match resp_body {
        Ok(r) => {
            let body_json = serde_json::from_str(&r).unwrap();
            body_json
        }
        Err(e) => panic!("TODO: {:?}", e),
    };

    let mut depends_headers: Vec<ReqHash> = Vec::new();

    for header_field in &depends.header_fields {
        let fields: Vec<_> = header_field.split('.').collect();
        let size = fields.len();
        let result_field = get_field(&result, fields, size, 0);

        let mut fields_values = HashMap::new();
        if result_field != &Null {
            fields_values.insert(header_field.to_string(), result_field.to_string());
        }

        depends_headers.push(fields_values);
    }

    let mut depends_body: Vec<ReqHash> = Vec::new();

    for body_field in &depends.body_fields {
        let fields: Vec<_> = body_field.split('.').collect();
        let size = fields.len();
        let result_field = get_field(&result, fields, size, 0);

        let mut fields_values = HashMap::new();
        if result_field != &Null {
            fields_values.insert(
                body_field.to_string(),
                serde_json::to_string(result_field).unwrap(),
            );
        }

        depends_body.push(fields_values);
    }

    let mut depends_results: HashMap<String, Vec<ReqHash>> = HashMap::new();
    depends_results.insert("depends_body".to_string(), depends_body);
    depends_results.insert("depends_headers".to_string(), depends_headers);

    depends_results
}

#[tokio::main]
async fn verify_api(api: &Api) {
    let mut headers_map = HeaderMap::new();

    println!("{:?}", api.depends_on);

    let fields_required = match &api.depends_on {
        Some(depends) => get_depends_result(&depends).await,
        None => HashMap::new(),
    };
   
    println!("headers main: {:?}", api.request.headers);

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

    let response = api_config(&api.request, headers_map, &body).await;

    let status = match response {
        Ok(resp) => resp.status().as_u16(),
        Err(e) => {
            println!("{0}: OK -> {1}", api.name, e);
            notify(
                api.name.as_str(),
                "dialog-error",
                "Parece que deu ruim da uma verificada na api",
            )
            .unwrap();
            return ();
        }
    };

    println!("received {0}: spected {1}", status, api.expected_status);
    if status == api.expected_status {
        println!("{0}: OK -> {1}", api.name, status);
        if api.notify_type != "ERROR" {
            notify(
                api.name.as_str(),
                "dialog-information",
                "Tudo normal segue o jogo!",
            )
            .unwrap();
        }
    } else {
        println!("{0}: ERROR -> {1}", api.name, status);
        notify(
            api.name.as_str(),
            "dialog-error",
            "Parece que deu ruim da uma verificada na api",
        )
        .unwrap();
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

fn load_config(config_path: &str) -> Result<Data, serde_yaml::Error> {
    println!("Searching for {}", config_path);

    let config_str_content =
        fs::read_to_string(config_path).expect("Something went wrong reading the file");

    let deserialized = serde_yaml::from_str::<Data>(&config_str_content);

    deserialized
}

fn main() -> std::io::Result<()> {
    println!("Start Application");

    let matches = Command::new("My Test Program")
        .version("0.1.0")
        .author("Elton de Andrade Rodrigues <xxxxxxxxxxxx@xx>")
        .about("Teaches argument parsing")
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
