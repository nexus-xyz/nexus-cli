use nexus_rt::testing::*;

#[test]
fn test_valid_signature() {
    // These are example test vectors - in a real test we would use actual Ed25519 test vectors
    let test_signature = [1u8; 64];
    let test_public_key = [2u8; 32];
    let test_message = [3u8; 32];

    let mut inputs = TestInputs::default();
    inputs.add_private_bytes(&test_signature);
    inputs.add_private_bytes(&test_public_key);
    inputs.add_private_bytes(&test_message);

    // This will pass since we're mocking the ed25519_verify to return true
    let result = mock_program_with_inputs(inputs, |ctx| {
        ctx.mock_ed25519_verify(|_, _, _| true);
    });

    assert_eq!(result, 0);
}
