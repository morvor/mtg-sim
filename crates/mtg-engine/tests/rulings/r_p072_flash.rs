//! Rulings batch P072 — casting spells as though they had flash (CR 601.3b, 702.8): what
//! such permissions do and don't change, "the next [kind of] spell you cast this turn"
//! effects (CR 611.2f), and Tidal Barracuda's restriction (CR 101.2).

use crate::r_s01_common::supported;
use crate::r_s02_common::{can_activate, can_cast, can_play_land, destroy};
use crate::r_s06_common::activate_containing;
use crate::r_s21_common::castable;
use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn can_suspend(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    t.g.legal_actions(p).iter().any(
        |a| matches!(a, Action::Special(SpecialAction::Suspend { card: c }) if *c == card),
    )
}

/// Casts `card` (with lands for its cost) and resolves the stack.
fn cast_and_resolve(t: &mut TestGame, p: PlayerId, card: ObjectId) {
    let name = t.obj_now(card).chars.name.to_string();
    lands_for_cost(t, p, &name);
    t.g.turn.priority = Some(p);
    t.cast(p, card).go();
    t.resolve_all();
}

#[test]
fn flash_permissions_dont_change_sorcery_speed_activations() {
    cr!("601.3b", "602.5d", "702.6a");
    ruling!(
        "Final-Word Phantom",
        "Final-Word Phantom’s last ability applies only to casting spells. It does not, for example, change when you may activate abilities that can be activated “only as a sorcery.”"
    );
    ruling!(
        "High Fae Trickster",
        "High Fae Trickster's last ability applies only to casting spells. It does not, for example, change when you may activate abilities that can be activated \"only as a sorcery.\""
    );
    ruling!(
        "Hypersonic Dragon",
        "Hypersonic Dragon’s last ability has no effect on abilities that you can activate “any time you could cast a sorcery.”"
    );
    ruling!(
        "Borne Upon a Wind",
        "This applies only to casting spells. It does not, for example, change when you can activate abilities that can be activated \"only as a sorcery.\""
    );
    // (the permission's card, a card it lets P0 cast at instant speed, the step and whose
    // turn it is)
    for (perm, spell, active, step) in [
        ("Final-Word Phantom", "Grizzly Bears", P1, Step::End),
        ("High Fae Trickster", "Grizzly Bears", P0, Step::BeginningOfCombat),
        ("Hypersonic Dragon", "Divination", P0, Step::BeginningOfCombat),
        ("Borne Upon a Wind", "Grizzly Bears", P0, Step::BeginningOfCombat),
    ] {
        supported(perm);
        let mut t = TestGame::new(2);
        let splitter = t.battlefield(P0, "Bonesplitter");
        t.battlefield(P0, "Hill Giant");
        if perm == "Borne Upon a Wind" {
            cast_new(&mut t, P0, perm, &[]);
            t.resolve_all();
        } else {
            t.battlefield(P0, perm);
        }
        let card = t.hand(P0, spell);
        lands_for_cost(&mut t, P0, spell);
        t.lands(P0, "Wastes", 1);
        t.set_step(active, step);
        assert!(castable(&mut t, P0, card), "{perm}: casting");
        assert!(!can_activate(&mut t, P0, splitter), "{perm}: equip");
    }
}

#[test]
fn flash_permissions_apply_to_cards_in_any_zone() {
    cr!("601.3b", "702.34a");
    ruling!(
        "Hypersonic Dragon",
        "Hypersonic Dragon’s last ability applies to sorcery cards in any zone, provided something is allowing you to cast them. For example, you could cast a sorcery with flashback as though it had flash."
    );
    ruling!(
        "Alchemist's Refuge",
        "The last ability applies to nonland cards in any zone, provided something is allowing you to cast them. For example, you could cast a sorcery with flashback as though it had flash."
    );
    ruling!(
        "Winding Canyons",
        "The ability isn’t limited to creature cards in your hand. If an effect allows you to cast creature cards from some other zone, then Winding Canyons will allow you to do so as though they had flash."
    );
    let fb = CastMethod::Keyword(KeywordKind::Flashback);
    // Hypersonic Dragon: Deep Analysis (flashback sorcery) from the graveyard.
    let mut t = TestGame::new(2);
    let deep = t.graveyard(P0, "Deep Analysis");
    t.lands(P0, "Island", 2);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_cast(&mut t, P0, deep, fb.clone()));
    t.battlefield(P0, "Hypersonic Dragon");
    assert!(can_cast(&mut t, P0, deep, fb.clone()));
    // Alchemist's Refuge, activated.
    supported("Alchemist's Refuge");
    let mut t = TestGame::new(2);
    let deep = t.graveyard(P0, "Deep Analysis");
    let refuge = t.battlefield(P0, "Alchemist's Refuge");
    t.lands(P0, "Island", 3);
    t.lands(P0, "Forest", 1);
    activate_containing(&mut t, P0, refuge, "flash").unwrap();
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_cast(&mut t, P0, deep, fb.clone()));
    // Winding Canyons, activated: Squee, the Immortal from the graveyard.
    supported("Winding Canyons");
    let mut t = TestGame::new(2);
    let squee = t.graveyard(P0, "Squee, the Immortal");
    let canyons = t.battlefield(P0, "Winding Canyons");
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Wastes", 2);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!castable(&mut t, P0, squee));
    t.set_step(P0, Step::PrecombatMain);
    activate_containing(&mut t, P0, canyons, "flash").unwrap();
    t.resolve_all();
    lands_for_cost(&mut t, P0, "Squee, the Immortal");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(castable(&mut t, P0, squee));
}

#[test]
fn vedalken_orrery_any_time_you_could_cast_an_instant() {
    cr!("601.3b", "117.1a");
    ruling!(
        "Vedalken Orrery",
        "The controller of Vedalken Orrery can cast nonland artifact, creature, enchantment, and sorcery cards any time they could cast an instant. This includes on other players' turns and during the upkeep step, draw step, combat phase, and end step."
    );
    ruling!(
        "Alchemist's Refuge",
        "You can cast a nonland card with flash any time you could cast an instant."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vedalken Orrery");
    let cards: Vec<ObjectId> = ["Ornithopter", "Grizzly Bears", "Pacifism", "Divination"]
        .iter()
        .map(|n| t.hand(P0, n))
        .collect();
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Island", 2);
    t.lands(P0, "Forest", 2);
    t.battlefield(P1, "Hill Giant");
    let land = t.hand(P0, "Island");
    for (active, step) in [
        (P1, Step::Upkeep),
        (P1, Step::Draw),
        (P0, Step::DeclareAttackers),
        (P1, Step::End),
    ] {
        t.set_step(active, step);
        for c in &cards {
            assert!(castable(&mut t, P0, *c), "{step:?}");
        }
        assert!(!can_play_land(&mut t, P0, land));
    }
    // A flash card any time P0 could cast an instant, Refuge or not.
    let mut t = TestGame::new(2);
    let myr = t.hand(P0, "Shimmer Myr");
    t.lands(P0, "Wastes", 3);
    t.set_step(P1, Step::Upkeep);
    assert!(castable(&mut t, P0, myr));
}

#[test]
fn removing_the_permission_after_casting_doesnt_affect_the_spell() {
    cr!("601.3b", "601.2");
    ruling!(
        "Tidal Barracuda",
        "Once a player announces that they’re casting a spell, players can’t try to remove Tidal Barracuda to make it so that spell couldn’t be cast at that time. Spells cast as though they had flash will still resolve if able, even if Tidal Barracuda is no longer on the battlefield at that time."
    );
    ruling!(
        "Shimmer Myr",
        "Once you announce that you're casting a spell, players can't attempt to remove Shimmer Myr from the battlefield to make that casting illegal. Removing Shimmer My after you've cast a spell won't affect that spell."
    );
    // (permission, spell cast as though it had flash)
    for (perm, spell) in [
        ("Tidal Barracuda", "Grizzly Bears"),
        ("Shimmer Myr", "Ornithopter"),
    ] {
        supported(perm);
        let mut t = TestGame::new(2);
        let source = t.battlefield(P0, perm);
        t.set_step(P0, Step::BeginningOfCombat);
        let from = t.asked().len();
        cast_new(&mut t, P0, spell, &[]);
        // Nobody got priority during the casting.
        assert!(!t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, mtg_engine::decision::Decision::Priority { .. })));
        destroy(&mut t, source);
        t.resolve_all();
        assert_eq!(t.named_on_battlefield(spell).len(), 1, "{perm}");
    }
}

#[test]
fn shimmer_myr_doesnt_let_you_play_artifact_lands_at_instant_speed() {
    cr!("305.1", "601.3b");
    ruling!(
        "Shimmer Myr",
        "Shimmer Myr has no effect on artifact lands because they aren't cast as spells."
    );
    supported("Shimmer Myr");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shimmer Myr");
    let seat = t.hand(P0, "Seat of the Synod");
    let thopter = t.hand(P0, "Ornithopter");
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(castable(&mut t, P0, thopter));
    assert!(!can_play_land(&mut t, P0, seat));
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_play_land(&mut t, P0, seat));
}

#[test]
fn tidal_barracuda_restriction_and_permission() {
    cr!("101.2", "601.3b", "602.5");
    ruling!(
        "Tidal Barracuda",
        "If an effect says that an opponent may cast a spell during your turn, Tidal Barracuda’s restriction overrules that permission."
    );
    ruling!(
        "Tidal Barracuda",
        "Tidal Barracuda’s abilities apply only while Tidal Barracuda is on the battlefield. Notably, its first ability doesn’t apply to Tidal Barracuda as you’re casting it."
    );
    ruling!(
        "Tidal Barracuda",
        "Tidal Barracuda’s first ability doesn’t grant flash to any objects. It just changes when spells can be cast."
    );
    ruling!(
        "Tidal Barracuda",
        "Your opponents can still activate abilities during your turn."
    );
    supported("Tidal Barracuda");
    // Not while casting it: in combat, P0 can't cast it.
    let mut t = TestGame::new(2);
    let cuda = t.hand(P0, "Tidal Barracuda");
    t.lands(P0, "Island", 4);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!castable(&mut t, P0, cuda));
    // On the battlefield: P1 can't cast Lightning Bolt or (with its own Orrery) Divination
    // during P0's turn, but can activate abilities.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tidal Barracuda");
    t.battlefield(P1, "Vedalken Orrery");
    let bolt = t.hand(P1, "Lightning Bolt");
    let div = t.hand(P1, "Divination");
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Island", 3);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!castable(&mut t, P1, bolt));
    assert!(!castable(&mut t, P1, div));
    assert!(can_activate(&mut t, P1, sorcerer));
    // On P1's turn, P1 may cast Grizzly Bears as though it had flash, and it doesn't
    // have flash.
    t.set_step(P1, Step::BeginningOfCombat);
    let bears = t.hand(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 2);
    assert!(castable(&mut t, P1, bears));
    assert!(!t.obj_now(bears).chars.has_keyword(KeywordKind::Flash));
}

#[test]
fn next_sorcery_or_creature_spell_permissions_stack_onto_one_spell() {
    cr!("611.2f", "601.3b");
    ruling!(
        "Quicken",
        "If you cast multiple Quickens on the same turn, they'll all apply to the very next sorcery spell you cast."
    );
    ruling!(
        "Scout's Warning",
        "If you cast multiple Scout's Warnings on the same turn, they'll all apply to the very next creature spell you cast."
    );
    // (permission, two cards it lets P0 cast)
    for (perm, a, b) in [
        ("Quicken", "Divination", "Divination"),
        ("Scout's Warning", "Grizzly Bears", "Grizzly Bears"),
    ] {
        supported(perm);
        let mut t = TestGame::new(2);
        cast_new(&mut t, P0, perm, &[]);
        cast_new(&mut t, P0, perm, &[]);
        t.resolve_all();
        let first = t.hand(P0, a);
        let second = t.hand(P0, b);
        lands_for_cost(&mut t, P0, a);
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(castable(&mut t, P0, first), "{perm}");
        cast_and_resolve(&mut t, P0, first);
        lands_for_cost(&mut t, P0, b);
        assert!(!castable(&mut t, P0, second), "{perm}");
    }
}

#[test]
fn next_spell_permission_is_used_up_by_a_spell_cast_normally() {
    cr!("611.2f");
    ruling!(
        "Quicken",
        "You don't choose a sorcery card when Quicken resolves. Rather, this sets up a rule that is true for you until the turn ends or until you cast a sorcery card, even if you cast that sorcery at a time you normally could."
    );
    ruling!(
        "Scout's Warning",
        "You don't choose a creature card when Scout's Warning resolves. Rather, this sets up a rule that is true for you until the turn ends or until you cast a creature card, even if you cast that creature at a time you normally could."
    );
    for (perm, spell) in [("Quicken", "Divination"), ("Scout's Warning", "Grizzly Bears")] {
        let mut t = TestGame::new(2);
        cast_new(&mut t, P0, perm, &[]);
        t.resolve_all();
        let first = t.hand(P0, spell);
        let second = t.hand(P0, spell);
        // Cast in the main phase with an empty stack.
        cast_and_resolve(&mut t, P0, first);
        lands_for_cost(&mut t, P0, spell);
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(!castable(&mut t, P0, second), "{perm}");
        // Control: without the first spell, it would be castable.
        let mut t = TestGame::new(2);
        cast_new(&mut t, P0, perm, &[]);
        t.resolve_all();
        let second = t.hand(P0, spell);
        lands_for_cost(&mut t, P0, spell);
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(castable(&mut t, P0, second), "{perm}");
    }
}

#[test]
fn scouts_warning_lets_you_suspend_a_creature_at_instant_speed_until_you_cast_one() {
    cr!("702.62a", "611.2f");
    ruling!(
        "Scout's Warning",
        "After Scout's Warning resolves, you can suspend a creature card in your hand any time you can cast an instant. As soon as you actually cast a creature card, you lose this capability."
    );
    supported("Scout's Warning");
    let mut t = TestGame::new(2);
    let baloth = t.hand(P0, "Durkwood Baloth");
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_suspend(&mut t, P0, baloth));
    t.set_step(P0, Step::PrecombatMain);
    cast_new(&mut t, P0, "Scout's Warning", &[]);
    t.resolve_all();
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_suspend(&mut t, P0, baloth));
    cast_and_resolve(&mut t, P0, bears);
    assert!(!can_suspend(&mut t, P0, baloth));
}

#[test]
fn savage_summoning_effects() {
    cr!("611.2f", "601.3b", "101.2");
    ruling!(
        "Savage Summoning",
        "If you cast multiple Savage Summonings in succession, they’ll apply to the next creature spell you cast. That creature will enter the battlefield with that many additional +1/+1 counters on it."
    );
    ruling!(
        "Savage Summoning",
        "You don’t choose a creature card when Savage Summoning resolves. Rather, it applies to the next creature card you cast that turn, even if you cast it at a time when you normally could cast a creature and even if you cast it from a zone other than your hand."
    );
    ruling!(
        "Savage Summoning",
        "Spells that would counter Savage Summoning or the relevant creature spell can still be cast. They won’t counter the spell, but any additional effects they have will happen."
    );
    supported("Savage Summoning");
    // Two of them: two counters on the next creature.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Savage Summoning", &[]);
    cast_new(&mut t, P0, "Savage Summoning", &[]);
    t.resolve_all();
    let bears = t.hand(P0, "Grizzly Bears");
    cast_and_resolve(&mut t, P0, bears);
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    // Cast normally, from the graveyard (Squee): it still applies.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Savage Summoning", &[]);
    t.resolve_all();
    let squee = t.graveyard(P0, "Squee, the Immortal");
    cast_and_resolve(&mut t, P0, squee);
    let squee = t.named_on_battlefield("Squee, the Immortal")[0];
    assert_eq!(t.counters(squee, counters::PLUS1), 1);
    // Exclude ("Counter target creature spell. Draw a card.") on the creature spell: it
    // isn't countered, but P1 draws.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Savage Summoning", &[]);
    t.resolve_all();
    let bears = t.hand(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    let hand = t.hand_size(P1);
    cast_new(&mut t, P1, "Exclude", &[Entity::Object(spell)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}
