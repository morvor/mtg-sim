//! The decisions log (`docs/DECISIONS.md`): its parsing (dates, attributions, the user's
//! words, the "Revisit later" section) and that the committed file is current.

use mtg_data::decisions::{self, Date, Decider};
use mtg_data::interpretations::{parse_file, Entry};
use std::collections::BTreeSet;
use std::path::Path;

fn date(month: u8, day: u8) -> Date {
    Date {
        year: None,
        month,
        day,
    }
}

fn entry(decision: &str, why: &str) -> Entry {
    let text = format!(
        "## some-id: A title\n- Kind: policy\n- Status: decided\n- Decision: {decision}\n- Why: {why}\n"
    );
    parse_file(Path::new("x/area.md"), &text).remove(0)
}

/// The register ids a test's entries can name: `some-id` (the entry's own) and `other-id`.
fn ids() -> BTreeSet<&'static str> {
    ["some-id", "other-id"].into_iter().collect()
}

#[test]
fn dates_are_normalized() {
    let d = |s: &str| decisions::parse_date(s).map(|(d, _)| d);
    assert_eq!(d("Nov 5"), Some(date(11, 5)));
    assert_eq!(d("Nov 5. Yes"), Some(date(11, 5)));
    assert_eq!(d("on Nov 5: follow"), Some(date(11, 5)));
    assert_eq!(d(" Nov 19 (as recommended)"), Some(date(11, 19)));
    assert_eq!(d("November 5"), Some(date(11, 5)));
    assert_eq!(d("Nov. 5th"), Some(date(11, 5)));
    assert_eq!(d("Sept 3"), Some(date(9, 3)));
    assert_eq!(d("5 Nov"), Some(date(11, 5)));
    assert_eq!(d("Oct 21, for now"), Some(date(10, 21)));
    let with_year = Date {
        year: Some(2026),
        month: 11,
        day: 5,
    };
    assert_eq!(d("2026-11-05"), Some(with_year));
    assert_eq!(d("Nov 5, 2026"), Some(with_year));
    assert_eq!(d("5 November 2026"), Some(with_year));
    // A year the same as the default orders and shows like no year.
    assert_eq!(with_year.key(), date(11, 5).key());
    assert_eq!(with_year.to_string(), "Nov 5");
    let other = Date {
        year: Some(2025),
        month: 12,
        day: 1,
    };
    assert_eq!(other.to_string(), "Dec 1, 2025");
    assert!(other.key() < date(1, 2).key());
    // Not dates.
    for s in [
        "",
        "Novel 5",
        "No 5",
        "Nov 32",
        "Nov",
        "the user",
        "Nov 5a",
        "2026-13-01",
    ] {
        assert_eq!(d(s), None, "{s:?}");
    }
    // The length taken.
    assert_eq!(
        decisions::parse_date("on Nov 12, for now"),
        Some((date(11, 12), 9))
    );
}

#[test]
fn user_attributions_are_read_with_their_date() {
    for (decision, d) in [
        ("Decided by the user, Nov 4. Yes.", date(11, 4)),
        (
            "Decided by the user on Nov 5: follow the ruling.",
            date(11, 5),
        ),
        ("Decided by the user Nov 9. No.", date(11, 9)),
        (
            "Decided by the user, Nov 12 (as recommended). It deals the damage.",
            date(11, 12),
        ),
        (
            "Decided by the user, Nov 12, for now (accepting the recommendation): \"I agree\".",
            date(11, 12),
        ),
        (
            "Yes, always. Decided by the user, Oct 21: follow 118.5.",
            date(10, 21),
        ),
    ] {
        let a = decisions::attribute(&entry(decision, "Because."));
        assert_eq!(a.decider, Decider::User(d), "{decision}");
        assert_eq!(a.field, "Decision");
    }
    let a = decisions::attribute(&entry("Decided by the user, Nov 4. Yes.", ""));
    assert_eq!(a.clause, "Decided by the user, Nov 4");
    // Read from the Why line when the Decision line names no one.
    let a = decisions::attribute(&entry(
        "Draw.",
        "An unendable loop is a draw. Decided by the user, Oct 21: an unendable loop ends the game.",
    ));
    assert_eq!(a.decider, Decider::User(date(10, 21)));
    assert_eq!(a.field, "Why");
}

#[test]
fn sources_coordinator_and_unstated_attributions() {
    let who = |decision: &str| decisions::attribute(&entry(decision, "")).decider;
    for d in [
        "Decided by the ruling (Progenitor Mimic, 2013-04-15: \"two instances\"). Twice.",
        "Decided by the rulings (Arsenal Thresher, 2009-05-01): yes.",
        "Decided by the rule (118.12 and its Standstill example). None.",
        "Decided by the sources (Nov 20, work item ruling-conflicts-13). As it begins.",
        "Decided by the WotC Seasinger ruling (2007-09-16), applied generally.",
        "Decided by the rulings (the user, Nov 20, pointing to the first: \"If ...\"). LKI.",
    ] {
        assert_eq!(who(d), Decider::Sources, "{d}");
    }
    for d in [
        "No (option B, decided by the coordinator on Oct 14). Only the ability.",
        "The whole cost. Decided in the review of work item small-fixes-4 (Nov 20).",
    ] {
        assert_eq!(who(d), Decider::Coordinator, "{d}");
    }
    assert_eq!(
        decisions::attribute(&entry(
            "No (option B, decided by the coordinator on Oct 14). Only.",
            ""
        ))
        .clause,
        "decided by the coordinator on Oct 14"
    );
    // No decider named: "not decided by" and "would be decided by" don't count.
    assert_eq!(who("Keep."), Decider::Unstated);
    assert_eq!(
        decisions::attribute(&entry(
            "One event.",
            "Not decided by any source: no ruling names it. The copy would be decided by the copy."
        ))
        .decider,
        Decider::Unstated
    );
    // A decider in words that can't be read is never guessed.
    assert!(matches!(
        who("Decided by the user. Yes."),
        Decider::Unparsed(_)
    ));
    assert!(matches!(
        who("Decided by consensus. Yes."),
        Decider::Unparsed(_)
    ));
    // A clause about another entry isn't this entry's attribution.
    assert_eq!(
        who("One event, as `other-id` (decided by the user, Nov 9) has it."),
        Decider::Unstated
    );
}

#[test]
fn the_user_deciding_in_another_form_needs_a_source_line() {
    let log = |decision: &str, why: &str| {
        decisions::attribution_for_log(&entry(decision, why), &ids()).decider
    };
    for (decision, why) in [
        (
            "(to be fixed later; the user, Nov 12) Not in the engine.",
            "",
        ),
        (
            "Keep.",
            "The extension was accepted by the user (\"Sure\").",
        ),
        ("Keep.", "The user agreed on Nov 20."),
        ("Keep, as the user's decision of Nov 5 has it.", ""),
        ("Keep, which the user on Nov 20 called right.", ""),
    ] {
        assert!(
            matches!(log(decision, why), Decider::Unparsed(_)),
            "{decision} / {why}"
        );
    }
    // Not when the sentence is about another entry, or says the user didn't decide, or
    // only names the user.
    for (decision, why) in [
        (
            "Keep.",
            "It ends at the limit, which the user accepted on Oct 21 (`other-id`).",
        ),
        ("Keep.", "This part was not decided by the user."),
        ("Keep.", "The loops work the user postponed is later."),
        ("Keep.", "The user asked whether it holds; it does."),
    ] {
        assert_eq!(log(decision, why), Decider::Unstated, "{decision} / {why}");
    }
    // The words shown are the sentence.
    let a = decisions::attribution_for_log(
        &entry("Keep. The extension was accepted by the user. More.", ""),
        &ids(),
    );
    assert_eq!(a.clause, "The extension was accepted by the user.");
    assert_eq!(a.field, "Decision");
    // An entry whose attribution is read is filed by it.
    assert_eq!(
        log("Decided by the user, Nov 4. Yes, the user agreed.", ""),
        Decider::User(date(11, 4))
    );
}

#[test]
fn the_users_words_are_the_quotations_a_sentence_gives_them() {
    assert_eq!(
        decisions::user_quotes(
            "Decided by the user, Nov 19: \"when \"an opponent chooses\" the controller should pick.\" The controller picks (\"An opponent chooses one\")."
        ),
        vec!["when \"an opponent chooses\" the controller should pick.".to_string()]
    );
    assert_eq!(
        decisions::user_quotes(
            "Decided by the user, Nov 20. As recommended; the user: \"22. Agreed\". For every grant (\"look at\")."
        ),
        vec!["22. Agreed".to_string()]
    );
    assert_eq!(
        decisions::user_quotes("Decided by the user, Nov 19 (\"I agree.\"): as recommended."),
        vec!["I agree.".to_string()]
    );
    assert_eq!(
        decisions::user_quotes(
            "Decided by the user, Nov 19. Accepted tentatively (\"I'm really not sure\"; on the list)."
        ),
        vec!["I'm really not sure".to_string()]
    );
    assert_eq!(
        decisions::user_quotes(
            "Provisional (the user, Nov 19: \"I am really not sure here\"; research)."
        ),
        vec!["I am really not sure here".to_string()]
    );
    // "The user's decision of <date>:" introduces the user's words; a title in parentheses
    // after it doesn't.
    assert_eq!(
        decisions::user_quotes(
            "The user's decision of Nov 5: \"It looks like some rulings are templates.\""
        ),
        vec!["It looks like some rulings are templates.".to_string()]
    );
    assert!(decisions::user_quotes(
        "The user's decision of Nov 5 (evening) accepted the recommendation (\"For the user\", item 1)."
    )
    .is_empty());
    // Nor is a quotation after a parenthesis that named the user.
    assert!(decisions::user_quotes(
        "One event (`x`, decided by the user, Nov 19: one event), as the rulings say (\"at the same time\")."
    )
    .is_empty());
    assert_eq!(
        decisions::user_quotes(
            "Decided by the user, Nov 12, for now (accepting the recommendation): \"I agree\"."
        ),
        vec!["I agree".to_string()]
    );
    // Card text and rulings are not the user's words, nor is what "the user decided".
    assert!(decisions::user_quotes(
        "Decided by the user, Nov 4. No: \"If you do\" is false. The part the user decided is N (\"Create two\")."
    )
    .is_empty());
    assert!(decisions::user_quotes(
        "Decided by the ruling (Strionic Resonator, 2018-03-16: \"copies are linked\")."
    )
    .is_empty());
    // "The user's <word> (<date>):" introduces them too.
    assert_eq!(
        decisions::user_quotes(
            "Right in its own case. The user's reading of that ruling (Nov 19): \"I think it clarifies\". So it holds."
        ),
        vec!["I think it clarifies".to_string()]
    );
    // An acceptance introduces them only as a short lead ("Accepted tentatively ("), not as
    // a sentence that goes on to quote card text.
    assert!(decisions::user_quotes(
        "Accepted for every card with this template and its reminder (\"Flying\")."
    )
    .is_empty());
    // The words introducing them, from their sentence (or the Rules line's part).
    let q = decisions::user_quote_spans(
        "729.2 \"Randomly determine.\" / The user's decision of Nov 12 (entry `x`): \"always choose\"",
    );
    assert_eq!(q.len(), 1);
    assert_eq!(q[0].lead, "The user's decision of Nov 12 (entry `x`)");
    assert_eq!(q[0].text, "always choose");
    // Curly quotes pair as written.
    assert_eq!(
        decisions::quotations("a “b “c” d” e"),
        vec![(2, 19, "b “c” d".to_string())]
    );
}

#[test]
fn the_users_words_are_looked_for_in_the_other_fields_too() {
    let text = "## a: T\n- Kind: policy\n- Status: decided\n- Rules: 1.1 / The user's decision of Nov 5: \"Agreed.\"\n- Decision: Decided by the user, Nov 5. Yes.\n- Why: W.\n";
    let e = parse_file(Path::new("x/area.md"), text).remove(0);
    let (field, quotes) = decisions::entry_user_quotes(&e, "Decision");
    assert_eq!(field, "Rules");
    assert_eq!(
        quotes.iter().map(|q| q.text.as_str()).collect::<Vec<_>>(),
        vec!["Agreed."]
    );
    assert_eq!(quotes[0].lead, "The user's decision of Nov 5");
}

#[test]
fn low_confidence_marks_are_found() {
    let e = entry(
        "Decided by the user, Nov 12, for now (accepting it).",
        "The user: \"I'm not sure\".",
    );
    let marks = decisions::low_confidence(&e, &ids());
    let names: Vec<&str> = marks.iter().map(|(m, _)| *m).collect();
    assert_eq!(names, vec!["for now", "not sure"]);
    assert_eq!(marks[1].1, "The user: \"I'm not sure\".");
    // The words around a mark, cut at spaces.
    let long = entry(
        "Decided by the user, Nov 12. The first part of this decision is long enough to cut, and it holds for now, until the user looks at it again later on in the project. And more words follow here.",
        "",
    );
    assert_eq!(
        decisions::low_confidence(&long, &ids())[0].1,
        "...part of this decision is long enough to cut, and it holds for now, until the user looks at it again later on in the project...."
    );
    assert!(
        decisions::low_confidence(&entry("Decided by the user, Nov 4. Yes.", ""), &ids())
            .is_empty()
    );
    // A mark in a sentence about another entry is about that entry; the next occurrence,
    // about this one, counts.
    let e = entry(
        "Decided by the user, Nov 11. Yes. A neighboring call is separate, decided by the user on Nov 12 for now (`other-id`).",
        "",
    );
    assert!(decisions::low_confidence(&e, &ids()).is_empty());
    let e = entry(
        "Decided by the user, Nov 11. A neighboring call is separate, decided for now (`other-id`). This one holds for now too.",
        "",
    );
    // The context is around the second occurrence (the first's would start before "A
    // neighboring").
    assert!(decisions::low_confidence(&e, &ids())[0]
        .1
        .starts_with("...is separate, decided for now (`other-id`). This one holds for now"));
    // Unless the sentence says it's about this decision; and two marks in one sentence are
    // one.
    let e = entry(
        "Decided by the user, Nov 19. Yes.",
        "The user later (deciding `other-id`) called this decision one of the low confidence ones, a lower-confidence one.",
    );
    let marks = decisions::low_confidence(&e, &ids());
    assert_eq!(
        marks.iter().map(|(m, _)| *m).collect::<Vec<_>>(),
        vec!["low confidence"]
    );
    let e = entry(
        "Decided by the user, Nov 20: \"I'll take it, but not very sure.\"",
        "",
    );
    assert_eq!(decisions::low_confidence(&e, &ids())[0].0, "not very sure");
}

#[test]
fn sentences_respect_quotations_brackets_and_abbreviations() {
    let split = |t: &str| -> Vec<String> {
        decisions::sentence_spans(t)
            .into_iter()
            .map(|(s, e)| t[s..e].to_string())
            .collect()
    };
    assert_eq!(
        split("Yes. The ruling: \"It is. So.\" Then (see 608.2h. Not here) it ends. Last"),
        vec![
            "Yes.",
            "The ruling: \"It is. So.\"",
            "Then (see 608.2h. Not here) it ends.",
            "Last"
        ]
    );
    assert_eq!(
        split("Decided by the user, Nov. 20. Many, e.g. Shock, or 2. 3 is next... Fine."),
        vec![
            "Decided by the user, Nov. 20.",
            "Many, e.g. Shock, or 2. 3 is next... Fine."
        ]
    );
    // A sentence may start with a rule number or a backtick.
    assert_eq!(
        split("It holds. 613.1 applies. `x` too."),
        vec!["It holds.", "613.1 applies.", "`x` too."]
    );
}

#[test]
fn excerpts_end_at_a_sentence_and_mark_the_cut() {
    let one = "First sentence here.";
    assert_eq!(decisions::excerpt(one, 10, 100), "First sentence here.");
    let text = "Short. Then a longer second sentence follows. And a third one.";
    // Sentences are added while the excerpt is shorter than the minimum.
    assert_eq!(decisions::excerpt(text, 5, 100), "Short. …");
    assert_eq!(
        decisions::excerpt(text, 10, 100),
        "Short. Then a longer second sentence follows. …"
    );
    // A sentence that would pass the maximum isn't added, unless the excerpt is still
    // shorter than half the minimum: then it's cut at a space.
    assert_eq!(decisions::excerpt(text, 10, 30), "Short. …");
    assert_eq!(
        decisions::excerpt(text, 100, 30),
        "Short. Then a longer second …"
    );
    // A first sentence longer than the maximum is cut at a space outside quotations.
    assert_eq!(
        decisions::excerpt("One two (\"three four five\") six seven.", 1, 30),
        "One two (\"three four five\") …"
    );
}

#[test]
fn single_case_affects_lines() {
    assert!(decisions::affects_a_single_case("Cyclopean Tomb."));
    assert!(!decisions::affects_a_single_case(
        "Wastes, Snow-Covered Wastes in `check_limited`."
    ));
    assert!(!decisions::affects_a_single_case(
        "every card with \"an opponent chooses\""
    ));
    assert!(!decisions::affects_a_single_case(
        "A Lich's Mirror and any other loop"
    ));
    assert!(!decisions::affects_a_single_case(""));
}

#[test]
fn the_revisit_later_section_is_extracted() {
    let plan = "# Plan\n\n### The user's answers\n\n- not this\n\n### Revisit later (user, Nov 12)\n\nReadings to look at again:\n- **A.** The user: \"x\".\n  - Entry `a-id`: y.\n- **B.** `b-id`\n  continued.\n\n### Already decided\n\n- not this either\n";
    let r = decisions::revisit_later(plan).unwrap();
    assert_eq!(r.heading, "Revisit later (user, Nov 12)");
    assert_eq!(r.intro, vec!["Readings to look at again:".to_string()]);
    assert_eq!(
        r.bullets,
        vec![
            "- **A.** The user: \"x\".\n  - Entry `a-id`: y.".to_string(),
            "- **B.** `b-id`\n  continued.".to_string(),
        ]
    );
    assert_eq!(decisions::revisit_later("# Plan\n- x\n"), None);
}

#[test]
fn anchors_follow_githubs_rule() {
    assert_eq!(
        decisions::anchor("default-yes-no: A default yes/no answer does what \"you may\" offers"),
        "default-yes-no-a-default-yesno-answer-does-what-you-may-offers"
    );
    assert_eq!(
        decisions::anchor("Revisit later (user, Nov 12)"),
        "revisit-later-user-nov-12"
    );
}

#[test]
fn render_sorts_entries_into_sections() {
    let text = "## u-old: Old\n- Kind: interpretation\n- Status: decided\n- Decision: Decided by the user, Oct 21. Old.\n- Alternatives: none\n\n\
                ## u-new: New\n- Kind: policy\n- Status: decided\n- Decision: Decided by the user, Nov 19: \"my words\". New.\n- Alternatives: other\n- Affects: every card\n\n\
                ## s: Source\n- Kind: interpretation\n- Status: decided\n- Decision: Decided by the ruling (X, 2020-01-01). S.\n- Why: W.\n\n\
                ## w: Worker\n- Kind: policy\n- Status: decided\n- Decision: Keep.\n- Why: Least commitment. More.\n\n\
                ## p: Prov\n- Kind: interpretation\n- Status: provisional\n- Question: Q?\n- Decision: P.\n\n\
                ## n: Needs\n- Kind: interpretation\n- Status: decided\n- Decision: Decided by the user. N.\n";
    let entries = parse_file(Path::new("x/area.md"), text);
    let plan = "### Revisit later\n- `w` again\n";
    let log = "## Outdated rulings\n\n### Card A: \"old\"\n- Kind: outdated\n- Status: open for review\n- Ruling, Card A (2010-01-01): \"Old words.\"\n";
    let out = decisions::render_from(&entries, plan, log);
    let pos = |s: &str| out.find(s).unwrap_or_else(|| panic!("{s:?} not in\n{out}"));
    // Newest first, with the user's words, and the reading after them.
    assert!(pos("### Nov 19") < pos("### Oct 21"));
    assert!(pos("**`u-new`**") < pos("**`u-old`**"));
    assert!(out.contains("  - The user: \"my words\"\n"));
    assert!(out.contains("  - Reading chosen: New.\n"));
    assert!(out.contains("  - Reading chosen: Old.\n"));
    assert!(out.contains("  - Alternatives not chosen: other\n"));
    assert!(pos("## Decided by the sources") < pos("**`s`**"));
    assert!(out.contains("Basis: Decided by the ruling (X, 2020-01-01).\n"));
    assert!(pos("### Naming no decider") < pos("**`w`**"));
    assert!(out.contains("Basis: Least commitment. … On the \"Revisit later\" list.\n"));
    assert!(pos("## Provisional") < pos("**`p`**"));
    assert!(pos("## Revisit later") < pos("- `w` again"));
    assert!(out.contains("| outdated | 1 | 0 | 0 | 0 | 1 |"));
    assert!(out.contains("| [Card A: \"old\"](RULING_DISCREPANCIES.md#card-a-old) | open for review | Card A | 2010-01-01 |"));
    assert!(pos("## Policies the user set") < pos("- **`u-new`** (Nov 19"));
    assert!(pos("## Needs a source line") < pos("**`n`**"));
    let counts = decisions::section_counts(&entries, plan, log);
    assert_eq!(
        counts,
        vec![
            ("Decided by the user", 2),
            ("Decided by the sources", 1),
            ("Decided by the coordinator or a worker", 1),
            ("Provisional", 1),
            ("Revisit later", 1),
            ("Ruling discrepancies", 1),
            ("Policies the user set", 1),
            ("Needs a source line", 1),
        ]
    );
}

#[test]
fn decisions_log_is_current() {
    let root = mtg_data::interpretations::repo_root();
    let log = decisions::render(&root);
    let current = std::fs::read_to_string(root.join(decisions::OUTPUT))
        .unwrap_or_default()
        .replace("\r\n", "\n");
    assert!(
        current == log,
        "docs/DECISIONS.md is out of date: run \
         `cargo run --release -p mtg-tools -- interpretations --write`"
    );
}
