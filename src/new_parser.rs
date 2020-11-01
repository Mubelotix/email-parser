#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    Known(&'static str)
}

#[derive(Debug)]
pub enum String<'a> {
    Reference(&'a [u8]),
    Owned(std::string::String),
}

impl<'a> String<'a> {
    fn as_str(&self) -> &str {
        match self {
            String::Reference(string) => unsafe {
                // the parser is using only safe ASCII characters
                std::str::from_utf8_unchecked(string)
            },
            String::Owned(string) => &string
        }
    }

    fn into_owned(self) -> String<'static> {
        match self {
            String::Reference(_) => String::Owned(self.as_str().to_string()),
            String::Owned(string) => String::Owned(string)
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Reference(s) => s.len(),
            Self::Owned(s) => s.len(),
        }
    }
}

impl<'a> std::ops::Add for String<'a> {
    type Output = String<'a>;

    fn add(mut self, rhs: String<'a>) -> Self::Output {
        self += rhs;
        self
    }
}

impl<'a> std::cmp::PartialEq<&str> for String<'a> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str().eq(*other)
    }
}

impl<'a> std::ops::AddAssign for String<'a> {
    fn add_assign(&mut self, rhs: Self) {
        match self {
            String::Reference(data1) => {
                if let String::Reference(data2) = rhs {
                    if data2.is_empty() {
                        return;
                    }
                    if data1.is_empty() {
                        *data1 = data2;
                        return;
                    }
                    if let (Some(first1), Some(last1), Some(first2), Some(last2)) = (data1.first(), data1.last(), data2.first(), data2.last()) {
                        // if the two references are consecutive in memory, we create a third reference containing them
                        unsafe {
                            let first1 = first1 as *const u8;
                            let last1 = last1 as *const u8;
                            let first2 = first2 as *const u8;
                            let last2 = last2 as *const u8;
                            if last1 as usize + std::mem::size_of::<u8>() == first2 as usize { // this is what guarantee safety
                                let slice = std::slice::from_raw_parts(first1, last2 as usize - first1 as usize + 1);
                                *data1 = slice;
                                return;
                            }
                        }
                    }
                }
                let string = self.as_str().to_string();
                *self = String::Owned(string + rhs.as_str());
            },
            String::Owned(ref mut string) => {
                string.push_str(rhs.as_str());
            },
        }
    }
}

#[test]
fn unsafe_add_test() {
    let data = b"abcdef";
    let data1 = String::Reference(&data[..3]);
    let data2 = String::Reference(&data[3..]);

    let data3 = String::Reference(&data[..2]);
    let data4 = String::Reference(&data[3..]);

    assert!(matches!(data1+data2, String::Reference(_)));
    assert!(matches!(data3+data4, String::Owned(_)));
}

pub mod combinators {
    use super::{Error, String};

    pub fn inc_while<F>(input: &[u8], idx: &mut usize, mut condition: F) where
        F: FnMut(u8) -> bool {
        while let Some(character) = input.get(*idx) {
            if condition(*character) {
                *idx += 1;
            } else {
                break;
            }
        }
    }

    pub fn inc_while1<F>(input: &[u8], idx: &mut usize, mut condition: F) -> Result<(), ()> where
        F: FnMut(u8) -> bool {
        match input.get(*idx) {
            Some(c) if condition(*c) => {
                *idx += 1;
            }
            _e => {
                return Err(());
            }
        };
        while let Some(character) = input.get(*idx) {
            if condition(*character) {
                *idx += 1;
            } else {
                break;
            }
        }
        Ok(())
    }

    pub fn take_while1<F>(input: &[u8], condition: F) -> Result<(&[u8], String), ()> where
    F: FnMut(u8) -> bool {
        let mut idx = 0;
        inc_while1(input, &mut idx, condition)?;
        Ok((&input[idx..], String::Reference(&input[..idx])))
    }

    pub fn inc_tag(input: &[u8], idx: &mut usize, tag: &[u8]) -> Result<(), ()> {
        if input[*idx..].starts_with(tag) {
            *idx += tag.len();
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn inc_after_opt<F, G>(input: &[u8], idx: &mut usize, mut optional_pattern: F, mut required_pattern: G) -> Result<(), Error> where
        F: FnMut(&[u8], &mut usize) -> Result<(), Error>,
        G: FnMut(&[u8], &mut usize) -> Result<(), Error> {
        let before = *idx;
        let _ = optional_pattern(input, idx);
        match required_pattern(input, idx) {
            Ok(()) => Ok(()),
            Err(e) => {
                *idx = before;
                Err(e)
            }
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;
        use super::super::character_groups::*;

        #[test]
        fn test_inc_while() {
            let mut idx = 0;
            let input = b"     aaa";
            inc_while(input, &mut idx, is_wsp);
            assert_eq!(idx, 5);
            assert_eq!(&input[..idx], b"     ");
            assert_eq!(&input[idx..], b"aaa");
        }
    }
}

pub mod character_groups {
    #[inline]
    pub fn is_wsp(character: u8) -> bool {
        character == 9 || character == 32
    }

    #[inline]
    pub fn is_ctext(character: u8) -> bool {
        (character >= 33 && character <= 39) ||
        (character >= 42 && character <= 91) ||
        (character >= 93 && character <= 126)
    }

    #[inline]
    pub fn is_vchar(character: u8) -> bool {
        character >= 0x21 && character <= 0x7e
    }

    #[inline]
    pub fn is_alpha(c: u8) -> bool {
        (c >= 0x41 && c <= 0x5a) ||
        (c >= 0x61 && c <= 0x7a)
    }

    #[inline]
    pub fn is_digit(c: u8) -> bool {
        c >= 0x30 && c <= 0x39
    }

    pub fn take_digit(input: &[u8]) -> Result<(&[u8], u8), super::Error> {
        match input.get(0) {
            Some(b'0') => Ok((&input[1..], 0)),
            Some(b'1') => Ok((&input[1..], 1)),
            Some(b'2') => Ok((&input[1..], 2)),
            Some(b'3') => Ok((&input[1..], 3)),
            Some(b'4') => Ok((&input[1..], 4)),
            Some(b'5') => Ok((&input[1..], 5)),
            Some(b'6') => Ok((&input[1..], 6)),
            Some(b'7') => Ok((&input[1..], 7)),
            Some(b'8') => Ok((&input[1..], 8)),
            Some(b'9') => Ok((&input[1..], 9)),
            _ => Err(super::Error::Known("Invalid digit"))
        }
    }

    pub fn take_two_digits(input: &[u8]) -> Result<(&[u8], u8), super::Error> {
        let (input, first) = take_digit(input)?;
        let (input, second) = take_digit(input)?;

        Ok((input, first * 10 + second))
    }

    #[inline]
    pub fn is_atext(c: u8) -> bool {
        is_alpha(c) ||
        is_digit(c) ||
        c == b'!' ||
        c == b'#' ||
        c == b'$' ||
        c == b'%' ||
        c == b'&' ||
        c == b'\''||
        c == b'*' ||
        c == b'+' ||
        c == b'-' ||
        c == b'/' ||
        c == b'=' ||
        c == b'?' ||
        c == b'^' ||
        c == b'_' ||
        c == b'`' ||
        c == b'{' ||
        c == b'|' ||
        c == b'}' ||
        c == b'~'
    }

    #[inline]
    pub fn special(c: u8) -> bool {
        c == b'(' ||
        c == b')' ||
        c == b'<' ||
        c == b'>' ||
        c == b'[' ||
        c == b']' ||
        c == b':' ||
        c == b';' ||
        c == b'@' ||
        c == b'\\' ||
        c == b',' ||
        c == b'.' ||
        c == b'"'
    }

    #[inline]
    pub fn is_qtext(c: u8) -> bool {
        (c >= 35 && c <= 126 && c != 92) || c == 33
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_wsp() {
            assert!(is_wsp(b' '));
            assert!(is_wsp(b'\t'));
            assert!(!is_wsp(b'a'));
            assert!(!is_wsp(b'e'));
        }
    }
}

pub mod whitespaces {
    use super::*;
    use super::combinators::*;
    use super::character_groups::*;

    pub fn inc_fws(input: &[u8], mut idx: &mut usize) -> Result<(), Error> {
        let first_value = *idx;
        inc_while(input, &mut idx, is_wsp);
        if inc_tag(input, &mut idx, b"\r\n").is_err() {
            *idx = first_value;
        }
        if inc_while1(input, &mut idx, is_wsp).is_err() {
            return Err(Error::Known("Missing whitespaces in a folding whitespace"));
        }
        Ok(())
    }
    
    pub fn take_fws(input: &[u8]) -> Result<(&[u8], String), Error> {
        let mut idx = 0;
        inc_fws(input, &mut idx)?;
    
        Ok((&input[idx..], String::Reference(&input[..idx])))
    }
    
    fn inc_ccontent(input: &[u8], idx: &mut usize) -> Result<(), Error> {
        if inc_while1(input, idx, is_ctext).is_ok() || inc_quoted_pair(input, idx).is_ok() || inc_comment(input, idx).is_ok() {
            Ok(())
        } else {
            Err(Error::Known("Invalid ccontent"))
        }
    }
    
    pub fn inc_comment(input: &[u8], idx: &mut usize) -> Result<(), Error> {
        if inc_tag(input, idx, b"(").is_err() {
            return Err(Error::Known("Comment is expected to start with a '('."));
        }
        
        while inc_after_opt(input, idx, inc_fws, inc_ccontent).is_ok() { }
    
        let _ = inc_fws(input, idx);
        if inc_tag(input, idx, b")").is_err() {
            return Err(Error::Known("Comment is expected to end with a ')'."));
        }
    
        Ok(())
    }
    
    pub fn inc_cfws(input: &[u8], idx: &mut usize) -> Result<(), Error> {
        if inc_after_opt(input, idx, inc_fws, inc_comment).is_ok() {
            while inc_after_opt(input, idx, inc_fws, inc_comment).is_ok() {}
            //let _ = inc_fws(input, idx);
            inc_fws(input, idx)
        } else {
            inc_fws(input, idx)
        }
    }

    pub fn take_cfws(input: &[u8]) -> Result<(&[u8], String), Error> {
        let mut idx = 0;
        inc_cfws(input, &mut idx)?;
        Ok((&input[idx..], String::Reference(&input[..idx])))
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_fws() {
            assert_eq!(take_fws(b"   test").unwrap().1, "   ");
            assert_eq!(take_fws(b" test").unwrap().1, " ");
            assert_eq!(take_fws(b"   \r\n  test").unwrap().1, "   \r\n  ");
        
            assert!(take_fws(b"  \r\ntest").is_err());
            assert!(take_fws(b"\r\ntest").is_err());
            assert!(take_fws(b"test").is_err());
        }

        #[test]
        fn test_inc_ccontent() {
            let mut idx = 0;
            inc_ccontent(b"abcde", &mut idx).unwrap();
            assert_eq!(idx, 5);
        
            let mut idx = 0;
            inc_ccontent(b"ab)cde", &mut idx).unwrap();
            assert_eq!(idx, 2);
        }

        #[test]
        fn test_inc_comment() {
            let mut idx = 0;
            inc_comment(b"(this is a comment)", &mut idx).unwrap();
            assert_eq!(idx, 19);
            
            let mut idx = 0;
            inc_comment(b"(a comment) and a value", &mut idx).unwrap();
            assert_eq!(idx, 11);
        
            let mut idx = 0;
            inc_comment(b"(this is a comment (and another comment)) and a value", &mut idx).unwrap();
            assert_eq!(idx, 41);
        
            assert!(inc_comment(b"a value", &mut 0).is_err());
            assert!(inc_comment(b"(unclosed comment", &mut 0).is_err());
        }

        #[test]
        fn test_inc_cfws() {
            let mut idx = 0;
            inc_cfws(b"  (this is a comment)\r\n (this is a second comment)  value", &mut idx).unwrap();
            assert_eq!(idx, 52);
        
            idx = 0;
            inc_cfws(b"  (this is a comment)\r\n (this is a second comment)\r\n  value", &mut idx).unwrap();
            assert_eq!(idx, 54);
        }
    }
}

pub mod date {
    use super::{Error, String, whitespaces::*, combinators::*, character_groups::*};

    #[derive(Debug, PartialEq)]
    pub enum Day {
        Monday, Tuesday, Wednesday, Thursday, Friday, Saturday, Sunday
    }

    #[derive(Debug, PartialEq)]
    pub enum Month {
        January,
        February,
        March,
        April,
        May,
        June,
        July,
        August,
        September,
        October,
        November,
        December,
    }

    pub fn take_day_name(input: &[u8]) -> Result<(&[u8], Day), Error> {
        if let (Some(input), Some(letters)) = (input.get(3..), input.get(..3)) {
            let letters = letters.to_ascii_lowercase();
            match letters.as_slice() {
                b"mon" => Ok((input, Day::Monday)),
                b"tue" => Ok((input, Day::Tuesday)),
                b"wed" => Ok((input, Day::Wednesday)),
                b"thu" => Ok((input, Day::Thursday)),
                b"fri" => Ok((input, Day::Friday)),
                b"sat" => Ok((input, Day::Saturday)),
                b"sun" => Ok((input, Day::Sunday)),
                _ => Err(Error::Known("Not a valid day_name")),
            }
        } else {
            Err(Error::Known("Expected day_name, but characters are missing (at least 3)."))
        }
    }

    pub fn take_month(input: &[u8]) -> Result<(&[u8], Month), Error> {
        if let (Some(input), Some(letters)) = (input.get(3..), input.get(..3)) {
            let letters = letters.to_ascii_lowercase();
            match letters.as_slice() {
                b"jan" => Ok((input, Month::January)),
                b"feb" => Ok((input, Month::February)),
                b"mar" => Ok((input, Month::March)),
                b"apr" => Ok((input, Month::April)),
                b"may" => Ok((input, Month::May)),
                b"jun" => Ok((input, Month::June)),
                b"jul" => Ok((input, Month::July)),
                b"aug" => Ok((input, Month::August)),
                b"sep" => Ok((input, Month::September)),
                b"oct" => Ok((input, Month::October)),
                b"nov" => Ok((input, Month::November)),
                b"dec" => Ok((input, Month::December)),
                _ => Err(Error::Known("Not a valid month")),
            }
        } else {
            Err(Error::Known("Expected month, but characters are missing (at least 3)."))
        }
    }

    pub fn take_day_of_week(mut input: &[u8]) -> Result<(&[u8], Day), Error> {
        if let Ok((new_input, _fws)) = take_fws(input) {
            input = new_input;
        }

        let (input, day) = take_day_name(input)?;

        if input.starts_with(b",") {
            Ok((&input[1..], day))
        } else {
            Err(Error::Known("day_of_week must end with a comma."))
        }
    }

    pub fn take_year(input: &[u8]) -> Result<(&[u8], usize), Error> {
        let (input, _) = take_fws(input)?;

        let (input, year) = take_while1(input, is_digit).map_err(|()| Error::Known("no digit in year"))?;
        if year.len() < 4 {
            return Err(Error::Known("year is expected to have 4 digits or more"))
        }
        let year: usize = year.as_str().parse().map_err(|_e| Error::Known("Failed to parse year"))?;

        if year < 1990 {
            return Err(Error::Known("year must be after 1990"))
        }

        let (input, _) = take_fws(input)?;

        Ok((input, year))
    }

    pub fn take_day(mut input: &[u8]) -> Result<(&[u8], usize), Error> {
        if let Ok((new_input, _)) = take_fws(input) {
            input = new_input;
        };

        let (mut input, mut day) = take_digit(input)?;

        if let Ok((new_input, digit)) = take_digit(input) {
            day *= 10;
            day += digit;
            input = new_input;
        }

        if day > 31 {
            return Err(Error::Known("day must be less than 31"))
        }

        let (input, _) = take_fws(input)?;

        Ok((input, day as usize))
    }

    pub fn take_time_of_day(input: &[u8]) -> Result<(&[u8], (u8, u8, u8)), Error> {
        let (mut input, hour) = take_two_digits(input)?;
        if hour > 23 {
            return Err(Error::Known("There is only 24 hours in a day"));
        }
        if input.starts_with(b":") {
            input = &input[1..];
        } else {
            return Err(Error::Known("Expected colon after hour"));
        }

        let (input, minutes) = take_two_digits(input)?;
        if minutes > 59 {
            return Err(Error::Known("There is only 60 minutes per hour"));
        }

        if input.starts_with(b":") {
            let new_input = &input[1..];
            if let Ok((new_input, seconds)) = take_two_digits(new_input) {
                if seconds > 60 { // leap second allowed
                    return Err(Error::Known("There is only 60 seconds in a minute"));
                }
                return Ok((new_input, (hour, minutes, seconds)));
            }
        }

        Ok((input, (hour, minutes, 0)))
    }

    pub fn take_zone(input: &[u8]) -> Result<(&[u8], (bool, u8, u8)), Error> {
        let (mut input, _fws) = take_fws(input)?;

        let sign = match input.get(0) {
            Some(b'+') => true,
            Some(b'-') => false,
            None => return Err(Error::Known("Expected more characters in zone")),
            _ => return Err(Error::Known("Invalid sign character in zone")),
        };
        input = &input[1..];

        let (input, hours) = take_two_digits(input)?;
        let (input, minutes) = take_two_digits(input)?;

        if minutes > 59 {
            return Err(Error::Known("zone minutes out of range"));
        }

        Ok((input, (sign, hours, minutes)))
    }

    pub fn take_time(input: &[u8]) -> Result<(&[u8], ((u8, u8, u8), (bool, u8, u8))), Error> {
        let (input, time_of_day) = take_time_of_day(input)?;
        let (input, zone) = take_zone(input)?;
        Ok((input, (time_of_day, zone)))
    }

    pub fn take_date(input: &[u8]) -> Result<(&[u8], (usize, Month, usize)), Error> {
        let (input, day) = take_day(input)?;
        let (input, month) = take_month(input)?;
        let (input, year) = take_year(input)?;
        Ok((input, (day, month, year)))
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_day() {
            assert_eq!(take_day_name(b"Mon ").unwrap().1, Day::Monday);
            assert_eq!(take_day_name(b"moN ").unwrap().1, Day::Monday);
            assert_eq!(take_day_name(b"thu").unwrap().1, Day::Thursday);

            assert_eq!(take_day_of_week(b"   thu, ").unwrap().1, Day::Thursday);
            assert_eq!(take_day_of_week(b"wed, ").unwrap().1, Day::Wednesday);
            assert_eq!(take_day_of_week(b" Sun,").unwrap().1, Day::Sunday);

            assert_eq!(take_day(b"31 ").unwrap().1, 31);
            assert_eq!(take_day(b"9 ").unwrap().1, 9);
            assert_eq!(take_day(b"05 ").unwrap().1, 5);
            assert_eq!(take_day(b"23 ").unwrap().1, 23);
        }

        #[test]
        fn test_month_and_year() {
            assert_eq!(take_month(b"Apr ").unwrap().1, Month::April);
            assert_eq!(take_month(b"may ").unwrap().1, Month::May);
            assert_eq!(take_month(b"deC ").unwrap().1, Month::December);

            assert_eq!(take_year(b" 2020 ").unwrap().1, 2020);
            assert_eq!(take_year(b"\r\n 1958 ").unwrap().1, 1958);
            assert_eq!(take_year(b" 250032 ").unwrap().1, 250032);
        }
        
        #[test]
        fn test_date() {
            assert_eq!(take_date(b"1 nov 2020 ").unwrap().1, (1, Month::November, 2020));
            assert_eq!(take_date(b"25 dec 2038 ").unwrap().1, (25, Month::December, 2038));
        }

        #[test]
        fn test_time() {
            assert_eq!(take_time_of_day(b"10:40:29").unwrap().1, (10, 40, 29));
            assert_eq!(take_time_of_day(b"10:40 ").unwrap().1, (10, 40, 0));
            assert_eq!(take_time_of_day(b"05:23 ").unwrap().1, (5, 23, 0));

            assert_eq!(take_zone(b" +1000 ").unwrap().1, (true, 10, 0));
            assert_eq!(take_zone(b" -0523 ").unwrap().1, (false, 5, 23));

            assert_eq!(take_time(b"06:44 +0100").unwrap().1, ((6, 44, 0), (true, 1, 0)));
            assert_eq!(take_time(b"23:57 +0000").unwrap().1, ((23, 57, 0), (true, 0, 0)));
            assert_eq!(take_time(b"08:23:02 -0500").unwrap().1, ((8, 23, 2), (false, 5, 0)));

        }
    }
}

use character_groups::*;
use combinators::*;
use whitespaces::*;
use date::*;

pub fn inc_quoted_pair(input: &[u8], idx: &mut usize) -> Result<(), Error> {
    if input[*idx..].starts_with(b"\\") {
        if let Some(character) = input.get(*idx + 1) {
            if is_vchar(*character) || is_wsp(*character) {
                *idx += 2;
                Ok(())
            } else {
                Err(Error::Known("The quoted-pair character is no a vchar or a wsp."))
            }
        } else {
            Err(Error::Known("The quoted-pair has no second character."))
        }
    } else {
        Err(Error::Known("The quoted-pair does not start with a '\\'."))
    }
}

pub fn take_quoted_pair(input: &[u8]) -> Result<(&[u8], String), Error> {
    if input.starts_with(b"\\") {
        if let Some(character) = input.get(1) {
            if is_vchar(*character) || is_wsp(*character) {
                Ok((&input[2..], String::Reference(&input[1..2])))
            } else {
                Err(Error::Known("The quoted-pair character is no a vchar or a wsp."))
            }
        } else {
            Err(Error::Known("The quoted-pair has no second character."))
        }
    } else {
        Err(Error::Known("The quoted-pair does not start with a '\\'."))
    }
}

pub fn take_quoted_string(input: &[u8]) -> Result<(&[u8], String), Error> {
    let input = if let Ok((input, _cfws)) = take_cfws(input) {
        input
    } else {
        input
    };

    let mut input = if input.starts_with(b"\"") {
        &input[1..]
    } else {
        return Err(Error::Known("Quoted string must begin with a dquote"));
    };
    let mut output = String::Reference(&[]);

    loop {
        let mut additionnal_output = String::Reference(&[]);
        
        let new_input = if let Ok((new_input, fws)) = take_fws(input) {
            additionnal_output += fws;
            new_input
        } else {
            input
        };

        let new_input = if let Ok((new_input, str)) = take_while1(new_input, is_qtext) {
            additionnal_output += str;
            new_input
        } else if let Ok((new_input, str)) = take_quoted_pair(new_input) {
            additionnal_output += str;
            new_input
        } else {
            break;
        };

        output += additionnal_output;
        input = new_input;
    }

    let input = if let Ok((input, fws)) = take_fws(input) {
        output += fws;
        input
    } else {
        input
    };

    let input = if input.starts_with(b"\"") {
        &input[1..]
    } else {
        return Err(Error::Known("Quoted string must end with a dquote"));
    };

    let input = if let Ok((input, _cfws)) = take_cfws(input) {
        input
    } else {
        input
    };

    Ok((input, output))
}

pub fn inc_atom(input: &[u8], idx: &mut usize) -> Result<(), Error> {
    let _ = inc_cfws(input, idx);
    if inc_while1(input, idx, is_atext).is_err() {
        return Err(Error::Known("No atom here"));
    }
    let _ = inc_cfws(input, idx);
    Ok(())
}

pub fn inc_dot_atom_text(input: &[u8], idx: &mut usize) -> Result<(), Error> {
    if inc_while1(input, idx, is_atext).is_err() {
        return Err(Error::Known("Expected atom character at the beggining of a dot_atom_text"));
    }

    loop {
        let idx_before = *idx;
        if inc_tag(input, idx, b".").is_err() {
            break;
        }
        if inc_while1(input, idx, is_atext).is_err() {
            *idx = idx_before;
            break;
        }
    }

    Ok(())
}

pub fn inc_dot_atom(input: &[u8], idx: &mut usize) -> Result<(), Error> {
    let _ = inc_cfws(input, idx);
    inc_dot_atom_text(input, idx)?;
    let _ = inc_cfws(input, idx);

    Ok(())
}

pub fn take_atom(mut input: &[u8]) -> Result<(&[u8], String), Error> {
    if let Ok((new_input, _)) = take_cfws(input) {
        input = new_input
    }
    let (mut input, atom) = take_while1(input, is_atext).map_err(|_| Error::Known("Atom required"))?;
    if let Ok((new_input, _)) = take_cfws(input) {
        input = new_input
    }
    Ok((input, atom))
}

pub fn take_dot_atom_text(input: &[u8]) -> Result<(&[u8], String), Error> {
    let mut idx = 0;
    inc_dot_atom_text(input, &mut idx)?;
    Ok((&input[idx..], String::Reference(&input[..idx])))
}

pub fn take_dot_atom(mut input: &[u8]) -> Result<(&[u8], String), Error> {
    if let Ok((new_input, _)) = take_cfws(input) {
        input = new_input
    }
    let (mut input, dot_atom) = take_dot_atom_text(input)?;
    if let Ok((new_input, _)) = take_cfws(input) {
        input = new_input
    }
    Ok((input, dot_atom))
}

pub fn take_word(input: &[u8]) -> Result<(&[u8], String), Error> {
    if let Ok((input, word)) = take_atom(input) {
        Ok((input, word))
    } else if let Ok((input, word)) = take_quoted_string(input) {
        Ok((input, word))
    } else {
        Err(Error::Known("Word is not an atom and is not a quoted_string."))
    }
}

pub fn take_phrase(input: &[u8]) -> Result<(&[u8], Vec<String>), Error> {
    let mut words = Vec::new();
    let (mut input, word) = take_word(input)?;
    words.push(word);

    while let Ok((new_input, word)) = take_word(input) {
        input = new_input;
        words.push(word)
    }

    Ok((input, words))
}

pub fn take_unstructured(mut input: &[u8]) -> Result<(&[u8], String), Error>{
    let mut output = String::Reference(&[]);

    loop {
        let (new_input, fws) = if let Ok((new_input, fws)) = take_fws(input) {
            (new_input, fws)
        } else {
            (input, String::Reference(&[]))
        };

        if let Ok((new_input, characters)) = take_while1(new_input, is_vchar) {
            output += fws;
            output += characters;
            input = new_input;
        } else {
            break;
        };
    }

    while let Ok((new_input, _wsp)) = take_while1(input, is_wsp) {
        input = new_input;
    }

    Ok((input, output))
}

#[test]
fn test_word_and_phrase() {
    assert_eq!(take_word(b" this is a \"rust\\ test\" ").unwrap().1, "this");
    assert_eq!(take_phrase(b" this is a \"rust\\ test\" ").unwrap().1, vec!["this", "is", "a", "rust test"]);
}

#[test]
fn test_unstructured() {
    assert_eq!(take_unstructured(b"the quick brown fox jumps\r\n over the lazy dog   ").unwrap().1, "the quick brown fox jumps\r\n over the lazy dog");
}

#[test]
fn test_quoted_pair() {
    assert!(inc_quoted_pair(b"\\rtest", &mut 0).is_ok());
    assert!(inc_quoted_pair(b"\\ test", &mut 0).is_ok());

    assert_eq!(take_quoted_pair(b"\\rtest").unwrap().1, "r");
    assert_eq!(take_quoted_pair(b"\\ test").unwrap().1, " ");

    assert!(inc_quoted_pair(b"\\", &mut 0).is_err());
    assert!(inc_quoted_pair(b"\\\0", &mut 0).is_err());
    assert!(inc_quoted_pair(b"test", &mut 0).is_err());
}

#[test]
fn test_quoted_string() {
    assert_eq!(take_quoted_string(b" \"This\\ is\\ a\\ test\"").unwrap().1, "This is a test");
    assert_eq!(take_quoted_string(b"\r\n  \"This\\ is\\ a\\ test\"  ").unwrap().1, "This is a test");

    assert!(matches!(take_quoted_string(b"\r\n  \"This\\ is\\ a\\ test\"  ").unwrap().1, String::Owned(_)));
    assert!(matches!(take_quoted_string(b"\r\n  \"hey\"  ").unwrap().1, String::Reference(_)));
}

#[test]
fn test_atom() {
    assert_eq!(take_atom(b"this is a test").unwrap().1, "this");
    assert_eq!(take_atom(b"   averylongatom ").unwrap().1, "averylongatom");
    assert_eq!(take_dot_atom_text(b"this.is.a.test").unwrap().1, "this.is.a.test");
    assert_eq!(take_dot_atom(b"  this.is.a.test ").unwrap().1, "this.is.a.test");
}