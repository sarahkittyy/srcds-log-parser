use super::{MessageType, User};
use nom::{branch::Alt, Err};
use regex::Regex;

#[allow(unused_imports)]
use nom::{
    bytes::complete::{tag, tag_no_case, take_until, take_until1, take_while, take_while1},
    character::{
        complete::{alpha0, char, digit1},
        is_space,
    },
    combinator::fail,
    error,
    multi::{many0_count, many1},
    sequence::{delimited, preceded, Tuple},
    IResult, Parser,
};
use std::net::Ipv4Addr;

pub fn get_message_type(i: &str) -> IResult<&str, MessageType> {
    log_file_started
        .or(log_file_closed)
        .or(server_cvars_start)
        .or(server_cvars_end)
        .or(loading_map)
        .or(starting_map)
        .or(rcon)
        .or(chat_message)
        .or(connect_message)
        .or(disconnect_message)
        .or(inter_player_action)
        .or(join_team_msg)
        .parse(i)
}

pub fn rcon(i: &str) -> IResult<&str, MessageType> {
    let (i, _) = tag_no_case("rcon from ").parse(i)?;
    let (i, (ip, port)) = delimited(char('"'), ipv4_with_port, char('"'))(i)?;
    let (i, _) = tag(": command ")(i)?;
    let (i, command) = delimited(char('"'), take_until1("\""), char('"'))(i)?;
    Ok((
        i,
        MessageType::Rcon {
            ip,
            port,
            command: command.to_owned(),
        },
    ))
}

pub fn log_file_closed(i: &str) -> IResult<&str, MessageType> {
    let _ = tag_no_case("log file closed")(i)?;
    Ok((i, MessageType::LogFileClosed))
}

pub fn server_cvars_start(i: &str) -> IResult<&str, MessageType> {
    let _ = tag_no_case("server cvars start")(i)?;
    Ok((i, MessageType::ServerCvarsStart))
}

pub fn server_cvars_end(i: &str) -> IResult<&str, MessageType> {
    let _ = tag_no_case("server cvars end")(i)?;
    Ok((i, MessageType::ServerCvarsEnd))
}

pub fn loading_map(i: &str) -> IResult<&str, MessageType> {
    let (i, _) = tag("loading map ")(i)?;
    let (i, name) = delimited(char('"'), take_until1("\""), char('"'))(i)?;
    Ok((
        i,
        MessageType::LoadingMap {
            name: name.to_owned(),
        },
    ))
}

pub fn starting_map(i: &str) -> IResult<&str, MessageType> {
    let (i, _) = tag_no_case("started map ")(i)?;
    let (i, name) = delimited(char('"'), take_until1("\""), char('"'))(i)?;
    let (i, _) = take_while(char::is_whitespace)(i)?;
    let (i, (_, crc)) = kv_pair(i)?;
    Ok((
        i,
        MessageType::StartedMap {
            name: name.to_owned(),
            crc: crc.to_owned(),
        },
    ))
}

pub fn log_file_started(i: &str) -> IResult<&str, MessageType> {
    let (i, _) = tag_no_case("log file started ")(i)?;
    let (i, (_, file)) = kv_pair(i)?;
    let (i, _) = take_while1(char::is_whitespace)(i)?;
    let (i, (_, game)) = kv_pair(i)?;
    let (i, _) = take_while1(char::is_whitespace)(i)?;
    let (i, (_, version)) = kv_pair(i)?;

    Ok((
        i,
        MessageType::LogFileStarted {
            file: file.to_owned(),
            game: game.to_owned(),
            version: version.to_owned(),
        },
    ))
}

pub fn kv_pair<'a>(i: &'a str) -> IResult<&'a str, (&'a str, &'a str)> {
    delimited(
        char('('),
        |i: &'a str| {
            (
                take_until(" "),
                preceded(char(' '), delimited(char('"'), take_until("\""), char('"'))),
            )
                .parse(i)
        },
        char(')'),
    )
    .parse(i)
}

pub fn join_team_msg(i: &str) -> IResult<&str, MessageType> {
    let (i, user) = user(i)?;
    let (i, _) = tag(" joined team ")(i)?;
    let (i, team) = delimited(char('"'), take_until1("\""), char('"'))(i)?;
    Ok((
        i,
        MessageType::JoinedTeam {
            user,
            team: team.to_owned(),
        },
    ))
}

pub fn inter_player_action(i: &str) -> IResult<&str, MessageType> {
    let (i, from) = user(i)?;
    let (i, _) = tag_no_case(" triggered ")(i)?;
    let (i, action) = delimited(char('"'), take_until1("\""), char('"'))(i)?;
    let (i, _) = tag_no_case(" against ")(i)?;
    let (i, against) = user(i)?;

    Ok((
        i,
        MessageType::InterPlayerAction {
            from,
            action: action.to_owned(),
            against,
        },
    ))
}

pub fn ipv4_with_port(i: &str) -> IResult<&str, (Ipv4Addr, u16)> {
    let (i, ip) = ipv4(i)?;
    let (i, _) = char(':')(i)?;
    let (i, port) = port(i)?;
    Ok((i, (ip, port)))
}

pub fn port(i: &str) -> IResult<&str, u16> {
    let (i, port) = digit1(i)?;
    if let Ok(port) = port.parse::<u16>() {
        Ok((i, port))
    } else {
        fail(i)
    }
}

pub fn ipv4(i: &str) -> IResult<&str, Ipv4Addr> {
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

pub fn user(i: &str) -> IResult<&str, User> {
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

pub fn disconnect_message(i: &str) -> IResult<&str, MessageType> {
    let (i, user) = user(i)?;
    let (i, _) = tag(" disconnected (reason ")(i)?;
    let (i, reason) = delimited(char('"'), take_until1("\""), tag("\")")).parse(i)?;
    Ok((
        i,
        MessageType::Disconnected {
            user,
            reason: reason.to_owned(),
        },
    ))
}

pub fn connect_message(i: &str) -> IResult<&str, MessageType> {
    let (i, user) = user(i)?;
    let (i, _) = tag(" connected, address ")(i)?;
    let (i, (ip, port)) = delimited(char('"'), ipv4_with_port, char('"')).parse(i)?;
    Ok((i, MessageType::Connected { user, ip, port }))
}

pub fn chat_message(i: &str) -> IResult<&str, MessageType> {
    let (i, user) = user(i)?;
    let (i, say) = (tag(" say "), tag(" say_team ")).choice(i)?;
    let (i, message) = delimited(char('"'), take_until1("\""), char('"'))(i)?;

    Ok((
        i,
        MessageType::ChatMessage {
            from: user,
            message: message.to_owned(),
            team: say == " say_team ",
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connect_message() {
        const LINE: &str =
            "\"User<1><[U:1:123456789]><>\" connected, address \"192.168.0.1:27005\"";
        let parsed = get_message_type(LINE).unwrap();
        dbg!(&parsed);
        match parsed.1 {
            MessageType::Connected { .. } => (),
            _ => panic!("grr"),
        }
    }

    #[test]
    fn test_ipv4() {
        const IP: &str = "192.168.0.225";
        let ip: Ipv4Addr = ipv4(IP).unwrap().1;
        dbg!(&ip);
        assert!(ip.to_string() == "192.168.0.225");
    }

    #[test]
    fn test_ipv4_with_port() {
        const IP: &str = "192.168.0.115:12345";
        let (ip, port) = ipv4_with_port(IP).unwrap().1;
        dbg!(&ip);
        assert!(ip.to_string() == "192.168.0.115");
        assert!(port == 12345);
    }

    #[test]
    fn start_map() {
        const LINE: &str =
            "Started map \"koth_highpass\" (CRC \"505b4fbf2a1661d2fb1b96f444ef268c\")";
        let parsed = get_message_type(LINE).unwrap();
        assert!(
            parsed.1
                == MessageType::StartedMap {
                    name: "koth_highpass".to_owned(),
                    crc: "505b4fbf2a1661d2fb1b96f444ef268c".to_owned()
                }
        );
    }
}
