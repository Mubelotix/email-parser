use std::{borrow::Cow, collections::HashMap};
use crate::prelude::*;

pub fn decode_parameter<'a, 'b>(input: Cow<'a, str>, charset: Cow<'b, str>) -> Result<Cow<'a, str>, Error> {
    if !["utf-8", "us-ascii", "iso-8859-1", "iso-8859-2", "iso-8859-3", "iso-8859-4", "iso-8859-5", "iso-8859-6", "iso-8859-7", "iso-8859-8", "iso-8859-9", "iso-8859-10", "iso-8859-11", "iso-8859-13", "iso-8859-14", "iso-8859-15", "iso-8859-16", "iso-6937", "gb2312"].contains(&charset.as_ref()) {
        return Ok(input);
    }

    let mut input = match input {
        Cow::Borrowed(input) => input.as_bytes().to_vec(),
        Cow::Owned(input) => input.into_bytes(),
    };

    let mut percents = Vec::new();
    for (idx, byte) in input.iter().enumerate() {
        if *byte == b'%' {
            percents.push(idx);
        }
    }

    for percent in percents.iter().rev() {
        fn from_hex(n: &u8) -> Option<u8> {
            match n {
                b'0'..=b'9' => Some(n - b'0'),
                b'A'..=b'F' => Some(10 + n - b'A'),
                b'a'..=b'f' => Some(10 + n - b'a'),
                _ => None,
            }
        }

        if let (Some(first), Some(second)) = (input.get(percent + 1), input.get(percent + 2)) {
            if let (Some(first), Some(second)) = (from_hex(first), from_hex(second)) {
                let n = first * 16 + second;
                unsafe {
                    *input.get_unchecked_mut(*percent) = n;
                    input.remove(percent + 2);
                    input.remove(percent + 1);
                }
            }
        }
    }

    use textcode::*;
    let text: Cow<str> = match charset.as_ref() {
        "utf-8" | "us-ascii" => {
            Cow::Owned(String::from_utf8(input).map_err(|_| Error::Known("Invalid text encoding"))?)
        }
        "iso-8859-1" => Cow::Owned(iso8859_1::decode_to_string(&input)),
        "iso-8859-2" => Cow::Owned(iso8859_2::decode_to_string(&input)),
        "iso-8859-3" => Cow::Owned(iso8859_3::decode_to_string(&input)),
        "iso-8859-4" => Cow::Owned(iso8859_4::decode_to_string(&input)),
        "iso-8859-5" => Cow::Owned(iso8859_5::decode_to_string(&input)),
        "iso-8859-6" => Cow::Owned(iso8859_6::decode_to_string(&input)),
        "iso-8859-7" => Cow::Owned(iso8859_7::decode_to_string(&input)),
        "iso-8859-8" => Cow::Owned(iso8859_8::decode_to_string(&input)),
        "iso-8859-9" => Cow::Owned(iso8859_9::decode_to_string(&input)),
        "iso-8859-10" => Cow::Owned(iso8859_10::decode_to_string(&input)),
        "iso-8859-11" => Cow::Owned(iso8859_11::decode_to_string(&input)),
        "iso-8859-13" => Cow::Owned(iso8859_13::decode_to_string(&input)),
        "iso-8859-14" => Cow::Owned(iso8859_14::decode_to_string(&input)),
        "iso-8859-15" => Cow::Owned(iso8859_15::decode_to_string(&input)),
        "iso-8859-16" => Cow::Owned(iso8859_16::decode_to_string(&input)),
        "iso-6937" => Cow::Owned(iso6937::decode_to_string(&input)),
        "gb2312" => Cow::Owned(gb2312::decode_to_string(&input)),
        _ => return Err(Error::Known("Unknown charset")),
    };

    Ok(text)
}

pub fn collect_parameters(parameters_vec: Vec<(Cow<str>, Option<u8>, bool, Cow<str>)>) -> Result<HashMap<Cow<str>, Cow<str>>, Error> {
    let mut parameters: HashMap<_, _> = HashMap::new();
    let mut complex_parameters: HashMap<_, HashMap<_, _>> = HashMap::new();
    for (name, index, encoded, value) in parameters_vec {
        if let Some(index) = index {
            if !complex_parameters.contains_key(&name) {
                complex_parameters.insert(name.clone(), HashMap::new());
            }
            complex_parameters.get_mut(&name).unwrap().insert(index, (encoded, value));
        } else {
            parameters.insert(name, value);
        }
    }
    for (name, values) in complex_parameters.iter_mut() {
        if let Some((encoded, value)) = values.remove(&0) {
            let (mut value, charset, language) = if encoded {
                let value = unsafe {
                    std::mem::transmute::<_, &'static [u8]>(value.as_ref())
                };
                let (value, charset) = take_while(value, |c| c!=b'\'')?;
                let charset = Cow::Owned(charset.to_lowercase());
                let (value, _) = tag(value, b"'")?;
                let (value, language) = take_while(value, |c| c!=b'\'')?;
                let (value, _) = tag(value, b"'")?;
                (decode_parameter(unsafe {Cow::Borrowed(std::str::from_utf8_unchecked(value))}, Cow::Borrowed(&charset))?, Some(charset), Some(language))
            } else {
                (value, None, None)
            };

            let mut idx = 1;
            while let Some((encoded, new_value)) = values.remove(&idx) {
                if encoded && charset.is_some() {
                    add_string(&mut value, decode_parameter(new_value, charset.clone().unwrap())?);
                } else {
                    add_string(&mut value, new_value);
                }
                idx+=1;
            }

            parameters.insert(Cow::Owned(name.clone().into_owned()), value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_encoding() {
        assert_eq!("This is even more ", decode_parameter("This%20is%20even%20more%20".into(), "us-ascii".into()).unwrap());
        assert_eq!("***fun*** ", decode_parameter("%2A%2A%2Afun%2A%2A%2A%20".into(), "us-ascii".into()).unwrap());
    }
}