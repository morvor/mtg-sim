//! Keywords granted to spells you cast and to cards in your hand
//! (`oracle/patterns/spell_keyword_grants.rs`; CR 610.5, 613.1f): costs equal to each
//! spell's or card's own mana cost (CR 702.119a, 702.152a, 702.143a, 702.94a), quoted
//! keyword lines, "the first spell you cast from exile each turn".

use mtg_engine::decision::{Action, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn keyword_count(t: &TestGame, id: ObjectId, k: KeywordKind) -> usize {
    t.g.obj(id).chars.keywords().filter(|x| x.kind == k).count()
}

#[test]
fn herigast_creature_spells_have_emerge_for_their_mana_cost() {
    cr!("702.119a", "610.5");
    assert_supported(&["Herigast, Erupting Nullkite"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herigast, Erupting Nullkite");
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.hand(P0, "Craw Wurm");
    t.lands(P0, "Forest", 3);
    // {4}{G}{G} reduced by Hill Giant's mana value 3: {1}{G}{G}.
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, wurm)
        .method(CastMethod::Keyword(KeywordKind::Emerge))
        .go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    // A noncreature spell has no emerge.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Herigast, Erupting Nullkite");
    t.battlefield(P0, "Hill Giant");
    let wall = t.hand(P0, "Divination");
    t.lands(P0, "Island", 3);
    assert!(t
        .cast(P0, wall)
        .method(CastMethod::Keyword(KeywordKind::Emerge))
        .try_go()
        .is_err());
}

#[test]
fn henzie_big_creature_spells_have_blitz_for_their_mana_cost() {
    cr!("702.152a", "610.5");
    assert_supported(&["Henzie \"Toolbox\" Torre"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Henzie \"Toolbox\" Torre");
    let wurm = t.hand(P0, "Craw Wurm");
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 8);
    t.cast(P0, wurm)
        .method(CastMethod::Keyword(KeywordKind::Blitz))
        .go();
    t.resolve_all();
    let w = t.named_on_battlefield("Craw Wurm")[0];
    assert!(t.obj_now(w).has_keyword(KeywordKind::Haste));
    // Mana value 2: no blitz.
    assert!(t
        .cast(P0, bears)
        .method(CastMethod::Keyword(KeywordKind::Blitz))
        .try_go()
        .is_err());
}

#[test]
fn bohn_cards_in_hand_have_foretell_for_their_mana_cost_reduced_by_two() {
    cr!("702.143a", "613.1f");
    assert_supported(&["Bohn, Beguiling Balladeer"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bohn, Beguiling Balladeer");
    let wurm = t.hand(P0, "Craw Wurm");
    let land = t.hand(P0, "Forest");
    t.g.recompute();
    assert_eq!(keyword_count(&t, wurm, KeywordKind::Foretell), 1);
    assert_eq!(keyword_count(&t, land, KeywordKind::Foretell), 0);
    let pool = &mut t.g.players[P0.idx()].mana_pool;
    pool.add_type(ManaType::C, 2);
    t.g.turn.priority = Some(P0);
    t.g.take_action(P0, Action::Special(SpecialAction::Foretell { card: wurm }));
    t.g.flush_events();
    let exiled = t.g.current(wurm);
    assert_eq!(t.zone(exiled), Zone::Exile);
    // Next turn: {2}{G}{G}.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Forest", 4);
    t.cast(P0, exiled)
        .method(CastMethod::Keyword(KeywordKind::Foretell))
        .go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Craw Wurm").len(), 1);
    assert_eq!(
        t.g.permanents()
            .filter(|o| o.chars.name == "Forest" && !o.tapped)
            .count(),
        0
    );
}

#[test]
fn aminatou_enchantment_cards_in_hand_have_miracle() {
    cr!("702.94a", "613.1f");
    assert_supported(&["Aminatou, Veil Piercer"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aminatou, Veil Piercer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // Sigil of the Empty Throne {3}{W}{W}: miracle {W}{W}.
    t.library_top(P0, "Sigil of the Empty Throne");
    t.lands(P0, "Plains", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.g.draw_cards(P0, 1);
    t.settle();
    t.resolve_all();
    let _ = bears;
    assert_eq!(t.named_on_battlefield("Sigil of the Empty Throne").len(), 1);
}

#[test]
fn zhulodok_colorless_spells_from_hand_have_two_cascades() {
    cr!("702.85a", "702.85c", "610.5");
    assert_supported(&["Zhulodok, Void Gorger"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zhulodok, Void Gorger");
    let golem = t.hand(P0, "Myr Battlesphere");
    t.lands(P0, "Plains", 7);
    let spell = t.cast(P0, golem).go();
    t.g.recompute();
    assert_eq!(keyword_count(&t, spell, KeywordKind::Cascade), 2);
    // Mana value 2: none.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zhulodok, Void Gorger");
    let x = t.hand(P0, "Ornithopter");
    let spell = t.cast(P0, x).go();
    t.g.recompute();
    assert_eq!(keyword_count(&t, spell, KeywordKind::Cascade), 0);
}

#[test]
fn wild_magic_sorcerer_first_spell_from_exile_has_cascade() {
    cr!("702.85a", "610.5", "715.4");
    assert_supported(&["Wild-Magic Sorcerer"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wild-Magic Sorcerer");
    t.lands(P0, "Mountain", 5);
    let giant = t.hand(P0, "Bonecrusher Giant");
    // Stomp, from the hand: no cascade.
    let stomp = t
        .cast(P0, giant)
        .method(CastMethod::Half(1))
        .target(Entity::Player(P1))
        .go();
    t.g.recompute();
    assert_eq!(keyword_count(&t, stomp, KeywordKind::Cascade), 0);
    t.resolve_all();
    let exiled = t.g.current(giant);
    assert_eq!(t.zone(exiled), Zone::Exile);
    // The creature, from exile: the first spell cast from exile this turn.
    let spell = t.cast(P0, exiled).go();
    t.g.recompute();
    assert_eq!(keyword_count(&t, spell, KeywordKind::Cascade), 1);
}

#[test]
fn ashling_elemental_permanent_spells_from_hand_gain_evoke() {
    cr!("702.74a", "610.5");
    assert_supported(&["Ashling, the Limitless"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ashling, the Limitless");
    let air = t.hand(P0, "Air Elemental");
    t.lands(P0, "Island", 4);
    t.cast(P0, air)
        .method(CastMethod::Keyword(KeywordKind::Evoke))
        .go();
    t.resolve_all();
    // Evoked: sacrificed as it enters.
    assert!(t.in_graveyard(P0, "Air Elemental"));
    // Not an Elemental: no evoke.
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 4);
    assert!(t
        .cast(P0, bears)
        .method(CastMethod::Keyword(KeywordKind::Evoke))
        .try_go()
        .is_err());
}

#[test]
fn abaddon_spells_with_mana_value_up_to_the_life_lost_have_cascade() {
    cr!("702.85a", "610.5");
    assert_supported(&["Abaddon the Despoiler"]);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Abaddon the Despoiler");
    t.lands(P0, "Forest", 4);
    let bears = t.hand(P0, "Grizzly Bears");
    // An opponent lost 2 life this turn: X is 2.
    t.g.lose_life(P1, 2);
    let spell = t.cast(P0, bears).go();
    t.g.recompute();
    assert_eq!(keyword_count(&t, spell, KeywordKind::Cascade), 1);
    t.resolve_all();
    let giant = t.hand(P0, "Hill Giant");
    t.lands(P0, "Mountain", 3);
    let spell = t.cast(P0, giant).go();
    t.g.recompute();
    assert_eq!(keyword_count(&t, spell, KeywordKind::Cascade), 0);
}

#[test]
fn yidris_spells_from_hand_this_turn_gain_cascade() {
    cr!("702.85a", "611.2f");
    assert_supported(&["Yidris, Maelstrom Wielder"]);
    let mut t = TestGame::new(2);
    let y = t.battlefield(P0, "Yidris, Maelstrom Wielder");
    t.attack(&[(y, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    t.lands(P0, "Forest", 4);
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Grizzly Bears");
    let s1 = t.cast(P0, a).go();
    t.g.recompute();
    assert_eq!(keyword_count(&t, s1, KeywordKind::Cascade), 1);
    t.resolve_all();
    // Every spell from the hand this turn, not only the first.
    let s2 = t.cast(P0, b).go();
    t.g.recompute();
    assert_eq!(keyword_count(&t, s2, KeywordKind::Cascade), 1);
}

#[test]
fn smoldering_stagecoach_next_instant_and_next_sorcery_have_cascade() {
    cr!("702.85a", "611.2f");
    assert_supported(&["Smoldering Stagecoach"]);
    let mut t = TestGame::new(2);
    let crew = t.battlefield(P0, "Hill Giant");
    let sc = t.battlefield(P0, "Smoldering Stagecoach");
    let _ = crew;
    // Run the attack trigger's effect directly.
    let e = mtg_engine::oracle::effects::parse_effect_text(
        "The next instant spell and the next sorcery spell you cast this turn each have cascade.",
        &mut mtg_engine::oracle::effects::Builder::new(&mtg_engine::oracle::CompileContext {
            card_name: "x",
            full_name: "x",
            type_line: &mtg_engine::types::TypeLine::parse("Artifact"),
            layout: mtg_engine::card::Layout::Normal,
            face_index: 0,
            keywords: &[],
            power: None,
            toughness: None,
        }),
    )
    .expect("parses");
    let mut ctx = mtg_engine::eval::Ctx::new(Some(sc), P0);
    t.g.exec(&e, &mut ctx);
    t.lands(P0, "Mountain", 3);
    let bolt = t.hand(P0, "Lightning Bolt");
    let bolt2 = t.hand(P0, "Lightning Bolt");
    let s = t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.g.recompute();
    assert_eq!(keyword_count(&t, s, KeywordKind::Cascade), 1);
    t.resolve_all();
    let s = t.cast(P0, bolt2).target(Entity::Player(P1)).go();
    t.g.recompute();
    assert_eq!(keyword_count(&t, s, KeywordKind::Cascade), 0);
}
