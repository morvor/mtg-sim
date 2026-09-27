//! "Exile the top card of target player's library. You may cast that card for as long as
//! it remains exiled, and mana of any type can be spent to cast that spell." (Cruelclaw's
//! Heist, The Magic Bandit, Ramirez DePietro): a permission to cast the exiled card while
//! it stays in exile. It lets a spell be cast, never a land be played (CR 305.9).

use mtg_engine::card::{CardDef, Layout};
use mtg_engine::decision::Action;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{Characteristics, Zone};
use mtg_engine::oracle::{self, CompileContext};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;
use std::sync::Arc;

const TEXT: &str = "Exile the top card of target player's library. You may cast that card for as long as it remains exiled, and mana of any type can be spent to cast that spell.";

/// A {0} sorcery with `TEXT`, compiled with the real compiler.
fn heist() -> CardDef {
    let tl = TypeLine::parse("Sorcery");
    let ctx = CompileContext {
        card_name: "Top Heist",
        full_name: "Top Heist",
        type_line: &tl,
        layout: Layout::Normal,
        face_index: 0,
        keywords: &[],
        power: None,
        toughness: None,
    };
    let compiled = oracle::compile(TEXT, &ctx);
    assert!(
        compiled.unsupported.is_empty(),
        "{:?}",
        compiled.unsupported
    );
    CardDef::custom(Characteristics {
        name: SmolStr::new("Top Heist"),
        mana_cost: mtg_engine::mana::ManaCost::parse("{0}"),
        card_types: tl.card_types,
        abilities: compiled.abilities,
        rules_text: Arc::from(TEXT),
        ..Default::default()
    })
}

/// P0 casts the heist on P1, whose top library card is `top`. Returns the exiled card.
fn heist_top(t: &mut TestGame, top: &str) -> ObjectId {
    let card = t.library_top(P1, top);
    let spell = t.custom(P0, heist(), Zone::Hand(P0));
    t.cast(P0, spell).target(Entity::Player(P1)).go();
    t.resolve_all();
    let exiled = t.g.current(card);
    assert_eq!(t.zone(exiled), Zone::Exile);
    exiled
}

#[test]
fn the_exiled_spell_may_be_cast_with_mana_of_any_type() {
    cr!("118.14");
    assert!(card("Cruelclaw's Heist").unsupported_text().is_empty());
    let mut t = TestGame::new(2);
    let bolt = heist_top(&mut t, "Lightning Bolt");
    // {R} paid with green mana, on a later turn too.
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    t.g.players[P0.idx()].mana_pool.add_type(ManaType::G, 1);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn an_exiled_land_card_cant_be_played_with_a_permission_to_cast_it() {
    cr!("305.9");
    // Seat of the Synod is an artifact land: it can be played only as a land, never cast,
    // and the permission is only to cast.
    let mut t = TestGame::new(2);
    let seat = heist_top(&mut t, "Seat of the Synod");
    t.g.turn.priority = Some(P0);
    assert!(!t
        .g
        .legal_actions(P0)
        .contains(&Action::PlayLand { card: seat }));
    assert!(t.play_land(P0, seat).is_err());
    assert_eq!(t.zone(seat), Zone::Exile);
    // A land in hand can still be played this turn.
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).expect("play a land from hand");
}
