//! Rulings batch P214 — exert (CR 701.43): "You may exert [this creature] as it attacks"
//! is an optional choice made as attackers are declared (CR 508.1g, 701.43d); an exerted
//! permanent won't untap during its controller's next untap step.

use crate::r_p214_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s05_common::*;
use crate::r_s07_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::game::Game;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P1 casts Decision Paralysis ("Tap up to two target creatures. Those creatures don't
/// untap during their controller's next untap step.") on `target`.
fn paralyze(t: &mut TestGame, target: ObjectId) {
    t.lands(P1, "Island", 4);
    let dp = t.hand(P1, "Decision Paralysis");
    t.answer_targets(P1, &[Entity::Object(target)]);
    t.cast(P1, dp).go();
    t.resolve_all();
}

/// Puts `name` onto the battlefield under P0's control attacking P1 (during an attack by
/// Grizzly Bears), and finishes combat. Returns the new creature.
fn put_onto_battlefield_attacking(t: &mut TestGame, name: &str) -> ObjectId {
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(t, &[(bears, Entity::Player(P1))]);
    let card = t.exile(P0, name);
    let new = t
        .g
        .move_object_ev(mtg_engine::replacement::MoveEv {
            obj: card,
            to: Zone::Battlefield,
            pos: mtg_engine::ability::LibraryPosition::Top,
            cause: mtg_engine::events::MoveCause::Effect,
            by: Some(P0),
            etb: mtg_engine::replacement::EtbInfo {
                controller: Some(P0),
                attacking: Some(Entity::Player(P1)),
                ..Default::default()
            },
            source: None,
        })
        .unwrap();
    t.g.flush_events();
    t.settle();
    assert!(t.g.is_attacking(new));
    block_and_finish(t, P1, &[]);
    new
}

#[test]
fn a_creature_put_onto_the_battlefield_by_champion_of_rhonas_doesnt_see_the_exert() {
    cr!("701.43d", "508.1g", "603.2");
    ruling!(
        "Champion of Rhonas",
        "If the creature put onto the battlefield has any abilities that trigger when creatures attack or when you exert creatures, those abilities won’t trigger."
    );
    supported("Champion of Rhonas");
    supported("Trueheart Twins");
    // Champion of Rhonas: "You may exert this creature as it attacks. When you do, you may
    // put a creature card from your hand onto the battlefield." Trueheart Twins (4/4):
    // "Whenever you exert a creature, creatures you control get +1/+0 until end of turn."
    let mut t = TestGame::new(2);
    let champ = t.battlefield(P0, "Champion of Rhonas");
    let twins = t.hand(P0, "Trueheart Twins");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(twins)]);
    t.answer_yes(P0, true);
    t.attack(&[(champ, Entity::Player(P1))], &[]);
    assert!(exerted(&t, champ));
    let twins = t.g.current(twins);
    assert!(t.on_battlefield(twins));
    // The Twins' "whenever you exert" didn't trigger: no +1/+0.
    assert_eq!(t.pt(twins), (4, 4));
    assert_eq!(t.pt(champ), (3, 3));
    assert!(!t.g.is_attacking(twins));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_creature_returned_by_devoted_crop_mate_doesnt_see_the_exert() {
    cr!("701.43d", "508.1g", "603.2");
    ruling!(
        "Devoted Crop-Mate",
        "If the creature returned to the battlefield has any abilities that trigger when creatures attack or when you exert creatures, those abilities won't trigger."
    );
    supported("Devoted Crop-Mate");
    supported("Battlefield Scavenger");
    // Battlefield Scavenger (mana value 2): "Whenever you exert a creature, you may discard
    // a card. If you do, draw a card."
    // Control: a Battlefield Scavenger already on the battlefield sees the exert.
    let mut t = TestGame::new(2);
    let mate = t.battlefield(P0, "Devoted Crop-Mate");
    t.battlefield(P0, "Battlefield Scavenger");
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(mate, Entity::Player(P1))]);
    assert!(exerted(&t, mate));
    assert_eq!(triggers_on_stack(&t, "discard a card"), 1);
    // Returned by the Crop-Mate's exert trigger: it doesn't.
    let mut t = TestGame::new(2);
    let mate = t.battlefield(P0, "Devoted Crop-Mate");
    let scav = t.graveyard(P0, "Battlefield Scavenger");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(scav)]);
    attack_with(&mut t, &[(mate, Entity::Player(P1))]);
    assert!(exerted(&t, mate));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    let scav = t.g.current(scav);
    assert!(t.on_battlefield(scav));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(triggers_on_stack(&t, "discard a card"), 0);
}

#[test]
fn two_exerted_ahn_crop_champions_untap_each_other() {
    cr!("701.43d", "508.1g", "603.2");
    ruling!(
        "Ahn-Crop Champion",
        "If you attack with two Ahn-Crop Champions and exert both, each will untap the other."
    );
    supported("Ahn-Crop Champion");
    // Ahn-Crop Champion: "You may exert this creature as it attacks. When you do, untap
    // all other creatures you control."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ahn-Crop Champion");
    let b = t.battlefield(P0, "Ahn-Crop Champion");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(a, Entity::Player(P1)), (b, Entity::Player(P1))]);
    assert!(exerted(&t, a) && exerted(&t, b));
    t.resolve_all();
    assert!(!tapped(&t, a));
    assert!(!tapped(&t, b));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 12);
}

/// P0 casts Act of Treason on P1's `name`, attacks with it (exerting it), and the game
/// advances to P1's upkeep. Returns the creature.
fn steal_and_exert(t: &mut TestGame, name: &str) -> ObjectId {
    let c = t.battlefield(P1, name);
    t.lands(P0, "Mountain", 3);
    let spell = t.hand(P0, "Act of Treason");
    t.answer_targets(P0, &[Entity::Object(c)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert_eq!(t.obj_now(c).controller, P0);
    t.answer_yes(P0, true);
    t.attack(&[(c, Entity::Player(P1))], &[]);
    assert!(exerted(t, c) && tapped(t, c));
    t.advance_to(P1, Step::Upkeep);
    c
}

#[test]
fn a_stolen_clockwork_droid_you_exert_untaps_during_its_owners_untap_step() {
    cr!("701.43a", "502.3");
    ruling!(
        "Clockwork Droid",
        "If you gain control of another player's creature until end of turn and exert it, and then that player regains control of it, it will untap during that player's untap step."
    );
    supported("Clockwork Droid");
    supported("Act of Treason");
    let mut t = TestGame::new(2);
    let c = steal_and_exert(&mut t, "Clockwork Droid");
    assert_eq!(t.obj_now(c).controller, P1);
    assert!(!tapped(&t, c));
}

#[test]
fn changing_the_blockers_power_doesnt_unblock_rhonass_stalwart() {
    cr!("509.1b", "509.1h", "506.4");
    ruling!(
        "Rhonas's Stalwart",
        "Once a creature with power 3 or greater has blocked an exerted Rhonas’s Stalwart, changing the power of the blocking creature won’t cause Rhonas’s Stalwart to become unblocked."
    );
    supported("Rhonas's Stalwart");
    // Rhonas's Stalwart: "You may exert this creature as it attacks. When you do, it gets
    // +1/+1 until end of turn and can't be blocked by creatures with power 2 or less this
    // turn." Hill Giant (3/3) blocks it; then the Giant gets -2/-0.
    let mut t = TestGame::new(2);
    let stalwart = t.battlefield(P0, "Rhonas's Stalwart");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(stalwart, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(stalwart), (3, 3));
    // A 2-power creature couldn't block it.
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!t.g.can_block(bears, stalwart));
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![(giant, stalwart)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(t.g.combat.as_ref().unwrap().is_blocked(stalwart));
    pump(&mut t, giant, -2, 0);
    assert_eq!(t.pt(giant), (1, 3));
    assert!(t.g.combat.as_ref().unwrap().is_blocked(stalwart));
    t.advance_to(P0, Step::EndOfCombat);
    // Still blocked: no damage to P1; the Giant was dealt 3 damage and died.
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(giant));
}

/// Glory-Bound-Initiate-style check: exert `name` as it's declared as an attacker; the
/// exert trigger resolves before blockers are declared; a creature put onto the
/// battlefield attacking can't be exerted. `look` reports the state seen as P1 declares
/// blockers, and `expect` is what should be seen.
fn exert_as_declared<T: PartialEq + std::fmt::Debug + Send + 'static>(
    name: &str,
    setup: fn(&mut TestGame, ObjectId),
    look: fn(&Game) -> T,
    expect: T,
) {
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, name);
    t.battlefield(P1, "Grizzly Bears");
    let is_blocks = |d: &Decision| matches!(d, Decision::DeclareBlockers { .. });
    let seen = watch(&mut t, P1, is_blocks, look);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    setup(&mut t, c);
    t.attack(&[(c, Entity::Player(P1))], &[]);
    assert!(exerted(&t, c));
    assert_eq!(*seen.lock().unwrap(), vec![expect]);
    // Asked once, as it was declared: not again later in combat.
    assert_eq!(count_asked(&t, from, is_exert_question), 1);
    cant_exert_later(name);
}

/// Declining to exert `name` as it's declared, it can't be exerted later in combat; put
/// onto the battlefield attacking, it can't be exerted.
fn cant_exert_later(name: &str) {
    // Declined as it's declared: it can't be exerted later in combat.
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, name);
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.attack(&[(c, Entity::Player(P1))], &[]);
    assert!(!exerted(&t, c));
    assert_eq!(count_asked(&t, from, is_exert_question), 1);
    // Put onto the battlefield attacking: it can't be exerted.
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    let new = put_onto_battlefield_attacking(&mut t, name);
    assert_eq!(count_asked(&t, from, is_exert_question), 0);
    assert!(!exerted(&t, new));
}

fn exiled_count(g: &Game) -> usize {
    g.exile.len()
}

#[test]
fn anep_is_exerted_as_it_is_declared_as_an_attacker() {
    cr!("508.1g", "701.43d", "509.1");
    ruling!(
        "Anep, Vizier of Hazoret",
        "You can exert Anep as you declare it as an attacking creature. You can't do so later in combat, and creatures put onto the battlefield attacking can't be exerted. Any abilities that trigger on exerting an attacking creature will resolve before blockers are declared."
    );
    supported("Anep, Vizier of Hazoret");
    // Anep: "When you do, exile the top two cards of your library." Two cards were exiled
    // before blockers were declared.
    exert_as_declared("Anep, Vizier of Hazoret", |_, _| {}, exiled_count, 2);
}

#[test]
fn hydra_trainer_is_exerted_as_it_is_declared_as_an_attacker() {
    cr!("508.1g", "701.43d", "509.1");
    ruling!(
        "Hydra Trainer",
        "You can exert Hydra Trainer as you declare it as an attacking creature. You can't do so later in combat, and creatures put onto the battlefield attacking can't be exerted. Any abilities that trigger on exerting an attacking creature will resolve before blockers are declared."
    );
    // Hydra Trainer (1/1) with a +1/+1 counter targets itself: +1/+1 (X = 1) before
    // blockers.
    fn setup(t: &mut TestGame, c: ObjectId) {
        t.g.add_counters(Entity::Object(c), mtg_engine::types::counters::PLUS1, 1, None);
        t.answer_targets(P0, &[Entity::Object(c)]);
    }
    fn trainer_pt(g: &Game) -> (i32, i32) {
        let id = g.find_in_zone(Zone::Battlefield, "Hydra Trainer")[0];
        (g.obj(id).power(), g.obj(id).toughness())
    }
    exert_as_declared("Hydra Trainer", setup, trainer_pt, (3, 3));
}

#[test]
fn sandstorm_crasher_is_exerted_as_it_is_declared_as_an_attacker() {
    cr!("508.1g", "701.43d", "509.1");
    ruling!(
        "Sandstorm Crasher",
        "You can exert Sandstorm Crasher as you declare it as an attacking creature. You can't do so later in combat, and creatures put onto the battlefield attacking can't be exerted. Any abilities that trigger on exerting an attacking creature will resolve before blockers are declared."
    );
    supported("Sandstorm Crasher");
    // Sandstorm Crasher: "When you do, create a tapped and attacking token that's a copy of
    // target creature you control." It copies itself before blockers.
    fn setup(t: &mut TestGame, c: ObjectId) {
        t.answer_targets(P0, &[Entity::Object(c)]);
    }
    fn crashers(g: &Game) -> usize {
        g.find_in_zone(Zone::Battlefield, "Sandstorm Crasher").len()
    }
    exert_as_declared("Sandstorm Crasher", setup, crashers, 2);
}

#[test]
fn clockwork_droid_is_exerted_only_as_it_attacks() {
    cr!("508.1g", "701.43d", "509.1");
    ruling!(
        "Clockwork Droid",
        "You may exert Clockwork Droid only as you attack with it. You can't do so later in combat, and creatures put onto the battlefield attacking can't be exerted. The ability that triggers on exerting it will resolve before blockers are declared."
    );
    // Clockwork Droid: "When you do, it can't be blocked this turn and you scry 1." The
    // trigger resolves in the declare attackers step: the Droid is unblockable before
    // blockers are declared.
    let mut t = TestGame::new(2);
    let droid = t.battlefield(P0, "Clockwork Droid");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(droid, Entity::Player(P1))]);
    assert!(exerted(&t, droid));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(!t.g.can_block(bears, droid));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(count_asked(&t, from, is_exert_question), 1);
    assert_eq!(t.life(P1), 17);
    cant_exert_later("Clockwork Droid");
}

#[test]
fn tap_and_freeze_doesnt_exert_clockwork_droid() {
    cr!("701.43a");
    ruling!(
        "Clockwork Droid",
        "You can't exert a creature unless an effect allows you to do so. Similar effects that \"tap and freeze\" a creature don't exert that creature."
    );
    supported("Decision Paralysis");
    let mut t = TestGame::new(2);
    let droid = t.battlefield(P0, "Clockwork Droid");
    paralyze(&mut t, droid);
    assert!(tapped(&t, droid));
    assert!(!exerted(&t, droid));
    // Its exert trigger ("it can't be blocked this turn and you scry 1") didn't trigger.
    assert!(t.g.stack.is_empty());
    // Nothing else lets P0 exert it: not asked outside of declaring attackers.
    let from = t.asked().len();
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(count_asked(&t, from, is_exert_question), 0);
}

#[test]
fn hydra_trainers_target_is_chosen_as_the_reflexive_trigger_goes_on_the_stack() {
    cr!("603.12", "701.43d", "117.3c");
    ruling!(
        "Hydra Trainer",
        "You don't choose a target for Hydra Trainer's first ability at the time it triggers. Rather, a second \"reflexive\" ability triggers when you exert Hydra Trainer this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    // When P0 chooses the target, the Trainer is already exerted (and attacking).
    fn is_targets(d: &Decision) -> bool {
        matches!(d, Decision::ChooseTargets { .. })
    }
    fn trainer_exerted(g: &Game) -> bool {
        let id = g.find_in_zone(Zone::Battlefield, "Hydra Trainer")[0];
        g.obj(id).exerted && g.is_attacking(id)
    }
    let mut t = TestGame::new(2);
    let trainer = t.battlefield(P0, "Hydra Trainer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(trainer), mtg_engine::types::counters::PLUS1, 2, None);
    let seen = watch(&mut t, P0, is_targets, trainer_exerted);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_with(&mut t, &[(trainer, Entity::Player(P1))]);
    assert_eq!(*seen.lock().unwrap(), vec![true]);
    // The reflexive trigger is on the stack, and P1 may respond: P1 kills the target.
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.pt(trainer), (3, 3));
}

#[test]
fn anep_follows_the_timing_rules_for_the_exiled_cards() {
    cr!("305.2", "305.3", "601.3");
    ruling!(
        "Anep, Vizier of Hazoret",
        "You pay all costs and follow all timing rules for cards played with the permission granted by the ability that triggers when you exert Anep. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    // The top two cards: Forest and Hill Giant ({3}{R}).
    let mut t = TestGame::new(2);
    let anep = t.battlefield(P0, "Anep, Vizier of Hazoret");
    let cards = stack_library(&mut t, P0, &["Forest", "Hill Giant"]);
    t.answer_yes(P0, true);
    t.attack(&[(anep, Entity::Player(P1))], &[]);
    let forest = t.g.current(cards[0]);
    let giant = t.g.current(cards[1]);
    assert_eq!(t.zone(forest), Zone::Exile);
    assert_eq!(t.zone(giant), Zone::Exile);
    // In combat: the land can't be played, and the creature spell can't be cast.
    assert!(!can_play_land(&mut t, P0, forest));
    t.lands(P0, "Mountain", 4);
    assert!(!can_cast(&mut t, P0, giant, mtg_engine::object::CastMethod::Normal));
    // In the postcombat main phase with the stack empty, both can be played.
    t.advance_to(P0, Step::PostcombatMain);
    assert!(can_play_land(&mut t, P0, forest));
    assert!(can_cast(&mut t, P0, giant, mtg_engine::object::CastMethod::Normal));
    // With something on the stack, the land can't be played.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let _ = bears;
    t.lands(P0, "Forest", 2);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(anep).go();
    assert_eq!(t.stack_len(), 1);
    assert!(!can_play_land(&mut t, P0, forest));
    // And the costs are paid: casting the Giant needs {3}{R}.
    t.resolve_all();
    let mana_before = tapped_lands(&t, P0);
    t.cast(P0, giant).go();
    assert_eq!(tapped_lands(&t, P0), mana_before + 4);
}
