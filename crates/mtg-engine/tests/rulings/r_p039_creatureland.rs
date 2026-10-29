//! Rulings batch P039 — lands that become creatures (Crawling Barrens, Hostile Desert,
//! Mishra's Foundry): counters on a noncreature land, summoning sickness, and
//! power/toughness setting effects (CR 302.6, 613.4b).

use crate::r_p039_common::*;
use crate::r_s02_common::{can_attack, can_play_land};
use mtg_engine::ability::AbilityKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Activates the activated ability of `src` whose text contains `needle`.
pub fn activate_text(
    t: &mut TestGame,
    p: PlayerId,
    src: ObjectId,
    needle: &str,
) -> Result<Option<ObjectId>, mtg_engine::casting::Illegal> {
    t.g.recompute();
    let src = t.g.current(src);
    let uid = t
        .g
        .obj(src)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.contains(needle))
        .map(|a| a.uid)
        .unwrap_or_else(|| panic!("no ability {needle:?}"));
    t.g.turn.priority = Some(p);
    let r = t.g.activate_ability(p, src, uid);
    t.g.flush_events();
    r
}

/// Whether `p` could activate the ability of `src` whose text contains `needle` now.
pub fn can_activate_text(t: &mut TestGame, p: PlayerId, src: ObjectId, needle: &str) -> bool {
    use mtg_engine::decision::Action;
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let src = t.g.current(src);
    let uids: Vec<_> = t
        .g
        .obj(src)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.contains(needle))
        .map(|a| a.uid)
        .collect();
    t.g.legal_actions(p).iter().any(|a| {
        matches!(a, Action::Activate { source, ability, .. } if *source == src && uids.contains(ability))
    })
}

/// Whether `src` is among `p`'s usable mana sources now.
pub fn taps_for_mana(t: &mut TestGame, p: PlayerId, src: ObjectId) -> bool {
    t.g.recompute();
    let src = t.g.current(src);
    mtg_engine::mana_abilities::mana_sources(&t.g, p, None)
        .iter()
        .any(|s| s.obj == src)
}

/// Activates Crawling Barrens's {4} ability (paid from the pool), answering whether it
/// becomes a creature, and resolves it.
fn grow_barrens(t: &mut TestGame, b: ObjectId, animate: bool) {
    mana(t, P0, ManaType::C, 4);
    activate_text(t, P0, b, "Put two +1/+1 counters").expect("activate");
    yes(t, P0, animate);
    t.resolve_all();
}

#[test]
fn crawling_barrens_keeps_its_counters_while_not_a_creature() {
    cr!("122.1", "613.4c");
    ruling!(
        "Crawling Barrens",
        "Counters on Crawling Barrens remain on it when it stops being a creature. If it becomes a creature later, they'll apply to it."
    );
    supported_card("Crawling Barrens");
    let mut t = TestGame::new(2);
    let b = old(&mut t, P0, "Crawling Barrens");
    grow_barrens(&mut t, b, true);
    assert!(creature_now(&t, b));
    assert_eq!(t.pt(b), (2, 2));
    t.advance_to(P1, Step::Upkeep);
    assert!(!creature_now(&t, b));
    assert_eq!(t.counters(b, counters::PLUS1), 2);
    t.advance_to(P0, Step::PrecombatMain);
    grow_barrens(&mut t, b, true);
    assert_eq!(t.pt(b), (4, 4));
}

#[test]
fn crawling_barrens_counters_go_on_a_noncreature_land() {
    cr!("122.6", "614.1a");
    ruling!(
        "Crawling Barrens",
        "Unless Crawling Barrens is already a creature, its second ability causes the +1/+1 counters to be put onto a noncreature land."
    );
    // Hardened Scales: "If one or more +1/+1 counters would be put on a creature you
    // control, that many plus one +1/+1 counters are put on it instead."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hardened Scales");
    let b = old(&mut t, P0, "Crawling Barrens");
    grow_barrens(&mut t, b, true);
    // Put on a land (Hardened Scales doesn't apply), then it becomes a 0/0 creature.
    assert_eq!(t.counters(b, counters::PLUS1), 2);
    // Already a creature: Hardened Scales applies.
    grow_barrens(&mut t, b, false);
    assert_eq!(t.counters(b, counters::PLUS1), 5);
    assert_eq!(t.pt(b), (5, 5));
}

#[test]
fn crawling_barrens_can_grow_while_already_a_creature() {
    cr!("602.1");
    ruling!(
        "Crawling Barrens",
        "You can activate Crawling Barrens's second ability even if it's already a creature."
    );
    let mut t = TestGame::new(2);
    let b = old(&mut t, P0, "Crawling Barrens");
    grow_barrens(&mut t, b, true);
    assert!(creature_now(&t, b));
    mana(&mut t, P0, ManaType::C, 4);
    assert!(can_activate_text(&mut t, P0, b, "Put two +1/+1 counters"));
    t.g.players[0].mana_pool.empty();
    grow_barrens(&mut t, b, true);
    assert_eq!(t.pt(b), (4, 4));
}

#[test]
fn hostile_desert_animated_the_turn_it_entered_is_summoning_sick() {
    cr!("302.6");
    ruling!(
        "Hostile Desert",
        "If Hostile Desert becomes a creature the same turn it enters the battlefield, you can't attack with it or tap it for mana."
    );
    supported_card("Hostile Desert");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Forest");
    let d = t.hand(P0, "Hostile Desert");
    assert!(can_play_land(&mut t, P0, d));
    t.play_land(P0, d).unwrap();
    let d = t.g.current(d);
    // As a land, it can tap for mana the turn it entered.
    assert!(taps_for_mana(&mut t, P0, d));
    mana(&mut t, P0, ManaType::C, 2);
    activate_text(&mut t, P0, d, "becomes a 3/4").expect("activate");
    t.resolve_all();
    assert!(creature_now(&t, d));
    assert!(!taps_for_mana(&mut t, P0, d));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, d));
}

#[test]
fn mishras_foundry_animated_while_summoning_sick() {
    cr!("302.6");
    ruling!(
        "Mishra's Foundry",
        "If Mishra's Foundry becomes a creature and you haven't controlled it continuously since the beginning of your most recent turn, you won't be able to activate its first or last abilities, and it won't be able to attack (unless it somehow gains haste)."
    );
    let mut t = TestGame::new(2);
    let f = t.battlefield_sick(P0, "Mishra's Foundry");
    // Two old Foundries: one attacks (the target of the last ability), one stays home.
    let attacker = old(&mut t, P0, "Mishra's Foundry");
    let home = old(&mut t, P0, "Mishra's Foundry");
    mana(&mut t, P0, ManaType::C, 6);
    for x in [f, attacker, home] {
        activate_text(&mut t, P0, x, "becomes a 2/2").expect("animate");
        t.resolve_all();
    }
    assert!(creature_now(&t, f));
    assert!(!taps_for_mana(&mut t, P0, f));
    assert!(taps_for_mana(&mut t, P0, home));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, f));
    assert!(can_attack(&mut t, home));
    t.answer(
        P0,
        DecisionKind::Attackers,
        mtg_engine::decision::Answer::Attackers(vec![(attacker, pl(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    mana(&mut t, P0, ManaType::C, 1);
    assert!(!can_activate_text(&mut t, P0, f, "Target attacking"));
    assert!(can_activate_text(&mut t, P0, home, "Target attacking"));
}

#[test]
fn mishras_foundry_reanimating_overrides_set_power_and_toughness() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Mishra's Foundry",
        "If Mishra's Foundry is already a creature, activating the second ability will override any previous effects that set its power and/or toughness to specific values."
    );
    let mut t = TestGame::new(2);
    let f = old(&mut t, P0, "Mishra's Foundry");
    mana(&mut t, P0, ManaType::C, 2);
    activate_text(&mut t, P0, f, "becomes a 2/2").unwrap();
    t.resolve_all();
    // Diminish: base power and toughness 1/1 until end of turn.
    cast_named(&mut t, P0, "Diminish", &[o(f)]);
    t.resolve_all();
    t.g.add_counters(o(f), counters::PLUS1, 1, None);
    cast_named(&mut t, P0, "Giant Growth", &[o(f)]);
    t.resolve_all();
    assert_eq!(t.pt(f), (5, 5));
    mana(&mut t, P0, ManaType::C, 2);
    activate_text(&mut t, P0, f, "becomes a 2/2").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(f), (6, 6));
}

fn supported_card(name: &str) {
    crate::r_s01_common::supported(name);
}
