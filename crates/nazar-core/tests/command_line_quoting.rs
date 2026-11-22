use nazar_core::command_line::{join_arguments, quote_argument};

#[test]
fn plain_argument_is_left_as_it_is() {
    assert_eq!(quote_argument("--output"), "--output");
}

#[test]
fn argument_with_a_space_is_wrapped_in_quotes() {
    assert_eq!(
        quote_argument("C:\\Program Files\\App\\setup.exe"),
        "\"C:\\Program Files\\App\\setup.exe\""
    );
}

#[test]
fn backslashes_before_the_closing_quote_are_doubled() {
    assert_eq!(quote_argument("C:\\My Folder\\"), "\"C:\\My Folder\\\\\"");
}

#[test]
fn embedded_quote_is_escaped() {
    assert_eq!(quote_argument("say \"hi\""), "\"say \\\"hi\\\"\"");
}

#[test]
fn empty_argument_stays_an_argument() {
    assert_eq!(quote_argument(""), "\"\"");
}

#[test]
fn arguments_are_joined_with_single_spaces() {
    let arguments = ["watch".to_owned(), "C:\\My Setup.exe".to_owned()];

    assert_eq!(join_arguments(&arguments), "watch \"C:\\My Setup.exe\"");
}
