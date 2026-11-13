//! CR 702.114 Devoid.

use crate::common_k702_111_124::*;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn modify(t: &mut TestGame, id: ObjectId, mods: Vec<Modification>) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(id)]];
    t.g.exec(
        &Effect::Modify {
            what: Sel::Target(0),
            mods,
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
}

#[test]
fn a_card_with_devoid_is_colorless_in_every_zone() {
    cr!("702.114", "702.114a");
    ruling!(
        "Forerunner of Slaughter",
        "Devoid works in all zones, not just on the battlefield."
    );
    ruling!(
        "Forerunner of Slaughter",
        "A card with devoid is just colorless. It’s not colorless and the colors of mana in its mana cost."
    );
    assert_supported_card("Forerunner of Slaughter");
    let mut t = TestGame::new(2);
    // Forerunner of Slaughter: {B}{R}, devoid.
    let ids = [
        t.hand(P0, "Forerunner of Slaughter"),
        t.library_top(P0, "Forerunner of Slaughter"),
        t.graveyard(P0, "Forerunner of Slaughter"),
        t.exile(P0, "Forerunner of Slaughter"),
        t.battlefield(P0, "Forerunner of Slaughter"),
        t.custom(P0, (*card("Forerunner of Slaughter")).clone(), Zone::Outside(P0)),
    ];
    for id in ids {
        let o = t.g.obj(id);
        assert!(o.chars.colors.is_colorless(), "colorless in {:?}", o.zone);
        assert!(!o.chars.colors.contains(Color::Black));
    }
    // As a spell on the stack.
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    let c = t.hand(P0, "Forerunner of Slaughter");
    let spell = t.cast(P0, c).go();
    assert!(t.g.obj(spell).chars.colors.is_colorless());
    // Its own ability sees it as a colorless creature.
    t.resolve_all();
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    assert!(t.g.matches(t.g.current(c), &Filter::Colorless, &ctx));
}

#[test]
fn devoid_doesnt_change_color_identity() {
    cr!("702.114a");
    // Outside the game the card is colorless, but its color identity still includes the
    // colors of the mana symbols in its mana cost (CR 903.4).
    let def = card("Forerunner of Slaughter");
    assert!(def.front().chars.colors.is_colorless());
    assert!(def.color_identity.contains(Color::Black));
    assert!(def.color_identity.contains(Color::Red));
}

#[test]
fn other_effects_can_give_a_card_with_devoid_a_color() {
    cr!("702.114a");
    ruling!(
        "Forerunner of Slaughter",
        "Other cards and abilities can give a card with devoid color. If that happens, it’s just the new color, not that color and colorless."
    );
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Forerunner of Slaughter");
    // A characteristic-defining ability applies first in its layer (CR 604.3, 613.3):
    // an effect that makes it green overrides it.
    modify(
        &mut t,
        f,
        vec![Modification::SetColors(ColorSet::single(Color::Green))],
    );
    let colors = t.obj_now(f).chars.colors;
    assert!(colors.contains(Color::Green));
    assert_eq!(colors.count(), 1);
}

#[test]
fn a_card_that_loses_devoid_is_still_colorless() {
    cr!("702.114a");
    ruling!(
        "Forerunner of Slaughter",
        "If a card loses devoid, it will still be colorless. This is because effects that change an object’s color (like the one created by devoid) are considered before the object loses devoid."
    );
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Forerunner of Slaughter");
    modify(&mut t, f, vec![Modification::RemoveAllAbilities]);
    assert!(!has(&t, f, KeywordKind::Devoid));
    assert!(t.obj_now(f).chars.colors.is_colorless());
}
