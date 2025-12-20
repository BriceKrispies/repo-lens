#[derive(Debug)]
pub struct Diff {
    pub expected_len: usize,
    pub actual_len: usize,
    pub first_mismatch: Option<usize>,
    pub expected_sample: Vec<String>,
    pub actual_sample: Vec<String>,
}

pub fn compare_lines(expected: &[String], actual: &[String]) -> Result<(), Diff> {
    if expected.len() != actual.len() {
        return Err(Diff {
            expected_len: expected.len(),
            actual_len: actual.len(),
            first_mismatch: None,
            expected_sample: expected.iter().take(10).cloned().collect(),
            actual_sample: actual.iter().take(10).cloned().collect(),
        });
    }

    for (i, (exp, act)) in expected.iter().zip(actual.iter()).enumerate() {
        if exp != act {
            return Err(Diff {
                expected_len: expected.len(),
                actual_len: actual.len(),
                first_mismatch: Some(i),
                expected_sample: expected.iter().take(10).cloned().collect(),
                actual_sample: actual.iter().take(10).cloned().collect(),
            });
        }
    }

    Ok(())
}
