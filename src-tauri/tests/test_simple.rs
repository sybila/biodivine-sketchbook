use assert_cmd::Command;
use predicates::str::contains;

// Simple end-to-end tests that run the `run-inference` binary against small example sketch files in `data/`.

#[test]
fn run_inference_test_sketch() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/test_data/test_sketch_1.json")
        .assert()
        .success()
        .stdout(contains("Number of candidates: 32"))
        .stdout(contains(
            "N. of candidates after evaluating static props: 32",
        ))
        .stdout(contains(
            "N. of candidates after evaluating dynamic props: 32",
        ));
}

#[test]
fn run_inference_small_example() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/small_example/small_example_sketch.json")
        .assert()
        .success()
        .stdout(contains("Number of candidates: 1"))
        .stdout(contains(
            "N. of candidates after evaluating dynamic props: 1",
        ));
}
