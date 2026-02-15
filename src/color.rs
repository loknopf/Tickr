/// Color utilities for categories and UI.
use rand::RngExt;

/// Validate if a string is a valid hex color (e.g., #RRGGBB).
pub fn is_valid_hex(s: &str) -> bool {
    s.starts_with('#') && s.len() == 7 && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Generate a random color from a predefined palette.
pub fn random_color() -> String {
    const PALETTE: &[&str] = &[
        "#FF5733", "#33FF57", "#3357FF", "#F333FF", "#33FFF5", "#F5FF33", "#FF33A8",
        "#A833FF", "#33FFA8", "#FFA833", "#FF3380", "#8033FF", "#33FF80", "#FF8033",
    ];
    let mut rng = rand::rng();
    PALETTE[rng.random_range(0..PALETTE.len())].to_string()
}
