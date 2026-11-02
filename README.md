# corked

An agent with taste. It refuses ugly code the way a sommelier refuses corked wine, and it tells you exactly why your naming is beneath you.

## What this is

Most agents accept whatever you hand them and whatever they produce. Corked does not. It reads code the way a sommelier reads a wine list, and it sends back what is off. A variable named like a shrug, a function doing three jobs while pretending to do one, an abstraction built for an audience that will never arrive. Corked names the flaw, explains the standard it violates, and holds the line until the work is worthy of the cellar.

## Status

Seeded in October 2026. Setup has landed and Round 1 has closed, with slice 2 merged on main. Part of the Spectacle line in the 39 repo storefront perfection programme.

## Programme status

- Closed perfection rounds, 1 of 100. A round counts only when a full cycle of work, audit, review, red team, and approval has closed. Seeding and setup are not rounds.
- Contract, SETUP-1 to SETUP-4 and CK-1 to CK-4 met. CK-1 to CK-4 stand in full form in CONTRACT.md with the Round 1 repair amendments, and Round 1 closed on the merged slice 2 after three reviews, a repair pass, and an approving re-review.
- Contract, CK-5 and CK-6 now stand in full form in CONTRACT.md under the Round 2 obligations, and their standing is not claimed here, as Round 2 has not closed.
- Slice 2 has landed. It carries the review core, the built in checker with its four named shapes, the shared rendering, and the check and render commands, all running offline on recorded or local input. The first slice is a taste review of one diff, objections named with the rewrite each one would accept.
- Slice 3 is built and in review. It carries the two remaining definition time shapes, the call default and the class attribute with its class context read from the diff itself, and the set level review with the review command, over one synthetic change set under examples.
- Local gate, python3 scripts/codepoint_scan.py and python3 -m pytest tests -q. CI runs the same commands.

## Licence

MIT. See LICENSE.
