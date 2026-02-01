// Unit tests for TypedKey system
use std::path::PathBuf;
use tweers_core::pipeline::{BASE64, PipeMap, SOURCES};

#[test]
fn test_typed_key_basic() {
    let mut data = PipeMap::new();
    data.insert_typed(SOURCES, vec![PathBuf::from("test.twee")]);

    let sources = data.get_typed(SOURCES).unwrap();
    assert_eq!(sources.len(), 1);
    assert_eq!(sources[0], PathBuf::from("test.twee"));
}

#[test]
fn test_typed_key_bool() {
    let mut data = PipeMap::new();
    data.insert_typed(BASE64, true);

    let base64 = data.get_typed(BASE64).unwrap();
    assert_eq!(*base64, true);
}
