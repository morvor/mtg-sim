//! Riders on a permission to play cards, about the spells cast or lands played with it
//! (`oracle/patterns/permission_riders.rs`; CR 601.3, 400.7a, 614.1c, 118.9b, 601.2f):
//! "If you cast a spell this way, that artifact enters tapped", "... enters with a
//! finality counter on it", "If you do, it gains \"...\"", "Spells you cast this way cost
//! {2} less to cast", "If you do, it has affinity for artifacts". The same card cast
//! another way doesn't get the rider.

use mtg_engine::casting::PERMISSION_COST;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::turn::Step;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn on_bf(t: &TestGame, name: &str) -> ObjectId {
    let v = t.named_on_battlefield(name);
    assert_eq!(v.len(), 1, "{name} on the battlefield");
    v[0]
}

#[test]
fn edgar_artifact_cast_from_graveyard_enters_tapped() {
    cr!("614.1c", "400.7a", "601.3");
    assert_supported(&["Edgar, Master Machinist"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Edgar, Master Machinist");
    let thopter = t.graveyard(P0, "Ornithopter");
    t.cast(P0, thopter).go();
    t.resolve_all();
    let o = on_bf(&t, "Ornithopter");
    assert!(t.obj_now(o).tapped, "cast this way: enters tapped");
    // Once during each of your turns: a second artifact can't be cast this way.
    let other = t.graveyard(P0, "Ornithopter");
    assert!(t.cast(P0, other).try_go().is_err());
    // Cast from the hand: untapped.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Edgar, Master Machinist");
    let thopter = t.hand(P0, "Ornithopter");
    t.cast(P0, thopter).go();
    t.resolve_all();
    let o = on_bf(&t, "Ornithopter");
    assert!(!t.obj_now(o).tapped);
}

#[test]
fn leonardo_creature_cast_from_graveyard_gets_a_finality_counter() {
    cr!("614.1c", "400.7a", "122.6");
    assert_supported(&["Leonardo, Sewer Samurai"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leonardo, Sewer Samurai");
    let thopter = t.graveyard(P0, "Ornithopter");
    t.cast(P0, thopter).go();
    t.resolve_all();
    let o = on_bf(&t, "Ornithopter");
    assert_eq!(t.counters(o, "finality"), 1);
    // A 2/2 can't be cast from the graveyard this way.
    t.lands(P0, "Forest", 2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    // From the hand: no counter.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leonardo, Sewer Samurai");
    let thopter = t.hand(P0, "Ornithopter");
    t.cast(P0, thopter).go();
    t.resolve_all();
    assert_eq!(t.counters(on_bf(&t, "Ornithopter"), "finality"), 0);
}

#[test]
fn noctis_artifact_from_graveyard_costs_life_and_gets_a_finality_counter() {
    cr!("601.2f", "614.1c", "400.7a");
    assert_supported(&["Noctis, Prince of Lucis"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Noctis, Prince of Lucis");
    let thopter = t.graveyard(P0, "Ornithopter");
    t.cast(P0, thopter).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.counters(on_bf(&t, "Ornithopter"), "finality"), 1);
}

#[test]
fn osteomancer_adept_creature_cast_by_foraging_gets_a_finality_counter() {
    cr!("601.2f", "614.1c", "400.7a");
    assert_supported(&["Osteomancer Adept"]);
    let mut t = TestGame::new(2);
    let adept = t.battlefield(P0, "Osteomancer Adept");
    t.battlefield(P0, "Food");
    let thopter = t.graveyard(P0, "Ornithopter");
    t.activate(P0, adept, 0, &[]).unwrap();
    t.resolve_all();
    t.cast(P0, thopter).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Food").is_empty(), "foraged");
    assert_eq!(t.counters(on_bf(&t, "Ornithopter"), "finality"), 1);
}

#[test]
fn worldheart_phoenix_cast_from_graveyard_enters_with_counters() {
    cr!("118.9b", "614.1c");
    assert_supported(&["Worldheart Phoenix"]);
    let mut t = TestGame::new(2);
    let phoenix = t.graveyard(P0, "Worldheart Phoenix");
    for l in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        t.battlefield(P0, l);
    }
    t.cast(P0, phoenix)
        .method(CastMethod::Alternative(PERMISSION_COST))
        .go();
    t.resolve_all();
    let p = on_bf(&t, "Worldheart Phoenix");
    assert_eq!(t.counters(p, "+1/+1"), 2);
    assert_eq!(t.pt(p), (4, 4));
    // Cast from the hand: no counters.
    let mut t = TestGame::new(2);
    let phoenix = t.hand(P0, "Worldheart Phoenix");
    t.lands(P0, "Mountain", 4);
    t.cast(P0, phoenix).go();
    t.resolve_all();
    assert_eq!(t.counters(on_bf(&t, "Worldheart Phoenix"), "+1/+1"), 0);
}

#[test]
fn urianger_spells_cast_this_way_cost_two_less() {
    cr!("601.2f", "607.2a");
    assert_supported(&["Urianger Augurelt"]);
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Urianger Augurelt");
    let bears = t.library_top(P0, "Grizzly Bears");
    // Draw Arcanum: exile it face down.
    t.answer_yes(P0, true);
    t.activate(P0, u, 0, &[]).unwrap();
    t.resolve_all();
    let bears = t.g.current(bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    t.g.objects[u.0 as usize].tapped = false;
    t.activate(P0, u, 1, &[]).unwrap();
    t.resolve_all();
    // No mana needed for a {1}{G} spell that costs {2} less ... but {G} remains.
    assert!(t.cast(P0, bears).try_go().is_err());
    t.lands(P0, "Forest", 1);
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn chiss_goria_artifact_cast_from_among_them_has_affinity() {
    cr!("702.41a", "601.2f", "400.7a");
    assert_supported(&["Chiss-Goria, Forge Tyrant"]);
    let mut t = TestGame::new(2);
    let cg = t.battlefield(P0, "Chiss-Goria, Forge Tyrant");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    // Five cards; Steel Wall ({3}, artifact creature) on top.
    for _ in 0..4 {
        t.library_top(P0, "Island");
    }
    let wall = t.library_top(P0, "Steel Wall");
    t.lands(P0, "Mountain", 1);
    t.attack(&[(cg, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    let wall = t.g.current(wall);
    assert_eq!(t.zone(wall), Zone::Exile);
    // Two artifacts: {3} costs {1}.
    let spell = t.cast(P0, wall).go();
    t.g.recompute();
    assert!(t.g.obj(spell).has_keyword(KeywordKind::Affinity));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Steel Wall").len(), 1);
}

#[test]
fn the_eighth_doctor_land_played_from_graveyard_gains_the_exile_replacement() {
    cr!("400.7i", "614.1a");
    assert_supported(&["The Eighth Doctor"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Eighth Doctor");
    // Legendary land: historic.
    let land = t.graveyard(P0, "Urza's Saga");
    let land2 = t.graveyard(P0, "Academy Ruins");
    let _ = land;
    t.play_land(P0, land2).unwrap();
    t.resolve_all();
    let ruins = on_bf(&t, "Academy Ruins");
    // Destroyed: exiled instead.
    t.g.destroy(ruins, None);
    t.settle();
    assert!(t.in_exile("Academy Ruins"));
    assert!(!t.in_graveyard(P0, "Academy Ruins"));
}

#[test]
fn durnan_spell_cast_from_exile_has_undaunted() {
    cr!("702.125a", "601.2f");
    assert_supported(&["Durnan of the Yawning Portal"]);
    let mut t = TestGame::new(3);
    let d = t.battlefield(P0, "Durnan of the Yawning Portal");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.attack(&[(d, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    let bears = t.g.current(bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    // Two opponents: {1}{G} costs {G}.
    t.lands(P0, "Forest", 1);
    let spell = t.cast(P0, bears).go();
    t.g.recompute();
    assert!(t.g.obj(spell).has_keyword(KeywordKind::Undaunted));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn glimpse_the_cosmos_cast_from_graveyard_for_u_is_exiled() {
    cr!("118.9b", "614.1a");
    assert_supported(&["Glimpse the Cosmos"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 1);
    let g = t.graveyard(P0, "Glimpse the Cosmos");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    t.cast(P0, g)
        .method(CastMethod::Alternative(PERMISSION_COST))
        .go();
    t.resolve_all();
    assert!(t.in_exile("Glimpse the Cosmos"));
    // Without a Giant it can't be cast from the graveyard.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 2);
    let g = t.graveyard(P0, "Glimpse the Cosmos");
    assert!(t.cast(P0, g).try_go().is_err());
    assert!(t
        .cast(P0, g)
        .method(CastMethod::Alternative(PERMISSION_COST))
        .try_go()
        .is_err());
}

#[test]
fn tomb_of_aclazotz_creature_enters_with_finality_counter_as_a_vampire() {
    cr!("614.1c", "400.7a", "205.1b");
    assert_supported(&["Tarrian's Journal // The Tomb of Aclazotz"]);
    let mut t = TestGame::new(2);
    let tomb = t.battlefield(P0, "Tarrian's Journal // The Tomb of Aclazotz");
    assert!(mtg_engine::dfc::transform(&mut t.g, tomb));
    let tomb = t.g.current(tomb);
    t.lands(P0, "Forest", 2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    // (The Tomb's first ability is its mana ability.)
    let n = t
        .obj_now(tomb)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .count();
    t.activate(P0, tomb, n - 1, &[]).unwrap();
    t.resolve_all();
    let spell = t.cast(P0, bears).go();
    t.g.recompute();
    assert!(!t.g.obj(spell).chars.subtypes.iter().any(|s| s == "Vampire"));
    t.resolve_all();
    let b = on_bf(&t, "Grizzly Bears");
    assert_eq!(t.counters(b, "finality"), 1);
    assert!(t.obj_now(b).chars.subtypes.iter().any(|s| s == "Vampire"));
    assert!(t.obj_now(b).chars.subtypes.iter().any(|s| s == "Bear"));
}

#[test]
fn valgavoth_spells_cast_from_its_exile_cost_life() {
    cr!("118.9b", "607.2a");
    assert_supported(&["Valgavoth, Terror Eater"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Valgavoth, Terror Eater");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bears, None);
    t.settle();
    assert!(t.in_exile("Grizzly Bears"));
    let exiled = t.g.exile.clone()[0];
    t.cast(P0, exiled)
        .method(CastMethod::Alternative(PERMISSION_COST))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    let b = on_bf(&t, "Grizzly Bears");
    assert_eq!(t.obj_now(b).controller, P0);
}

#[test]
fn ignite_the_future_cast_from_graveyard_lets_the_cards_be_cast_free() {
    cr!("118.9", "702.34a");
    assert_supported(&["Ignite the Future"]);
    // Flashback: the exiled spell may be cast without paying its mana cost.
    let mut t = TestGame::new(2);
    let ignite = t.graveyard(P0, "Ignite the Future");
    t.lands(P0, "Mountain", 8);
    let bears = t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    t.cast(P0, ignite)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    t.resolve_all();
    let bears = t.g.current(bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    t.cast(P0, bears).method(CastMethod::Free).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Cast from the hand: the cards must be paid for.
    let mut t = TestGame::new(2);
    let ignite = t.hand(P0, "Ignite the Future");
    t.lands(P0, "Mountain", 4);
    let bears = t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    t.cast(P0, ignite).go();
    t.resolve_all();
    let bears = t.g.current(bears);
    assert!(t.cast(P0, bears).method(CastMethod::Free).try_go().is_err());
}

#[test]
fn nahiri_equipment_spells_cast_this_way_are_free() {
    cr!("118.9");
    assert_supported(&["Nahiri, Forged in Fury"]);
    let setup = |top: &str| {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Nahiri, Forged in Fury");
        let bear = t.battlefield(P0, "Grizzly Bears");
        let eq = t.battlefield(P0, "Bonesplitter");
        t.g.attach(eq, Entity::Object(bear));
        let card = t.library_top(P0, top);
        t.attack(&[(bear, Entity::Player(P1))], &[]);
        t.advance_to(P0, Step::PostcombatMain);
        let card = t.g.current(card);
        assert_eq!(t.zone(card), Zone::Exile);
        (t, card)
    };
    let (mut t, card) = setup("Short Sword");
    t.cast(P0, card).method(CastMethod::Free).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Short Sword").len(), 1);
    // Not an Equipment: only for its mana cost.
    let (mut t, card) = setup("Ornithopter");
    assert!(t.cast(P0, card).method(CastMethod::Free).try_go().is_err());
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
}

fn foods(t: &TestGame) -> usize {
    t.g.battlefield
        .iter()
        .filter(|o| t.g.obj(**o).chars.has_subtype("Food"))
        .count()
}

#[test]
fn the_fourth_doctor_creates_food_when_a_card_is_played_this_way() {
    cr!("400.7i", "603.2");
    assert_supported(&["The Fourth Doctor"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Fourth Doctor");
    let land = t.library_top(P0, "Academy Ruins");
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    assert_eq!(foods(&t), 1);
    // A land from the hand: no Food.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The Fourth Doctor");
    let land = t.hand(P0, "Academy Ruins");
    t.play_land(P0, land).unwrap();
    t.resolve_all();
    assert_eq!(foods(&t), 0);
}

#[test]
fn quintorius_spell_cast_this_way_goes_to_the_bottom_of_the_library() {
    cr!("614.1a", "118.9");
    assert_supported(&["Quintorius, Loremaster"]);
    let mut t = TestGame::new(2);
    let q = t.battlefield(P0, "Quintorius, Loremaster");
    t.lands(P0, "Mountain", 2);
    t.battlefield(P0, "Plains");
    t.library_top(P0, "Island");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    t.answer_targets(P0, &[Entity::Object(bolt)]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let bolt = t.g.current(bolt);
    assert_eq!(t.zone(bolt), Zone::Exile);
    t.activate(P0, q, 0, &[Entity::Object(bolt)]).unwrap();
    t.resolve_all();
    t.cast(P0, bolt)
        .method(CastMethod::Free)
        .target(Entity::Player(P1))
        .go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    let bottom = t.g.player(P0).library[0];
    assert_eq!(t.g.obj(bottom).chars.name, "Lightning Bolt");
    assert!(!t.in_graveyard(P0, "Lightning Bolt"));
}

#[test]
fn lukka_exiled_creature_cards_may_be_cast_while_you_control_a_lukka() {
    cr!("611.2c", "601.3");
    assert_supported(&["Lukka, Coppercoat Outcast"]);
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Lukka, Coppercoat Outcast");
    t.library_top(P0, "Island");
    t.library_top(P0, "Lightning Bolt");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.activate(P0, l, 0, &[]).unwrap();
    t.resolve_all();
    let bears = t.g.current(bears);
    assert_eq!(t.zone(bears), Zone::Exile);
    t.lands(P0, "Forest", 2);
    // The noncreature card gained nothing.
    let bolt = t.g.exile.iter().copied().find(|o| t.g.obj(*o).chars.name == "Lightning Bolt").unwrap();
    t.lands(P0, "Mountain", 1);
    assert!(t.cast(P0, bolt).target(Entity::Player(P1)).try_go().is_err());
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn lukka_permission_ends_without_a_lukka() {
    cr!("611.2c");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Lukka, Coppercoat Outcast");
    let bears = t.library_top(P0, "Grizzly Bears");
    t.activate(P0, l, 0, &[]).unwrap();
    t.resolve_all();
    let bears = t.g.current(bears);
    t.g.destroy(l, None);
    t.settle();
    t.lands(P0, "Forest", 2);
    assert!(t.cast(P0, bears).try_go().is_err());
}

#[test]
fn sinister_concierge_both_exiled_cards_gain_suspend() {
    cr!("702.62a");
    assert_supported(&["Sinister Concierge"]);
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Sinister Concierge");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.g.destroy(c, None);
    t.settle();
    t.resolve_all();
    let c = t.g.current(c);
    let giant = t.g.current(giant);
    for o in [c, giant] {
        assert_eq!(t.zone(o), Zone::Exile);
        assert_eq!(t.counters(o, "time"), 3);
        assert!(t.obj_now(o).has_keyword(KeywordKind::Suspend));
    }
}
