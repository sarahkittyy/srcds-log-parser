use std::net::Ipv4Addr;

mod parsers;
use parsers::*;

/// https://developer.valvesoftware.com/wiki/HL_Log_Standard#Appendix_B_-_Example_Log_Files
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MessageType {
    LogFileStarted {
        file: String,
        game: String,
        version: String,
    },
    LogFileClosed,
    ServerCvarsStart,
    ServerCvar {
        var: String,
        value: String,
    },
    ServerCvarsEnd,
    LoadingMap {
        name: String,
    },
    StartedMap {
        name: String,
        crc: String,
    },
    Rcon {
        ip: Ipv4Addr,
        port: u16,
        command: String,
    },
    ChatMessage {
        from: User,
        message: String,
        team: bool,
    },
    Connected {
        user: User,
        ip: Ipv4Addr,
        port: u16,
    },
    Disconnected {
        user: User,
        reason: String,
    },
    JoinedTeam {
        user: User,
        team: String,
    },
    InterPlayerAction {
        from: User,
        action: String,
        against: User,
    },
    Unknown,
}

/// A source user's data
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct User {
    pub name: String,
    pub uid: u32,
    pub steamid: String,
    pub team: String,
}

impl MessageType {
    pub fn from_message<'a>(msg: impl Into<&'a str>) -> Self {
        match get_message_type(msg.into()) {
            Ok((_, m)) => m,
            Err(_) => MessageType::Unknown,
        }
    }

    pub fn is_unknown(&self) -> bool {
        match self {
            Self::Unknown => true,
            _ => false,
        }
    }
}
