use std::io;

use smsru::{AddStoplistEntry, Auth, RawPhoneNumber, SmsRuClient, StoplistText};

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
    let note =
        std::env::var("SMSRU_STOPLIST_TEXT").unwrap_or_else(|_| "added from example".to_owned());

    let client = SmsRuClient::new(Auth::api_id(api_id)?);
    let request = AddStoplistEntry::new(RawPhoneNumber::new(phone)?, StoplistText::new(note)?);
    let response = client.add_stoplist_entry(request).await?;

    println!(
        "status: {:?}, status_code: {:?}, status_text: {:?}",
        response.status, response.status_code, response.status_text
    );

    Ok(())
}
