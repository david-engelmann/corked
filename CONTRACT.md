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
  and written for this repo. Nothing real is ever committed. Check, a
  manual sanitization review of every added fixture and example, with
  its result recorded in the gate report.

## First slice obligations, stub form

The full Contract for the first slice is the first build act of this repo. These stubs state the obligations the first slice will be held to, in the form they will be extended.

- CK-1. A review takes one diff and returns a verdict of accepted or sent back. A sent back verdict carries at least one objection and an accepted verdict carries none. Test, the first slice tests.
- CK-2. Every objection names the flaw, the standard it violates, and the rewrite that would be accepted, and an objection missing any of the three is not a valid objection. Test, the first slice tests.
- CK-3. Objections speak to naming, structure, and smell, and each objection states which of the three it is. Test, the first slice tests.
- CK-4. The review runs against recorded model output or local rules and never needs a paid API call. Test, the first slice tests.

## First slice obligations, full form

This section extends the stub obligations above. The stub text stands as the record of what was promised at setup, and this section states the same four obligations in the form the first slice is held to. Where the two readings disagree, this section governs.

- CK-1. A review is a JSON object holding diff_label and objections. The diff_label is a non empty string and objections is a list. The validate function in src/corked/review.py returns the list of defects it finds and never raises, whatever it is handed. A value that is not an object, a missing field, a field of the wrong type, and a blank string are each named as defects in that list. The verdict is accepted when the defect list is empty and the objections list is empty. The verdict is sent back when the defect list is empty and the objections list holds at least one objection. The verdict is invalid when the defect list is not empty, and an invalid review is never accepted and never sent back. Test, tests/test_review.py.
- CK-2. Every objection is an object holding flaw, standard, rewrite, and category. The flaw, the standard, and the rewrite are non empty strings. The category is one of naming, structure, and smell. An objection missing a field, holding a blank field, or holding a category outside the three is named in the defect list with its position in the objections list, and a review holding such an objection is invalid. The built in checker emits only complete objections, so a review built by the checker is never invalid for incompleteness. Test, tests/test_review.py and tests/test_checker.py.
- CK-3. Objections speak to naming, structure, and smell, and the built in checker in src/corked/checker.py raises four shapes and no others. The checker reads a unified diff as text and reads only added lines. An added line begins with a plus sign, and the three plus header line is not an added line. Removed lines and context lines are never read. Each shape raises at most one objection on one line, and objections come back in line order and, on one line, in the order the shapes are stated here. The first shape is a bare except clause on an added line, and it raises a smell objection whose rewrite is the same line with the bare clause replaced by a handler naming Exception. The second shape is a print call on an added line, and it raises a smell objection whose rewrite is the same line with the print call replaced by a logger.debug call. A print name that belongs to a longer name or to an attribute does not raise. The third shape is a mutable default on an added def line, and it raises a structure objection whose rewrite is the same def line with the default replaced by None. The mutable defaults seen are an empty list, an empty dict, and an empty set call, and they are seen on the def line itself. The fourth shape is a placeholder name, read as the target of an assignment or as the name of a function on a def line, and a placeholder is exactly one of data2, temp2, foo, bar, stuff, and thing. A placeholder raises a naming objection. The rewrite for a placeholder assignment draws its name from the assigned value when the right hand side is a simple call on a plain name or a literal. For a simple call the drawn name is the callee name with its leading word dropped when the callee name holds more than one word, and the callee name itself when it holds one word. For a literal the drawn name is text for a string, items for a list, mapping for a dict, and number for a number. When the drawn name would itself be a placeholder or a Python keyword, and in every other case, and for a placeholder function name, the rewrite invents no name and instead instructs the rename, stating that the name must say what the value holds or what the function does. The limits are part of the obligation. The checker reads text and not syntax, so a matching shape inside a string literal on an added line raises like code, a statement split across lines is read one line at a time, and shapes the four do not name never raise. Test, tests/test_checker.py.
- CK-4. The review runs against recorded model output or local rules and never needs a paid API call. The command line in src/corked/cli.py carries two commands. The check command reads a unified diff file, builds a review from the checker output under the file name as its label, and prints the rendering. The render command reads a recorded review as JSON and prints the rendering. The rendering in src/corked/render.py is one deterministic form shared by both commands, so the same review always renders byte identical output. The exit code is 0 for an accepted review, 1 for a sent back review, and 2 for invalid input, with the defects printed in the rendering. Input that cannot be read, and a render file that does not hold a JSON review, are invalid input. A recorded review committed under examples is held byte identical against its stored rendering by a golden test. Test, tests/test_cli.py and tests/test_render.py.

## Round 1 repair amendments

This section amends the obligations above. Each amendment names the obligation it amends and the review finding it closes, and the earlier text is left standing.

- FIX-CK-1. Amends CK-4 and closes spec finding S2 and red team finding M1. Input that cannot be decoded as UTF-8 text is input that cannot be read. Both commands treat input that cannot be decoded as invalid input, name the defect in the rendering, and exit with code 2. Test, tests/test_cli.py.
- FIX-CK-2. Amends CK-3 and closes spec finding S1. A drawn name that is empty, or that is not a usable Python identifier, is a case where no name may be invented, and the rewrite falls back to the instructed rename. Test, tests/test_checker.py.
- FIX-CK-3. Amends CK-3 and closes spec finding S3 and red team finding m2. The placeholder shape reads an annotated assignment target as an assignment target, and the rewrite replaces the name and preserves the annotation. A placeholder name among the targets of a tuple assignment also raises the naming objection, and its rewrite is the instructed rename, because a tuple binds several values and no single drawn name names the share the placeholder holds. Test, tests/test_checker.py.
- FIX-CK-4. Amends CK-3 and closes spec finding S4. The rewrite for a def line holding a mutable default replaces every mutable default the shape can see on that line with None, and not only the first. Test, tests/test_checker.py.
- FIX-CK-5. Amends CK-3 and closes spec finding S5. A set literal is none of the four named literal kinds. A brace literal draws the dict name only when it is empty or holds a top level colon, and every other brace literal falls under the other case, so the rewrite instructs the rename. Test, tests/test_checker.py.
- FIX-CK-6. Amends CK-3 and closes spec finding S6. The empty set call is seen with or without spaces between the name and the parentheses and inside them. Test, tests/test_checker.py.
- FIX-CK-7. Amends SETUP-3 and closes spec finding S7 and red team finding m3. The codepoint scan counts U+2014 and U+2013 in files whose suffix is .md, .txt, .py, .json, .toml, .yml, or .yaml, and in extensionless files, and it keeps skipping the ignored environment and build directories the scan already names. The action pins and the pyproject restatement made in the step 0 repair commit are recorded here as step 0 repairs. Test, tests/test_scan.py.
