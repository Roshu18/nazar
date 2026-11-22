const PRODUCT_CODE_DIGIT_COUNT: usize = 32;
const REVERSED_GROUPS: [(usize, usize); 3] = [(0, 8), (8, 12), (12, 16)];
const PAIRED_DIGITS_START: usize = 16;
const PAIR_LENGTH: usize = 2;

pub fn packed_product_code(product_code: &str) -> Option<String> {
    let digits: Vec<char> = product_code
        .trim()
        .strip_prefix('{')?
        .strip_suffix('}')?
        .chars()
        .filter(|character| *character != '-')
        .collect();
    let is_product_code =
        digits.len() == PRODUCT_CODE_DIGIT_COUNT && digits.iter().all(char::is_ascii_hexdigit);
    if !is_product_code {
        return None;
    }
    let reversed_groups = REVERSED_GROUPS
        .iter()
        .flat_map(|(start, end)| digits[*start..*end].iter().rev());
    let swapped_pairs = digits[PAIRED_DIGITS_START..]
        .chunks(PAIR_LENGTH)
        .flat_map(|pair| pair.iter().rev());
    Some(
        reversed_groups
            .chain(swapped_pairs)
            .map(char::to_ascii_uppercase)
            .collect(),
    )
}
