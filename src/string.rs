use std::borrow::Cow;

#[inline]
pub(crate) fn empty_string() -> Cow<'static, str> {
    Cow::Borrowed("")
}

#[inline]
pub(crate) fn from_slice(slice: &[u8]) -> Cow<str> {
    unsafe { Cow::Borrowed(std::str::from_utf8_unchecked(slice)) }
}

#[inline]
pub(crate) fn add_string<'a, 'b>(s1: &'b mut Cow<'a, str>, s2: Cow<'a, str>) {
    match s1 {
        Cow::Borrowed(data1) => {
            if let Cow::Borrowed(data2) = s2 {
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
            *s1 = Cow::Owned(s1.to_string() + &s2);
        }
        Cow::Owned(ref mut string) => {
            string.push_str(&s2);
        }
    }
}
