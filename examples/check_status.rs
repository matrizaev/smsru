use std::io;

use smsru::{Auth, CheckStatus, SmsId, SmsRuClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_id = std::env::var("SMSRU_API_ID").map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "SMSRU_API_ID environment variable is required",
        )
    })?;
    let sms_ids_raw = std::env::var("SMSRU_SMS_IDS").map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "SMSRU_SMS_IDS environment variable is required (comma-separated ids)",
        )
    })?;

    let sms_ids = sms_ids_raw
        .split(',')
        .map(SmsId::new)
        .collect::<Result<Vec<_>, _>>()?;
    let request = CheckStatus::new(sms_ids)?;

    let client = SmsRuClient::new(Auth::api_id(api_id)?);
    let response = client.check_status(request).await?;

    println!(
        "status: {:?}, status_code: {:?}, balance: {:?}, status_text: {:?}, sms: {:?}",
        response.status, response.status_code, response.balance, response.status_text, response.sms
    );

    Ok(())
}
