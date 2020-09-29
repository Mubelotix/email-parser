use nom::IResult;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::sequence::tuple;
use nom::combinator::opt;
use nom::multi::many0;

fn is_ftext(character: u8) -> bool {
    character >= 33 && character <= 57 ||
    character >= 59 && character <= 126
}

fn is_text(character: u8) -> bool {
    character >= 1 && character <= 9 ||
    character >= 14 && character <= 127 ||
    character == 11 ||
    character == 12
}

fn unstructured_header_value(data: &[u8]) -> IResult<&[u8], Vec<u8>> {
    #[derive(PartialEq)]
    enum State {
        CarriageReturn,
        LineFeed,
        AnythingButWhitespace,
    }

    let mut state = State::CarriageReturn;
    for (mut idx, character) in data.iter().enumerate() {
        match state {
            State::CarriageReturn => if *character == b'\r' {
                state = State::LineFeed
            },
            State::LineFeed => if *character == b'\n' {
                state = State::AnythingButWhitespace
            } else {
                state = State::CarriageReturn
            },
            State::AnythingButWhitespace => if *character != b' ' && *character != b'\t' {
                idx -= 2;
                return Ok((&data[idx..], (&data[..idx]).to_vec()))
            } else {
                state = State::CarriageReturn
            }
        }
    }

    if state == State::AnythingButWhitespace {
        Ok((&data[data.len() - 2..], (&data[..data.len() - 2]).to_vec()))
    } else {
        Err(nom::Err::Incomplete(nom::Needed::Size(1)))
    }
}

fn field_name(data: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(is_ftext)(data)
}

fn header_separator(data: &[u8]) -> IResult<&[u8], &[u8]> {
    let mut idx = 0;
    while idx < data.len() && (data[idx] == b' ' || data[idx] == b'\t') {
        idx += 1;
    }
    if idx < data.len() && data[idx] == b':' {
        idx += 1;
        Ok((&data[idx..], &data[..idx]))
    } else {
        Err(nom::Err::Failure((&data[idx..],nom::error::ErrorKind::CrLf)))
    }
}

fn optionnal_field(data: &[u8]) -> IResult<&[u8], (&[u8], Vec<u8>)> {
    let result = tuple((field_name, header_separator, unstructured_header_value, tag("\r\n")))(data)?;
    Ok((result.0, ((result.1).0, (result.1).2)))
}
fn optionnal_field_with_separator(data: &[u8]) -> IResult<&[u8], (&[u8], &[u8], Vec<u8>)> {
    let result = tuple((field_name, header_separator, unstructured_header_value, tag("\r\n")))(data)?;
    Ok((result.0, ((result.1).0, (result.1).1, (result.1).2)))
}

fn fields(data: &[u8]) -> IResult<&[u8], Vec<(&[u8], Vec<u8>)>> {
    many0(optionnal_field)(data) // TODO obs-fields
}
fn fields_with_separators(data: &[u8]) -> IResult<&[u8], Vec<(&[u8], &[u8], Vec<u8>)>> {
    many0(optionnal_field_with_separator)(data) // TODO obs-fields
}

fn body(data: &[u8]) -> IResult<&[u8], &[u8]> {
    let mut idx = 0;
    let mut line_lenght = 0;
    while idx < data.len() {
        if is_text(data[idx]) && line_lenght < 998 {
            idx += 1;
            line_lenght += 1
        } else if idx + 1 < data.len() && data[idx] == b'\r' && data[idx + 1] == b'\n' {
            idx += 2;
            line_lenght = 0;
        } else {
            return Err(nom::Err::Failure((&data[idx..],nom::error::ErrorKind::CrLf))); // todo
        }
    }
    Ok((&data[idx..], &data[..idx]))
}

/// Parse a raw email into a vec of headers and a body.
/// Keeps header values as they are in the mail (no unfolding).
pub fn parse_message(data: &[u8]) -> Result<(Vec<(&[u8], Vec<u8>)>, Option<&[u8]>), ()> {
    match tuple((fields, opt(tuple((tag("\r\n"), body)))))(data) {
        Ok(result) => Ok(((result.1).0, (result.1).1.map(|b| b.1))),
        Err(_e) => {
            Err(())
        }
    }
}

/// Like `parse_message()` but headers are a tuple of three values: (name, separator, value).  
///   
/// Example: `"HeaderName  : header value\r\n"` gives `("HeaderName", "  :", "header value")`
pub fn parse_message_with_separators(data: &[u8]) -> Result<(Vec<(&[u8], &[u8], Vec<u8>)>, Option<&[u8]>), ()> {
    if let Ok(result) = tuple((fields_with_separators, opt(tuple((tag("\r\n"), body)))))(data) {
        Ok(((result.1).0, (result.1).1.map(|b| b.1)))
    } else {
        Err(())
    }
}

#[cfg(test)]
mod test {
    use super::{parse_message, header_separator};

    #[test]
    fn main_test() {
        use super::parse_message;

        let mail = "Received: by mail-oi1-f177.google.com with SMTP id e4so8660662oib.1\r\n        for <mubelotix@mubelotix.dev>; Tue, 30 Jun 2020 01:43:28 -0700 (PDT)\r\nDKIM-Signature: v=1; a=rsa-sha256; c=relaxed/relaxed;\r\n        d=gmail.com; s=20161025;\r\n        h=mime-version:from:date:message-id:subject:to;\r\n        bh=5NNwu8gdOD3ZoZD58FM4gy7PeYn+BudAJmLL+5Moe58=;\r\n        b=aTCNiDUsK2fSxrWf9zWJo03xIpgkFIaV6v/OpcIgEtysxN26K/UR6BofP2KL24DSZl\r\n         FfQLpoWmD0GyU9sN294CUtcYW9xZR5LkQCicxFos/qHOYIaYn/BTwApvyAwdio1OYMM4\r\n         EYJybljPidGHVRaVcLqKfjy0U7HdjHMzm4rTIsvzn7nVm1ziWaZKS0O8QSAMXyXVTkPH\r\n         cIIHa2e1fc76ZLCFLtcI+e/SszpBwVqnvNgWYWBYiGFvjC4CCGJouGxb9z58rzA03XhW\r\n         Ix0uR2YeRYxugTVP/tAf5mo34KWjwKr98IbmYs8nDrZZSliCiyV8B7bWHXM3qXyepXnl\r\n         CQOA==\r\nX-Google-DKIM-Signature: v=1; a=rsa-sha256; c=relaxed/relaxed;\r\n        d=1e100.net; s=20161025;\r\n        h=x-gm-message-state:mime-version:from:date:message-id:subject:to;\r\n        bh=5NNwu8gdOD3ZoZD58FM4gy7PeYn+BudAJmLL+5Moe58=;\r\n        b=SEIL6qJEGH/+sVou4i84kC4vEEsLShYrmKLAlM/7V1fIIbpyQWDRpehMKnlGFKmTCx\r\n         Mz1NijW6tbjDJ+1eF3aE/MNSzhim2eO4JmcK5kZ4vlZzzPWE+GacZqc3QNtAufgA/EqP\r\n         eWTuFSPtSY2vHJdRX21vq8WpP31KdG0JKcv3ZykDqH0y1dAM1sAGR3Gmrcyu+HGA9Ug5\r\n         BrYx1ZPyjYOtlXEiGqaKRsrBlB5P42n2aU0TwZYrEVi9N5TULM4bS+bLtP3FmxP7uIP2\r\n         ZKuFKbcTTveG3+DaaOE7HK/dHXWXZZC9RaS/yzGettgXiwmaAENcONpTwg1jD70DU5a9\r\n         DYHg==\r\nX-Gm-Message-State: AOAM533sOvLV7q5oj9SIWatwQ3kCiOgSZHBhJb0R93ImzSZav4QObpV2\r\n        pLSheyz34dtdedvMg8G3go4HsIP3ytqkN8f9j+ZTvFkx\r\nX-Google-Smtp-Source: ABdhPJzLJRsIQigY2u6fwn04UxksGTqbklM5igDK5fVI2kljDUPeTOPWxkM4IEUQpRb6Ciacz58Kj9Dqy61/LiiyDyA=\r\nX-Received: by 2002:aca:d681:: with SMTP id n123mr15403808oig.82.1593506599851;\r\n Tue, 30 Jun 2020 01:43:19 -0700 (PDT)\r\nMIME-Version: 1.0\r\nFrom: Mubelotix <mubelotix@gmail.com>\r\nDate: Tue, 30 Jun 2020 10:43:08 +0200\r\nMessage-ID: <CANc=2UXAvRBx-A7SP9JWm=pby29s_zdFvfMDUprZ+PN_8XuO+w@mail.gmail.com>\r\nSubject: Test email\r\nTo: mubelotix@mubelotix.dev\r\nContent-Type: multipart/alternative; boundary=\"000000000000d4d95805a9492a3c\"\r\n\r\n--000000000000d4d95805a9492a3c\r\nContent-Type: text/plain; charset=\"UTF-8\"\r\n\r\nTest body\r\n\r\n--000000000000d4d95805a9492a3c\r\nContent-Type: text/html; charset=\"UTF-8\"\r\n\r\n<div dir=\"ltr\">Test body</div>\r\n\r\n--000000000000d4d95805a9492a3c--";

        let (headers, body) = parse_message(mail.as_bytes()).unwrap();
        for (name, value) in headers {
            println!("{}:{}", std::str::from_utf8(name).unwrap(), std::str::from_utf8(&value).unwrap())
        }
        println!("{}", std::str::from_utf8(body.unwrap()).unwrap())
    }

    #[test]
    fn secondary_test() {
        let mail = b"A: X\r\nB : Y\t\r\n\tZ  \r\n\r\n C \r\nD \t E\r\n\r\n\r\n";
        
        let (headers, _body) = parse_message(mail).unwrap();
        
        assert_eq!(headers.len(), 2);
    }

    #[test]
    fn test_separator() {
        header_separator(b"   :  ").unwrap();
        header_separator(b"   :").unwrap();
        header_separator(b"   ").unwrap_err();
    }
}