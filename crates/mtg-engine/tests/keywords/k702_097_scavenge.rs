//! CR 702.97 Scavenge.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_027_037::activate_named;
use crate::common_k702_052_066::destroy;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn scavenge(
    t: &mut TestGame,
    p: PlayerId,
    card: ObjectId,
    target: ObjectId,
) -> Result<(), casting::Illegal> {
    t.answer_targets(p, &[Entity::Object(target)]);
    let r = activate_named(t, p, card, "Scavenge", 0).map(|_| ());
    if r.is_err() {
        t.clear_answers();
    }
    r
}

#[test]
fn scavenge_exiles_the_card_to_put_counters_equal_to_its_power() {
    cr!("702.97", "702.97a");
    ruling!(
        "Slitherhead",
        "Exiling the creature card with scavenge is part of the cost of activating the scavenge ability."
    );
    assert_supported("Dreg Mangler");
    let mut t = TestGame::new(2);
    let mangler = t.graveyard(P0, "Dreg Mangler");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Forest", 2);
    scavenge(&mut t, P0, mangler, bears).expect("scavenge");
    // The card is exiled as a cost, before the ability resolves.
    assert!(t.in_exile("Dreg Mangler"));
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.counters(bears, counters::PLUS1), 3);
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn scavenge_functions_only_from_the_graveyard() {
    cr!("702.97a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    let in_hand = t.hand(P0, "Slitherhead");
    assert!(scavenge(&mut t, P0, in_hand, bears).is_err());
    let on_bf = t.battlefield(P0, "Slitherhead");
    assert!(scavenge(&mut t, P0, on_bf, bears).is_err());
    let in_gy = t.graveyard(P0, "Slitherhead");
    assert!(scavenge(&mut t, P0, in_gy, bears).is_ok());
    t.resolve();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
}

#[test]
fn scavenge_can_be_activated_only_as_a_sorcery() {
    cr!("702.97a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = t.graveyard(P0, "Slitherhead");
    let b = t.graveyard(P0, "Slitherhead");
    // Not while something is on the stack.
    scavenge(&mut t, P0, a, bears).unwrap();
    assert!(scavenge(&mut t, P0, b, bears).is_err());
    t.resolve();
    // Not during combat or an opponent's turn.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(scavenge(&mut t, P0, b, bears).is_err());
    t.set_step(P1, Step::PrecombatMain);
    assert!(scavenge(&mut t, P0, b, bears).is_err());
    t.set_step(P0, Step::PostcombatMain);
    assert!(scavenge(&mut t, P0, b, bears).is_ok());
    t.resolve();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
}

#[test]
fn the_number_of_counters_is_the_power_the_card_last_had_in_the_graveyard() {
    cr!("702.97a", "608.2h");
    ruling!(
        "Boneyard Mycodrax",
        "The number of counters given by Boneyard Mycodrax’s scavenge ability is its power as it last existed in your graveyard before exiling it."
    );
    assert_supported("Boneyard Mycodrax");
    let mut t = TestGame::new(2);
    let myco = t.graveyard(P0, "Boneyard Mycodrax");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    // It counts the other creature cards in its owner's graveyard, in the graveyard too.
    assert_eq!(t.pt(myco), (2, 2));
    let target = t.battlefield(P0, "Llanowar Elves");
    let victim = t.battlefield(P0, "Elite Vanguard");
    t.lands(P0, "Swamp", 5);
    scavenge(&mut t, P0, myco, target).unwrap();
    // More creature cards reach the graveyard in response: too late to matter.
    destroy(&mut t, victim);
    t.settle();
    t.resolve();
    assert_eq!(t.counters(target, counters::PLUS1), 2);
}

#[test]
fn scavenge_does_nothing_if_its_target_becomes_illegal() {
    cr!("702.97a", "608.2b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let slither = t.graveyard(P0, "Slitherhead");
    scavenge(&mut t, P0, slither, bears).unwrap();
    destroy(&mut t, bears);
    t.settle();
    t.resolve();
    // The card stays exiled.
    assert_eq!(t.zone(slither), Zone::Exile);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}
