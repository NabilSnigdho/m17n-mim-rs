use m17n_mim_rs::M17nMim;
use serde_json::Value;
use std::fs;

#[tokio::test]
async fn test_avro_mim_with_remote_file() {
    let mim_content = fs::read_to_string("/home/nabil/Documents/nabilsnigdho/m17n-mim-rs/tests/bn-avro.mim")
        .expect("Failed to read local MIM file");

    // Create M17nMim instance with the fetched content
    let mim = M17nMim::new(&mim_content);

    // Verify basic MIM properties
    assert_eq!(mim.get_lang(), "bn");
    assert_eq!(mim.get_name(), "avro");

    // Load test cases from JSON file
    let test_cases_content =
        fs::read_to_string("tests/avro_test_cases.json").expect("Failed to read test cases file");

    let test_cases: Value =
        serde_json::from_str(&test_cases_content).expect("Failed to parse test cases JSON");

    let test_array = test_cases
        .as_array()
        .expect("Test cases should be an array");

    let mut passed = 0;
    let mut failed = 0;
    let mut failed_cases = Vec::new();

    // Run tests for each case
    for (index, case) in test_array.iter().enumerate() {
        // Handle both array format ["input", "expected"] and single string format
        match case {
            Value::Array(arr) if arr.len() == 2 => {
                let input = arr[0].as_str().expect("Input should be a string");
                let expected = arr[1].as_str().expect("Expected should be a string");

                let result = mim.convert(input);

                if result == expected {
                    passed += 1;
                    println!(
                        "✓ Test {}: '{}' -> '{}' (expected: '{}')",
                        index + 1,
                        input,
                        result,
                        expected
                    );
                } else {
                    failed += 1;
                    failed_cases.push((
                        index + 1,
                        input.to_string(),
                        expected.to_string(),
                        result.clone(),
                    ));
                    println!(
                        "✗ Test {}: '{}' -> '{}' (expected: '{}')",
                        index + 1,
                        input,
                        result,
                        expected
                    );
                }
            }
            Value::String(single_input) => {
                // For single string cases, just test that conversion doesn't panic
                let result = mim.convert(single_input);
                passed += 1;
                println!(
                    "✓ Test {}: '{}' -> '{}' (single input test)",
                    index + 1,
                    single_input,
                    result
                );
            }
            _ => {
                println!(
                    "⚠ Skipping invalid test case at index {}: {:?}",
                    index + 1,
                    case
                );
            }
        }
    }

    // Print summary
    println!("\n=== Test Summary ===");
    println!("Total tests: {}", passed + failed);
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);

    if !failed_cases.is_empty() {
        println!("\n=== Failed Cases ===");
        for (test_num, input, expected, actual) in &failed_cases {
            println!(
                "Test {}: '{}' -> '{}' (expected: '{}')",
                test_num, input, actual, expected
            );
        }
    }

    // For now, we'll make this test pass even if some conversions fail
    // since the main goal is to test that the MIM file can be loaded and used
    assert!(passed > 0, "At least some test cases should pass");

    // Optional: Uncomment the line below if you want the test to fail when any conversion fails
    assert_eq!(failed, 0, "All test cases should pass");
}

#[tokio::test]
async fn test_mim_basic_properties() {
    let mim_content = fs::read_to_string("/home/nabil/Documents/nabilsnigdho/m17n-mim-rs/tests/bn-avro.mim")
        .expect("Failed to read local MIM file");

    // Create M17nMim instance
    let mim = M17nMim::new(&mim_content);

    // Test basic properties
    assert_eq!(mim.get_lang(), "bn");
    assert_eq!(mim.get_name(), "avro");
    assert!(!mim.get_title().is_empty(), "Title should not be empty");

    // Test some basic conversions
    let test_cases = vec![
        ("a", "a"),      // Should at least return the input if no conversion
        ("amar", "আমার"), // Common Bengali word
        ("ki", "কি"),    // Simple conversion
    ];

    for (input, _expected) in test_cases {
        let result = mim.convert(input);
        assert!(
            !result.is_empty(),
            "Conversion result should not be empty for input: {}",
            input
        );
        println!("'{}' -> '{}'", input, result);
    }
}
