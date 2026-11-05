//! Rulings batch P040 — "whenever ... deals combat damage to a player, draw a card"
//! abilities ("curiosity") and the cards grouped with them: how often they trigger, which
//! creatures they apply to, values locked in or counted on resolution, last known
//! information, blocks that stay put, and cost reductions (Korvold).

use crate::r_p039_common::cast_named;
use crate::r_p039_creatureland::activate_text;
use crate::r_p040_common::*;
use crate::r_s01_common::{attack_with, stack_library, supported, triggers_on_stack, watch};
use crate::r_s02_common::{can_play_land, destroy};
use crate::r_s04_common::next_upkeep;
use crate::r_s04_common::spell_targets;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Runs a combat for the active player and resolves everything (scripted answers or
/// defaults) until the end of combat step.
fn combat(t: &mut TestGame, attackers: &[(ObjectId, Entity)], blocks: &[(ObjectId, ObjectId)]) {
    fight(t, attackers, blocks);
    t.resolve_all();
}

/// Declares `attackers` for the active player, then `blocks` for P1, and stops in the
/// declare blockers step with priority (blocks declared, no damage yet).
fn blocked(t: &mut TestGame, attackers: &[(ObjectId, Entity)], blocks: &[(ObjectId, ObjectId)]) {
    attack_with(t, attackers);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
    let ap = t.g.turn.active;
    t.advance_to(ap, Step::DeclareBlockers);
}

/// Goes from the declare attackers/blockers step to the combat damage step (combat damage
/// dealt, its triggers put on the stack, not resolved).
fn to_damage(t: &mut TestGame) {
    let ap = t.g.turn.active;
    t.advance_to(ap, Step::CombatDamage);
    t.settle();
}

fn is_blocked(t: &TestGame, id: ObjectId) -> bool {
    t.g.combat.as_ref().is_some_and(|c| c.is_blocked(id))
}

// ---------------------------------------------------------------------------
// Which creatures the ability applies to
// ---------------------------------------------------------------------------

#[test]
fn jolrael_draws_for_any_land_creature() {
    cr!("510.2", "603.2");
    ruling!(
        "Jolrael, Voice of Zhalfir",
        "Jolrael's last ability applies to any land creature you control that deals combat damage to a player, not only lands that become land creatures because of Jolrael's first ability."
    );
    supported("Jolrael, Voice of Zhalfir");
    let mut t = TestGame::new(2);
    old(&mut t, P0, "Jolrael, Voice of Zhalfir");
    // Dryad Arbor is a land creature on its own; the Bears aren't lands.
    let arbor = old(&mut t, P0, "Dryad Arbor");
    let bears = old(&mut t, P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    combat(&mut t, &[(arbor, pl(P1)), (bears, pl(P1))], &[]);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn keeper_of_fables_triggers_even_if_dealt_lethal_damage_at_the_same_time() {
    cr!("510.2", "603.2");
    ruling!(
        "Keeper of Fables",
        "Keeper of Fables's ability triggers even if it's dealt lethal damage at the same time that non-Humans you control deal combat damage to a player."
    );
    let mut t = TestGame::new(2);
    let keeper = old(&mut t, P0, "Keeper of Fables");
    let bears = old(&mut t, P0, "Grizzly Bears");
    let wurm = old(&mut t, P1, "Craw Wurm");
    let hand = t.hand_size(P0);
    combat(
        &mut t,
        &[(keeper, pl(P1)), (bears, pl(P1))],
        &[(wurm, keeper)],
    );
    assert!(t.in_graveyard(P0, "Keeper of Fables"));
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn edric_lets_you_draw_when_your_creature_hits_an_opponent() {
    cr!("510.2", "603.2");
    ruling!(
        "Edric, Spymaster of Trest",
        "When a creature you control deals combat damage to one of your opponents, you may draw a card."
    );
    for draw in [true, false] {
        let mut t = TestGame::new(2);
        old(&mut t, P0, "Edric, Spymaster of Trest");
        let bears = old(&mut t, P0, "Grizzly Bears");
        yes(&mut t, P0, draw);
        let from = mark(&t);
        let hand = t.hand_size(P0);
        combat(&mut t, &[(bears, pl(P1))], &[]);
        assert_eq!(
            yes_nos(&t, from)
                .into_iter()
                .map(|(p, _)| p)
                .collect::<Vec<_>>(),
            vec![P0]
        );
        assert_eq!(t.hand_size(P0), hand + usize::from(draw));
    }
}

#[test]
fn ohran_frostfang_is_a_snow_creature_with_no_extra_rules() {
    cr!("205.4a");
    ruling!(
        "Ohran Frostfang",
        "Snow is a supertype, not a card type. It has no rules meaning or function by itself, but other cards and abilities may refer to it."
    );
    supported("Ohran Frostfang");
    let mut t = TestGame::new(2);
    let snake = old(&mut t, P0, "Ohran Frostfang");
    let chars = &t.obj_now(snake).chars;
    assert!(chars.supertypes.contains(Supertype::Snow));
    assert_eq!(
        chars.card_types.iter().collect::<Vec<_>>(),
        vec![CardType::Creature]
    );
    // It's an ordinary creature otherwise: it attacks, deals damage and its abilities work
    // (attacking creatures you control have deathtouch; a creature dealing combat damage
    // to a player draws a card).
    let bears = old(&mut t, P0, "Grizzly Bears");
    let wall = old(&mut t, P1, "Wall of Stone");
    let hand = t.hand_size(P0);
    combat(
        &mut t,
        &[(snake, pl(P1)), (bears, pl(P1))],
        &[(wall, bears)],
    );
    assert_eq!(t.life(P1), 18);
    assert!(t.in_graveyard(P1, "Wall of Stone"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn kutzil_compares_power_with_base_power() {
    cr!("604.3", "613.4a", "613.4c");
    ruling!(
        "Kutzil, Malamet Exemplar",
        "Some creatures have base power and toughness 0/0 and an ability that gives them a bonus based on some criteria. Those are not characteristic-defining abilities, and that ability doesn't change its base power and toughness. Such a creature will have power greater than its base power if the ability is raising its power above 0."
    );
    supported("Kutzil, Malamet Exemplar");
    supported("Moon-Vigil Adherents");
    // Moon-Vigil Adherents: 0/0, "gets +1/+1 for each creature you control and each
    // creature card in your graveyard" (not a characteristic-defining ability).
    let mut t = TestGame::new(2);
    old(&mut t, P0, "Kutzil, Malamet Exemplar");
    let adherents = old(&mut t, P0, "Moon-Vigil Adherents");
    assert_eq!(t.pt(adherents), (2, 2));
    let hand = t.hand_size(P0);
    combat(&mut t, &[(adherents, pl(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), hand + 1);
    // Control: Tarmogoyf's power comes from a characteristic-defining ability, so its
    // base power is that value and it isn't greater than its base power.
    let mut t = TestGame::new(2);
    old(&mut t, P0, "Kutzil, Malamet Exemplar");
    let goyf = old(&mut t, P0, "Tarmogoyf");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P0, "Forest");
    assert_eq!(t.pt(goyf), (2, 3));
    let hand = t.hand_size(P0);
    combat(&mut t, &[(goyf, pl(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn tovolar_triggers_for_each_wolf_or_werewolf() {
    cr!("510.2", "603.2c");
    ruling!(
        "Tovolar, Dire Overlord // Tovolar, the Midnight Scourge",
        "The first ability of each of Tovolar's faces triggers once for each Wolf or Werewolf you control that deals combat damage to a player. For example, if you attack with three Wolf creatures and they all deal combat damage to a player, the ability triggers three times."
    );
    supported("Tovolar, Dire Overlord // Tovolar, the Midnight Scourge");
    // Front face (day) and back face (night).
    for night in [false, true] {
        let mut t = TestGame::new(2);
        let tovolar = old(
            &mut t,
            P0,
            "Tovolar, Dire Overlord // Tovolar, the Midnight Scourge",
        );
        if night {
            t.g.day = Some(false);
            t.g.recompute();
            t.settle();
            assert_eq!(
                t.obj_now(tovolar).chars.name,
                "Tovolar, the Midnight Scourge"
            );
        }
        let wolves: Vec<ObjectId> = (0..3).map(|_| old(&mut t, P0, "Timberpack Wolf")).collect();
        let bears = old(&mut t, P0, "Grizzly Bears");
        let mut attackers: Vec<(ObjectId, Entity)> = wolves.iter().map(|w| (*w, pl(P1))).collect();
        attackers.push((bears, pl(P1)));
        attack_with(&mut t, &attackers);
        to_damage(&mut t);
        assert_eq!(triggers_on_stack(&t, "draw a card"), 3, "night: {night}");
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 3);
    }
}

// ---------------------------------------------------------------------------
// One card per trigger, no matter how much damage
// ---------------------------------------------------------------------------

#[test]
fn one_card_per_creature_no_matter_how_much_damage() {
    cr!("510.2", "603.2c");
    ruling!(
        "Neurok Commando",
        "Only one card is drawn no matter how much combat damage was dealt."
    );
    ruling!(
        "Scroll Thief",
        "The ability triggers just once each time Scroll Thief deals combat damage to a player, regardless of how much damage it deals."
    );
    ruling!(
        "Scroll Thief",
        "The ability is mandatory. You must draw a card when it resolves."
    );
    ruling!(
        "Coastal Piracy",
        "You draw one card per creature, not one per point of damage."
    );
    ruling!(
        "Reconnaissance Mission",
        "Reconnaissance Mission's triggered ability triggers for each creature you control that deals combat damage to a player. You may draw one card each time the ability resolves, no matter how much damage was dealt."
    );
    // (card, the attacker with it, whether the draw is optional)
    // (card, the attacker with it, whether the draw is optional, damage dealt to P1)
    let cases: [(&str, &str, bool, i32); 4] = [
        ("Neurok Commando", "Neurok Commando", true, 3),
        ("Scroll Thief", "Scroll Thief", false, 2),
        ("Coastal Piracy", "Grizzly Bears", true, 3 + 4),
        ("Reconnaissance Mission", "Grizzly Bears", true, 3 + 4),
    ];
    for (name, attacker, optional, dealt) in cases {
        supported(name);
        let mut t = TestGame::new(2);
        // Glorious Anthem makes the damage of each creature bigger than 1.
        t.battlefield(P0, "Glorious Anthem");
        let card = if name == attacker {
            old(&mut t, P0, name)
        } else {
            t.battlefield(P0, name);
            old(&mut t, P0, attacker)
        };
        let other = old(&mut t, P0, "Hill Giant");
        let mut attackers = vec![(card, pl(P1))];
        if name != attacker {
            // Two creatures: one trigger and one card each.
            attackers.push((other, pl(P1)));
        }
        attack_with(&mut t, &attackers);
        for _ in 0..2 {
            yes(&mut t, P0, true);
        }
        to_damage(&mut t);
        let n = attackers.len();
        assert_eq!(20 - t.life(P1), dealt, "{name}");
        assert_eq!(triggers_on_stack(&t, "draw a card"), n, "{name}");
        let from = mark(&t);
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + n, "{name}");
        // Optional draws ask; Scroll Thief's mandatory draw doesn't.
        let asked = yes_nos(&t, from).len();
        assert_eq!(asked, if optional { n } else { 0 }, "{name}");
    }
}

#[test]
fn thopter_spy_network_triggers_once_per_damage_step() {
    cr!("510.2", "510.4", "702.4b", "702.7b");
    ruling!(
        "Thopter Spy Network",
        "The last ability will trigger, at most, once per combat damage step per player. However, if at least one artifact creature you control has first strike and others don't, or if an artifact creature you control has double strike, the ability could trigger twice per combat: once in each combat damage step."
    );
    supported("Thopter Spy Network");
    // Two artifact creatures in one damage step: once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thopter Spy Network");
    let a = old(&mut t, P0, "Ornithopter");
    let b = old(&mut t, P0, "Esper Sentinel");
    t.battlefield(P0, "Glorious Anthem");
    let hand = t.hand_size(P0);
    combat(&mut t, &[(a, pl(P1)), (b, pl(P1))], &[]);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), hand + 1);
    // One with first strike, one without: twice.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thopter Spy Network");
    let a = old(&mut t, P0, "Esper Sentinel");
    let b = old(&mut t, P0, "Coiled Tinviper");
    let hand = t.hand_size(P0);
    combat(&mut t, &[(a, pl(P1)), (b, pl(P1))], &[]);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), hand + 2);
    // One with double strike: twice.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Thopter Spy Network");
    let a = old(&mut t, P0, "Esper Sentinel");
    cast_resolve(&mut t, P0, "Assault Strobe", &[o(a)]);
    let hand = t.hand_size(P0);
    combat(&mut t, &[(a, pl(P1))], &[]);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), hand + 2);
}

// ---------------------------------------------------------------------------
// Intervening "if" clauses, values counted on resolution, values locked in
// ---------------------------------------------------------------------------

#[test]
fn thopter_spy_network_upkeep_trigger_has_an_intervening_if() {
    cr!("603.4");
    ruling!(
        "Thopter Spy Network",
        "Thopter Spy Network's first ability has an “intervening ‘if' clause.” That means (1) the ability won't trigger at all unless you control an artifact as your upkeep begins, and (2) the ability will do nothing if you don't control an artifact as it resolves."
    );
    let thopters = |t: &TestGame| {
        t.g.permanents()
            .filter(|o| o.controller == P0 && o.is_token() && o.chars.has_subtype("Thopter"))
            .count()
    };
    // (1) No artifact as the upkeep begins: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thopter Spy Network");
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "create a 1/1"), 0);
    // (2) It triggers, but the artifact is gone as it resolves: nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thopter Spy Network");
    let orni = t.battlefield(P0, "Ornithopter");
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "create a 1/1"), 1);
    destroy(&mut t, orni);
    t.resolve_all();
    assert_eq!(thopters(&t), 0);
    // Control: the artifact is still there: a Thopter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thopter Spy Network");
    t.battlefield(P0, "Ornithopter");
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(thopters(&t), 1);
}

#[test]
fn biorganic_carapace_counts_modified_creatures_on_resolution() {
    cr!("608.2h", "700.9");
    ruling!(
        "Biorganic Carapace",
        "The number of modified creatures you control is counted at the time the ability resolves, not the time that damage was dealt."
    );
    supported("Biorganic Carapace");
    let mut t = TestGame::new(2);
    let bears = old(&mut t, P0, "Grizzly Bears");
    let other = old(&mut t, P0, "Hill Giant");
    t.answer_targets(P0, &[o(bears)]);
    t.enter(P0, "Biorganic Carapace");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    attack_with(&mut t, &[(bears, pl(P1))]);
    to_damage(&mut t);
    assert_eq!(triggers_on_stack(&t, "draw a card for each modified"), 1);
    // When the damage was dealt only the Bears were modified; a counter on the Hill
    // Giant before the ability resolves makes it two.
    t.g.add_counters(o(other), counters::PLUS1, 1, None);
    t.g.flush_events();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn jolrael_locks_in_x_as_its_first_ability_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Jolrael, Voice of Zhalfir",
        "Use the number of cards in your hand as Jolrael's first ability resolves to determine the value of X. Once the ability resolves, that value is locked in. The power and toughness of the land creature won't change if the number of cards in your hand changes later in the turn."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    old(&mut t, P0, "Jolrael, Voice of Zhalfir");
    let forest = old(&mut t, P0, "Forest");
    t.g.players[P0.idx()].hand.clear();
    for _ in 0..3 {
        t.hand(P0, "Island");
    }
    t.answer_targets(P0, &[o(forest)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(
        triggers_on_stack(&t, "becomes an X/X"),
        1,
        "{:?} {:?}",
        crate::r_s04_common::stack_items(&t),
        t.g.turn.step
    );
    // One more card before it resolves: X is 4.
    t.hand(P0, "Island");
    t.resolve_all();
    assert!(t.obj_now(forest).is(CardType::Creature));
    assert!(t.obj_now(forest).is(CardType::Land));
    assert_eq!(t.pt(forest), (4, 4));
    // Later changes to the hand don't change it.
    t.g.draw_cards(P0, 2);
    t.g.flush_events();
    t.g.recompute();
    assert_eq!(t.pt(forest), (4, 4));
    t.g.players[P0.idx()].hand.clear();
    t.g.recompute();
    assert_eq!(t.pt(forest), (4, 4));
}

#[test]
fn subira_checks_power_right_after_damage() {
    cr!("603.7a", "603.2", "510.2");
    ruling!(
        "Subira, Tulzidi Caravanner",
        "Once Subira's last ability resolves, check to see if the delayed triggered ability should trigger based on the power of the creatures immediately after damage is dealt, not based on their power as the ability resolves."
    );
    ruling!(
        "Subira, Tulzidi Caravanner",
        "You can activate Subira's last ability if your hand is empty. You simply discard zero cards."
    );
    supported("Subira, Tulzidi Caravanner");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let subira = old(&mut t, P0, "Subira, Tulzidi Caravanner");
    let bears = old(&mut t, P0, "Grizzly Bears");
    let giant = old(&mut t, P0, "Hill Giant");
    t.g.players[P0.idx()].hand.clear();
    mana_all(&mut t, P0, 2);
    activate_text(&mut t, P0, subira, "Discard your hand").expect("activate with an empty hand");
    t.resolve_all();
    assert!(t.obj_now(subira).tapped);
    // As the ability resolved, the Bears had power 2 and the Giant 3. Then the Bears get
    // +3/+3 and the Giant -2/-2.
    cast_resolve(&mut t, P0, "Giant Growth", &[o(bears)]);
    cast_resolve(&mut t, P0, "Disfigure", &[o(giant)]);
    assert_eq!((t.pt(bears).0, t.pt(giant).0), (5, 1));
    let hand = t.hand_size(P0);
    combat(&mut t, &[(bears, pl(P1)), (giant, pl(P1))], &[]);
    assert_eq!(t.life(P1), 14);
    // Only the Giant (power 1 when it dealt damage) draws.
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------
// Blocked creatures stay blocked
// ---------------------------------------------------------------------------

#[test]
fn nimble_brigand_stays_blocked_after_a_crime() {
    cr!("509.1h", "506.4");
    ruling!(
        "Nimble Brigand",
        "Once Nimble Brigand has been blocked, committing a crime won’t cause Nimble Brigand to become unblocked."
    );
    supported("Nimble Brigand");
    let mut t = TestGame::new(2);
    let brigand = old(&mut t, P0, "Nimble Brigand");
    let wall = old(&mut t, P1, "Wall of Stone");
    blocked(&mut t, &[(brigand, pl(P1))], &[(wall, brigand)]);
    assert!(is_blocked(&t, brigand));
    // A crime: Shock targeting P1.
    cast_named(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert!(is_blocked(&t, brigand));
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(damage(&t, wall), 1);
    assert_eq!(t.hand_size(P0), hand);
    // Control: a crime before blocks makes it unblockable.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let brigand = old(&mut t, P0, "Nimble Brigand");
    let wall = old(&mut t, P1, "Wall of Stone");
    cast_named(&mut t, P0, "Shock", &[pl(P1)]);
    t.resolve_all();
    attack_with(&mut t, &[(brigand, pl(P1))]);
    assert!(!t.g.can_block(wall, brigand));
}

#[test]
fn subira_and_jhessian_balmgiver_dont_unblock_a_blocked_creature() {
    cr!("509.1h", "506.4");
    ruling!(
        "Subira, Tulzidi Caravanner",
        "Once a creature with power 2 or less has been blocked, Subira's middle ability won't cause it to become unblocked."
    );
    ruling!(
        "Jhessian Balmgiver",
        "After blockers have been declared, activating the second ability won’t cause those blocks to change."
    );
    for (card, needle) in [
        ("Subira, Tulzidi Caravanner", "can't be blocked this turn"),
        ("Jhessian Balmgiver", "can't be blocked this turn"),
    ] {
        supported(card);
        // After blocks: still blocked, no damage to P1.
        let mut t = TestGame::new(2);
        let src = old(&mut t, P0, card);
        let bears = old(&mut t, P0, "Grizzly Bears");
        let wall = old(&mut t, P1, "Wall of Stone");
        blocked(&mut t, &[(bears, pl(P1))], &[(wall, bears)]);
        mana_all(&mut t, P0, 1);
        t.answer_targets(P0, &[o(bears)]);
        activate_text(&mut t, P0, src, needle).expect("activate");
        t.resolve_all();
        assert!(is_blocked(&t, bears), "{card}");
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20, "{card}");
        assert_eq!(damage(&t, wall), 2, "{card}");
        // Control: before blocks, the Wall can't block it.
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::BeginningOfCombat);
        let src = old(&mut t, P0, card);
        let bears = old(&mut t, P0, "Grizzly Bears");
        let wall = old(&mut t, P1, "Wall of Stone");
        mana_all(&mut t, P0, 1);
        t.answer_targets(P0, &[o(bears)]);
        activate_text(&mut t, P0, src, needle).expect("activate");
        t.resolve_all();
        attack_with(&mut t, &[(bears, pl(P1))]);
        assert!(!t.g.can_block(wall, bears), "{card}");
    }
}

#[test]
fn legolas_stays_blocked_when_the_blockers_power_drops() {
    cr!("509.1h", "506.4");
    ruling!(
        "Legolas Greenleaf",
        "Reducing a creature's power after it has blocked Legolas Greenleaf will not remove that blocking creature from combat or make Legolas Greenleaf unblocked."
    );
    supported("Legolas Greenleaf");
    let mut t = TestGame::new(2);
    let legolas = old(&mut t, P0, "Legolas Greenleaf");
    let giant = old(&mut t, P1, "Hill Giant");
    let bears = old(&mut t, P1, "Grizzly Bears");
    attack_with(&mut t, &[(legolas, pl(P1))]);
    // Control: power 2 or less can't block it; the Giant (3) can.
    assert!(!t.g.can_block(bears, legolas));
    assert!(t.g.can_block(giant, legolas));
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(giant, legolas)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(is_blocked(&t, legolas));
    // Disfigure: the Giant becomes 1/1.
    cast_named(&mut t, P0, "Disfigure", &[o(giant)]);
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
    assert!(t.g.is_blocking(giant));
    assert!(is_blocked(&t, legolas));
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.hand_size(P0), hand);
}

// ---------------------------------------------------------------------------
// Noncombat damage, any reason
// ---------------------------------------------------------------------------

#[test]
fn lu_xun_and_vedalken_heretic_trigger_on_any_damage_to_an_opponent() {
    cr!("603.2", "120.1");
    ruling!(
        "Lu Xun, Scholar General",
        "Lu Xun's second ability triggers whenever it deals damage to an opponent for any reason. It's not limited to combat damage."
    );
    ruling!(
        "Vedalken Heretic",
        "Vedalken Heretic’s ability triggers when it deals any damage to an opponent, not just combat damage."
    );
    for card in ["Lu Xun, Scholar General", "Vedalken Heretic"] {
        supported(card);
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let src = old(&mut t, P0, card);
        let theirs = old(&mut t, P1, "Craw Wurm");
        // Soul's Fire: "Target creature you control on the battlefield deals damage equal
        // to its power to any target." At P1: draws.
        yes(&mut t, P0, true);
        let hand = t.hand_size(P0);
        cast_resolve(&mut t, P0, "Soul's Fire", &[o(src), pl(P1)]);
        assert_eq!(t.life(P1), 19, "{card}");
        assert_eq!(t.hand_size(P0), hand + 1, "{card}");
        // At P1's creature or at P0 (not an opponent): no trigger.
        let from = mark(&t);
        let hand = t.hand_size(P0);
        cast_resolve(&mut t, P0, "Soul's Fire", &[o(src), o(theirs)]);
        cast_resolve(&mut t, P0, "Soul's Fire", &[o(src), pl(P0)]);
        assert_eq!(damage(&t, theirs), 1, "{card}");
        assert_eq!(t.life(P0), 19, "{card}");
        assert!(yes_nos(&t, from).is_empty(), "{card}");
        assert_eq!(t.hand_size(P0), hand, "{card}");
    }
}

#[test]
fn piper_wright_triggers_on_any_clue_sacrifice() {
    cr!("701.21a", "603.2");
    ruling!(
        "Piper Wright, Publick Reporter",
        "Piper Wright's last ability triggers whenever you sacrifice a Clue for any reason, not just to activate a Clue's activated ability."
    );
    supported("Piper Wright, Publick Reporter");
    let mut t = TestGame::new(2);
    let piper = old(&mut t, P0, "Piper Wright, Publick Reporter");
    let atog = old(&mut t, P0, "Atog");
    // Investigate once (1 damage) through her own ability.
    combat(&mut t, &[(piper, pl(P1))], &[]);
    let clues = crate::r_s01_common::with_subtype(&t, P0, "Clue");
    assert_eq!(clues.len(), 1);
    // Atog: "Sacrifice an artifact: Atog gets +2/+2 until end of turn."
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![o(clues[0])]),
    );
    t.answer_targets(P0, &[o(piper)]);
    activate_text(&mut t, P0, atog, "Sacrifice an artifact").expect("Atog");
    t.resolve_all();
    assert!(crate::r_s01_common::with_subtype(&t, P0, "Clue").is_empty());
    assert_eq!(t.pt(atog), (3, 4));
    assert_eq!(t.counters(piper, counters::PLUS1), 1);
}

// ---------------------------------------------------------------------------
// Delayed triggers and granted abilities
// ---------------------------------------------------------------------------

#[test]
fn tamiyo_delayed_trigger_works_after_she_leaves() {
    cr!("603.7c", "610.3");
    ruling!(
        "Tamiyo, Field Researcher",
        "Tamiyo’s first ability sets up a delayed triggered ability that triggers even if Tamiyo has left the battlefield before those creatures deal combat damage."
    );
    supported("Tamiyo, Field Researcher");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let tamiyo = t.battlefield(P0, "Tamiyo, Field Researcher");
    let bears = old(&mut t, P0, "Grizzly Bears");
    loyalty(&mut t, P0, tamiyo, "+1:", &[o(bears)]);
    destroy(&mut t, tamiyo);
    assert!(!t.on_battlefield(tamiyo));
    let hand = t.hand_size(P0);
    combat(&mut t, &[(bears, pl(P1))], &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn nimble_trapfinder_grants_only_to_creatures_controlled_as_it_resolves() {
    cr!("611.2c", "603.4", "700.8");
    ruling!(
        "Nimble Trapfinder",
        "Nimble Trapfinder’s second ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t get the triggered ability."
    );
    supported("Nimble Trapfinder");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // A full party: Cleric, Rogue (the Trapfinder), Warrior, Wizard.
    let trap = old(&mut t, P0, "Nimble Trapfinder");
    old(&mut t, P0, "Cleric of the Forward Order");
    old(&mut t, P0, "Kargan Warleader");
    old(&mut t, P0, "Prodigal Sorcerer");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(
        triggers_on_stack(&t, "creatures you control gain"),
        1,
        "{:?}",
        crate::r_s04_common::stack_items(&t)
    );
    t.resolve_all();
    // A creature with haste that arrives later doesn't get it.
    let goblin = t.enter(P0, "Raging Goblin");
    t.g.flush_events();
    t.settle();
    let hand = t.hand_size(P0);
    combat(&mut t, &[(trap, pl(P1)), (goblin, pl(P1))], &[]);
    assert_eq!(t.life(P1), 20 - 2 - 1);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn nimble_trapfinder_checks_the_types_entering_permanents_had_as_they_entered() {
    cr!("509.1b");
    ruling!(
        "Nimble Trapfinder",
        "Nimble Trapfinder’s first ability checks only whether a permanent entered the battlefield under your control earlier in the turn and had one of the party creature types as it entered. It doesn’t apply if an entering object gained the type later in the turn. It does apply if the entering object left your control since. It doesn’t matter whether Nimble Trapfinder was under your control as that object entered."
    );
    // A Cleric entered, then left, all before the Trapfinder was on the battlefield: it
    // can't be blocked.
    let mut t = TestGame::new(2);
    let cleric = t.enter(P0, "Cleric of the Forward Order");
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    destroy(&mut t, cleric);
    let trap = old(&mut t, P0, "Nimble Trapfinder");
    let wall = old(&mut t, P1, "Wall of Stone");
    attack_with(&mut t, &[(trap, pl(P1))]);
    assert!(!t.g.can_block(wall, trap));
    // A creature that entered as a Bear and became a Cleric later: it can be blocked.
    let mut t = TestGame::new(2);
    let trap = old(&mut t, P0, "Nimble Trapfinder");
    let bears = t.enter(P0, "Grizzly Bears");
    t.g.flush_events();
    t.settle();
    let cleric = subtype_lists()
        .creature
        .iter()
        .position(|x| x == "Cleric")
        .unwrap();
    t.answer(P0, DecisionKind::Option, Answer::Index(cleric));
    t.enter(P0, "Arcane Adaptation");
    t.g.flush_events();
    t.settle();
    assert!(has_subtype(&t, bears, "Cleric"));
    let wall = old(&mut t, P1, "Wall of Stone");
    attack_with(&mut t, &[(trap, pl(P1))]);
    assert!(t.g.can_block(wall, trap));
}

#[test]
fn family_s_favor_triggers_are_mandatory() {
    cr!("603.5", "122.1c");
    ruling!(
        "Family's Favor",
        "Neither the triggered ability that puts a shield counter on a target attacking creature nor the triggered ability that removes a shield counter from it are optional."
    );
    supported("Family's Favor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Family's Favor");
    let bears = old(&mut t, P0, "Grizzly Bears");
    let from = mark(&t);
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::SHIELD), 1);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.counters(bears, counters::SHIELD), 0);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(yes_nos(&t, from).is_empty());
}

// ---------------------------------------------------------------------------
// Last known information
// ---------------------------------------------------------------------------

#[test]
fn foot_chopper_uses_the_sacrificed_creatures_last_known_power() {
    cr!("608.2h");
    ruling!(
        "Foot Chopper",
        "Use the sacrificed creature's power as it last existed on the battlefield to determine how many cards to draw with Foot Chopper's third ability."
    );
    supported("Foot Chopper");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.enter(P0, "Foot Chopper");
    t.g.flush_events();
    t.resolve_all();
    let ninja = crate::r_s01_common::with_subtype(&t, P0, "Ninja")[0];
    t.g.objects[ninja.0 as usize].entered_turn = 0;
    t.g.objects[ninja.0 as usize].summoning_sick = false;
    // Giant Growth: 4/4 until end of turn.
    cast_resolve(&mut t, P0, "Giant Growth", &[o(ninja)]);
    yes(&mut t, P0, true);
    let hand = t.hand_size(P0);
    combat(&mut t, &[(ninja, pl(P1))], &[]);
    assert_eq!(t.life(P1), 16);
    assert!(!t.on_battlefield(ninja));
    assert_eq!(t.hand_size(P0), hand + 4);
}

#[test]
fn the_gitrog_uses_last_known_power_and_its_lands_arent_land_plays() {
    cr!("305.4", "608.2h");
    ruling!(
        "The Gitrog, Ravenous Ride",
        "Use the power of the sacrificed creature as it last existed on the battlefield to determine the value of X."
    );
    ruling!(
        "The Gitrog, Ravenous Ride",
        "Putting one or more land cards onto the battlefield with The Gitrog's triggered ability doesn't count as playing a land. It can put land cards onto the battlefield even if you've already played your land for the turn."
    );
    supported("The Gitrog, Ravenous Ride");
    for played_before in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        if played_before {
            let land = t.hand(P0, "Swamp");
            t.play_land(P0, land).expect("land play");
        }
        let gitrog = old(&mut t, P0, "The Gitrog, Ravenous Ride");
        let bears = old(&mut t, P0, "Grizzly Bears");
        t.answer_choose(P0, &[o(bears)]);
        activate_text(&mut t, P0, gitrog, "Saddle").expect("saddle");
        t.resolve_all();
        // Giant Growth on the saddler: power 5 as it's sacrificed.
        cast_resolve(&mut t, P0, "Giant Growth", &[o(bears)]);
        t.g.players[P0.idx()].hand.clear();
        let forests: Vec<ObjectId> = (0..5).map(|_| t.hand(P0, "Forest")).collect();
        yes(&mut t, P0, true);
        t.answer_choose(P0, &[o(bears)]);
        t.answer_choose(P0, &forests.iter().map(|f| o(*f)).collect::<Vec<_>>());
        combat(&mut t, &[(gitrog, pl(P1))], &[]);
        assert!(t.in_graveyard(P0, "Grizzly Bears"));
        // Drew 5 (X = 5) and put the 5 Forests onto the battlefield.
        let put = forests
            .iter()
            .filter(|f| t.zone(**f) == mtg_engine::object::Zone::Battlefield)
            .count();
        assert_eq!(put, 5, "{played_before} {}", t.dump_log());
        assert_eq!(t.hand_size(P0), 5, "{played_before}");
        // They weren't land plays: without an earlier land play, one is still available.
        t.set_step(P0, Step::PostcombatMain);
        let land = t.hand(P0, "Island");
        assert_eq!(can_play_land(&mut t, P0, land), !played_before);
    }
}

// ---------------------------------------------------------------------------
// Korvold
// ---------------------------------------------------------------------------

/// P0 sacrifices permanents of all six permanent types this turn (Ornithopter is an
/// artifact and a creature).
fn sacrifice_six_types(t: &mut TestGame) {
    let ids = [
        t.battlefield(P0, "Ornithopter"),
        t.battlefield(P0, "Glorious Anthem"),
        t.battlefield(P0, "Forest"),
        t.battlefield(P0, "Jace Beleren"),
        t.battlefield(P0, "Invasion of Tarkir // Defiant Thundermaw"),
    ];
    for id in ids {
        t.g.sacrifice(id, P0);
    }
    t.g.flush_events();
}

#[test]
fn korvold_cost_reduction_doesnt_reduce_colored_mana_or_its_mana_value() {
    cr!("601.2f", "202.3", "118.7d");
    ruling!(
        "Korvold, Gleeful Glutton",
        "Korvold's cost-reduction ability doesn't affect the colored mana in its mana cost. It can't reduce its cost to less than {B}{R}{G}."
    );
    ruling!(
        "Korvold, Gleeful Glutton",
        "The cost-reduction ability of Korvold, Gleeful Glutton doesn't change its mana cost or mana value, only the total cost you pay. Specifically, the mana value of Korvold, Gleeful Glutton is always 8."
    );
    supported("Korvold, Gleeful Glutton");
    // Six types sacrificed: {1} less for each would be {6} less, but only the {5} goes.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    sacrifice_six_types(&mut t);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    let k = t.hand(P0, "Korvold, Gleeful Glutton");
    assert!(t.cast(P0, k).try_go().is_err(), "{{B}}{{R}} isn't enough");
    t.lands(P0, "Forest", 1);
    let spell = t.cast(P0, k).go();
    assert_eq!(t.g.obj(spell).chars.mana_value(), 8);
    t.resolve_all();
    let korvold = t.named_on_battlefield("Korvold, Gleeful Glutton")[0];
    assert_eq!(t.g.obj(korvold).chars.mana_value(), 8);
    // Control: without the sacrifices it costs all eight.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    let k = t.hand(P0, "Korvold, Gleeful Glutton");
    assert!(t.cast(P0, k).try_go().is_err());
}

#[test]
fn korvold_counts_only_permanent_types() {
    cr!("110.4", "205.4a", "608.2h");
    ruling!(
        "Korvold, Gleeful Glutton",
        "The permanent types are artifact, battle, creature, enchantment, land, and planeswalker. Things like \"legendary\" or \"snow\" are supertypes, not types. \"Token\" is not a type."
    );
    // (graveyard, X)
    let cases: [(&[&str], usize); 3] = [
        (
            &[
                "Ornithopter",
                "Glorious Anthem",
                "Forest",
                "Jace Beleren",
                "Invasion of Tarkir // Defiant Thundermaw",
            ],
            6,
        ),
        // Legendary and snow are supertypes; instants and sorceries aren't permanents.
        (
            &[
                "Isamaru, Hound of Konda",
                "Snow-Covered Forest",
                "Lightning Bolt",
                "Shock",
            ],
            2,
        ),
        (&["Lightning Bolt"], 0),
    ];
    for (gy, x) in cases {
        let mut t = TestGame::new(2);
        let k = old(&mut t, P0, "Korvold, Gleeful Glutton");
        for n in gy {
            t.graveyard(P0, n);
        }
        let hand = t.hand_size(P0);
        combat(&mut t, &[(k, pl(P1))], &[]);
        assert_eq!(t.counters(k, counters::PLUS1), x as u32, "{gy:?}");
        assert_eq!(t.hand_size(P0), hand + x, "{gy:?}");
    }
    // A token the trigger could see as it resolves would be a creature token: it never
    // reaches the graveyard as a card (it ceases to exist), so it adds nothing.
    let mut t = TestGame::new(2);
    let k = old(&mut t, P0, "Korvold, Gleeful Glutton");
    t.graveyard(P0, "Lightning Bolt");
    let tok = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    destroy(&mut t, tok);
    let hand = t.hand_size(P0);
    combat(&mut t, &[(k, pl(P1))], &[]);
    assert_eq!(t.counters(k, counters::PLUS1), 0);
    assert_eq!(t.hand_size(P0), hand);
}

// ---------------------------------------------------------------------------
// Other cards of the group
// ---------------------------------------------------------------------------

#[test]
fn mondo_gecko_becomes_only_the_chosen_color_and_keeps_earlier_hexproof() {
    cr!("105.3", "702.11d", "613.1e");
    ruling!(
        "Mondo Gecko",
        "Mondo Gecko's first ability causes it to become only the chosen color. It will no longer be the color or colors it was previously. (It will still have hexproof from any other colors chosen that turn.)"
    );
    supported("Mondo Gecko");
    let mut t = TestGame::new(2);
    let gecko = old(&mut t, P0, "Mondo Gecko");
    assert_eq!(t.obj_now(gecko).chars.colors, ColorSet::single(Color::Blue));
    let pick = |c: Color| Color::ALL.iter().position(|x| *x == c).unwrap();
    for color in [Color::Red, Color::Green] {
        t.hand(P0, "Island");
        t.lands(P0, "Wastes", 1);
        t.answer(P0, DecisionKind::Option, Answer::Index(pick(color)));
        activate_text(&mut t, P0, gecko, "Discard a card").expect("activate");
        t.resolve_all();
        assert_eq!(t.obj_now(gecko).chars.colors, ColorSet::single(color));
    }
    // Hexproof from red (chosen first) and green (chosen second); not from blue.
    assert!(!spell_targets(&mut t, P1, "Shock").contains(&o(gecko)));
    assert!(!spell_targets(&mut t, P1, "Giant Growth").contains(&o(gecko)));
    assert!(spell_targets(&mut t, P1, "Unsummon").contains(&o(gecko)));
}

#[test]
fn inquisitor_eisenhorn_reveals_only_the_first_of_several_draws() {
    cr!("121.2", "121.1");
    ruling!(
        "Inquisitor Eisenhorn",
        "Multiple card draws are always treated as a sequence of individual card draws. For example, if you haven't drawn any cards yet during a turn and cast a spell that instructs you to draw three cards, you'll draw them one at a time. Only the first card drawn this way may be revealed with Inquisitor Eisenhorn's ability."
    );
    supported("Inquisitor Eisenhorn");
    // (library top first, Cherubaels)
    for (top, n) in [
        (["Island", "Shock", "Lightning Bolt"], 0),
        (["Shock", "Island", "Island"], 1),
    ] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.battlefield(P0, "Inquisitor Eisenhorn");
        stack_library(&mut t, P0, &top);
        for _ in 0..3 {
            yes(&mut t, P0, true);
        }
        let from = mark(&t);
        t.g.draw_cards(P0, 3);
        t.g.flush_events();
        t.settle();
        t.resolve_all();
        // Asked once (for the first card), whatever it was.
        assert_eq!(yes_nos(&t, from).len(), 1, "{top:?}");
        assert_eq!(t.named_on_battlefield("Cherubael").len(), n, "{top:?}");
    }
}

#[test]
fn sturmgeist_power_works_in_every_zone() {
    cr!("604.3", "208.2");
    ruling!(
        "Sturmgeist",
        "The ability that defines Sturmgeist's power and toughness works in all zones, not just the battlefield."
    );
    supported("Sturmgeist");
    let mut t = TestGame::new(2);
    t.g.players[P0.idx()].hand.clear();
    let in_hand = t.hand(P0, "Sturmgeist");
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    t.g.recompute();
    assert_eq!(t.pt(in_hand), (3, 3));
    let in_gy = t.graveyard(P0, "Sturmgeist");
    t.g.recompute();
    assert_eq!(t.pt(in_gy), (3, 3));
    let in_exile = t.exile(P0, "Sturmgeist");
    t.hand(P0, "Island");
    t.g.recompute();
    assert_eq!(t.pt(in_exile), (4, 4));
    assert_eq!(t.pt(in_gy), (4, 4));
}

#[test]
fn arm_mounted_anchor_discards_a_pirate_or_two_cards() {
    cr!("608.2c", "701.9a");
    ruling!(
        "Arm-Mounted Anchor",
        "You can discard either one Pirate card or two cards which may or may not be Pirates. If you really want to, you can discard two Pirate cards. (Admiral Brass will remember this.)"
    );
    supported("Arm-Mounted Anchor");
    // (hand before the two Islands drawn, whether to discard a Pirate card, the cards
    // discarded). Swab Goblin is a Pirate; Fortune Thief and the Bears aren't.
    let cases: [(&[&str], bool, &[&str]); 4] = [
        (&["Fortune Thief", "Swab Goblin"], true, &["Swab Goblin"]),
        (
            &["Fortune Thief", "Swab Goblin"],
            false,
            &["Fortune Thief", "Swab Goblin"],
        ),
        (
            &["Swab Goblin", "Swab Goblin"],
            false,
            &["Swab Goblin", "Swab Goblin"],
        ),
        (
            &["Fortune Thief", "Grizzly Bears"],
            false,
            &["Fortune Thief", "Grizzly Bears"],
        ),
    ];
    for (hand, pirate, discard) in cases {
        let mut t = TestGame::new(2);
        let bears = old(&mut t, P0, "Grizzly Bears");
        let anchor = t.battlefield(P0, "Arm-Mounted Anchor");
        t.g.attach(anchor, o(bears));
        t.g.players[P0.idx()].hand.clear();
        let in_hand: Vec<ObjectId> = hand.iter().map(|n| t.hand(P0, n)).collect();
        stack_library(&mut t, P0, &["Island", "Island"]);
        yes(&mut t, P0, pirate);
        let picks: Vec<Entity> = if pirate {
            vec![o(in_hand[1])]
        } else {
            in_hand.iter().map(|c| o(*c)).collect()
        };
        t.answer_choose(P0, &picks);
        let from = mark(&t);
        combat(&mut t, &[(bears, pl(P1))], &[]);
        assert_eq!(t.life(P1), 16, "{hand:?}");
        let mut got = crate::r_s04_common::graveyard_names(&t, P0);
        let mut want: Vec<String> = discard.iter().map(|s| s.to_string()).collect();
        got.sort();
        want.sort();
        assert_eq!(
            got,
            want,
            "{hand:?} {:?}",
            crate::r_s01_common::asked_since(&t, from)
        );
        assert_eq!(t.hand_size(P0), 4 - discard.len(), "{hand:?}");
    }
}

#[test]
fn the_ravens_warning_doesnt_reveal_the_card_from_outside_the_game() {
    cr!("400.11a", "714.2b");
    ruling!(
        "The Raven's Warning",
        "You don't have to reveal the card you bring into the game via the chapter III ability."
    );
    supported("The Raven's Warning");
    let mut t = TestGame::new(2);
    let side =
        t.g.add_to_sideboard(P0, vec![mtg_engine::card::card("Grizzly Bears")]);
    let saga = t.battlefield(P0, "The Raven's Warning");
    t.g.add_counters(o(saga), "lore", 2, None);
    t.resolve_all();
    yes(&mut t, P0, true);
    t.answer_choose(P0, &[o(side[0])]);
    t.g.add_counters(o(saga), "lore", 1, None);
    t.g.flush_events();
    t.resolve_all();
    let top = t.g.library_top(P0).expect("a library");
    assert_eq!(t.g.obj(top).chars.name, "Grizzly Bears");
    assert!(t.g.reveals.revealed.is_empty());
    assert!(!logged(&t, "reveals"));
}

#[test]
fn tomebound_lich_draws_and_discards_with_nothing_in_between() {
    cr!("608.2c", "603.3");
    ruling!(
        "Tomebound Lich",
        "You draw a card and discard a card all while Tomebound Lich’s triggered ability is resolving. Nothing can happen between the two, and no player may choose to take actions."
    );
    supported("Tomebound Lich");
    let mut t = TestGame::new(2);
    // Sheoldred, the Apocalypse: "Whenever you draw a card, you gain 2 life."
    t.battlefield(P0, "Sheoldred, the Apocalypse");
    let lich = old(&mut t, P0, "Tomebound Lich");
    t.hand(P0, "Island");
    // At the discard choice: P0's life, Sheoldred triggers waiting on the stack, the hand.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.to_lowercase().contains("discard")),
        |g| {
            let waiting = g
                .stack
                .iter()
                .filter(|id| {
                    g.obj(**id).stack.as_ref().is_some_and(|si| {
                        matches!(&si.kind, mtg_engine::object::StackKind::Triggered { ability, .. }
                            if ability.text.contains("you gain 2 life"))
                    })
                })
                .count();
            (g.player(P0).life, waiting, g.player(P0).hand.len())
        },
    );
    let hand = t.hand_size(P0);
    combat(&mut t, &[(lich, pl(P1))], &[]);
    // The discard was chosen right after the draw, with the Lich's ability still
    // resolving: the card drawn is in hand, but Sheoldred's trigger for that draw wasn't
    // on the stack yet (nothing happened in between). 20 + 1 (lifelink).
    let seen = seen.lock().unwrap().clone();
    assert_eq!(seen, vec![(21, 0, hand + 1)]);
    // Afterwards Sheoldred's trigger resolved.
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.graveyard_size(P0), 1);
}

/// Whether P1's declaration of blockers `decl` would be legal now (CR 509.1b-c).
fn legal_blocks(t: &mut TestGame, decl: &[(ObjectId, ObjectId)]) -> bool {
    t.g.recompute();
    let defenders =
        t.g.combat
            .as_ref()
            .map(|c| c.defending_players.clone())
            .unwrap_or_default();
    let options = mtg_engine::combat::block_options(&t.g, &defenders);
    mtg_engine::combat::block_declaration_legal(&t.g, &options, decl)
}

#[test]
fn driven_and_despair_trample_and_menace_twice_are_redundant() {
    cr!("702.19g", "702.111c", "702.19b", "702.111b");
    ruling!(
        "Driven // Despair",
        "Multiple instances of trample or menace are redundant."
    );
    // War Mammoth (3/3 trample) gets a second trample from Driven: it still assigns lethal
    // damage (2) to its blocker first and only the excess (1) to P1.
    let mut t = TestGame::new(2);
    let mammoth = old(&mut t, P0, "War Mammoth");
    let bears = old(&mut t, P1, "Grizzly Bears");
    let c = t.hand(P0, "Driven // Despair");
    mana_all(&mut t, P0, 2);
    let m = crate::r_p039_common::cast_methods(&mut t, P0, c);
    assert_eq!(m.len(), 1);
    t.cast(P0, c).method(m[0].clone()).go();
    t.resolve_all();
    let tramples = t
        .obj_now(mammoth)
        .chars
        .keywords_of(mtg_engine::keywords::KeywordKind::Trample)
        .count();
    assert_eq!(tramples, 2);
    attack_with(&mut t, &[(mammoth, pl(P1))]);
    let from = mark(&t);
    t.answer(P0, DecisionKind::Damage, Answer::Numbers(vec![2, 1]));
    crate::r_s01_common::block_and_finish(&mut t, P1, &[(bears, mammoth)]);
    assert_eq!(lethal_offers(&t, from), vec![vec![2]]);
    assert_eq!(t.life(P1), 19);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Boggart Brute (3/2 menace) gets a second menace from Despair: two blockers are
    // still enough, one still isn't.
    let mut t = TestGame::new(2);
    let brute = old(&mut t, P0, "Boggart Brute");
    let a = old(&mut t, P1, "Grizzly Bears");
    let b = old(&mut t, P1, "Grizzly Bears");
    let c = t.graveyard(P0, "Driven // Despair");
    mana_all(&mut t, P0, 2);
    let m = crate::r_p039_common::cast_methods(&mut t, P0, c);
    assert_eq!(m.len(), 1);
    t.cast(P0, c).method(m[0].clone()).go();
    t.resolve_all();
    let menaces = t
        .obj_now(brute)
        .chars
        .keywords_of(mtg_engine::keywords::KeywordKind::Menace)
        .count();
    assert_eq!(menaces, 2);
    attack_with(&mut t, &[(brute, pl(P1))]);
    assert!(!legal_blocks(&mut t, &[(a, brute)]));
    assert!(legal_blocks(&mut t, &[(a, brute), (b, brute)]));
}

#[test]
fn thopter_spy_network_triggers_once_per_player_dealt_damage() {
    cr!("510.2", "603.2c");
    ruling!(
        "Thopter Spy Network",
        "The last ability will trigger, at most, once per combat damage step per player."
    );
    // Three players: two artifact creatures hit P1, one hits P2: two triggers, two cards.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Thopter Spy Network");
    let a = old(&mut t, P0, "Esper Sentinel");
    let b = old(&mut t, P0, "Esper Sentinel");
    let c = old(&mut t, P0, "Esper Sentinel");
    let hand = t.hand_size(P0);
    attack_with(&mut t, &[(a, pl(P1)), (b, pl(P1)), (c, pl(P2))]);
    to_damage(&mut t);
    assert_eq!((t.life(P1), t.life(P2)), (18, 19));
    assert_eq!(triggers_on_stack(&t, "draw a card"), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn edric_lets_each_creatures_controller_draw_in_multiplayer() {
    cr!("510.2", "603.2");
    ruling!(
        "Edric, Spymaster of Trest",
        "When a creature you control deals combat damage to one of your opponents, you may draw a card."
    );
    // Four players, P0's Edric. P0's creature hitting P2 lets P0 draw; on P1's turn, P1's
    // creature hitting P2 (P0's opponent) lets P1 draw; P1's creature hitting P0 doesn't.
    let mut t = TestGame::new(4);
    old(&mut t, P0, "Edric, Spymaster of Trest");
    let mine = old(&mut t, P0, "Grizzly Bears");
    yes(&mut t, P0, true);
    let hand = t.hand_size(P0);
    combat(&mut t, &[(mine, pl(P2))], &[]);
    assert_eq!(t.hand_size(P0), hand + 1);
    t.advance_to(P1, Step::PrecombatMain);
    let a = old(&mut t, P1, "Grizzly Bears");
    let b = old(&mut t, P1, "Hill Giant");
    for _ in 0..2 {
        yes(&mut t, P1, true);
    }
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    combat(&mut t, &[(a, pl(P2)), (b, pl(P0))], &[]);
    assert_eq!((t.life(P0), t.life(P2)), (17, 16));
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (h0, h1 + 1));
}
