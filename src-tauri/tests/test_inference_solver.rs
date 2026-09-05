use assert_cmd::Command;
use predicates::str::contains;

// End-to-end tests that run the `run-inference` binary against sketch files in `data/`.
// These mirror the integration test setup in biodivine-algo-smt-inference and are meant
// to grow into broader coverage (including perturbation cases) over time.

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only end-to-end test")]
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
#[cfg_attr(debug_assertions, ignore = "release-only end-to-end test")]
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

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only end-to-end test")]
fn run_inference_tlgl() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/real_cases/tlgl/tlgl_sketch.json")
        .assert()
        .success()
        .stdout(contains(
            "N. of candidates before evaluating any properties: 72057594037927936",
        ))
        .stdout(contains(
            "N. of candidates after evaluating static props: 1296",
        ))
        .stdout(contains(
            "N. of candidates after evaluating dynamic props: 486",
        ))
        .stdout(contains("Number of candidates: 486"));
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only end-to-end test")]
fn run_inference_tlgl_hctl() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/real_cases/tlgl/tlgl_sketch_hctl.json")
        .assert()
        .success()
        .stdout(contains(
            "N. of candidates before evaluating any properties: 72057594037927936",
        ))
        .stdout(contains(
            "N. of candidates after evaluating static props: 1296",
        ))
        .stdout(contains(
            "N. of candidates after evaluating dynamic props: 486",
        ))
        .stdout(contains("Number of candidates: 486"));
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only end-to-end test")]
fn run_inference_arabidopsis() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/real_cases/arabidopsis/arabidopsis_sketch.json")
        .assert()
        .success()
        .stdout(contains(
            "N. of candidates before evaluating any properties: 295147905179352825856",
        ))
        .stdout(contains(
            "N. of candidates after evaluating static props: 4761711360",
        ))
        .stdout(contains(
            "N. of candidates after evaluating dynamic props: 439296",
        ))
        .stdout(contains("Number of candidates: 439296"));
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only end-to-end test")]
fn run_inference_arabidopsis_hctl() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/real_cases/arabidopsis/arabidopsis_sketch_hctl.json")
        .assert()
        .success()
        .stdout(contains(
            "N. of candidates before evaluating any properties: 295147905179352825856",
        ))
        .stdout(contains(
            "N. of candidates after evaluating static props: 4761711360",
        ))
        .stdout(contains(
            "N. of candidates after evaluating dynamic props: 439296",
        ))
        .stdout(contains("Number of candidates: 439296"));
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only end-to-end test")]
fn run_inference_arabidopsis_with_additional_prop() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/real_cases/arabidopsis/arabidopsis_with_additional_prop.json")
        .assert()
        .success()
        .stdout(contains(
            "N. of candidates before evaluating any properties: 295147905179352825856",
        ))
        .stdout(contains(
            "N. of candidates after evaluating static props: 4761711360",
        ))
        .stdout(contains(
            "N. of candidates after evaluating dynamic props: 48352",
        ))
        .stdout(contains("Number of candidates: 48352"));
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only end-to-end test")]
fn run_inference_myeloid_fully_specified() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/other_sketches/myeloid/myeloid-fully-defined-krumsiek.json")
        .assert()
        .success()
        .stdout(contains(
            "N. of candidates before evaluating any properties: 1",
        ))
        .stdout(contains(
            "N. of candidates after evaluating static props: 1",
        ))
        .stdout(contains(
            "N. of candidates after evaluating dynamic props: 0",
        ))
        .stdout(contains("Number of candidates: 0"));
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only end-to-end test")]
fn run_inference_myeloid_psbn() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/other_sketches/myeloid/myeloid-sketch-no-updates.json")
        .assert()
        .success()
        .stdout(contains(
            "N. of candidates before evaluating any properties: 19807040628566084398385987584",
        ))
        .stdout(contains(
            "N. of candidates after evaluating static props: 48642052608",
        ))
        .stdout(contains(
            "N. of candidates after evaluating dynamic props: 5451624",
        ))
        .stdout(contains("Number of candidates: 5451624"));
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only end-to-end test")]
fn run_inference_fgf_small() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/other_sketches/fgf_signalling/fgf_sketch.json")
        .assert()
        .success()
        .stdout(contains(
            "N. of candidates before evaluating any properties: 4096",
        ))
        .stdout(contains(
            "N. of candidates after evaluating static props: 8",
        ))
        .stdout(contains(
            "N. of candidates after evaluating dynamic props: 1",
        ))
        .stdout(contains("Number of candidates: 1"));
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only end-to-end test")]
fn run_inference_benchmark_celldivb() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/benchmarks/celldivb/celldivb_sketch.aeon")
        .arg("--input-format=aeon")
        .assert()
        .success()
        .stdout(contains(
            "N. of candidates before evaluating any properties: 16777216",
        ))
        .stdout(contains(
            "N. of candidates after evaluating static props: 64000",
        ))
        .stdout(contains(
            "N. of candidates after evaluating dynamic props: 14088",
        ))
        .stdout(contains("Number of candidates: 14088"));
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only end-to-end test")]
fn run_inference_benchmark_eprotein() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/benchmarks/eprotein/eprotein_sketch.aeon")
        .arg("--input-format=aeon")
        .assert()
        .success()
        .stdout(contains(
            "N. of candidates before evaluating any properties: 18889465931478580854784",
        ))
        .stdout(contains(
            "N. of candidates after evaluating static props: 944784",
        ))
        .stdout(contains(
            "N. of candidates after evaluating dynamic props: 1008",
        ))
        .stdout(contains("Number of candidates: 1008"));
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release-only end-to-end test")]
fn run_inference_benchmark_nsp4() {
    Command::cargo_bin("run-inference")
        .unwrap()
        .arg("../data/benchmarks/nsp4/nsp4_sketch.aeon")
        .arg("--input-format=aeon")
        .assert()
        .success()
        .stdout(contains(
            "N. of candidates before evaluating any properties: 75557863725914323419136",
        ))
        .stdout(contains(
            "N. of candidates after evaluating static props: 1179648",
        ))
        .stdout(contains(
            "N. of candidates after evaluating dynamic props: 128",
        ))
        .stdout(contains("Number of candidates: 128"));
}
