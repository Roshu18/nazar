use std::iter;

const QUOTE: char = '"';
const BACKSLASH: char = '\\';
const ARGUMENT_SEPARATOR: &str = " ";

pub fn join_arguments(arguments: &[String]) -> String {
    let quoted_arguments: Vec<String> = arguments
        .iter()
        .map(|argument| quote_argument(argument))
        .collect();
    quoted_arguments.join(ARGUMENT_SEPARATOR)
}

pub fn quote_argument(argument: &str) -> String {
    let needs_quotes = argument.is_empty() || argument.contains([' ', '\t', QUOTE]);
    if !needs_quotes {
        return argument.to_owned();
    }
    let mut quoted = String::from(QUOTE);
    let mut pending_backslash_count = 0;
    for character in argument.chars() {
        if character == BACKSLASH {
            pending_backslash_count += 1;
            continue;
        }
        let escaped_backslash_count = if character == QUOTE {
            pending_backslash_count * 2 + 1
        } else {
            pending_backslash_count
        };
        quoted.extend(iter::repeat_n(BACKSLASH, escaped_backslash_count));
        quoted.push(character);
        pending_backslash_count = 0;
    }
    quoted.extend(iter::repeat_n(BACKSLASH, pending_backslash_count * 2));
    quoted.push(QUOTE);
    quoted
}
