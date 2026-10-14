//! Where a permission lets cards be played from (CR 601.3, 305.1; `permission_grammar`):
//! the top of a graveyard, any graveyard, cards in exile with a counter, a face-down card
//! the player may look at.

use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn bosium_strip_casts_only_the_top_card_of_your_graveyard() {
    cr!("601.3", "614.1a");
    assert_supported(&["Bösium Strip"]);
    let mut t = TestGame::new(2);
    let strip = t.battlefield(P0, "Bösium Strip");
    t.lands(P0, "Mountain", 5);
    let shock = t.graveyard(P0, "Shock");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.activate(P0, strip, 0, &[]).unwrap();
    t.resolve_all();
    // Shock isn't on top.
    assert!(t.cast(P0, shock).target(Entity::Player(P1)).try_go().is_err());
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
    // Now Shock is on top.
    t.cast(P0, shock).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert!(t.in_exile("Shock"));
}

/// A sorcery with `text`, compiled.
fn sorcery(name: &str, text: &str) -> mtg_engine::card::CardDef {
    let tl = mtg_engine::types::TypeLine::parse("Sorcery");
    let ctx = mtg_engine::oracle::CompileContext {
        card_name: name,
        full_name: name,
        type_line: &tl,
        layout: mtg_engine::card::Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let compiled = mtg_engine::oracle::compile(text, &ctx);
    assert!(compiled.unsupported.is_empty(), "{:?}", compiled.unsupported);
    mtg_engine::card::CardDef::custom(mtg_engine::object::Characteristics {
        name: name.into(),
        mana_cost: mtg_engine::mana::ManaCost::parse("{0}"),
        card_types: tl.card_types,
        abilities: compiled.abilities,
        rules_text: std::sync::Arc::from(text),
        ..Default::default()
    })
}

#[test]
fn urabrask_chapter_three_casts_from_any_graveyard() {
    cr!("601.3");
    assert_supported(&["Urabrask // The Great Work"]);
    // The Great Work's chapter III permission.
    let mut t = TestGame::new(2);
    let s = t.custom(
        P0,
        sorcery(
            "Great Work III",
            "Until end of turn, you may cast instant and sorcery spells from any graveyard.",
        ),
        Zone::Hand(P0),
    );
    t.cast(P0, s).go();
    t.resolve_all();
    t.lands(P0, "Mountain", 1);
    let bolt = t.graveyard(P1, "Lightning Bolt");
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    t.lands(P0, "Forest", 2);
    assert!(t.cast(P0, bears).try_go().is_err());
}

#[test]
fn lara_croft_plays_a_card_with_a_discovery_counter() {
    cr!("601.3", "122.1");
    assert_supported(&["Lara Croft, Tomb Raider"]);
    let mut t = TestGame::new(2);
    let lara = t.battlefield(P0, "Lara Croft, Tomb Raider");
    let ruins = t.graveyard(P1, "Academy Ruins");
    t.answer_targets(P0, &[Entity::Object(ruins)]);
    t.attack(&[(lara, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    let ruins = t.g.current(ruins);
    assert_eq!(t.zone(ruins), Zone::Exile);
    assert_eq!(t.counters(ruins, "discovery"), 1);
    t.play_land(P0, ruins).unwrap();
    assert_eq!(t.named_on_battlefield("Academy Ruins").len(), 1);
}

#[test]
fn headliner_scarlett_looks_at_and_plays_the_exiled_card() {
    cr!("406.3", "601.3");
    assert_supported(&["Headliner Scarlett"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Headliner Scarlett");
    t.lands(P0, "Forest", 2);
    t.library_top(P0, "Island");
    t.library_top(P0, "Grizzly Bears");
    t.advance_to(P1, Step::End);
    let bears = t.g.library_top(P0).unwrap();
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let bears = t.g.current(bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(mtg_engine::facedown::can_look_at(&t.g, P0, bears));
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P1, bears));
    t.advance_to(P0, Step::PrecombatMain);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}
