//! CR 702.73 Changeling.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_052_066::{on_top, run_effect};
use crate::k702_001_010_common::grant;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::eval::Ctx;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;
use smol_str::SmolStr;

/// Whether the object (followed across zones) matches "a [subtype]" as a filter would.
fn is_a(t: &TestGame, id: ObjectId, subtype: &str) -> bool {
    let id = t.g.current(id);
    t.g.matches(
        id,
        &Filter::Subtype(SmolStr::new(subtype)),
        &Ctx::new(None, P0),
    )
}

#[test]
fn a_changeling_is_every_creature_type() {
    cr!("702.73", "702.73a");
    ruling!(
        "Masked Vandal",
        "A creature card with changeling is just as much an Elf, a Dwarf, a Sliver, a Goat, a Coward, and a Zombie as it is a Shapeshifter."
    );
    assert_supported("Woodland Changeling");
    let mut t = TestGame::new(2);
    let wc = t.battlefield(P0, "Woodland Changeling");
    for ty in ["Shapeshifter", "Elf", "Dwarf", "Sliver", "Goat", "Coward", "Zombie", "Time Lord"] {
        assert!(t.obj_now(wc).chars.has_subtype(ty), "{ty}");
        assert!(is_a(&t, wc, ty), "{ty}");
    }
    // Not other kinds of subtypes.
    assert!(!is_a(&t, wc, "Forest"));
    assert!(!is_a(&t, wc, "Equipment"));
    // "Other Elf creatures you control get +1/+1."
    t.battlefield(P0, "Elvish Archdruid");
    assert_eq!(t.pt(wc), (3, 3));
}

#[test]
fn changeling_works_in_the_library() {
    cr!("702.73a");
    ruling!(
        "Masked Vandal",
        "Changeling is a characteristic-defining ability. It functions in all zones, not only while a card that has it is on the battlefield."
    );
    assert_supported("Goblin Matron");
    let mut t = TestGame::new(2);
    let wc = on_top(&mut t, P0, "Woodland Changeling");
    on_top(&mut t, P0, "Grizzly Bears");
    // "When this creature enters, you may search your library for a Goblin card ..."
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(wc)]);
    t.enter(P0, "Goblin Matron");
    t.resolve_all();
    let offered: Vec<Entity> = t
        .asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities {
                prompt, candidates, ..
            } if prompt.starts_with("Search") => Some(candidates),
            _ => None,
        })
        .unwrap_or_default();
    assert_eq!(offered, vec![Entity::Object(wc)]);
    assert!(t.in_hand(P0, "Woodland Changeling"));
}

#[test]
fn changeling_works_in_the_hand_even_on_a_kindred_card() {
    cr!("702.73a");
    ruling!(
        "Crib Swap",
        "Changeling applies in all zones, not just the battlefield."
    );
    assert_supported("Rustic Clachan");
    assert_supported("Crib Swap");
    let mut t = TestGame::new(2);
    // "As this land enters, you may reveal a Kithkin card from your hand. If you don't,
    // this land enters tapped."
    let swap = t.hand(P0, "Crib Swap");
    assert!(is_a(&t, swap, "Kithkin"));
    t.answer(P0, DecisionKind::OptionalCost, mtg_engine::decision::Answer::Bool(true));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(swap)]);
    let clachan = t.enter(P0, "Rustic Clachan");
    assert!(!t.obj_now(clachan).tapped);
}

#[test]
fn changeling_works_in_the_graveyard_and_outside_the_game() {
    cr!("702.73a");
    ruling!(
        "Yixlid Jailer",
        "If a card with changeling is in a graveyard, it still has all creature types."
    );
    let mut t = TestGame::new(2);
    let gy = t.graveyard(P0, "Woodland Changeling");
    assert!(is_a(&t, gy, "Merfolk"));
    let exiled = t.exile(P0, "Woodland Changeling");
    assert!(is_a(&t, exiled, "Giant"));
    let def = mtg_engine::card::card("Woodland Changeling");
    let outside = t.custom(P0, (*def).clone(), Zone::Outside(P0));
    t.g.recompute();
    assert!(t.obj_now(outside).chars.has_subtype("Wizard"));
}

#[test]
fn a_changeling_that_loses_all_abilities_is_still_every_creature_type() {
    cr!("702.73a", "613.1d");
    ruling!(
        "Graveshifter",
        "If an effect causes a creature with changeling to lose all abilities, it will remain all creature types, even though it will no longer have changeling."
    );
    let mut t = TestGame::new(2);
    let wc = t.battlefield(P0, "Woodland Changeling");
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(wc)],
    );
    assert!(!t.obj_now(wc).chars.has_keyword(KeywordKind::Changeling));
    assert!(is_a(&t, wc, "Elf"));
    assert!(is_a(&t, wc, "Goblin"));
}

#[test]
fn an_effect_setting_its_creature_type_overwrites_changeling() {
    cr!("702.73a");
    ruling!(
        "Graveshifter",
        "It will still have changeling; the effect making it all creature types will simply be overwritten."
    );
    let mut t = TestGame::new(2);
    let wc = t.battlefield(P0, "Woodland Changeling");
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::SetTypes {
                types: vec![mtg_engine::types::CardType::Creature],
                subtypes: vec![SmolStr::new("Treefolk")],
            }],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(wc)],
    );
    assert!(t.obj_now(wc).chars.has_keyword(KeywordKind::Changeling));
    assert!(is_a(&t, wc, "Treefolk"));
    assert!(!is_a(&t, wc, "Elf"));
}

#[test]
fn lignify_makes_a_changeling_only_a_treefolk() {
    cr!("702.73a");
    ruling!(
        "Lignify",
        "or that specify its creature types (such as changeling does)"
    );
    assert_supported("Lignify");
    let mut t = TestGame::new(2);
    let wc = t.battlefield(P1, "Woodland Changeling");
    t.lands(P0, "Forest", 2);
    let lignify = t.hand(P0, "Lignify");
    t.cast(P0, lignify).target(wc).go();
    t.resolve_all();
    assert!(is_a(&t, wc, "Treefolk"));
    assert!(!is_a(&t, wc, "Elf"));
    assert_eq!(t.pt(wc), (0, 4));
}

#[test]
fn a_creature_given_changeling_by_an_effect_is_every_creature_type() {
    cr!("702.73a");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(!is_a(&t, bears, "Goblin"));
    grant(&mut t, bears, Keyword::new(KeywordKind::Changeling));
    assert!(is_a(&t, bears, "Goblin"));
    assert!(is_a(&t, bears, "Bear"));
}
