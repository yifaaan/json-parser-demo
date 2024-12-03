pub fn tokenize(input: String) -> Result<Vec<Token>, TokenizeError> {
    let chars: Vec<char> = input.chars().collect();
    let mut index = 0;

    let mut tokens = Vec::new();
    while index < chars.len() {
        let token = make_token(&chars, &mut index)?;
        tokens.push(token);
        index += 1;
    }

    Ok(tokens)
}

fn make_token(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    let mut ch = chars[*index];

    while ch.is_ascii_whitespace() {
        *index += 1;
        if *index > chars.len() {
            return Err(TokenizeError::UnexpectedEof);
        }
        ch = chars[*index];
    }

    let token = match ch {
        '[' => Token::LeftBracket,
        ']' => Token::RightBracket,
        '{' => Token::LeftBrace,
        '}' => Token::RightBrace,
        ',' => Token::Comma,
        ':' => Token::Colon,
        'n' => tokenize_null(chars, index)?,
        't' => tokenize_true(chars, index)?,
        'f' => tokenize_false(chars, index)?,
        c if c.is_ascii_digit() => tokenize_float(chars, index)?,
        '"' => tokenize_string(chars, index)?,
        c => return Err(TokenizeError::CharNotRecognized(c)),
        _ => todo!("implement other tokens"),
    };
    Ok(token)
}

/// One of the possible errors that could occur while tokenizing the input string
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TokenizeError {
    /// The input apperaed to be the start of the literal value but dit not finished
    UnfinishedLiteralValue,
    /// Unable to parse the float value
    ParseNumberError,
    /// String was never completed
    UnclosedQuotes,
    /// The input ended early
    UnexpectedEof,
    /// Character is not part of a json token
    CharNotRecognized(char),
}

fn tokenize_null(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    for expected_char in "null".chars() {
        if expected_char != chars[*index] {
            return Err(TokenizeError::UnfinishedLiteralValue);
        }
        *index += 1;
    }
    *index -= 1;
    Ok(Token::Null)
}

fn tokenize_false(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    for expected_char in "false".chars() {
        if expected_char != chars[*index] {
            return Err(TokenizeError::UnfinishedLiteralValue);
        }
        *index += 1;
    }
    *index -= 1;
    Ok(Token::False)
}

fn tokenize_true(chars: &Vec<char>, index: &mut usize) -> Result<Token, TokenizeError> {
    for expected_char in "true".chars() {
        if expected_char != chars[*index] {
            return Err(TokenizeError::UnfinishedLiteralValue);
        }
        *index += 1;
    }
    *index -= 1;
    Ok(Token::True)
}

fn tokenize_float(chars: &Vec<char>, cur_idx: &mut usize) -> Result<Token, TokenizeError> {
    let mut unparsed_num = String::new();
    let mut has_decimal = false;

    while *cur_idx < chars.len() {
        let ch = chars[*cur_idx];
        match ch {
            c if c.is_ascii_digit() => unparsed_num.push(ch),
            c if c == '.' && !has_decimal => {
                unparsed_num.push(ch);
                has_decimal = true;
            }
            _ => break,
        }
        *cur_idx += 1;
    }
    *cur_idx -= 1;
    unparsed_num
        .parse()
        .map(|num| Token::Number(num))
        .map_err(|_| TokenizeError::ParseNumberError)
}

fn tokenize_string(chars: &Vec<char>, cur_idx: &mut usize) -> Result<Token, TokenizeError> {
    let mut string = String::new();
    let mut is_escaping = false;

    loop {
        *cur_idx += 1;
        if *cur_idx >= chars.len() {
            return Err(TokenizeError::UnclosedQuotes);
        }
        let ch = chars[*cur_idx];
        match ch {
            '"' if !is_escaping => break,
            '\\' => is_escaping = !is_escaping,
            _ => is_escaping = false,
        }
        string.push(ch);
    }
    Ok(Token::String(string))
}

///
///
/// {
///   "nums": [1.2, 3.4]
/// }
///
///
/// Tokens are:
///
/// LeftBrace,
/// String("nums"),
/// Colon,
/// LeftBracket,
/// Number(1.2),
/// Comma,
/// Number(3.4),
/// RightBracket,
/// RightBrace,
///

#[derive(Debug, PartialEq)]
pub enum Token {
    /// `{`
    LeftBrace,

    /// `}`
    RightBrace,

    /// `[`
    LeftBracket,

    /// `]`
    RightBracket,

    /// `,`
    Comma,

    /// `:`
    Colon,

    /// `null`
    Null,

    /// `false`
    False,

    /// `true`
    True,

    /// Any number literal
    Number(f64),

    /// Key of the key/value pair or string value
    String(String),
}

#[cfg(test)]
mod tests {
    use super::{tokenize, Token, TokenizeError};

    #[test]
    fn just_comma() {
        let input = String::from(",");
        let expected = [Token::Comma];

        let actual = tokenize(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn all_punctuation() {
        let input = String::from("[{]},:");
        let expected = [
            Token::LeftBracket,
            Token::LeftBrace,
            Token::RightBracket,
            Token::RightBrace,
            Token::Comma,
            Token::Colon,
        ];

        let actual = tokenize(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn just_null() {
        let input = String::from("null");
        let expected = [Token::Null];

        let actual = tokenize(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn just_false() {
        let input = String::from("false");
        let expected = [Token::False];

        let actual = tokenize(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn just_true() {
        let input = String::from("true");
        let expected = [Token::True];

        let actual = tokenize(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn true_comma() {
        let input = String::from("true,");
        let expected = [Token::True, Token::Comma];

        let actual = tokenize(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn just_integer() {
        let input = String::from("123");
        let expected = [Token::Number(123.0)];

        let actual = tokenize(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn integer_comma() {
        let input = String::from("123,");
        let expected = [Token::Number(123.0), Token::Comma];

        let actual = tokenize(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn negative_integer() {
        let input = String::from("-123");
        let expected = [Token::Number(-123.0)];

        let actual = tokenize(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn just_float() {
        let input = String::from("123.4");
        let expected = [Token::Number(123.4)];

        let actual = tokenize(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn float_comma() {
        let input = String::from("123.4,");
        let expected = [Token::Number(123.4), Token::Comma];

        let actual = tokenize(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn just_ken() {
        let input = String::from("\"ken\"");
        let expected = [Token::String(String::from("ken"))];

        let actual = tokenize(input).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn unclosed_string() {
        let input = String::from("\"unclosed");
        let expected = Err(TokenizeError::UnclosedQuotes);

        let actual = tokenize(input);
        assert_eq!(actual, expected);
    }

    #[test]
    fn escaped_quote() {
        let input = String::from(r#""The \" is Ok ""#);
        let expected = [Token::String(String::from(r#"The \" is Ok "#))];

        let actual = tokenize(input).unwrap();
        assert_eq!(actual, expected);
    }
}
