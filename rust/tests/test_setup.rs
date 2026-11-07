//! Setup pins. Each test can fail for a real reason against a
//! repo that lost a setup file or let its status drift from
//! its contract. Translated from tests/test_setup.py, with the
//! pins the port adds for its own status line, contract
//! section, and CI step.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(root().join(relative)).unwrap()
}

#[test]
fn agents_carries_the_canonical_marker() {
    let text = read("AGENTS.md");
    assert!(text.contains("<!-- repo annex below this line -->"));
    assert!(text.contains("39 repo storefront perfection program"));
}

#[test]
fn readme_states_the_closed_round_count() {
    let text = read("README.md");
    assert!(text.contains("Closed perfection rounds, 8 of 100"));
}

#[test]
fn contract_names_the_borderline_test_file_for_ck_7() {
    let text = read("CONTRACT.md");
    assert!(text.contains("Test, tests/test_borderline.py and tests/test_verdicts.py"));
}

#[test]
fn contract_carries_the_setup_obligations() {
    let text = read("CONTRACT.md");
    for oid in ["SETUP-1", "SETUP-2", "SETUP-3", "SETUP-4", "CK-1"] {
        assert!(text.contains(oid), "obligation {oid}");
    }
}

#[test]
fn readme_reports_the_contract_status() {
    let text = read("README.md");
    let expected = "- Contract, SETUP-1 to SETUP-4 and CK-1 to CK-6 met. \
        CK-1 to CK-4 stand in full form in CONTRACT.md with the Round 1 \
        repair amendments, and Round 1 closed on the merged slice 2 after \
        three reviews, a repair pass, and an approving re-review.";
    assert!(
        text.lines().any(|line| line == expected),
        "the Round 1 status line stands"
    );
}

#[test]
fn readme_reports_the_round_3_standing() {
    let text = read("README.md");
    let standing = "- Contract, CK-7 and CK-8 met. They stand in full form in \
        CONTRACT.md with the Round 3 repair amendments, and Round 3 \
        closed on the merged slice 4 after three reviews, a repair \
        pass, and an approving re-review.";
    assert!(text.lines().any(|line| line == standing));
    let slice_line = "- Slice 4 has landed. It carries the borderline \
        flagging with its three patterns, the verdict seat reading \
        both decision model routes, the recorded verdict path \
        through the set review, and the recorded borderline pilot \
        under examples, with the suite running offline on the \
        recorded envelopes.";
    assert!(text.lines().any(|line| line == slice_line));
    assert!(!text.contains("Slice 4 is built and in review"));
}

#[test]
fn readme_reports_the_round_4_standing() {
    let text = read("README.md");
    let standing = "- Contract, CK-9 and CK-10 met. They stand in full form in \
        CONTRACT.md with the Round 4 repair amendments, and Round 4 \
        closed on the merged slice 5 after three reviews, a repair \
        pass, and an approving re-review.";
    assert!(text.lines().any(|line| line == standing));
    let slice_line = "- Slice 5 has landed. It carries the contested \
        line reading of the set review, with the contested section \
        before the summary and the contested count in the summary, \
        and the constructed contested example under examples, with \
        the suite running offline on the recorded pilot and the \
        constructed example.";
    assert!(text.lines().any(|line| line == slice_line));
    // Adjusted with the Round 5 slice, as the named mechanical
    // consequence of slice 6 standing in review. The count of
    // slices standing in review was one while Round 5 was
    // built, until the Round 5 closure landed slice 6 and the
    // count returned to zero. The Round 5 pin below holds the
    // standing line and the slice line themselves. Adjusted
    // again with the Round 6 slice, as the named mechanical
    // consequence of slice 7 standing in review, so the count
    // stood at one while Round 6 was built, until the Round 6
    // closure landed slice 7 and the count returned to zero.
    // Adjusted again with the Round 7 slice, as the named
    // mechanical consequence of slice 8 standing in review,
    // so the count stood at one while Round 7 was built,
    // until the Round 7 closure landed slice 8 and the
    // count returned to zero. Adjusted again with the
    // Round 8 slice, as the named mechanical consequence
    // of slice 9 standing in review, so the count stood
    // at one while Round 8 was built, until the Round 8
    // closure landed slice 9 and the count returned to zero.
    // Adjusted again with the Round 9 slice, as the named
    // mechanical consequence of slice 10 standing in
    // review, so the count stands at one while Round 9
    // is built.
    assert_eq!(text.matches("built and in review").count(), 1);
}

#[test]
fn readme_reports_the_round_5_standing() {
    let text = read("README.md");
    let standing = "- Contract, CK-11 and CK-12 met. They stand in full form in \
        CONTRACT.md, and Round 5 closed on the merged slice 6 after \
        three reviews, a repair pass, and an approving owner gate.";
    assert!(text.lines().any(|line| line == standing));
    let slice_line = "- Slice 6 has landed. It carries the \
        judging census over a stated collection of recorded review \
        inputs, with the census command beside check, render, and \
        review, and the census example under examples standing over \
        the Round 2 set, the pilot set, and the constructed Round 4 \
        example named as constructed, with the suite running offline \
        on the recorded and constructed sets.";
    assert!(text.lines().any(|line| line == slice_line));
    assert!(text.contains("Closed perfection rounds, 8 of 100"));
}

#[test]
fn readme_reports_the_round_6_standing() {
    let text = read("README.md");
    let standing = "- Contract, CK-13 and CK-14 met. They stand in full form in \
        CONTRACT.md with the Round 6 repair amendment, and Round 6 \
        closed on the merged slice 7 after three reviews, a repair \
        pass, and an approving re-review.";
    assert!(text.lines().any(|line| line == standing));
    let slice_line = "- Slice 7 has landed. It carries the \
        rejudging run over the pilot flags through the verdict seat's \
        Jev route, recorded as the rejudged set under examples, and \
        the rejudged ledger reading over a stated pair of recorded \
        sets, with the rejudged command beside check, render, review, \
        and census, with the suite running offline on the recorded \
        envelopes and sets.";
    assert!(text.lines().any(|line| line == slice_line));
    assert!(text.contains("Closed perfection rounds, 8 of 100"));
}

#[test]
fn readme_reports_the_round_7_standing() {
    let text = read("README.md");
    let standing = "- Contract, CK-15 and CK-16 met. They stand in full form in \
        CONTRACT.md, and Round 7 closed on the merged slice 8 after three \
        reviews, a repair pass, and an approving re-review.";
    assert!(text.lines().any(|line| line == standing));
    let slice_line = "- Slice 8 has landed. It carries the split \
        reading over one recorded set, stating the judgement of each of \
        its flags under the first named route alone, under the second \
        named route alone, and under both routes jointly, with the split \
        example over the pilot set under examples, and the split command \
        beside check, render, review, census, and rejudged, with the \
        suite running offline on the recorded verdicts.";
    assert!(text.lines().any(|line| line == slice_line));
    assert!(text.contains("Closed perfection rounds, 8 of 100"));
}

#[test]
fn readme_reports_the_round_8_standing() {
    let text = read("README.md");
    let standing = "- Contract, CK-17 and CK-18 met. They stand in full form in \
        CONTRACT.md with the Round 8 repair amendments, and Round 8 \
        closed on the merged slice 9 after three reviews, a repair \
        pass, and an approving re-review.";
    assert!(text.lines().any(|line| line == standing));
    let slice_line = "- Slice 9 has landed. It carries the \
        contested reading over a recorded collection of change sets, one \
        section per set in collection order, naming each set's contested \
        flags with file, line, and pattern, or the no contest line where \
        a set holds none, with per set flags read and contested counts \
        and closing counts over the whole collection, with the contested \
        example under examples standing over the same collection as the \
        census example, and the contested command beside check, render, \
        review, census, rejudged, and split, with the suite running \
        offline on the recorded verdicts.";
    assert!(text.lines().any(|line| line == slice_line));
    assert!(text.contains("Closed perfection rounds, 8 of 100"));
}

#[test]
fn readme_reports_the_round_9_standing() {
    let text = read("README.md");
    let standing = "- Contract, CK-19 and CK-20 stated. They stand in full form in \
        CONTRACT.md, and Round 9 stands open on slice 10 in review.";
    assert!(text.lines().any(|line| line == standing));
    let slice_line = "- Slice 10 is built and in review. It carries the \
        pattern reading over a recorded collection of change sets, one \
        section per borderline pattern in the order the contract states them, \
        naming each flag of the collection standing under that pattern \
        with file, line, and the set it stands in, with per section flags \
        read and contested counts and closing counts over the whole \
        collection, with the pattern example under examples standing \
        over the same collection as the census example, and the pattern \
        command beside check, render, review, census, rejudged, split, \
        and contested, and the library fencing closure over the census, \
        rejudged, split, and set review renderings, with the suite \
        running offline on the recorded verdicts.";
    assert!(text.lines().any(|line| line == slice_line));
    assert!(text.contains("Closed perfection rounds, 8 of 100"));
}

#[test]
fn ci_runs_the_same_gate_as_the_local_gate() {
    let text = read(".github/workflows/ci.yml");
    let live = live_lines(&text);
    // Adjusted with the removal slice of PORT-9, as the named
    // mechanical consequence of the Python gate leaving CI. The
    // two pins that read the Python commands in CI are replaced
    // by pins that read their absence, so the cargo gate stands
    // alone under guard rather than merely unopposed.
    assert!(!live.contains("python3 scripts/codepoint_scan.py"));
    assert!(!live.contains("python3 -m pytest tests -q"));
    assert!(!live.contains("setup-python"));
}

/// The live text of a file, comment text removed the way the
/// Python pin removes it.
fn live_lines(text: &str) -> String {
    text.lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .map(|line| line.split('#').next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn readme_reports_the_rust_port() {
    let text = read("README.md");
    // Adjusted with the removal slice of PORT-9, as the named
    // mechanical consequence of the port landing. The pinned
    // line changes from the in review wording to the landed
    // wording the slice writes.
    let port_line = "- The Rust port has landed. \
        It carries the behavior the Python package carried, \
        checked against the same goldens, and the Python tree \
        stands removed under PORT-9.";
    assert!(
        text.lines().any(|line| line == port_line),
        "the port status line stands"
    );
}

#[test]
fn contract_carries_the_port_obligations() {
    let text = read("CONTRACT.md");
    for oid in [
        "PORT-1", "PORT-2", "PORT-3", "PORT-4", "PORT-5", "PORT-6", "PORT-7", "PORT-8", "PORT-9",
    ] {
        assert!(text.contains(oid), "obligation {oid}");
    }
}

#[test]
fn ci_runs_the_cargo_gate_beside_the_python_gate() {
    let text = read(".github/workflows/ci.yml");
    let live = live_lines(&text);
    assert!(live.contains("cargo test"), "the cargo step stands");
    assert!(live.contains("cargo fmt --check"), "the format step stands");
    assert!(
        live.contains("cargo clippy --all-targets -- -D warnings"),
        "the clippy step stands"
    );
}
