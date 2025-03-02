use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub text: String,
    pub begin: usize,
    pub end: usize,
    pub quoted: bool,
}

impl Token {
    fn new(text: &str, begin: usize, end: usize, quoted: bool) -> Self {
        Self {
            text: text.into(),
            begin,
            end,
            quoted,
        }
    }
}

pub fn tokenize(line: &str) -> Option<VecDeque<Token>> {
    enum Status {
        Idle,
        Token,
        Quote(char),
    }

    let mut status = Status::Idle;
    let mut tokens = VecDeque::new();
    let mut token = String::new();
    let mut begin = 0;
    for (i, c) in line.char_indices() {
        match status {
            Status::Idle => {
                if c == '"' || c == '\'' {
                    begin = i + 1;
                    status = Status::Quote(c);
                } else if c != ' ' {
                    begin = i;
                    token.push(c);
                    status = Status::Token;
                }
            }
            Status::Token => {
                if c == '"' || c == '\'' {
                    return None;
                } else if c == ' ' {
                    tokens.push_back(Token::new(&token, begin, i, false));
                    token.clear();
                    status = Status::Idle;
                } else {
                    token.push(c);
                }
            }
            Status::Quote(q) => {
                if c == q {
                    tokens.push_back(Token::new(&token, begin, i, true));
                    token.clear();
                    status = Status::Idle;
                } else {
                    token.push(c);
                }
            }
        }
    }
    match status {
        Status::Token => tokens.push_back(Token::new(&token, begin, line.chars().count(), false)),
        Status::Quote(_) => return None,
        _ => {}
    }
    Some(tokens)
}

#[test]
fn test_tokenize() {
    assert_eq!(tokenize("a"), Some(VecDeque::from([Token::new("a", 0, 1, false)])));
    assert_eq!(tokenize("    a    b    "), Some(VecDeque::from([Token::new("a", 4, 5, false), Token::new("b", 9, 10, false)])));
    assert_eq!(tokenize("hello"), Some(VecDeque::from([Token::new("hello", 0, 5, false)])));
    assert_eq!(tokenize("  hello  "), Some(VecDeque::from([Token::new("hello", 2, 7, false)])));
    assert_eq!(tokenize("hello world"), Some(VecDeque::from([Token::new("hello", 0, 5, false), Token::new("world", 6, 11, false)])));
    assert_eq!(tokenize("hello  world"), Some(VecDeque::from([Token::new("hello", 0, 5, false), Token::new("world", 7, 12, false)])));
    assert_eq!(tokenize(" hello world "), Some(VecDeque::from([Token::new("hello", 1, 6, false), Token::new("world", 7, 12, false)])));
    assert_eq!(
        tokenize("say hello world"),
        Some(VecDeque::from([Token::new("say", 0, 3, false), Token::new("hello", 4, 9, false), Token::new("world", 10, 15, false)]))
    );
    assert_eq!(
        tokenize("say \"hello world\""),
        Some(VecDeque::from([Token::new("say", 0, 3, false), Token::new("hello world", 5, 16, true)]))
    );
    assert_eq!(
        tokenize("  say \"hello world\"  "),
        Some(VecDeque::from([Token::new("say", 2, 5, false), Token::new("hello world", 7, 18, true)]))
    );
    assert_eq!(
        tokenize("say \"hello world\" twice"),
        Some(VecDeque::from([Token::new("say", 0, 3, false), Token::new("hello world", 5, 16, true), Token::new("twice", 18, 23, false)]))
    );
    assert_eq!(
        tokenize("say \"nothing\" twice"),
        Some(VecDeque::from([Token::new("say", 0, 3, false), Token::new("nothing", 5, 12, true), Token::new("twice", 14, 19, false)]))
    );
    assert_eq!(
        tokenize("say \"\" twice"),
        Some(VecDeque::from([Token::new("say", 0, 3, false), Token::new("", 5, 5, true), Token::new("twice", 7, 12, false)]))
    );
    assert_eq!(tokenize("hello \"world"), None);
    assert_eq!(tokenize("hello\"world"), None);
}
