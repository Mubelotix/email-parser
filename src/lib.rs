use faster_pest::*;

#[derive(Parser)]
#[grammar = "src/address.pest"]
struct Parser {

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
    
    let input = "(  sdqf sqfdfsdqf sdqf sf  ) trail";
    let output = Parser::parse_comment(input).map_err(|e| e.print(input)).unwrap();
    let comment = output.into_iter().next().unwrap();
    assert_eq!(comment.as_str(), "(  sdqf sqfdfsdqf sdqf sf  )");
    let ctext_seqs = comment.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(ctext_seqs, vec!["sdqf", "sqfdfsdqf", "sdqf", "sf"]);

    let input = "(  level1 level1 (level2 level2  )  level1 ) trail";
    let output = Parser::parse_comment(input).map_err(|e| e.print(input)).unwrap();
    let comment = output.into_iter().next().unwrap();
    assert_eq!(comment.as_str(), "(  level1 level1 (level2 level2  )  level1 )");
    let ctext_seqs = comment.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(ctext_seqs, vec!["level1", "level1", "(level2 level2  )", "level1"]);

    for input in ["  \t  \t after", "   \r\n after", "\r\n after"] {
        let after = Parser_faster_pest::parse_FWS(input, &mut Vec::new()).unwrap();
        assert_eq!(after, "after");    
    }

    let input = "after";
    Parser_faster_pest::parse_FWS(input, &mut Vec::new()).unwrap_err();

    let input = "   \r\nafter";
    Parser_faster_pest::parse_FWS(input, &mut Vec::new()).unwrap_err();

    let input = "(  level1 level1  \r\n  level1 \r\n test \r\n ) trail";
    let output = Parser::parse_comment(input).map_err(|e| e.print(input)).unwrap();
    let comment = output.into_iter().next().unwrap();
    assert_eq!(comment.as_str(), "(  level1 level1  \r\n  level1 \r\n test \r\n )");
    let ctext_seqs = comment.children().map(|c| c.as_str()).collect::<Vec<_>>();
    assert_eq!(ctext_seqs, vec!["level1", "level1", "level1", "test"]);

    let input = "  ( cest un commentaire \r\n suivi d'un espace  )  \r\n trail";
    let after = Parser_faster_pest::parse_CFWS(input, &mut Vec::new()).unwrap();
    assert_eq!(after, "trail");
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
