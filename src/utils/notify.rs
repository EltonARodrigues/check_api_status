use notify_rust::Notification;

pub fn send_notify(
    name: &str,
    icon_type: &str,
    message: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    Notification::new()
        .summary(format!("API: {}", name).as_str())
        .body(message)
        .icon(icon_type)
        .show()?;
    Ok(())
}
