//! Aggregates over players (CR 608.2h, 101.4): Balance and its relatives.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn count(t: &TestGame, p: PlayerId, ty: types::CardType) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| {
            let o = t.g.obj(**id);
            o.controller == p && o.chars.card_types.contains(ty)
        })
        .count()
}

#[test]
fn balance_evens_out_lands_hands_and_creatures() {
    cr!("101.4", "608.2h");
    ruling!(
        "Balance",
        "Each type of object is counted during the corresponding part of the process."
    );
    ruling!(
        "Balance",
        "Balance doesn't have targets, so permanents that can't be targeted"
    );
    for c in [
        "Balance",
        "Restore Balance",
        "Balancing Act",
        "Magus of the Balance",
    ] {
        compiles(c);
    }
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    t.lands(P1, "Forest", 2);
    t.battlefield(P0, "Grizzly Bears");
    for _ in 0..3 {
        t.battlefield(P1, "Llanowar Elves");
    }
    // Hexproof doesn't matter.
    t.battlefield(P1, "Gladecover Scout");
    for _ in 0..3 {
        t.hand(P0, "Island");
    }
    for _ in 0..5 {
        t.hand(P1, "Island");
    }
    let balance = t.hand(P0, "Balance");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, balance).go();
    t.resolve_all();
    use types::CardType::{Creature, Land};
    assert_eq!((count(&t, P0, Land), count(&t, P1, Land)), (2, 2));
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (3, 3));
    assert_eq!((count(&t, P0, Creature), count(&t, P1, Creature)), (1, 1));
}

#[test]
fn balancing_act_evens_out_permanents_then_hands() {
    cr!("101.4");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    t.battlefield(P1, "Forest");
    t.battlefield(P1, "Llanowar Elves");
    t.hand(P0, "Island");
    for _ in 0..4 {
        t.hand(P1, "Island");
    }
    let act = t.hand(P0, "Balancing Act");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, act).go();
    t.resolve_all();
    let perms = |p: PlayerId| {
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).controller == p)
            .count()
    };
    assert_eq!((perms(P0), perms(P1)), (2, 2));
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (1, 1));
}

fn perms(t: &TestGame, p: PlayerId) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| t.g.obj(**id).controller == p)
        .count()
}

#[test]
fn pox_plague_halves_life_hand_and_permanents_rounding_down() {
    cr!("107.1a", "101.4");
    compiles("Pox Plague");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    t.lands(P1, "Forest", 3);
    t.g.lose_life(P1, 5);
    for _ in 0..3 {
        t.hand(P0, "Island");
    }
    for _ in 0..4 {
        t.hand(P1, "Island");
    }
    let pox = t.hand(P0, "Pox Plague");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, pox).go();
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (10, 8));
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (2, 2));
    assert_eq!((perms(&t, P0), perms(&t, P1)), (3, 2));
}

#[test]
fn fraying_omnipotence_rounds_the_amounts_lost_up() {
    cr!("107.1a", "101.4");
    ruling!(
        "Fraying Omnipotence",
        "The amounts that are lost are rounded up, not the amounts kept."
    );
    compiles("Fraying Omnipotence");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 6);
    t.g.lose_life(P1, 15);
    for _ in 0..5 {
        t.hand(P1, "Island");
        t.battlefield(P1, "Grizzly Bears");
    }
    let spell = t.hand(P0, "Fraying Omnipotence");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell).go();
    t.resolve_all();
    // 5 life, five cards, five creatures: loses 3, discards 3, sacrifices 3.
    assert_eq!(t.life(P1), 2);
    assert_eq!(t.hand_size(P1), 2);
    assert_eq!(perms(&t, P1), 2);
    assert_eq!(t.life(P0), 10);
}

#[test]
fn split_the_party_returns_half_the_creatures_rounded_up() {
    cr!("107.1a");
    ruling!(
        "Split the Party",
        "The player that controls Split the Party chooses which creatures to return"
    );
    compiles("Split the Party");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let mut theirs = Vec::new();
    for _ in 0..3 {
        theirs.push(t.battlefield(P1, "Grizzly Bears"));
    }
    let spell = t.hand(P0, "Split the Party");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(theirs[0]), Entity::Object(theirs[2])]);
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(!t.on_battlefield(theirs[0]) && !t.on_battlefield(theirs[2]));
    assert!(t.on_battlefield(theirs[1]));
    assert_eq!(t.hand_size(P1), 2);
}

#[test]
fn legate_lanius_decimates_rounding_up() {
    cr!("107.1a");
    ruling!(
        "Legate Lanius, Caesar's Ace",
        "an opponent who controls between one and ten creatures will sacrifice one creature"
    );
    compiles("Legate Lanius, Caesar's Ace");
    let mut t = TestGame::new(2);
    for _ in 0..11 {
        t.battlefield(P1, "Grizzly Bears");
    }
    t.enter(P0, "Legate Lanius, Caesar's Ace");
    t.resolve_all();
    assert_eq!(perms(&t, P1), 9, "a tenth of eleven, rounded up");
}
