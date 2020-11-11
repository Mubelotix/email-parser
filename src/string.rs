pub enum String<'a> {
    Str(&'a str),
    Owned(std::string::String),
}

impl<'a> String<'a> {
    pub fn new() -> String<'static> {
        String::Reference(&[])
    }

    pub fn is_owned(&self) -> bool {
        matches!(self, String::Owned(_))
    }

    pub fn as_str(&self) -> &str {
        match self {
            String::Str(string) => string,
            String::Owned(string) => &string,
        }
    }

    pub fn into_owned(self) -> String<'static> {
        match self {
            String::Str(_) => String::Owned(self.as_str().to_string()),
            String::Owned(string) => String::Owned(string),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Self::Str(s) => s.len(),
            Self::Owned(s) => s.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[deprecated(note = "Please use the Str variant instead")]
    #[inline]
    pub fn Reference(slice: &[u8]) -> String {
        String::Str(unsafe {
            std::str::from_utf8_unchecked(slice)
        })
    }
}

impl<'a> Default for String<'a> {
    fn default() -> Self {
        String::new()
    }
}

impl<'a> std::fmt::Debug for String<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{:?}",
            if self.is_owned() { "" } else { "&" },
            self.as_str()
        )
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
            String::Str(data1) => {
                if let String::Str(data2) = rhs {
                    if data2.is_empty() {
                        return;
                    }
                    if data1.is_empty() {
                        *data1 = data2;
                        return;
                    }
                    // if the two references are consecutive in memory, we create a third reference containing them
                    unsafe {
                        let first1 = data1.as_ptr();
                        let first2 = data2.as_ptr();
                        if first1 as usize + data1.len() == first2 as usize {
                            // this is what guarantee safety
                            let slice = std::slice::from_raw_parts(
                                first1, 
                                first2 as usize + data2.len() - first1 as usize,
                            );
                            *data1 = std::str::from_utf8_unchecked(slice);
                            return;
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
