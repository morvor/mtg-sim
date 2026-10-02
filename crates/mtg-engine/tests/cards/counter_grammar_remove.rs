//! The counter grammar (`oracle/patterns/counter_grammar.rs`), removing counters (CR 122):
//! "remove all [kind] counters from [holder]", "remove a counter from [holder]" (a counter
//! of a kind the player removing it chooses), "remove up to N counters", "remove any
//! number of counters", holders that are players ("target permanent or opponent"),
//! groups ("each creature you control"), chosen objects ("a creature you control") and
//! suspended cards; and what was removed this way ("for each counter removed this way",
//! "that many", "if no counters were removed this way").

use mtg_engine::decision::Answer;
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

fn add(t: &mut TestGame, id: ObjectId, kind: &str, n: u32) {
    t.g.add_counters(Entity::Object(id), kind, n, None);
}

#[test]
fn vampire_hexmage_removes_every_counter_of_every_kind() {
    cr!("122.1", "122.1e", "704.5i");
    ruling!(
        "Vampire Hexmage",
        "Any permanent can be targeted by the second ability"
    );
    assert_supported(&["Vampire Hexmage"]);
    let mut t = TestGame::new(2);
    let hexmage = t.battlefield(P0, "Vampire Hexmage");
    let walker = t.battlefield(P1, "Ajani Goldmane");
    assert_eq!(t.counters(walker, "loyalty"), 4);
    let giant = t.battlefield(P1, "Hill Giant");
    add(&mut t, giant, "+1/+1", 2);
    add(&mut t, giant, "flying", 1);
    // The planeswalker loses all its loyalty and is put into the graveyard.
    t.activate(P0, hexmage, 0, &[Entity::Object(walker)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ajani Goldmane"));
    // Another one: counters of two kinds, all removed.
    let hexmage2 = t.battlefield(P0, "Vampire Hexmage");
    t.activate(P0, hexmage2, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(giant, "+1/+1"), 0);
    assert_eq!(t.counters(giant, "flying"), 0);
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn render_inert_removes_the_chosen_number_and_kinds_of_counters() {
    cr!("122.1", "107.1c");
    ruling!(
        "Render Inert",
        "You choose which counters to remove from the permanent, no matter who controls it."
    );
    assert_supported(&["Render Inert"]);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    add(&mut t, giant, "+1/+1", 3);
    add(&mut t, giant, "charge", 2);
    t.lands(P0, "Swamp", 3);
    let r = t.hand(P0, "Render Inert");
    // Three counters: two +1/+1 counters and a charge counter ("+1/+1" is listed first).
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.cast(P0, r).target(giant).go();
    t.resolve_all();
    assert_eq!(t.counters(giant, "+1/+1"), 1);
    assert_eq!(t.counters(giant, "charge"), 1);
}

#[test]
fn price_of_betrayal_removes_counters_from_an_opponent() {
    cr!("122.1", "122.1f");
    ruling!(
        "Price of Betrayal",
        "You may remove different kinds of counters from that one target"
    );
    assert_supported(&["Price of Betrayal"]);
    let mut t = TestGame::new(2);
    t.g.add_counters(Entity::Player(P1), "poison", 4, None);
    t.lands(P0, "Swamp", 1);
    let p = t.hand(P0, "Price of Betrayal");
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    t.cast(P0, p).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.g.player(P1).counter("poison"), 1);
}

#[test]
fn thrull_parasite_removes_one_counter_of_the_chosen_kind() {
    cr!("122.1");
    ruling!(
        "Thrull Parasite",
        "If it has more than one kind of counter, you'll choose one of those counters when the ability resolves."
    );
    assert_supported(&["Thrull Parasite"]);
    let mut t = TestGame::new(2);
    let parasite = t.battlefield(P0, "Thrull Parasite");
    let giant = t.battlefield(P1, "Hill Giant");
    add(&mut t, giant, "+1/+1", 2);
    add(&mut t, giant, "charge", 2);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, parasite, 0, &[Entity::Object(giant)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(giant, "+1/+1"), 2);
    assert_eq!(t.counters(giant, "charge"), 1);
}

#[test]
fn heartmender_removes_a_counter_from_each_creature_you_control() {
    cr!("122.1a");
    assert_supported(&["Heartmender"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heartmender");
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Craw Wurm");
    let theirs = t.battlefield(P1, "Hill Giant");
    add(&mut t, a, "-1/-1", 2);
    add(&mut t, b, "-1/-1", 1);
    add(&mut t, theirs, "-1/-1", 1);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(a, "-1/-1"), 1);
    assert_eq!(t.counters(b, "-1/-1"), 0);
    assert_eq!(t.counters(theirs, "-1/-1"), 1);
}

#[test]
fn pestilent_haze_removes_two_loyalty_from_each_planeswalker() {
    cr!("122.1e", "700.2");
    assert_supported(&["Pestilent Haze"]);
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Ajani Goldmane");
    let theirs = t.battlefield(P1, "Ajani Goldmane");
    t.lands(P0, "Swamp", 3);
    let h = t.hand(P0, "Pestilent Haze");
    t.cast(P0, h).modes(&[1]).go();
    t.resolve_all();
    assert_eq!(t.counters(mine, "loyalty"), 2);
    assert_eq!(t.counters(theirs, "loyalty"), 2);
}

#[test]
fn mister_hyde_draws_only_if_a_counter_was_removed() {
    cr!("608.2c", "700.2");
    assert_supported(&["Mister Hyde, Monster Within"]);
    let mut t = TestGame::new(2);
    let hyde = t.battlefield(P0, "Mister Hyde, Monster Within");
    let hand = t.hand_size(P0);
    // No creature with a counter: nothing is removed and no card is drawn.
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    // Now with a counter on a creature.
    add(&mut t, hyde, "+1/+1", 1);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(hyde, "+1/+1"), 0);
    // One card from this ability (the draw step hasn't happened yet).
    assert_eq!(t.hand_size(P0), hand + 1 + 1);
}

#[test]
fn coalition_relic_adds_a_mana_of_any_color_for_each_counter_removed() {
    cr!("106.1a", "608.2c");
    ruling!(
        "Coalition Relic",
        "you may add a different color of mana for each one"
    );
    assert_supported(&["Coalition Relic"]);
    let mut t = TestGame::new(2);
    let relic = t.battlefield(P0, "Coalition Relic");
    add(&mut t, relic, "charge", 2);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.counters(relic, "charge"), 0);
    assert_eq!(t.g.player(P0).mana_pool.mana.len(), 2);
    let _ = ManaType::W;
}

#[test]
fn give_take_draws_as_many_cards_as_counters_removed() {
    cr!("608.2c");
    ruling!(
        "Give // Take",
        "then remove all +1/+1 counters from it, then draw that many cards"
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    add(&mut t, giant, "+1/+1", 3);
    add(&mut t, giant, "charge", 1);
    assert!(card("Give // Take").faces[1].unsupported.is_empty());
    t.lands(P0, "Island", 3);
    let c = t.hand(P0, "Give // Take");
    let hand = t.hand_size(P0);
    t.cast(P0, c)
        .method(mtg_engine::object::CastMethod::Half(1))
        .target(giant)
        .go();
    t.resolve_all();
    assert_eq!(t.counters(giant, "+1/+1"), 0);
    assert_eq!(t.counters(giant, "charge"), 1);
    // Cast from hand (-1), three drawn.
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
}

#[test]
fn hex_parasite_gets_the_bonus_only_for_counters_removed() {
    cr!("107.3", "608.2c");
    ruling!(
        "Hex Parasite",
        "Hex Parasite only gets the bonus for counters actually removed"
    );
    assert_supported(&["Hex Parasite"]);
    let mut t = TestGame::new(2);
    let parasite = t.battlefield(P0, "Hex Parasite");
    let walker = t.battlefield(P1, "Ajani Goldmane");
    t.lands(P0, "Swamp", 6);
    // X = 5, choosing the most: only four counters there.
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.answer(P0, DecisionKind::Number, Answer::Number(4));
    t.activate(P0, parasite, 0, &[Entity::Object(walker)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ajani Goldmane"));
    assert_eq!(t.pt(parasite), (1 + 4, 1));
}

#[test]
fn ashling_deals_damage_equal_to_the_counters_removed_on_the_third_resolution() {
    cr!("608.2c", "608.2h");
    ruling!(
        "Ashling the Pilgrim",
        "\"That much damage\" refers to the number of +1/+1 counters removed from Ashling the Pilgrim."
    );
    assert_supported(&["Ashling the Pilgrim"]);
    let mut t = TestGame::new(2);
    let ashling = t.battlefield(P0, "Ashling the Pilgrim");
    add(&mut t, ashling, "+1/+1", 2);
    let giant = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 6);
    for _ in 0..3 {
        t.activate(P0, ashling, 0, &[]).unwrap();
        t.resolve_all();
    }
    // 2 + 3 counters were removed: 5 damage to each creature and each player.
    assert_eq!(t.life(P1), 20 - 5);
    assert_eq!(t.life(P0), 20 - 5);
    assert!(t.in_graveyard(P1, "Craw Wurm"), "{:?}", t.zone(giant));
    assert!(t.in_graveyard(P0, "Ashling the Pilgrim"));
}

#[test]
fn bounty_of_the_luxa_alternates_between_a_counter_and_mana() {
    cr!("608.2c");
    ruling!(
        "Bounty of the Luxa",
        "you'll still get only {C}{G}{U} when you remove them all"
    );
    assert_supported(&["Bounty of the Luxa"]);
    let mut t = TestGame::new(2);
    let bounty = t.battlefield(P0, "Bounty of the Luxa");
    t.advance_to(P1, Step::Upkeep);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    // No counters removed: a flood counter and a card.
    assert_eq!(t.counters(bounty, "flood"), 1);
    assert!(t.g.player(P0).mana_pool.mana.is_empty());
    // P0's draw step and this ability.
    assert_eq!(t.hand_size(P0), hand + 1 + 1);
    // Next time, with two flood counters: only {C}{G}{U}.
    add(&mut t, bounty, "flood", 1);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.counters(bounty, "flood"), 0);
    assert_eq!(t.g.player(P0).mana_pool.mana.len(), 3);
}

#[test]
fn timecrafting_removes_time_counters_from_a_suspended_card() {
    cr!("702.62b", "122.1");
    assert_supported(&["Timecrafting"]);
    let mut t = TestGame::new(2);
    let rift = t.exile(P1, "Rift Bolt");
    add(&mut t, rift, "time", 3);
    t.lands(P0, "Mountain", 3);
    let tc = t.hand(P0, "Timecrafting");
    t.cast(P0, tc).modes(&[0]).x(2).target(rift).go();
    t.resolve_all();
    assert_eq!(t.counters(rift, "time"), 1);
}

#[test]
fn dust_of_moments_affects_permanents_and_suspended_cards() {
    cr!("702.62b");
    assert_supported(&["Dust of Moments"]);
    let mut t = TestGame::new(2);
    let rift = t.exile(P1, "Rift Bolt");
    add(&mut t, rift, "time", 3);
    // A card in exile without suspend isn't suspended.
    let bolt = t.exile(P1, "Lightning Bolt");
    add(&mut t, bolt, "time", 3);
    let relic = t.battlefield(P0, "Coalition Relic");
    add(&mut t, relic, "time", 2);
    t.lands(P0, "Plains", 3);
    let d = t.hand(P0, "Dust of Moments");
    t.cast(P0, d).modes(&[0]).go();
    t.resolve_all();
    assert_eq!(t.counters(rift, "time"), 1);
    assert_eq!(t.counters(bolt, "time"), 3);
    assert_eq!(t.counters(relic, "time"), 0);
}

#[test]
fn temporal_distortion_removes_hourglass_counters_of_that_players_permanents() {
    cr!("122.1", "502.3");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Temporal Distortion");
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Hill Giant");
    add(&mut t, mine, "hourglass", 1);
    add(&mut t, theirs, "hourglass", 1);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(theirs, "hourglass"), 0);
    assert_eq!(t.counters(mine, "hourglass"), 1);
}
