//! CR 702.40 Storm.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_038_051::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Casts and resolves a Lightning Bolt at `p` for `caster`.
fn bolt(t: &mut TestGame, caster: PlayerId, at: PlayerId) {
    t.lands(caster, "Mountain", 1);
    let b = t.hand(caster, "Lightning Bolt");
    cast_at(t, caster, b, &[Entity::Player(at)]);
    t.resolve_all();
}

#[test]
fn storm_copies_the_spell_for_each_spell_cast_before_it() {
    cr!("702.40", "702.40a");
    assert_supported("Grapeshot");
    let mut t = TestGame::new(2);
    bolt(&mut t, P0, P1);
    bolt(&mut t, P0, P1);
    assert_eq!(t.life(P1), 14);
    t.lands(P0, "Mountain", 2);
    let shot = t.hand(P0, "Grapeshot");
    cast_at(&mut t, P0, shot, &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(triggers_named(&t, "Storm").len(), 1);
    // The storm trigger resolves first: two copies go on the stack above Grapeshot.
    t.resolve();
    assert_eq!(spell_copies_on_stack(&t, "Grapeshot"), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 11);
    // The copies weren't cast.
    assert_eq!(t.g.history.spells_cast.len(), 3);
}

#[test]
fn storm_counts_every_players_spells_even_countered_ones() {
    cr!("702.40a");
    ruling!(
        "Grapeshot",
        "Spells cast from zones other than a player's hand and spells that were countered are counted by the storm ability."
    );
    let mut t = TestGame::new(2);
    // P0 casts a Lightning Bolt; P1 counters it with Counterspell.
    t.lands(P0, "Mountain", 1);
    t.lands(P1, "Island", 2);
    let b = t.hand(P0, "Lightning Bolt");
    let bolt_spell = cast_at(&mut t, P0, b, &[Entity::Player(P1)]);
    let cs = t.hand(P1, "Counterspell");
    cast_at(&mut t, P1, cs, &[Entity::Object(bolt_spell)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    t.lands(P0, "Mountain", 2);
    let shot = t.hand(P0, "Grapeshot");
    cast_at(&mut t, P0, shot, &[Entity::Player(P1)]);
    t.resolve();
    // Two spells were cast before it (one of them P1's, one countered).
    assert_eq!(spell_copies_on_stack(&t, "Grapeshot"), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn spells_cast_after_the_storm_spell_dont_count() {
    cr!("702.40a");
    let mut t = TestGame::new(2);
    bolt(&mut t, P0, P1);
    t.lands(P0, "Mountain", 2);
    let shot = t.hand(P0, "Grapeshot");
    cast_at(&mut t, P0, shot, &[Entity::Player(P1)]);
    t.settle();
    // In response to the storm trigger, P1 casts a spell.
    t.lands(P1, "Mountain", 1);
    let b = t.hand(P1, "Lightning Bolt");
    cast_at(&mut t, P1, b, &[Entity::Player(P0)]);
    t.resolve(); // P1's Lightning Bolt
    assert_eq!(t.life(P0), 17);
    t.resolve(); // the storm trigger
    assert_eq!(spell_copies_on_stack(&t, "Grapeshot"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

#[test]
fn copies_arent_cast_and_dont_count_for_later_storm_spells() {
    cr!("702.40a");
    ruling!(
        "Grapeshot",
        "They aren't cast and won't be counted by other spells with storm cast later in the turn."
    );
    let mut t = TestGame::new(2);
    bolt(&mut t, P0, P1);
    t.lands(P0, "Mountain", 4);
    let s1 = t.hand(P0, "Grapeshot");
    cast_at(&mut t, P0, s1, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 15); // 3 + 1 + 1
    let s2 = t.hand(P0, "Grapeshot");
    cast_at(&mut t, P0, s2, &[Entity::Player(P1)]);
    t.resolve();
    // Lightning Bolt and the first Grapeshot: two copies, not three.
    assert_eq!(spell_copies_on_stack(&t, "Grapeshot"), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 12);
}

#[test]
fn copies_may_have_new_targets_chosen_separately() {
    cr!("702.40a");
    ruling!(
        "Grapeshot",
        "You may choose new targets for any of the copies. You can make different choices for each copy."
    );
    let mut t = TestGame::new(2);
    bolt(&mut t, P0, P1);
    bolt(&mut t, P0, P1);
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let shot = t.hand(P0, "Grapeshot");
    cast_at(&mut t, P0, shot, &[Entity::Player(P1)]);
    // First copy: new target (the Bears); second copy: keep the target (P1).
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bear)]);
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(spell_copies_on_stack(&t, "Grapeshot"), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 6 - 2);
    assert_eq!(t.obj_now(bear).damage, 1);
}

#[test]
fn countering_the_storm_trigger_makes_no_copies() {
    cr!("702.40a");
    ruling!(
        "Grapeshot",
        "The triggered ability that creates the copies can itself be countered by anything that can counter a triggered ability. If it is countered, no copies will be put onto the stack."
    );
    let mut t = TestGame::new(2);
    bolt(&mut t, P0, P1);
    t.lands(P0, "Mountain", 2);
    let shot = t.hand(P0, "Grapeshot");
    cast_at(&mut t, P0, shot, &[Entity::Player(P1)]);
    t.settle();
    let trig = triggers_named(&t, "Storm")[0];
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    cast_at(&mut t, P1, stifle, &[Entity::Object(trig)]);
    t.resolve_all();
    // Only the original Grapeshot resolved.
    assert_eq!(t.life(P1), 16);
}

#[test]
fn countering_the_original_doesnt_affect_the_copies() {
    cr!("702.40a");
    ruling!(
        "Grapeshot",
        "Countering a spell with storm won't affect the copies."
    );
    let mut t = TestGame::new(2);
    bolt(&mut t, P0, P1);
    t.lands(P0, "Mountain", 2);
    let shot = t.hand(P0, "Grapeshot");
    let spell = cast_at(&mut t, P0, shot, &[Entity::Player(P1)]);
    t.resolve(); // storm trigger: one copy
    assert_eq!(spell_copies_on_stack(&t, "Grapeshot"), 1);
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    cast_at(&mut t, P1, cs, &[Entity::Object(spell)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grapeshot"));
    assert_eq!(t.life(P1), 16);
}

#[test]
fn spells_given_storm_have_it_as_they_are_cast() {
    cr!("702.40a", "702.40b");
    ruling!(
        "Prismari, the Inspiration",
        "If a spell has multiple instances of storm, each will trigger separately."
    );
    let mut t = TestGame::new(2);
    // Prismari: "Instant and sorcery spells you cast have storm."
    t.battlefield(P0, "Prismari, the Inspiration");
    bolt(&mut t, P0, P1);
    // Lightning Bolt now has storm: one copy.
    t.lands(P0, "Mountain", 1);
    let b = t.hand(P0, "Lightning Bolt");
    cast_at(&mut t, P0, b, &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(triggers_named(&t, "Storm").len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 3 - 3 - 3);
    // Grapeshot has storm twice: each instance triggers.
    t.lands(P0, "Mountain", 2);
    let shot = t.hand(P0, "Grapeshot");
    cast_at(&mut t, P0, shot, &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(triggers_named(&t, "Storm").len(), 2);
    t.resolve_all();
    // Two spells before it, twice: four copies plus the original.
    assert_eq!(t.life(P1), 11 - 5);
}

#[test]
fn the_next_spell_can_be_given_storm() {
    cr!("702.40a");
    ruling!(
        "Crackling Spellslinger",
        "The copies of a spell with storm created by its storm ability are put directly onto the stack. They aren’t cast and won’t be counted by other spells with storm cast later in the turn."
    );
    assert_supported("Crackling Spellslinger");
    let mut t = TestGame::new(2);
    // Crackling Spellslinger: "When this creature enters, if you cast it, the next
    // instant or sorcery spell you cast this turn has storm."
    t.lands(P0, "Mountain", 6);
    let ss = t.hand(P0, "Crackling Spellslinger");
    t.cast(P0, ss).go();
    t.resolve_all();
    let b = t.hand(P0, "Lightning Bolt");
    cast_at(&mut t, P0, b, &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(triggers_named(&t, "Storm").len(), 1);
    t.resolve_all();
    // The Spellslinger was cast before it: one copy.
    assert_eq!(t.life(P1), 14);
    // Only that spell had storm.
    let b2 = t.hand(P0, "Lightning Bolt");
    t.lands(P0, "Mountain", 1);
    cast_at(&mut t, P0, b2, &[Entity::Player(P1)]);
    t.settle();
    assert!(triggers_named(&t, "Storm").is_empty());
}

#[test]
fn each_instance_of_storm_triggers_separately() {
    cr!("702.40b");
    let def = with_cost(
        custom_card(
            "Double Shot",
            "Sorcery",
            None,
            "Double Shot deals 1 damage to any target.\nStorm\nStorm",
        ),
        "{R}",
    );
    let mut t = TestGame::new(2);
    bolt(&mut t, P0, P1);
    t.lands(P0, "Mountain", 1);
    let s = t.custom(P0, def, Zone::Hand(P0));
    cast_at(&mut t, P0, s, &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(triggers_named(&t, "Storm").len(), 2);
    t.resolve_all();
    // One copy from each instance, plus the original.
    assert_eq!(t.life(P1), 17 - 3);
}
