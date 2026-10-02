//! Tokens whose power and toughness are a value ("create an X/X ... token, where X is
//! ..."): the creating effect defines the token's P/T, determined once as the token is
//! created (CR 111.3, 107.3a, 608.2h).

use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::types::Color;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn gelatinous_genesis_creates_x_x_by_x_tokens() {
    cr!("111.3", "107.3a");
    assert_supported("Gelatinous Genesis");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 7);
    let spell = t.hand(P0, "Gelatinous Genesis");
    t.cast(P0, spell).x(3).go();
    t.resolve_all();
    let oozes = t.named_on_battlefield("Ooze Token");
    assert_eq!(oozes.len(), 3);
    for o in oozes {
        assert_eq!(t.pt(o), (3, 3));
        assert!(t.obj_now(o).chars.colors.contains(Color::Green));
    }
}

#[test]
fn tumbleweed_rising_fixes_x_as_the_token_is_created() {
    cr!("608.2h", "111.3");
    assert_supported("Tumbleweed Rising");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 4);
    let spell = t.hand(P0, "Tumbleweed Rising");
    t.cast(P0, spell).go();
    t.resolve_all();
    let tok = t.named_on_battlefield("Elemental Token")[0];
    assert_eq!(t.pt(tok), (3, 3));
    // A later change to the greatest power doesn't change the token.
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(giant).go();
    t.resolve_all();
    assert_eq!(t.pt(giant), (6, 6));
    assert_eq!(t.pt(tok), (3, 3));
}

#[test]
fn value_pt_is_part_of_the_tokens_copiable_values() {
    cr!("111.3", "707.2");
    assert_supported("Formless Genesis");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Forest");
    t.graveyard(P0, "Island");
    t.lands(P0, "Forest", 6);
    let spell = t.hand(P0, "Formless Genesis");
    t.cast(P0, spell).go();
    t.resolve_all();
    let tok = t.named_on_battlefield("Shapeshifter Token")[0];
    assert_eq!(t.pt(tok), (2, 2));
    assert!(t.obj_now(tok).has_keyword(KeywordKind::Deathtouch));
    // More lands in the graveyard later: the token keeps its P/T, and a copy of it has
    // the same P/T (the creating effect's values are its copiable values).
    t.graveyard(P0, "Plains");
    t.lands(P0, "Island", 3);
    let copy = t.hand(P0, "Cackling Counterpart");
    t.cast(P0, copy).target(tok).go();
    t.resolve_all();
    let shapes = t.named_on_battlefield("Shapeshifter Token");
    assert_eq!(shapes.len(), 2);
    for s in shapes {
        assert_eq!(t.pt(s), (2, 2));
    }
}
