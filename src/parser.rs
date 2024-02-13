use chrono::{self, NaiveDateTime};
use std::{fmt, str::FromStr};

mod message_type;
pub use message_type::{MessageType, User};

const PACKET_HEADER: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF];
const MAGIC_NOPASSWORD_BYTE: u8 = 0x52; // R
const MAGIC_PASSWORD_BYTE: u8 = 0x53; // S
const MAGIC_STRING_END: u8 = 0x4C; // L

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogParseError {
    TooShort,
    InvalidHeader,
    BadPasswordByte(u8),
    NoMagicStringEnd,
    BadTimestamp,
}

impl fmt::Display for LogParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // TODO: prettify
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for LogParseError {}

/// Single log line
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogMessage {
    /// The raw timestamp at the start of the line
    pub timestamp: NaiveDateTime,
    /// The raw string message with timestamps and headers removed.
    pub message: String,
    /// If sv_logsecret is set on the server and this log was received over UDP, this will be the received secret
    pub secret: Option<String>,
}

impl FromStr for LogMessage {
    type Err = LogParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        LogMessage::from_bytes(s.as_bytes())
    }
}

impl LogMessage {
    /// Parses a single log line
    pub fn from_bytes(data: &[u8]) -> Result<Self, LogParseError> {
        // parse off the header
        let (header, rest) = match data.iter().position(|&e| e == MAGIC_STRING_END) {
            None => return Err(LogParseError::NoMagicStringEnd),
            Some(idx) => (&data[..idx], &data[(idx + 2)..]),
        };

        let secret: Option<String> = if header.len() > 0 {
            let mut header = header;
            // udp packets start with four 0xFF bytes
            if header.len() > 4 {
                let udp_base = &header[..4];
                if udp_base == PACKET_HEADER {
                    // cut them off
                    header = &header[4..];
                }
            }

            // secret indication byte
            let secret_byte = header[0];
            if secret_byte == MAGIC_PASSWORD_BYTE {
                // has secret, then grab
                Some(String::from_utf8_lossy(&header[1..]).to_string())
            } else if secret_byte == MAGIC_NOPASSWORD_BYTE {
                // no secret
                None
            } else {
                // there is a header, but it's not a password byte, so error
                return Err(LogParseError::BadPasswordByte(secret_byte).into());
            }
        } else {
            // no header = no secret
            None
        };

        // convert rest of header to string for NaiveDateTime's parser
        let message = String::from_utf8_lossy(rest).to_string();
        // strip timestamp
        let (timestamp, rest) =
            NaiveDateTime::parse_and_remainder(&message, "%m/%d/%Y - %H:%M:%S: ")
                .map_err(|_| LogParseError::BadTimestamp)?;

        // get message
        let message = rest[0..rest.len()].to_owned();

        Ok(Self {
            timestamp,
            message,
            secret,
        })
    }

    pub fn parse_message_type(&self) -> MessageType {
        MessageType::from_message(self.message.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_log_line() {
        const LINE: &str = &"L 02/09/2024 - 08:00:50: \"TheirUsername<6><[U:1:1324124512]><>\" connected, address \"192.168.0.1\"";
        let parsed = LogMessage::from_str(LINE).unwrap();
        assert!(
            parsed.message
                == "\"TheirUsername<6><[U:1:1324124512]><>\" connected, address \"192.168.0.1\""
        );
        assert!(parsed.secret.is_none());
    }

    #[test]
    fn no_password() {
        const LINE: &str = &"RL 02/09/2024 - 08:00:50: \"TheirUsername<6><[U:1:1324124512]><>\" connected, address \"192.168.0.1\"";
        let parsed = LogMessage::from_str(LINE).unwrap();
        assert!(
            parsed.message
                == "\"TheirUsername<6><[U:1:1324124512]><>\" connected, address \"192.168.0.1\""
        );
        assert!(parsed.secret.is_none());
    }

    #[test]
    fn with_password() {
        const LINE: &str = &"SnyaL 02/09/2024 - 08:00:50: \"TheirUsername<6><[U:1:1324124512]><>\" connected, address \"192.168.0.1\"";
        let parsed = LogMessage::from_str(LINE).unwrap();
        assert!(
            parsed.message
                == "\"TheirUsername<6><[U:1:1324124512]><>\" connected, address \"192.168.0.1\""
        );
        assert!(parsed.secret.is_some_and(|s| s == "nya"));
    }

    #[test]
    fn magic_bytes_with_password() {
        const LINE: &str = &"SnyaL 02/09/2024 - 08:00:50: \"TheirUsername<6><[U:1:1324124512]><>\" connected, address \"192.168.0.1\"";
        let mut v: Vec<u8> = vec![0xFF, 0xFF, 0xFF, 0xFF];
        v.extend(LINE.bytes());
        let parsed = LogMessage::from_bytes(&v).unwrap();
        assert!(
            parsed.message
                == "\"TheirUsername<6><[U:1:1324124512]><>\" connected, address \"192.168.0.1\""
        );
        assert!(parsed.secret.is_some_and(|s| s == "nya"));
    }

    #[test]
    fn bad_format() {
        const LINE: &str = &"KmeowL 02/09/2024 - 08:00:50: \"TheirUsername<6><[U:1:1324124512]><>\" connected, address \"192.168.0.1\"";
        let parsed = LogMessage::from_str(LINE);
        assert!(parsed.is_err_and(|e| e == LogParseError::BadPasswordByte(75)));
    }

    #[test]
    fn direct_parse() {
        const LINE: &str = &"SmeowL 02/09/2024 - 08:00:50: \"TheirUsername<6><[U:1:1324124512]><>\" connected, address \"192.168.0.1\"";
        let parsed: LogMessage = LINE.parse().unwrap();
        assert!(
            parsed.message
                == "\"TheirUsername<6><[U:1:1324124512]><>\" connected, address \"192.168.0.1\""
        );
        assert!(parsed.secret.is_some_and(|s| s == "meow"));
    }
}
