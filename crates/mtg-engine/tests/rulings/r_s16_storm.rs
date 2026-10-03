//! Rulings on storm (CR 702.40): which spells it counts, what the copies are, and how they
//! resolve and can be countered.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s16_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, ObjKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);

/// `caster` casts a Lightning Bolt at `at` (with a Mountain to pay for it).
fn bolt(t: &mut TestGame, caster: PlayerId, at: Entity) -> ObjectId {
    t.lands(caster, "Mountain", 1);
    let b = t.hand(caster, "Lightning Bolt");
    t.cast(caster, b).target(at).go()
}

/// `p` counters `spell` with a Counterspell (with two Islands to pay for it).
fn counterspell(t: &mut TestGame, p: PlayerId, spell: ObjectId) {
    t.lands(p, "Island", 2);
    let cs = t.hand(p, "Counterspell");
    t.cast(p, cs).target(Entity::Object(spell)).go();
}

/// P0 casts four spells: Think Twice from the graveyard (flashback), a Lightning Bolt that
/// fails to resolve (its target is gone), and a Lightning Bolt that P1 counters with a
/// Counterspell.
fn four_spells_that_didnt_resolve_normally(t: &mut TestGame) {
    t.lands(P0, "Island", 3);
    let tt = t.graveyard(P0, "Think Twice");
    t.cast(P0, tt).method(FLASHBACK).go();
    t.resolve_all();
    let elves = t.battlefield(P1, "Llanowar Elves");
    bolt(t, P0, Entity::Object(elves));
    destroy(t, elves);
    t.resolve_all();
    let b = bolt(t, P0, Entity::Player(P1));
    counterspell(t, P1, b);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.g.history.spells_cast.len(), 4);
}

fn copies_of(t: &TestGame, name: &str) -> Vec<ObjectId> {
    t.g.stack
        .iter()
        .copied()
        .filter(|id| t.g.obj(*id).kind == ObjKind::SpellCopy && t.g.obj(*id).chars.name == name)
        .collect()
}

#[test]
fn storm_counts_spells_from_other_zones_countered_ones_and_ones_that_failed_to_resolve() {
    cr!("702.40a", "707.10c");
    ruling!(
        "Spreading Insurrection",
        "Spells cast from zones other than a player's hand and spells that were countered or otherwise failed to resolve are counted by the storm ability."
    );
    ruling!(
        "Spreading Insurrection",
        "If a spell with storm has targets, you may choose new targets for any of the copies. You can make different choices for each copy."
    );
    supported("Spreading Insurrection");
    let mut t = TestGame::new(2);
    four_spells_that_didnt_resolve_normally(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let ogre = t.battlefield(P1, "Gray Ogre");
    t.g.objects[giant.0 as usize].tapped = true;
    // Spreading Insurrection: "Gain control of target creature you don't control until end
    // of turn. Untap that creature. It gains haste until end of turn." Storm.
    t.lands(P0, "Mountain", 5);
    let si = t.hand(P0, "Spreading Insurrection");
    t.cast(P0, si).target(bears).go();
    // Four copies: the first gets a new target (Hill Giant), the second another one (Gray
    // Ogre), the other two keep targeting the Bears.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(ogre)]);
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(copies_of(&t, "Spreading Insurrection").len(), 4);
    t.resolve_all();
    for c in [bears, giant, ogre] {
        let o = t.obj_now(c);
        assert_eq!(o.controller, P0);
        assert!(!o.tapped);
        assert!(o.chars.has_keyword(KeywordKind::Haste));
    }
}

#[test]
fn each_copy_must_be_countered_individually_and_all_kinds_of_spells_count() {
    cr!("702.40a", "701.6a");
    ruling!(
        "Elemental Eruption",
        "Spells cast from zones other than a player’s hand and spells that were countered or otherwise failed to resolve are counted by the storm ability."
    );
    ruling!(
        "Elemental Eruption",
        "A copy of a spell can be countered like any other spell, but it must be countered individually. Countering a spell with storm won’t affect the copies."
    );
    supported("Elemental Eruption");
    let mut t = TestGame::new(2);
    four_spells_that_didnt_resolve_normally(&mut t);
    // Elemental Eruption: "Create a 4/4 red Dragon Elemental creature token with flying and
    // prowess." Storm.
    t.lands(P0, "Mountain", 6);
    let ee = t.hand(P0, "Elemental Eruption");
    let original = t.cast(P0, ee).go();
    t.resolve();
    let copies = copies_of(&t, "Elemental Eruption");
    assert_eq!(copies.len(), 4);
    // P1 counters one copy and the original: the other three copies still resolve.
    counterspell(&mut t, P1, copies[0]);
    t.resolve();
    counterspell(&mut t, P1, original);
    t.resolve();
    assert_eq!(copies_of(&t, "Elemental Eruption").len(), 3);
    assert!(t.in_graveyard(P0, "Elemental Eruption"));
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Dragon").len(), 3);
}

#[test]
fn the_storm_trigger_and_its_copies_resolve_first_even_if_the_spell_is_countered() {
    cr!("702.40a", "405.5");
    ruling!(
        "Weather the Storm",
        "The storm ability and the copies it creates all resolve before the spell with storm. They resolve even if the spell with storm is countered."
    );
    supported("Weather the Storm");
    let mut t = TestGame::new(2);
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    // Weather the Storm: "You gain 3 life." Storm.
    t.lands(P0, "Forest", 2);
    let wts = t.hand(P0, "Weather the Storm");
    let spell = t.cast(P0, wts).go();
    t.settle();
    // The storm trigger is above the spell, and the copy it creates too.
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    let copy = copies_of(&t, "Weather the Storm")[0];
    assert_eq!(t.g.stack.last(), Some(&copy));
    t.resolve();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.g.stack.last(), Some(&spell));
    // The spell itself is countered: the copy has already resolved.
    counterspell(&mut t, P1, spell);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert!(t.in_graveyard(P0, "Weather the Storm"));
}

/// P1 attacks P0 with `n` Grizzly Bears; returns them.
fn p1_attacks_with_bears(t: &mut TestGame, n: usize) -> Vec<ObjectId> {
    let bears: Vec<ObjectId> = (0..n).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    t.set_step(P1, Step::BeginningOfCombat);
    let decl: Vec<(ObjectId, Entity)> = bears.iter().map(|b| (*b, Entity::Player(P0))).collect();
    attack_with(t, &decl);
    bears
}

#[test]
fn storm_copies_arent_cast_so_cast_triggers_dont_trigger_for_them() {
    cr!("702.40a", "707.10", "603.2");
    ruling!(
        "Wing Shards",
        "The copies storm creates are created on the stack, so they’re not cast. Abilities that trigger when a player casts a spell (such as storm) won’t trigger."
    );
    supported("Wing Shards");
    supported("Young Pyromancer");
    let mut t = TestGame::new(2);
    // Young Pyromancer: "Whenever you cast an instant or sorcery spell, create a 1/1 red
    // Elemental creature token."
    t.battlefield(P0, "Young Pyromancer");
    let bears = p1_attacks_with_bears(&mut t, 2);
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    // Wing Shards: "Target player sacrifices an attacking creature of their choice." Storm.
    t.lands(P0, "Plains", 3);
    let ws = t.hand(P0, "Wing Shards");
    t.cast(P0, ws).target(P1).go();
    t.resolve();
    assert_eq!(copies_of(&t, "Wing Shards").len(), 1);
    t.resolve_all();
    // The copy sacrificed an attacker too.
    assert!(bears.iter().all(|b| !t.on_battlefield(*b)));
    // Two Elementals: for Lightning Bolt and Wing Shards, none for the copy.
    assert_eq!(with_subtype(&t, P0, "Elemental").len(), 2);
    assert_eq!(t.g.history.spells_cast.len(), 2);
}

#[test]
fn spells_cast_after_the_storm_spell_arent_counted() {
    cr!("702.40a");
    ruling!(
        "Wing Shards",
        "Storm counts spells cast before the spell with storm was cast. Spells cast after the spell with storm was cast but before the storm ability resolves aren’t counted."
    );
    let mut t = TestGame::new(2);
    let bears = p1_attacks_with_bears(&mut t, 3);
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    t.lands(P0, "Plains", 3);
    let ws = t.hand(P0, "Wing Shards");
    t.cast(P0, ws).target(P1).go();
    t.settle();
    // In response to the storm trigger, P0 casts another Lightning Bolt.
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve(); // the second Lightning Bolt
    t.resolve(); // the storm trigger
    assert_eq!(copies_of(&t, "Wing Shards").len(), 1);
    t.resolve_all();
    // The original and one copy: one attacker is left.
    assert_eq!(bears.iter().filter(|b| t.on_battlefield(**b)).count(), 1);
}

#[test]
fn permanent_storm_copies_arent_cast_and_arent_counted_later() {
    cr!("702.40a", "707.10f");
    ruling!(
        "Stormscale Scion",
        "The copies are put directly onto the stack. They aren’t cast and won’t be counted by other spells with storm cast later in the turn."
    );
    supported("Stormscale Scion");
    let mut t = TestGame::new(2);
    bolt(&mut t, P0, Entity::Player(P1));
    t.resolve_all();
    // Stormscale Scion: 4/4 flying; "Other Dragons you control get +1/+1." Storm; copies
    // become tokens.
    t.lands(P0, "Mountain", 6);
    let scion = t.hand(P0, "Stormscale Scion");
    t.cast(P0, scion).go();
    t.resolve_all();
    let scions = t.named_on_battlefield("Stormscale Scion");
    assert_eq!(scions.len(), 2);
    assert_eq!(token_names(&t, P0), vec!["Stormscale Scion".to_string()]);
    for s in &scions {
        assert_eq!(t.pt(*s), (5, 5));
    }
    // A later storm spell counts Lightning Bolt and Stormscale Scion, not the copy.
    t.lands(P0, "Forest", 2);
    let wts = t.hand(P0, "Weather the Storm");
    t.cast(P0, wts).go();
    t.resolve();
    assert_eq!(copies_of(&t, "Weather the Storm").len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 29);
}
