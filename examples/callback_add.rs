use std::io;

use smsru::{AddCallback, Auth, CallbackUrl, SmsRuClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_id = std::env::var("SMSRU_API_ID").map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "SMSRU_API_ID environment variable is required",
        )
    })?;
    let callback_url = std::env::var("SMSRU_CALLBACK_URL").map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "SMSRU_CALLBACK_URL environment variable is required",
        )
    })?;

    let client = SmsRuClient::new(Auth::api_id(api_id)?);
    let request = AddCallback::new(CallbackUrl::new(callback_url)?);
    let response = client.add_callback(request).await?;

    println!(
        "status: {:?}, status_code: {:?}, status_text: {:?}, callback: {:?}",
        response.status, response.status_code, response.status_text, response.callback
    );

    Ok(())
}
