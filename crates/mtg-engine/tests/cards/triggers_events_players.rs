//! Player-event trigger conditions (CR 603.2): creating tokens (CR 111.2), sacrificing
//! (CR 701.21), discarding (CR 701.9), milling (CR 701.17, per card or per batch), gaining
//! or losing life, getting energy (CR 107.14), clashing (CR 701.30), the Nth card drawn,
//! and an opponent gaining control of your permanents (CR 603.10d).

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

fn count_subtype(t: &TestGame, subtype: &str) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| t.g.obj(**id).chars.has_subtype(subtype))
        .count()
}

#[test]
fn whenever_you_create_a_token() {
    cr!("111.2", "603.2c");
    supported("Rosie Cotton of South Lane");
    ruling!(
        "Rosie Cotton of South Lane",
        "will trigger once for each token you created"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rosie Cotton of South Lane");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let fodder = t.hand(P0, "Dragon Fodder");
    t.answer_targets(P0, &[Entity::Object(bear)]);
    t.answer_targets(P0, &[Entity::Object(bear)]);
    t.cast(P0, fodder).go();
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 2, "two Goblins created");
    // An opponent's token: no.
    let theirs = t.hand(P1, "Dragon Fodder");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 2);
    t.cast(P1, theirs).go();
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 2);
}

#[test]
fn one_or_more_creature_tokens_and_first_time_each_turn() {
    cr!("111.2", "603.2c");
    supported("Staff of the Storyteller");
    supported("Akim, the Soaring Wind");
    let mut t = TestGame::new(2);
    let staff = t.battlefield(P0, "Staff of the Storyteller");
    t.battlefield(P0, "Akim, the Soaring Wind");
    t.lands(P0, "Mountain", 4);
    let fodder = t.hand(P0, "Dragon Fodder");
    t.cast(P0, fodder).go();
    t.resolve_all();
    let birds = |t: &TestGame| {
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).chars.has_subtype("Bird") && t.g.obj(**id).is_token())
            .count()
    };
    // One story counter for the two Goblins, one for Akim's Bird.
    assert_eq!(t.counters(staff, "story"), 2);
    assert_eq!(birds(&t), 1);
    let fodder = t.hand(P0, "Dragon Fodder");
    t.cast(P0, fodder).go();
    t.resolve_all();
    assert_eq!(t.counters(staff, "story"), 3);
    assert_eq!(birds(&t), 1, "Akim: only the first time each turn");
}

#[test]
fn whenever_you_create_or_sacrifice_a_token() {
    cr!("111.2", "701.21a");
    supported("Mirkwood Bats");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mirkwood Bats");
    t.lands(P0, "Mountain", 2);
    let fodder = t.hand(P0, "Dragon Fodder");
    t.cast(P0, fodder).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    let goblin =
        t.g.battlefield
            .iter()
            .copied()
            .find(|id| t.g.obj(*id).chars.has_subtype("Goblin"))
            .unwrap();
    t.g.sacrifice(goblin, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // A nontoken: no.
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.sacrifice(bear, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn whenever_you_sacrifice_this_or_another_creature() {
    cr!("701.21a", "603.10a");
    supported("Kingpin, Wilson Fisk");
    let mut t = TestGame::new(2);
    let kingpin = t.battlefield(P0, "Kingpin, Wilson Fisk");
    t.g.sacrifice(kingpin, P0);
    t.resolve_all();
    assert_eq!(count_subtype(&t, "Treasure"), 2, "sacrificing itself");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kingpin, Wilson Fisk");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.g.sacrifice(a, P0);
    t.resolve_all();
    t.g.sacrifice(b, P0);
    t.resolve_all();
    assert_eq!(count_subtype(&t, "Treasure"), 2, "only once each turn");
}

#[test]
fn one_or_more_players_sacrifice_one_or_more_creatures() {
    cr!("701.21a", "603.2c");
    supported("Evin, Waterdeep Opportunist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Evin, Waterdeep Opportunist");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.sacrifice(theirs, P1);
    t.resolve_all();
    assert_eq!(count_subtype(&t, "Treasure"), 1);
    let ring = t.battlefield(P1, "Sol Ring");
    t.g.sacrifice(ring, P1);
    t.resolve_all();
    assert_eq!(count_subtype(&t, "Treasure"), 1);
}

#[test]
fn one_or_more_players_discard_one_or_more_cards() {
    cr!("701.9a", "603.2c");
    supported("Hostile Investigator");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hostile Investigator");
    let a = t.hand(P1, "Island");
    let b = t.hand(P0, "Island");
    t.g.discard(P1, a, None);
    t.resolve_all();
    t.g.discard(P0, b, None);
    t.resolve_all();
    assert_eq!(count_subtype(&t, "Clue"), 1, "only once each turn");
}

#[test]
fn discard_a_spirit_card_or_a_card_with_disturb() {
    cr!("701.9a");
    supported("Shipwreck Sifters");
    let mut t = TestGame::new(2);
    let sifters = t.battlefield(P0, "Shipwreck Sifters");
    for name in [
        "Island",
        "Spectral Sailor",
        "Lunarch Veteran // Luminous Phantom",
    ] {
        let c = t.hand(P0, name);
        t.g.discard(P0, c, None);
        t.resolve_all();
    }
    // Spectral Sailor is a Spirit; Lunarch Veteran has disturb.
    assert_eq!(t.counters(sifters, "+1/+1"), 2);
}

#[test]
fn a_player_mills_a_nonland_card() {
    cr!("701.17a", "701.17c");
    supported("Glowing One");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Glowing One");
    t.library_top(P1, "Grizzly Bears");
    t.library_top(P1, "Island");
    t.library_top(P1, "Sol Ring");
    t.g.mill(P1, 3);
    t.resolve_all();
    assert_eq!(t.life(P0), 22, "once for each nonland card milled");
    // Other ways from a library to a graveyard aren't milling.
    let c = t.library_top(P1, "Grizzly Bears");
    t.g.move_object(
        c,
        mtg_engine::object::Zone::Graveyard(P1),
        mtg_engine::events::MoveCause::Effect,
        Some(P1),
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn one_or_more_nonland_cards_are_milled() {
    cr!("701.17a", "603.2c");
    supported("Mirelurk Queen");
    supported("The Wise Mothman");
    ruling!(
        "Mirelurk Queen",
        "will trigger exactly once as long as at least one nonland card was milled"
    );
    let mut t = TestGame::new(2);
    let queen = t.battlefield(P0, "Mirelurk Queen");
    t.library_top(P1, "Grizzly Bears");
    t.library_top(P1, "Sol Ring");
    t.library_top(P1, "Island");
    let hand = t.hand_size(P0);
    t.g.mill(P1, 3);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.counters(queen, "+1/+1"), 1);
    // Only lands: no.
    let mut t = TestGame::new(2);
    let queen = t.battlefield(P0, "Mirelurk Queen");
    t.library_top(P1, "Island");
    t.g.mill(P1, 1);
    t.resolve_all();
    assert_eq!(t.counters(queen, "+1/+1"), 0);
    // "where X is the number of nonland cards milled this way".
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Wise Mothman");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let c = t.battlefield(P0, "Grizzly Bears");
    t.library_top(P1, "Grizzly Bears");
    t.library_top(P1, "Sol Ring");
    t.library_top(P1, "Island");
    t.answer_targets(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.g.mill(P1, 3);
    t.resolve_all();
    let total: u32 = [a, b, c].iter().map(|x| t.counters(*x, "+1/+1")).sum();
    assert_eq!(total, 2, "up to two targets");
}

#[test]
fn a_player_mills_one_or_more_creature_cards() {
    cr!("701.17a", "603.2c");
    supported("Zellix, Sanity Flayer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zellix, Sanity Flayer");
    let horrors = count_subtype(&t, "Horror");
    t.library_top(P1, "Grizzly Bears");
    t.library_top(P1, "Grizzly Bears");
    t.g.mill(P1, 2);
    t.resolve_all();
    assert_eq!(count_subtype(&t, "Horror"), horrors + 1);
    t.library_top(P0, "Island");
    t.g.mill(P0, 1);
    t.resolve_all();
    assert_eq!(count_subtype(&t, "Horror"), horrors + 1);
}

#[test]
fn that_many_nonland_cards_milled_only_once_each_turn() {
    cr!("603.2c", "603.2h", "701.17a");
    supported("Screeching Scorchbeast");
    ruling!(
        "Screeching Scorchbeast",
        "If you choose not to create tokens, the ability will trigger again the next time"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Screeching Scorchbeast");
    let mill = |t: &mut TestGame, cards: &[&str], create: bool| {
        for c in cards {
            t.library_top(P1, c);
        }
        t.answer_yes(P0, create);
        t.g.mill(P1, cards.len() as u32);
        t.resolve_all();
    };
    // Declined: it triggers again.
    mill(&mut t, &["Grizzly Bears"], false);
    assert_eq!(count_subtype(&t, "Zombie"), 0);
    // Two nonland cards and a land: two Zombie Mutants.
    mill(&mut t, &["Grizzly Bears", "Island", "Sol Ring"], true);
    assert_eq!(count_subtype(&t, "Zombie"), 2);
    // Done this turn.
    mill(&mut t, &["Grizzly Bears", "Sol Ring"], true);
    assert_eq!(count_subtype(&t, "Zombie"), 2);
}

#[test]
fn an_opponent_mills_a_nonland_card_from_your_graveyard() {
    cr!("701.17a", "113.6");
    supported("Infesting Radroach");
    ruling!(
        "Infesting Radroach",
        "milled at the same time an opponent mills a nonland card"
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Infesting Radroach");
    t.library_top(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.g.mill(P1, 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Infesting Radroach"));
    // Your own mill: no.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Infesting Radroach");
    t.library_top(P0, "Grizzly Bears");
    t.g.mill(P0, 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Infesting Radroach"));
}

#[test]
fn an_opponent_discards_a_card_or_mills_one_or_more_cards() {
    cr!("701.9a", "701.17a");
    supported("Lo and Li, Royal Advisors");
    let mut t = TestGame::new(2);
    let lo = t.battlefield(P0, "Lo and Li, Royal Advisors");
    t.g.mill(P1, 2);
    t.resolve_all();
    assert_eq!(t.counters(lo, "+1/+1"), 1, "one milling");
    let c = t.hand(P1, "Island");
    t.g.discard(P1, c, None);
    t.resolve_all();
    assert_eq!(t.counters(lo, "+1/+1"), 2);
    t.g.mill(P0, 1);
    t.resolve_all();
    assert_eq!(t.counters(lo, "+1/+1"), 2, "not an opponent");
}

#[test]
fn whenever_you_gain_or_lose_life_during_your_turn() {
    cr!("603.2", "119.3");
    supported("Wax-Wane Witness");
    ruling!(
        "Wax-Wane Witness",
        "no matter how much life was gained or lost"
    );
    let mut t = TestGame::new(2);
    let witness = t.battlefield(P0, "Wax-Wane Witness");
    t.g.gain_life(P0, 3);
    t.resolve_all();
    t.g.lose_life(P0, 2);
    t.resolve_all();
    assert_eq!(t.pt(witness), (4, 4));
    t.set_step(P1, Step::PrecombatMain);
    t.g.lose_life(P0, 1);
    t.resolve_all();
    assert_eq!(t.pt(witness), (4, 4));
}

#[test]
fn whenever_you_get_one_or_more_energy() {
    cr!("107.14", "122.6");
    supported("Territorial Gorger");
    supported("Aether Revolt");
    ruling!("Territorial Gorger", "only gets +2/+2 once, not +2/+2 per");
    let mut t = TestGame::new(2);
    let gorger = t.battlefield(P0, "Territorial Gorger");
    // Aether Hub: "When this land enters, you get {E}." — then pay energy for mana.
    t.enter(P0, "Aether Hub");
    t.resolve_all();
    assert_eq!(t.pt(gorger), (4, 4));
    // Aether Revolt: "~ deals that much damage to any target".
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aether Revolt");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Aether Hub");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // An opponent's energy: no.
    t.enter(P1, "Aether Hub");
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn whenever_you_clash_and_win() {
    cr!("701.30a", "701.30c");
    supported("Sylvan Echoes");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sylvan Echoes");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    t.library_top(P0, "Craw Wurm");
    t.library_top(P1, "Island");
    let lash = t.hand(P0, "Lash Out");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.cast(P0, lash).target(bear).go();
    t.resolve_all();
    // Lash Out left the hand; a card was drawn.
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.life(P1), 17, "won the clash");
}

#[test]
fn draw_your_first_or_second_card_each_turn() {
    cr!("121.2");
    supported("Lady Octopus, Inspired Inventor");
    let mut t = TestGame::new(2);
    let octo = t.battlefield(P0, "Lady Octopus, Inspired Inventor");
    for _ in 0..3 {
        t.g.draw_cards(P0, 1);
        t.resolve_all();
    }
    assert_eq!(t.counters(octo, "ingenuity"), 2);
}

#[test]
fn a_player_draws_their_second_card_during_their_turn() {
    cr!("121.2");
    supported("The Council of Four");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Council of Four");
    t.set_step(P1, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    t.g.draw_cards(P1, 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    t.g.draw_cards(P1, 1);
    t.resolve_all();
    assert_eq!(
        t.hand_size(P0),
        hand + 1,
        "their second card, during their turn"
    );
    // P0's own second draw during P1's turn isn't during their turn.
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn an_opponent_gains_control_of_a_permanent_from_you() {
    cr!("603.10d");
    supported("Zidane, Tantalus Thief");
    ruling!(
        "Zidane, Tantalus Thief",
        "If an opponent gains control of Zidane from you"
    );
    let mut t = TestGame::new(2);
    let zidane = t.battlefield(P0, "Zidane, Tantalus Thief");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 6);
    let treason = t.hand(P1, "Act of Treason");
    t.cast(P1, treason).target(bear).go();
    t.resolve_all();
    let treasures = |t: &TestGame| {
        t.g.battlefield
            .iter()
            .filter(|id| {
                t.g.obj(**id).chars.has_subtype("Treasure") && t.g.obj(**id).controller == P0
            })
            .count()
    };
    assert_eq!(treasures(&t), 1);
    let treason = t.hand(P1, "Act of Treason");
    t.cast(P1, treason).target(zidane).go();
    t.resolve_all();
    assert_eq!(treasures(&t), 2, "Zidane itself");
}

#[test]
fn a_player_puts_a_nontoken_creature_onto_the_battlefield() {
    cr!("110.2a", "608.3a");
    supported("Overburden");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Overburden");
    let forest = t.battlefield(P1, "Forest");
    t.enter(P1, "Grizzly Bears");
    t.resolve_all();
    // That player returns a land they control to its owner's hand.
    assert!(!t.on_battlefield(forest));
    assert!(t.in_hand(P1, "Forest"));
}
