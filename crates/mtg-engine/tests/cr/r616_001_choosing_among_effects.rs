//! The affected player's choice among replacement effects (CR 616.1): an answer naming no
//! option applies the first one listed (`default-first-listed-choice`), and the parts one
//! printed ability was compiled into are one effect: once one part is applied, the others
//! follow before the player chooses among the remaining effects (CR 616.1e–f).

use crate::r614_audit_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn an_answer_one_past_the_last_replacement_option_applies_the_first_listed() {
    cr!("616.1");
    interp!("default-first-listed-choice");
    supported("Darksteel Colossus");
    supported("Rest in Peace");
    // Darksteel Colossus ("If Darksteel Colossus would be put into a graveyard from
    // anywhere, reveal Darksteel Colossus and shuffle it into its owner's library
    // instead.") is sacrificed while Rest in Peace ("If a card or token would be put into a
    // graveyard from anywhere, exile it instead.") is on the battlefield. P0 answers the
    // choice between the two effects with an index one past the last option: an invalid
    // answer, so the first option listed is applied.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rest in Peace");
    let colossus = t.battlefield(P0, "Darksteel Colossus");
    t.answer(P0, DecisionKind::Replacement, Answer::Index(2));
    t.g.sacrifice(colossus, P0);
    let choices = replacement_choices(&t);
    assert_eq!(choices.len(), 1, "{choices:?}");
    let options = &choices[0].1;
    assert_eq!(options.len(), 2, "{options:?}");
    let expected = if options[0].starts_with("Rest in Peace") {
        Zone::Exile
    } else {
        Zone::Library(P0)
    };
    assert_eq!(t.zone(colossus), expected, "{options:?}");
}

/// P0 controls Master Biomancer ("Each other creature you control enters with a number of
/// additional +1/+1 counters on it equal to this creature's power and as a Mutant in
/// addition to its other types.", compiled in two parts) and Grumgully, the Generous ("Each
/// other non-Human creature you control enters with an additional +1/+1 counter on it.")
/// and casts Spike Feeder ("This creature enters with two +1/+1 counters on it."),
/// answering the first choice among the effects with `first` (the default if `None`).
/// Returns the game and the Feeder.
fn feeder_under_biomancer_and_grumgully(first: Option<usize>) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Master Biomancer");
    t.battlefield(P0, "Grumgully, the Generous");
    if let Some(i) = first {
        t.answer(P0, DecisionKind::Replacement, Answer::Index(i));
    }
    let feeder = cast_resolve(&mut t, P0, "Spike Feeder", &[]);
    let feeder = t.g.current(feeder);
    (t, feeder)
}

#[test]
fn master_biomancers_second_part_follows_before_the_next_choice() {
    cr!("616.1", "616.1e", "616.1f");
    supported("Master Biomancer");
    supported("Grumgully, the Generous");
    supported("Spike Feeder");
    let (probe, _) = feeder_under_biomancer_and_grumgully(None);
    let first = option_index(&replacement_choices(&probe)[0].1, "Master Biomancer");
    // P0 applies Master Biomancer's effect first: its counters, then (without a question)
    // its "as a Mutant"; then P0 chooses between the two effects left, Grumgully's and the
    // Feeder's own. Master Biomancer's isn't offered again.
    let (t, feeder) = feeder_under_biomancer_and_grumgully(Some(first));
    let choices = replacement_choices(&t);
    assert_eq!(choices.len(), 2, "{choices:?}");
    assert_eq!(choices[0].1.len(), 3, "{choices:?}");
    let second = &choices[1].1;
    assert_eq!(second.len(), 2, "{second:?}");
    assert!(
        second.iter().any(|o| o.starts_with("Grumgully")),
        "{second:?}"
    );
    assert!(
        second.iter().any(|o| o.starts_with("Spike Feeder")),
        "{second:?}"
    );
    assert!(
        !second.iter().any(|o| o.starts_with("Master Biomancer")),
        "{second:?}"
    );
    // Its own two, Master Biomancer's two (its power) and Grumgully's one.
    assert_eq!(plus_counters(&t, feeder), 2 + 2 + 1);
    assert!(t.obj_now(feeder).chars.has_subtype("Mutant"));
}
