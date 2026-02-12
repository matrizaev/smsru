use std::io;

use smsru::{Auth, CheckCost, CheckCostOptions, MessageText, RawPhoneNumber, SmsRuClient};

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
        .unwrap_or_else(|_| "Hello from the smsru check_cost example.".to_owned());

    let client = SmsRuClient::new(Auth::api_id(api_id)?);
    let phone = RawPhoneNumber::new(phone_raw)?;
    let text = MessageText::new(message)?;
    let request = CheckCost::to_many(vec![phone], text, CheckCostOptions::default())?;

    let response = client.check_cost(request).await?;
    println!(
        "status: {:?}, status_code: {:?}, status_text: {:?}, total_cost: {:?}, total_sms: {:?}, sms: {:?}",
        response.status,
        response.status_code,
        response.status_text,
        response.total_cost,
        response.total_sms,
        response.sms
    );

    Ok(())
}
