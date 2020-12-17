#![feature(test)]

extern crate test;
use test::Bencher;

#[cfg(feature = "benchmarking")]
#[bench]
fn base64_decoding(b: &mut Bencher) {
    b.iter(|| email_parser::prelude::decode_base64(b"TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQsIGNvbnNlY3RldHVyIGFkaXBpc2NpbmcgZWxpdCwgc2VkIGRvIGVpdXNtb2QgdGVtcG9yIGluY2lkaWR1bnQgdXQgbGFib3JlIGV0IGRvbG9yZSBtYWduYSBhbGlxdWEuIFV0IGVuaW0gYWQgbWluaW0gdmVuaWFtLCBxdWlzIG5vc3RydWQgZXhlcmNpdGF0aW9uIHVsbGFtY28gbGFib3JpcyBuaXNpIHV0IGFsaXF1aXAgZXggZWEgY29tbW9kbyBjb25zZXF1YXQuIER1aXMgYXV0ZSBpcnVyZSBkb2xvciBpbiByZXByZWhlbmRlcml0IGluIHZvbHVwdGF0ZSB2ZWxpdCBlc3NlIGNpbGx1bSBkb2xvcmUgZXUgZnVnaWF0IG51bGxhIHBhcmlhdHVyLiBFeGNlcHRldXIgc2ludCBvY2NhZWNhdCBjdXBpZGF0YXQgbm9uIHByb2lkZW50LCBzdW50IGluIGN1bHBhIHF1aSBvZmZpY2lhIGRlc2VydW50IG1vbGxpdCBhbmltIGlkIGVzdCBsYWJvcnVtLg==".to_vec()));
}

#[cfg(feature = "benchmarking")]
#[bench]
fn quoted_printables_decoding(b: &mut Bencher) {
    b.iter(|| email_parser::prelude::decode_qp(br#"<html>\r\n<head>\r\n<meta http-equiv=3D"Content-Type" content=3D"text/html; charset=3Diso-8859-=\r\n1">\r\n<style type=3D"text/css" style=3D"display:none;"> P {margin-top:0;margin-bo=\r\nttom:0;} </style>\r\n</head>
    <body dir=3D"ltr">\r\n<div style=3D"font-family: Calibri, Arial, Helvetica, sans-serif; font-size=\r\n: 12pt; color: rgb(0, 0, 0);">\r\nqzdzq</div>\r\n</body>\r\n</html>"#.to_vec()));
}
