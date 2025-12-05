use std::fs::File;
use std::io::Write;
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

fn create_temp_csv(filename: &str, content: &str) -> String {
    let path = format!("/tmp/{}", filename);
    let mut file = File::create(&path).expect("Failed to create temp file");
    file.write_all(content.as_bytes())
        .expect("Failed to write to temp file");
    path
}

static SAMPLE_OUTPUT: &str = "\
client,available,held,total,locked
1,1.5000,0.0000,1.5000,false
2,2.0000,0.0000,2.0000,false
";

static SAMPLE_OUTPUT_WITH_WITHDRAWAL: &str = "\
client,available,held,total,locked
1,1.5000,0.0000,1.5000,false
2,1.0000,0.0000,1.0000,false
";

#[test]
fn test_sample_transactions() {
    let output = run_file("./src/test/sample_transactions.csv", false);
    let expected_output = SAMPLE_OUTPUT;
    assert_eq!(output, expected_output);

    let output = run_file("./src/test/sample_transactions.csv", true);
    let expected_output = "client,available,held,total,locked\n";
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
    let expected_output = "client,available,held,total,locked\n";
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
    let expected_output = "client,available,held,total,locked\n";
    assert_eq!(output, expected_output);
}

// ============ RESOLVE TESTS ============

#[test]
fn test_dispute_and_resolve() {
    let csv_content = "type,client,tx,amount
deposit,1,1,100.0
dispute,1,1,
resolve,1,1,
";
    let path = create_temp_csv("test_dispute_resolve.csv", csv_content);
    let output = run_file(&path, false);

    // After dispute and resolve: available=100, held=0, total=100, locked=false
    assert!(output.contains("100.0000"));
    assert!(output.contains("0.0000"));
    assert!(
        !output.contains("true"),
        "Account should not be locked after resolve"
    );
}

#[test]
fn test_dispute_and_resolve_with_withdrawal() {
    let csv_content = "type,client,tx,amount
deposit,1,1,100.0
withdrawal,1,2,30.0
dispute,1,1,
resolve,1,1,
";
    let path = create_temp_csv("test_dispute_resolve_withdrawal.csv", csv_content);
    let output = run_file(&path, false);

    // available: 70 (100-30), held: 0 (resolved), total: 70
    assert!(output.contains("70.0000"));
}

#[test]
fn test_resolve_without_dispute_ignored() {
    let csv_content = "type,client,tx,amount
deposit,1,1,100.0
resolve,1,1,
";
    let path = create_temp_csv("test_resolve_no_dispute.csv", csv_content);
    let output = run_file(&path, false);

    // Resolve on non-disputed transaction should be ignored
    assert!(output.contains("100.0000")); // available unchanged
}

#[test]
fn test_multiple_disputes_and_resolves() {
    let csv_content = "type,client,tx,amount
deposit,1,1,100.0
deposit,1,2,50.0
dispute,1,1,
dispute,1,2,
resolve,1,1,
resolve,1,2,
";
    let path = create_temp_csv("test_multiple_disputes_resolves.csv", csv_content);
    let output = run_file(&path, false);

    // Both resolved: available=150, held=0, total=150
    assert!(output.contains("150.0000"));
    assert!(output.contains("0.0000"));
}

// ============ CHARGEBACK TESTS ============

#[test]
fn test_dispute_and_chargeback() {
    let csv_content = "type,client,tx,amount
deposit,1,1,100.0
dispute,1,1,
chargeback,1,1,
";
    let path = create_temp_csv("test_dispute_chargeback.csv", csv_content);
    let output = run_file(&path, false);

    // After chargeback: available=0, held=0, total=0, locked=true
    assert!(output.contains("0.0000"));
    assert!(
        output.contains("true"),
        "Account should be locked after chargeback"
    );
}

#[test]
fn test_chargeback_locks_account_from_further_operations() {
    let csv_content = "type,client,tx,amount
deposit,1,1,100.0
dispute,1,1,
chargeback,1,1,
deposit,1,2,50.0
";
    let path = create_temp_csv("test_chargeback_locks.csv", csv_content);
    let output = run_file(&path, false);

    // After chargeback, account is locked, so deposit is rejected
    // Total should be 0 (not 50)
    assert!(output.contains("true"), "Account should be locked");
    // Check that available is still 0 (deposit was rejected)
    let lines: Vec<&str> = output.lines().collect();
    let account_line = lines.iter().find(|line| line.starts_with("1,")).unwrap();
    assert!(account_line.contains("0.0000")); // No additional deposit applied
}

#[test]
fn test_chargeback_with_held_funds() {
    let csv_content = "type,client,tx,amount
deposit,1,1,100.0
deposit,1,2,50.0
dispute,1,1,
chargeback,1,1,
";
    let path = create_temp_csv("test_chargeback_with_held.csv", csv_content);
    let output = run_file(&path, false);

    // available: 50 (second deposit), held: 0, total: 50 (charged back 100)
    assert!(output.contains("50.0000"));
    assert!(output.contains("0.0000")); // held
    assert!(output.contains("true"));
}

#[test]
fn test_chargeback_on_non_disputed_ignored() {
    let csv_content = "type,client,tx,amount
deposit,1,1,100.0
chargeback,1,1,
";
    let path = create_temp_csv("test_chargeback_no_dispute.csv", csv_content);
    let output = run_file(&path, false);

    // Chargeback without dispute should be ignored
    assert!(output.contains("100.0000")); // available unchanged
    assert!(!output.contains("true"), "Account should not be locked");
}

// ============ DISPUTE -> RESOLVE/CHARGEBACK TRANSITIONS ============

#[test]
fn test_resolve_after_chargeback_ignored() {
    let csv_content = "type,client,tx,amount
deposit,1,1,100.0
dispute,1,1,
chargeback,1,1,
resolve,1,1,
";
    let path = create_temp_csv("test_resolve_after_chargeback.csv", csv_content);
    let output = run_file(&path, false);

    // Resolve after chargeback should be ignored, account stays locked
    assert!(output.contains("true"), "Account should be locked");
    assert!(output.contains("0.0000")); // total is 0 after chargeback
}

#[test]
fn test_chargeback_after_resolve_ignored() {
    let csv_content = "type,client,tx,amount
deposit,1,1,100.0
dispute,1,1,
resolve,1,1,
chargeback,1,1,
";
    let path = create_temp_csv("test_chargeback_after_resolve.csv", csv_content);
    let output = run_file(&path, false);

    // Chargeback after resolve should be ignored (no longer disputed)
    assert!(!output.contains("true"), "Account should not be locked");
    assert!(output.contains("100.0000")); // available unchanged from resolve
}

// ============ BATCH MODE TESTS ============

#[test]
fn test_resolve_with_insufficient_held_batch_mode() {
    let csv_content = "type,client,tx,amount
deposit,1,1,100.0
dispute,1,1,
resolve,1,1,
resolve,1,1,
deposit,1,2,50.0
";
    let path = create_temp_csv("test_resolve_insufficient_batch.csv", csv_content);
    let output = run_file(&path, true);

    // Batch mode ignores the second resolve error, continues with deposit
    // available: 150, held: 0, total: 150
    assert!(output.contains("150.0000"));
}

#[test]
fn test_chargeback_with_insufficient_held_batch_mode() {
    let csv_content = "type,client,tx,amount
deposit,1,1,100.0
dispute,1,1,
resolve,1,1,
chargeback,1,1,
deposit,1,2,50.0
";
    let path = create_temp_csv("test_chargeback_insufficient_batch.csv", csv_content);
    let output = run_file(&path, true);

    // Batch mode: chargeback fails (no held), account not locked, deposit succeeds
    assert!(output.contains("150.0000")); // 100 + 50
    assert!(
        !output.contains("true"),
        "Account should not be locked (chargeback failed)"
    );
}

// ============ COMPLEX WORKFLOWS ============

#[test]
fn test_complex_workflow_multiple_clients() {
    let csv_content = "type,client,tx,amount
deposit,1,1,100.0
deposit,2,2,200.0
dispute,1,1,
deposit,1,3,50.0
resolve,1,1,
withdrawal,2,4,50.0
deposit,2,5,100.0
";
    let path = create_temp_csv("test_complex_workflow.csv", csv_content);
    let output = run_file(&path, false);

    // Client 1: 100 (resolved) + 50 = 150, held = 0
    // Client 2: 200 - 50 + 100 = 250, held = 0
    assert!(output.contains("150.0000"));
    assert!(output.contains("250.0000"));
}

#[test]
fn test_partial_dispute_with_resolution() {
    let csv_content = "type,client,tx,amount
deposit,1,1,100.0
deposit,1,2,50.0
dispute,1,1,
withdrawal,1,3,40.0
resolve,1,1,
";
    let path = create_temp_csv("test_partial_dispute_resolve.csv", csv_content);
    let output = run_file(&path, false);

    // After dispute: available=50 (150-100), held=100
    // After withdrawal: available=10 (50-40), held=100
    // After resolve: available=110 (10+100), held=0, total=110
    assert!(output.contains("110.0000"));
}

#[test]
fn test_multiple_transactions_same_client_with_chargeback() {
    let csv_content = "type,client,tx,amount
deposit,1,1,100.0
deposit,1,2,100.0
deposit,1,3,100.0
dispute,1,2,
chargeback,1,2,
";
    let path = create_temp_csv("test_multiple_chargeback.csv", csv_content);
    let output = run_file(&path, false);

    // available: 200 (tx1 + tx3), held: 0, total: 200, locked: true
    assert!(output.contains("200.0000"));
    assert!(output.contains("true"));
}
