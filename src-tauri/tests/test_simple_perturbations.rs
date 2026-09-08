use assert_cmd::Command;
use predicates::str::contains;

// Simplend-to-end tests that run the `run-inference` binary against small sketches with perturbations in `data/test_data/perturbations/`.

#[test]
fn run_inference_pertubation_1v_sat() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/test_data/perturbations/example-perturbed-1v-sat.json")
        .assert()
        .success()
        .stdout(contains("Number of candidates: 1"));
}

#[test]
fn run_inference_pertubation_2v_sat() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/test_data/perturbations/example-perturbed-2v-sat.json")
        .assert()
        .success()
        .stdout(contains("Number of candidates: 1"));
}

#[test]
fn run_inference_pertubation_1v_unsat() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/test_data/perturbations/example-perturbed-1v-unsat.json")
        .assert()
        .success()
        .stdout(contains("Number of candidates: 0"));
}

#[test]
fn run_inference_pertubation_2v_unsat() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/test_data/perturbations/example-perturbed-2v-unsat.json")
        .assert()
        .success()
        .stdout(contains("Number of candidates: 0"));
}

#[test]
fn run_inference_myeloid_unperturbed() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/test_data/perturbations/example-myeloid-unperturbed.json")
        .assert()
        .success()
        .stdout(contains("Number of candidates: 41"));
}

#[test]
fn run_inference_myeloid_perturbed_only() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/test_data/perturbations/example-myeloid-perturbed-only.json")
        .assert()
        .success()
        .stdout(contains("Number of candidates: 9"));
}

#[test]
fn run_inference_myeloid_combined() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/test_data/perturbations/example-myeloid-combined.json")
        .assert()
        .success()
        .stdout(contains("Number of candidates: 4"));
}
