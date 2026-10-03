//! Rulings batch P203 — convert (CR 701.28): converting attacking creatures between the
//! first-strike and regular combat damage steps, modular and converting, and abilities
//! that try to convert a permanent that already converted.

use crate::r_s01_common::*;
use crate::r_s03_common::run_effect;
use crate::r_s06_common::attach_new;
use mtg_engine::ability::{Effect, Filter, KeywordAction, PlayerRef, Sel, Value};
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::FaceState;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Converts `id` (an effect with no source).
fn convert(t: &mut TestGame, id: ObjectId) {
    run_effect(
        t,
        None,
        P0,
        Effect::KeywordAction {
            action: KeywordAction::Convert,
            who: PlayerRef::You,
            what: Sel::All(Filter::Objects(vec![id])),
            n: Value::c(1),
        },
        &[],
    );
}

/// The Transformers card `full` on P0's battlefield with its back face up.
fn back_face_up(t: &mut TestGame, full: &str) -> ObjectId {
    supported(full);
    let id = t.battlefield(P0, full);
    convert(t, id);
    assert_eq!(t.obj_now(id).face, FaceState::Back);
    id
}

#[test]
fn blitzwing_converted_by_first_strike_damage_keeps_attacking_and_deals_regular_damage() {
    cr!("701.28a", "712.18", "510.4", "702.4b");
    ruling!(
        "Blitzwing, Cruel Tormentor // Blitzwing, Adaptive Assailant",
        "After Blitzwing, Adaptive Assailant converts into Blitzwing, Cruel Tormentor, it's still an attacking creature in combat. It will still have any gained abilities, including the ability it gained at the beginning of combat. Notably, if it has somehow gained double strike, it will convert before dealing combat damage in the regular combat damage step."
    );
    supported("Fireshrieker");
    // Blitzwing, Adaptive Assailant (a 3/5 Vehicle with living metal; "At the beginning of
    // combat on your turn, choose flying or indestructible at random. Blitzwing gains that
    // ability until end of turn. Whenever Blitzwing deals combat damage to a player,
    // convert it.") equipped with Fireshrieker (double strike). It deals 3 first-strike
    // damage, converts into Blitzwing, Cruel Tormentor (6/5), and deals 6 more.
    let mut t = TestGame::new(2);
    let b = back_face_up(&mut t, "Blitzwing, Cruel Tormentor // Blitzwing, Adaptive Assailant");
    attach_new(&mut t, P0, "Fireshrieker", b);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let gained = |t: &TestGame| {
        let o = t.obj_now(b);
        o.has_keyword(KeywordKind::Flying) || o.has_keyword(KeywordKind::Indestructible)
    };
    assert!(gained(&t));
    t.attack(&[(b, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20 - 3 - 6);
    let o = t.obj_now(b);
    assert_eq!(o.face, FaceState::Front);
    assert_eq!(o.chars.name, "Blitzwing, Cruel Tormentor");
    assert!(gained(&t));
    assert!(o.has_keyword(KeywordKind::DoubleStrike));
}

#[test]
fn cyclonus_converted_by_first_strike_damage_can_convert_again() {
    cr!("701.28a", "712.18", "510.4", "702.4b", "701.50a");
    ruling!(
        "Cyclonus, the Saboteur // Cyclonus, Cybertronian Fighter",
        "After Cyclonus, Cybertronian Fighter converts into Cyclonus, the Saboteur, it's still an attacking creature in combat, and will still have any gained abilities. Notably, if it has somehow gained double strike, it will convert before dealing combat damage in the regular combat damage step. This may cause it to convert again when it deals combat damage to a player a second time."
    );
    supported("Fireshrieker");
    // Cyclonus, Cybertronian Fighter (5/5 flying, "Whenever Cyclonus deals combat damage
    // to a player, convert it. If you do, there is an additional beginning phase after
    // this phase.") with three +1/+1 counters and Fireshrieker: 8 first-strike damage, it
    // converts into Cyclonus, the Saboteur (2/5, +3 = 5/8; "Whenever Cyclonus deals combat
    // damage to a player, it connives. Then if Cyclonus's power is 5 or greater, convert
    // it."), which deals 5 regular damage, connives and converts again.
    let mut t = TestGame::new(2);
    let c = back_face_up(
        &mut t,
        "Cyclonus, the Saboteur // Cyclonus, Cybertronian Fighter",
    );
    t.g.add_counters(Entity::Object(c), counters::PLUS1, 3, None);
    attach_new(&mut t, P0, "Fireshrieker", c);
    t.hand(P0, "Plains");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(c, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20 - 8 - 5);
    let o = t.obj_now(c);
    assert_eq!(o.face, FaceState::Back);
    assert_eq!(o.chars.name, "Cyclonus, Cybertronian Fighter");
    assert!(o.has_keyword(KeywordKind::DoubleStrike));
}

#[test]
fn converting_into_blaster_morale_booster_gives_no_modular_counters() {
    cr!("701.28a", "702.43a", "712.18");
    ruling!(
        "Blaster, Combat DJ // Blaster, Morale Booster",
        "Converting into Blaster, Morale Booster from Blaster, Combat DJ does not cause it to get counters from its modular ability."
    );
    supported("Blaster, Combat DJ // Blaster, Morale Booster");
    // Blaster, Combat DJ: "Whenever you put one or more +1/+1 counters on Blaster, convert
    // it." Blaster, Morale Booster has modular 3.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Blaster, Combat DJ // Blaster, Morale Booster");
    t.g.add_counters(Entity::Object(b), counters::PLUS1, 1, None);
    t.g.flush_events();
    t.resolve_all();
    let o = t.obj_now(b);
    assert_eq!(o.face, FaceState::Back);
    assert_eq!(o.chars.name, "Blaster, Morale Booster");
    assert_eq!(o.counter(counters::PLUS1), 1);
}

const RATCHET: &str = "Ratchet, Field Medic // Ratchet, Rescue Racer";

#[test]
fn ratchet_converts_only_once_for_simultaneous_life_gains() {
    cr!("701.28e", "603.2c", "510.2", "702.15b");
    ruling!(
        "Ratchet, Field Medic // Ratchet, Rescue Racer",
        "Multiple instances of gaining life that happen at the same time will cause Ratchet, Field Medic's last ability to trigger that many times. However, it can convert only once when that happens. Once it has converted due to that triggered ability, the rest of the abilities will resolve and do nothing."
    );
    supported(RATCHET);
    supported("Vampire Nighthawk");
    // Ratchet, Field Medic (2/4 lifelink: "Whenever you gain life, you may convert
    // Ratchet. When you do, return target artifact card with mana value less than or equal
    // to the amount of life you gained this turn from your graveyard to the battlefield
    // tapped.") and Vampire Nighthawk (lifelink) deal combat damage at the same time: two
    // life gains, two triggers. P0 says yes to both; Ratchet converts once.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, RATCHET);
    let n = t.battlefield(P0, "Vampire Nighthawk");
    let orni = t.graveyard(P0, "Ornithopter");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(orni)]);
    let from = t.asked().len();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(r, Entity::Player(P1)), (n, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P0), 24);
    let convert_asks = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { prompt, .. } if prompt.to_lowercase().contains("convert")))
        .count();
    assert_eq!(convert_asks, 2, "{}", t.dump_log());
    let o = t.obj_now(r);
    assert_eq!(o.face, FaceState::Back);
    assert_eq!(o.chars.name, "Ratchet, Rescue Racer");
    // Only one reflexive trigger: the Ornithopter returned once.
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
}

#[test]
fn ratchets_target_is_chosen_for_a_reflexive_trigger_after_it_converts() {
    cr!("603.12", "701.28a");
    ruling!(
        "Ratchet, Field Medic // Ratchet, Rescue Racer",
        "Ratchet, Field Medic's last ability doesn't require a target. If you convert Ratchet as it resolves, a second ability triggers and you choose a target for it. Players may respond to this new triggered ability as normal."
    );
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, RATCHET);
    let orni = t.graveyard(P0, "Ornithopter");
    let from = t.asked().len();
    t.g.gain_life(P0, 3);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(orni)]);
    t.resolve();
    // Converted; the reflexive trigger is on the stack with its target.
    assert_eq!(t.obj_now(r).face, FaceState::Back);
    assert_eq!(t.stack_len(), 1);
    assert!(t.in_graveyard(P0, "Ornithopter"));
    let from = t.asked().len();
    assert!(t.g.run_until(1_000, |g| g.stack.is_empty()));
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::Priority { .. })));
    let o = t.named_on_battlefield("Ornithopter");
    assert_eq!(o.len(), 1);
    assert!(t.obj(o[0]).tapped);
    // Declining to convert: no reflexive trigger.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, RATCHET);
    t.graveyard(P0, "Ornithopter");
    t.g.gain_life(P0, 3);
    t.g.flush_events();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.obj_now(r).face, FaceState::Front);
    assert!(t.in_graveyard(P0, "Ornithopter"));
}
