//! Rulings batch P224 — suspect (CR 701.60) and suspend (CR 702.62).

use crate::r_s01_common::*;
use crate::r_s04_common::next_upkeep;
use crate::r_s08_common::actions_of;
use crate::r_s13_common::{commander, commander_game};
use mtg_engine::decision::{Action, Answer, Decision, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kwa::suspect_detain::is_suspected;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

fn suspected(t: &TestGame, id: ObjectId) -> bool {
    is_suspected(&t.g, t.g.current(id))
}

/// J. Jonah Jameson enters under P0's control and suspects `target`.
fn jjj_suspects(t: &mut TestGame, target: ObjectId) {
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.enter(P0, "J. Jonah Jameson");
    t.resolve_all();
    assert!(suspected(t, target));
}

#[test]
fn a_suspected_creature_that_loses_all_abilities_stays_suspected() {
    cr!("701.60b", "701.60c", "613.1f");
    ruling!(
        "J. Jonah Jameson",
        "If a suspected creature loses all abilities, it will lose menace and \"This creature can't block,\" but it won't stop being suspected."
    );
    supported("J. Jonah Jameson");
    supported("Turn to Frog");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    jjj_suspects(&mut t, bears);
    assert!(t.obj_now(bears).chars.has_keyword(KeywordKind::Menace));
    assert!(!t.g.can_block_at_all(t.g.current(bears)));
    // Turn to Frog: "Until end of turn, target creature loses all abilities and becomes a
    // blue Frog with base power and toughness 1/1."
    t.lands(P0, "Island", 2);
    let frog = t.hand(P0, "Turn to Frog");
    t.cast(P0, frog).target(bears).go();
    t.resolve_all();
    assert!(!t.obj_now(bears).chars.has_keyword(KeywordKind::Menace));
    assert!(t.g.can_block_at_all(t.g.current(bears)));
    assert!(suspected(&t, bears));
}

#[test]
fn suspecting_another_creature_doesnt_unsuspect_the_first() {
    cr!("701.60a", "701.60c");
    ruling!(
        "J. Jonah Jameson",
        "Though JJJ suspects only one creature when he enters, there is generally no limit to the number of creatures that can be suspected. Suspecting a new creature doesn't cause other creatures to stop being suspected."
    );
    supported("Reasonable Doubt");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    jjj_suspects(&mut t, bears);
    // Reasonable Doubt: "Counter target spell unless its controller pays {2}. Suspect up
    // to one target creature." At a Lightning Bolt P1 casts; P1 doesn't pay.
    t.lands(P1, "Mountain", 1);
    let b = t.hand(P1, "Lightning Bolt");
    let b = t.cast(P1, b).target(P0).go();
    t.lands(P0, "Island", 2);
    let rd = t.hand(P0, "Reasonable Doubt");
    t.cast(P0, rd)
        .target(Entity::Object(b))
        .target(Entity::Object(giant))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(suspected(&t, giant));
    assert!(suspected(&t, bears));
}

#[test]
fn reasonable_doubt_needs_a_spell_to_target() {
    cr!("601.2c", "115.1");
    ruling!(
        "Reasonable Doubt",
        "You can't cast Reasonable Doubt just to suspect a creature. There has to be a spell on the stack that's a legal target for Reasonable Doubt."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let rd = t.hand(P0, "Reasonable Doubt");
    assert!(t.cast(P0, rd).target(bears).try_go().is_err());
    t.clear_answers();
    assert!(t.in_hand(P0, "Reasonable Doubt"));
    assert!(!suspected(&t, bears));
}

#[test]
fn a_suspected_rune_brand_juggler_can_be_sacrificed_for_its_own_ability() {
    cr!("602.2b", "118.3", "701.60a");
    ruling!(
        "Rune-Brand Juggler",
        "If Rune-Brand Juggler is suspected, you can sacrifice it to pay for its own last ability."
    );
    supported("Rune-Brand Juggler");
    // Rune-Brand Juggler: "When this creature enters, suspect up to one target creature
    // you control. {3}{B}{R}, Sacrifice a suspected creature: Target creature gets -5/-5
    // until end of turn." It suspects itself.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let juggler = t.enter(P0, "Rune-Brand Juggler");
    // Its enters trigger targets the Juggler itself.
    t.answer_targets(P0, &[Entity::Object(juggler)]);
    t.resolve_all();
    assert!(suspected(&t, juggler));
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Mountain", 2);
    t.answer_choose(P0, &[Entity::Object(juggler)]);
    t.activate(P0, juggler, 0, &[Entity::Object(giant)])
        .expect("activation failed");
    assert!(!t.on_battlefield(juggler));
    assert!(t.in_graveyard(P0, "Rune-Brand Juggler"));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

/// Puts the real card `name` into `p`'s exile, suspended with `n` time counters.
fn suspended(t: &mut TestGame, p: PlayerId, name: &str, n: u32) -> ObjectId {
    let card = t.exile(p, name);
    t.g.add_counters(Entity::Object(card), counters::TIME, n, None);
    t.g.recompute();
    card
}

#[test]
fn a_suspended_card_not_cast_stays_exiled_and_is_no_longer_suspended() {
    cr!("702.62a", "702.62b", "702.62d");
    ruling!(
        "Suspended Sentence",
        "As the second triggered ability of suspend resolves, you may cast the card. Timing permissions based on the card's type are ignored. If you choose not to (or can't) cast the card, it remains exiled with no time counters on it, and it's no longer suspended."
    );
    supported("Suspended Sentence");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let card = suspended(&mut t, P0, "Suspended Sentence", 1);
    t.answer_yes(P0, false);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    assert_eq!(t.counters(card, counters::TIME), 0);
    assert!(t.in_exile("Suspended Sentence"));
    assert!(t.on_battlefield(t.named_on_battlefield("Grizzly Bears")[0]));
    // It's no longer suspended: nothing triggers at P0's next upkeep.
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.zone(card), Zone::Exile);
}

#[test]
fn arc_blade_suspends_itself_again_each_time_it_resolves() {
    cr!("702.62a", "702.62b", "702.62d");
    ruling!(
        "Arc Blade",
        "Each time you cast Arc Blade, it will suspend itself again when it resolves so you can cast it again three turns later."
    );
    supported("Arc Blade");
    // Arc Blade (sorcery): "Arc Blade deals 2 damage to any target. Exile Arc Blade with
    // three time counters on it." Cast during P0's upkeep from suspend.
    let mut t = TestGame::new(2);
    suspended(&mut t, P0, "Arc Blade", 1);
    for _ in 0..2 {
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    let blade = t.g.find_in_zone(Zone::Exile, "Arc Blade")[0];
    assert_eq!(t.counters(blade, counters::TIME), 3);
    // Three more upkeeps: cast again on the third.
    next_upkeep(&mut t, P0);
    t.resolve_all();
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    let blade = t.g.find_in_zone(Zone::Exile, "Arc Blade")[0];
    assert_eq!(t.counters(blade, counters::TIME), 3);
}

#[test]
fn dinosaurs_on_a_spaceship_last_counter_triggers_both_abilities_in_either_order() {
    cr!("702.62a", "603.3b");
    ruling!(
        "Dinosaurs on a Spaceship",
        "If this card is suspended, then when the last time counter is removed from it, both its last ability and the \"cast this spell\" part of the suspend ability will trigger. They can be put on the stack in either order."
    );
    supported("Dinosaurs on a Spaceship");
    // "Whenever a time counter is removed from this card while it's exiled, create a 2/2
    // red and white Dinosaur creature token with flying and haste."
    let mut tops = vec![];
    for first in [0usize, 1] {
        let mut t = TestGame::new(2);
        suspended(&mut t, P0, "Dinosaurs on a Spaceship", 1);
        t.answer(
            P0,
            DecisionKind::Order,
            Answer::Indices(vec![first, 1 - first]),
        );
        t.answer_yes(P0, false);
        let from = t.asked().len();
        next_upkeep(&mut t, P0);
        t.resolve();
        // The last counter is removed: both abilities trigger, and P0 orders them.
        assert_eq!(t.stack_len(), 2);
        let order = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::Order { .. }))
            .count();
        assert_eq!(order, 1);
        let top = *t.g.stack.last().unwrap();
        tops.push(
            triggers_on_stack(&t, "Dinosaur") == 1 && {
                let si = t.g.obj(top).stack.as_ref().unwrap();
                matches!(&si.kind, mtg_engine::object::StackKind::Triggered { ability, .. }
                if ability.text.contains("Dinosaur"))
            },
        );
        t.resolve_all();
        assert_eq!(with_subtype(&t, P0, "Dinosaur").len(), 1);
    }
    // Either one can be on top.
    tops.sort();
    assert_eq!(tops, vec![false, true]);
}

#[test]
fn a_commander_cant_be_suspended_from_the_command_zone() {
    cr!("702.62a", "903.8");
    ruling!(
        "Ith, High Arcanist",
        "In a Commander game where this card is your commander, you cannot suspend it from the Command zone."
    );
    supported("Ith, High Arcanist");
    let mut t = commander_game();
    let ith = commander(&mut t, P0, "Ith, High Arcanist");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Island", 1);
    let suspends = |t: &mut TestGame| {
        actions_of(t, P0)
            .iter()
            .filter(|a| matches!(a, Action::Special(SpecialAction::Suspend { .. })))
            .count()
    };
    assert_eq!(suspends(&mut t), 0);
    assert_eq!(t.zone(ith), Zone::Command);
    // From the hand, it can be suspended.
    t.hand(P0, "Ith, High Arcanist");
    assert_eq!(suspends(&mut t), 1);
}
