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
    assert re.search(r"Closed perfection rounds, 0 of 100", text)

def test_contract_carries_the_setup_obligations():
    text = (ROOT / "CONTRACT.md").read_text(encoding="utf-8")
    for oid in ("SETUP-1", "SETUP-2", "SETUP-3", "SETUP-4", "CK-1"):
        assert oid in text, oid

def test_ci_runs_the_same_gate_as_the_local_gate():
    text = (ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
    assert "codepoint_scan.py" in text
    assert "pytest" in text
