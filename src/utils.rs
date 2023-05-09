const VERSIONS: [(&str, i32); 10] = [
    ("1.12.2", 2),
    ("1.12", 2),
    ("1.16.5", 6),
    ("1.16", 6),
    ("1.18.2", 8),
    ("1.18", 8),
    ("1.19.2", 9),
    ("1.19.3", 9),
    ("1.19.4", 9),
    ("1.19", 9),
];

pub fn convert_int_to_version(input: &[i32]) -> Vec<&str> {
    input
        .iter()
        .map(|x| {
            VERSIONS
                .iter()
                .find(|(_, y)| y == x)
                .map(|(s, _)| *s)
                .unwrap_or("")
        })
        .collect()
}
