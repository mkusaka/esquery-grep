use serde_json::Value;

/// Load a JSON AST fixture by name.
pub fn load(name: &str) -> Value {
    let path = format!("{}/tests/fixtures/{}.json", env!("CARGO_MANIFEST_DIR"), name);
    let data = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", name, e));
    serde_json::from_str(&data).unwrap_or_else(|e| panic!("Failed to parse fixture {}: {}", name, e))
}

/// Navigate a JSON value by a dot-separated path.
/// Numeric segments are treated as array indices.
/// Example: "body.0.test.left"
pub fn nav<'a>(root: &'a Value, path: &str) -> &'a Value {
    let mut current = root;
    for segment in path.split('.') {
        if segment.is_empty() {
            continue;
        }
        if let Ok(idx) = segment.parse::<usize>() {
            current = current
                .as_array()
                .unwrap_or_else(|| panic!("Expected array at '{}' in path '{}'", segment, path))
                .get(idx)
                .unwrap_or_else(|| panic!("Index {} out of bounds in path '{}'", idx, path));
        } else {
            current = current
                .get(segment)
                .unwrap_or_else(|| panic!("Key '{}' not found in path '{}'", segment, path));
        }
    }
    current
}

/// Assert that all expected nodes are present in results (by pointer equality).
/// Equivalent to chai's `assert.includeMembers`.
pub fn assert_includes(results: &[&Value], expected: &[&Value]) {
    for (i, exp) in expected.iter().enumerate() {
        assert!(
            results.iter().any(|r| std::ptr::eq(*r, *exp)),
            "Expected node at index {} not found in {} results",
            i,
            results.len()
        );
    }
}

/// Assert that results contain exactly the expected nodes (same set, same count).
/// Equivalent to chai's `assert.sameMembers`.
#[allow(dead_code)]
pub fn assert_same(results: &[&Value], expected: &[&Value]) {
    assert_eq!(
        results.len(),
        expected.len(),
        "Expected {} matches but got {}",
        expected.len(),
        results.len()
    );
    assert_includes(results, expected);
}
