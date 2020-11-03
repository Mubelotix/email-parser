#![feature(test)]

extern crate test;
use email_parser::parser::*;
use test::Bencher;

const MAIL: &[u8] = include_bytes!("../mail.txt");
const MAIL2: &str = include_str!("../mail.txt");

#[bench]
fn email_parser(b: &mut Bencher) {
    b.iter(|| parse_message(MAIL));
}

#[bench]
fn email(b: &mut Bencher) {
    b.iter(|| email::rfc5322::Rfc5322Parser::new(MAIL2).consume_message());
}

#[bench]
fn email_format(b: &mut Bencher) {
    use email_format::rfc5322::Parsable;
    use email_format::Email;
    b.iter(|| Email::parse(MAIL));
}

#[bench]
fn mailparse(b: &mut Bencher) {
    b.iter(|| mailparse::parse_mail(MAIL));
}
