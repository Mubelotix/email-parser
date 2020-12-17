use crate::prelude::*;
use std::{borrow::Cow, collections::HashMap};

pub fn decode_parameter(mut input: Vec<u8>, charset: Cow<str>) -> Result<String, Error> {
    if ![
        "utf-8",
        "us-ascii",
        "iso-8859-1",
        "iso-8859-2",
        "iso-8859-3",
        "iso-8859-4",
        "iso-8859-5",
        "iso-8859-6",
        "iso-8859-7",
        "iso-8859-8",
        "iso-8859-9",
        "iso-8859-10",
        "iso-8859-11",
        "iso-8859-13",
        "iso-8859-14",
        "iso-8859-15",
        "iso-8859-16",
        "iso-6937",
        "gb2312",
    ]
    .contains(&charset.as_ref())
    {
        // JUSTIFICATION
        //  Benefit
        //      Gain performances by avoiding the utf8 string check.
        //  Correctness
        //      It's valid ASCII so it cannot be invalid utf8.
        return Ok(unsafe { String::from_utf8_unchecked(input.to_vec()) });
    }

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
                // JUSTIFICATION
                //  Benefit
                //      Improve performances by avoiding useless index checks.
                //  Correctness:
                //      We never delete items before `percent`. Since `percent` is decreasing, there is always an item at this index.
                unsafe {
                    *input.get_unchecked_mut(*percent) = n;
                }
                input.remove(percent + 2);
                input.remove(percent + 1);
            }
        }
    }

    use textcode::*;
    let text = match charset.as_ref() {
        "utf-8" | "us-ascii" => {
            String::from_utf8(input).map_err(|_| Error::Known("Invalid text encoding"))?
        }
        "iso-8859-1" => iso8859_1::decode_to_string(&input),
        "iso-8859-2" => iso8859_2::decode_to_string(&input),
        "iso-8859-3" => iso8859_3::decode_to_string(&input),
        "iso-8859-4" => iso8859_4::decode_to_string(&input),
        "iso-8859-5" => iso8859_5::decode_to_string(&input),
        "iso-8859-6" => iso8859_6::decode_to_string(&input),
        "iso-8859-7" => iso8859_7::decode_to_string(&input),
        "iso-8859-8" => iso8859_8::decode_to_string(&input),
        "iso-8859-9" => iso8859_9::decode_to_string(&input),
        "iso-8859-10" => iso8859_10::decode_to_string(&input),
        "iso-8859-11" => iso8859_11::decode_to_string(&input),
        "iso-8859-13" => iso8859_13::decode_to_string(&input),
        "iso-8859-14" => iso8859_14::decode_to_string(&input),
        "iso-8859-15" => iso8859_15::decode_to_string(&input),
        "iso-8859-16" => iso8859_16::decode_to_string(&input),
        "iso-6937" => iso6937::decode_to_string(&input),
        "gb2312" => gb2312::decode_to_string(&input),
        _ => return Err(Error::Known("Unknown charset")),
    };

    Ok(text)
}

pub fn collect_parameters<'a>(
    parameters_vec: Vec<(Cow<'a, str>, Option<u8>, bool, Cow<'a, str>)>,
) -> Result<HashMap<Cow<'a, str>, Cow<'a, str>>, Error> {
    let mut parameters: HashMap<_, _> = HashMap::new();
    let mut complex_parameters: HashMap<_, HashMap<_, _>> = HashMap::new();
    for (name, index, encoded, value) in parameters_vec {
        if let Some(index) = index {
            if !complex_parameters.contains_key(&name) {
                complex_parameters.insert(name.clone(), HashMap::new());
            }
            complex_parameters
                .get_mut(&name)
                .unwrap()
                .insert(index, (encoded, value));
        } else {
            parameters.insert(name, value);
        }
    }
    for (name, values) in complex_parameters.iter_mut() {
        if let Some((encoded, value)) = values.remove(&0) {
            let (mut value, charset, _language) = if encoded {
                match value {
                    Cow::Borrowed(value) => {
                        let (value, charset) = take_while(value.as_bytes(), |c| c != b'\'')?;
                        let charset = lowercase(Cow::Borrowed(charset));
                        let (value, _) = tag(value, b"'")?;
                        let (value, language) = take_while(value, |c| c != b'\'')?;
                        let (value, _) = tag(value, b"'")?;
                        (
                            Cow::Owned(decode_parameter(value.to_vec(), charset.clone())?),
                            Some(charset),
                            Some(Cow::Borrowed(language)),
                        )
                    }
                    Cow::Owned(value) => {
                        let (value, charset) = take_while(value.as_bytes(), |c| c != b'\'')?;
                        let charset = lowercase(Cow::Borrowed(charset));
                        let (value, _) = tag(value, b"'")?;
                        let (value, language) = take_while(value, |c| c != b'\'')?;
                        let (value, _) = tag(value, b"'")?;
                        (
                            Cow::Owned(decode_parameter(value.to_vec(), charset.clone())?),
                            Some(Cow::Owned(charset.into_owned())),
                            Some(Cow::Owned(language.to_owned())),
                        )
                    }
                }
            } else {
                (value, None, None)
            };

            let mut idx = 1;
            while let Some((encoded, new_value)) = values.remove(&idx) {
                if encoded && charset.is_some() {
                    add_string(
                        &mut value,
                        Cow::Owned(decode_parameter(
                            new_value.into_owned().into_bytes(),
                            charset.clone().unwrap(),
                        )?),
                    );
                } else {
                    add_string(&mut value, new_value);
                }
                idx += 1;
            }

            parameters.insert(Cow::Owned(name.clone().into_owned()), value);
        }
    }
    Ok(parameters)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percent_encoding() {
        assert_eq!(
            "This is even more ",
            decode_parameter("This%20is%20even%20more%20".into(), "us-ascii".into()).unwrap()
        );
        assert_eq!(
            "***fun*** ",
            decode_parameter("%2A%2A%2Afun%2A%2A%2A%20".into(), "us-ascii".into()).unwrap()
        );
    }
}
