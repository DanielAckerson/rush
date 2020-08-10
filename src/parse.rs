use unicode_segmentation::UnicodeSegmentation;

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Path(String),
    Arg(String),
    Text(String),
    Whitespace(String),
    Escape(String),
    DoubleQuote(String),
    SingleQuote(String),
}


impl Token {
    pub fn unwrap(&self) -> String {
        use Token::*;
        return match self {
            Text(text) 
                | Whitespace(text) | Path(text) | Arg(text) 
                | SingleQuote(text) | DoubleQuote(text) | Escape(text)
                => text.clone(),
        }
    }
}


lazy_static! {
    static ref LEXICON: HashMap<&'static str, Token> = {
        use Token::*;
        let mut m = HashMap::new();
        m.insert("\"", DoubleQuote("\"".to_owned()));
        m.insert("\'", SingleQuote("\'".to_owned()));
        m.insert("\\", Escape("\\".to_owned()));
        m.insert(" ",  Whitespace(" ".to_owned()));
        m.insert("\t", Whitespace("\t".to_owned()));
        m.insert("\n", Whitespace("\n".to_owned()));
        m.insert("\r", Whitespace("\r".to_owned()));
        m
    };
}


pub fn lexer(expr: &str) -> Option<Vec<Token>> {
    use Token::*;
        
    let mut tokens: Vec<Token> = Vec::new();
    let mut start = UnicodeSegmentation::grapheme_indices(expr, true);
    let mut cur = start.clone();

    while let Some((i_cur, g_cur)) = cur.next() {
        if let Some(token) = LEXICON.get(g_cur) {
            let (i_start, _) = start
                .next()
                .expect("lexer: start iterator had no next value");

            let text = expr[i_start..i_cur].to_owned();

            if !text.is_empty() {
                tokens.push(Text(text));
            }

            tokens.push(token.clone());
            start = cur.clone();
        }
    }

    if let Some((i, _)) = start.next() {
        tokens.push(Text(expr[i..].to_owned()));
    }

    if tokens.is_empty() {
        None
    } else {
        Some(tokens)
    }
}


// TODO: create error types

// TODO: instead of working directly with path and args after parsing, generate a 
// graph of tasks to execute and then execute them

pub fn parse(expr: &str) -> Result<Vec<Token>, String> {
    use Token::*;

    let mut tokens: Vec<Token> = Vec::new();
    let mut arg_text = String::new();
    let tokens_raw = lexer(expr).ok_or("no tokens")?; // TODO: use better name than raw
    let mut t_raw = tokens_raw.into_iter();

    if let Some(token) = t_raw.next() {
        match token {
            Text(text) => tokens.push(Path(text)),
            _ => return Err("parser error: first token isn't Text".to_owned()),
        }
    }

    while let Some(token) = t_raw.next() {
        match token {
            Text(text) => arg_text.push_str(&text),
            Escape(_) => {
                let token = parse_escaped(&mut t_raw)?;
                arg_text.push_str(&token.unwrap());
            },
            DoubleQuote(_) => {
                let token = parse_double_quoted(&mut t_raw)?;
                arg_text.push_str(&token.unwrap());
            },
            SingleQuote(_) => {
                let token = parse_single_quoted(&mut t_raw)?;
                arg_text.push_str(&token.unwrap());
            },
            Whitespace(_) => if !arg_text.is_empty() {
                tokens.push(Arg(arg_text.clone()));
                arg_text.clear();
            },
            _ => return Err("parser error: unexpected token".to_owned()),
        }
    }

    if !arg_text.is_empty() {
        tokens.push(Arg(arg_text.clone()));
        arg_text.clear();
    }

    if tokens.is_empty() {
        Err("parser error: no tokens".to_owned())
    } else {
        Ok(tokens)
    }
}


// TODO: should return type be Result<String, String> instead?
fn parse_escaped<I>(tokens: &mut I) -> Result<Token, String>
where
    I: Iterator<Item = Token>
{
    use Token::*;
    let token = tokens.next().ok_or("unexpected EOL")?;
    match token {
        Whitespace(text)
            | Escape(text) | DoubleQuote(text) | SingleQuote(text)
            => Ok(Text(text)),
        x => Err(format!("unexpected token {:?}", x)),
    }
}


// expand all tokens to Text until double quote reached
fn parse_double_quoted<I>(tokens: &mut I) -> Result<Token, String>
where
    I: Iterator<Item = Token>
{
    use Token::*;

    let mut quoted_text = String::new();
    while let Some(token) = tokens.next() {
        match token {
            Text(text) | Whitespace(text) | SingleQuote(text)
                => quoted_text.push_str(&text),
            Escape(_) => {
                let token = parse_escaped(tokens)?;
                quoted_text.push_str(&token.unwrap());
            },
            DoubleQuote(_) => return Ok(Text(quoted_text)),
            x => return Err(format!("invalid token in double quote: {:?}", x)),
        }
    }
    Err("no matching double quote".to_owned())
}


// expand all tokens to Text until single quote reached
fn parse_single_quoted<I>(tokens: &mut I) -> Result<Token, String>
where
    I: Iterator<Item = Token>
{
    use Token::*;

    let mut quoted_text = String::new();
    while let Some(token) = tokens.next() {
        match token {
            Text(text) | Whitespace(text) | DoubleQuote(text)
                => quoted_text.push_str(&text),
            Escape(_) => {
                let token = parse_escaped(tokens)?;
                quoted_text.push_str(&token.unwrap());
            },
            SingleQuote(_) => return Ok(Text(quoted_text)),
            x => return Err(format!("invalid token in double quote: {:?}", x)),
        }
    }
    Err("no matching single quote".to_owned())
}


#[test]
fn test_lexer() {
    use Token::*;
    assert_eq!(
        lexer("echo \"This is a \'test\'. I repeat, a \\\"TEST\\\"\" \'See?\'"),
        Some(vec![
            Text("echo".to_owned()), Whitespace(" ".to_owned()),
            DoubleQuote("\"".to_owned()),
                Text("This".to_owned()), Whitespace(" ".to_owned()),
                Text("is".to_owned()), Whitespace(" ".to_owned()),
                Text("a".to_owned()), Whitespace(" ".to_owned()),
                SingleQuote("\'".to_owned()),
                    Text("test".to_owned()),
                SingleQuote("\'".to_owned()),
                Text(".".to_owned()), Whitespace(" ".to_owned()),
                Text("I".to_owned()), Whitespace(" ".to_owned()),
                Text("repeat,".to_owned()), Whitespace(" ".to_owned()),
                Text("a".to_owned()), Whitespace(" ".to_owned()),
                Escape("\\".to_owned()), DoubleQuote("\"".to_owned()),
                    Text("TEST".to_owned()),
                Escape("\\".to_owned()), DoubleQuote("\"".to_owned()),
            DoubleQuote("\"".to_owned()),
            Whitespace(" ".to_owned()),
            SingleQuote("\'".to_owned()),
                Text("See?".to_owned()),
            SingleQuote("\'".to_owned()),
    ]));
}


#[test]
fn test_parser() {
    use Token::*;
    assert_eq!(
        parse("echo \"This is a \'test\'. I repeat, a \\\"TEST\\\"\" \'See?\'"),
        Ok(vec![
            Path("echo".to_owned()),
            Arg("This is a \'test\'. I repeat, a \"TEST\"".to_owned()),
            Arg("See?".to_owned()),
    ]));
}
