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
