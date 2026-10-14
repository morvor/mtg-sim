//! Rulings batch P109 — putting several lands onto the battlefield: values fixed as the
//! ability resolves (CR 608.2h), "up to" and "any number" choices (CR 107.1c, 701.23b),
//! names of double-faced cards in the library (CR 712.8a, 201.2), and additional land
//! plays (CR 305.2).

use crate::r_p009_common::obj;
use crate::r_s01_common::supported;
use crate::r_s02_common::can_play_land;
use crate::r_s06_common::activate_containing;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn lands_on_battlefield(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(CardType::Land))
        .count()
}

#[test]
fn aurora_awakener_counts_colors_as_its_trigger_resolves() {
    cr!("608.2h", "603.3");
    ruling!(
        "Aurora Awakener",
        "The value of X is calculated only once, as Aurora Awakener's triggered ability resolves."
    );
    supported("Aurora Awakener");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Library, top first: Grizzly Bears, Lightning Bolt, Goblin Piker, Forest.
    let forest = t.library_top(P0, "Forest");
    let piker = t.library_top(P0, "Goblin Piker");
    t.library_top(P0, "Lightning Bolt");
    let bears = t.library_top(P0, "Grizzly Bears");
    // Only green when it triggers (X would be 1) ...
    t.enter(P0, "Aurora Awakener");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // ... but a blue permanent arrives before it resolves: X = 2.
    t.battlefield(P0, "Merfolk of the Pearl Trident");
    t.answer_choose(P0, &[obj(bears), obj(piker)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Two permanent cards were revealed; the Goblin Piker (red) entering doesn't make X 3.
    assert!(t.on_battlefield(piker));
    assert_eq!(t.zone(forest), Zone::Library(P0));
    assert!(!t.on_battlefield(forest), "X isn't recalculated");
}

#[test]
fn summer_bloom_lets_you_play_three_more_lands_normally() {
    cr!("305.2", "305.1");
    ruling!(
        "Summer Bloom",
        "This spell increases the number of lands you can play in a turn. The land cards are played as you would normally play lands."
    );
    supported("Summer Bloom");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 2);
    let bloom = t.hand(P0, "Summer Bloom");
    t.cast(P0, bloom).go();
    t.resolve_all();
    let hand: Vec<ObjectId> = (0..5).map(|_| t.hand(P0, "Forest")).collect();
    for (i, f) in hand.iter().enumerate() {
        let ok = can_play_land(&mut t, P0, *f);
        assert_eq!(ok, i < 4, "land play {}", i + 1);
        if ok {
            t.play_land(P0, *f).unwrap();
        }
    }
    assert_eq!(lands_on_battlefield(&t, P0), 6);
    // Played normally: not while a spell is on the stack.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 2);
    let bloom = t.hand(P0, "Summer Bloom");
    t.cast(P0, bloom).go();
    t.resolve_all();
    let f = t.hand(P0, "Forest");
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(!can_play_land(&mut t, P0, f), "not with a spell on the stack");
}

#[test]
fn genesis_wave_may_put_nothing_onto_the_battlefield() {
    cr!("107.1c");
    ruling!(
        "Genesis Wave",
        "You don't have to put permanent cards revealed this way onto the battlefield if you choose not to, regardless of their mana values."
    );
    supported("Genesis Wave");
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 5);
    let wave = t.hand(P0, "Genesis Wave");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_choose(P0, &[]);
    t.cast(P0, wave).go();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert_eq!(t.zone(forest), Zone::Graveyard(P0));
}

#[test]
fn patron_of_the_moon_puts_zero_one_or_two_lands() {
    cr!("608.2h", "602.2");
    ruling!(
        "Patron of the Moon",
        "You may put zero, one, or two lands onto the battlefield with Patron of the Moon’s ability. The number is determined as the ability resolves. You don’t need lands in hand to activate the ability."
    );
    supported("Patron of the Moon");
    for n in 0..=2usize {
        let mut t = TestGame::new(2);
        let patron = t.battlefield(P0, "Patron of the Moon");
        t.lands(P0, "Island", 1);
        // No lands in hand: it can still be activated.
        assert_eq!(t.hand_size(P0), 0);
        activate_containing(&mut t, P0, patron, "Put up to two")
            .expect("activated with no lands in hand");
        // Lands arrive in hand before it resolves; `n` of them are chosen then.
        let hand: Vec<ObjectId> = (0..3).map(|_| t.hand(P0, "Forest")).collect();
        let chosen: Vec<Entity> = hand[..n].iter().map(|f| obj(*f)).collect();
        t.answer_choose(P0, &chosen);
        t.resolve_all();
        let forests = t.named_on_battlefield("Forest");
        assert_eq!(forests.len(), n, "{n} lands");
        assert!(forests.iter().all(|f| t.obj(*f).tapped));
    }
}

#[test]
fn omenpath_journey_lands_with_different_names() {
    cr!("701.23b", "201.2", "712.8a");
    ruling!(
        "Omenpath Journey",
        "You may search your library for only one land card and exile it, if you like."
    );
    supported("Omenpath Journey");
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    t.library_top(P0, "Island");
    t.answer_choose(P0, &[obj(forest)]);
    t.enter(P0, "Omenpath Journey");
    t.resolve_all();
    assert_eq!(t.zone(forest), Zone::Exile);
    assert!(t.g.find_in_zone(Zone::Exile, "Island").is_empty());

    ruling!(
        "Omenpath Journey",
        "While in your library, a double-faced card has only the name of its front face. It’s not possible to reveal two of the same double-faced card this way."
    );
    let dfc = "Westvale Abbey // Ormendahl, Profane Prince";
    let mut t = TestGame::new(2);
    let a = t.library_top(P0, dfc);
    let b = t.library_top(P0, dfc);
    let forest = t.library_top(P0, "Forest");
    assert_eq!(t.g.obj(a).chars.name, "Westvale Abbey");
    t.answer_choose(P0, &[obj(a), obj(b), obj(forest)]);
    t.enter(P0, "Omenpath Journey");
    t.resolve_all();
    let exiled = [a, b]
        .iter()
        .filter(|id| t.zone(**id) == Zone::Exile)
        .count();
    assert!(exiled <= 1, "two cards named Westvale Abbey can't both be found");
}
