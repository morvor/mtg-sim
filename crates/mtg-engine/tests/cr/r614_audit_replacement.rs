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
    cr!("614.5", "616.2");
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
    cr!("616.1", "616.1e", "616.1f", "614.5");
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

#[test]
fn laboratory_maniac_wins_on_the_second_of_thought_reflections_two_draws() {
    cr!("614.11", "614.11a", "616.2", "104.2b");
    supported("Laboratory Maniac");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Laboratory Maniac");
    t.battlefield(P0, "Thought Reflection");
    // One card left: as the draw would happen the library isn't empty, so only Thought
    // Reflection applies. Of its two draws, the first takes the last card; the second is
    // replaced by Laboratory Maniac's effect.
    let lib = t.g.players[0].library.clone();
    t.g.players[0].library = vec![lib[0]];
    cast_resolve(&mut t, P0, "Opt", &[]);
    assert_eq!(t.library_size(P0), 0);
    assert!(t.has_lost(P1));
    assert!(!t.has_lost(P0));
}

/// P1's Drudge Skeletons, enchanted with Hyena Umbra and with a regeneration shield, is
/// the target of P0's Murder; P1's answers are `picks`.
fn murder_umbra_skeletons(picks: &[usize]) -> (TestGame, ObjectId, ObjectId) {
    supported("Hyena Umbra");
    let mut t = TestGame::new(2);
    let skel = t.battlefield(P1, "Drudge Skeletons");
    let umbra = t.battlefield(P1, "Hyena Umbra");
    t.g.attach(umbra, obj(skel));
    t.lands(P1, "Swamp", 1);
    t.activate(P1, skel, 0, &[]).expect("regenerate");
    t.resolve_all();
    pick_replacements(&mut t, P1, picks);
    cast_resolve(&mut t, P0, "Murder", &[obj(skel)]);
    (t, skel, umbra)
}

#[test]
fn umbra_armor_or_regeneration_the_controller_chooses_and_the_other_waits() {
    cr!("616.1", "616.1e", "614.8", "702.89a", "614.3");
    let (probe, _, _) = murder_umbra_skeletons(&[]);
    let (who, opts) = replacement_choices(&probe)[0].clone();
    assert_eq!(who, P1);
    // Umbra armor: the Aura is destroyed instead; the Skeletons stay untapped, and the
    // shield is still there for the next destruction this turn.
    let (mut t, skel, umbra) = murder_umbra_skeletons(&[option_index(&opts, "Hyena Umbra")]);
    assert!(t.on_battlefield(skel));
    assert!(!t.obj_now(skel).tapped);
    assert!(t.in_graveyard(P1, "Hyena Umbra"));
    assert!(!t.on_battlefield(umbra));
    cast_resolve(&mut t, P0, "Murder", &[obj(skel)]);
    assert!(t.on_battlefield(skel), "the regeneration shield is used now");
    assert!(t.obj_now(skel).tapped);
    // Regeneration: tapped, and the Aura stays.
    let regen = opts
        .iter()
        .position(|o| !o.contains("Hyena Umbra"))
        .expect("the regeneration shield");
    let (t, skel, umbra) = murder_umbra_skeletons(&[regen]);
    assert!(t.on_battlefield(skel));
    assert!(t.obj_now(skel).tapped);
    assert!(t.on_battlefield(umbra));
}

#[test]
fn rest_in_peace_destroyed_with_the_creatures_still_exiles_them_all() {
    cr!("614.1a", "614.6", "603.10a");
    supported("Akroma's Vengeance");
    supported("Blood Artist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rest in Peace");
    t.battlefield(P0, "Blood Artist");
    t.battlefield(P1, "Grizzly Bears");
    // Rest in Peace leaves at the same time as the creatures: it still applies to all of
    // them (and to itself), so nothing dies and Blood Artist never triggers.
    cast_resolve(&mut t, P0, "Akroma's Vengeance", &[]);
    for n in ["Rest in Peace", "Blood Artist", "Grizzly Bears"] {
        assert!(t.in_exile(n), "{n} is exiled");
    }
    // The spell itself is put into the graveyard after it resolves, with Rest in Peace
    // gone.
    assert!(t.in_graveyard(P0, "Akroma's Vengeance"));
    assert_eq!(t.graveyard_size(P0) + t.graveyard_size(P1), 1);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_land_put_onto_the_battlefield_tapped_under_spelunking_enters_untapped_without_a_choice() {
    cr!("614.1d");
    ruling!(
        "Spelunking",
        "If a land you control is simply put onto the battlefield tapped without a replacement effect being applied, it always enters untapped if you control Spelunking."
    );
    supported("Spelunking");
    supported("Rampant Growth");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spelunking");
    let forest = t.library_top(P0, "Forest");
    t.answer_choose_moved(P0, &[obj(forest)]);
    cast_resolve(&mut t, P0, "Rampant Growth", &[]);
    let forest = t.g.current(forest);
    assert!(t.on_battlefield(forest));
    // "Tapped" is part of the spell's instruction, not a replacement effect Spelunking's
    // could be ordered with: Spelunking's effect has it enter untapped.
    assert!(!is_tapped(&t, forest));
    assert!(replacement_choices(&t).is_empty());
}

#[test]
fn whiptongue_hydra_counts_only_the_fliers_actually_destroyed() {
    cr!("614.8", "701.19a", "702.89a", "614.6");
    ruling!(
        "Whiptongue Hydra",
        "A creature with indestructible or a creature that regenerates won't count as having been \"destroyed this way.\""
    );
    ruling!(
        "Whiptongue Hydra",
        "If a creature with flying would be destroyed, but an Aura with the umbra armor ability is destroyed instead, Whiptongue Hydra won't count that creature or that Aura as a \"creature destroyed this way.\""
    );
    supported("Whiptongue Hydra");
    supported("Ghost Ship");
    supported("Hyena Umbra");
    supported("Darksteel Gargoyle");
    let mut t = TestGame::new(2);
    // P0's fliers: Ghost Ship with a regeneration shield, Serra Angel enchanted with
    // Hyena Umbra, Darksteel Gargoyle (indestructible) and another Serra Angel.
    let ship = t.battlefield(P0, "Ghost Ship");
    t.lands(P0, "Island", 3);
    t.activate(P0, ship, 0, &[]).expect("regenerate");
    t.resolve_all();
    let angel = t.battlefield(P0, "Serra Angel");
    let umbra = t.battlefield(P0, "Hyena Umbra");
    t.g.attach(umbra, obj(angel));
    let gargoyle = t.battlefield(P0, "Darksteel Gargoyle");
    let plain = t.battlefield(P0, "Serra Angel");
    t.set_step(P1, Step::PrecombatMain);
    // "When this creature enters, destroy all creatures with flying. Put a +1/+1 counter
    // on this creature for each creature destroyed this way."
    let hydra = cast_resolve(&mut t, P1, "Whiptongue Hydra", &[]);
    let hydra = t.g.current(hydra);
    for (id, stays) in [(ship, true), (angel, true), (gargoyle, true), (plain, false)] {
        assert_eq!(t.on_battlefield(id), stays);
    }
    assert!(t.obj_now(ship).tapped, "regenerated");
    assert!(t.in_graveyard(P0, "Hyena Umbra"));
    // Only the unprotected Angel was destroyed this way.
    assert_eq!(plus_counters(&t, hydra), 1);
}
