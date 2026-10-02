//! The counter grammar (`oracle/patterns/counter_grammar.rs`): how many counters —
//! "that many" counters an object had as it last existed, "whichever is greater", and
//! "from each of two creatures" (done only if both have one).

use mtg_engine::decision::Answer;
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
fn dismantle_puts_as_many_counters_as_the_destroyed_artifact_had() {
    cr!("608.2h", "603.10a");
    assert_supported(&["Dismantle"]);
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Coalition Relic");
    add(&mut t, theirs, "charge", 3);
    let mine = t.battlefield(P0, "Coalition Relic");
    t.lands(P0, "Mountain", 3);
    let d = t.hand(P0, "Dismantle");
    // The charge counters option.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, d).target(theirs).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Coalition Relic"));
    assert_eq!(t.counters(mine, "charge"), 3);
}

#[test]
fn dismantle_puts_no_counters_if_the_artifact_had_none() {
    cr!("608.2c");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Coalition Relic");
    let mine = t.battlefield(P0, "Coalition Relic");
    t.lands(P0, "Mountain", 3);
    let d = t.hand(P0, "Dismantle");
    t.cast(P0, d).target(theirs).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Coalition Relic"));
    assert!(t.obj_now(mine).counters.is_empty());
}

#[test]
fn yuna_puts_as_many_counters_as_the_permanent_had() {
    cr!("603.10a", "603.4");
    assert_supported(&["Yuna, Grand Summoner"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Yuna, Grand Summoner");
    let giant = t.battlefield(P0, "Hill Giant");
    let bear = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, giant, "+1/+1", 2);
    add(&mut t, giant, "charge", 1);
    t.answer_targets(P0, &[Entity::Object(bear)]);
    t.answer_yes(P0, true);
    t.g.destroy(giant, None);
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 3);
}

#[test]
fn willowdusk_uses_the_greater_of_life_gained_and_life_lost() {
    cr!("119.3", "107.1");
    assert_supported(&["Willowdusk, Essence Seer"]);
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Willowdusk, Essence Seer");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.gain_life(P0, 2);
    t.g.lose_life(P0, 5);
    t.lands(P0, "Swamp", 1);
    t.activate(P0, w, 0, &[Entity::Object(bear)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 5);
}

#[test]
fn dyadrine_draws_only_if_two_creatures_each_had_a_counter_removed() {
    cr!("608.2c", "603.12");
    assert_supported(&["Dyadrine, Synthesis Amalgam"]);
    for with_two in [false, true] {
        let mut t = TestGame::new(2);
        let d = t.battlefield(P0, "Dyadrine, Synthesis Amalgam");
        let bear = t.battlefield(P0, "Grizzly Bears");
        // Dyadrine has no counters here; one or two other creatures do.
        t.g.remove_counters(Entity::Object(d), "+1/+1", 100);
        add(&mut t, bear, "+1/+1", 1);
        if with_two {
            let giant = t.battlefield(P0, "Hill Giant");
            add(&mut t, giant, "+1/+1", 1);
        }
        t.answer_yes(P0, true);
        let hand = t.hand_size(P0);
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(bear, Entity::Player(P1))], &[]);
        if with_two {
            assert_eq!(t.hand_size(P0), hand + 1);
            assert_eq!(t.counters(bear, "+1/+1"), 0);
            let robots = t
                .g
                .battlefield
                .iter()
                .filter(|o| t.obj_now(**o).chars.has_subtype("Robot"))
                .count();
            assert_eq!(robots, 1);
        } else {
            assert_eq!(t.hand_size(P0), hand);
            assert_eq!(t.counters(bear, "+1/+1"), 1);
        }
    }
}

#[test]
fn sab_sunen_draws_when_it_has_an_odd_number_of_counters() {
    cr!("122.1", "608.2c");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sab-Sunen, Luxa Embodied");
    // Two counters before: three after (odd), draw two.
    add(&mut t, s, "charge", 2);
    t.advance_to(P1, Step::Upkeep);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1 + 2);
    // Four after (even): no cards.
    t.advance_to(P1, Step::Upkeep);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn ka_zars_zabu_grows_on_landfall() {
    cr!("111.4", "603.2");
    assert_supported(&["Ka-Zar of the Savage Land"]);
    let mut t = TestGame::new(2);
    t.enter(P0, "Ka-Zar of the Savage Land");
    t.resolve_all();
    let zabu = t.named_on_battlefield("Zabu");
    assert_eq!(zabu.len(), 1);
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(zabu[0], "+1/+1"), 1);
}
