use timezone_abbreviations;

use crate::prelude::*;

pub fn day_name(input: &[u8]) -> Res<Day> {
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
            _ => Err(Error::Unknown("Not a valid day_name")),
        }
    } else {
        Err(Error::Unknown(
            "Expected day_name, but characters are missing (at least 3).",
        ))
    }
}

pub fn month(input: &[u8]) -> Res<Month> {
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
            _ => Err(Error::Unknown("Not a valid month")),
        }
    } else {
        Err(Error::Unknown(
            "Expected month, but characters are missing (at least 3).",
        ))
    }
}

pub fn day_of_week(input: &[u8]) -> Res<Day> {
    let (input, _fws) = optional(input, fws);
    let (input, day) = day_name(input)?;
    let (input, ()) = tag(
        input,
        b",",
        "TAG ERROR: In a day_of_week, a day name must be followed by a comma.",
    )?;
    Ok((input, day))
}

pub fn year(input: &[u8]) -> Res<usize> {
    let (input, _) = fws(input)?;

    let (input, year) =
        take_while1(input, is_digit).map_err(|_e| Error::Unknown("no digit in year"))?;

    // Some emails have year only as 10 (for 2010)
    if year.len() == 2 {
        let year: usize = year
            .parse()
            .map_err(|_e| Error::Unknown("Failed to parse year"))?;
        let (input, _) = fws(input)?;
        return Ok((input, 2000 + year));
    } else if year.len() < 4 {
        return Err(Error::Unknown("year is expected to have 4 digits or more"));
    }
    let year: usize = year
        .parse()
        .map_err(|_e| Error::Unknown("Failed to parse year"))?;

    if year < 1990 {
        return Err(Error::Unknown("year must be after 1990"));
    }

    let (input, _) = fws(input)?;

    Ok((input, year))
}

pub fn day(input: &[u8]) -> Res<u8> {
    let (input, _fws) = optional(input, fws);
    let (mut input, mut day) = digit(input)?;
    if let Ok((new_input, digit)) = digit(input) {
        day *= 10;
        day += digit;
        input = new_input;
    }
    if day > 31 {
        return Err(Error::Unknown("day must be less than 31"));
    }
    let (input, _) = fws(input)?;
    Ok((input, day))
}

pub fn time_of_day(input: &[u8]) -> Res<Time> {
    let (input, hour) = match digit(input) {
        // Support `9:23:23` time format
        Ok((new_input, digit)) if new_input.starts_with(b":") => (new_input, digit),
        // Support `09:23:23` time format
        _ => two_digits(input)?,
    };
    if hour > 23 {
        return Err(Error::Unknown("There is only 24 hours in a day"));
    }
    let (input, ()) = tag(
        input,
        b":",
        "TAG ERROR: In a time_of_day, the hour must be followed by a colon.",
    )?;

    let (input, minute) = two_digits(input)?;
    if minute > 59 {
        return Err(Error::Unknown("There is only 60 minutes per hour"));
    }

    if input.starts_with(b":") {
        let new_input = &input[1..];
        if let Ok((new_input, second)) = two_digits(new_input) {
            if second > 60 {
                // leap second allowed
                return Err(Error::Unknown("There is only 60 seconds in a minute"));
            }
            return Ok((
                new_input,
                Time {
                    hour,
                    minute,
                    second,
                },
            ));
        }
    }

    Ok((
        input,
        Time {
            hour,
            minute,
            second: 0,
        },
    ))
}

pub fn zone(input: &[u8]) -> Res<Zone> {
    let (mut input, _fws) = fws(input)?;

    let sign = match input.get(0) {
        Some(b'+') => true,
        Some(b'-') => false,
        None => return Err(Error::Unknown("Expected more characters in zone")),
        _ => {
            // find the end of the line to match against existing timezone abbreviations
            if let Some(break_position) = input.iter().position(|e| e == &b'\r' || e == &b'\n') {
                let position = break_position - 1;
                if break_position > 0 && position <= timezone_abbreviations::max_abbreviation_len()
                {
                    let sub_input = &input[0..=position];
                    if let Some(abbr) = std::str::from_utf8(&sub_input)
                        .ok()
                        .and_then(|abbr| timezone_abbreviations::timezone(&abbr))
                        .and_then(|abbrs| abbrs.first())
                    {
                        return Ok((
                            &input[break_position..],
                            Zone {
                                sign: abbr.sign.is_plus(),
                                hour_offset: abbr.hour_offset,
                                minute_offset: abbr.minute_offset,
                            },
                        ));
                    }
                }
            }
            return Err(Error::Unknown("Invalid sign character in zone"));
        }
    };
    input = &input[1..];

    let (input, hour_offset) = two_digits(input)?;

    // Some mails have a "00:00" timezone
    let (input, minute_offset) = if input.starts_with(b":") {
        two_digits(&input[1..])?
    } else {
        two_digits(input)?
    };

    if minute_offset > 59 {
        return Err(Error::Unknown("zone minute_offset out of range"));
    }

    Ok((
        input,
        Zone {
            sign,
            hour_offset,
            minute_offset,
        },
    ))
}

pub fn time(input: &[u8]) -> Res<TimeWithZone> {
    let (input, time) = time_of_day(input)?;
    let (input, zone) = zone(input)?;
    Ok((input, TimeWithZone { time, zone }))
}

pub fn date(input: &[u8]) -> Res<Date> {
    let (input, day) = day(input)?;
    let (input, month) = month(input)?;
    let (input, year) = year(input)?;
    Ok((input, Date { day, month, year }))
}

pub fn date_time(input: &[u8]) -> Res<DateTime> {
    let (input, day) = optional(input, day_of_week);
    let (input, date) = date(input)?;
    let (input, time) = time(input)?;
    let (input, _cfws) = optional(input, cfws);
    Ok((
        input,
        DateTime {
            day_name: day,
            date,
            time,
        },
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_day() {
        assert_eq!(day_name(b"Mon ").unwrap().1, Day::Monday);
        assert_eq!(day_name(b"moN ").unwrap().1, Day::Monday);
        assert_eq!(day_name(b"thu").unwrap().1, Day::Thursday);

        assert_eq!(day_of_week(b"   thu, ").unwrap().1, Day::Thursday);
        assert_eq!(day_of_week(b"wed, ").unwrap().1, Day::Wednesday);
        assert_eq!(day_of_week(b" Sun,").unwrap().1, Day::Sunday);

        assert_eq!(day(b"31 ").unwrap().1, 31);
        assert_eq!(day(b"9 ").unwrap().1, 9);
        assert_eq!(day(b"05 ").unwrap().1, 5);
        assert_eq!(day(b"23 ").unwrap().1, 23);
    }

    #[test]
    fn test_month_and_year() {
        assert_eq!(month(b"Apr ").unwrap().1, Month::April);
        assert_eq!(month(b"may ").unwrap().1, Month::May);
        assert_eq!(month(b"deC ").unwrap().1, Month::December);

        assert_eq!(year(b" 2020 ").unwrap().1, 2020);
        assert_eq!(year(b"\r\n 1995 ").unwrap().1, 1995);
        assert_eq!(year(b" 250032 ").unwrap().1, 250032);
    }

    #[test]
    fn test_date() {
        assert_eq!(
            date(b"1 nov 2020 ").unwrap().1,
            Date {
                day: 1,
                month: Month::November,
                year: 2020
            }
        );
        assert_eq!(
            date(b"25 dec 2038 ").unwrap().1,
            Date {
                day: 25,
                month: Month::December,
                year: 2038
            }
        );

        assert_eq!(
            date_time(b"Mon, 12 Apr 2023 10:25:03 +0000").unwrap().1,
            DateTime {
                day_name: Some(Day::Monday),
                date: Date {
                    day: 12,
                    month: Month::April,
                    year: 2023
                },
                time: TimeWithZone {
                    time: Time {
                        hour: 10,
                        minute: 25,
                        second: 3
                    },
                    zone: Zone {
                        sign: true,
                        hour_offset: 0,
                        minute_offset: 0
                    }
                },
            }
        );
        assert_eq!(
            date_time(b"5 May 2003 18:59:03 +0000").unwrap().1,
            DateTime {
                day_name: None,
                date: Date {
                    day: 5,
                    month: Month::May,
                    year: 2003
                },
                time: TimeWithZone {
                    time: Time {
                        hour: 18,
                        minute: 59,
                        second: 3
                    },
                    zone: Zone {
                        sign: true,
                        hour_offset: 0,
                        minute_offset: 0
                    }
                }
            }
        );
    }

    #[test]
    fn test_time() {
        assert_eq!(
            time_of_day(b"10:40:29").unwrap().1,
            Time {
                hour: 10,
                minute: 40,
                second: 29
            }
        );
        assert_eq!(
            time_of_day(b"10:40 ").unwrap().1,
            Time {
                hour: 10,
                minute: 40,
                second: 0
            }
        );
        assert_eq!(
            time_of_day(b"05:23 ").unwrap().1,
            Time {
                hour: 5,
                minute: 23,
                second: 0
            }
        );

        assert_eq!(
            zone(b" +1000 ").unwrap().1,
            Zone {
                sign: true,
                hour_offset: 10,
                minute_offset: 0
            }
        );
        assert_eq!(
            zone(b" -0523 ").unwrap().1,
            Zone {
                sign: false,
                hour_offset: 5,
                minute_offset: 23
            }
        );

        assert_eq!(
            time(b"06:44 +0100").unwrap().1,
            TimeWithZone {
                time: Time {
                    hour: 6,
                    minute: 44,
                    second: 0
                },
                zone: Zone {
                    sign: true,
                    hour_offset: 1,
                    minute_offset: 0
                }
            }
        );
        assert_eq!(
            time(b"23:57 +0000").unwrap().1,
            TimeWithZone {
                time: Time {
                    hour: 23,
                    minute: 57,
                    second: 0
                },
                zone: Zone {
                    sign: true,
                    hour_offset: 0,
                    minute_offset: 0
                }
            }
        );
        assert_eq!(
            time(b"08:23:02 -0500").unwrap().1,
            TimeWithZone {
                time: Time {
                    hour: 8,
                    minute: 23,
                    second: 2
                },
                zone: Zone {
                    sign: false,
                    hour_offset: 5,
                    minute_offset: 0
                }
            }
        );
    }
}
