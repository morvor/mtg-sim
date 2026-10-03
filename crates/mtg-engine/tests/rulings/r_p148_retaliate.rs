//! Rulings batch P148 — abilities that answer damage: "whenever a source an opponent
//! controls deals damage to you" (each damage event, the source's controller as the damage
//! is dealt or its last known information, CR 120, 603.2, 603.10), "destroy target creature
//! that dealt damage this turn", and what happens when the damage is lethal.

use crate::r_p148_common::*;
use crate::r_s04_common::cycle;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Number of triggered abilities on the stack from `source`.
fn triggers_from(t: &TestGame, source: ObjectId) -> usize {
    t.g.stack
        .iter()
        .filter(|id| {
            matches!(
                t.g.obj(**id).stack.as_deref().map(|s| &s.kind),
                Some(mtg_engine::object::StackKind::Triggered { source: s, .. }) if *s == source
            )
        })
        .count()
}

/// P1 attacks P0 with `attackers` (unblocked unless `blocks`), and the game stops in the
/// combat damage step with the combat damage triggers on the stack.
fn p1_attacks(t: &mut TestGame, attackers: &[ObjectId], blocks: &[(ObjectId, ObjectId)]) {
    to_combat(t, P1);
    attack_with(t, &at(P0, attackers));
    declare_blocks(t, P0, blocks);
    t.advance_to(P1, Step::CombatDamage);
    t.settle();
}

// --- Farsight Mask ----------------------------------------------------------------------------

#[test]
fn farsight_mask_triggers_for_each_instance_of_combat_damage() {
    cr!("510.4", "603.2", "702.4b");
    ruling!(
        "Farsight Mask",
        "Each instance of each creature’s combat damage is counted separately. If three creatures with double strike attack you and all of them are unblocked, you may draw up to six cards."
    );
    supported("Farsight Mask");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Farsight Mask");
    let aces: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P1, "Fencing Ace")).collect();
    for _ in 0..6 {
        t.answer_yes(P0, true);
    }
    to_combat(&mut t, P1);
    attack_with(&mut t, &at(P0, &aces));
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P0), 14);
    assert_eq!(t.hand_size(P0), 6);
}

#[test]
fn farsight_mask_triggers_each_time_the_same_source_deals_damage() {
    cr!("603.2");
    ruling!(
        "Farsight Mask",
        "This triggers each time a source an opponent controls deals damage to you. If the same source deals damage more than once in a turn, it triggers for each of those times."
    );
    let mut t = TestGame::new(2);
    let mask = t.battlefield(P0, "Farsight Mask");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.g.deal_damage(pyro, Entity::Player(P0), 1, false);
    t.settle();
    t.g.deal_damage(pyro, Entity::Player(P0), 1, false);
    t.settle();
    assert_eq!(triggers_from(&t, mask), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    // A source P0 controls doesn't trigger it.
    let own = t.battlefield(P0, "Prodigal Pyromancer");
    t.g.deal_damage(own, Entity::Player(P0), 1, false);
    t.settle();
    assert_eq!(triggers_from(&t, mask), 0);
}

#[test]
fn farsight_mask_must_be_untapped_when_damage_is_dealt_and_when_you_would_draw() {
    cr!("603.4");
    ruling!(
        "Farsight Mask",
        "Farsight Mask must be untapped both when the damage is dealt and when you would draw the card."
    );
    let mut t = TestGame::new(2);
    let mask = t.battlefield(P0, "Farsight Mask");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.answer_yes(P0, true);
    // Untapped as the damage is dealt, tapped before the ability resolves: no card.
    t.g.deal_damage(pyro, Entity::Player(P0), 1, false);
    t.settle();
    assert_eq!(triggers_from(&t, mask), 1);
    t.g.tap(mask);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    // Tapped as the damage is dealt: it doesn't trigger.
    t.g.deal_damage(pyro, Entity::Player(P0), 1, false);
    t.settle();
    assert_eq!(triggers_from(&t, mask), 0);
}

// --- Retaliator Griffin -----------------------------------------------------------------------

#[test]
fn retaliator_griffin_dealt_lethal_damage_at_the_same_time() {
    cr!("603.10a", "704.5g", "510.2");
    ruling!(
        "Retaliator Griffin",
        "If Retaliator Griffin is dealt lethal damage at the same time that you’re dealt damage by a source an opponent controls, Retaliator Griffin will be put into a graveyard before it would receive any counters."
    );
    supported("Retaliator Griffin");
    let mut t = TestGame::new(2);
    let griffin = t.battlefield(P0, "Retaliator Griffin");
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    t.answer_yes(P0, true);
    // The Dreadmaw assigns lethal damage to the blocking Griffin and the rest to P0.
    p1_attacks(&mut t, &[wurm], &[(griffin, wurm)]);
    assert_eq!(t.life(P0), 16);
    assert!(!t.on_battlefield(griffin));
    assert!(t.in_graveyard(P0, "Retaliator Griffin"));
    // Its ability still triggered.
    assert_eq!(triggers_from(&t, griffin), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Retaliator Griffin"));
}

#[test]
fn retaliator_griffin_spell_damage_comes_from_the_spell() {
    cr!("120.2", "603.2");
    ruling!(
        "Retaliator Griffin",
        "If a spell causes damage to be dealt, that spell will always identify the source of the damage. In most cases, the source is the spell itself."
    );
    let mut t = TestGame::new(2);
    let griffin = t.battlefield(P0, "Retaliator Griffin");
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.answer_yes(P0, true);
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.counters(griffin, counters::PLUS1), 3);
}

#[test]
fn retaliator_griffin_ability_damage_comes_from_its_source() {
    cr!("120.2", "603.2");
    ruling!(
        "Retaliator Griffin",
        "If an ability causes damage to be dealt, that ability will always identify the source of the damage. The ability itself is never the source."
    );
    let mut t = TestGame::new(2);
    let griffin = t.battlefield(P0, "Retaliator Griffin");
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.answer_yes(P0, true);
    t.activate(P1, pyro, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.counters(griffin, counters::PLUS1), 1);
}

#[test]
fn retaliator_griffin_uses_last_known_information_and_owners_of_cards() {
    cr!("120.2", "608.2h", "603.2");
    ruling!(
        "Retaliator Griffin",
        "If the permanent has left the battlefield by then, its last known information is used."
    );
    let mut t = TestGame::new(2);
    let griffin = t.battlefield(P0, "Retaliator Griffin");
    // The Pyromancer leaves the battlefield before its ability resolves.
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.answer_yes(P0, true);
    t.activate(P1, pyro, 0, &[Entity::Player(P0)]).unwrap();
    destroy(&mut t, pyro);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.counters(griffin, counters::PLUS1), 1);
    // A cycled Jund Sojourners (a card in a graveyard) owned by P1.
    supported("Jund Sojourners");
    let soj = t.hand(P1, "Jund Sojourners");
    t.lands(P1, "Mountain", 3);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.answer_yes(P0, true);
    cycle(&mut t, P1, soj, 0).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.counters(griffin, counters::PLUS1), 2);
}

// --- Flameblade Angel -------------------------------------------------------------------------

#[test]
fn flameblade_angel_triggers_for_each_recipient() {
    cr!("603.2", "702.19c");
    ruling!(
        "Flameblade Angel",
        "If a source deals damage to you and/or one or more permanents you control at the same time, Flameblade Angel’s last ability will trigger that many times."
    );
    supported("Flameblade Angel");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Flameblade Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    p1_attacks(&mut t, &[wurm], &[(bears, wurm)]);
    assert_eq!(t.life(P0), 16);
    assert_eq!(triggers_from(&t, angel), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn two_flameblade_angels_trade_damage_until_one_player_declines() {
    cr!("603.2", "603.5");
    ruling!(
        "Flameblade Angel",
        "This cycle will repeat until the game ends or one player declines to use the ability, putting an end to the carnage."
    );
    let mut t = TestGame::new(2);
    let a0 = t.battlefield(P0, "Flameblade Angel");
    let a1 = t.battlefield(P1, "Flameblade Angel");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    // P1's Angel answers the Bolt; P0's Angel answers that; P1's answers that; then P0
    // declines.
    t.answer_yes(P1, true);
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    t.answer_yes(P0, false);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 3 - 1);
    assert_eq!(t.life(P0), 20 - 2);
    let _ = (a0, a1);
}

// --- Darien, King of Kjeldor ------------------------------------------------------------------

#[test]
fn darien_controller_loses_before_the_tokens() {
    cr!("704.3", "704.5a");
    ruling!(
        "Darien, King of Kjeldor",
        "If you're dealt damage that causes your life total to become 0 or less, you lose the game before the tokens can be created."
    );
    supported("Darien, King of Kjeldor");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Darien, King of Kjeldor");
    t.g.players[0].life = 3;
    let bolt = t.hand(P1, "Lightning Bolt");
    t.lands(P1, "Mountain", 1);
    t.answer_yes(P0, true);
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert!(t.has_lost(P0));
    assert!(t
        .g
        .objects
        .iter()
        .all(|o| !(o.is_token() && o.chars.name.contains("Soldier"))));
}

// --- Avenging Arrow, Executioner's Swing --------------------------------------------------------

#[test]
fn avenging_arrow_doesnt_undo_the_damage() {
    cr!("120.3", "701.8a");
    ruling!(
        "Avenging Arrow",
        "Avenging Arrow doesn’t affect the damage the creature dealt. It isn’t retroactively prevented."
    );
    supported("Avenging Arrow");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    p1_attacks(&mut t, &[giant], &[]);
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 17);
    let arrow = t.hand(P0, "Avenging Arrow");
    t.lands(P0, "Plains", 3);
    t.cast(P0, arrow).target(giant).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P0), 17);
}

#[test]
fn avenging_arrow_doesnt_care_what_was_dealt_damage() {
    cr!("115.1", "601.2c");
    ruling!(
        "Avenging Arrow",
        "It doesn’t matter what the creature dealt damage to. If the creature dealt damage to another creature or a planeswalker, it doesn’t matter whether that creature or planeswalker is still on the battlefield."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let idle = t.battlefield(P0, "Grizzly Bears");
    to_combat(&mut t, P0);
    attack_with(&mut t, &at(P1, &[giant]));
    declare_blocks(&mut t, P1, &[(bears, giant)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert!(!t.on_battlefield(bears));
    let arrow = t.hand(P1, "Avenging Arrow");
    t.lands(P1, "Plains", 3);
    let from = t.asked().len();
    t.cast(P1, arrow).target(giant).go();
    // A creature that dealt no damage can't be targeted; the Hill Giant can.
    let offered = crate::r_s02_common::target_candidates(&t, P1, from);
    assert!(offered
        .iter()
        .all(|c| c.contains(&obj(giant)) && !c.contains(&obj(idle))));
    assert!(!offered.is_empty());
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn executioners_swing_between_first_strike_and_regular_damage() {
    cr!("510.4", "601.2c");
    ruling!(
        "Executioner's Swing",
        "You can’t choose a creature that hasn’t yet dealt damage during the turn as the target of Executioner’s Swing"
    );
    supported("Executioner's Swing");
    let mut t = TestGame::new(2);
    let ace = t.battlefield(P1, "Fencing Ace");
    let swing = t.hand(P0, "Executioner's Swing");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    to_combat(&mut t, P1);
    attack_with(&mut t, &at(P0, &[ace]));
    // It hasn't dealt damage yet.
    assert!(t.cast(P0, swing).target(ace).try_go().is_err());
    t.advance_to(P1, Step::FirstStrikeDamage);
    assert_eq!(t.life(P0), 19);
    let swing = t.g.current(swing);
    t.cast(P0, swing).target(ace).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Fencing Ace"));
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 19);
}
