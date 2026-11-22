//! A permanent's replacement effect can make another permanent leave the battlefield as it
//! enters (Rescuer Sphinx: "As this creature enters, you may return a nonland permanent you
//! control to its owner's hand. If you do, this creature enters with a +1/+1 counter on
//! it."). The effects that permanent generated for the entering one still apply to it, as
//! Rhythm of the Wild's ruling says of riot ("If Rhythm of the Wild leaves the battlefield
//! at the same time that a nontoken creature enters the battlefield (most likely because
//! that creature has a replacement effect, such as that of Rescuer Sphinx), that creature
//! still gets a +1/+1 counter or haste."), and as the engine does for Grumgully, the
//! Generous's counter (CR 614.12, 616.1f; `EntrySnapshot`).

use crate::r614_audit_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 controls Rhythm of the Wild ("Nontoken creatures you control have riot.") and
/// Grumgully, the Generous ("Each other non-Human creature you control enters with an
/// additional +1/+1 counter on it."), and Rescuer Sphinx enters. P0 answers the first
/// choice among replacement effects with `first` (the default if `None`), returns Rhythm of
/// the Wild with the Sphinx's ability and takes riot's +1/+1 counter.
fn sphinx_returning_rhythm(first: Option<usize>) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let rhythm = t.battlefield(P0, "Rhythm of the Wild");
    t.battlefield(P0, "Grumgully, the Generous");
    if let Some(i) = first {
        t.answer(P0, DecisionKind::Replacement, Answer::Index(i));
    }
    // The Sphinx's "you may return" and riot's "+1/+1 counter?", in whichever order.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(rhythm)]);
    let sphinx = t.enter(P0, "Rescuer Sphinx");
    (t, sphinx)
}

#[test]
fn riot_from_a_rhythm_of_the_wild_returned_as_the_sphinx_enters_is_still_one_of_the_choices() {
    cr!("614.12", "616.1", "616.1f", "702.136a");
    interp!("questions-only-for-a-choice");
    ruling!(
        "Rhythm of the Wild",
        "If Rhythm of the Wild leaves the battlefield at the same time that a nontoken creature enters the battlefield (most likely because that creature has a replacement effect, such as that of Rescuer Sphinx), that creature still gets a +1/+1 counter or haste."
    );
    supported("Rhythm of the Wild");
    supported("Grumgully, the Generous");
    supported("Rescuer Sphinx");
    let (probe, _) = sphinx_returning_rhythm(None);
    let first = option_index(&replacement_choices(&probe)[0].1, "you may return");
    // P0 applies the Sphinx's effect first, returning Rhythm of the Wild. Two effects are
    // left: Grumgully's, and riot, which the Sphinx had as the event was proposed. P0
    // chooses between them (CR 616.1f), and both apply. (Riot isn't held back until
    // Grumgully's effect has been applied.)
    let (t, sphinx) = sphinx_returning_rhythm(Some(first));
    assert!(t.in_hand(P0, "Rhythm of the Wild"));
    let choices = replacement_choices(&t);
    assert_eq!(choices.len(), 2, "{choices:?}");
    assert_eq!(choices[0].1.len(), 3, "{choices:?}");
    let second = &choices[1].1;
    assert_eq!(second.len(), 2, "{second:?}");
    assert!(
        second.iter().any(|o| o.starts_with("Grumgully")),
        "{second:?}"
    );
    assert!(second.iter().any(|o| o.contains("Riot")), "{second:?}");
    // The Sphinx's own counter, Grumgully's, and riot's.
    let n = plus_counters(&t, sphinx);
    assert!(
        n == 3 || (n == 2 && t.obj_now(sphinx).has_keyword(KeywordKind::Haste)),
        "{n} counters"
    );
}

/// P0 controls Grumgully, the Generous; `name` enters (Clone entering as a copy of a Rescuer
/// Sphinx P0 controls, if `name` is "Clone"). P0 answers the first choice among replacement
/// effects with `first` (the default if `None`) and returns Grumgully with the Sphinx's
/// ability. Returns the game and the entering permanent.
fn entering_returning_grumgully(name: &str, first: Option<usize>) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let grumgully = t.battlefield(P0, "Grumgully, the Generous");
    if name == "Clone" {
        // "You may have this creature enter as a copy of any creature on the battlefield."
        let sphinx = t.battlefield(P0, "Rescuer Sphinx");
        t.answer_choose(P0, &[obj(sphinx)]);
    }
    if let Some(i) = first {
        t.answer(P0, DecisionKind::Replacement, Answer::Index(i));
    }
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(grumgully)]);
    let id = t.enter(P0, name);
    (t, id)
}

/// The first choice among replacement effects when `name` enters returning Grumgully: the
/// index of the Sphinx's ability.
fn sphinx_ability_first(name: &str) -> usize {
    let (probe, _) = entering_returning_grumgully(name, None);
    option_index(&replacement_choices(&probe)[0].1, "you may return")
}

#[test]
fn rescuer_sphinx_returning_grumgully_still_gets_its_counter() {
    cr!("614.12", "616.1f");
    supported("Grumgully, the Generous");
    supported("Rescuer Sphinx");
    // Rescuer Sphinx returns Grumgully as it enters; Grumgully's effect, which applied to it
    // as the event was proposed, still gives it a counter (as Rhythm of the Wild's ruling
    // says of riot).
    let first = sphinx_ability_first("Rescuer Sphinx");
    let (t, sphinx) = entering_returning_grumgully("Rescuer Sphinx", Some(first));
    assert!(t.in_hand(P0, "Grumgully, the Generous"));
    assert_eq!(plus_counters(&t, sphinx), 2);
}

/// Regression test: once Clone's copy effect had been applied (CR 616.1c), `Game::replace_rec`
/// kept the snapshot of the effects determined before it (`EntrySnapshot`, whose entering
/// mode no longer matched) and never took another, so no effect was kept for the permanent
/// the copy went on to enter as: a Clone entering as a copy of Rescuer Sphinx that returns
/// Grumgully got only the Sphinx's own counter, where Rescuer Sphinx itself gets Grumgully's
/// too (the test above). The Clone has the Sphinx's replacement effect (Clone's ruling: "Any
/// "as [this creature] enters" or "[this creature] enters with" abilities of the chosen
/// creature will also work."), and Grumgully's effect applied to it once it was entering as
/// the Sphinx, just before Grumgully left.
#[test]
fn a_clone_entering_as_rescuer_sphinx_returning_grumgully_still_gets_its_counter() {
    cr!("614.12", "616.1c", "616.1f");
    interp!("entry-effects-of-a-leaving-source");
    ruling!(
        "Clone",
        "Any \"as [this creature] enters\" or \"[this creature] enters with\" abilities of the chosen creature will also work"
    );
    supported("Grumgully, the Generous");
    supported("Rescuer Sphinx");
    supported("Clone");
    let first = sphinx_ability_first("Clone");
    let (t, clone) = entering_returning_grumgully("Clone", Some(first));
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Rescuer Sphinx");
    assert!(t.in_hand(P0, "Grumgully, the Generous"));
    assert_eq!(plus_counters(&t, clone), 2);
}

/// P0 controls Metallic Mimic naming Mutant ("Each other creature you control of the chosen
/// type enters with an additional +1/+1 counter on it.") and Master Biomancer (2/4: "Each
/// other creature you control enters with a number of additional +1/+1 counters on it equal
/// to this creature's power and as a Mutant in addition to its other types."), and Rescuer
/// Sphinx enters. P0 answers the choices among replacement effects with `picks` (the default
/// for any left) and returns Metallic Mimic with the Sphinx's ability if asked. Returns the
/// game, the Sphinx, and the choices among replacement effects asked as the Sphinx entered.
fn sphinx_under_biomancer_and_mimic(picks: &[usize]) -> (TestGame, ObjectId, Vec<Vec<String>>) {
    supported("Master Biomancer");
    supported("Metallic Mimic");
    supported("Rescuer Sphinx");
    let mut t = TestGame::new(2);
    let mutant = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|x| x == "Mutant")
        .expect("Mutant is a creature type");
    t.answer(P0, DecisionKind::Option, Answer::Index(mutant));
    let mimic = t.enter(P0, "Metallic Mimic");
    t.settle();
    assert!(t.obj_now(mimic).chars.has_subtype("Mutant"));
    t.battlefield(P0, "Master Biomancer");
    for i in picks {
        t.answer(P0, DecisionKind::Replacement, Answer::Index(*i));
    }
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(mimic)]);
    let before = replacement_choices(&t).len();
    let sphinx = t.enter(P0, "Rescuer Sphinx");
    let asked = replacement_choices(&t)[before..]
        .iter()
        .map(|(p, o)| {
            assert_eq!(*p, P0);
            o.clone()
        })
        .collect();
    (t, sphinx, asked)
}

/// An effect that became applicable through another one (CR 616.2) and whose source then
/// leaves: P0 applies Master Biomancer's effect first, so the Sphinx would enter as a Mutant
/// and Metallic Mimic's effect applies to it too; then the Sphinx's, returning Metallic
/// Mimic. Mimic's effect applied to the event just before Mimic left, so it still applies:
/// Biomancer's 2 counters, the Sphinx's 1 and Mimic's 1. (Keeping only the effects
/// determined as the event was proposed, before Biomancer's made it a Mutant, would leave 3.)
#[test]
fn metallic_mimics_effect_for_a_sphinx_made_a_mutant_still_applies_once_the_sphinx_returns_it(
) {
    cr!("614.12", "616.1f", "616.2");
    interp!("entry-effects-of-a-leaving-source");
    // (Rhythm of the Wild's ruling, cited by the riot test above, is the principle applied
    // here to Metallic Mimic: an effect whose source leaves as the creature enters still
    // applies.)
    ruling!(
        "Rescuer Sphinx",
        "No player may take actions between the time you choose which permanent to return and the time Rescuer Sphinx is on the battlefield with a +1/+1 counter."
    );
    // The first choice: Master Biomancer's effect or the Sphinx's (Mimic's doesn't apply
    // yet: the Sphinx isn't a Mutant).
    let (_, _, asked) = sphinx_under_biomancer_and_mimic(&[]);
    let first = &asked[0];
    assert_eq!(first.len(), 2, "{first:?}");
    let biomancer = option_index(first, "Master Biomancer");
    // The second: Mimic's, now applicable (CR 616.2), or the Sphinx's.
    let (_, _, asked) = sphinx_under_biomancer_and_mimic(&[biomancer]);
    let second = &asked[1];
    assert_eq!(second.len(), 2, "{second:?}");
    assert!(
        second.iter().any(|o| o.starts_with("Metallic Mimic")),
        "{second:?}"
    );
    let sphinx_first = option_index(second, "you may return");
    let (t, sphinx, asked) = sphinx_under_biomancer_and_mimic(&[biomancer, sphinx_first]);
    // Mimic's effect is then the only one left: nothing more to choose.
    assert_eq!(asked.len(), 2, "{asked:?}");
    assert!(t.in_hand(P0, "Metallic Mimic"));
    assert!(t.obj_now(t.g.current(sphinx)).chars.has_subtype("Mutant"));
    assert_eq!(plus_counters(&t, sphinx), 4);
}
