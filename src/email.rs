use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::env;

#[derive(Debug)]
pub struct Config {
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_address: String,
    pub to_address: String,
    missing_config: bool,
}

impl Config {
    pub fn new() -> Result<Config, &'static str> {
        let mut missing_config = false;

        let from_address = match env::var_os("EMAIL_FROM_ADDRESS") {
            Some(v) => v.into_string().unwrap(),
            None => {
                missing_config = true;
                "".to_string()
            }
        };
        let to_address = match env::var_os("EMAIL_TO_ADDRESS") {
            Some(v) => v.into_string().unwrap(),
            None => {
                missing_config = true;
                "".to_string()
            }
        };
        let smtp_username = match env::var_os("SMTP_USERNAME") {
            Some(v) => v.into_string().unwrap(),
            None => {
                missing_config = true;
                "".to_string()
            }
        };
        let smtp_password = match env::var_os("SMTP_PASSWORD") {
            Some(v) => v.into_string().unwrap(),
            None => {
                missing_config = true;
                "".to_string()
            }
        };

        Ok(Config {
            smtp_username,
            smtp_password,
            from_address,
            to_address,
            missing_config,
        })
    }
}

pub fn send_email(message: String) -> Result<(), String> {
    let config: Config = Config::new().unwrap();
    println!("{:?}", config);
    if config.missing_config {
        println!("Missing email env configs!");
        println!("Email no sended");
    }
    println!("{0}", config.from_address);
    // println!("{:?}", config.from_address.parse().unwrap());
    let email = Message::builder()
        .from(config.from_address.parse().map_err(|err| {
            format!(
                "Could not convert email for 'from' from text '{}'. Error: {}",
                config.from_address, err
            )
        })?)
        .to(config.to_address.parse().map_err(|err| {
            format!(
                "Could not convert email for 'to' from text '{}'. Error: {}",
                config.to_address, err
            )
        })?)
        .subject("ALERT: Api error")
        .body(String::from(message))
        .map_err(|err| format!("Error when creating email: {}", err))?;

    let creds = Credentials::new(config.smtp_username, config.smtp_password);

    // Open a remote connection to gmail
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .unwrap()
        .credentials(creds)
        .build();

    // Send the email
    let result = mailer.send(&email);
    if let Err(err) = result {
        println!("E-mail message was NOT sent successfully.\nError:\n{}", err);
        return Err("Could not send email.".to_owned());
    } else {
        println!("E-mail message was sent successfully.");
    }
    Ok(())
}
