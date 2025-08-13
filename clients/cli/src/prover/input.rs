//! Input parsing and validation

use super::types::ProverError;

/// Input parser for proving tasks
pub struct InputParser;

impl InputParser {
    /// Parse triple public input from byte data (n, init_a, init_b)
    pub fn parse_triple_input(input_data: &[u8]) -> Result<(u32, u32, u32), ProverError> {
        if input_data.len() < (u32::BITS / 8 * 3) as usize {
            return Err(ProverError::MalformedTask(
                "Public inputs buffer too small, expected at least 12 bytes for three u32 values"
                    .to_string(),
            ));
        }

        let mut bytes = [0u8; 4];

        bytes.copy_from_slice(&input_data[0..4]);
        let n = u32::from_le_bytes(bytes);

        bytes.copy_from_slice(&input_data[4..8]);
        let init_a = u32::from_le_bytes(bytes);

        bytes.copy_from_slice(&input_data[8..12]);
        let init_b = u32::from_le_bytes(bytes);

        Ok((n, init_a, init_b))
    }
}
