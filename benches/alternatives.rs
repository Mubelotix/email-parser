#![feature(test)]

extern crate test;
use test::Bencher;

const MAIL: &[u8] = include_bytes!("../mail.txt");
const MAIL2: &str = include_str!("../mail.txt");

#[bench]
fn email_parser2(b: &mut Bencher) {
    b.iter(|| email_parser2::Email::parse(MAIL2));
}

#[bench]
fn email_parser(b: &mut Bencher) {
    b.iter(|| email_parser::email::Email::parse(MAIL));
}
