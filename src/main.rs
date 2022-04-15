extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use crate::Value::Null;
use clap::Arg;
use clap::Command;
use notify_rust::Notification;
use reqwest::Method;
use reqwest::Response;
use reqwest::{Client, StatusCode, Url};
use serde_json::Error;
use serde_json::Value;
use std::{fs, thread, time::Duration};

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
enum ConfigMethod {
    GET,
    POST,
    // DELETE,
    // PUT,
    // PATH,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Request {
    url: String,
    headers: Value,
    method: ConfigMethod,
    body: Value,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Config {
    name: String,
    request: Request,
    expected_status: u16,
    time_loop: u64,
    gnome_notify: bool,
    notify_type: String,
    show: bool,
}

fn notify(name: &str, icon_type: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    Notification::new()
        .summary(name)
        .body(message)
        .icon(icon_type)
        .show()?;
    Ok(())
}

#[tokio::main]
async fn api_config(request: &Request) -> Result<StatusCode, Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;

    // Missing all methods
    let method: &'static [u8] = if request.method == ConfigMethod::POST {
        b"POST"
    } else {
        b"GET"
    };

    // TODO
    let resp: Response;
    if request.body != Null {
        resp = client
            .request(Method::from_bytes(method)?, Url::parse(&request.url)?)
            .json(&request.body)
            .send()
            .await?;
    } else {
        resp = client
            .request(Method::from_bytes(method)?, Url::parse(&request.url)?)
            .send()
            .await?;
    };

    Ok(resp.status())
}

fn verify_api(name: &str, request: &Request, expected_status: &u16, notify_type: &str) {
    let response = api_config(request);

    let status = match response {
        Ok(status_code) => status_code.as_u16(),
        Err(e) => {
            println!("{0}: OK -> {1}", name, e);
            notify(
                name,
                "dialog-error",
                "Parece que deu ruim da uma verificada na api",
            )
            .unwrap();
            return ();
        }
    };

    if &status == expected_status {
        println!("{0}: OK -> {1}", name, status);
        if notify_type != "ERROR" {
            notify(name, "dialog-information", "Tudo normal segue o jogo!").unwrap();
        }
    } else {
        println!("{0}: ERROR -> {1}", name, status);
        notify(
            name,
            "dialog-error",
            "Parece que deu ruim da uma verificada na api",
        )
        .unwrap();
    }
}

fn start_monitor(config: Config) -> thread::JoinHandle<()> {
    return thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(config.time_loop));
        println!("Verifing {0} ...", config.name);
        verify_api(
            &config.name,
            &config.request,
            &config.expected_status,
            &config.notify_type,
        );
    });
}

fn load_config(config_path: &str) -> Result<Vec<Config>, Error> {
    println!("Searching for {}", config_path);

    let config_str_content =
        fs::read_to_string(config_path).expect("Something went wrong reading the file");

    let deserialized: Vec<Config> = serde_json::from_str(&config_str_content).unwrap();

    Ok(deserialized)
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
    println!("{:?}", config_object);
    let mut monitor_threads = Vec::new();

    match config_object {
        Ok(configs) => {
            for config in configs.iter() {
                println!("Start thread to monitor endpoint: {0}", config.name);
                let api_thread = start_monitor(config.clone());
                monitor_threads.push(api_thread);
            }

            for handle in monitor_threads {
                thread::sleep(Duration::from_millis(3000));
                handle.join().unwrap()
            }
        }
        Err(e) => println!("error parsing: {:?}", e),
    }

    Ok(())
}
