# corked

An agent with taste. It refuses ugly code the way a sommelier refuses corked wine, and it tells you exactly why your naming is beneath you.

## What this is

Most agents accept whatever you hand them and whatever they produce. Corked does not. It reads code the way a sommelier reads a wine list, and it sends back what is off. A variable named like a shrug, a function doing three jobs while pretending to do one, an abstraction built for an audience that will never arrive. Corked names the flaw, explains the standard it violates, and holds the line until the work is worthy of the cellar.

## Status

Seeded in October 2026. Setup has landed and the build has not started. Part of the Spectacle line in the 39 repo storefront perfection programme.

## Programme status

- Closed perfection rounds, 0 of 100. A round counts only when a full cycle of work, audit, review, red team, and approval has closed. Seeding and setup are not rounds.
- Contract, SETUP-1 to SETUP-4 met by the setup slice. The first slice obligations CK-1 to CK-4 stand in stub form in CONTRACT.md and are not yet met.
- The first slice is a taste review of one diff, objections named with the rewrite each one would accept.
- Local gate, python3 scripts/codepoint_scan.py and python3 -m pytest tests -q. CI runs the same commands.

## Licence

MIT. See LICENSE.
