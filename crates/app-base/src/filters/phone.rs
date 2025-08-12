use alloc::string::String;

pub fn filter_phone(phone: &str) -> String {
    phone
        .chars()
        .enumerate()
        .filter_map(|(idx, c)| {
            if c.is_ascii_digit() || (c == '+' && idx == 0) {
                Some(c)
            } else {
                None
            }
        })
        .collect()
}
