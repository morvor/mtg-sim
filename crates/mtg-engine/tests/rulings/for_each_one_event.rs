//! One "for each" instruction, one event (`for_each_events.rs`; register
//! `for-each-tokens-counters-life-one-event`): the tokens, counters and life one
//! instruction creates, puts or gains "for each" of several players or objects are one
//! creation, one putting of counters on each object and one life gain, whatever shape the
//! instruction compiles to, so an additive replacement effect applies to it once.

use crate::r_s01_common::{supported, with_subtype};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::ability::{Effect, PlayerRef, Value};
use mtg_engine::*;

fn foods(t: &TestGame, p: PlayerId) -> usize {
    with_subtype(t, p, "Food").len()
}

fn treasures(t: &TestGame, p: PlayerId) -> usize {
    with_subtype(t, p, "Treasure").len()
}

#[test]
fn endless_ranks_of_hydra_creates_its_tokens_for_each_opponent_as_one_creation() {
    // "For each opponent, you create a 2/1 black Villain creature token with menace."
    // (compiled to a repetition): with Peregrin Took ("If one or more tokens would be
    // created under your control, those tokens plus an additional Food token are created
    // instead."), three Villains and one Food.
    cr!("608.2f", "614.1a", "614.5");
    interp!("for-each-tokens-counters-life-one-event");
    supported("Endless Ranks of HYDRA");
    supported("Peregrin Took");
    let mut t = TestGame::new(4);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Peregrin Took");
    t.lands(P0, "Swamp", 4);
    let s = t.hand(P0, "Endless Ranks of HYDRA");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Villain").len(), 3);
    assert_eq!(foods(&t, P0), 1);
}

#[test]
fn wurmquake_creates_the_rest_of_its_wurms_together() {
    cr!("608.2f", "614.1a", "614.5");
    interp!("for-each-tokens-counters-life-one-event");
    ruling!(
        "Wurmquake",
        "The first token that Wurmquake creates as it resolves is created and enters the battlefield before the rest of them, then you create a number of tokens equal to the number of opponents with three of more poison counters and they enter the battlefield at the same time."
    );
    supported("Wurmquake");
    // Three opponents with three or more poison counters: one Wurm, then three Wurms
    // together; Peregrin Took adds a Food to each of the two creations.
    let mut t = TestGame::new(4);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Peregrin Took");
    for p in [P1, P2, P3] {
        t.g.add_counters(Entity::Player(p), "poison", 3, None);
    }
    t.lands(P0, "Forest", 6);
    let w = t.hand(P0, "Wurmquake");
    t.cast(P0, w).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Phyrexian Wurm Token").len(), 4);
    assert_eq!(foods(&t, P0), 2);
}

#[test]
fn tempting_contract_creates_your_treasures_as_one_creation() {
    // "At the beginning of your upkeep, each opponent may create a Treasure token. For
    // each opponent who does, you create a Treasure token." With Xorn ("If you would
    // create one or more Treasure tokens, instead create those tokens plus an additional
    // Treasure token."): your three Treasures are one creation, so four. Each opponent's
    // Treasure is their own creation (each is a different "you"): the opponent with a
    // Xorn gets two.
    cr!("608.2f", "614.1a", "614.5", "101.4");
    interp!("for-each-tokens-counters-life-one-event");
    ruling!(
        "Tempting Contract",
        "Then you create your Treasure tokens all at the same time."
    );
    supported("Tempting Contract");
    supported("Xorn");
    let mut t = TestGame::new(4);
    t.battlefield(P0, "Tempting Contract");
    t.battlefield(P0, "Xorn");
    t.battlefield(P2, "Xorn");
    t.advance_to(P0, Step::Upkeep);
    for p in [P1, P2, P3] {
        t.answer_yes(p, true);
    }
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 4);
    assert_eq!(treasures(&t, P1), 1);
    assert_eq!(treasures(&t, P2), 2);
    assert_eq!(treasures(&t, P3), 1);
}

#[test]
fn brass_bounty_creates_its_treasures_as_one_creation() {
    // "For each land you control, create a Treasure token." (one creation with a count).
    cr!("608.2f", "614.1a");
    interp!("for-each-tokens-counters-life-one-event");
    supported("Brass's Bounty");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Xorn");
    t.lands(P0, "Mountain", 7);
    let s = t.hand(P0, "Brass's Bounty");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(treasures(&t, P0), 8);
}

#[test]
fn blood_money_creates_its_treasures_as_one_creation() {
    // "Destroy all creatures. For each nontoken creature destroyed this way, you create a
    // tapped Treasure token." (compiled to a loop over the destroyed creatures). With
    // Worldwalker Helm ("If you would create one or more artifact tokens, instead create
    // those tokens plus an additional Map token."): three Bears destroyed, three tapped
    // Treasures and one Map.
    cr!("608.2f", "614.1a", "614.5");
    interp!("for-each-tokens-counters-life-one-event");
    supported("Blood Money");
    supported("Worldwalker Helm");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Worldwalker Helm");
    for _ in 0..3 {
        t.battlefield(P1, "Grizzly Bears");
    }
    t.lands(P0, "Swamp", 7);
    let s = t.hand(P0, "Blood Money");
    t.cast(P0, s).go();
    t.resolve_all();
    let tr = with_subtype(&t, P0, "Treasure");
    assert_eq!(tr.len(), 3);
    assert!(tr.iter().all(|o| t.obj(*o).tapped));
    assert_eq!(with_subtype(&t, P0, "Map").len(), 1);
}

#[test]
fn tempt_with_glory_your_counters_for_each_opponent_who_does_are_one_event() {
    // "Put a +1/+1 counter on each creature you control. Each opponent may put a +1/+1
    // counter on each creature they control. For each opponent who does, put a +1/+1
    // counter on each creature you control." With Hardened Scales ("If one or more +1/+1
    // counters would be put on a creature you control, that many plus one +1/+1 counters
    // are put on it instead.") and two opponents accepting: 1 + 1, then 2 + 1.
    cr!("608.2f", "614.1a", "614.5");
    interp!("for-each-tokens-counters-life-one-event");
    supported("Tempt with Glory");
    supported("Hardened Scales");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hardened Scales");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P2, "Grizzly Bears");
    t.answer_yes(P1, true);
    t.answer_yes(P2, true);
    t.lands(P0, "Plains", 6);
    let s = t.hand(P0, "Tempt with Glory");
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.obj_now(bears).counter(counters::PLUS1), 5);
}

#[test]
fn skullslither_worms_counters_for_each_opponent_who_cant_are_one_event() {
    // "When this creature enters, each opponent discards a card. For each opponent who
    // can't, put two +1/+1 counters on this creature." Two opponents with empty hands, and
    // Hardened Scales: 4 + 1.
    cr!("608.2f", "614.1a", "614.5");
    interp!("for-each-tokens-counters-life-one-event");
    supported("Skullslither Worm");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Hardened Scales");
    let worm = t.enter(P0, "Skullslither Worm");
    t.resolve_all();
    assert_eq!(t.obj_now(worm).counter(counters::PLUS1), 5);
}

#[test]
fn cruel_grimnarchs_life_for_each_opponent_who_cant_is_one_life_gain() {
    // "When this creature enters, each opponent discards a card. For each opponent who
    // can't, you gain 4 life." Two opponents with empty hands, and Heron of Hope ("If you
    // would gain life, you gain that much life plus 1 instead."): 8 + 1.
    cr!("608.2f", "614.1a", "614.5", "119.10");
    interp!("for-each-tokens-counters-life-one-event");
    ruling!(
        "Heron of Hope",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and the ability of Heron of Hope applies only once."
    );
    supported("Cruel Grimnarch");
    supported("Heron of Hope");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Heron of Hope");
    let before = t.life(P0);
    t.enter(P0, "Cruel Grimnarch");
    t.resolve_all();
    assert_eq!(t.life(P0), before + 9);
}

#[test]
fn each_players_own_tokens_stay_their_own_creations() {
    // "Each player creates a Treasure token" is a creation by each player: Xorn adds one
    // to its controller's only, and two players' identical tokens aren't combined.
    cr!("608.2f", "614.1a", "101.4");
    interp!("for-each-tokens-counters-life-one-event");
    supported("Xorn");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Xorn");
    t.battlefield(P1, "Xorn");
    let spec = mtg_engine::tokens::predefined("Treasure").expect("predefined token");
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    t.g.exec(
        &Effect::ForEachPlayer {
            who: PlayerRef::EachPlayer,
            effect: Box::new(Effect::AsPlayer {
                who: PlayerRef::Iterated,
                effect: Box::new(Effect::CreateToken {
                    spec,
                    count: Value::Const(1),
                    controller: PlayerRef::You,
                    tapped: false,
                    attacking: false,
                }),
            }),
        },
        &mut ctx,
    );
    t.g.settle();
    assert_eq!(treasures(&t, P0), 2);
    assert_eq!(treasures(&t, P1), 2);
    assert_eq!(treasures(&t, P2), 1);
}
