# stickler

Stickler is a command line tool that reviews code against named rules, checks a diff with a built in checker, and prints readings over verdicts that decision model routes recorded while judging a change set against those rules.

Stickler takes one file per command and prints one rendering. Every command runs offline, and no command calls a model. The recorded verdicts the readings run over were produced in earlier rounds by the routes they name and stand committed under examples as fixtures.

[![ci](https://github.com/david-engelmann/stickler/actions/workflows/ci.yml/badge.svg)](https://github.com/david-engelmann/stickler/actions/workflows/ci.yml)

```text
Pattern

Pattern, near placeholder name
harvest.py, line 6, set pilot set
harvest.py, line 7, set pilot set
sorting.py, line 6, set pilot set
sorting.py, line 11, set pilot set
sorting.py, line 12, set pilot set
ledger.py, line 6, set constructed round 4 example
ledger.py, line 7, set constructed round 4 example
cache.py, line 15, set constructed round 4 example
cache.py, line 19, set constructed round 4 example
Flags read, 9
Contested, 2

Pattern, broad handler
harvest.py, line 12, set pilot set
harvest.py, line 18, set pilot set
sorting.py, line 15, set pilot set
cache.py, line 10, set constructed round 4 example
Flags read, 4
Contested, 0

Pattern, shared member default
windows.py, line 6, set pilot set
windows.py, line 10, set pilot set
windows.py, line 14, set pilot set
windows.py, line 18, set pilot set
cache.py, line 7, set constructed round 4 example
Flags read, 5
Contested, 0

Sets read, 3
Flags read, 18
Contested flags, 2
```

## Install

```sh
git clone https://github.com/david-engelmann/stickler.git
cd stickler
cargo install --path .
```

## Quickstart

Run the pattern command over the pattern example from the root of the clone.

```sh
stickler pattern examples/pattern.json
```

```text
Pattern

Pattern, near placeholder name
harvest.py, line 6, set pilot set
harvest.py, line 7, set pilot set
sorting.py, line 6, set pilot set
sorting.py, line 11, set pilot set
sorting.py, line 12, set pilot set
ledger.py, line 6, set constructed round 4 example
ledger.py, line 7, set constructed round 4 example
cache.py, line 15, set constructed round 4 example
cache.py, line 19, set constructed round 4 example
Flags read, 9
Contested, 2

Pattern, broad handler
harvest.py, line 12, set pilot set
harvest.py, line 18, set pilot set
sorting.py, line 15, set pilot set
cache.py, line 10, set constructed round 4 example
Flags read, 4
Contested, 0

Pattern, shared member default
windows.py, line 6, set pilot set
windows.py, line 10, set pilot set
windows.py, line 14, set pilot set
windows.py, line 18, set pilot set
cache.py, line 7, set constructed round 4 example
Flags read, 5
Contested, 0

Sets read, 3
Flags read, 18
Contested flags, 2
```

## Usage

The nine commands each read one file and print one rendering. The examples below run over the files under examples, and every output block is captured from the command it follows.

The check command reads a unified diff and builds a review with the built in checker. The checker holds a fixed set of rules over the shapes of code a diff can carry, and each objection it raises names the rule the code breaks and the change that would clear the objection.

```sh
stickler check examples/smelly.diff
```

```text
Review of smelly.diff
Verdict, sent back
Objections, 2

Objection 1, smell
Flaw, The added line catches every exception with a bare except clause
Standard, A handler names the failures it expects so the failures nobody expected still rise where they can be seen
Rewrite,     except Exception:

Objection 2, smell
Flaw, The added line sends diagnostics through a print call
Standard, Diagnostics travel through the logging module so they can be levelled, filtered, and silenced
Rewrite,         logger.debug(total)
```

The render command reads a review recorded as JSON and prints the review rendering from the record.

```sh
stickler render examples/recorded_review.json
```

```text
Review of orders change, recorded review
Verdict, sent back
Objections, 2

Objection 1, naming
Flaw, The added line binds the order rows to the placeholder name foo
Standard, A name says what its value holds so the reader never has to hold the assignment in memory to follow the code
Rewrite, rows = load_rows()

Objection 2, smell
Flaw, The added line catches every exception with a bare except clause
Standard, A handler names the failures it expects so the failures nobody expected still rise where they can be seen
Rewrite,     except Exception:
```

The review command reads a change set. A change set lists flags, and each flag names a file, a line, and a pattern. The change set also carries the verdicts on its flags, recorded by decision model routes through the verdict seat. The set review prints the judgement of each flag, the lines where the routes that judged one flag disagree as the contested lines, and a summary tally of the verdicts.

```sh
stickler review examples/change_set.json
```

```text
Change set review
Files, 3

File, basket.py
Judgements, 2

Judgement 1, structure
Flaw, The added line gives the class attribute seen a mutable literal that every instance shares
Standard, A mutable value that belongs to each instance is built in the initializer, so one instance changing it never changes it for every other instance
Rewrite, Move seen into the initializer as self.seen = [] and remove the class level assignment

Judgement 2, structure
Flaw, The added def line gives a parameter a default that is a call, so the call runs once when the definition is read and every call that omits the argument shares the one result
Standard, A default that must be computed on each call is written as None and the value is built inside the body under a None check
Rewrite,     def stamp(self, at=None): and build each replaced default inside the body under a None check

File, report.py
Judgements, 3

Judgement 1, structure
Flaw, The added def line gives a parameter a mutable default that every call shares
Standard, A default that must be fresh on each call is written as None and the value is built inside the body
Rewrite, def render(total, history=None):

Judgement 2, smell
Flaw, The added line catches every exception with a bare except clause
Standard, A handler names the failures it expects so the failures nobody expected still rise where they can be seen
Rewrite,     except Exception:

Judgement 3, smell
Flaw, The added line sends diagnostics through a print call
Standard, Diagnostics travel through the logging module so they can be levelled, filtered, and silenced
Rewrite,         logger.debug(total)

File, tidy.py
Clean

Summary
Judgements, 5
Shape, bare except, 1
Shape, print call, 1
Shape, mutable default, 1
Shape, call default, 1
Shape, class attribute, 1
Files with judgements, worst first
Worst, report.py, 3
Worst, basket.py, 2
```

The census command reads a census file that names a collection of recorded review inputs and prints the judging census. The census gives a tally of the judgements for each set in the collection and a total tally over the whole collection, counting files, judgements, and verdicts.

```sh
stickler census examples/census.json
```

```text
Census
Sets, 3

Set, round 2 set
Files, 3
Judgements, 5
Shape, bare except, 1
Shape, print call, 1
Shape, mutable default, 1
Shape, placeholder name, 0
Shape, call default, 1
Shape, class attribute, 1
Borderline, 0
Pattern, near placeholder name, 0
Pattern, broad handler, 0
Pattern, shared member default, 0
Verdicts, none recorded

Set, pilot set
Files, 3
Judgements, 0
Shape, bare except, 0
Shape, print call, 0
Shape, mutable default, 0
Shape, placeholder name, 0
Shape, call default, 0
Shape, class attribute, 0
Borderline, 12
Pattern, near placeholder name, 5
Pattern, broad handler, 3
Pattern, shared member default, 4
Verdict lines, stickler 11, clean 0
Unjudged, 1
Contested, 0

Set, constructed round 4 example
Files, 2
Judgements, 0
Shape, bare except, 0
Shape, print call, 0
Shape, mutable default, 0
Shape, placeholder name, 0
Shape, call default, 0
Shape, class attribute, 0
Borderline, 6
Pattern, near placeholder name, 4
Pattern, broad handler, 1
Pattern, shared member default, 1
Verdict lines, stickler 3, clean 2
Unjudged, 1
Contested, 2

Total
Sets, 3
Files, 8
Judgements, 5
Shape, bare except, 1
Shape, print call, 1
Shape, mutable default, 1
Shape, placeholder name, 0
Shape, call default, 1
Shape, class attribute, 1
Borderline, 18
Pattern, near placeholder name, 9
Pattern, broad handler, 4
Pattern, shared member default, 5
Verdict lines, stickler 14, clean 2
Unjudged, 2
Contested, 2
```

The rejudged command reads a file that names two recorded sets of verdicts over the same flags and prints the rejudged ledger. The ledger states the judgement of each flag in the first set and in the second set, and whether that judgement held or changed between the two sets.

```sh
stickler rejudged examples/rejudged.json
```

```text
Rejudged
First, pilot set
Second, rejudged set

harvest.py, line 6, near placeholder name
First, unjudged
Second, stickler
Changed

harvest.py, line 7, near placeholder name
First, stickler
Second, stickler
Held

harvest.py, line 12, broad handler
First, stickler
Second, stickler
Held

harvest.py, line 18, broad handler
First, stickler
Second, stickler
Held

windows.py, line 6, shared member default
First, stickler
Second, stickler
Held

windows.py, line 10, shared member default
First, stickler
Second, stickler
Held

windows.py, line 14, shared member default
First, stickler
Second, stickler
Held

windows.py, line 18, shared member default
First, stickler
Second, stickler
Held

sorting.py, line 6, near placeholder name
First, stickler
Second, stickler
Held

sorting.py, line 11, near placeholder name
First, stickler
Second, unjudged
Changed

sorting.py, line 12, near placeholder name
First, stickler
Second, stickler
Held

sorting.py, line 15, broad handler
First, stickler
Second, stickler
Held

Inputs read, 24
Standing in both, 12
Held, 10
Changed, 2
```

The split command reads a file that names one recorded set and two routes and prints the split reading. The split reading states the judgement of each flag under the first route alone, under the second route alone, and under both routes jointly, and it closes with counts of the flags read, the flags where the routes agree, the flags where they differ, and the flags where the joint judgement differs from a route.

```sh
stickler split examples/split.json
```

```text
Split
First route, jev-1.13.0
Second route, openai/gpt-6-luna

harvest.py, line 6, near placeholder name
First route, unjudged
Second route, unjudged
Joint, unjudged

harvest.py, line 7, near placeholder name
First route, stickler
Second route, unjudged
Joint, stickler

harvest.py, line 12, broad handler
First route, stickler
Second route, stickler
Joint, stickler

harvest.py, line 18, broad handler
First route, stickler
Second route, stickler
Joint, stickler

windows.py, line 6, shared member default
First route, stickler
Second route, stickler
Joint, stickler

windows.py, line 10, shared member default
First route, stickler
Second route, stickler
Joint, stickler

windows.py, line 14, shared member default
First route, stickler
Second route, stickler
Joint, stickler

windows.py, line 18, shared member default
First route, stickler
Second route, stickler
Joint, stickler

sorting.py, line 6, near placeholder name
First route, stickler
Second route, stickler
Joint, stickler

sorting.py, line 11, near placeholder name
First route, unjudged
Second route, stickler
Joint, stickler

sorting.py, line 12, near placeholder name
First route, stickler
Second route, stickler
Joint, stickler

sorting.py, line 15, broad handler
First route, stickler
Second route, stickler
Joint, stickler

Flags read, 12
Routes agree, 10
Routes differ, 2
Joint differs from a route, 2
```

The contested command reads a collection in the census form and prints the contested reading. The reading states one section per set in collection order, naming each contested flag with its file, line, and pattern, or the no contest line where a set holds none, and it closes with counts over the whole collection.

```sh
stickler contested examples/contested.json
```

```text
Contested

Set, round 2 set
No contested lines
Flags read, 0
Contested, 0

Set, pilot set
No contested lines
Flags read, 12
Contested, 0

Set, constructed round 4 example
ledger.py, line 6, near placeholder name
cache.py, line 15, near placeholder name
Flags read, 6
Contested, 2

Sets read, 3
Flags read, 18
Contested flags, 2
Sets with contested flags, 1
```

The pattern command reads a collection in the census form and prints the pattern reading. The reading states one section per borderline pattern, naming each flag of the collection standing under that pattern with its file, line, and the set it stands in, and it closes with counts over the whole collection.

```sh
stickler pattern examples/pattern.json
```

```text
Pattern

Pattern, near placeholder name
harvest.py, line 6, set pilot set
harvest.py, line 7, set pilot set
sorting.py, line 6, set pilot set
sorting.py, line 11, set pilot set
sorting.py, line 12, set pilot set
ledger.py, line 6, set constructed round 4 example
ledger.py, line 7, set constructed round 4 example
cache.py, line 15, set constructed round 4 example
cache.py, line 19, set constructed round 4 example
Flags read, 9
Contested, 2

Pattern, broad handler
harvest.py, line 12, set pilot set
harvest.py, line 18, set pilot set
sorting.py, line 15, set pilot set
cache.py, line 10, set constructed round 4 example
Flags read, 4
Contested, 0

Pattern, shared member default
windows.py, line 6, set pilot set
windows.py, line 10, set pilot set
windows.py, line 14, set pilot set
windows.py, line 18, set pilot set
cache.py, line 7, set constructed round 4 example
Flags read, 5
Contested, 0

Sets read, 3
Flags read, 18
Contested flags, 2
```

The files command reads a collection in the census form and prints the file reading. The reading states one section per file standing in the collection, naming each flag standing on that file with its line, its pattern, and the set it stands in, and it closes with counts over the whole collection.

```sh
stickler files examples/files.json
```

```text
Files

File, ledger.py
line 6, near placeholder name, set constructed round 4 example
line 7, near placeholder name, set constructed round 4 example
line 6, shared member default, set constructed round 10 example
line 9, broad handler, set constructed round 10 example
Flags read, 4
Contested, 2

File, cache.py
line 7, shared member default, set constructed round 4 example
line 10, broad handler, set constructed round 4 example
line 15, near placeholder name, set constructed round 4 example
line 19, near placeholder name, set constructed round 4 example
Flags read, 4
Contested, 1

File, tidy.py
line 6, near placeholder name, set constructed round 10 example
Flags read, 1
Contested, 0

Sets read, 2
Files read, 3
Flags read, 9
Contested flags, 3
```

## Status

Seeded in October 2026. Setup has landed, and Rounds 1 to 9 have closed, with all slices merged on main. Part of the Spectacle line in the 39 repo storefront perfection programme.

## Programme status

- Closed perfection rounds, 9 of 100. A round counts only when a full cycle of work, audit, review, red team, and approval has closed. Seeding and setup are not rounds.
- Contract, SETUP-1 to SETUP-4 and CK-1 to CK-6 met. CK-1 to CK-4 stand in full form in CONTRACT.md with the Round 1 repair amendments, and Round 1 closed on the merged slice 2 after three reviews, a repair pass, and an approving re-review.
- Contract, CK-5 and CK-6 met. They stand in full form in CONTRACT.md with the Round 2 repair amendments, and Round 2 closed on the merged slice 3 after three reviews, a repair pass, and an approving re-review.
- Slice 2 has landed. It carries the review core, the built in checker with its four named shapes, the shared rendering, and the check and render commands, all running offline on recorded or local input. The first slice is a strict review of one diff against named standards, objections named with the rewrite each one would accept.
- Slice 3 has landed. It carries the two remaining definition time shapes, the call default and the class attribute with its class context read from the diff itself, and the set level review with the review command, over one synthetic change set under examples.
- Contract, CK-7 and CK-8 met. They stand in full form in CONTRACT.md with the Round 3 repair amendments, and Round 3 closed on the merged slice 4 after three reviews, a repair pass, and an approving re-review.
- Slice 4 has landed. It carries the borderline flagging with its three patterns, the verdict seat reading both decision model routes, the recorded verdict path through the set review, and the recorded borderline pilot under examples, with the suite running offline on the recorded envelopes.
- Contract, CK-9 and CK-10 met. They stand in full form in CONTRACT.md with the Round 4 repair amendments, and Round 4 closed on the merged slice 5 after three reviews, a repair pass, and an approving re-review.
- Slice 5 has landed. It carries the contested line reading of the set review, with the contested section before the summary and the contested count in the summary, and the constructed contested example under examples, with the suite running offline on the recorded pilot and the constructed example.
- Contract, CK-11 and CK-12 met. They stand in full form in CONTRACT.md, and Round 5 closed on the merged slice 6 after three reviews, a repair pass, and an approving owner gate.
- Slice 6 has landed. It carries the judging census over a stated collection of recorded review inputs, with the census command beside check, render, and review, and the census example under examples standing over the Round 2 set, the pilot set, and the constructed Round 4 example named as constructed, with the suite running offline on the recorded and constructed sets.
- Contract, CK-13 and CK-14 met. They stand in full form in CONTRACT.md with the Round 6 repair amendment, and Round 6 closed on the merged slice 7 after three reviews, a repair pass, and an approving re-review.
- Slice 7 has landed. It carries the rejudging run over the pilot flags through the verdict seat's Jev route, recorded as the rejudged set under examples, and the rejudged ledger reading over a stated pair of recorded sets, with the rejudged command beside check, render, review, and census, with the suite running offline on the recorded envelopes and sets.
- Contract, CK-15 and CK-16 met. They stand in full form in CONTRACT.md, and Round 7 closed on the merged slice 8 after three reviews, a repair pass, and an approving re-review.
- Slice 8 has landed. It carries the split reading over one recorded set, stating the judgement of each of its flags under the first named route alone, under the second named route alone, and under both routes jointly, with the split example over the pilot set under examples, and the split command beside check, render, review, census, and rejudged, with the suite running offline on the recorded verdicts.
- Contract, CK-17 and CK-18 met. They stand in full form in CONTRACT.md with the Round 8 repair amendments, and Round 8 closed on the merged slice 9 after three reviews, a repair pass, and an approving re-review.
- Slice 9 has landed. It carries the contested reading over a recorded collection of change sets, one section per set in collection order, naming each set's contested flags with file, line, and pattern, or the no contest line where a set holds none, with per set flags read and contested counts and closing counts over the whole collection, with the contested example under examples standing over the same collection as the census example, and the contested command beside check, render, review, census, rejudged, and split, with the suite running offline on the recorded verdicts.
- Contract, CK-19 and CK-20 met. They stand in full form in CONTRACT.md with the FIX-CK-21 fencing closure, and Round 9 closed on the merged slice 10 after three reviews approving with no findings, with the red team's one observation on the single review rendering carried to Round 10 as a fencing component.
- Slice 10 has landed. It carries the pattern reading over a recorded collection of change sets, one section per borderline pattern in the order the contract states them, naming each flag of the collection standing under that pattern with file, line, and the set it stands in, with per section flags read and contested counts and closing counts over the whole collection, with the pattern example under examples standing over the same collection as the census example, and the pattern command beside check, render, review, census, rejudged, split, and contested, and the library fencing closure over the census, rejudged, split, and set review renderings, with the suite running offline on the recorded verdicts.
- Contract, CK-21 and CK-22 stated. They stand in full form in CONTRACT.md, and Round 10 stands open on slice 11 in review.
- Slice 11 is built and in review. It carries the file reading over a recorded collection of change sets, one section per file in first appearance order across the collection, naming each flag standing on that file with line, pattern, and the set it stands in, with per file flags read and contested counts and closing counts over the whole collection, with the constructed files example under examples standing over two constructed sets sharing one file, and the files command beside check, render, review, census, rejudged, split, contested, and pattern, and the review rendering fencing closure over the single review rendering, and the README brought to the fleet README standard, with the suite running offline on the recorded verdicts.
- The Rust port has landed. It carries the behavior the Python package carried, checked against the same goldens, and the Python tree stands removed under PORT-9.
- Local gate, cargo test, cargo fmt --check, and cargo clippy --all-targets -- -D warnings. CI runs the same commands.

## Licence

MIT. See LICENSE.
