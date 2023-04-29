use email_parser2::*;

#[test]
fn test_message() {
    let input = include_str!("../mail.txt");
    let output = Parser::parse_message(input).map_err(|e| e.print(input)).unwrap();
    let message = output.into_iter().next().unwrap();
    println!("{:#?}", message);
    println!("{} idents", message.children_count());
    assert_eq!(format!("{:?}", message), r#"message { text: "X-COM: 2\r\n\r\nbody", children: [unknown_field { text: "X-COM: 2\r\n", children: [field_name { text: "X-COM", children: [] }, unstructured { text: " 2", children: [vchar_seq { text: "2", children: [] }] }] }, body { text: "body", children: [text_seq { text: "body", children: [] }] }] }"#);
}
