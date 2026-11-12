//! CR 614 audit: a replacement effect gets one opportunity to affect an event and the
//! events that replace it (614.5), however the modified events are carried out (damage
//! bounced between redirection effects, draws an "instead" effect performs later); and a
//! destruction replaced by regeneration never becomes a death for "would die" effects.

use crate::r614_audit_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn damage_bounced_between_two_palisade_giants_stops_on_the_second() {
    cr!("614.5", "614.9", "616.1e", "616.2");
    supported("Palisade Giant");
    for first in [0usize, 1] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Palisade Giant");
        let b = t.battlefield(P0, "Palisade Giant");
        pick_replacements(&mut t, P0, &[first]);
        cast_resolve(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
        assert_eq!(t.life(P0), 20);
        // The damage is redirected to the Giant whose effect P0 applies first; the other
        // Giant's effect then redirects it from that Giant ("other permanents you
        // control") to itself; the first can't apply again.
        let choices = replacement_choices(&t);
        assert_eq!(choices.len(), 1, "one choice, between the two Giants");
        let first_is_a = choices[0].1[first].contains(&format!("object {}", a.0));
        let (got, spared) = if first_is_a { (b, a) } else { (a, b) };
        assert_eq!(damage_on(&t, got), 3, "first pick {first}");
        assert_eq!(damage_on(&t, spared), 0, "first pick {first}");
    }
}

#[test]
fn two_thought_reflections_make_one_draw_four() {
    cr!("614.5", "614.11", "616.2");
    supported("Thought Reflection");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thought Reflection");
    t.battlefield(P0, "Thought Reflection");
    let hand = t.hand_size(P0);
    let lib = t.library_size(P0);
    // Opt: scry 1, then draw a card. Each Reflection doubles the draws once: 1 -> 2 -> 4.
    cast_resolve(&mut t, P0, "Opt", &[]);
    assert_eq!(t.hand_size(P0), hand + 4);
    assert_eq!(t.library_size(P0), lib - 4);
}

/// P0 controls Notion Thief; P1 controls Thought Reflection and casts Opt on P1's turn
/// (not the first draw of a draw step); P1's answers are `picks`.
fn opt_under_notion_thief_and_reflection(picks: &[usize]) -> TestGame {
    supported("Notion Thief");
    supported("Thought Reflection");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Notion Thief");
    t.battlefield(P1, "Thought Reflection");
    pick_replacements(&mut t, P1, picks);
    cast_resolve(&mut t, P1, "Opt", &[]);
    t
}

#[test]
fn reflection_then_notion_thief_gives_the_thief_two_cards_thief_first_gives_one() {
    cr!("616.1", "616.1e", "616.1f", "614.5", "614.11");
    let base = TestGame::new(2).hand_size(P0);
    // Reflection first: two draws instead, and the Thief applies to each of them.
    let t = opt_under_notion_thief_and_reflection(&[]);
    let opts = replacement_choices(&t)[0].1.clone();
    assert_eq!(replacement_choices(&t)[0].0, P1);
    let reflection = option_index(&opts, "Thought Reflection");
    let thief = option_index(&opts, "Notion Thief");
    let t = opt_under_notion_thief_and_reflection(&[reflection]);
    assert_eq!(t.hand_size(P0), base + 2);
    assert_eq!(t.hand_size(P1), base);
    // The Thief first: P0 draws instead, and P1's Reflection doesn't apply to P0's draw.
    let t = opt_under_notion_thief_and_reflection(&[thief]);
    assert_eq!(t.hand_size(P0), base + 1);
    assert_eq!(t.hand_size(P1), base);
}

#[test]
fn a_regenerated_creature_isnt_exiled_by_kalitas() {
    cr!("614.8", "701.19a", "614.7");
    supported("Drudge Skeletons");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kalitas, Traitor of Ghet");
    let skel = t.battlefield(P1, "Drudge Skeletons");
    t.lands(P1, "Swamp", 1);
    t.activate(P1, skel, 0, &[]).expect("regenerate");
    t.resolve_all();
    cast_resolve(&mut t, P0, "Murder", &[obj(skel)]);
    // The destruction is replaced by regeneration: the creature never would die.
    assert!(t.on_battlefield(skel));
    assert!(t.obj_now(skel).tapped);
    assert!(t.named_on_battlefield("Zombie Token").is_empty());
}

#[test]
fn a_creature_that_cant_be_regenerated_dies_into_kalitas_despite_its_shield() {
    cr!("701.19c", "614.8");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kalitas, Traitor of Ghet");
    let skel = t.battlefield(P1, "Drudge Skeletons");
    t.lands(P1, "Swamp", 1);
    t.activate(P1, skel, 0, &[]).expect("regenerate");
    t.resolve_all();
    cast_resolve(&mut t, P0, "Wrath of God", &[]);
    assert_eq!(t.zone(t.g.current(skel)), Zone::Exile);
    assert_eq!(t.named_on_battlefield("Zombie Token").len(), 1);
}
