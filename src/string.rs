#[derive(Debug)]
pub enum String<'a> {
    Reference(&'a [u8]),
    Owned(std::string::String),
}

impl<'a> String<'a> {
    pub fn new() -> String<'static> {
        String::Reference(&[])
    }

    pub fn as_str(&self) -> &str {
        match self {
            String::Reference(string) => unsafe {
                // the parser is using only safe ASCII characters
                std::str::from_utf8_unchecked(string)
            },
            String::Owned(string) => &string,
        }
    }

    pub fn into_owned(self) -> String<'static> {
        match self {
            String::Reference(_) => String::Owned(self.as_str().to_string()),
            String::Owned(string) => String::Owned(string),
        }
    }

    pub fn len(&self) -> usize {
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
                    if let (Some(first1), Some(last1), Some(first2), Some(last2)) =
                        (data1.first(), data1.last(), data2.first(), data2.last())
                    {
                        // if the two references are consecutive in memory, we create a third reference containing them
                        unsafe {
                            let first1 = first1 as *const u8;
                            let last1 = last1 as *const u8;
                            let first2 = first2 as *const u8;
                            let last2 = last2 as *const u8;
                            if last1 as usize + std::mem::size_of::<u8>() == first2 as usize {
                                // this is what guarantee safety
                                let slice = std::slice::from_raw_parts(
                                    first1,
                                    last2 as usize - first1 as usize + 1,
                                );
                                *data1 = slice;
                                return;
                            }
                        }
                    }
                }
                let string = self.as_str().to_string();
                *self = String::Owned(string + rhs.as_str());
            }
            String::Owned(ref mut string) => {
                string.push_str(rhs.as_str());
            }
        }
    }
}
