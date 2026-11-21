//! Rulings on support (CR 701.41): whose creatures it can target, one counter per target,
//! other targets of the same spell, and partially illegal targets.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::StackKind;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn support_and_another_target_of_the_spell_can_be_the_same_creature() {
    cr!("701.41a", "115.3");
    ruling!(
        "Press into Service",
        "If a spell with support has other abilities that target creatures, those abilities and the support ability can target the same creature."
    );
    supported("Press into Service");
    // Press into Service ({4}{R}): "Support 2. Gain control of target creature until end
    // of turn. Untap that creature. It gains haste until end of turn."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.objects[giant.0 as usize].tapped = true;
    t.lands(P0, "Mountain", 5);
    let spell = t.hand(P0, "Press into Service");
    t.cast(P0, spell).targets(&[giant.into()]).target(giant).go();
    t.resolve_all();
    let o = t.obj_now(giant);
    assert_eq!(o.controller, P0);
    assert!(!o.tapped);
    assert!(o.chars.has_keyword(KeywordKind::Haste));
    assert_eq!(t.counters(giant, counters::PLUS1), 1);
    assert_eq!(t.pt(giant), (4, 4));
}

#[test]
fn support_targets_creatures_you_dont_control_one_counter_each() {
    cr!("701.41a", "115.3");
    ruling!(
        "Relief Captain",
        "Support can target a creature you don't control."
    );
    ruling!(
        "Relief Captain",
        "You can't put more than one +1/+1 counter on any one target using the support action."
    );
    supported("Relief Captain");
    // Relief Captain: "When this creature enters, support 3."
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[mine.into(), theirs.into()]);
    t.enter(P0, "Relief Captain");
    t.resolve_all();
    assert_eq!(t.counters(mine, counters::PLUS1), 1);
    assert_eq!(t.counters(theirs, counters::PLUS1), 1);
    assert_eq!(t.obj_now(theirs).controller, P1);
    // Naming the same creature more than once isn't a legal choice: no creature gets
    // more than one counter.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[bears.into(), bears.into(), bears.into()]);
    t.enter(P0, "Relief Captain");
    t.resolve_all();
    assert!(t.counters(bears, counters::PLUS1) <= 1);
    assert!(t.counters(mine, counters::PLUS1) <= 2);
    assert!(t.counters(theirs, counters::PLUS1) <= 2);
}

#[test]
fn support_can_target_a_creature_another_player_controls() {
    cr!("701.41a");
    ruling!(
        "The Crowd Goes Wild",
        "Support can target a creature another player controls."
    );
    supported("The Crowd Goes Wild");
    // The Crowd Goes Wild ({X}{G}): "Support X. Each creature with a +1/+1 counter on it
    // gains trample until end of turn."
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Gray Ogre");
    t.lands(P0, "Forest", 3);
    let spell = t.hand(P0, "The Crowd Goes Wild");
    t.cast(P0, spell)
        .x(2)
        .targets(&[mine.into(), theirs.into()])
        .go();
    t.resolve_all();
    assert_eq!(t.counters(theirs, counters::PLUS1), 1);
    assert_eq!(t.counters(mine, counters::PLUS1), 1);
    assert!(t.obj_now(theirs).chars.has_keyword(KeywordKind::Trample));
    assert!(t.obj_now(mine).chars.has_keyword(KeywordKind::Trample));
    assert!(!t.obj_now(other).chars.has_keyword(KeywordKind::Trample));
}

#[test]
fn support_affects_its_remaining_legal_targets() {
    cr!("701.41a", "608.2b");
    ruling!(
        "Soulblade Renewer",
        "If some, but not all, targets for a spell become illegal, the remaining targets are affected as appropriate. If all of a spell's targets become illegal, that spell doesn't resolve."
    );
    supported("Soulblade Renewer");
    // Soulblade Renewer: "When this creature enters, support 2." (and partner with).
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[a.into(), b.into()]);
    t.enter(P0, "Soulblade Renewer");
    t.settle();
    let trigger = support_trigger(&t);
    // One target leaves before the ability resolves: the other still gets its counter.
    destroy(&mut t, b);
    t.resolve_all();
    assert_eq!(t.counters(a, counters::PLUS1), 1);
    assert!(t.g.turn_events.iter().any(
        |e| matches!(e, Event::AbilityResolved { ability, .. } if *ability == trigger)
    ));

    // All of its targets gone: it doesn't resolve.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[a.into(), b.into()]);
    t.enter(P0, "Soulblade Renewer");
    t.settle();
    let trigger = support_trigger(&t);
    destroy(&mut t, a);
    destroy(&mut t, b);
    t.resolve_all();
    assert!(!t.g.stack.contains(&trigger));
    assert!(!t.g.turn_events.iter().any(
        |e| matches!(e, Event::AbilityResolved { ability, .. } if *ability == trigger)
    ));
}

/// The support triggered ability on the stack.
fn support_trigger(t: &TestGame) -> ObjectId {
    *t.g.stack
        .iter()
        .find(|id| {
            t.g.obj(**id).stack.as_ref().is_some_and(|si| {
                matches!(&si.kind, StackKind::Triggered { ability, .. }
                    if ability.text.to_lowercase().contains("support"))
            })
        })
        .expect("no support trigger")
}
