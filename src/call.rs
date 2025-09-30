use crate::{Client, FromMap, TwilioError, GET, POST};
use serde::Deserialize;
use std::collections::BTreeMap;

pub enum CallInstructions<'a> {
    Url(&'a str),
    Twiml(&'a str),
}

pub struct OutboundCall<'a> {
    pub from: &'a str,
    pub to: &'a str,
    pub instructions: CallInstructions<'a>,
}

impl<'a> OutboundCall<'a> {
    pub fn new(from: &'a str, to: &'a str, url: &'a str) -> OutboundCall<'a> {
        OutboundCall {
            from,
            to,
            instructions: CallInstructions::Url(url),
        }
    }

    pub fn new_with_twiml(from: &'a str, to: &'a str, twiml: &'a str) -> OutboundCall<'a> {
        OutboundCall {
            from,
            to,
            instructions: CallInstructions::Twiml(twiml),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CallStatus {
    Queued,
    Ringing,
    InProgress,
    Canceled,
    Completed,
    Failed,
    Busy,
    NoAnswer,
}

#[derive(Debug, Deserialize)]
pub struct Call {
    pub from: String,
    pub to: String,
    pub sid: String,
    pub status: CallStatus,
}

impl Client {
    pub async fn make_call(&self, call: OutboundCall<'_>) -> Result<Call, TwilioError> {
        let mut opts = vec![("To", call.to), ("From", call.from)];

        match &call.instructions {
            CallInstructions::Url(url) => opts.push(("Url", url)),
            CallInstructions::Twiml(twiml) => opts.push(("Twiml", twiml)),
        }

        self.send_request(POST, "Calls", &opts).await
    }

    pub async fn retrieve_call(&self, sid: &str) -> Result<Call, TwilioError> {
        self.send_request(GET, &format!("Calls/{sid}"), &[]).await
    }
}

impl FromMap for Call {
    fn from_map(mut m: BTreeMap<String, String>) -> Result<Box<Call>, TwilioError> {
        let from = match m.remove("From") {
            Some(v) => v,
            None => return Err(TwilioError::ParsingError),
        };
        let to = match m.remove("To") {
            Some(v) => v,
            None => return Err(TwilioError::ParsingError),
        };
        let sid = match m.remove("CallSid") {
            Some(v) => v,
            None => return Err(TwilioError::ParsingError),
        };
        let stat = match m.get("CallStatus").map(|s| s.as_str()) {
            Some("queued") => CallStatus::Queued,
            Some("ringing") => CallStatus::Ringing,
            Some("in-progress") => CallStatus::InProgress,
            Some("canceled") => CallStatus::Canceled,
            Some("completed") => CallStatus::Completed,
            Some("failed") => CallStatus::Failed,
            Some("busy") => CallStatus::Busy,
            Some("no-answer") => CallStatus::NoAnswer,
            _ => return Err(TwilioError::ParsingError),
        };
        Ok(Box::new(Call {
            from,
            to,
            sid,
            status: stat,
        }))
    }
}
