use notify_rust::Notification;

use crate::utils::email::send_email;

pub fn send_notify(
    name: &str,
    icon_type: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    send_email(message.to_string());

    Notification::new()
        .summary(name)
        .body(message)
        .icon(icon_type)
        .show()?;
    Ok(())
}
