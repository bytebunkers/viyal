use analyzer::analyze;

#[test]
fn test_analyzer_empty_block() {
    let source = "class A { void run() {} }";
    let warnings = analyze(source).expect("Failed to parse");
    
    assert_eq!(warnings.len(), 1);
    assert_eq!(warnings[0].message, "Empty block detected. Consider removing it or adding a comment.");
}

#[test]
fn test_analyzer_no_warnings() {
    let source = "class A { void run() { print(\"hello\"); } }";
    let warnings = analyze(source).expect("Failed to parse");
    
    assert_eq!(warnings.len(), 0);
}
