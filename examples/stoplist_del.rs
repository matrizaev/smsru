use std::io;

use smsru::{Auth, RawPhoneNumber, RemoveStoplistEntry, SmsRuClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_id = std::env::var("SMSRU_API_ID").map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "SMSRU_API_ID environment variable is required",
        )
    })?;
    let phone = std::env::var("SMSRU_PHONE").map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "SMSRU_PHONE environment variable is required",
        )
    })?;

    let client = SmsRuClient::new(Auth::api_id(api_id)?);
    let request = RemoveStoplistEntry::new(RawPhoneNumber::new(phone)?);
    let response = client.remove_stoplist_entry(request).await?;

    println!(
        "status: {:?}, status_code: {:?}, status_text: {:?}",
        response.status, response.status_code, response.status_text
    );

    Ok(())
}
