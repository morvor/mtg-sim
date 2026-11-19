//! Rulings batch P086 — hate for noncreature spells and spell-based restrictions:
//! Sanctum Prelate, Aurelia's Fury, Permission Denied, Hope of Ghirapur, Change the
//! Equation, Reidane, Mage Hunter, Laquatus's Disdain, Grafdigger's Cage and Cerulean
//! Drake.

use crate::r_p086_common::*;
use mtg_engine::decision::{Action, Answer};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// --- X in mana values -----------------------------------------------------------------------

#[test]
fn sanctum_prelate_uses_the_value_chosen_for_x() {
    cr!("202.3e", "601.2b", "601.2e");
    ruling!(
        "Sanctum Prelate",
        "For spells with {X} in their mana costs, use the value chosen for X to determine if the spell's mana value is the chosen number."
    );
    supported("Sanctum Prelate");
    // Rolling Thunder: {X}{R}{R}, a noncreature spell.
    for (x, ok) in [(2, false), (1, true), (3, true)] {
        let mut t = TestGame::new(2);
        t.answer(P0, DecisionKind::Number, Answer::Number(4));
        t.enter(P0, "Sanctum Prelate");
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Mountain", 2 + x as usize);
        let thunder = t.hand(P1, "Rolling Thunder");
        let r = t.cast(P1, thunder).x(x).target(P0).try_go();
        assert_eq!(r.is_ok(), ok, "X={x}");
        if !ok {
            assert!(t.in_hand(P1, "Rolling Thunder"));
        }
    }
}

#[test]
fn change_the_equation_uses_the_value_chosen_for_x() {
    cr!("202.3e", "115.1");
    ruling!(
        "Change the Equation",
        "If a spell has {X} in its mana cost, use the value chosen for X when calculating that spell’s mana value."
    );
    supported("Change the Equation");
    // Endless One ({X}, colorless) for the first mode, mana value 2 or less; Disintegrate
    // ({X}{R}, red) for the second, mana value 6 or less.
    for (name, mode, x, ok) in [
        ("Endless One", 0, 2, true),
        ("Endless One", 0, 3, false),
        ("Disintegrate", 1, 5, true),
        ("Disintegrate", 1, 6, false),
    ] {
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Mountain", 1 + x as usize);
        let d = t.hand(P1, name);
        let spell = t.cast(P1, d).x(x).target(P0).go();
        let from = t.asked().len();
        t.lands(P0, "Island", 2);
        let cte = t.hand(P0, "Change the Equation");
        t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![mode]));
        let _ = t.cast(P0, cte).try_go();
        let offered = target_candidates(&t, P0, from);
        let can = offered.iter().any(|c| c.contains(&obj(spell)));
        assert_eq!(can, ok, "{name} X={x}");
    }
}

#[test]
fn reidane_uses_the_value_of_x_for_its_tax() {
    cr!("202.3e", "601.2f");
    ruling!(
        "Reidane, God of the Worthy // Valkmira, Protector's Shield",
        "If a spell has {X} in its mana cost, use the value of X to determine whether its mana value is 4 or greater. If it is, Reidane's last ability will apply. If not, it won't."
    );
    // Disintegrate with X=3 has mana value 4: it costs {2} more. With X=2 it doesn't.
    for (x, extra) in [(3usize, 2usize), (2, 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(
            P0,
            "Reidane, God of the Worthy // Valkmira, Protector's Shield",
        );
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Mountain", 1 + x + 2);
        let d = t.hand(P1, "Disintegrate");
        t.cast(P1, d).x(x as i64).target(P0).go();
        let tapped =
            t.g.permanents()
                .filter(|o| o.controller == P1 && o.tapped)
                .count();
        assert_eq!(tapped, 1 + x + extra, "X={x}");
    }
}

// --- Aurelia's Fury ---------------------------------------------------------------------------

/// P0 casts Aurelia's Fury with X = the sum of `split`, dividing it among `targets`.
fn aurelias_fury(t: &mut TestGame, targets: &[Entity], split: &[i64]) -> ObjectId {
    let x: i64 = split.iter().sum();
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", x as usize);
    let card = t.hand(P0, "Aurelia's Fury");
    t.answer_targets(P0, targets);
    divide(t, P0, split);
    t.cast(P0, card).x(x).go()
}

#[test]
fn aurelias_fury_is_divided_as_it_is_cast_at_least_1_each() {
    cr!("601.2d", "601.2b");
    ruling!(
        "Aurelia's Fury",
        "You announce the value of X and how the damage will be divided as part of casting Aurelia’s Fury. Each chosen target must receive at least 1 damage."
    );
    supported("Aurelia's Fury");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    let from = t.asked().len();
    let spell = aurelias_fury(&mut t, &[obj(a), obj(b)], &[2, 1]);
    // The division was asked while casting, with a minimum of 1 for each target.
    let divisions: Vec<(u32, u32)> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::Divide {
                total, min_each, ..
            } => Some((*total, *min_each)),
            _ => None,
        })
        .collect();
    assert_eq!(divisions, vec![(3, 1)]);
    assert_eq!(crate::r_s25_common::x_of(&t, spell), Some(3));
    t.resolve_all();
    assert_eq!(damage_marked(&t, a), 2);
    assert_eq!(damage_marked(&t, b), 1);
    // A division giving a target 0 isn't accepted.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    aurelias_fury(&mut t, &[obj(a), obj(b)], &[3, 0]);
    t.resolve_all();
    assert!(damage_marked(&t, a) >= 1 && damage_marked(&t, b) >= 1);
}

#[test]
fn aurelias_fury_keeps_the_division_for_the_remaining_legal_targets() {
    cr!("608.2b", "601.2d");
    ruling!(
        "Aurelia's Fury",
        "If Aurelia’s Fury has multiple targets, and some but not all of them are illegal targets when Aurelia’s Fury resolves, Aurelia’s Fury will still deal damage to the remaining legal targets according to the original damage division."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    aurelias_fury(&mut t, &[obj(a), obj(b), Entity::Player(P1)], &[1, 2, 3]);
    cast_new(&mut t, P1, "Unsummon", &[obj(a)]);
    t.resolve();
    t.resolve_all();
    assert_eq!(damage_marked(&t, b), 2);
    assert!(is_tapped(&t, b));
    assert_eq!(t.life(P1), 17);
    // P1 was dealt damage: no noncreature spells this turn.
    assert!(!can_cast_new(&mut t, P1, "Shock"));
}

#[test]
fn aurelias_fury_with_all_targets_illegal_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Aurelia's Fury",
        "If all of the targets are illegal when Aurelia’s Fury tries to resolve, it won’t resolve and none of its effects will happen. Nothing will be dealt damage."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Craw Wurm");
    let b = t.battlefield(P1, "Craw Wurm");
    aurelias_fury(&mut t, &[obj(a), obj(b)], &[2, 2]);
    // Both targets become illegal.
    cast_new(&mut t, P1, "Unsummon", &[obj(a)]);
    t.resolve();
    destroy(&mut t, b);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Aurelia's Fury"));
    assert_eq!(t.life(P1), 20);
    // Nothing was dealt damage: P1 can still cast noncreature spells.
    let c = in_hand_with_mana(&mut t, P1, "Healing Salve");
    assert!(castable(&mut t, P1, c));
}

// --- Permission Denied ------------------------------------------------------------------------

/// P1 casts `name` (with `targets`), then P0 casts Permission Denied targeting it.
fn deny(t: &mut TestGame, name: &str, targets: &[Entity]) -> (ObjectId, ObjectId) {
    let spell = cast_new(t, P1, name, targets);
    let pd = cast_new(t, P0, "Permission Denied", &[obj(spell)]);
    (spell, pd)
}

#[test]
fn permission_denied_still_stops_noncreature_spells_if_the_spell_isnt_countered() {
    cr!("608.2c", "101.2");
    ruling!(
        "Permission Denied",
        "If Permission Denied resolves but the spell isn't countered (perhaps because it can't be countered), opponents will still be unable to cast noncreature spells this turn."
    );
    supported("Permission Denied");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Abrupt Decay can't be countered.
    deny(&mut t, "Abrupt Decay", &[obj(bears)]);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(!can_cast_new(&mut t, P1, "Healing Salve"));
}

#[test]
fn permission_denied_with_an_illegal_target_doesnt_stop_noncreature_spells() {
    cr!("608.2b");
    ruling!(
        "Permission Denied",
        "If the spell is an illegal target as Permission Denied tries to resolve, it won't resolve and none of its effects will happen. Your opponents will still be able to cast noncreature spells this turn."
    );
    let mut t = TestGame::new(2);
    let (shock, _) = deny(&mut t, "Shock", &[Entity::Player(P0)]);
    // The Shock is countered by something else first.
    cast_new(&mut t, P0, "Counterspell", &[obj(shock)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Permission Denied"));
    assert!(can_cast_new(&mut t, P1, "Healing Salve"));
}

#[test]
fn permission_denied_applies_to_every_opponent() {
    cr!("102.2", "608.2c");
    ruling!(
        "Permission Denied",
        "Permission Denied's effect applies to all opponents, not just the player who controlled the spell targeted by Permission Denied."
    );
    let mut t = TestGame::new(3);
    deny(&mut t, "Shock", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(!can_cast_new(&mut t, P1, "Healing Salve"));
    assert!(!can_cast_new(&mut t, P2, "Healing Salve"));
    // Its controller isn't affected.
    assert!(can_cast_new(&mut t, P0, "Healing Salve"));
}

// --- Hope of Ghirapur -------------------------------------------------------------------------

/// P0's Hope of Ghirapur deals combat damage to `p` and is sacrificed targeting them; the
/// ability resolves. Ends in P0's end of combat step.
fn hope(t: &mut TestGame, p: PlayerId) {
    let hope = t.battlefield(P0, "Hope of Ghirapur");
    attack_with(t, &[(hope, Entity::Player(p))]);
    block_and_finish(t, p, &[]);
    assert_eq!(t.life(p), 19);
    activate(t, P0, hope, "Sacrifice", &[Entity::Player(p)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hope of Ghirapur"));
}

#[test]
fn hope_of_ghirapur_leaves_abilities_lands_and_creature_spells() {
    cr!("101.2", "305.1");
    ruling!(
        "Hope of Ghirapur",
        "The affected player may still activate abilities, play lands, and cast creature spells."
    );
    supported("Hope of Ghirapur");
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    hope(&mut t, P1);
    next_main(&mut t, P1);
    assert!(!can_cast_new(&mut t, P1, "Shock"));
    assert!(can_cast_new(&mut t, P1, "Raging Goblin"));
    let forest = t.hand(P1, "Forest");
    assert!(crate::r_s02_common::can_play_land(&mut t, P1, forest));
    assert!(crate::r_s02_common::can_activate(&mut t, P1, sorcerer));
}

#[test]
fn hope_of_ghirapur_lasts_until_after_your_upkeep_triggers() {
    cr!("611.2a", "503.1a", "117.3a");
    ruling!(
        "Hope of Ghirapur",
        "The affected player won’t be able to cast any noncreature spells until after the time that your “beginning of upkeep” triggered abilities are put onto the stack on your next turn."
    );
    let mut t = TestGame::new(2);
    hope(&mut t, P1);
    next_main(&mut t, P1);
    assert!(!can_cast_new(&mut t, P1, "Shock"));
    // P0's next upkeep: the effect has ended by the time anyone gets priority, with P0's
    // upkeep trigger on the stack.
    t.battlefield(P0, "Plague Sliver");
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    assert!(can_cast_new(&mut t, P1, "Shock"));
}

#[test]
fn hope_of_ghirapur_lasts_through_a_skipped_turn() {
    cr!("611.2a", "614.10", "500.7");
    ruling!(
        "Hope of Ghirapur",
        "If you skip your turn, the affected player will be unable to cast noncreature spells until you actually start a turn."
    );
    let mut t = TestGame::new(2);
    hope(&mut t, P1);
    t.g.players[P0.idx()]
        .skips
        .push(mtg_engine::ability::StepKind::Turn);
    next_main(&mut t, P1);
    assert!(!can_cast_new(&mut t, P1, "Shock"));
    // P0's next turn is skipped: P1's following turn still has the restriction.
    next_main(&mut t, P1);
    assert_eq!(t.g.player(P0).turns_taken, 1);
    assert!(!can_cast_new(&mut t, P1, "Shock"));
    next_main(&mut t, P0);
    assert!(can_cast_new(&mut t, P1, "Shock"));
}

#[test]
fn hope_of_ghirapur_doesnt_affect_spells_cast_before_or_in_response() {
    cr!("611.2c", "117.7", "608.2");
    ruling!(
        "Hope of Ghirapur",
        "The target player (and any other player) may cast spells in response to the activated ability of Hope of Ghirapur. The ability won’t affect those spells and it won’t affect spells that the target player cast before you activated it. (In other words, the ability can’t be used to counter a spell.)"
    );
    let mut t = TestGame::new(2);
    let hope = t.battlefield(P0, "Hope of Ghirapur");
    attack_with(&mut t, &[(hope, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    // P1 casts a spell, P0 sacrifices Hope in response, P1 responds again.
    cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    activate(&mut t, P0, hope, "Sacrifice", &[Entity::Player(P1)]);
    cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 15);
    assert!(!can_cast_new(&mut t, P1, "Shock"));
}

#[test]
fn hope_of_ghirapur_lasts_until_a_departed_players_turn_would_have_begun() {
    cr!("800.4m", "611.2a");
    ruling!(
        "Hope of Ghirapur",
        "When a player leaves a multiplayer game, any continuous effects with durations that last until that player’s next turn will last until that turn would have begun. They neither expire immediately nor last indefinitely."
    );
    let mut t = TestGame::new(3);
    hope(&mut t, P1);
    t.take_action(P0, Action::Concede);
    t.settle();
    assert!(t.has_lost(P0));
    next_main(&mut t, P1);
    assert!(!can_cast_new(&mut t, P1, "Shock"));
    next_main(&mut t, P2);
    assert!(!can_cast_new(&mut t, P1, "Shock"));
    // P0's turn would have begun before P1's next one.
    next_main(&mut t, P1);
    assert!(can_cast_new(&mut t, P1, "Shock"));
}

// --- Mage Hunter, Laquatus's Disdain, Grafdigger's Cage, Cerulean Drake -----------------------

#[test]
fn mage_hunters_trigger_resolves_before_the_spell() {
    cr!("603.3", "405.5");
    ruling!(
        "Mage Hunter",
        "Mage Hunter’s triggered ability will resolve before the spell that caused it to trigger."
    );
    supported("Mage Hunter");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mage Hunter");
    let shock = cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert!(t.g.stack.contains(&shock));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn laquatuss_disdain_counters_a_flashback_spell() {
    cr!("702.34a", "601.2a");
    ruling!(
        "Laquatus's Disdain",
        "Can be used to counter a spell cast using the Flashback ability, or cast from the graveyard using any other effect."
    );
    supported("Laquatus's Disdain");
    let mut t = TestGame::new(2);
    let tt = t.graveyard(P1, "Think Twice");
    t.lands(P1, "Island", 3);
    let spell = t
        .cast(P1, tt)
        .method(mtg_engine::object::CastMethod::Keyword(
            mtg_engine::keywords::KeywordKind::Flashback,
        ))
        .go();
    cast_new(&mut t, P0, "Laquatus's Disdain", &[obj(spell)]);
    t.resolve();
    assert!(!t.g.stack.contains(&spell));
    assert!(t.in_exile("Think Twice"));
    // A spell cast from hand can't be targeted.
    let mut t = TestGame::new(2);
    let shock = cast_new(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    assert!(!spell_targets(&mut t, P0, "Laquatus's Disdain").contains(&obj(shock)));
}

#[test]
fn grafdiggers_cage_allows_casting_a_card_exiled_from_a_library() {
    cr!("601.2a", "601.3");
    ruling!(
        "Grafdigger's Cage",
        "If an effect exiles a card from a library and allows a player to cast it, that player may do so. The spell is cast from exile, not a library."
    );
    supported("Grafdigger's Cage");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grafdigger's Cage");
    crate::r_s01_common::stack_library(&mut t, P0, &["Shock", "Shock"]);
    cast_new(&mut t, P0, "Light Up the Stage", &[]);
    t.resolve_all();
    let exiled = t.g.find_in_zone(mtg_engine::object::Zone::Exile, "Shock");
    assert_eq!(exiled.len(), 2);
    t.lands(P0, "Mountain", 1);
    assert!(castable(&mut t, P0, exiled[0]));
    t.cast(P0, exiled[0]).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn cerulean_drake_can_counter_a_spell_with_several_targets_one_of_them_you() {
    cr!("115.10", "115.1");
    ruling!(
        "Cerulean Drake",
        "Cerulean Drake’s activated ability can target a spell that has multiple targets if at least one of those targets is you."
    );
    supported("Cerulean Drake");
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P0, "Cerulean Drake");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    // Forked Bolt divides 2 damage among one or two targets.
    t.answer_targets(P1, &[Entity::Player(P0), obj(bears)]);
    divide(&mut t, P1, &[1, 1]);
    lands_for_cost(&mut t, P1, "Forked Bolt");
    let card = t.hand(P1, "Forked Bolt");
    let spell = t.cast(P1, card).go();
    assert!(ability_targets(&mut t, drake, 0).contains(&obj(spell)));
    activate(&mut t, P0, drake, "Counter", &[obj(spell)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(t.on_battlefield(bears));
    // A spell that targets only a creature isn't a legal target.
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P0, "Cerulean Drake");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let shock = cast_new(&mut t, P1, "Shock", &[obj(bears)]);
    assert!(!ability_targets(&mut t, drake, 0).contains(&obj(shock)));
}
