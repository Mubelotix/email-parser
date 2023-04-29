use email_parser2::*;

#[test]
fn test_unstructured_field() {
    let input = "Value: This is a test\r\n";
    let output = Parser::parse_unknown_field(input).map_err(|e| e.print(input)).unwrap();
    let field = output.into_iter().next().unwrap();
    let children = field.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(children, vec!["Value", " This is a test"]);

    let input = "Value: This is a test\r\n test\r\ntest";
    let output = Parser::parse_unknown_field(input).map_err(|e| e.print(input)).unwrap();
    let field = output.into_iter().next().unwrap();
    let children = field.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(children, vec!["Value", " This is a test\r\n test"]);
}

#[test]
fn test_origin_fields() {
    let input = "Date: Mon, 5 May 2003 18:58:55 +0100\r\n";
    let output = Parser::parse_origination_date(input).map_err(|e| e.print(input)).unwrap();
    let field = output.into_iter().next().unwrap();
    let children = field.children().collect::<Vec<_>>();
    assert_eq!(format!("{children:?}"), "[date_time { text: \" Mon, 5 May 2003 18:58:55 +0100\", children: [day_name { text: \"Mon\" }, day { text: \"5\" }, month { text: \"May\" }, year { text: \"2003\" }, hour { text: \"18\" }, minute { text: \"58\" }, second { text: \"55\" }, zone { text: \"+0100\" }] }]");

    let input = "From: \"Andreas M. Antonopoulos\" <contact@aantonop.com>, mubelotix@gmail.com\r\n";
    let output = Parser::parse_from(input).map_err(|e| e.print(input)).unwrap();
    let field = output.into_iter().next().unwrap();
    let children = field.children().collect::<Vec<_>>();
    assert_eq!(format!("{children:?}"), "[mailbox_list { text: \" \\\"Andreas M. Antonopoulos\\\" <contact@aantonop.com>, mubelotix@gmail.com\", children: [mailbox { text: \" \\\"Andreas M. Antonopoulos\\\" <contact@aantonop.com>\", children: [display_name { text: \" \\\"Andreas M. Antonopoulos\\\" \", children: [phrase { text: \" \\\"Andreas M. Antonopoulos\\\" \", children: [word { text: \" \\\"Andreas M. Antonopoulos\\\" \", children: [quoted_string { text: \" \\\"Andreas M. Antonopoulos\\\" \", children: [qtext_seq { text: \"Andreas\" }, WSP_seq2 { text: \" \" }, qtext_seq { text: \"M.\" }, WSP_seq2 { text: \" \" }, qtext_seq { text: \"Antonopoulos\" }] }] }] }] }, addr_spec { text: \"contact@aantonop.com\", children: [dot_atom_text { text: \"contact\", children: [atext_seq { text: \"contact\" }] }, dot_atom_text { text: \"aantonop.com\", children: [atext_seq { text: \"aantonop\" }, atext_seq { text: \"com\" }] }] }] }, mailbox { text: \" mubelotix@gmail.com\", children: [addr_spec { text: \" mubelotix@gmail.com\", children: [dot_atom_text { text: \"mubelotix\", children: [atext_seq { text: \"mubelotix\" }] }, dot_atom_text { text: \"gmail.com\", children: [atext_seq { text: \"gmail\" }, atext_seq { text: \"com\" }] }] }] }] }]");

    let input = "Sender: \"Chloe Helloco\" <chloe.helloco@ac-orleans-tours.fr>\r\n";
    let output = Parser::parse_sender(input).map_err(|e| e.print(input)).unwrap();
    let field = output.into_iter().next().unwrap();
    let children = field.children().collect::<Vec<_>>();
    assert_eq!(format!("{children:?}"), "[mailbox { text: \" \\\"Chloe Helloco\\\" <chloe.helloco@ac-orleans-tours.fr>\", children: [display_name { text: \" \\\"Chloe Helloco\\\" \", children: [phrase { text: \" \\\"Chloe Helloco\\\" \", children: [word { text: \" \\\"Chloe Helloco\\\" \", children: [quoted_string { text: \" \\\"Chloe Helloco\\\" \", children: [qtext_seq { text: \"Chloe\" }, WSP_seq2 { text: \" \" }, qtext_seq { text: \"Helloco\" }] }] }] }] }, addr_spec { text: \"chloe.helloco@ac-orleans-tours.fr\", children: [dot_atom_text { text: \"chloe.helloco\", children: [atext_seq { text: \"chloe\" }, atext_seq { text: \"helloco\" }] }, dot_atom_text { text: \"ac-orleans-tours.fr\", children: [atext_seq { text: \"ac-orleans-tours\" }, atext_seq { text: \"fr\" }] }] }] }]");

    let input = "ReplY-To: thevoid@4chan.org\r\n";
    let output = Parser::parse_reply_to(input).map_err(|e| e.print(input)).unwrap();
    let field = output.into_iter().next().unwrap();
    let children = field.children().collect::<Vec<_>>();
    assert_eq!(format!("{children:?}"), "[address_list { text: \" thevoid@4chan.org\", children: [address { text: \" thevoid@4chan.org\", children: [mailbox { text: \" thevoid@4chan.org\", children: [addr_spec { text: \" thevoid@4chan.org\", children: [dot_atom_text { text: \"thevoid\", children: [atext_seq { text: \"thevoid\" }] }, dot_atom_text { text: \"4chan.org\", children: [atext_seq { text: \"4chan\" }, atext_seq { text: \"org\" }] }] }] }] }] }]");
}

#[test]
fn test_destination_fields() {
    let input = "To: \"John Wick\" <john.wick@cock.li>\r\n";
    let output = Parser::parse_to(input).map_err(|e| e.print(input)).unwrap();
    let field = output.into_iter().next().unwrap();
    let children = field.children().collect::<Vec<_>>();
    assert_eq!(format!("{children:?}"), "[address_list { text: \" \\\"John Wick\\\" <john.wick@cock.li>\", children: [address { text: \" \\\"John Wick\\\" <john.wick@cock.li>\", children: [mailbox { text: \" \\\"John Wick\\\" <john.wick@cock.li>\", children: [display_name { text: \" \\\"John Wick\\\" \", children: [phrase { text: \" \\\"John Wick\\\" \", children: [word { text: \" \\\"John Wick\\\" \", children: [quoted_string { text: \" \\\"John Wick\\\" \", children: [qtext_seq { text: \"John\" }, WSP_seq2 { text: \" \" }, qtext_seq { text: \"Wick\" }] }] }] }] }, addr_spec { text: \"john.wick@cock.li\", children: [dot_atom_text { text: \"john.wick\", children: [atext_seq { text: \"john\" }, atext_seq { text: \"wick\" }] }, dot_atom_text { text: \"cock.li\", children: [atext_seq { text: \"cock\" }, atext_seq { text: \"li\" }] }] }] }] }] }]");

    let input = "Cc: postmaster@hey.com\r\n";
    let output = Parser::parse_cc(input).map_err(|e| e.print(input)).unwrap();
    let field = output.into_iter().next().unwrap();
    let children = field.children().collect::<Vec<_>>();
    assert_eq!(format!("{children:?}"), "[address_list { text: \" postmaster@hey.com\", children: [address { text: \" postmaster@hey.com\", children: [mailbox { text: \" postmaster@hey.com\", children: [addr_spec { text: \" postmaster@hey.com\", children: [dot_atom_text { text: \"postmaster\", children: [atext_seq { text: \"postmaster\" }] }, dot_atom_text { text: \"hey.com\", children: [atext_seq { text: \"hey\" }, atext_seq { text: \"com\" }] }] }] }] }] }]");

    // TODO
    /*let input = "Bcc: \r\n";
    let output = Parser::parse_bcc(input).map_err(|e| e.print(input)).unwrap();
    let field = output.into_iter().next().unwrap();
    let children = field.children().collect::<Vec<_>>();
    assert_eq!(format!("{children:?}"), "");*/
}

#[test]
fn test_msg_id_fields() {
    let input = "Message-ID: <sadqf54d@test.com>\r\n";
    let output = Parser::parse_message_id(input).map_err(|e| e.print(input)).unwrap();
    let field = output.into_iter().next().unwrap();
    let children = field.children().collect::<Vec<_>>();
    assert_eq!(format!("{children:?}"), "[msg_id { text: \" <sadqf54d@test.com>\", children: [id_left { text: \"sadqf54d\", children: [dot_atom_text { text: \"sadqf54d\", children: [atext_seq { text: \"sadqf54d\" }] }] }, id_right { text: \"test.com\", children: [dot_atom_text { text: \"test.com\", children: [atext_seq { text: \"test\" }, atext_seq { text: \"com\" }] }] }] }]");

    let input = "In-Reply-To: <sqdfsf@test.com> <47@[127.0.0.1]>\r\n";
    let output = Parser::parse_in_reply_to(input).map_err(|e| e.print(input)).unwrap();
    let field = output.into_iter().next().unwrap();
    let children = field.children().collect::<Vec<_>>();
    assert_eq!(format!("{children:?}"), "[msg_id { text: \" <sqdfsf@test.com> \", children: [id_left { text: \"sqdfsf\", children: [dot_atom_text { text: \"sqdfsf\", children: [atext_seq { text: \"sqdfsf\" }] }] }, id_right { text: \"test.com\", children: [dot_atom_text { text: \"test.com\", children: [atext_seq { text: \"test\" }, atext_seq { text: \"com\" }] }] }] }, msg_id { text: \"<47@[127.0.0.1]>\", children: [id_left { text: \"47\", children: [dot_atom_text { text: \"47\", children: [atext_seq { text: \"47\" }] }] }, id_right { text: \"[127.0.0.1]\", children: [no_fold_literal { text: \"[127.0.0.1]\" }] }] }]");

    let input = "References: <sqdfsf@test.com> <47@[127.0.0.1]>\r\n";
    let output = Parser::parse_references(input).map_err(|e| e.print(input)).unwrap();
    let field = output.into_iter().next().unwrap();
    let children = field.children().collect::<Vec<_>>();
    assert_eq!(format!("{children:?}"), "[msg_id { text: \" <sqdfsf@test.com> \", children: [id_left { text: \"sqdfsf\", children: [dot_atom_text { text: \"sqdfsf\", children: [atext_seq { text: \"sqdfsf\" }] }] }, id_right { text: \"test.com\", children: [dot_atom_text { text: \"test.com\", children: [atext_seq { text: \"test\" }, atext_seq { text: \"com\" }] }] }] }, msg_id { text: \"<47@[127.0.0.1]>\", children: [id_left { text: \"47\", children: [dot_atom_text { text: \"47\", children: [atext_seq { text: \"47\" }] }] }, id_right { text: \"[127.0.0.1]\", children: [no_fold_literal { text: \"[127.0.0.1]\" }] }] }]");
}

#[test]
fn test_informational_fields() {
    let input = "Subject: Hello world!\r\n";
    let output = Parser::parse_subject(input).map_err(|e| e.print(input)).unwrap();
    let field = output.into_iter().next().unwrap();
    let children = field.children().collect::<Vec<_>>();
    assert_eq!(format!("{children:?}"), "[unstructured { text: \" Hello world!\", children: [WSP_seq2 { text: \" \" }, vchar_seq { text: \"Hello\" }, WSP_seq2 { text: \" \" }, vchar_seq { text: \"world!\" }] }]");

    let input = "Comments: Hello world!\r\n";
    let output = Parser::parse_comments(input).map_err(|e| e.print(input)).unwrap();
    let field = output.into_iter().next().unwrap();
    let children = field.children().collect::<Vec<_>>();
    assert_eq!(format!("{children:?}"), "[unstructured { text: \" Hello world!\", children: [WSP_seq2 { text: \" \" }, vchar_seq { text: \"Hello\" }, WSP_seq2 { text: \" \" }, vchar_seq { text: \"world!\" }] }]");

    let input = "Keywords: bitcoin is king\r\n";
    let output = Parser::parse_keywords(input).map_err(|e| e.print(input)).unwrap();
    let field = output.into_iter().next().unwrap();
    let children = field.children().collect::<Vec<_>>();
    assert_eq!(format!("{children:?}"), "[phrase { text: \" bitcoin is king\", children: [word { text: \" bitcoin \", children: [WSP_seq2 { text: \" \" }, atext_seq { text: \"bitcoin\" }, WSP_seq2 { text: \" \" }] }, word { text: \"is \", children: [atext_seq { text: \"is\" }, WSP_seq2 { text: \" \" }] }, word { text: \"king\", children: [atext_seq { text: \"king\" }] }] }]");
}
