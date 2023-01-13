use faster_pest::*;

#[derive(Parser)]
#[grammar = "src/address.pest"]
struct Parser {

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
