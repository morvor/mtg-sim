//! CR 614.12: which replacement effects modify how a permanent enters is determined from
//! the permanent as it would exist on the battlefield, "taking into account replacement
//! effects that have already modified how it enters the battlefield (see rule 616.1),
//! continuous effects from the permanent's own static abilities that would apply to it once
//! it's on the battlefield, and continuous effects that already exist and would apply to
//! the permanent" — so an effect can become applicable through another (616.2). What a later
//! effect sees is the permanent it will be (`Game::with_hypothetical_entry`):
//!
//! - with the counters it enters with, from an "enters with" effect or from tribute, on
//!   which an existing effect can depend (Sigarda's Summons: "Creatures you control with
//!   +1/+1 counters on them have base power and toughness 4/4, have flying, and are Angels
//!   in addition to their other types.");
//! - with what the ability putting it onto the battlefield says it is (611.2e) applied after
//!   the effects of its own static abilities (613.7n), and before what another replacement
//!   effect already applied has it enter as ("as a Mutant in addition to its other types",
//!   Master Biomancer), as the permanent is once it's there.
//!
//! Metallic Mimic ("Each other creature you control of the chosen type enters with an
//! additional +1/+1 counter on it.") shows which creature type it would have.

use crate::r614_audit_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 controls Metallic Mimic naming `ty`.
fn mimic_naming(t: &mut TestGame, ty: &str) -> ObjectId {
    supported("Metallic Mimic");
    let i = mtg_engine::types::subtype_lists()
        .creature
        .iter()
        .position(|x| x == ty)
        .unwrap_or_else(|| panic!("no creature type {ty}"));
    t.answer(P0, DecisionKind::Option, Answer::Index(i));
    let mimic = t.enter(P0, "Metallic Mimic");
    t.settle();
    assert!(t.obj_now(mimic).chars.has_subtype(ty));
    mimic
}

/// P0 controls Sigarda's Summons and Metallic Mimic naming Angel.
fn summons_and_mimic_naming_angel(t: &mut TestGame) {
    supported("Sigarda's Summons");
    t.battlefield(P0, "Sigarda's Summons");
    mimic_naming(t, "Angel");
}

#[test]
fn a_creature_given_a_counter_by_grumgully_enters_as_an_angel_under_sigardas_summons() {
    cr!("614.12", "616.2", "614.1c", "122.6");
    supported("Grumgully, the Generous");
    let mut t = TestGame::new(2);
    summons_and_mimic_naming_angel(&mut t);
    t.battlefield(P0, "Grumgully, the Generous");
    // Grizzly Bears is no Angel as it's cast. Grumgully's effect ("Each other non-Human
    // creature you control enters with an additional +1/+1 counter on it.") applies, and with
    // that counter it would be an Angel on the battlefield: Metallic Mimic's effect applies
    // too.
    let bears = cast_resolve(&mut t, P0, "Grizzly Bears", &[]);
    let bears = t.g.current(bears);
    assert!(t.obj_now(bears).chars.has_subtype("Angel"));
    assert_eq!(plus_counters(&t, bears), 2);
    assert_eq!(t.pt(bears), (6, 6));
}

#[test]
fn tribute_counters_make_ornitharch_enter_as_an_angel_under_sigardas_summons() {
    cr!("614.12", "616.2", "702.104a");
    ruling!(
        "Ornitharch",
        "If the opponent pays tribute, the creature will enter the battlefield with the specified number of +1/+1 counters on it. It won’t enter the battlefield and then have the +1/+1 counters placed on it."
    );
    supported("Ornitharch");
    let mut t = TestGame::new(2);
    summons_and_mimic_naming_angel(&mut t);
    // Ornitharch (tribute 2) is an Archon. P1 pays tribute: it enters with the two +1/+1
    // counters P1 puts on it, so it would be an Angel on the battlefield, and Metallic
    // Mimic's effect applies.
    t.answer_yes(P1, true);
    let orni = cast_resolve(&mut t, P0, "Ornitharch", &[]);
    let orni = t.g.current(orni);
    assert!(t.obj_now(orni).chars.has_subtype("Angel"));
    assert_eq!(plus_counters(&t, orni), 3);
    assert_eq!(t.pt(orni), (7, 7));
    // Tribute was paid: its ability made no Birds.
    let tokens =
        t.g.permanents()
            .filter(|o| o.kind == mtg_engine::object::ObjKind::Token)
            .count();
    assert_eq!(tokens, 0);
}

#[test]
fn mephidross_vampire_put_onto_the_battlefield_by_the_master_is_a_mutant_and_no_vampire() {
    cr!("613.7n", "611.2e", "614.12", "205.1a");
    supported("The Master, Transcendent");
    supported("Mephidross Vampire");
    let mut t = TestGame::new(2);
    mimic_naming(&mut t, "Vampire");
    let master = t.battlefield(P0, "The Master, Transcendent");
    let vampire = t.library_top(P1, "Mephidross Vampire");
    t.g.mill(P1, 1);
    t.settle();
    let vampire = t.g.current(vampire);
    // "{T}: Put target creature card in a graveyard that was milled this turn onto the
    // battlefield under your control. It's a green Mutant with base power and toughness 3/3.
    // (It loses its other colors and creature types.)" The effect of Mephidross Vampire's
    // own ability ("Each creature you control is a Vampire in addition to its other creature
    // types ...") has the earlier timestamp (613.7n): The Master's effect, applied after it,
    // leaves Mephidross Vampire a Mutant and no Vampire, as Metallic Mimic sees it as it
    // enters.
    t.activate(P0, master, 0, &[obj(vampire)])
        .expect("The Master's ability");
    t.resolve_all();
    let vampire = t.g.current(vampire);
    assert!(t.on_battlefield(vampire));
    assert!(t.obj_now(vampire).chars.has_subtype("Mutant"));
    assert!(!t.obj_now(vampire).chars.has_subtype("Vampire"));
    assert_eq!(plus_counters(&t, vampire), 0);
    assert_eq!(t.pt(vampire), (3, 3));
    // Its ability makes P0's other creatures Vampires.
    assert!(t.obj_now(master).chars.has_subtype("Vampire"));
}

/// P0 controls Metallic Mimic naming Mutant, Master Biomancer (2/4: "Each other creature you
/// control enters with a number of additional +1/+1 counters on it equal to this creature's
/// power and as a Mutant in addition to its other types."), `rhythms` Rhythms of the Wild
/// ("Nontoken creatures you control have riot.") and Otherworldly Escort, which is destroyed
/// and returns: "When this creature dies, if it's not a Spirit, return it to the battlefield
/// under its owner's control with four charge counters on it. It's a Spirit Detective." P0
/// answers the choices among replacement effects with `picks` and takes haste for riot.
fn escort_returns(rhythms: usize, picks: &[usize]) -> (TestGame, ObjectId) {
    supported("Master Biomancer");
    supported("Otherworldly Escort");
    supported("Rhythm of the Wild");
    let mut t = TestGame::new(2);
    mimic_naming(&mut t, "Mutant");
    t.battlefield(P0, "Master Biomancer");
    for _ in 0..rhythms {
        t.battlefield(P0, "Rhythm of the Wild");
    }
    let escort = t.battlefield(P0, "Otherworldly Escort");
    for i in picks {
        t.answer(P0, DecisionKind::Replacement, Answer::Index(*i));
    }
    for _ in 0..rhythms {
        t.answer_yes(P0, false);
    }
    t.g.destroy(escort, None);
    t.resolve_all();
    (t, escort)
}

/// The returned Escort is a Spirit Detective Mutant: Master Biomancer's two counters, then
/// (it's entering as a Mutant) Metallic Mimic's one.
fn assert_spirit_detective_mutant(t: &TestGame, escort: ObjectId, what: &str) {
    let escort = t.g.current(escort);
    assert!(t.on_battlefield(escort), "{what}");
    let c = &t.obj_now(escort).chars;
    assert!(
        c.has_subtype("Spirit") && c.has_subtype("Detective") && c.has_subtype("Mutant"),
        "{what}"
    );
    assert!(!c.has_subtype("Human"), "{what}");
    assert_eq!(t.counters(escort, "charge"), 4, "{what}");
    assert_eq!(plus_counters(t, escort), 3, "{what}");
}

#[test]
fn otherworldly_escort_returning_under_master_biomancer_enters_as_a_spirit_detective_mutant() {
    cr!("614.12", "616.2", "611.2e", "614.1c", "702.136a", "702.136b");
    ruling!(
        "Rhythm of the Wild",
        "If a creature enters the battlefield with two instances of riot, you may choose to have it get two +1/+1 counters, one +1/+1 counter and haste, or two instances of haste."
    );
    // Master Biomancer's effect alone, first.
    let (t, escort) = escort_returns(0, &[]);
    assert_spirit_detective_mutant(&t, escort, "Biomancer alone");
    // With two instances of riot (two Rhythms of the Wild), P0 applies riot, taking haste,
    // whenever it's offered, so Master Biomancer's effect comes after both: its "as a
    // Mutant" is still what the Escort enters as after "It's a Spirit Detective". (The
    // answers are found by replaying the return with those found so far.)
    let mut picks: Vec<usize> = Vec::new();
    for _ in 0..4 {
        let (probe, _) = escort_returns(2, &picks);
        let choices = replacement_choices(&probe);
        match choices.get(picks.len()) {
            Some((_, options)) if options.iter().any(|o| o.contains("Riot")) => {
                picks.push(option_index(options, "Riot"));
            }
            _ => break,
        }
    }
    let (t, escort) = escort_returns(2, &picks);
    let choices = replacement_choices(&t);
    assert!(
        !choices.is_empty(),
        "riot and Master Biomancer's effect were offered"
    );
    assert!(
        choices[0]
            .1
            .iter()
            .any(|o| o.starts_with("Master Biomancer")),
        "{choices:?}"
    );
    let riots = t
        .asked()
        .iter()
        .filter(|(p, d)| {
            *p == P0 && matches!(d, Decision::YesNo { prompt, .. } if prompt.starts_with("Riot"))
        })
        .count();
    assert_eq!(riots, 2);
    assert!(t
        .obj_now(t.g.current(escort))
        .has_keyword(KeywordKind::Haste));
    assert_spirit_detective_mutant(&t, escort, "after riot twice");
}
