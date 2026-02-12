use std::io;

use smsru::{Auth, SmsRuClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_id = std::env::var("SMSRU_API_ID").map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "SMSRU_API_ID environment variable is required",
        )
    })?;

    let client = SmsRuClient::new(Auth::api_id(api_id)?);
    let response = client.get_free_usage().await?;

    println!(
        "status: {:?}, status_code: {:?}, status_text: {:?}, total_free: {:?}, used_today: {:?}",
        response.status,
        response.status_code,
        response.status_text,
        response.total_free,
        response.used_today
    );

    Ok(())
}
