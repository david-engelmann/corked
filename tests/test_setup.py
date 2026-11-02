"""Setup pins. Each test can fail for a real reason against a repo that
lost a setup file or let its status drift from its contract."""
import pathlib, re
ROOT = pathlib.Path(__file__).resolve().parent.parent

def test_agents_carries_the_canonical_marker():
    text = (ROOT / "AGENTS.md").read_text(encoding="utf-8")
    assert "<!-- repo annex below this line -->" in text
    assert "39 repo storefront perfection program" in text

def test_readme_states_the_closed_round_count():
    text = (ROOT / "README.md").read_text(encoding="utf-8")
    assert re.search(r"Closed perfection rounds, 2 of 100", text)

def test_contract_names_the_borderline_test_file_for_ck_7():
    text = (ROOT / "CONTRACT.md").read_text(encoding="utf-8")
    assert "Test, tests/test_borderline.py and tests/test_verdicts.py" in text

def test_contract_carries_the_setup_obligations():
    text = (ROOT / "CONTRACT.md").read_text(encoding="utf-8")
    for oid in ("SETUP-1", "SETUP-2", "SETUP-3", "SETUP-4", "CK-1"):
        assert oid in text, oid

def test_readme_reports_the_contract_status():
    text = (ROOT / "README.md").read_text(encoding="utf-8")
    expected = (
        "- Contract, SETUP-1 to SETUP-4 and CK-1 to CK-6 met. "
        "CK-1 to CK-4 stand in full form in CONTRACT.md with the Round 1 "
        "repair amendments, and Round 1 closed on the merged slice 2 after "
        "three reviews, a repair pass, and an approving re-review."
    )
    assert expected in text.splitlines()

def test_readme_reports_the_round_3_standing():
    text = (ROOT / "README.md").read_text(encoding="utf-8")
    standing = (
        "- Contract, CK-7 and CK-8 now stand in full form in "
        "CONTRACT.md. Their standing is not claimed here, as Round 3 "
        "has not closed."
    )
    assert standing in text.splitlines()
    slice_line = (
        "- Slice 4 is built and in review. It carries the borderline "
        "flagging with its three patterns, the verdict seat reading "
        "both decision model routes, the recorded verdict path "
        "through the set review, and the recorded borderline pilot "
        "under examples, with the suite running offline on the "
        "recorded envelopes."
    )
    assert slice_line in text.splitlines()
    assert text.count("built and in review") == 1

def test_ci_runs_the_same_gate_as_the_local_gate():
    text = (ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
    live = "\n".join(
        line.split("#", 1)[0]
        for line in text.splitlines()
        if not line.lstrip().startswith("#")
    )
    assert "python3 scripts/codepoint_scan.py" in live
    assert "python3 -m pytest tests -q" in live
