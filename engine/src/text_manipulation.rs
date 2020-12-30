use std::collections::HashSet;

/// Splits a command line (String) into a vector of arguments. Based on a solution posted to
/// [Stack Overflow](https://stackoverflow.com/a/23961658).
pub fn tokenize<'a, S: AsRef<str>, H: IntoIterator<Item = &'a char>>(
    arg_string: S,
    escape_char: char,
    quote_char: char,
    separator_chars: H,
) -> Vec<String> {
    let mut args = Vec::new();
    let separator_chars: HashSet<char> = separator_chars.into_iter().cloned().collect();

    let mut escape = false;
    let mut in_quote = false;
    let mut had_quote = false;
    let mut prev_char = '\0';
    let mut current_arg = String::new();

    for c in arg_string.as_ref().chars() {
        if c == escape_char && !escape {
            // The start of an escaped sequence.
            escape = true;
        } else if (c == escape_char || c == quote_char) && escape {
            // Keep the actual escape character if it appears twice.
            // Keep escaped quotes.
            current_arg.push(c);
            escape = false;
        } else if c == quote_char && !escape {
            // Toggle a quoted section.
            in_quote = !in_quote;
            had_quote = true;
            if in_quote && prev_char == quote_char {
                // Double quotes behave like double escapes in a quoted range.
                current_arg.push(c);
            }
        } else if c.is_whitespace() && !in_quote {
            // Add the pending escape character.
            if escape {
                current_arg.push(escape_char);
                escape = false;
            }

            // Add the current token to the list. Accept empty tokens only if they are quoted
            if !current_arg.is_empty() || had_quote {
                args.push(current_arg.clone());
            }

            // Reset the current argument
            current_arg.clear();
            had_quote = false;
        } else if separator_chars.iter().any(|sc| &c == sc) && !in_quote {
            // Add the pending escape character.
            if escape {
                current_arg.push(escape_char);
                escape = false;
            }

            // Add the current token to the list. Accept empty tokens only if they are quoted
            if !current_arg.is_empty() || had_quote {
                args.push(current_arg.clone());
            }

            // Reset the current argument
            current_arg.clear();
            had_quote = false;

            // Add the separator character to the argument list as its own token
            args.push(c.to_string());
        } else {
            // Add the pending escape character
            if escape {
                current_arg.push(escape_char);
                escape = false;
            }

            // Copy the character from input without a special meaning
            current_arg.push(c);
        }
        prev_char = c;
    }
    // Save the last argument
    if !current_arg.is_empty() || had_quote {
        args.push(current_arg.clone());
    }

    args
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_argument_list() {
        let args = tokenize(
            "command -f flagvalue  positional_argument 100 ",
            '\\',
            '"',
            &[],
        );

        assert_eq!(
            args,
            vec!["command", "-f", "flagvalue", "positional_argument", "100"]
        );
    }

    #[test]
    fn quoted_argument_list() {
        let args = tokenize(
            "command -f \"flag value\"  \"positional argument\" 100 ",
            '\\',
            '"',
            &[],
        );

        assert_eq!(
            args,
            vec!["command", "-f", "flag value", "positional argument", "100"]
        );
    }

    #[test]
    fn escaped_argument_list() {
        let args = tokenize(
            r"command -f flag\\ value  positional argument 100 ",
            '\\',
            '"',
            &[],
        );

        assert_eq!(
            args,
            vec![
                "command",
                "-f",
                "flag\\",
                "value",
                "positional",
                "argument",
                "100"
            ]
        );
    }

    #[test]
    fn special_separators() {
        let args = tokenize(
            "command -f flagvalue  positional_argument 100; othercommand -p flagvalue",
            '\\',
            '"',
            &[';'],
        );

        assert_eq!(
            args,
            vec![
                "command",
                "-f",
                "flagvalue",
                "positional_argument",
                "100",
                ";",
                "othercommand",
                "-p",
                "flagvalue"
            ]
        );
    }
}
