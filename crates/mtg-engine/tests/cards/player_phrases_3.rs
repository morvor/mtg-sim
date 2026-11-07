//! More player phrases (`oracle/patterns/player_phrases_2.rs`): "opponents can't ..."
//! (your opponents), "deals N damage to an opponent chosen at random", and "that player"
//! after "defending player".

use mtg_engine::decision::Answer;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    let c = t.g.current(card);
    t.g.turn.priority = Some(p);
    t.g.cast_spell(p, c, CastMethod::Normal).is_ok()
}

#[test]
fn academic_probation_opponents_cant_cast_the_chosen_name() {
    cr!("601.3");
    compiles("Academic Probation");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.lands(P1, "Mountain", 2);
    let bolt = t.hand(P1, "Lightning Bolt");
    let shock = t.hand(P1, "Shock");
    let own_bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    let spell = t.hand(P0, "Academic Probation");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer(P0, DecisionKind::Name, Answer::Text("Lightning Bolt".into()));
    t.cast(P0, spell).modes(&[0]).go();
    t.resolve_all();
    assert!(!can_cast(&mut t, P1, bolt), "an opponent can't cast the name");
    assert!(can_cast(&mut t, P1, shock), "other names are fine");
    t.resolve_all();
    assert!(can_cast(&mut t, P0, own_bolt), "the caster isn't an opponent");
}

#[test]
fn zaffai_deals_ten_damage_to_one_opponent_chosen_at_random() {
    cr!("608.2d");
    compiles("Zaffai, Thunder Conductor");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Zaffai, Thunder Conductor");
    t.lands(P0, "Island", 12);
    let spell = t.hand(P0, "Enter the Infinite");
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, spell).go();
    t.resolve_all();
    let lives = [t.life(P0), t.life(P1), t.life(P2)];
    assert_eq!(lives[0], 20);
    assert!(
        lives[1..] == [10, 20] || lives[1..] == [20, 10],
        "exactly one opponent is dealt 10: {lives:?}"
    );
}

/// The Motherlode attacks P1, who has a nonbasic land and tries to block with Grizzly
/// Bears; P0 pays the energy if `pay`. P1's life afterwards and whether the land is gone.
fn motherlode(pay: bool) -> (i32, bool) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let motherlode = t.battlefield(P0, "The Motherlode, Excavator");
    t.g.add_counters(Entity::Player(P0), "energy", 4, None);
    let land = t.battlefield(P1, "Ancient Tomb");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(pay));
    t.attack(
        &[(motherlode, Entity::Player(P1))],
        &[(bears, motherlode)],
    );
    (t.life(P1), t.on_battlefield(land))
}

#[test]
fn motherlode_creatures_defending_player_controls_cant_block() {
    cr!("508.1", "509.1b");
    compiles("The Motherlode, Excavator");
    assert_eq!(motherlode(true), (15, false), "unblocked, land destroyed");
    assert_eq!(motherlode(false), (20, true), "blocked by the Bears");
}
