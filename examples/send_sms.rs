use std::io;

use smsru::{Auth, MessageText, RawPhoneNumber, SendOptions, SendSms, SmsRuClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_id = std::env::var("SMSRU_API_ID").map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "SMSRU_API_ID environment variable is required",
        )
    })?;
    let phone_raw = std::env::var("SMSRU_PHONE").map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "SMSRU_PHONE environment variable is required",
        )
    })?;
    let message = std::env::var("SMSRU_MESSAGE")
        .unwrap_or_else(|_| "Hello from the smsru example.".to_owned());

    let client = SmsRuClient::new(Auth::api_id(api_id)?);
    let phone = RawPhoneNumber::new(phone_raw)?;
    let text = MessageText::new(message)?;
    let request = SendSms::to_many(vec![phone], text, SendOptions::default())?;

    let response = client.send_sms(request).await?;
    println!(
        "status: {:?}, status_code: {:?}, balance: {:?}",
        response.status, response.status_code, response.balance
    );

    Ok(())
}
