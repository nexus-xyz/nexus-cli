//! Ethereum address validation functions.

use sha3::{Digest, Keccak256};

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
    if !address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        return false;
    }

    // Validate EIP-55 checksum (with spec-compliant rules)
    validate_eip55_checksum(address)
}

/// Validates EIP-55 checksum for Ethereum addresses.
///
/// EIP-55 defines a checksum encoding for Ethereum addresses that uses mixed case
/// to encode the address hash. This allows for error detection when typing addresses.
///
/// Algorithm:
/// 1. Convert address to lowercase
/// 2. Compute Keccak256 hash of the lowercase address
/// 3. For each character in the address:
///    - If the corresponding bit in the hash is >= 8, the character should be uppercase
///    - Otherwise, the character should be lowercase
fn validate_eip55_checksum(address: &str) -> bool {
    // Work with the 40-hex part (skip the 0x/0X prefix)
    let hex = &address[2..];

    // If the address is all lower or all upper, it's valid per EIP-55
    if hex
        .chars()
        .all(|c| !c.is_ascii_alphabetic() || c.is_ascii_lowercase())
        || hex
            .chars()
            .all(|c| !c.is_ascii_alphabetic() || c.is_ascii_uppercase())
    {
        return true;
    }

    // Hash the lowercase hex (without 0x prefix)
    let hex_lower = hex.to_ascii_lowercase();
    let mut hasher = Keccak256::new();
    hasher.update(hex_lower.as_bytes());
    let hash = hasher.finalize();

    // Verify case only for alphabetic hex chars (a-f)
    for (i, ch) in hex.chars().enumerate() {
        if ch.is_ascii_digit() {
            continue;
        }

        // Determine the corresponding nibble from the hash
        let hash_byte = hash[i / 2];
        let nibble = if i % 2 == 0 {
            hash_byte >> 4
        } else {
            hash_byte & 0x0F
        };

        if nibble >= 8 {
            if !ch.is_ascii_uppercase() {
                return false;
            }
        } else if !ch.is_ascii_lowercase() {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Addresses with all lowercase are valid (no checksum validation)
    fn valid_all_lowercase() {
        assert!(is_valid_eth_address(
            "0xde709f2102306220921060314715629080e2fb77"
        ));
    }

    #[test]
    /// Addresses with all uppercase are valid (no checksum validation)
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
    /// Test EIP-55 checksum validation - incorrect case should fail
    fn invalid_checksum_address() {
        assert!(!is_valid_eth_address(
            "0x52908400098527886E0F7030069857D2E4169ee7"
        ));
    }

    #[test]
    /// Test with a known valid EIP-55 address
    fn valid_eip55_address() {
        // This is a known valid EIP-55 checksummed address
        assert!(is_valid_eth_address(
            "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed"
        ));
    }

    #[test]
    /// Test with invalid EIP-55 checksum
    fn invalid_eip55_address() {
        // Same address but with incorrect checksum (mixed case but wrong)
        assert!(!is_valid_eth_address(
            "0x5aAeb6053f3e94c9b9a09f33669435e7ef1beaed"
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
