//! The worked examples of CR 115 (targets), played out as tests.

use super::r105_util::{card_from_text, put_in_hand};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// A sorcery with a targeted cycling trigger (the CR 115.1a example's abstract card).
fn cycling_sorcery() -> CardDef {
    card_from_text(
        "Withering Study",
        "{1}{B}",
        "Sorcery",
        None,
        "Draw a card.\nCycling {2}\nWhen you cycle this card, target creature gets -1/-1 until end of turn.",
    )
}

#[test]
fn a_targeted_triggered_ability_doesnt_make_its_card_targeted() {
    cr!("115.1a", "115.1b");
    cr_example!("115.1a", "When you cycle this card, target creature gets -1/-1");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let card = put_in_hand(&mut t, P0, cycling_sorcery());
    t.library_top(P0, "Island");
    t.lands(P0, "Swamp", 2);
    // Cycling it: the triggered ability targets the creature.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.activate(P0, card, 0, &[]).unwrap();
    t.settle();
    let trigger = *t.g.stack.last().unwrap();
    let targets: Vec<Entity> = t
        .obj_now(trigger)
        .stack
        .as_ref()
        .unwrap()
        .chosen
        .iter()
        .flat_map(|c| c.targets.iter().flatten().cloned())
        .collect();
    assert_eq!(targets, vec![Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (1, 1));
    // The card itself isn't a targeted spell: cast, it has no targets, so Misdirection
    // ("Change the target of target spell with a single target.") can't target it.
    let other = put_in_hand(&mut t, P0, cycling_sorcery());
    t.library_top(P0, "Island");
    t.lands(P0, "Swamp", 2);
    let spell = t.cast(P0, other).go();
    t.lands(P1, "Island", 4);
    let mis = t.hand(P1, "Misdirection");
    t.g.turn.priority = Some(P1);
    assert!(t.cast(P1, mis).target(spell).try_go().is_err());
}

#[test]
fn arc_trails_two_targets_can_be_swapped_by_redirect() {
    cr!("115.7", "115.7e");
    cr_example!("115.7e", "You can change the first target to Llanowar Elves");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bear = t.battlefield(P1, "Runeclaw Bear");
    let elves = t.battlefield(P1, "Llanowar Elves");
    // Arc Trail: "Arc Trail deals 2 damage to any target and 1 damage to any other
    // target." Its targets are Runeclaw Bear and Llanowar Elves, in that order.
    t.lands(P0, "Mountain", 2);
    let arc = t.hand(P0, "Arc Trail");
    let s = t
        .cast(P0, arc)
        .targets(&[Entity::Object(bear), Entity::Object(elves)])
        .go();
    // Redirect: "You may choose new targets for target spell."
    t.lands(P1, "Island", 2);
    let redirect = t.hand(P1, "Redirect");
    t.g.turn.priority = Some(P1);
    t.cast(P1, redirect).target(s).go();
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Object(elves)]);
    t.answer_targets(P1, &[Entity::Object(bear)]);
    t.resolve();
    t.resolve();
    // 2 damage to the Elves, 1 to the Bear.
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert!(t.on_battlefield(bear));
    assert_eq!(t.obj_now(bear).damage, 1);
}
