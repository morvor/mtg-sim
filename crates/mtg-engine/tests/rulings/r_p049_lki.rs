//! Rulings batch P049 — abilities whose source has left the battlefield: last known
//! information (CR 608.2h, 113.7a) for Craig Boone's quest counters, Fell Beast of
//! Mordor's +1/+1 counters and Sorin, Vengeful Bloodlord's lifelink; values determined on
//! resolution (Sanctum of Stone Fangs, Eriette of the Charmed Apple); "for as long as you
//! control" effects whose duration has already ended (Nihiloor, CR 611.2b); and a new
//! object's link to cards exiled by the old one (Lobelia, CR 400.7).

use crate::r_p049_common::*;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Action;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether `p` could cast `card` by any method now.
fn can_cast_any(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let card = t.g.current(card);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, Action::Cast { card: c, .. } if *c == card))
}

#[test]
fn craig_boones_reflexive_trigger_uses_its_last_known_quest_counters() {
    cr!("608.2h", "603.12", "113.7a");
    ruling!(
        "Craig Boone, Novac Guard",
        "If Craig Boone is no longer on the battlefield when the reflexive triggered ability resolves, use the number of quest counters that were on it when it was last on the battlefield to determine how much damage it deals."
    );
    supported("Craig Boone, Novac Guard");
    let mut t = TestGame::new(2);
    let boone = t.battlefield(P0, "Craig Boone, Novac Guard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let target = t.battlefield(P1, "Grizzly Bears");
    // Attacking with two creatures: two quest counters, then the reflexive trigger
    // targets P1's Bears; P1 doesn't take the damage instead.
    t.answer_targets(P0, &[obj(target)]);
    t.answer_yes(P1, false);
    attack_with(
        &mut t,
        &[(boone, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.resolve();
    assert_eq!(t.counters(boone, "quest"), 2);
    assert_eq!(triggers_on_stack(&t, "reflexive trigger"), 1);
    // Boone leaves before it resolves: 2 damage all the same.
    exile_now(&mut t, boone);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);
}

#[test]
fn fell_beast_of_mordor_uses_its_last_known_counters_once_it_has_left() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Fell Beast of Mordor",
        "If Fell Beast of Mordor isn't on the battlefield as its ability resolves, use the number of +1/+1 counters that were on it when it was last on the battlefield to determine the value of X."
    );
    supported("Fell Beast of Mordor");
    let mut t = TestGame::new(2);
    let beast = t.battlefield(P0, "Fell Beast of Mordor");
    put_counters(&mut t, beast, counters::PLUS1, 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(beast, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "loses X life"), 1);
    exile_now(&mut t, beast);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (22, 18));
}

#[test]
fn sorins_damage_has_lifelink_after_he_has_left_the_battlefield() {
    cr!("702.15c", "608.2h", "113.7a");
    ruling!(
        "Sorin, Vengeful Bloodlord",
        "If Sorin leaves the battlefield after you've activated his first loyalty ability during your turn but before it resolves, you'll still gain 1 life as that ability deals damage."
    );
    supported("Sorin, Vengeful Bloodlord");
    let mut t = TestGame::new(2);
    let sorin = t.battlefield(P0, "Sorin, Vengeful Bloodlord");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, sorin, "1 damage").unwrap();
    destroy(&mut t, sorin);
    assert!(t.in_graveyard(P0, "Sorin, Vengeful Bloodlord"));
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn sanctum_of_stone_fangs_counts_shrines_as_its_ability_resolves() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Sanctum of Stone Fangs",
        "If Sanctum of Stone Fangs leaves the battlefield while its ability is on the stack, you may control no Shrines as the ability resolves. In that case, no player gains or loses any life."
    );
    supported("Sanctum of Stone Fangs");
    let mut t = TestGame::new(2);
    let sanctum = t.battlefield(P0, "Sanctum of Stone Fangs");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "loses X life"), 1);
    bounce(&mut t, sanctum);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // Left where it is, X is 1.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sanctum of Stone Fangs");
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn eriette_counts_the_auras_she_controls_including_roles_on_opponents_creatures() {
    cr!("111.2", "608.2h", "303.4");
    ruling!(
        "Eriette of the Charmed Apple",
        "By default, the controller of a Role token is the player who created it, even if that Role token is attached to a creature they don't control."
    );
    ruling!(
        "Eriette of the Charmed Apple",
        "Count the number of Auras you control as the ability resolves to determine the value of X."
    );
    supported("Eriette of the Charmed Apple");
    supported("Asinine Antics");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eriette of the Charmed Apple");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    // Asinine Antics: a Cursed Role attached to each creature P1 controls, created (and
    // controlled) by P0.
    cast_new(&mut t, P0, "Asinine Antics", &[]);
    t.resolve_all();
    let roles: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.is_token() && o.chars.has_subtype("Role"))
            .map(|o| o.id)
            .collect();
    assert_eq!(roles.len(), 2);
    assert!(roles.iter().all(|r| t.obj(*r).controller == P0));
    assert!(roles
        .iter()
        .all(|r| matches!(t.obj(*r).attached_to, Some(Entity::Object(x)) if x == a || x == b)));
    assert_eq!(t.pt(a), (1, 1));
    // P0's end step: X is counted on resolution; one Role is destroyed in response.
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "loses X life"), 1);
    destroy(&mut t, roles[0]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
}

#[test]
fn nihiloor_leaving_before_its_abilities_resolve_gains_no_control() {
    cr!("611.2b", "603.12", "113.7a");
    ruling!(
        "Nihiloor",
        "If Nihiloor leaves the battlefield before its enters-the-battlefield ability resolves or before its reflexive triggered abilities resolve, then you won't gain control of any creatures."
    );
    supported("Nihiloor");
    // Before the enters ability resolves.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(bears)]);
    let n = t.enter(P0, "Nihiloor");
    t.settle();
    exile_now(&mut t, n);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
    // Before the reflexive ability resolves: "for as long as you control Nihiloor" has
    // already ended, so the effect does nothing.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(bears)]);
    let n = t.enter(P0, "Nihiloor");
    t.settle();
    t.resolve();
    assert!(tapped(&t, giant));
    assert_eq!(triggers_on_stack(&t, "reflexive trigger"), 1);
    exile_now(&mut t, n);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn nihiloor_checks_both_powers_as_the_ability_is_put_on_the_stack_and_as_it_resolves() {
    cr!("608.2b", "603.3d", "603.12");
    ruling!(
        "Nihiloor",
        "Both the tapped creature's power and the target creature's power are checked both as you put the ability on the stack and as the ability resolves in order to make sure the target is legal."
    );
    supported("Nihiloor");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let big = t.battlefield(P1, "Serra Angel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(giant)]);
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Nihiloor");
    t.settle();
    t.resolve();
    // Only creatures with power 3 or less were offered.
    let offered = target_candidates(&t, P0, from);
    assert_eq!(offered.len(), 1);
    assert!(offered[0].contains(&obj(bears)) && !offered[0].contains(&obj(big)));
    // The Bears grow past the Giant's power in response: an illegal target.
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::ModifyPT(Value::c(3), Value::c(3))],
    );
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
    // The tapped creature's power shrinking below the target's also makes it illegal.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Nihiloor");
    t.settle();
    t.resolve();
    modify_until_eot(
        &mut t,
        giant,
        vec![Modification::ModifyPT(Value::c(-2), Value::c(0))],
    );
    assert_eq!(t.pt(giant).0, 1);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P1);
}

#[test]
fn nihiloor_gains_control_even_if_the_tapped_creature_is_untapped() {
    cr!("603.12", "608.2b");
    ruling!(
        "Nihiloor",
        "If a creature you tapped for Nihiloor's ability is somehow no longer tapped as the reflexive triggered ability resolves, that ability will still resolve and you will still gain control of the target as long as it has less power than the creature you tapped."
    );
    supported("Nihiloor");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(giant)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.enter(P0, "Nihiloor");
    t.settle();
    t.resolve();
    assert!(tapped(&t, giant));
    untap(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn a_new_lobelia_cant_play_cards_exiled_by_the_old_one() {
    cr!("400.7", "608.2h");
    ruling!(
        "Lobelia, Defender of Bag End",
        "You can't play cards exiled by the original Lobelia, Defender of Bag End with the second ability of the new Lobelia, Defender of Bag End."
    );
    supported("Lobelia, Defender of Bag End");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sol Ring");
    t.battlefield(P0, "Sol Ring");
    let first = t.library_top(P1, "Grizzly Bears");
    let lobelia = t.enter(P0, "Lobelia, Defender of Bag End");
    t.resolve_all();
    // Exiled face down: it has no name there.
    assert_eq!(t.zone(first), mtg_engine::object::Zone::Exile);
    // The original Lobelia lets P0 cast it (mode 1: play a card exiled with Lobelia
    // without paying its mana cost, until end of turn).
    t.g.objects[lobelia.0 as usize].summoning_sick = false;
    t.answer(P0, DecisionKind::Modes, mtg_engine::decision::Answer::Indices(vec![0]));
    activate_containing(&mut t, P0, lobelia, "Sacrifice an artifact").unwrap();
    t.resolve_all();
    assert!(can_cast_any(&mut t, P0, first));
    // Lobelia leaves and comes back: a new object. Its ability doesn't cover the card the
    // old one exiled (a new turn, so the old permission has expired too).
    bounce(&mut t, lobelia);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    let second = t.library_top(P1, "Hill Giant");
    let lobelia2 = t.enter(P0, "Lobelia, Defender of Bag End");
    t.resolve_all();
    assert_eq!(t.zone(second), mtg_engine::object::Zone::Exile);
    t.g.objects[lobelia2.0 as usize].summoning_sick = false;
    t.answer(P0, DecisionKind::Modes, mtg_engine::decision::Answer::Indices(vec![0]));
    activate_containing(&mut t, P0, lobelia2, "Sacrifice an artifact").unwrap();
    t.resolve_all();
    assert!(can_cast_any(&mut t, P0, second));
    assert!(!can_cast_any(&mut t, P0, first));
}
