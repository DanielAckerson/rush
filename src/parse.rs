use pom::parser::*;

use std::collections::HashMap;

// until more complex grammar is supported, only return (path, args)
pub fn parse(input: &str, vars: &HashMap<String, String>) -> Result<(String, Vec<String>), String> {
    program(&vars)
        .parse(input.as_bytes())
        .map_err(|_| "parse error".to_string())
}


fn whitespace<'a>() -> Parser<'a, u8, ()> {
    one_of(b" \t\r\n")
        .repeat(0..)
        .discard()
}


fn var_char<'a>() -> Parser<'a, u8, u8> {
    is_a(|x: u8| x.is_ascii_alphabetic())
}


fn escape_sequence<'a>() -> Parser<'a, u8, u8> {
    let special_char = one_of(b" $\'\"\\");

    sym(b'\\') * special_char
}


fn expand_var<'a>(dict: &'a HashMap<String, String>) -> Parser<'a, u8, String> {
    let identifier = var_char()
        .repeat(1..)
        .convert(String::from_utf8);

    let var = sym(b'$') * identifier;

    var.convert(move |key: String| match dict.get(&key) {
        Some(value) => Ok(value.to_owned()),
        None => Err("unkown variable".to_string()),
    })
}


fn text<'a>(dict: &'a HashMap<String, String>) -> Parser<'a, u8, String> {
    let text_char = escape_sequence() | none_of(b" \t\r\n$\'\"");

    let chars_as_string = text_char
        .repeat(1..)
        .convert(String::from_utf8);

    let text_as_string = expand_var(dict) | chars_as_string;

    text_as_string
        .repeat(1..)
        .map(|strings| strings.concat())
}


fn string<'a>(dict: &'a HashMap<String, String>) -> Parser<'a, u8, String> {
    let string_char_exclude = |x: &'static [u8]| escape_sequence() | none_of(x);

    let chars_as_string = |x: &'static [u8]| string_char_exclude(x)
        .repeat(1..)
        .convert(String::from_utf8);

    let partial_string = |x: &'static [u8]| expand_var(dict) | chars_as_string(x);

    let string = |x: &'static [u8]| partial_string(x)
        .repeat(1..)
        .map(|strings| strings.concat());

    let quoted = sym(b'\'') * string(b"\'") - sym(b'\'');
    let double_quoted = sym(b'\"') * string(b"$\"") - sym(b'\"');

    quoted | double_quoted
}


fn program<'a>(dict: &'a HashMap<String, String>) -> Parser<'a, u8, (String, Vec<String>)> {
    let token = || text(dict) | string(dict);

    let tokens = || token()
        .repeat(1..)
        .map(|strings| strings.concat());

    let spaces = sym(b' ')
        .repeat(1..)
        .discard();

    let path = whitespace() * tokens();
    let arg = spaces * tokens();

    path + arg.repeat(0..)
}


#[test]
fn parse_test() {
    let input = r#"echo "This is a 'test'. I repeat, a \"TEST\"" 'See?' Check\ it\ out!"#;
    let vars: HashMap<String, String> = HashMap::new();

    assert_eq!(
        Ok((
            "echo".to_string(),
            vec![
                r#"This is a 'test'. I repeat, a "TEST""#.to_string(),
                "See?".to_string(),
                "Check it out!".to_string()])),
        parse(input, &vars))
}
