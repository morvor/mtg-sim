//! Rulings batch P123 — "target creature can't block this turn" and related abilities of
//! the same cards: the restriction is on blocking at all (CR 509.1b), "up to" targets,
//! targets that became illegal (CR 608.2b), dividing damage as an ability is activated
//! (CR 601.2d), changing a spell's target (CR 115.7), and playing an exiled card (CR
//! 305.2, 601.2).

use crate::r_p123_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 attacks P1 with `attacker` and a Raging Goblin; P0's attack trigger targets P1's
/// Grizzly Bears. Afterwards the Bears can't block anything, not just the attacker whose
/// ability targeted it.
fn attack_trigger_stops_all_blocks(attacker: &str, setup: impl FnOnce(&mut TestGame)) {
    supported(attacker);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, attacker);
    let goblin = t.battlefield(P0, "Raging Goblin");
    let bears = t.battlefield(P1, "Llanowar Elves");
    let other = t.battlefield(P1, "Grizzly Bears");
    setup(&mut t);
    t.answer_targets(P0, &[obj(bears)]);
    to_combat(&mut t, P0);
    attack_with(
        &mut t,
        &[(a, Entity::Player(P1)), (goblin, Entity::Player(P1))],
    );
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P1, &[(bears, a)]), "{attacker}");
    assert!(
        !legal_blocks(&mut t, P1, &[(bears, goblin)]),
        "{attacker}: it can't block another attacker either"
    );
    assert!(legal_blocks(&mut t, P1, &[(other, goblin)]), "{attacker}");
}

#[test]
fn target_creature_cant_block_any_creature() {
    cr!("509.1b", "603.2");
    ruling!(
        "Maulfist Doorbuster",
        "The target creature can't block any creatures this turn. The effect isn't restricted to just whether the creature can block Maulfist Doorbuster."
    );
    ruling!(
        "Goma Fada Vanguard",
        "The target creature is restricted from blocking anything, not just Goma Fada Vanguard."
    );
    ruling!(
        "Grotag Thrasher",
        "The targeted creature is prohibited from blocking any creature this turn, not just Grotag Thrasher."
    );
    attack_trigger_stops_all_blocks("Grotag Thrasher", |_| {});
    // A Warrior: Llanowar Elves (power 1) is a legal target.
    attack_trigger_stops_all_blocks("Goma Fada Vanguard", |_| {});
    attack_trigger_stops_all_blocks("Maulfist Doorbuster", |t| {
        crate::r_p129_common::give_energy(t, P0, 1);
        t.answer_yes(P0, true);
    });
}

#[test]
fn fervent_cathar_target_cant_block_any_creature() {
    cr!("509.1b", "603.2");
    ruling!(
        "Fervent Cathar",
        "The targeted creature can’t block any creatures this turn. The effect doesn’t merely restrict whether it can block Fervent Cathar."
    );
    supported("Fervent Cathar");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let goblin = t.battlefield(P0, "Raging Goblin");
    t.answer_targets(P0, &[obj(bears)]);
    let cathar = t.enter(P0, "Fervent Cathar");
    t.resolve_all();
    to_combat(&mut t, P0);
    attack_with(
        &mut t,
        &[(cathar, Entity::Player(P1)), (goblin, Entity::Player(P1))],
    );
    assert!(!legal_blocks(&mut t, P1, &[(bears, cathar)]));
    assert!(!legal_blocks(&mut t, P1, &[(bears, goblin)]));
}

#[test]
fn merciless_javelineer_cant_block_without_the_counter() {
    cr!("608.2c", "614.1");
    ruling!(
        "Merciless Javelineer",
        "The target creature won’t be able to block even if the -1/-1 counter is removed from it or if the -1/-1 counter can’t be put on it."
    );
    supported("Merciless Javelineer");
    supported("Melira's Keepers");
    let javelin = |t: &mut TestGame, jav: ObjectId, target: ObjectId| {
        t.hand(P0, "Island");
        t.lands(P0, "Wastes", 2);
        t.answer_targets(P0, &[obj(target)]);
        activate_containing(t, P0, jav, "Discard").expect("activated");
        t.resolve_all();
    };
    let mut t = TestGame::new(2);
    let jav = t.battlefield(P0, "Merciless Javelineer");
    // The counter can't be put on it.
    let keepers = t.battlefield(P1, "Melira's Keepers");
    javelin(&mut t, jav, keepers);
    assert_eq!(t.counters(keepers, "-1/-1"), 0);
    assert!(!can_block(&mut t, keepers));
    // The counter is removed.
    let bears = t.battlefield(P1, "Grizzly Bears");
    javelin(&mut t, jav, bears);
    assert_eq!(t.counters(bears, "-1/-1"), 1);
    t.g.remove_counters(obj(bears), "-1/-1", 1);
    t.settle();
    assert_eq!(t.counters(bears, "-1/-1"), 0);
    assert!(!can_block(&mut t, bears));
}

#[test]
fn wrap_in_flames_cant_block_even_if_damage_is_prevented() {
    cr!("608.2c", "615.1");
    ruling!(
        "Wrap in Flames",
        "The “can’t block” effect applies to each of the targeted creatures—and to only those creatures—even if Wrap in Flames deals no damage to one or more of them"
    );
    supported("Wrap in Flames");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Craw Wurm");
    // Healing Salve: prevent the next 3 damage to the Bears.
    t.answer(P1, DecisionKind::Modes, Answer::Indices(vec![1]));
    cast_resolve_at(&mut t, P1, "Healing Salve", &[obj(bears)]);
    t.answer_targets(P0, &[obj(bears), obj(giant)]);
    cast_card(&mut t, P0, "Wrap in Flames");
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 1);
    assert!(!can_block(&mut t, bears) && !can_block(&mut t, giant));
    assert!(can_block(&mut t, other), "not targeted");
}

#[test]
fn renegade_wheelsmith_triggers_when_it_becomes_tapped() {
    cr!("603.2", "602.1");
    ruling!(
        "Renegade Wheelsmith",
        "This is a triggered ability, not an activated ability. It doesn’t allow you to tap Renegade Wheelsmith whenever you want; rather, you need some other way of tapping it"
    );
    supported("Renegade Wheelsmith");
    let mut t = TestGame::new(2);
    let smith = t.battlefield(P0, "Renegade Wheelsmith");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!can_activate(&mut t, P0, smith), "no activated ability");
    t.answer_targets(P0, &[obj(bears)]);
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(smith, Entity::Player(P1))]);
    t.resolve_all();
    assert!(!can_block(&mut t, bears));
}

#[test]
fn arachnus_web_doesnt_stop_triggered_abilities() {
    cr!("602.5", "603.2");
    ruling!("Arachnus Web", "Triggered abilities (starting with \"when,\" \"whenever,\" or \"at\") are unaffected.");
    supported("Arachnus Web");
    let mut t = TestGame::new(2);
    let mate = t.battlefield(P0, "Ajani's Pridemate");
    attach_new(&mut t, P1, "Arachnus Web", obj(mate));
    assert!(!can_block(&mut t, mate));
    gain_life(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(plus1(&t, mate), 1, "its triggered ability still works");
    // Its activated abilities can't be activated.
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    assert!(can_activate(&mut t, P0, pyro));
    attach_new(&mut t, P1, "Arachnus Web", obj(pyro));
    assert!(!can_activate(&mut t, P0, pyro));
}

#[test]
fn huatli_plus_two_with_no_creatures() {
    cr!("606.4", "119.4");
    ruling!(
        "Huatli, Warrior Poet",
        "You can activate Huatli's first ability even if you control no creatures. You simply won't gain any life."
    );
    supported("Huatli, Warrior Poet");
    let mut t = TestGame::new(2);
    let huatli = t.battlefield(P0, "Huatli, Warrior Poet");
    activate_containing(&mut t, P0, huatli, "gain life").expect("activated");
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(huatli, "loyalty"), 5);
}

#[test]
fn huatli_divides_as_activated_and_x_zero_has_no_targets() {
    cr!("601.2d", "606.4", "115.10");
    ruling!(
        "Huatli, Warrior Poet",
        "You divide the damage among the target creatures as you activate Huatli's last ability. Each target must be assigned at least 1 damage. If X is 0, you can't choose any targets."
    );
    let minus_x = |t: &mut TestGame, huatli: ObjectId, x: i64, targets: &[Entity], div: &[i64]| {
        t.answer(P0, DecisionKind::X, Answer::Number(x));
        t.answer_targets(P0, targets);
        t.answer(P0, DecisionKind::Divide, Answer::Numbers(div.to_vec()));
        activate_containing(t, P0, huatli, "divided")
    };
    let mut t = TestGame::new(2);
    let huatli = t.battlefield(P0, "Huatli, Warrior Poet");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let from = t.asked().len();
    let ab = minus_x(&mut t, huatli, 2, &[obj(bears), obj(wurm)], &[1, 1])
        .expect("activated")
        .expect("on the stack");
    // The division was asked while activating, with at least 1 for each target.
    let divide = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::Divide { total, min_each, .. } => Some((*total, *min_each)),
            _ => None,
        })
        .expect("division asked as it's activated");
    assert_eq!(divide, (2, 1));
    assert_eq!(t.g.current(ab), ab);
    t.resolve_all();
    assert_eq!((t.obj_now(bears).damage, t.obj_now(wurm).damage), (1, 1));
    // X = 0: no targets can be chosen.
    let mut t = TestGame::new(2);
    let huatli = t.battlefield(P0, "Huatli, Warrior Poet");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let r = minus_x(&mut t, huatli, 0, &[obj(bears)], &[0]);
    if let Ok(Some(ab)) = r {
        assert!(crate::r_s25_common::targets_of(&t, ab).is_empty());
    }
    t.resolve_all();
    assert!(can_block(&mut t, bears), "the Bears weren't a target");
}

#[test]
fn stun_on_a_creature_that_cant_block_still_draws() {
    cr!("608.2c", "121.1");
    ruling!(
        "Stun",
        "You can use it after combat or even on a creature which can’t possibly block this turn because it’s that player’s turn to attack, but it generally has no effect other than to let you draw a card."
    );
    supported("Stun");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PostcombatMain);
    t.lands(P0, "Mountain", 2);
    let stun = t.hand(P0, "Stun");
    let hand = t.hand_size(P0);
    t.g.turn.priority = Some(P0);
    t.cast(P0, stun).target(bears).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand, "Stun left the hand, a card was drawn");
    assert!(t.in_graveyard(P0, "Stun"));
}

#[test]
fn untimely_malfunction_changes_the_target_on_resolution_if_possible() {
    cr!("115.7", "115.7d", "608.2c");
    ruling!(
        "Untimely Malfunction",
        "You don't choose the new target for the spell or ability targeted with Untimely Malfunction's second mode until Untimely Malfunction resolves. You must change the target if possible."
    );
    supported("Untimely Malfunction");
    // P1's Shock targets P0's Bears; the new target is chosen as Malfunction resolves
    // (P1's Craw Wurm arrives after it was cast).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.answer_targets(P1, &[obj(bears)]);
    let shock = cast_card(&mut t, P1, "Shock");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[obj(shock)]);
    cast_card(&mut t, P0, "Untimely Malfunction");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm)]);
    t.resolve();
    assert_eq!(crate::r_s25_common::targets_of(&t, shock), vec![obj(wurm)]);
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).damage, 2);
    assert_eq!(t.obj_now(bears).damage, 0);
    // No other legal target (Giant Growth on the only creature): the target isn't changed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let growth = cast_card(&mut t, P0, "Giant Growth");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[obj(growth)]);
    cast_card(&mut t, P0, "Untimely Malfunction");
    t.resolve();
    assert_eq!(crate::r_s25_common::targets_of(&t, growth), vec![obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn absolving_lammasu_optional_target_and_illegal_target() {
    cr!("608.2b", "115.10");
    ruling!(
        "Absolving Lammasu",
        "You don't have to choose a target for Absolving Lammasu's last ability. However, if you do, and that permanent is an illegal target at the time the ability tries to resolve, it won't resolve and none of its effects will happen. You won't gain life."
    );
    supported("Absolving Lammasu");
    // No target: you gain 3 life.
    let mut t = TestGame::new(2);
    let lammasu = t.battlefield(P0, "Absolving Lammasu");
    t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    destroy(&mut t, lammasu);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    // The target becomes illegal: no life.
    let mut t = TestGame::new(2);
    let lammasu = t.battlefield(P0, "Absolving Lammasu");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    destroy(&mut t, lammasu);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn up_to_targets_may_be_zero() {
    cr!("115.10", "601.2c");
    ruling!(
        "Boros Battleshaper",
        "You may choose zero targets for either or both instances of “up to one target creature.”"
    );
    ruling!("Glarewielder", "You may choose zero, one, or two target creatures.");
    ruling!(
        "Markov Warlord",
        "You may target zero, one, or two creatures with Markov Warlord's \"enters\" triggered ability."
    );
    for name in ["Glarewielder", "Markov Warlord"] {
        supported(name);
        for n in 0..=2 {
            let mut t = TestGame::new(2);
            let ours: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
            let chosen: Vec<Entity> = ours[..n].iter().map(|o| obj(*o)).collect();
            t.answer_targets(P0, &chosen);
            t.enter(P0, name);
            t.resolve_all();
            for (i, b) in ours.iter().enumerate() {
                assert_eq!(can_block(&mut t, *b), i >= n, "{name} with {n} targets");
            }
        }
    }
    // Boros Battleshaper: zero targets for both; then only the second.
    supported("Boros Battleshaper");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Boros Battleshaper");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    t.answer_targets(P0, &[]);
    t.set_step(P0, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(can_block(&mut t, bears));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Boros Battleshaper");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    t.answer_targets(P0, &[obj(bears)]);
    t.set_step(P0, Step::Upkeep);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!can_block(&mut t, bears));
}

#[test]
fn spara_adjudicators_granted_mana_ability_pays_for_casting_it() {
    cr!("601.2g", "605.3a", "611.2b");
    ruling!(
        "Spara's Adjudicators",
        "You may use the mana ability Spara's Adjudicators grants to the land while casting Spara's Adjudicators from exile."
    );
    supported("Spara's Adjudicators");
    let mut t = TestGame::new(2);
    let card = t.hand(P0, "Spara's Adjudicators");
    // The {2} is paid from the pool, so the target land stays untapped.
    let land = t.battlefield(P0, "Wastes");
    mana(&mut t, P0, mtg_engine::mana::ManaType::C, 2);
    t.answer_targets(P0, &[obj(land)]);
    activate_containing(&mut t, P0, card, "from your hand").expect("activated");
    t.resolve_all();
    let exiled = t.g.current(card);
    assert_eq!(t.zone(exiled), Zone::Exile);
    // {2}{G}{W}{U}: only the granted ability makes green.
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    t.g.turn.priority = Some(P0);
    t.cast(P0, exiled).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Spara's Adjudicators").len(), 1);
    t.g.recompute();
    let w = t.g.current(land);
    assert!(
        t.g.obj(w).chars.abilities.len() <= 1,
        "the land's granted ability ended once the card was cast"
    );
}

#[test]
fn breeches_exiled_land_follows_normal_timing() {
    cr!("305.2", "505.6b");
    ruling!(
        "Breeches, Eager Pillager",
        "You pay all costs and follow all normal timing rules for cards played this way. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    supported("Breeches, Eager Pillager");
    let mut t = TestGame::new(2);
    let breeches = t.battlefield(P0, "Breeches, Eager Pillager");
    let forest = t.library_top(P0, "Forest");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![2]));
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(breeches, Entity::Player(P1))]);
    t.resolve_all();
    let forest = t.g.current(forest);
    assert_eq!(t.zone(forest), Zone::Exile);
    assert!(!can_play_land(&mut t, P0, forest), "not during combat");
    t.advance_to(P0, Step::PostcombatMain);
    assert!(can_play_land(&mut t, P0, forest), "in the main phase, stack empty");
    // Not while the stack isn't empty.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(!can_play_land(&mut t, P0, forest));
}
