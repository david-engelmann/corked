# stickler

A strict reviewer of code and naming. It sends back work that falls short, names the flaw and the standard it violates, and states the rewrite that would pass.

## What this is

Most agents accept whatever you hand them and whatever they produce. Stickler does not. It reads every line against a named standard and sends back whatever falls short of it. A variable named like a shrug, a function doing three jobs while pretending to do one, an abstraction built for an audience that will never arrive. Stickler names the flaw, states the standard it violates, and holds the line until the work meets that standard.

## Status

Seeded in October 2026. Setup has landed, and Rounds 1 to 4 have closed, with all four slices merged on main. Part of the Spectacle line in the 39 repo storefront perfection programme.

## Programme status

- Closed perfection rounds, 4 of 100. A round counts only when a full cycle of work, audit, review, red team, and approval has closed. Seeding and setup are not rounds.
- Contract, SETUP-1 to SETUP-4 and CK-1 to CK-6 met. CK-1 to CK-4 stand in full form in CONTRACT.md with the Round 1 repair amendments, and Round 1 closed on the merged slice 2 after three reviews, a repair pass, and an approving re-review.
- Contract, CK-5 and CK-6 met. They stand in full form in CONTRACT.md with the Round 2 repair amendments, and Round 2 closed on the merged slice 3 after three reviews, a repair pass, and an approving re-review.
- Slice 2 has landed. It carries the review core, the built in checker with its four named shapes, the shared rendering, and the check and render commands, all running offline on recorded or local input. The first slice is a strict review of one diff against named standards, objections named with the rewrite each one would accept.
- Slice 3 has landed. It carries the two remaining definition time shapes, the call default and the class attribute with its class context read from the diff itself, and the set level review with the review command, over one synthetic change set under examples.
- Contract, CK-7 and CK-8 met. They stand in full form in CONTRACT.md with the Round 3 repair amendments, and Round 3 closed on the merged slice 4 after three reviews, a repair pass, and an approving re-review.
- Slice 4 has landed. It carries the borderline flagging with its three patterns, the verdict seat reading both decision model routes, the recorded verdict path through the set review, and the recorded borderline pilot under examples, with the suite running offline on the recorded envelopes.
- Contract, CK-9 and CK-10 met. They stand in full form in CONTRACT.md with the Round 4 repair amendments, and Round 4 closed on the merged slice 5 after three reviews, a repair pass, and an approving re-review.
- Slice 5 has landed. It carries the contested line reading of the set review, with the contested section before the summary and the contested count in the summary, and the constructed contested example under examples, with the suite running offline on the recorded pilot and the constructed example.
- Local gate, python3 scripts/codepoint_scan.py and python3 -m pytest tests -q. CI runs the same commands.

## Licence

MIT. See LICENSE.
