use regex::Regex;
use std::{net::Ipv4Addr, str::FromStr};

use nom::{
    bytes::complete::{tag, take_until1},
    character::complete::{char, digit1},
    combinator::fail,
    sequence::Tuple,
    Err, IResult, Parser,
};

/// https://developer.valvesoftware.com/wiki/HL_Log_Standard#Appendix_B_-_Example_Log_Files
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MessageType {
    ChatMessage { from: User, message: String },
    Connected { user: User, ip: Ipv4Addr, port: u16 },
    Disconnected { user: User, reason: String },
    JoinedTeam { user: User, team: String },
    StartedMap(String),
    Domination { from: User, to: User },
    Revenge { from: User, to: User },
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

fn get_message_type(i: &str) -> IResult<&str, MessageType> {
    chat_message
        .or(connect_message)
        .or(disconnect_message)
        .or(start_map_message)
        .or(vengeance_message)
        .or(join_team_msg)
        .parse(i)
}

fn join_team_msg(i: &str) -> IResult<&str, MessageType> {
    let (i, user) = user(i)?;
    let (i, _) = tag(" joined team ")(i)?;
    let (i, (_, team, _)) = (char('"'), take_until1("\""), char('"')).parse(i)?;
    Ok((
        i,
        MessageType::JoinedTeam {
            user,
            team: team.to_owned(),
        },
    ))
}

fn vengeance_message(i: &str) -> IResult<&str, MessageType> {
    let (i, from) = user(i)?;
    if let Ok((i2, (_, to))) = (tag(" triggered \"domination\" against "), user).parse(i) {
        Ok((i2, MessageType::Domination { from, to }))
    } else if let Ok((i2, (_, to))) = (tag(" triggered \"revenge\" against "), user).parse(i) {
        Ok((i2, MessageType::Revenge { from, to }))
    } else {
        fail(i)
    }
}

fn start_map_message(i: &str) -> IResult<&str, MessageType> {
    let (i, _) = tag("Started map ")(i)?;
    let (i, (_, map, _)) = (char('"'), take_until1("\""), char('"')).parse(i)?;
    Ok((i, MessageType::StartedMap(map.to_owned())))
}

fn ipv4(i: &str) -> IResult<&str, Ipv4Addr> {
    let (i, (a, _, b, _, c, _, d)) = (
        digit1,
        char('.'),
        digit1,
        char('.'),
        digit1,
        char('.'),
        digit1,
    )
        .parse(i)?;

    Ok((
        i,
        Ipv4Addr::new(
            a.parse().unwrap(),
            b.parse().unwrap(),
            c.parse().unwrap(),
            d.parse().unwrap(),
        ),
    ))
}

fn user(i: &str) -> IResult<&str, User> {
    let re = Regex::new(r#""(.*?)<(\d+)><(\[U:\d:\d+\])><(\w*)?>""#).unwrap();
    let Some(caps) = re.captures(i) else {
        return Err(Err::Error(nom::error::Error::new(
            i,
            nom::error::ErrorKind::Tag,
        )));
    };

    let len = caps.get(0).unwrap().len();
    let name = caps.get(1).unwrap().as_str();
    let uid = caps.get(2).unwrap().as_str();
    let steamid = caps.get(3).unwrap().as_str();
    let team = caps.get(4).unwrap().as_str();

    Ok((
        &i[len..],
        User {
            name: name.to_owned(),
            uid: uid.parse().unwrap(),
            steamid: steamid.to_owned(),
            team: team.to_owned(),
        },
    ))
}

fn disconnect_message(i: &str) -> IResult<&str, MessageType> {
    let (i, user) = user(i)?;
    let (i, _) = tag(" disconnected (reason ")(i)?;
    let (i, (_, reason, _)) = (char('"'), take_until1("\""), tag("\")")).parse(i)?;
    Ok((
        i,
        MessageType::Disconnected {
            user,
            reason: reason.to_owned(),
        },
    ))
}

fn connect_message(i: &str) -> IResult<&str, MessageType> {
    let (i, user) = user(i)?;
    let (i, _) = tag(" connected, address ")(i)?;
    let (i, (_, ip, _)) = (char('"'), ipv4, char(':')).parse(i)?;
    let (i, port) = digit1(i)?;
    Ok((
        i,
        MessageType::Connected {
            user,
            ip,
            port: port.parse().unwrap(),
        },
    ))
}

fn chat_message(i: &str) -> IResult<&str, MessageType> {
    let (i, user) = user(i)?;
    let (i, _say) = tag(" say ")(i)?;
    let (i, (_, message, _)) = (char('"'), take_until1("\""), char('"')).parse(i)?;

    Ok((
        i,
        MessageType::ChatMessage {
            from: user,
            message: message.to_owned(),
        },
    ))
}
