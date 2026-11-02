"""Pins for CK-3. The four checker shapes, their rewrites, and their limits."""

from corked.checker import PLACEHOLDERS, check_diff_text
from corked.review import validate_review, verdict_for


def diff(*lines):
    return "\n".join(lines) + "\n"


def added(*contents):
    return diff("--- a/app.py", "+++ b/app.py", "@@ -1,1 +1,1 @@",
                *("+" + content for content in contents))


def test_bare_except_fires_with_a_handler_naming_exception():
    objections = check_diff_text(added("    except:"))
    assert len(objections) == 1
    assert objections[0]["category"] == "smell"
    assert objections[0]["rewrite"] == "    except Exception:"


def test_typed_except_does_not_fire():
    assert check_diff_text(added("    except ValueError:")) == []


def test_print_call_fires_with_a_logging_rewrite():
    objections = check_diff_text(added("    print(total)"))
    assert len(objections) == 1
    assert objections[0]["category"] == "smell"
    assert objections[0]["rewrite"] == "    logger.debug(total)"


def test_logging_call_does_not_fire():
    assert check_diff_text(added("    logger.debug(total)")) == []


def test_print_inside_a_longer_name_or_an_attribute_does_not_fire():
    assert check_diff_text(added("    sprint(total)")) == []
    assert check_diff_text(added("    printer(total)")) == []
    assert check_diff_text(added("    out.print(total)")) == []


def test_mutable_defaults_fire_with_the_none_sentinel_rewrite():
    cases = (
        ("def load(items=[]):", "def load(items=None):"),
        ("def load(cache={}):", "def load(cache=None):"),
        ("def load(seen=set()):", "def load(seen=None):"),
    )
    for line, rewrite in cases:
        objections = check_diff_text(added(line))
        assert len(objections) == 1, line
        assert objections[0]["category"] == "structure"
        assert objections[0]["rewrite"] == rewrite, line


def test_fresh_defaults_do_not_fire():
    assert check_diff_text(added("def load(items=None):")) == []
    assert check_diff_text(added("def load(count=0):")) == []
    assert check_diff_text(added("def load(name='x'):")) == []


def test_placeholder_assignment_draws_its_name_from_a_simple_call():
    objections = check_diff_text(added("foo = load_orders()"))
    assert len(objections) == 1
    assert objections[0]["category"] == "naming"
    assert objections[0]["rewrite"] == "orders = load_orders()"


def test_placeholder_assignment_drops_only_the_leading_word():
    objections = check_diff_text(added("bar = fetch_user_records()"))
    assert objections[0]["rewrite"] == "user_records = fetch_user_records()"


def test_placeholder_assignment_keeps_a_one_word_callee_as_the_name():
    objections = check_diff_text(added("foo = load()"))
    assert objections[0]["rewrite"] == "load = load()"


def test_placeholder_assignment_draws_its_name_from_literals():
    cases = (
        ('foo = "hello"', 'text = "hello"'),
        ("foo = [1, 2]", "items = [1, 2]"),
        ("foo = {'a': 1}", "mapping = {'a': 1}"),
        ("temp2 = 42", "number = 42"),
    )
    for line, rewrite in cases:
        objections = check_diff_text(added(line))
        assert len(objections) == 1, line
        assert objections[0]["rewrite"] == rewrite, line


def test_placeholder_rewrite_instructs_when_no_name_can_be_drawn():
    objections = check_diff_text(added("foo = other"))
    assert objections[0]["rewrite"] == (
        "Rename foo to a name that says what the value holds "
        "and keep the rest of the line as written"
    )


def test_placeholder_rewrite_instructs_when_the_drawn_name_is_a_placeholder():
    objections = check_diff_text(added("foo = thing()"))
    assert objections[0]["rewrite"].startswith("Rename foo to a name")


def test_placeholder_function_name_instructs_the_rename():
    objections = check_diff_text(added("def foo():"))
    assert len(objections) == 1
    assert objections[0]["category"] == "naming"
    assert objections[0]["rewrite"] == (
        "Rename foo to a name that says what the function does "
        "and keep the signature as written"
    )


def test_every_placeholder_fires_as_an_assignment_target():
    for name in PLACEHOLDERS:
        objections = check_diff_text(added(f"{name} = load()"))
        assert len(objections) == 1, name
        assert objections[0]["category"] == "naming", name


def test_descriptive_names_do_not_fire():
    assert check_diff_text(added("orders = load_orders()")) == []
    assert check_diff_text(added("def load_orders():")) == []


def test_removed_lines_never_fire():
    text = diff("--- a/app.py", "+++ b/app.py", "@@ -1,1 +1,1 @@",
                "-    print(total)", "-    except:", "-foo = load()")
    assert check_diff_text(text) == []


def test_context_lines_never_fire():
    text = diff("--- a/app.py", "+++ b/app.py", "@@ -1,1 +1,1 @@",
                "     print(total)", "     except:")
    assert check_diff_text(text) == []


def test_the_three_plus_header_is_not_an_added_line():
    text = diff("--- a/app.py", "+++ b/print(total).py", "@@ -1,0 +1,0 @@")
    assert check_diff_text(text) == []


def test_a_shape_inside_a_string_literal_raises_like_code():
    objections = check_diff_text(added('    label = "print(total)"'))
    assert len(objections) == 1
    assert objections[0]["category"] == "smell"


def test_objections_come_back_in_line_order():
    objections = check_diff_text(added("    print(total)", "    except:"))
    assert [o["rewrite"] for o in objections] == [
        "    logger.debug(total)",
        "    except Exception:",
    ]


def test_one_line_raises_at_most_one_objection_per_shape():
    objections = check_diff_text(added("def foo(items=[]):"))
    assert [o["category"] for o in objections] == ["structure", "naming"]


def test_checker_output_is_always_a_complete_valid_review():
    text = added("    except:", "    print(total)", "foo = load_orders()")
    objections = check_diff_text(text)
    built = {"diff_label": "checked change", "objections": objections}
    assert len(objections) == 3
    assert validate_review(built) == []
    assert verdict_for(built) == "sent back"


def test_a_clean_diff_raises_nothing():
    text = added("    orders = load_orders()", "    logger.debug(orders)")
    assert check_diff_text(text) == []


def test_placeholder_trailing_underscore_callee_instructs_the_rename():
    objections = check_diff_text(added("foo = load_()"))
    assert len(objections) == 1
    assert objections[0]["rewrite"] == (
        "Rename foo to a name that says what the value holds "
        "and keep the rest of the line as written"
    )


def test_placeholder_single_underscore_callee_instructs_the_rename():
    objections = check_diff_text(added("foo = _()"))
    assert len(objections) == 1
    assert objections[0]["rewrite"].startswith("Rename foo to a name")


def test_placeholder_keyword_stem_callee_instructs_the_rename():
    objections = check_diff_text(added("foo = class_()"))
    assert len(objections) == 1
    assert objections[0]["rewrite"].startswith("Rename foo to a name")


def test_placeholder_annotated_assignment_fires_and_keeps_the_annotation():
    objections = check_diff_text(added("foo: int = load_orders()"))
    assert len(objections) == 1
    assert objections[0]["category"] == "naming"
    assert objections[0]["rewrite"] == "orders: int = load_orders()"


def test_placeholder_annotated_assignment_keeps_a_one_word_callee():
    objections = check_diff_text(added("foo: int = load()"))
    assert len(objections) == 1
    assert objections[0]["rewrite"] == "load: int = load()"


def test_placeholder_in_a_tuple_target_instructs_the_rename():
    objections = check_diff_text(added("foo, other = load(), 1"))
    assert len(objections) == 1
    assert objections[0]["category"] == "naming"
    assert objections[0]["rewrite"].startswith("Rename foo to a name")


def test_placeholder_as_the_second_tuple_target_instructs_the_rename():
    objections = check_diff_text(added("other, foo = 1, load()"))
    assert len(objections) == 1
    assert objections[0]["rewrite"].startswith("Rename foo to a name")


def test_two_mutable_defaults_draw_one_objection_rewriting_both():
    objections = check_diff_text(added("def load(a=[], b={}):"))
    assert len(objections) == 1
    assert objections[0]["category"] == "structure"
    assert objections[0]["rewrite"] == "def load(a=None, b=None):"


def test_a_set_literal_instructs_the_rename():
    objections = check_diff_text(added("foo = {1, 2}"))
    assert len(objections) == 1
    assert objections[0]["rewrite"].startswith("Rename foo to a name")


def test_an_empty_dict_still_draws_the_dict_name():
    objections = check_diff_text(added("foo = {}"))
    assert len(objections) == 1
    assert objections[0]["rewrite"] == "mapping = {}"


def test_a_spaced_empty_set_call_fires_as_a_mutable_default():
    objections = check_diff_text(added("def load(seen=set ( )):"))
    assert len(objections) == 1
    assert objections[0]["category"] == "structure"
    assert objections[0]["rewrite"] == "def load(seen=None):"


def raw(*lines):
    return diff("--- a/app.py", "+++ b/app.py", "@@ -1,1 +1,1 @@", *lines)


CALL_REWRITE_END = " and build each replaced default inside the body under a None check"


def test_a_call_default_fires_with_the_none_check_rewrite():
    objections = check_diff_text(added("def stamp(at=now()):"))
    assert len(objections) == 1
    assert objections[0]["category"] == "structure"
    assert objections[0]["rewrite"] == "def stamp(at=None):" + CALL_REWRITE_END


def test_a_dotted_call_default_fires():
    objections = check_diff_text(added("def stamp(at=datetime.now()):"))
    assert len(objections) == 1
    assert objections[0]["rewrite"] == "def stamp(at=None):" + CALL_REWRITE_END


def test_a_call_default_with_keyword_arguments_fires():
    objections = check_diff_text(added("def load(at=now(tz=utc)):"))
    assert len(objections) == 1
    assert objections[0]["rewrite"] == "def load(at=None):" + CALL_REWRITE_END


def test_an_annotated_call_default_fires():
    objections = check_diff_text(added("def stamp(at: str = now()):"))
    assert len(objections) == 1
    assert objections[0]["rewrite"] == (
        "def stamp(at: str = None):" + CALL_REWRITE_END
    )


def test_two_call_defaults_draw_one_objection_rewriting_both():
    objections = check_diff_text(added("def stamp(a=now(), b=later()):"))
    assert len(objections) == 1
    assert objections[0]["rewrite"] == (
        "def stamp(a=None, b=None):" + CALL_REWRITE_END
    )


def test_a_mutable_default_and_a_call_default_fire_in_shape_order():
    objections = check_diff_text(added("def load(items=[], at=now()):"))
    assert [o["category"] for o in objections] == ["structure", "structure"]
    assert objections[0]["rewrite"] == "def load(items=None, at=now()):"
    assert objections[1]["rewrite"] == (
        "def load(items=[], at=None):" + CALL_REWRITE_END
    )


def test_the_empty_set_call_is_a_mutable_default_and_not_a_call_default():
    objections = check_diff_text(added("def load(seen=set()):"))
    assert len(objections) == 1
    assert objections[0]["rewrite"] == "def load(seen=None):"


def test_non_call_defaults_do_not_fire_the_call_default_shape():
    assert check_diff_text(added("def load(at=now):")) == []
    assert check_diff_text(added("def load(at=lambda: 1):")) == []
    assert check_diff_text(added("def load(at=now() + 1):")) == []
    assert check_diff_text(added("def load(at=(now())):")) == []


def test_a_call_default_on_a_placeholder_def_stands_after_the_naming():
    objections = check_diff_text(added("def foo(at=now()):"))
    assert [o["category"] for o in objections] == ["naming", "structure"]
    assert objections[1]["rewrite"] == "def foo(at=None):" + CALL_REWRITE_END


def test_a_signature_split_across_lines_reads_no_call_default():
    text = added("def load(at=now(),", "        other=1):")
    assert check_diff_text(text) == []


def test_a_class_attribute_list_literal_fires_with_the_move_rewrite():
    text = raw(" class Config:", "+    seen = []")
    objections = check_diff_text(text)
    assert len(objections) == 1
    assert objections[0]["category"] == "structure"
    assert objections[0]["flaw"] == (
        "The added line gives the class attribute seen "
        "a mutable literal that every instance shares"
    )
    assert objections[0]["rewrite"] == (
        "Move seen into the initializer as self.seen = [] "
        "and remove the class level assignment"
    )


def test_class_attribute_dict_and_set_literals_fire():
    cases = (
        ("    table = {'a': 1}", "self.table = {'a': 1}"),
        ("    tags = {1, 2}", "self.tags = {1, 2}"),
        ("    items = [1, 2]", "self.items = [1, 2]"),
    )
    for line, moved in cases:
        text = raw(" class Config:", "+" + line)
        objections = check_diff_text(text)
        assert len(objections) == 1, line
        name = line.strip().split(" ")[0]
        assert objections[0]["rewrite"] == (
            f"Move {name} into the initializer as {moved} "
            "and remove the class level assignment"
        ), line


def test_an_annotated_class_attribute_fires():
    text = raw(" class Config:", "+    seen: list = []")
    objections = check_diff_text(text)
    assert len(objections) == 1
    assert objections[0]["rewrite"] == (
        "Move seen into the initializer as self.seen = [] "
        "and remove the class level assignment"
    )


def test_a_class_with_bases_still_opens_a_class_body():
    text = raw(" class Config(Base, Mixin):", "+    seen = []")
    assert len(check_diff_text(text)) == 1


def test_a_module_level_literal_assignment_draws_nothing():
    assert check_diff_text(added("seen = []")) == []
    assert check_diff_text(added("table = {'a': 1}")) == []


def test_a_literal_assignment_inside_a_function_inside_a_class_draws_nothing():
    text = raw(" class Config:", "     def build(self):", "+        cache = []")
    assert check_diff_text(text) == []


def test_a_literal_assignment_inside_a_class_inside_a_function_fires():
    text = raw(" def build():", "     class Inner:", "+        seen = []")
    objections = check_diff_text(text)
    assert len(objections) == 1
    assert objections[0]["category"] == "structure"


def test_a_literal_assignment_inside_a_nested_class_fires():
    text = raw(" class Outer:", "     class Inner:", "+        seen = []")
    assert len(check_diff_text(text)) == 1


def test_a_literal_assignment_under_a_conditional_in_a_class_fires():
    text = raw(" class Config:", "     if DEBUG:", "+        seen = []")
    assert len(check_diff_text(text)) == 1


def test_a_second_hunk_without_the_class_line_carries_no_class_context():
    text = diff(
        "--- a/app.py", "+++ b/app.py",
        "@@ -1,2 +1,2 @@", " class Config:", "     value = 1",
        "@@ -20,1 +20,2 @@", "     other = 2", "+    seen = []",
    )
    assert check_diff_text(text) == []


def test_a_removed_class_line_opens_no_class_body():
    text = raw("-class Config:", "+    seen = []")
    assert check_diff_text(text) == []


def test_a_class_level_call_or_scalar_assignment_draws_nothing():
    for line in ("    seen = list()", "    count = 0", "    name = 'x'",
                 "    seen = [1] + [2]"):
        text = raw(" class Config:", "+" + line)
        assert check_diff_text(text) == [], line


def test_a_placeholder_in_a_class_body_draws_naming_then_structure():
    text = raw(" class Config:", "+    foo = []")
    objections = check_diff_text(text)
    assert [o["category"] for o in objections] == ["naming", "structure"]
    assert objections[0]["rewrite"] == "    items = []"
    assert objections[1]["rewrite"] == (
        "Move foo into the initializer as self.foo = [] "
        "and remove the class level assignment"
    )


def test_a_method_def_line_in_a_class_still_draws_the_def_shapes():
    text = raw(" class Config:", "+    def build(self, items=[]):")
    objections = check_diff_text(text)
    assert len(objections) == 1
    assert objections[0]["rewrite"] == "    def build(self, items=None):"


def test_a_one_line_class_draws_nothing():
    assert check_diff_text(added("class Config: seen = []")) == []


def test_a_blank_context_line_inside_a_class_body_closes_nothing():
    text = diff(
        "--- a/app.py", "+++ b/app.py", "@@ -1,1 +1,1 @@",
        " class Config:", "     value = 1", " ", "+    items = []",
    )
    objections = check_diff_text(text)
    assert len(objections) == 1
    assert objections[0]["rewrite"] == (
        "Move items into the initializer as self.items = [] "
        "and remove the class level assignment"
    )


def test_an_added_blank_line_inside_a_class_body_closes_nothing():
    text = raw(" class Config:", "+", "+    items = []")
    assert len(check_diff_text(text)) == 1


def test_the_written_forms_of_a_blank_line_read_the_same_way():
    forms = (
        (" class Config:", "", "+    items = []"),
        (" class Config:", " ", "+    items = []"),
        (" class Config:", "   ", "+    items = []"),
    )
    for lines in forms:
        assert len(check_diff_text(raw(*lines))) == 1, lines


def test_a_tab_only_context_line_inside_a_class_body_closes_nothing():
    text = raw(" class Config:", " \t", "+    items = []")
    objections = check_diff_text(text)
    assert len(objections) == 1
    assert objections[0]["rewrite"] == (
        "Move items into the initializer as self.items = [] "
        "and remove the class level assignment"
    )


def test_a_tab_only_added_line_inside_a_class_body_closes_nothing():
    text = raw(" class Config:", "+\t", "+    items = []")
    assert len(check_diff_text(text)) == 1


def test_a_call_default_holding_an_escaped_quote_fires():
    objections = check_diff_text(added('def f(a=make("a\\")b")):'))
    assert len(objections) == 1
    assert objections[0]["rewrite"] == "def f(a=None):" + CALL_REWRITE_END


def test_a_call_default_after_a_string_holding_an_escaped_quote_fires():
    objections = check_diff_text(added('def f(a="x\\")y", b=now()):'))
    assert len(objections) == 1
    assert objections[0]["rewrite"] == (
        'def f(a="x\\")y", b=None):' + CALL_REWRITE_END
    )


def test_a_class_attribute_literal_holding_an_escaped_quote_fires():
    text = raw(" class Config:", '+    x = ["a\\" ] ", "b"]')
    objections = check_diff_text(text)
    assert len(objections) == 1
    assert objections[0]["rewrite"] == (
        'Move x into the initializer as self.x = ["a\\" ] ", "b"] '
        "and remove the class level assignment"
    )


def test_a_placeholder_dict_holding_an_escaped_quote_draws_the_dict_name():
    objections = check_diff_text(added('foo = {"a\\"b": 1}'))
    assert len(objections) == 1
    assert objections[0]["rewrite"] == 'mapping = {"a\\"b": 1}'


def test_a_plain_file_header_empties_the_class_stack():
    text = diff(
        "--- a/x.py", "+++ b/x.py", "@@ -1,1 +1,1 @@",
        " class Config:", "+    seen = []",
        "--- a/y.py", "+++ b/y.py",
        "+    items = []",
    )
    objections = check_diff_text(text)
    assert len(objections) == 1
    assert objections[0]["rewrite"] == (
        "Move seen into the initializer as self.seen = [] "
        "and remove the class level assignment"
    )
