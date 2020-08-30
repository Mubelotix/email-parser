use nom::IResult;
use nom::bytes::complete::{tag, take_while, take_while1};
use nom::sequence::tuple;
use nom::combinator::opt;
use nom::multi::many0;

fn is_ftext(character: u8) -> bool {
    character >= 33 && character != 58 && character <= 126
}

fn is_text(character: u8) -> bool {
    character >= 1 && character <= 9 ||
    character >= 14 && character <= 127 ||
    character == 11 ||
    character == 12
}

fn unstructured_header_value(data: &[u8]) -> IResult<&[u8], &[u8]> {
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
                return Ok((&data[idx..], &data[..idx]))
            } else {
                state = State::CarriageReturn
            }
        }
    }

    if state == State::AnythingButWhitespace {
        Ok((&data[data.len() - 2..], &data[..data.len() - 2]))
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

fn optionnal_field(data: &[u8]) -> IResult<&[u8], (&[u8], &[u8])> {
    let result = tuple((field_name, header_separator, unstructured_header_value, tag("\r\n")))(data)?;
    Ok((result.0, ((result.1).0, (result.1).2)))
}
fn optionnal_field_with_separator(data: &[u8]) -> IResult<&[u8], (&[u8], &[u8], &[u8])> {
    let result = tuple((field_name, header_separator, unstructured_header_value, tag("\r\n")))(data)?;
    Ok((result.0, ((result.1).0, (result.1).1, (result.1).2)))
}

fn fields(data: &[u8]) -> IResult<&[u8], Vec<(&[u8], &[u8])>> {
    many0(optionnal_field)(data) // TODO obs-fields
}
fn fields_with_separators(data: &[u8]) -> IResult<&[u8], Vec<(&[u8], &[u8], &[u8])>> {
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
pub fn parse_message(data: &[u8]) -> Result<(Vec<(&[u8], &[u8])>, Option<&[u8]>), ()> {
    if let Ok(result) = tuple((fields, opt(tuple((tag("\r\n"), body)))))(data) {
        Ok(((result.1).0, (result.1).1.map(|b| b.1)))
    } else {
        Err(())
    }
}

/// Like `parse_message()` but headers are a tuple of three values: (name, separator, value).  
///   
/// Example: `"HeaderName  : header value\r\n"` gives `("HeaderName", "  :", "header value")`
pub fn parse_message_with_separators(data: &[u8]) -> Result<(Vec<(&[u8], &[u8], &[u8])>, Option<&[u8]>), ()> {
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
        let mail = include_bytes!("../mail.txt");
        
        println!("{:?}", parse_message(b"From: Once upon a time...\r\n There was a house\r\nTodo: But it has been destroyed\r\nEnd: That's the\r\n end.\r\n"));
        
        let (headers, body) = parse_message(mail).unwrap();
        for (name, value) in headers {
            println!("{}:{}", std::str::from_utf8(name).unwrap(), std::str::from_utf8(value).unwrap())
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