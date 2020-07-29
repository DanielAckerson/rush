use unicode_segmentation::UnicodeSegmentation;

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum Token {
    Path(String),
    Arg(String),
    Text(String),
    Whitespace(String),
    Escape,
    DoubleQuote,
    SingleQuote,
    EOL,
}


impl Token {
    pub fn to_text(&self) -> Result<Token, String> {
        use Token::*;
        return match self {
            Text(text) | Whitespace(text) | Path(text) | Arg(text) => Ok(Text(text.clone())),
            Escape => Ok(Text("\\".to_owned())),
            DoubleQuote => Ok(Text("\"".to_owned())),
            SingleQuote => Ok(Text("'".to_owned())),
            _ => Err("couldn't convert to Token::Text".to_owned()),
        }
    }

    pub fn to_path(&self) -> Result<Token, String> {
        use Token::*;
        return match self {
            Text(path) | Path(path) | Arg(path) => Ok(Path(path.clone())),
            _ => Err("couldn't convert to Token::Path".to_owned()),
        }
    }

    pub fn to_arg(&self) -> Result<Token, String> {
        use Token::*;
        return match self {
            Text(arg) | Path(arg) | Arg(arg) => Ok(Arg(arg.clone())),
            _ => Err("couldn't convert to Token::Arg".to_owned()),
        }
    }

    pub fn unwrap(&self) -> String {
        use Token::*;
        return match self {
            Text(text) | Whitespace(text) | Path(text) | Arg(text) => text.clone(),
            SingleQuote => "\'".to_owned(),
            DoubleQuote => "\"".to_owned(),
            Escape => "\\".to_owned(),
            _ => "".to_owned(),
        }
    }
}


lazy_static! {
    static ref LEXICON: HashMap<&'static str, Token> = {
        use Token::*;
        let mut m = HashMap::new();
        m.insert("\"", DoubleQuote);
        m.insert("\'", SingleQuote);
        m.insert("\\", Escape);
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

    loop {
        if let Some((i_cur, g_cur)) = cur.next() {
            if let Some(t) = LEXICON.get(g_cur) {
                let (i_start, _) = start
                    .next()
                    .expect("lexer: start iterator had no next value");

                let text = expr[i_start..i_cur].to_owned();

                if !text.is_empty() {
                    tokens.push(Text(text));
                }

                tokens.push(t.clone());
                start = cur.clone();
            }
        } else if let Some((i, _)) = start.next() {
            tokens.push(Text(expr[i..].to_owned()));
            return Some(tokens);
        } else if !tokens.is_empty() {
            return Some(tokens);
        } else {
            return None;
        }
    }
}


// TODO: create error types

// TODO: instead of working directly with path and args after parsing, generate a 
// graph of tasks to execute and then execute them

pub fn parse(expr: &str) -> Result<Vec<Token>, String> {
    use Token::*;
    let mut tokens: Vec<Token> = Vec::new();
    let tokens_raw = lexer(expr).ok_or("no tokens")?; // TODO: use better name than raw
    let mut t_raw = tokens_raw.into_iter();

    // assume token is Text
    if let Some(token) = t_raw.next() {
        tokens.push(token.to_path()?.clone());
    }

    let mut arg_text = String::new();
    loop {
        match t_raw.next() {
            Some(Text(text)) => arg_text.push_str(&text),
            Some(DoubleQuote) => if let Text(text) = parse_double_quoted(&mut t_raw)? {
                arg_text.push_str(&text);
            },
            Some(SingleQuote) => if let Text(text) = parse_single_quoted(&mut t_raw)? {
                arg_text.push_str(&text);
            },
            Some(Whitespace(text)) => if !arg_text.is_empty() {
                tokens.push(Arg(arg_text.clone()));
                arg_text.clear();
            },
            None => {
                if !arg_text.is_empty() {
                    tokens.push(Arg(arg_text.clone()));
                    arg_text.clear();
                }
                break;
            },
            _ => {},
        }
    }

    if !tokens.is_empty() {
        Ok(tokens)
    } else {
        Err("unknown parser error".to_owned())
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
        Whitespace(ref c) if c == "\n" => token.to_text(),
        Whitespace(_) => token.to_text(),
        Escape | DoubleQuote | SingleQuote => token.to_text(),
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
    loop {
        let token = tokens.next().ok_or("no matching double quote")?;
        match token {
            Text(text) | Whitespace(text) => quoted_text.push_str(&text),
            Escape => match parse_escaped(tokens)? {
                Text(escaped) => quoted_text.push_str(&escaped),
                _ => return Err("parse_escaped didn't return Text".to_owned()),
            },
            SingleQuote => quoted_text.push('\''),
            DoubleQuote => return Ok(Text(quoted_text)),
            x => return Err(format!("invalid token in double quote: {:?}", x)),
        }
    }
}


// expand all tokens to Text until single quote reached
fn parse_single_quoted<I>(tokens: &mut I) -> Result<Token, String>
where
    I: Iterator<Item = Token>
{
    use Token::*;
    let mut quoted_text = String::new();
    loop {
        let token = tokens.next().ok_or("no matching single quote")?;
        match token {
            Text(text) | Whitespace(text) => quoted_text.push_str(&text),
            Escape => match parse_escaped(tokens)? {
                Text(escaped) => quoted_text.push_str(&escaped),
                _ => return Err("parse_escaped didn't return Text".to_owned()),
            },
            DoubleQuote => quoted_text.push('\"'),
            SingleQuote => return Ok(Text(quoted_text)),
            x => return Err(format!("invalid token in double quote: {:?}", x)),
        }
    }
}
