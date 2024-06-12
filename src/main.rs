mod request;
mod utils;

use app::{ApiInformation, ListRequests};
use request::{get_depends_result, request_api};

use utils::notify::send_notify;
use utils::yarn::{Api, ApisConfig, ReqHash};

use clap::Arg;
use clap::Command;
use futures::executor::block_on;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderName;
use serde_json::Value;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;
use std::str::FromStr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::{error::Error, io};
use tokio::task;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

mod app;
mod ui;
use crate::{app::App, ui::ui};

async fn verify_api(api: &Api) -> ApiInformation {
    let mut headers_map = HeaderMap::new();

    let fields_required = match &api.depends_on {
        Some(depends) => block_on(get_depends_result(&depends, api.system_notify)),
        None => HashMap::new(),
    };

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
        Err(_) => 500,
    };

    let mut request_data = ApiInformation {
        name: api.name.to_string(),
        url: api.request.url.to_string(),
        method: api.request.method.to_string(),
        status: "WAINTING".to_string(),
    };

    if status == api.expected_status {
        if api.notify_type != "ERROR" {
            request_data.status = "OK".to_string();
        }
    } else {
        request_data.status = "ERROR".to_string();
        if api.system_notify == true {
            let notify_message = format!("Request failed with status {status}");
            send_notify(api.name.as_str(), "dialog-error", notify_message.as_str()).unwrap();
        }
    }

    request_data
}

fn load_config(config_path: &str) -> Result<ApisConfig, serde_yml::Error> {
    println!("Searching for {}", config_path);

    let config_str_content =
        fs::read_to_string(config_path).expect("Something went wrong reading the file");

    let deserialized = serde_yml::from_str::<ApisConfig>(&config_str_content);

    deserialized
}

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    configs: &HashMap<String, Api>,
) -> io::Result<bool> {
    let mut handles: HashMap<usize, task::JoinHandle<()>> = HashMap::new();

    let results = Arc::new(Mutex::new(app.apis_infos.clone()));
    let running = Arc::new(AtomicBool::new(true));

    loop {
        terminal.draw(|f| ui(f, app))?;

        for (id, config) in configs.iter().enumerate() {
            if handles.contains_key(&id) {
                if handles[&id].is_finished() {
                    handles.remove(&id);
                }
            }

            if !handles.contains_key(&id) {
                let results = Arc::clone(&results);
                let api_config = config.1.clone();
                let running = running.clone();

                let handle = task::spawn(async move {
                    let mut counter = api_config.interval;
                    while running.load(Ordering::SeqCst) {
                        thread::sleep(Duration::from_secs(1));
                        counter -= 1;

                        if counter < 1 {
                            let status_api: app::ApiInformation = verify_api(&api_config).await;

                            let mut results = results.lock().unwrap();
                            let new_request = ListRequests {
                                id: id.try_into().unwrap(),
                                data: status_api,
                                interval: api_config.interval,
                            };
                            if let Some(status) = results.iter_mut().find(|r| r.id == id) {
                                *status = new_request;
                            } else {
                                results.push(new_request);
                            }
                            break;
                        }

                        let mut results = results.lock().unwrap();
                        if let Some(status) = results.iter_mut().find(|r| r.id == id) {
                            status.interval = counter;
                        }
                    }
                });
                handles.insert(id, handle);
            }
        }

        let results = results.lock().unwrap();
        app.append_status(results.clone());

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Release {
                    // Skip events that are not KeyEventKind::Press
                    continue;
                }
                match key.code {
                    KeyCode::Char('q') => {
                        running.store(false, Ordering::SeqCst);
                        return Ok(true);
                    }
                    _ => {}
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cmd_matches = Command::new("Health Crab TUI")
        .version("0.1.0")
        .author("Elton de Andrade Rodrigues <xxxxxxxxxxxx@xx>")
        .about("Verify API status")
        .arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .help("config file with APIs"),
        )
        .get_matches();

    let config_path = cmd_matches
        .get_one::<String>("file")
        .unwrap_or_else(|| panic!("File not set"));

    let config_object = load_config(config_path);

    enable_raw_mode()?;
    let mut stdout: io::Stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    match config_object {
        Ok(configs) => {
            let mut app = App::new(configs.clone());
            app.format_api_infos();

            let _ = run_app(&mut terminal, &mut app, &configs.requests).await;

            disable_raw_mode()?;
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                DisableMouseCapture
            )?;
            terminal.show_cursor()?;
        }
        Err(e) => println!("error parsing: {:?}", e),
    }

    Ok(())
}
