use std::io;

use smsru::{
    Auth, CallCheckId, CheckCallAuthStatus, CheckCallAuthStatusOptions, RawPhoneNumber,
    SmsRuClient, StartCallAuth, StartCallAuthOptions,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_id = std::env::var("SMSRU_API_ID").map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "SMSRU_API_ID environment variable is required",
        )
    })?;

    let client = SmsRuClient::new(Auth::api_id(api_id)?);

    if let Ok(existing_check_id) = std::env::var("SMSRU_CHECK_ID") {
        let request = CheckCallAuthStatus::new(
            CallCheckId::new(existing_check_id)?,
            CheckCallAuthStatusOptions::default(),
        );
        let response = client.check_call_auth_status(request).await?;
        println!(
            "status: {:?}, status_code: {:?}, check_status: {:?}, check_status_text: {:?}",
            response.status,
            response.status_code,
            response.check_status,
            response.check_status_text
        );
        return Ok(());
    }

    let phone_raw = std::env::var("SMSRU_PHONE").map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "SMSRU_PHONE environment variable is required when SMSRU_CHECK_ID is not set",
        )
    })?;

    let start_request = StartCallAuth::new(
        RawPhoneNumber::new(phone_raw)?,
        StartCallAuthOptions::default(),
    );
    let started = client.start_call_auth(start_request).await?;

    println!(
        "status: {:?}, status_code: {:?}, check_id: {:?}, call_phone: {:?}, call_phone_pretty: {:?}",
        started.status,
        started.status_code,
        started.check_id,
        started.call_phone,
        started.call_phone_pretty
    );

    Ok(())
}
