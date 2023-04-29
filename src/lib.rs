use faster_pest::*;

#[derive(Parser)]
#[grammar = "src/address.pest"]
pub struct Parser {

}

#[test]
fn test_cfws() {
    let input = "sdqf sqfdfsdqf sdqf sf ";
    let output = Parser::parse_ctext_seq(input).map_err(|e| e.print(input)).unwrap();
    let ctext_seq = output.into_iter().next().unwrap().as_str();
    assert_eq!(ctext_seq, "sdqf");

    let input = "sd)qf sqfdfsdqf sdqf sf ";
    let output = Parser::parse_ctext_seq(input).map_err(|e| e.print(input)).unwrap();
    let ctext_seq = output.into_iter().next().unwrap().as_str();
    assert_eq!(ctext_seq, "sd");
    
    let input = "(  sdqf\\ sqfdfsdqf sdqf sf  ) trail";
    let output = Parser::parse_comment(input).map_err(|e| e.print(input)).unwrap();
    let comment = output.into_iter().next().unwrap();
    assert_eq!(comment.as_str(), "(  sdqf\\ sqfdfsdqf sdqf sf  )");
    let ctext_seqs = comment.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(ctext_seqs, vec!["sdqf", "\\ ", "sqfdfsdqf", "sdqf", "sf"]);

    let input = "(  level1 level1 (level2 level2  )  level1 ) trail";
    let output = Parser::parse_comment(input).map_err(|e| e.print(input)).unwrap();
    let comment = output.into_iter().next().unwrap();
    assert_eq!(comment.as_str(), "(  level1 level1 (level2 level2  )  level1 )");
    let ctext_seqs = comment.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(ctext_seqs, vec!["level1", "level1", "(level2 level2  )", "level1"]);

    for input in ["  \t  \t after", "   \r\n after", "\r\n after"] {
        let after = Parser_faster_pest::parse_FWS(input.as_bytes(), &mut Vec::new()).unwrap();
        assert_eq!(after, b"after");    
    }

    let input = "after";
    Parser_faster_pest::parse_FWS(input.as_bytes(), &mut Vec::new()).unwrap_err();

    let input = "   \r\nafter";
    Parser_faster_pest::parse_FWS(input.as_bytes(), &mut Vec::new()).unwrap_err();

    let input = "(  level1 level1  \r\n  level1 \r\n test \r\n ) trail";
    let output = Parser::parse_comment(input).map_err(|e| e.print(input)).unwrap();
    let comment = output.into_iter().next().unwrap();
    assert_eq!(comment.as_str(), "(  level1 level1  \r\n  level1 \r\n test \r\n )");
    let ctext_seqs = comment.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(ctext_seqs, vec!["level1", "level1", "level1", "test"]);

    let input = "  ( cest un commentaire \r\n suivi d'un espace  )  \r\n trail";
    let after = Parser_faster_pest::parse_CFWS(input.as_bytes(), &mut Vec::new()).unwrap();
    assert_eq!(after, b"trail");
}

#[test]
fn test() {
    let input = "rA9";
    let output = Parser::parse_atom(input).map_err(|e| e.print(input)).unwrap();
    let atom = output.into_iter().next().unwrap().as_str();
    assert_eq!(atom, "rA9");

    let input = "rA9   another";
    let output = Parser::parse_atom(input).map_err(|e| e.print(input)).unwrap();
    let atom = output.into_iter().next().unwrap().as_str();
    assert_eq!(atom, "rA9");

    let input = " rA9   one";
    let output = Parser::parse_atom(input).map_err(|e| e.print(input)).unwrap();
    let atom = output.into_iter().next().unwrap().as_str();
    assert_eq!(atom, "rA9");

    let input = "   bites.the.dust  ";
    let output = Parser::parse_dot_atom(input).map_err(|e| e.print(input)).unwrap();
    let dot_atom = output.into_iter().next().unwrap();
    assert_eq!(dot_atom.as_str(), "bites.the.dust");
    let atoms = dot_atom.children().map(|a| a.as_str()).collect::<Vec<_>>();
    assert_eq!(atoms, vec!["bites", "the", "dust"]);
}

#[test]
fn test_quoted_string() {
    let input = "  \" quoted string \" ";
    let output = Parser::parse_quoted_string(input).map_err(|e| e.print(input)).unwrap();
    let quoted_string = output.into_iter().next().unwrap();
    assert_eq!(quoted_string.as_str(), "  \" quoted string \" ");
    let qtext_seqs = quoted_string.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(qtext_seqs, vec!["quoted", "string"]);

    let input = "  \" quoted\\ string \" ";
    let output = Parser::parse_quoted_string(input).map_err(|e| e.print(input)).unwrap();
    let quoted_string = output.into_iter().next().unwrap();
    assert_eq!(quoted_string.as_str(), "  \" quoted\\ string \" ");
    let qtext_seqs = quoted_string.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(qtext_seqs, vec!["quoted", "\\ ", "string"]);

    let input = "  \" quoted\\ test \" \r\n test";
    let output = Parser::parse_quoted_string(input).map_err(|e| e.print(input)).unwrap();
    let quoted_string = output.into_iter().next().unwrap();
    assert_eq!(quoted_string.as_str(), "  \" quoted\\ test \" \r\n ");
    let qtext_seqs = quoted_string.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(qtext_seqs, vec!["quoted", "\\ ", "test"]);
}

#[test]
fn test_date() {
    let input = "Mon, 5 May 2003 18:58:55 +0100 trail";
    let output = Parser::parse_date_time(input).map_err(|e| e.print(input)).unwrap();
    let date_time = output.into_iter().next().unwrap();
    let children = date_time.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(children, vec!["Mon", "5", "May", "2003", "18", "58", "55", "+0100"]);

    let input = "3 Jan 2009 18:15:05 +0000";
    let output = Parser::parse_date_time(input).map_err(|e| e.print(input)).unwrap();
    let date_time = output.into_iter().next().unwrap();
    let children = date_time.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(children, vec!["3", "Jan", "2009", "18", "15", "05", "+0000"]);

    let input = "14 Jul 1789   14:00:00 +0100";
    let output = Parser::parse_date_time(input).map_err(|e| e.print(input)).unwrap();
    let date_time = output.into_iter().next().unwrap();
    let children = date_time.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(children, vec!["14", "Jul", "1789", "14", "00", "00", "+0100"]);
}

#[test]
fn test_addr() {
    let input = "Mubelotix <mubelotix@gmail.com>";
    let output = Parser::parse_mailbox(input).map_err(|e| e.print(input)).unwrap();
    let mailbox = output.into_iter().next().unwrap();
    let children = mailbox.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(children, vec!["Mubelotix ", "mubelotix@gmail.com"]);

    let input = "Cypherpunks: Satoshi Nakamoto <satoshin@gmx.com>, Hal Finney <hal@finney.org>;";
    let output = Parser::parse_group(input).map_err(|e| e.print(input)).unwrap();
    let mut group = output.into_iter().next().unwrap().children();
    let display_name = group.next().unwrap().as_str();
    assert_eq!(display_name, "Cypherpunks");
    let mailbox_list = group.next().unwrap();
    let mailboxes = mailbox_list.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(mailboxes, vec![" Satoshi Nakamoto <satoshin@gmx.com>", " Hal Finney <hal@finney.org>"]);
}

#[test]
fn test_addr_spec() {
    let input = "hal@finney.org";
    let output = Parser::parse_addr_spec(input).map_err(|e| e.print(input)).unwrap();
    let addr_spec = output.into_iter().next().unwrap();
    let children = addr_spec.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(children, vec!["hal", "finney.org"]);

    let input = "mubelotix@[192.168.1.1] ";
    let output = Parser::parse_addr_spec(input).map_err(|e| e.print(input)).unwrap();
    let addr_spec = output.into_iter().next().unwrap();
    let children = addr_spec.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(children, vec!["mubelotix", "[192.168.1.1]"]);
}

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
fn test_message() {
    let input = include_str!("../mail.txt");
    let output = Parser::parse_message(input).map_err(|e| e.print(input)).unwrap();
    let message = output.into_iter().next().unwrap();
    println!("{:#?}", message);
    assert_eq!(format!("{:?}", message), r#"message { text: "X-COM: 2\r\n\r\nbody", children: [unknown_field { text: "X-COM: 2\r\n", children: [field_name { text: "X-COM", children: [] }, unstructured { text: " 2", children: [vchar_seq { text: "2", children: [] }] }] }, body { text: "body", children: [text_seq { text: "body", children: [] }] }] }"#);
}
