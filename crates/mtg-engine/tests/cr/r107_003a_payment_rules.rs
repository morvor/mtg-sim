//! What a spell's or ability's text says about announcing and paying its cost, and other
//! changes to how a cost is paid that don't change the cost (see `payment_rules.rs`):
//! "X can't be 0" (CR 107.3a, 107.3b), "Spend only [color] mana on X" (CR 107.3a,
//! 601.2h), mana that can't pay generic mana (CR 106.6), and mana that triggers when it's
//! spent (CR 106.6, 603.7a).

use crate::r105_util::card_from_text;
use mtg_engine::card::card;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

/// The least value P0 was offered for X in the last "choose X" decision.
fn last_x_min(t: &TestGame) -> Option<i64> {
    t.asked().iter().rev().find_map(|(_, d)| match d {
        Decision::ChooseX { min, .. } => Some(*min),
        _ => None,
    })
}

/// "{X}{R} sorcery: X can't be 0. ~ deals X damage to any target."
fn blast() -> mtg_engine::card::CardDef {
    card_from_text(
        "Careful Blast",
        "{X}{R}",
        "Sorcery",
        None,
        "X can't be 0.\nCareful Blast deals X damage to any target.",
    )
}

#[test]
fn x_cant_be_0_is_the_least_value_that_may_be_announced() {
    cr!("107.3a", "601.2b");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let b = t.custom(P0, blast(), Zone::Hand(P0));
    // Announcing 0 isn't allowed: the greatest value that can be paid, 2, is announced
    // instead.
    t.cast(P0, b).target(P1).x(0).go();
    assert_eq!(last_x_min(&t), Some(1));
    t.resolve();
    assert_eq!(t.life(P1), 18);
    // X = 1 is.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let b = t.custom(P0, blast(), Zone::Hand(P0));
    t.cast(P0, b).target(P1).x(1).go();
    t.resolve();
    assert_eq!(t.life(P1), 19);
    // With one Mountain, X = 1 can't be paid, and X = 0 isn't an option.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let b = t.custom(P0, blast(), Zone::Hand(P0));
    assert!(t.cast(P0, b).target(P1).x(0).try_go().is_err());
    assert_eq!(t.zone(b), Zone::Hand(P0));
}

#[test]
fn x_cant_be_0_spells_cant_be_cast_without_paying_their_mana_cost() {
    cr!("107.3a", "107.3b");
    supported("Omniscience");
    // Omniscience: "You may cast spells from your hand without paying their mana costs."
    // X would have to be 0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Omniscience");
    t.lands(P0, "Mountain", 3);
    let b = t.custom(P0, blast(), Zone::Hand(P0));
    t.g.turn.priority = Some(P0);
    let free = Action::Cast {
        card: b,
        method: CastMethod::Free,
    };
    let normal = Action::Cast {
        card: b,
        method: CastMethod::Normal,
    };
    let actions = t.g.legal_actions(P0);
    assert!(!actions.contains(&free));
    assert!(actions.contains(&normal));
    assert!(t
        .cast(P0, b)
        .method(CastMethod::Free)
        .target(P1)
        .try_go()
        .is_err());
    // Lightning Bolt has no such rule.
    let bolt = t.hand(P0, "Lightning Bolt");
    t.g.turn.priority = Some(P0);
    assert!(t.g.legal_actions(P0).contains(&Action::Cast {
        card: bolt,
        method: CastMethod::Free,
    }));
}

#[test]
fn an_activated_ability_s_x_cant_be_0_is_offered_as_the_least_value() {
    cr!("107.3a", "602.2b");
    supported("Lair of the Hydra");
    // Lair of the Hydra: "{X}{G}: Until end of turn, this land becomes an X/X green Hydra
    // creature. It's still a land. X can't be 0."
    let mut t = TestGame::new(2);
    let lair = t.battlefield(P0, "Lair of the Hydra");
    t.lands(P0, "Forest", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    // (Its first activated ability is "{T}: Add {G}.")
    t.activate(P0, lair, 1, &[]).unwrap();
    assert_eq!(last_x_min(&t), Some(1));
    t.resolve();
    assert_eq!(t.pt(lair), (2, 2));
}

#[test]
fn only_mana_of_the_stated_colors_pays_an_ability_s_x() {
    cr!("107.3a", "601.2h", "602.2b");
    supported("Crypt Rats");
    // Crypt Rats: "{X}: This creature deals X damage to each creature and each player.
    // Spend only black mana on X."
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P0, "Crypt Rats");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    assert!(t.activate(P0, rats, 0, &[]).is_err());
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, rats, 0, &[]).unwrap();
    t.resolve();
    assert_eq!((t.life(P0), t.life(P1)), (19, 19));
    // The Mountain is still untapped.
    assert_eq!(
        t.g.battlefield
            .iter()
            .filter(|o| t.obj_now(**o).chars.name == "Mountain" && !t.obj_now(**o).tapped)
            .count(),
        1
    );

    supported("Crimson Hellkite");
    // Crimson Hellkite: "{X}, {T}: This creature deals X damage to target creature. Spend
    // only red mana on X."
    let mut t = TestGame::new(2);
    let kite = t.battlefield(P0, "Crimson Hellkite");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    assert!(t.activate(P0, kite, 0, &[Entity::Object(bears)]).is_err());
    t.lands(P0, "Mountain", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, kite, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn every_mode_s_spend_only_rule_applies_to_the_ability_s_x() {
    cr!("107.3a", "601.2h", "700.2");
    supported("Atalya, Samite Master");
    // Atalya: "{X}, {T}: Choose one — • Prevent the next X damage ... Spend only white
    // mana on X. • You gain X life. Spend only white mana on X."
    let mut t = TestGame::new(2);
    let atalya = t.battlefield(P0, "Atalya, Samite Master");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Forest", 2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    assert!(t.activate(P0, atalya, 0, &[]).is_err());
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, atalya, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn x_paid_with_mana_of_different_colors() {
    cr!("107.3a", "601.2h", "702.33a");
    supported("Emblazoned Golem");
    // Emblazoned Golem ({2}, Kicker {X}): "Spend only colored mana on X. No more than one
    // mana of each color may be spent this way." Kicked with X = 2 from four Plains: two
    // white mana can't both pay X.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let golem = t.hand(P0, "Emblazoned Golem");
    assert!(t.cast(P0, golem).kicked(true).x(2).try_go().is_err());
    // With an Island, white and blue mana pay X, and the Plains pay {2}.
    t.lands(P0, "Island", 1);
    t.cast(P0, golem).kicked(true).x(2).go();
    t.resolve();
    assert_eq!(t.counters(golem, counters::PLUS1), 2);
    // Colorless mana can't pay X at all.
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 3);
    let golem = t.hand(P0, "Emblazoned Golem");
    assert!(t.cast(P0, golem).kicked(true).x(1).try_go().is_err());
}

#[test]
fn mana_that_cant_pay_generic_mana() {
    cr!("106.6");
    supported("Jegantha, the Wellspring");
    // Jegantha: "{T}: Add {W}{U}{B}{R}{G}. This mana can't be spent to pay generic mana
    // costs." Lightning Bolt ({R}) is paid with its {R}; Mind Stone ({2}) can't be.
    let mut t = TestGame::new(2);
    let jegantha = t.battlefield(P0, "Jegantha, the Wellspring");
    t.activate(P0, jegantha, 0, &[]).unwrap();
    let stone = t.hand(P0, "Mind Stone");
    assert!(t.cast(P0, stone).try_go().is_err());
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    let left: Vec<ManaType> = t.g.player(P0).mana_pool.mana.iter().map(|m| m.ty).collect();
    assert_eq!(left.len(), 4);
    assert!(!left.contains(&ManaType::R));
}

#[test]
fn mana_that_triggers_when_spent_to_cast_a_spell() {
    cr!("106.6", "603.7a");
    supported("Scaled Nurturer");
    supported("Lapis Orb of Dragonkind");
    supported("Gilanra, Caller of Wirewood");
    // Scaled Nurturer: "{T}: Add {G}. When you spend this mana to cast a Dragon creature
    // spell, you gain 2 life." Shivan Dragon is one; Grizzly Bears isn't.
    let mut t = TestGame::new(2);
    let nurturer = t.battlefield(P0, "Scaled Nurturer");
    t.lands(P0, "Forest", 1);
    t.activate(P0, nurturer, 0, &[]).unwrap();
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    let mut t = TestGame::new(2);
    let nurturer = t.battlefield(P0, "Scaled Nurturer");
    t.lands(P0, "Mountain", 5);
    t.activate(P0, nurturer, 0, &[]).unwrap();
    let dragon = t.hand(P0, "Shivan Dragon");
    t.cast(P0, dragon).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // Lapis Orb of Dragonkind: "... When you spend this mana to cast a Dragon creature
    // spell, scry 2."
    let mut t = TestGame::new(2);
    let orb = t.battlefield(P0, "Lapis Orb of Dragonkind");
    t.lands(P0, "Mountain", 5);
    t.activate(P0, orb, 0, &[]).unwrap();
    let dragon = t.hand(P0, "Shivan Dragon");
    t.cast(P0, dragon).go();
    t.resolve_all();
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Scry { .. })));
    // Gilanra, Caller of Wirewood: "{T}: Add {G}. When you spend this mana to cast a spell
    // with mana value 6 or greater, draw a card."
    let mut t = TestGame::new(2);
    let gilanra = t.battlefield(P0, "Gilanra, Caller of Wirewood");
    t.lands(P0, "Mountain", 5);
    t.activate(P0, gilanra, 0, &[]).unwrap();
    let dragon = t.hand(P0, "Shivan Dragon");
    let hand = t.hand_size(P0);
    t.cast(P0, dragon).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}
