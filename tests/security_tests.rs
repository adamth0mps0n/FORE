// tests/signature_tests.rs
use fore::{ForeSystem, SignaturePlugin, GFp2, GFp2Vec};

#[test]
fn test_basic_signature_properties() {
    let system = ForeSystem::new(0xDEADBEEF);
    let data = b"Test message";

    // Create signature
    let frame_data = system.process_data(data);
    let bytes = Vec::<GFp2>::to_bytes(&frame_data);
    let signature = system.create_signature(&bytes).unwrap();

    // Verify basic properties
    assert!(system.verify_signature(&bytes, &signature),
           "Valid signature should verify");

    // Attempt modifications
    let mut modified = bytes.clone();
    modified[0] ^= 1;  // Flip one bit
    assert!(!system.verify_signature(&modified, &signature),
            "Modified data should not verify with original signature");
}

#[test]
fn test_signature_integrity() {
    let system = ForeSystem::new(0xDEADBEEF);
    let data = b"Original data";

    // Original signature
    let frame_data = system.process_data(data);
    let bytes = Vec::<GFp2>::to_bytes(&frame_data);
    let signature = system.create_signature(&bytes).unwrap();

    // Modify data and create new signature
    let modified = b"Modified data";
    let modified_frame = system.process_data(modified);
    let modified_bytes = Vec::<GFp2>::to_bytes(&modified_frame);
    let modified_sig = system.create_signature(&modified_bytes).unwrap();

    // Original signature should not verify modified data
    assert!(!system.verify_signature(&modified_bytes, &signature),
            "Original signature should not verify modified data");

    // Modified signature should not verify original data
    assert!(!system.verify_signature(&bytes, &modified_sig),
            "Modified signature should not verify original data");
}

#[test]
fn test_signature_uniqueness() {
    let system = ForeSystem::new(0xDEADBEEF);
    let data1 = b"First message";
    let data2 = b"Second message";

    // Create signatures for different messages
    let frame1 = system.process_data(data1);
    let bytes1 = Vec::<GFp2>::to_bytes(&frame1);
    let sig1 = system.create_signature(&bytes1).unwrap();

    let frame2 = system.process_data(data2);
    let bytes2 = Vec::<GFp2>::to_bytes(&frame2);
    let sig2 = system.create_signature(&bytes2).unwrap();

    // Each signature should only verify its own data
    assert!(system.verify_signature(&bytes1, &sig1),
            "First signature should verify its data");
    assert!(system.verify_signature(&bytes2, &sig2),
            "Second signature should verify its data");
    assert!(!system.verify_signature(&bytes1, &sig2),
            "First data should not verify with second signature");
    assert!(!system.verify_signature(&bytes2, &sig1),
            "Second data should not verify with first signature");
}
