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

#[test]
fn hildibrand_cast_from_graveyard_only_as_an_adventure() {
    cr!("715.3", "601.3");
    assert_supported(&["Hildibrand Manderville // Gentleman's Rise"]);
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hildibrand Manderville // Gentleman's Rise");
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Plains", 2);
    t.g.destroy(h, None);
    t.settle();
    t.resolve_all();
    let card = t.g.current(h);
    assert!(matches!(t.zone(card), Zone::Graveyard(_)));
    // Not as the creature.
    assert!(t.cast(P0, card).try_go().is_err());
    t.cast(P0, card)
        .method(mtg_engine::object::CastMethod::Half(1))
        .go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Zombie Token").len() + t.named_on_battlefield("Zombie").len(), 1);
    // On an adventure: it may be cast as the creature from exile.
    assert!(t.in_exile("Hildibrand Manderville // Gentleman's Rise") || t.in_exile("Hildibrand Manderville"));
}

#[test]
fn risen_executioner_costs_more_for_each_other_creature_card() {
    cr!("601.2f", "601.3");
    assert_supported(&["Risen Executioner"]);
    let mut t = TestGame::new(2);
    let re = t.graveyard(P0, "Risen Executioner");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Lightning Bolt");
    t.lands(P0, "Swamp", 5);
    // {2}{B}{B} + {2}: six mana.
    assert!(t.cast(P0, re).try_go().is_err());
    t.lands(P0, "Swamp", 1);
    t.cast(P0, re).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Risen Executioner").len(), 1);
}

#[test]
fn ashiok_casts_face_up_cards_opponents_own_in_exile() {
    cr!("608.2g", "118.9");
    assert_supported(&["Ashiok, Nightmare Muse"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ashiok, Nightmare Muse");
    t.g.objects[a.0 as usize]
        .counters
        .insert("loyalty".into(), 7);
    let wurm = t.exile(P1, "Craw Wurm");
    let mine = t.exile(P0, "Hill Giant");
    t.answer_choose(P0, &[Entity::Object(wurm)]);
    t.activate(P0, a, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    assert_eq!(t.zone(mine), Zone::Exile);
}

#[test]
fn kaya_emblem_casts_a_legendary_spell_from_hand_graveyard_or_exile() {
    cr!("608.2g", "114.1");
    assert_supported(&["Kaya the Inexorable"]);
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kaya the Inexorable");
    t.g.objects[k.0 as usize]
        .counters
        .insert("loyalty".into(), 7);
    t.activate(P0, k, 2, &[]).unwrap();
    t.resolve_all();
    // A legendary creature card P0 owns in exile; another in an opponent's graveyard.
    let mine = t.exile(P0, "Isamaru, Hound of Konda");
    let theirs = t.graveyard(P1, "Isamaru, Hound of Konda");
    t.advance_to(P1, Step::End);
    t.answer_choose(P0, &[Entity::Object(mine)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 1);
    assert!(matches!(t.zone(theirs), Zone::Graveyard(_)));
}

#[test]
fn whispersteel_dagger_casts_a_creature_from_the_damaged_players_graveyard() {
    cr!("601.3", "609.4b", "608.2h");
    assert_supported(&["Whispersteel Dagger"]);
    let mut t = TestGame::new(3);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let dagger = t.battlefield(P0, "Whispersteel Dagger");
    t.g.attach(dagger, Entity::Object(bear));
    let giant = t.graveyard(P1, "Hill Giant");
    let other = t.graveyard(P2, "Hill Giant");
    t.attack(&[(bear, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    // Paid with black mana as though it were red.
    t.lands(P0, "Swamp", 4);
    assert!(t.cast(P0, other).try_go().is_err());
    t.cast(P0, giant).go();
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant");
    assert_eq!(g.len(), 1);
    assert_eq!(t.obj_now(g[0]).controller, P0);
}

#[test]
fn tinybones_the_pickpocket_casts_target_card_from_that_players_graveyard() {
    cr!("608.2g", "118.14");
    assert_supported(&["Tinybones, the Pickpocket"]);
    let mut t = TestGame::new(2);
    let tb = t.battlefield(P0, "Tinybones, the Pickpocket");
    let giant = t.graveyard(P1, "Hill Giant");
    t.lands(P0, "Swamp", 4);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    t.attack(&[(tb, Entity::Player(P1))], &[]);
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant");
    assert_eq!(g.len(), 1);
    assert_eq!(t.obj_now(g[0]).controller, P0);
}
