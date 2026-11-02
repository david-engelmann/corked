"""Pins for the borderline flagging of CK-7.

A flagged line draws no judgement. The flag names the pattern,
carries the line as written, and carries the reading the flag
computes, and a line that raises any judgement under the six
shapes is never flagged.
"""

import unittest

from stickler.checker import check_diff_detailed, check_diff_flagged


def flagged(*added):
    diff = "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,9 @@\n"
    diff += "".join(f"+{line}\n" for line in added)
    return check_diff_flagged(diff)


class BorderlineFlagTests(unittest.TestCase):
    def test_assignment_near_name_flags_with_the_drawn_name(self):
        _judgements, flags = flagged("data = load_entries()")
        self.assertEqual(len(flags), 1)
        self.assertEqual(flags[0]["pattern"], "near placeholder name")
        self.assertEqual(flags[0]["reading"], "drawn name entries")
        self.assertEqual(flags[0]["content"], "data = load_entries()")

    def test_near_name_without_a_drawn_name(self):
        _judgements, flags = flagged("temp = left + right")
        self.assertEqual(flags[0]["reading"], "no drawn name")

    def test_function_name_near_placeholder(self):
        _judgements, flags = flagged("def tmp(rows):")
        self.assertEqual(len(flags), 1)
        self.assertEqual(flags[0]["pattern"], "near placeholder name")
        self.assertEqual(flags[0]["reading"], "no drawn name")

    def test_tuple_member_near_placeholder(self):
        _judgements, flags = flagged("data, extra = left, right")
        self.assertEqual(len(flags), 1)
        self.assertEqual(flags[0]["pattern"], "near placeholder name")

    def test_annotated_target_near_placeholder(self):
        _judgements, flags = flagged("tmp: list = []")
        self.assertEqual(len(flags), 1)
        self.assertEqual(flags[0]["reading"], "drawn name items")

    def test_the_named_placeholders_are_judged_not_flagged(self):
        for line in ("foo = 5", "data2 = 5", "thing = []"):
            judgements, flags = flagged(line)
            self.assertEqual(
                [shape for shape, _o in judgements], ["placeholder name"]
            )
            self.assertEqual(flags, [])

    def test_a_name_outside_both_lists_is_neither(self):
        judgements, flags = flagged("datum = 5")
        self.assertEqual(judgements, [])
        self.assertEqual(flags, [])

    def test_broad_handler_flags(self):
        _judgements, flags = flagged("    except BaseException:")
        self.assertEqual(len(flags), 1)
        self.assertEqual(flags[0]["pattern"], "broad handler")
        self.assertEqual(
            flags[0]["reading"], "handler names BaseException"
        )

    def test_broad_handler_with_a_binding_flags(self):
        _judgements, flags = flagged("except BaseException as err:")
        self.assertEqual(len(flags), 1)

    def test_a_named_handler_is_neither_judged_nor_flagged(self):
        judgements, flags = flagged("    except Exception:")
        self.assertEqual(judgements, [])
        self.assertEqual(flags, [])

    def test_base_exception_group_is_not_the_broad_handler(self):
        _judgements, flags = flagged("    except BaseExceptionGroup:")
        self.assertEqual(flags, [])

    def test_shared_member_default_kinds(self):
        cases = (
            ("def f(cache=([],)):", "default holds a list literal"),
            ('def f(pair=({"a": 1}, 2)):', "default holds a dict literal"),
            ("def f(seen=({1, 2},)):", "default holds a set literal"),
        )
        for line, reading in cases:
            judgements, flags = flagged(line)
            self.assertEqual(judgements, [])
            self.assertEqual(len(flags), 1)
            self.assertEqual(
                flags[0]["pattern"], "shared member default"
            )
            self.assertEqual(flags[0]["reading"], reading)

    def test_a_tuple_of_plain_values_is_not_flagged(self):
        judgements, flags = flagged("def f(span=(0, 30)):")
        self.assertEqual(judgements, [])
        self.assertEqual(flags, [])

    def test_a_string_holding_brackets_is_not_a_member(self):
        # A naive bracket count ends the tuple inside the string.
        # The walk is quote aware, so the member is the string and
        # no container member stands.
        judgements, flags = flagged('def f(pair=("]",)):')
        self.assertEqual(judgements, [])
        self.assertEqual(flags, [])

    def test_a_tuple_call_default_is_judged_not_flagged(self):
        judgements, flags = flagged("def f(pair=tuple([1])):")
        self.assertEqual(
            [shape for shape, _o in judgements], ["call default"]
        )
        self.assertEqual(flags, [])

    def test_a_judged_line_is_never_flagged(self):
        judgements, flags = flagged("tmp = print(x)")
        self.assertEqual(
            [shape for shape, _o in judgements], ["print call"]
        )
        self.assertEqual(flags, [])

    def test_a_mutable_default_on_the_line_blocks_the_flag(self):
        judgements, flags = flagged("def f(a=[], pair=([],)):")
        self.assertEqual(
            [shape for shape, _o in judgements], ["mutable default"]
        )
        self.assertEqual(flags, [])

    def test_a_class_attribute_line_is_judged_not_flagged(self):
        diff = (
            "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,2 @@\n"
            " class C:\n+    data = []\n"
        )
        judgements, flags = check_diff_flagged(diff)
        self.assertEqual(
            [shape for shape, _o in judgements], ["class attribute"]
        )
        self.assertEqual(flags, [])

    def test_at_most_one_flag_stands_on_a_line(self):
        _judgements, flags = flagged("def temp(cache=([],)):")
        self.assertEqual(len(flags), 1)
        # The name pattern is read first in the stated order.
        self.assertEqual(flags[0]["pattern"], "near placeholder name")

    def test_flags_stand_in_line_order(self):
        _judgements, flags = flagged(
            "    except BaseException:", "data = load_entries()"
        )
        self.assertEqual(
            [flag["pattern"] for flag in flags],
            ["broad handler", "near placeholder name"],
        )
        self.assertLess(flags[0]["line"], flags[1]["line"])

    def test_flagged_judgements_match_the_detailed_walk(self):
        diff = (
            "--- a/x.py\n+++ b/x.py\n@@ -1,1 +1,3 @@\n"
            "+print(x)\n+data = load_entries()\n"
        )
        judgements, _flags = check_diff_flagged(diff)
        self.assertEqual(judgements, check_diff_detailed(diff))


if __name__ == "__main__":
    unittest.main()
