//! Rulings batch P089 — impact effects (a creature entering deals or causes damage or
//! life loss) and imprint-like effects (cards exiled with a permanent).

use crate::r_p089_common::*;
use crate::r_s05_common::{enter, move_to};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{StackKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Impact effects
// ---------------------------------------------------------------------------------------

#[test]
fn impact_tremors_triggers_for_each_creature_entering_together() {
    cr!("603.2", "603.6a");
    ruling!(
        "Impact Tremors",
        "If multiple creatures you control enter at the same time, Impact Tremors will trigger once for each of those creatures. This is true even if Impact Tremors enters at the same time as those creatures."
    );
    supported("Impact Tremors");
    supported("Replenish");
    // Two Soldier tokens at once: two triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Impact Tremors");
    cast_and_resolve(&mut t, P0, "Raise the Alarm", &[]);
    assert_eq!(t.life(P1), 18);
    // Replenish returns Impact Tremors and two enchantment creatures together.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Impact Tremors");
    t.graveyard(P0, "Nyx-Fleece Ram");
    t.graveyard(P0, "Nyx-Fleece Ram");
    cast_and_resolve(&mut t, P0, "Replenish", &[]);
    assert_eq!(t.named_on_battlefield("Nyx-Fleece Ram").len(), 2);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn corpse_knight_in_two_headed_giant() {
    cr!("810.9");
    ruling!(
        "Corpse Knight",
        "In a Two-Headed Giant game, Corpse Knight's ability causes the opposing team to lose 1 life twice."
    );
    supported("Corpse Knight");
    let mut t = TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]));
    t.battlefield(P0, "Corpse Knight");
    let before = t.life(P2);
    enter(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P2), before - 2);
    assert_eq!(t.life(P3), before - 2);
}

/// P0 has `source` (Terror of the Peaks or Warstorm Surge); a Hill Giant enters under P0's
/// control (its trigger targeting P1); in response it gets +3/+3 and, if `leave`, it then
/// leaves the battlefield. Returns P1's life after everything resolves.
fn giant_enters(source: &str, leave: bool) -> i32 {
    let mut t = TestGame::new(2);
    t.battlefield(P0, source);
    t.answer_targets(P0, &[player(P1)]);
    let giant = enter(&mut t, P0, "Hill Giant");
    assert_eq!(t.stack_len(), 1);
    cast_new(&mut t, P0, "Giant Growth", &[obj(giant)]);
    t.resolve();
    assert_eq!(t.pt(giant), (6, 6));
    if leave {
        move_to(&mut t, giant, Zone::Graveyard(P0));
    }
    t.resolve_all();
    t.life(P1)
}

#[test]
fn entering_creatures_power_is_checked_on_resolution_or_last_known() {
    cr!("608.2h");
    ruling!(
        "Terror of the Peaks",
        "The amount of damage Terror of the Peaks deals is the entering creature's power as the triggered ability resolves. If that creature leaves the battlefield before the ability resolves, use its power as it last existed on the battlefield."
    );
    ruling!(
        "Warstorm Surge",
        "The creature that entered deals damage equal to its current power to the targeted permanent or player. If it's no longer on the battlefield, its last known existence on the battlefield is checked to determine its power."
    );
    for source in ["Terror of the Peaks", "Warstorm Surge"] {
        supported(source);
        assert_eq!(giant_enters(source, false), 14, "{source}");
        assert_eq!(giant_enters(source, true), 14, "{source}");
    }
}

#[test]
fn pandemonium_trigger_is_controlled_by_its_controller() {
    cr!("603.3a", "603.3d");
    ruling!(
        "Pandemonium",
        "If player A controls Pandemonium when a creature enters under player B's control, player A controls the triggered ability even though player B chooses the target and chooses whether to have the creature deal damage"
    );
    supported("Pandemonium");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pandemonium");
    let from = t.asked().len();
    t.answer_targets(P1, &[player(P0)]);
    t.answer_yes(P1, true);
    enter(&mut t, P1, "Hill Giant");
    assert_eq!(t.stack_len(), 1);
    let trig = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(trig).controller, P0);
    // P1 chose the target.
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseTargets { .. })));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseTargets { .. })));
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    // P1, not P0, chose whether the creature deals damage.
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::YesNo { .. })));
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
    // P1 may decline.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pandemonium");
    t.answer_targets(P1, &[player(P0)]);
    t.answer_yes(P1, false);
    enter(&mut t, P1, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn pandemonium_uses_power_on_resolution_or_last_known() {
    cr!("608.2h");
    ruling!(
        "Pandemonium",
        "The triggered ability does not check the creature's power until the ability resolves."
    );
    supported("Pandemonium");
    for leave in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Pandemonium");
        t.answer_targets(P0, &[player(P1)]);
        t.answer_yes(P0, true);
        let giant = enter(&mut t, P0, "Hill Giant");
        cast_new(&mut t, P0, "Giant Growth", &[obj(giant)]);
        t.resolve();
        if leave {
            move_to(&mut t, giant, Zone::Graveyard(P0));
        }
        t.resolve_all();
        assert_eq!(t.life(P1), 14, "leave: {leave}");
    }
}

// ---------------------------------------------------------------------------------------
// Lumbering Battlement
// ---------------------------------------------------------------------------------------

#[test]
fn lumbering_battlement_returns_new_objects() {
    cr!("400.7", "610.3", "704.5m", "704.5n");
    ruling!(
        "Lumbering Battlement",
        "Auras attached to the exiled creatures will be put into their owners' graveyards. Any Equipment will become unattached and remain on the battlefield."
    );
    supported("Lumbering Battlement");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    put_counters(&mut t, bears, "+1/+1", 2);
    attach_new(&mut t, P0, "Holy Strength", obj(bears));
    let sword = attach_new(&mut t, P0, "Bonesplitter", obj(bears));
    t.answer_choose(P0, &[obj(bears), obj(giant)]);
    let lb = enter(&mut t, P0, "Lumbering Battlement");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert!(t.on_battlefield(sword));
    assert_eq!(t.obj_now(sword).attached_to, None);
    // +2/+2 for each of the two cards.
    assert_eq!(t.pt(lb), (8, 9));
    // It leaves: the cards return as new objects.
    move_to(&mut t, lb, Zone::Graveyard(P0));
    let back = t.g.current(bears);
    assert_eq!(t.zone(bears), Zone::Battlefield);
    assert_eq!(t.counters(back, "+1/+1"), 0);
    assert_eq!(t.pt(back), (2, 2));
    assert_eq!(t.obj_now(sword).attached_to, None);
    assert!(t.on_battlefield(giant));
}

#[test]
fn lumbering_battlement_gone_before_its_trigger_resolves() {
    cr!("610.3b");
    ruling!(
        "Lumbering Battlement",
        "If Lumbering Battlement leaves the battlefield before its triggered ability resolves, you can't exile any nontoken creatures."
    );
    supported("Lumbering Battlement");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    let lb = enter(&mut t, P0, "Lumbering Battlement");
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, lb, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn lumbering_battlement_counts_only_cards_still_exiled() {
    cr!("607.2a", "613.4c");
    ruling!(
        "Lumbering Battlement",
        "If a card you chose to exile leaves exile or somehow never moved to the exile zone in the first place, Lumbering Battlement doesn't get +2/+2 for that card."
    );
    supported("Lumbering Battlement");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.answer_choose(P0, &[obj(bears), obj(giant), obj(elves)]);
    let lb = enter(&mut t, P0, "Lumbering Battlement");
    t.resolve_all();
    assert_eq!(t.pt(lb), (10, 11));
    // One of them leaves exile.
    move_to(&mut t, bears, Zone::Graveyard(P0));
    t.g.recompute();
    assert_eq!(t.pt(lb), (8, 9));
}

// ---------------------------------------------------------------------------------------
// Exclusion Ritual
// ---------------------------------------------------------------------------------------

#[test]
fn exclusion_ritual_ends_when_the_card_leaves_exile() {
    cr!("607.2a", "101.2");
    ruling!(
        "Exclusion Ritual",
        "If the exiled card somehow leaves exile, the last ability will no longer have any effect."
    );
    supported("Exclusion Ritual");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    enter(&mut t, P0, "Exclusion Ritual");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    t.set_step(P1, Step::PrecombatMain);
    let other = t.hand(P1, "Grizzly Bears");
    crate::r_p076_common::mana(&mut t, P1, ManaType::G, 2);
    assert!(t.cast(P1, other).try_go().is_err());
    // The exiled Bears goes to P1's hand: P1 can cast Grizzly Bears again.
    move_to(&mut t, bears, Zone::Hand(P1));
    crate::r_p076_common::mana(&mut t, P1, ManaType::G, 2);
    assert!(t.cast(P1, other).try_go().is_ok());
}

#[test]
fn exclusion_ritual_and_a_commander() {
    cr!("903.9a", "607.2a");
    ruling!(
        "Exclusion Ritual",
        "In a Commander game, a player may put their commander in the command zone once exclusion ritual's triggered ability has finished resolving."
    );
    supported("Exclusion Ritual");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    );
    let cmd = t.battlefield(P1, "Isamaru, Hound of Konda");
    t.g.objects[cmd.0 as usize].is_commander = true;
    t.g.players[1].commander_names.push("Isamaru, Hound of Konda".into());
    t.answer_targets(P0, &[obj(cmd)]);
    t.answer_yes(P1, true);
    enter(&mut t, P0, "Exclusion Ritual");
    t.resolve_all();
    assert_eq!(t.zone(cmd), Zone::Command);
    // Nothing is exiled with it: P1 can cast their commander.
    t.set_step(P1, Step::PrecombatMain);
    crate::r_p076_common::mana(&mut t, P1, ManaType::W, 1);
    let c = t.g.current(cmd);
    assert!(t.cast(P1, c).try_go().is_ok());
}

// ---------------------------------------------------------------------------------------
// Dark Impostor
// ---------------------------------------------------------------------------------------

/// P0's Dark Impostor `imp` exiles `victim` with its activated ability.
fn impostor_exiles(t: &mut TestGame, imp: ObjectId, victim: ObjectId) {
    crate::r_p076_common::mana(t, P0, ManaType::B, 6);
    t.activate(P0, imp, 0, &[obj(victim)]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(victim), Zone::Exile);
}

/// Whether the permanent has an activated ability whose text contains `needle`.
fn has_ability(t: &TestGame, id: ObjectId, needle: &str) -> bool {
    ability_texts(t, id).iter().any(|a| a.contains(needle))
}

#[test]
fn each_dark_impostor_has_only_its_own_cards_abilities() {
    cr!("607.2a", "613.1f");
    ruling!(
        "Dark Impostor",
        "If you control more than one Dark Impostor, each will have only the activated abilities of creature cards exiled with that specific Dark Impostor."
    );
    supported("Dark Impostor");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Dark Impostor");
    let b = t.battlefield(P0, "Dark Impostor");
    let tim = t.battlefield(P1, "Prodigal Sorcerer");
    let elves = t.battlefield(P1, "Llanowar Elves");
    impostor_exiles(&mut t, a, tim);
    impostor_exiles(&mut t, b, elves);
    assert!(has_ability(&t, a, "damage to any target"));
    assert!(!has_ability(&t, a, "Add {G}"));
    assert!(has_ability(&t, b, "Add {G}"));
    assert!(!has_ability(&t, b, "damage to any target"));
}

#[test]
fn dark_impostor_returning_is_a_new_object() {
    cr!("400.7", "607.2a");
    ruling!(
        "Dark Impostor",
        "Once Dark Impostor leaves the battlefield, it will no longer have the activated abilities of the creature cards exiled with it."
    );
    supported("Dark Impostor");
    let mut t = TestGame::new(2);
    let imp = t.battlefield(P0, "Dark Impostor");
    let tim = t.battlefield(P1, "Prodigal Sorcerer");
    impostor_exiles(&mut t, imp, tim);
    assert!(has_ability(&t, imp, "damage to any target"));
    // Flickered: a new Dark Impostor.
    let back = move_to(&mut t, imp, Zone::Exile).unwrap();
    let back = move_to(&mut t, back, Zone::Battlefield).unwrap();
    assert!(t.on_battlefield(back));
    assert!(!has_ability(&t, back, "damage to any target"));
    assert_eq!(t.zone(tim), Zone::Exile);
}

// ---------------------------------------------------------------------------------------
// Arcane Bombardment
// ---------------------------------------------------------------------------------------

/// P0 casts Shock at P1 (resolving everything); copies are cast when offered, targeting
/// P1.
fn shock_p1(t: &mut TestGame) {
    for _ in 0..4 {
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[player(P1)]);
    }
    cast_new(t, P0, "Shock", &[player(P1)]);
    t.resolve_all();
    t.clear_answers();
}

#[test]
fn arcane_bombardment_copies_only_cards_exiled_with_this_object() {
    cr!("607.2a", "400.7");
    ruling!(
        "Arcane Bombardment",
        "If Arcane Bombardment leaves the battlefield, the exiled cards remain in exile. If it returns to the battlefield or you play another copy of it, you won't get to cast copies of the cards previously exiled."
    );
    supported("Arcane Bombardment");
    let mut t = TestGame::new(2);
    let ab = t.battlefield(P0, "Arcane Bombardment");
    t.graveyard(P0, "Lightning Bolt");
    shock_p1(&mut t);
    // Shock (2) and the copy of Lightning Bolt (3).
    assert_eq!(t.life(P1), 15);
    assert!(t.in_exile("Lightning Bolt"));
    // It leaves; the Bolt stays exiled. A new Bombardment on a later turn copies only what
    // it exiled (the Shock from the graveyard).
    move_to(&mut t, ab, Zone::Graveyard(P0));
    assert!(t.in_exile("Lightning Bolt"));
    t.battlefield(P0, "Arcane Bombardment");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    shock_p1(&mut t);
    // Shock (2) and one copy of Shock (2): no Bolt.
    assert_eq!(t.life(P1), 11);
}

#[test]
fn arcane_bombardment_x_is_zero() {
    cr!("107.3b", "601.2b");
    ruling!(
        "Arcane Bombardment",
        "If you cast a spell with {X} in its cost without paying its mana cost, you must choose 0 for X."
    );
    supported("Arcane Bombardment");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arcane Bombardment");
    t.graveyard(P0, "Blaze");
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    shock_p1(&mut t);
    assert!(t.in_exile("Blaze"));
    // Only Shock's 2 damage: Blaze's copy had X = 0.
    assert_eq!(t.life(P1), 18);
}

#[test]
fn arcane_bombardment_entering_after_the_first_spell() {
    cr!("603.2");
    ruling!(
        "Arcane Bombardment",
        "If you've already cast an instant or sorcery spell the turn that Arcane Bombardment enters the battlefield, it won't trigger until you cast an instant or sorcery spell on a later turn."
    );
    supported("Arcane Bombardment");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Lightning Bolt");
    shock_p1(&mut t);
    assert_eq!(t.life(P1), 18);
    t.battlefield(P0, "Arcane Bombardment");
    shock_p1(&mut t);
    assert_eq!(t.life(P1), 16);
    assert!(!t.in_exile("Lightning Bolt"));
}

#[test]
fn arcane_bombardment_triggering_spell_is_still_on_the_stack() {
    cr!("603.3", "608.2");
    ruling!(
        "Arcane Bombardment",
        "That spell won't be among the cards in your graveyard when you exile an instant or sorcery from it at random."
    );
    supported("Arcane Bombardment");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arcane Bombardment");
    let shock = cast_new(&mut t, P0, "Shock", &[player(P1)]);
    t.settle();
    // The trigger is above the Shock.
    assert_eq!(t.stack_len(), 2);
    assert!(t.g.obj(shock).zone == Zone::Stack);
    assert!(matches!(
        t.g.obj(*t.g.stack.last().unwrap()).stack.as_ref().map(|s| &s.kind),
        Some(StackKind::Triggered { .. })
    ));
    t.resolve_all();
    // Nothing was exiled: the Shock is in the graveyard.
    assert!(t.in_graveyard(P0, "Shock"));
    assert_eq!(t.life(P1), 18);
}
