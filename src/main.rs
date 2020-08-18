extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use notify_rust::Notification;
use reqwest::StatusCode;
use serde_json::Error;
use serde_json::Value;
use std::fs::File;
use std::io::prelude::*;
use std::{thread, time::Duration};

const CONFIG_FILE: &str = "config.json";

#[derive(Serialize, Deserialize, Debug)]
struct Request {
    url: String,
    headers: Value,
    body: Value,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    request: Request,
    time_loop: u64,
    show: String,
}

fn load_config(data: std::string::String) -> Result<Vec<Config>, Error> {
    let convert_data = String::from(data);
    let v: Vec<Config> = serde_json::from_str(&convert_data).unwrap();

    Ok(v)
}

#[tokio::main]
async fn api_config(url: &str) -> Result<StatusCode, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?;
    Ok(resp.status())
}

fn notify(icon_type: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    Notification::new()
        .summary("Status do Broker")
        .body(message)
        .icon(icon_type)
        .show()?;
    Ok(())
}

fn verify_api(url: &str) {
    let response = api_config(url);

    let f = match response {
        Ok(file) => file,
        Err(_e) => {
            notify(
                "dialog-error",
                "Parece que deu ruim da uma verificada na api",
            )
            .unwrap();
            return ();
        }
    };
    if f == 200 {
        notify("dialog-information", "Tudo normal segue o jogo!").unwrap();
    } else {
        notify(
            "dialog-error",
            "Parece que deu ruim da uma verificada na api",
        )
        .unwrap();
    }
}

fn start_monitor(config: &Config) -> thread::JoinHandle<()> {
    let url = config.request.url.clone();
    let time_loop = config.time_loop;

    return thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(time_loop));
        verify_api(&url);
    });
}

fn main() -> std::io::Result<()> {
    let mut file = File::open(CONFIG_FILE)?;
    let mut contents = String::new();

    file.read_to_string(&mut contents)?;

    let resp = load_config(contents);

    let mut threads = Vec::new();
    match resp {
        Ok(configs) => {
            for config in configs.iter() {
                let aa = start_monitor(config);
                threads.push(aa);
            }

            for handle in threads {
                handle.join().unwrap()
            }
        }
        Err(e) => println!("error parsing: {:?}", e),
    }

    Ok(())
}
