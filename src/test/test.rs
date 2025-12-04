/// Test including sample transactions
/// and verifying output correctness.
use std::process::{Command, Stdio};
use std::str;

fn run_file(file_path: &str, batch_mode: bool) -> String {
    let mut args = vec!["run", "--", file_path];
    if batch_mode {
        args.push("--batch");
    }
    let output = Command::new("cargo")
        .args(&args)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to start process")
        .wait_with_output()
        .expect("Failed to wait on process");
    str::from_utf8(&output.stdout)
        .expect("Failed to read stdout")
        .to_string()
}

static SAMPLE_OUTPUT: &str = "\
client,available,held,total,locked
1,1.5,0.0,1.5,false
2,2.0,0.0,2.0,false
";

static SAMPLE_OUTPUT_WITH_WITHDRAWAL: &str = "\
client,available,held,total,locked
1,1.5,0.0,1.5,false
2,1.0,0.0,1.0,false
";
#[test]
fn test_sample_transactions() {
    let output = run_file("./src/test/sample_transactions.csv", false);
    let expected_output = SAMPLE_OUTPUT;
    assert_eq!(output, expected_output);

    let output = run_file("./src/test/sample_transactions.csv", true);
    let expected_output = "";
    assert_eq!(output, expected_output);
}

#[test]
fn test_sample_transactions_with_invalid_disputes() {
    let output = run_file(
        "./src/test/sample_transactions_with_invalid_dispute.csv",
        false,
    );
    let expected_output = SAMPLE_OUTPUT;
    assert_eq!(output, expected_output);

    let output = run_file(
        "./src/test/sample_transactions_with_invalid_dispute.csv",
        true,
    );
    let expected_output = "";
    assert_eq!(output, expected_output);
}

#[test]
fn test_sample_transaction_with_dismpute_with_non_sufficient_fund_left() {
    let output = run_file(
        "./src/test/sample_transactions_with_dispute_with_non_sufficient_fund_left.csv",
        false,
    );
    let expected_output = SAMPLE_OUTPUT_WITH_WITHDRAWAL;
    assert_eq!(output, expected_output);

    let output = run_file(
        "./src/test/sample_transactions_with_dispute_with_non_sufficient_fund_left.csv",
        true,
    );
    let expected_output = "";
    assert_eq!(output, expected_output);
}
