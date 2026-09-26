//! "Add that much [mana]" in triggered abilities: the amount of the triggering event (the
//! damage dealt, the number of attacking creatures), CR 603.2, 608.2h.

use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort();
    v
}

#[test]
fn that_much_mana_compiles() {
    assert_supported(&[
        "Sakiko, Mother of Summer",
        "Raphael, Ninja Destroyer",
        "Photon, Mighty Marvel",
        "Grand Warlord Radha",
        "Mark of Sakiko",
    ]);
    // "That much" after another instruction ("discard any number of cards. If you do,
    // draw that many cards and add that much {R}") isn't the event's amount.
    assert!(!card("Neheb, Dreadhorde Champion")
        .unsupported_text()
        .is_empty());
    assert!(!card("Mana Seism").unsupported_text().is_empty());
}

#[test]
fn that_much_is_the_damage_dealt() {
    cr!("603.2", "106.4");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sakiko, Mother of Summer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(bears, Entity::Player(P1)), (giant, Entity::Player(P1))],
        &[],
    );
    assert_eq!(t.life(P1), 15);
    // Two triggers: 2 and 3 green mana, kept through the end of combat.
    assert_eq!(pool(&t, P0), vec![ManaType::G; 5]);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(pool(&t, P0), vec![ManaType::G; 5]);
}

#[test]
fn that_much_is_the_damage_dealt_to_it() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    let raphael = t.battlefield(P0, "Raphael, Ninja Destroyer");
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(raphael).go();
    t.resolve(); // Lightning Bolt
    t.resolve(); // the enrage trigger
    assert_eq!(pool(&t, P0), vec![ManaType::R; 3]);
}

#[test]
fn that_much_is_the_number_of_attackers() {
    cr!("603.2", "106.1a");
    ruling!(
        "Grand Warlord Radha",
        "The amount of mana you’ll add is the number of creatures you attack with."
    );
    let mut t = TestGame::new(2);
    let radha = t.battlefield(P0, "Grand Warlord Radha");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.attack(
        &[
            (radha, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
            (giant, Entity::Player(P1)),
        ],
        &[],
    );
    assert_eq!(pool(&t, P0), vec![ManaType::R, ManaType::G, ManaType::G]);
}
