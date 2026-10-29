//! Rulings batch P098 — "mana value N or less" among cards outside the stack: {X} is 0 in
//! graveyards, hands and libraries (CR 202.3e), split cards add both halves (CR 709.4),
//! printed costs determine the mana value, a card's mana value as it is in the library
//! (Turntimber Symbiosis), and "X" of a permanent that wasn't cast (Dune Drifter).

use crate::r_s01_common::*;
use crate::r_s03_common::choice_candidates;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// The candidates of every target or entity choice asked of P0 since `from`, merged.
fn offered(t: &TestGame, from: usize) -> Vec<Entity> {
    let mut v: Vec<Entity> = t.asked()[from..]
        .iter()
        .flat_map(|(p, d)| match d {
            Decision::ChooseTargets { candidates, .. } if *p == P0 => candidates.clone(),
            Decision::ChooseEntities { candidates, .. } if *p == P0 => candidates.clone(),
            _ => vec![],
        })
        .collect();
    v.sort();
    v.dedup();
    v
}

/// Each case: the card whose ability chooses among cards, the zone it chooses from, a card
/// that must be offered and one that must not, and how the ability is made to trigger or
/// resolve. Endless One ({X}) counts as mana value 0 in every zone.
#[test]
fn x_is_zero_and_printed_costs_count_for_cards_outside_the_stack() {
    cr!("202.3e", "202.3a");
    ruling!(
        "Renegade Rallier",
        "If the mana cost of a card in your graveyard includes {X}, X is considered to be 0."
    );
    ruling!(
        "Shepherd of the Cosmos",
        "If the mana cost of a card in your graveyard includes {X}, X is considered to be 0."
    );
    ruling!(
        "Sun Titan",
        "If the mana cost of a card in your graveyard includes {X}, X is considered to be 0."
    );
    ruling!(
        "Teshar, Ancestor's Apostle",
        "If the mana cost of a card in your graveyard includes {X}, X is considered to be 0."
    );
    ruling!(
        "Kellan Joins Up",
        "If the mana cost of a card in your hand includes {X}, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Spellseeker",
        "If the mana cost of a card in your library includes {X}, X is considered to be 0."
    );
    ruling!(
        "Devoted Crop-Mate",
        "The mana value of a card in your graveyard is determined solely by the mana symbols printed in its upper right corner."
    );
    ruling!(
        "Renegade Rallier",
        "The mana value of a card in your graveyard is determined solely by the mana symbols printed in its upper right corner. The mana value is the total amount of mana in that cost, regardless of color. For example, a card with mana cost {1}{U}{U} has mana value 3."
    );
    ruling!(
        "Sun Titan",
        "For example, a card with mana cost {3}{U}{U} has mana value 5."
    );
    ruling!(
        "Kellan Joins Up",
        "The mana value of a split card in your hand is based on the combined mana cost of its two halves."
    );
    ruling!(
        "Renegade Rallier",
        "You can target any permanent card in your graveyard with mana value 2 or less, not just one that was put there from the battlefield this turn."
    );
    // (card, zone, offered, not offered)
    type Case = (&'static str, Zone, &'static str, &'static str);
    let cases: [Case; 8] = [
        (
            "Sun Titan",
            Zone::Graveyard(P0),
            "Endless One",
            "Air Elemental",
        ),
        (
            "Shepherd of the Cosmos",
            Zone::Graveyard(P0),
            "Endless One",
            "Phantom Warrior",
        ),
        (
            "Renegade Rallier",
            Zone::Graveyard(P0),
            "Endless One",
            "Phantom Warrior",
        ),
        (
            "Renegade Rallier",
            Zone::Graveyard(P0),
            "Grizzly Bears",
            "Hill Giant",
        ),
        (
            "Devoted Crop-Mate",
            Zone::Graveyard(P0),
            "Grizzly Bears",
            "Phantom Warrior",
        ),
        (
            "Teshar, Ancestor's Apostle",
            Zone::Graveyard(P0),
            "Endless One",
            "Air Elemental",
        ),
        (
            "Kellan Joins Up",
            Zone::Hand(P0),
            "Endless One",
            "Fire // Ice",
        ),
        (
            "Spellseeker",
            Zone::Library(P0),
            "Blaze",
            "Stroke of Genius",
        ),
    ];
    for (name, zone, yes, no) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        let yes_id = place(&mut t, P0, yes, zone);
        let no_id = place(&mut t, P0, no, zone);
        let from = t.asked().len();
        match name {
            "Renegade Rallier" => {
                // Revolt: a Sol Ring P0 controlled left the battlefield this turn (it's
                // offered too: it's in the graveyard with mana value 1).
                let ring = t.battlefield(P0, "Sol Ring");
                crate::r_s02_common::destroy(&mut t, ring);
                t.enter(P0, name);
            }
            "Devoted Crop-Mate" => {
                let mate = t.battlefield(P0, name);
                t.answer_yes(P0, true);
                attack_with(&mut t, &[(mate, Entity::Player(P1))]);
            }
            "Teshar, Ancestor's Apostle" => {
                t.battlefield(P0, name);
                let ring = t.hand(P0, "Sol Ring");
                t.lands(P0, "Wastes", 1);
                t.cast(P0, ring).go();
            }
            "Kellan Joins Up" | "Spellseeker" => {
                t.answer_yes(P0, true);
                t.enter(P0, name);
            }
            _ => {
                t.answer_yes(P0, true);
                t.enter(P0, name);
            }
        }
        t.resolve_all();
        let o = offered(&t, from);
        assert!(o.contains(&obj(yes_id)), "{name}: {yes} not offered: {o:?}");
        assert!(!o.contains(&obj(no_id)), "{name}: {no} offered");
    }
}

#[test]
fn no_one_left_behind_x_creature_card_gets_the_reduction() {
    cr!("202.3e", "601.2f");
    ruling!(
        "No One Left Behind",
        "If the targeted creature card has an {X} in its mana cost, X is 0."
    );
    supported("No One Left Behind");
    // "This spell costs {3} less to cast if it targets a creature card with mana value 3
    // or less." {4}{B} - {3} = {1}{B}.
    let mut t = TestGame::new(2);
    let one = t.graveyard(P0, "Endless One");
    let spell = t.hand(P0, "No One Left Behind");
    t.lands(P0, "Swamp", 2);
    // Two lands pay for it.
    assert!(t.cast(P0, spell).target(one).try_go().is_ok());
    assert_eq!(tapped_lands(&t, P0), 2);
    t.resolve_all();
    // (Endless One returns as a 0/0 and dies.)
    assert!(t.in_graveyard(P0, "No One Left Behind"));
}

#[test]
fn dune_drifter_not_cast_x_is_zero() {
    cr!("107.3f", "603.2");
    ruling!(
        "Dune Drifter",
        "If this Vehicle enters and wasn’t cast, or it was cast without paying its mana cost, X is 0."
    );
    supported("Dune Drifter");
    let mut t = TestGame::new(2);
    let orni = t.graveyard(P0, "Ornithopter");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.enter(P0, "Dune Drifter");
    t.resolve_all();
    let o = offered(&t, from);
    assert!(o.contains(&obj(orni)));
    assert!(!o.contains(&obj(bears)));
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
}

#[test]
fn turntimber_symbiosis_uses_the_mana_value_in_the_library() {
    cr!("614.12", "202.3");
    ruling!(
        "Turntimber Symbiosis // Turntimber, Serpentine Wood",
        "Use the mana value of the creature card as it exists in your library to determine whether it receives +1/+1 counters, ignoring any replacement effects (such as that of Glasspool Mimic) that will change its mana value once it's on the battlefield."
    );
    supported("Turntimber Symbiosis // Turntimber, Serpentine Wood");
    let mut t = TestGame::new(2);
    let djinn = t.battlefield(P0, "Air Elemental");
    let mimic = t.library_top(P0, "Glasspool Mimic");
    let spell = t.hand(P0, "Turntimber Symbiosis // Turntimber, Serpentine Wood");
    t.lands(P0, "Forest", 7);
    t.answer_choose(P0, &[obj(mimic)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(djinn)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    let copies = t.named_on_battlefield("Air Elemental");
    assert_eq!(copies.len(), 2, "{}", t.dump_log());
    let copy = *copies.iter().find(|c| **c != djinn).unwrap();
    assert_eq!(t.counters(copy, counters::PLUS1), 3);
    assert_eq!(t.pt(copy), (7, 7));
}

#[test]
fn ranger_of_eos_finds_zero_one_or_two_creatures_of_mana_value_1_or_less() {
    cr!("701.23d");
    ruling!(
        "Ranger of Eos",
        "You may choose to find zero, one, or two creature cards in your library. Each card you find must have mana value 1 or less."
    );
    supported("Ranger of Eos");
    for n in 0..=2usize {
        let mut t = TestGame::new(2);
        let elves = t.library_top(P0, "Llanowar Elves");
        let one = t.library_top(P0, "Endless One");
        let bears = t.library_top(P0, "Grizzly Bears");
        let from = t.asked().len();
        t.answer_yes(P0, true);
        let picks: Vec<Entity> = [elves, one][..n].iter().map(|c| obj(*c)).collect();
        t.answer_choose(P0, &picks);
        t.enter(P0, "Ranger of Eos");
        t.resolve_all();
        let o: Vec<Entity> = choice_candidates(&t, from, "")
            .into_iter()
            .flatten()
            .collect();
        assert!(o.contains(&obj(elves)) && o.contains(&obj(one)));
        assert!(!o.contains(&obj(bears)));
        assert_eq!(t.hand_size(P0), n, "{n}");
    }
}
