# corked Contract

This Contract is the executable spec for corked. Every obligation is checked by a named test or by a named check. The Contract is reported as met or not met in the README programme status.

## Setup obligations

- SETUP-1. The repo carries AGENTS.md with the canonical template verbatim
  above its marked line and the repo annex below it. Test,
  tests/test_setup.py.
- SETUP-2. The README programme status states the closed round count out
  of 100 and reports this Contract as met or not met. Test,
  tests/test_setup.py.
- SETUP-3. The codepoint scan at scripts/codepoint_scan.py counts U+2014
  and U+2013 in every Markdown and text file in the repo and requires
  zero of each. The scan runs in the local gate and in CI.
- SETUP-4. Every fixture and example committed to this repo is synthetic
  and written for this repo. Nothing real is ever committed.

## First slice obligations, stub form

The full Contract for the first slice is the first build act of this repo. These stubs state the obligations the first slice will be held to, in the form they will be extended.

- CK-1. A review takes one diff and returns a verdict of accepted or sent back. A sent back verdict carries at least one objection and an accepted verdict carries none. Test, the first slice tests.
- CK-2. Every objection names the flaw, the standard it violates, and the rewrite that would be accepted, and an objection missing any of the three is not a valid objection. Test, the first slice tests.
- CK-3. Objections speak to naming, structure, and smell, and each objection states which of the three it is. Test, the first slice tests.
- CK-4. The review runs against recorded model output or local rules and never needs a paid API call. Test, the first slice tests.
