//! Lands losing their land types and abilities and gaining quoted mana abilities
//! (`src/oracle/patterns/statics.rs`, `pump_combat_tricks.rs`, `grant_grammar.rs`,
//! `kw/grant_filters.rs`; CR 205.3i, 305.7, 613.1d, 613.1f), grants conditioned on the
//! top card of the library, and a group narrowed by its controller's lands.

use mtg_engine::ability::AbilityKind;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn mana_abilities(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(&a.kind, AbilityKind::Activated(x) if x.is_mana_ability))
        .count()
}

#[test]
fn lithoform_blight_enchanted_land_loses_land_types_and_abilities() {
    cr!("205.3i", "305.7", "613.1f");
    ruling!(
        "Lithoform Blight",
        "keeps any other card types (such as artifact) and supertypes"
    );
    assert_supported(&["Lithoform Blight"]);
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let blight = t.battlefield(P0, "Lithoform Blight");
    t.g.attach(blight, Entity::Object(forest));
    t.settle();
    let o = t.obj_now(forest);
    assert!(!o.chars.has_subtype("Forest"));
    assert!(o.chars.card_types.contains(CardType::Land));
    assert!(o.chars.supertypes.contains(mtg_engine::types::Supertype::Basic));
    // "{T}: Add {C}" and "{T}, Pay 1 life: Add one mana of any color", not {G}.
    assert_eq!(mana_abilities(&t, forest), 2);
}

#[test]
fn ultima_blight_counter_land_loses_its_types_while_the_counter_stays() {
    cr!("611.2b", "305.7");
    ruling!(
        "Ultima, Origin of Oblivion",
        "keeps any other card types (such as artifact) and supertypes"
    );
    assert_supported(&["Ultima, Origin of Oblivion"]);
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Ultima, Origin of Oblivion");
    let island = t.battlefield(P1, "Island");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(u, Entity::Player(P1))], &[]);
    assert_eq!(t.counters(island, "blight"), 1);
    assert!(!t.obj_now(island).chars.has_subtype("Island"));
    assert_eq!(mana_abilities(&t, island), 1);
    t.g.remove_counters(Entity::Object(island), "blight", 1);
    t.settle();
    assert!(t.obj_now(island).chars.has_subtype("Island"));
}

#[test]
fn mul_daya_channelers_bonuses_follow_the_top_card() {
    cr!("611.3a");
    ruling!(
        "Mul Daya Channelers",
        "both a creature card and a land card (as Dryad Arbor is)"
    );
    assert_supported(&["Mul Daya Channelers"]);
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Mul Daya Channelers");
    t.library_top(P0, "Lightning Bolt");
    t.settle();
    assert_eq!(t.pt(m), (2, 2));
    assert_eq!(mana_abilities(&t, m), 0);
    t.library_top(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.pt(m), (5, 5));
    t.library_top(P0, "Dryad Arbor");
    t.settle();
    assert_eq!(t.pt(m), (5, 5));
    assert_eq!(mana_abilities(&t, m), 1);
}

#[test]
fn sheltering_prayers_basic_lands_of_players_with_few_lands_have_shroud() {
    cr!("611.3a", "702.18a");
    assert_supported(&["Sheltering Prayers"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sheltering Prayers");
    let few = t.lands(P0, "Plains", 3);
    let many = t.lands(P1, "Swamp", 4);
    t.settle();
    assert!(t.obj_now(few[0]).has_keyword(KeywordKind::Shroud));
    assert!(!t.obj_now(many[0]).has_keyword(KeywordKind::Shroud));
    // Only basic lands.
    let nonbasic = t.battlefield(P0, "Mutavault");
    t.settle();
    assert!(!t.obj_now(nonbasic).has_keyword(KeywordKind::Shroud));
    assert!(!t.obj_now(few[0]).has_keyword(KeywordKind::Shroud), "now four lands");
}

#[test]
fn alpine_moon_named_lands_lose_their_abilities_and_gain_any_color() {
    cr!("305.7", "613.1f");
    ruling!(
        "Alpine Moon",
        "won’t remove the artifact card type from an artifact land"
    );
    assert_supported(&["Alpine Moon"]);
    let mut t = TestGame::new(2);
    t.answer(
        P0,
        DecisionKind::Name,
        mtg_engine::decision::Answer::Text("Darksteel Citadel".into()),
    );
    t.enter(P0, "Alpine Moon");
    let theirs = t.battlefield(P1, "Darksteel Citadel");
    let mine = t.battlefield(P0, "Darksteel Citadel");
    let other = t.battlefield(P1, "Mutavault");
    t.settle();
    let o = t.obj_now(theirs);
    assert!(o.chars.card_types.contains(CardType::Artifact));
    assert!(!o.has_keyword(KeywordKind::Indestructible));
    assert_eq!(mana_abilities(&t, theirs), 1);
    assert!(o
        .chars
        .abilities
        .iter()
        .any(|a| a.text.to_lowercase().contains("one mana of any color")));
    // Not your own lands, nor lands with another name.
    assert!(t.obj_now(mine).has_keyword(KeywordKind::Indestructible));
    assert_eq!(mana_abilities(&t, other), 1);
    assert_eq!(t.obj_now(other).chars.abilities.len(), mtg_engine::card("Mutavault").faces[0].chars.abilities.len());
}
