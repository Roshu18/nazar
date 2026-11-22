const MINIMUM_DISTINCTIVE_LENGTH: usize = 3;
const ARCHITECTURE_WORDS: [&str; 7] = [
    "x64", "x86", "64-bit", "32-bit", "64bit", "32bit", "version",
];
const GENERIC_NAMES: [&str; 16] = [
    "setup",
    "install",
    "installer",
    "uninstall",
    "uninstaller",
    "unins000",
    "update",
    "updater",
    "launcher",
    "app",
    "application",
    "program",
    "programs",
    "bin",
    "microsoft",
    "windows",
];

pub fn name_key(name: &str) -> String {
    without_parenthesised_parts(name)
        .split_whitespace()
        .filter(|word| !is_version_or_architecture(word))
        .flat_map(str::chars)
        .filter(|character| character.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

pub fn is_distinctive(name_key: &str) -> bool {
    name_key.chars().count() >= MINIMUM_DISTINCTIVE_LENGTH && !GENERIC_NAMES.contains(&name_key)
}

fn is_version_or_architecture(word: &str) -> bool {
    let lowercase_word = word.to_lowercase();
    let digits_start = lowercase_word.trim_start_matches('v');
    let looks_like_version =
        digits_start.starts_with(|first: char| first.is_ascii_digit()) && word.contains('.');
    looks_like_version || ARCHITECTURE_WORDS.contains(&lowercase_word.as_str())
}

fn without_parenthesised_parts(name: &str) -> String {
    let mut open_parenthesis_count: usize = 0;
    let mut kept = String::new();
    for character in name.chars() {
        match character {
            '(' => open_parenthesis_count += 1,
            ')' => open_parenthesis_count = open_parenthesis_count.saturating_sub(1),
            _ if open_parenthesis_count == 0 => kept.push(character),
            _ => {}
        }
    }
    kept
}
