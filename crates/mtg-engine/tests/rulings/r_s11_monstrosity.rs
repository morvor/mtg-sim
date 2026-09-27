//! Rulings batch S11 — monstrosity (CR 701.37): "Monstrosity N" means "If this permanent
//! isn't monstrous, put N +1/+1 counters on it and it becomes monstrous." Monstrous is a
//! designation, not an ability or a copiable value (CR 701.37b).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::run_with;
use crate::r_s11_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn monstrous(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).monstrous
}

fn plus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}

/// Until end of turn, `id` stops being a creature (it becomes an artifact).
fn stop_being_a_creature(t: &mut TestGame, id: ObjectId) {
    run_with(
        t,
        P1,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetTypes {
                types: vec![CardType::Artifact],
                subtypes: vec![],
            }],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(id)],
    );
}

#[test]
fn monstrous_stays_true_after_losing_abilities_or_being_a_creature() {
    cr!("701.37b", "613.1f");
    ruling!(
        "Fleecemane Lion",
        "Monstrous isn't an ability that a creature has. It's just something true about that creature. If the creature stops being a creature or loses its abilities, it will continue to be monstrous."
    );
    supported("Fleecemane Lion");
    supported("Humility");
    // Fleecemane Lion: "{3}{G}{W}: Monstrosity 1." "As long as this creature is monstrous,
    // it has hexproof and indestructible."
    let mut t = TestGame::new(2);
    let lion = t.battlefield(P0, "Fleecemane Lion");
    t.lands(P0, "Forest", 8);
    t.lands(P0, "Plains", 6);
    t.activate(P0, lion, 0, &[]).unwrap();
    t.resolve_all();
    assert!(monstrous(&t, lion));
    assert!(t.obj(lion).chars.has_keyword(KeywordKind::Hexproof));
    // Humility takes its abilities away; it's still monstrous, and when Humility leaves,
    // it has hexproof and indestructible again.
    let humility = t.battlefield(P1, "Humility");
    t.g.recompute();
    assert!(!t.obj(lion).chars.has_keyword(KeywordKind::Indestructible));
    assert!(monstrous(&t, lion));
    crate::r_s02_common::destroy(&mut t, humility);
    assert!(t.obj(lion).chars.has_keyword(KeywordKind::Indestructible));
    // It stops being a creature for the turn: still monstrous.
    stop_being_a_creature(&mut t, lion);
    assert!(!t.obj(lion).chars.is(CardType::Creature));
    assert!(monstrous(&t, lion));
    assert!(t.obj(lion).chars.has_keyword(KeywordKind::Hexproof));
    // Its monstrosity ability again does nothing: no second counter.
    t.activate(P0, lion, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(plus1(&t, lion), 1);
}

#[test]
fn alpha_deathclaw_stays_monstrous_without_its_abilities() {
    cr!("701.37b", "603.2");
    ruling!(
        "Alpha Deathclaw",
        "Monstrous isn’t an ability that a creature has. It’s just something true about that creature. If the creature stops being a creature or loses its abilities, it will continue to be monstrous."
    );
    supported("Alpha Deathclaw");
    // Alpha Deathclaw: "When this creature enters or becomes monstrous, destroy target
    // permanent." "{5}{B}{G}: Monstrosity 4."
    let mut t = TestGame::new(2);
    let first = t.battlefield(P1, "Grizzly Bears");
    let second = t.battlefield(P1, "Hill Giant");
    let third = t.battlefield(P1, "Colossal Dreadmaw");
    t.answer_targets(P0, &[Entity::Object(first)]);
    let claw = crate::r_s05_common::enter(&mut t, P0, "Alpha Deathclaw");
    t.resolve_all();
    assert!(!t.on_battlefield(first));
    t.lands(P0, "Swamp", 8);
    t.lands(P0, "Forest", 12);
    t.activate(P0, claw, 0, &[]).unwrap();
    t.answer_targets(P0, &[Entity::Object(second)]);
    t.resolve_all();
    assert!(monstrous(&t, claw));
    assert!(!t.on_battlefield(second));
    assert_eq!(plus1(&t, claw), 4);
    // It loses its abilities: still monstrous.
    let humility = t.battlefield(P1, "Humility");
    t.g.recompute();
    assert!(t.obj(claw).chars.abilities.is_empty());
    assert!(monstrous(&t, claw));
    destroy(&mut t, humility);
    // It stops being a creature: still monstrous.
    stop_being_a_creature(&mut t, claw);
    assert!(!t.obj(claw).chars.is(CardType::Creature));
    assert!(monstrous(&t, claw));
    // Monstrosity again: nothing happens, so its "becomes monstrous" ability doesn't
    // trigger.
    t.activate(P0, claw, 0, &[]).unwrap();
    t.answer_targets(P0, &[Entity::Object(third)]);
    t.resolve_all();
    assert_eq!(plus1(&t, claw), 4);
    assert!(t.on_battlefield(third));
    // Its two triggers this turn: entering, and becoming monstrous the first time.
    assert_eq!(triggered_from(&t, claw), 2);
}

#[test]
fn an_already_monstrous_creature_cant_become_monstrous_again() {
    cr!("701.37a");
    ruling!(
        "Vitality Hunter",
        "Once a creature becomes monstrous, it can’t become monstrous again. If the creature is already monstrous when the monstrosity ability resolves, nothing happens."
    );
    supported("Vitality Hunter");
    // Vitality Hunter: "{X}{W}{W}: Monstrosity X." "When this creature becomes monstrous,
    // put a lifelink counter on each of up to X target creatures." Activated for X=3, then
    // for X=1 in response.
    let mut t = TestGame::new(2);
    let hunter = t.battlefield(P0, "Vitality Hunter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 9);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.activate(P0, hunter, 0, &[]).unwrap();
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, hunter, 0, &[]).unwrap();
    // X=1 resolves first: one counter, monstrous, and its trigger.
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(monstrous(&t, hunter));
    assert_eq!(plus1(&t, hunter), 1);
    assert_eq!(triggered_from(&t, hunter), 1);
    assert_eq!(t.counters(bears, "lifelink"), 1);
}

#[test]
fn becomes_monstrous_doesnt_trigger_if_the_creature_left_before_monstrosity_resolved() {
    cr!("701.37a", "608.2b");
    ruling!(
        "Kalemne's Captain",
        "An ability that triggers when a creature becomes monstrous won’t trigger if that creature isn’t on the battlefield when its monstrosity ability resolves."
    );
    ruling!(
        "Alpha Deathclaw",
        "An ability that triggers when a creature becomes monstrous won’t trigger if that creature isn’t on the battlefield when its monstrosity ability resolves."
    );
    supported("Kalemne's Captain");
    // Kalemne's Captain: "{5}{W}{W}: Monstrosity 3." "When this creature becomes
    // monstrous, exile all artifacts and enchantments."
    let mut t = TestGame::new(2);
    let captain = t.battlefield(P0, "Kalemne's Captain");
    let stone = t.battlefield(P1, "Millstone");
    t.lands(P0, "Plains", 7);
    t.activate(P0, captain, 0, &[]).unwrap();
    destroy(&mut t, captain);
    t.resolve_all();
    assert!(t.on_battlefield(stone));
    assert_eq!(triggered_from(&t, captain), 0);
    // The same for Alpha Deathclaw.
    let mut t = TestGame::new(2);
    let claw = t.battlefield(P0, "Alpha Deathclaw");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 6);
    t.activate(P0, claw, 0, &[]).unwrap();
    destroy(&mut t, claw);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(triggered_from(&t, claw), 0);
}
