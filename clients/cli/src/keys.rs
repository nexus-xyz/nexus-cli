//! Ethereum address validation functions.

/// Check if a given string is a valid Ethereum address.
pub fn is_valid_eth_address(address: &str) -> bool {
    // Must be 42 characters: "0x" + 40 hex digits
    if address.len() != 42 {
        return false;
    }

    // Must start with "0x" or "0X"
    if !address.starts_with("0x") && !address.starts_with("0X") {
        return false;
    }

    // Check that the remaining 40 characters are all valid hex digits
    address[2..].chars().all(|c| c.is_ascii_hexdigit())

    // TODO: validate EIP-55 checksum
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_checksum_address() {
        assert!(is_valid_eth_address(
            "0x52908400098527886E0F7030069857D2E4169EE7"
        )); // correct checksum
    }

    #[test]
    /// Validation should be case-insensitive for hex digits.
    fn valid_all_lowercase() {
        assert!(is_valid_eth_address(
            "0xde709f2102306220921060314715629080e2fb77"
        ));
    }

    #[test]
    /// Validation should be case-insensitive for hex digits.
    fn valid_all_uppercase() {
        assert!(is_valid_eth_address(
            "0xDE709F2102306220921060314715629080E2FB77"
        ));
    }

    #[test]
    /// Validation should be case-insensitive for prefix "0x".
    fn valid_uppercase_prefix() {
        assert!(is_valid_eth_address(
            "0X52908400098527886E0F7030069857D2E4169EE7"
        ));
    }

    #[test]
    #[ignore]
    /// TODO: Validate EIP-55 checksum
    fn invalid_checksum_address() {
        assert!(!is_valid_eth_address(
            "0x52908400098527886E0F7030069857D2E4169ee7"
        ));
    }

    #[test]
    /// Address must be exactly 42 characters long.
    fn invalid_length() {
        assert!(!is_valid_eth_address("0x123")); // too short
    }

    #[test]
    /// Check for invalid characters (e.g. non-hex characters) in the address.
    fn invalid_chars() {
        assert!(!is_valid_eth_address(
            "0xZ2908400098527886E0F7030069857D2E4169EE7"
        )); // 'Z' is not hex
    }

    #[test]
    /// Address must start with "0x" or "0X".
    fn missing_prefix() {
        assert!(!is_valid_eth_address(
            "52908400098527886E0F7030069857D2E4169EE7"
        )); // no 0x
    }
}
