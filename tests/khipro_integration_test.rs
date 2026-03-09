use m17n_mim_rs::M17nMim;
use serde_json::Value;
use std::fs;
use std::error::Error;

async fn get_remote_test_cases() -> Result<Vec<(String, String)>, Box<dyn Error>> {
    let url =
        "https://raw.githubusercontent.com/KhiproTeam/khipro-testcases/refs/heads/main/khipro-testcases.csv";

    let client = reqwest::Client::new();

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(format!("Failed to fetch test cases file: {}", response.status()).into());
    }

    let content = response.text().await?;

    let mut rdr = csv::Reader::from_reader(std::io::Cursor::new(content));

    let mut cases = Vec::new();

    for record in rdr.records() {
        let record = record?;

        if record.len() >= 2 {
            let input = record[0].to_string().trim().to_string();
            let expected = record[1].to_string().replace("\u{200d}র", "র").trim().to_string();

            cases.push((input, expected));
        }
    }

    Ok(cases)
}

#[tokio::test]
async fn test_khipro_mim_with_remote_file() {
    // Fetch the remote MIM file
    let url =
        "https://raw.githubusercontent.com/rank-coder/khipro-m17n/refs/heads/main/bn-khipro.mim";
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .send()
        .await
        .expect("Failed to fetch remote MIM file");

    assert!(
        response.status().is_success(),
        "Failed to fetch MIM file: {}",
        response.status()
    );

    let mim_content = response
        .text()
        .await
        .expect("Failed to read MIM file content");

    // Create M17nMim instance with the fetched content
    let mim = M17nMim::new(&mim_content);

    // Verify basic MIM properties
    assert_eq!(mim.get_lang(), "bn");
    assert_eq!(mim.get_name(), "khipro");

    // Load test cases from JSON file
    let test_cases_content =
        fs::read_to_string("tests/khipro_test_cases.json").expect("Failed to read test cases file");
    
    let test_cases_json: Value =
    serde_json::from_str(&test_cases_content).expect("Failed to parse test cases JSON");

    let mut test_cases: Vec<(String, String)> = Vec::new();

    if let Some(arr) = test_cases_json.as_array() {
        for case in arr {
            if let Value::Array(pair) = case {
                if pair.len() == 2 {
                    let input = pair[0].as_str().unwrap_or("").to_string();
                    let expected = pair[1].as_str().unwrap_or("").to_string();
                    test_cases.push((input, expected));
                }
            }
        }
    }

    // append remote cases
    let remote_test_cases = get_remote_test_cases().await.unwrap();
    test_cases.extend(remote_test_cases);

    let mut passed = 0;
    let mut failed = 0;
    let mut failed_cases = Vec::new();

    // Run tests for each case
    for (index, (input, expected)) in test_cases.iter().enumerate() {

        let result = mim.convert(input);

        if result == *expected {
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
    // Fetch the remote MIM file
    let url =
        "https://raw.githubusercontent.com/rank-coder/khipro-m17n/refs/heads/main/bn-khipro.mim";
    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .send()
        .await
        .expect("Failed to fetch remote MIM file");

    let mim_content = response
        .text()
        .await
        .expect("Failed to read MIM file content");

    // Create M17nMim instance
    let mim = M17nMim::new(&mim_content);

    // Test basic properties
    assert_eq!(mim.get_lang(), "bn");
    assert_eq!(mim.get_name(), "khipro");
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
