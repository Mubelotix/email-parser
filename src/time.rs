#[derive(Debug, PartialEq)]
pub enum Day {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
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

#[derive(Debug, PartialEq)]
pub struct Zone {
    pub sign: bool,
    pub hour_offset: u8,
    pub minute_offset: u8,
}

#[derive(Debug, PartialEq)]
pub struct Time {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

#[derive(Debug, PartialEq)]
pub struct TimeWithZone {
    pub time: Time,
    pub zone: Zone,
}

#[derive(Debug, PartialEq)]
pub struct Date {
    pub day: u8,
    pub month: Month,
    pub year: usize,
}

#[derive(Debug, PartialEq)]
pub struct DateTime {
    pub day_name: Option<Day>,
    pub date: Date,
    pub time: TimeWithZone,
}
