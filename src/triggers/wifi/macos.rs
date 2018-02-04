use std::io;
use std::process::Command;
use std::str;
use std::time::{Duration, Instant};

use futures::prelude::*;
use serde_yaml::Value;
use tokio_core::reactor::{Handle, Timeout};

use triggers::{Activity, Trigger};

const AIRPORT_UTIL_PATH: &'static str = "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport";

/// A wifi evidence source that signals when a specific wifi network
/// is joined or left.
#[derive(Debug)]
pub struct WifiTrigger(String);

impl WifiTrigger {
    pub fn new<N: Into<String>>(wifi_name: N) -> Self {
        WifiTrigger(wifi_name.into())
    }

    pub fn from_config(cfg: &Value) -> io::Result<Self> {
        match *cfg {
            Value::String(ref ssid) => Ok(Self::new(ssid.as_ref())),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown configuration format")),
        }
    }
}

impl Trigger for WifiTrigger {
    fn listen(&mut self, handle: Handle) -> Box<Stream<Item = Activity, Error = io::Error>> {
        Box::new(WifiStream::new(self.0.clone(), handle))
    }
}

#[derive(Debug)]
struct WifiStream {
    name: String,
    timeout: Timeout,
    was_same: bool,
}

impl WifiStream {
    pub fn new(name: String, handle: Handle) -> Self {
        WifiStream {
            name,
            timeout: Timeout::new(Duration::from_millis(0), &handle).unwrap(),
            was_same: false,
        }
    }

    /// Parse wifi SSID out of airport utility's output.
    ///
    /// Returns `None` if wifi is turned off and the SSID otherwise.
    fn get_wifi_name() -> io::Result<Option<String>> {
        let output = Command::new(AIRPORT_UTIL_PATH)
            .arg("-I")
            .output()?;

        let mut line_parts = str::from_utf8(&output.stdout)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Got non-UTF-8 output from airport utility"))?
            .lines()
            .map(|l| l.trim())
            .filter(|l| l.starts_with("SSID: ") || l.starts_with("AirPort: "))
            .nth(0)
            .ok_or(io::Error::new(io::ErrorKind::InvalidData, "Missing SSID or AirPort line"))?
            .splitn(2, ": ");

        // If this is "AirPort", than we have the line "AirPort: Off", which
        // signals that wifi is turned off.
        match line_parts.next() {
            Some("AirPort") | None => return Ok(None),
            _ => {}
        }

        Ok(line_parts.next().map(Into::into))
    }
}

impl Stream for WifiStream {
    type Item = Activity;
    type Error = io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        try_ready!(self.timeout.poll());
        self.timeout.reset(Instant::now() + Duration::from_millis(5000));

        let new_ssid = Self::get_wifi_name()?;
        match new_ssid {
            Some(ssid) => {
                let is_same = ssid == self.name;

                if is_same && !self.was_same {
                    self.was_same = true;
                    Ok(Async::Ready(Some(Activity::Active)))
                } else if !is_same && self.was_same {
                    self.was_same = false;
                    Ok(Async::Ready(Some(Activity::Inactive)))
                } else {
                    try_ready!(self.timeout.poll());

                    Ok(Async::NotReady)
                }
            },
            None => {
                if self.was_same {
                    self.was_same = false;
                    Ok(Async::Ready(Some(Activity::Inactive)))
                } else {
                    try_ready!(self.timeout.poll());

                    Ok(Async::NotReady)
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wifi_name() {
        get_wifi_name().unwrap();
    }
}
