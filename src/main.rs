extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;

use clap::Arg;
use clap::Command;
use notify_rust::Notification;
use reqwest::StatusCode;
use serde_json::Error;
use serde_json::Value;
use std::fs;
use std::{env, fs::File, io::prelude::*, thread, time::Duration};

const CONFIG_FILE: &str = "config.json";

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Request {
    url: String,
    headers: Value,
    body: Value,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Config {
    name: String,
    request: Request,
    time_loop: u64,
    gnome_notify: bool,
    notify_type: String,
    show: bool,
}

#[tokio::main]
async fn api_config(url: &str) -> Result<StatusCode, Box<dyn std::error::Error>> {
    let resp = reqwest::get(url).await?;
    Ok(resp.status())
}

fn notify(name: &str, icon_type: &str, message: &str) -> Result<(), Box<dyn std::error::Error>> {
    Notification::new()
        .summary(name)
        .body(message)
        .icon(icon_type)
        .show()?;
    Ok(())
}

fn verify_api(name: &str, url: &str, notify_type: &str) {
    let response = api_config(url);

    let f = match response {
        Ok(file) => file,
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
    if f == 200 {
        println!("{0}: OK -> {1}", name, f);
        if notify_type != "ERROR" {
            notify(name, "dialog-information", "Tudo normal segue o jogo!").unwrap();
        }
    } else {
        println!("{0}: ERROR -> {1}", name, f);
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
        verify_api(&config.name, &config.request.url, &config.notify_type);
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

    let mut monitor_threads = Vec::new();

    match config_object {
        Ok(configs) => {
            for config in configs.iter() {
                println!("Start thread to monitor endpoint: {0}", config.name);
                let aa = start_monitor(config.clone());
                monitor_threads.push(aa);
            }

            for handle in monitor_threads {
                thread::sleep(Duration::from_millis(3000));
                handle.join().unwrap()
            }
        }
        Err(e) => println!("error parsing: {:?}", e),
    }

    // println!("{:?}", resp);
    // println!("In file {}", filename);
    // let mut file = File::open(CONFIG_FILE)?;
    // let mut contents = String::new();

    // file.read_to_string(&mut contents)?;

    // println!("Load config file");
    // let resp = load_config(contents);

    // let mut threads = Vec::new();
    // match resp {
    //     Ok(configs) => {
    //         for config in configs.iter() {
    //             println!("Start thread to monitor endpoint: {0}", config.name);
    //             let aa = start_monitor(config);
    //             threads.push(aa);
    //         }

    //         for handle in threads {
    //             thread::sleep(Duration::from_millis(3000));
    //             handle.join().unwrap()
    //         }
    //     }
    //     Err(e) => println!("error parsing: {:?}", e),
    // }

    Ok(())
}
