//! Soul Sculptor (hand-written, `src/cards/soul_sculptor.rs`).

use mtg_engine::testing::*;
use mtg_engine::types::CardType;
use mtg_engine::*;

#[test]
fn an_enchantment_until_a_player_casts_a_creature_spell() {
    cr!("611.2b");
    ruling!("Soul Sculptor", "is just an enchantment with no abilities");
    let mut t = TestGame::new(2);
    let sculptor = t.battlefield(P0, "Soul Sculptor");
    let angel = t.battlefield(P1, "Serra Angel");
    t.lands(P0, "Plains", 2);
    t.activate(P0, sculptor, 0, &[Entity::Object(angel)]).unwrap();
    t.resolve_all();
    let o = t.obj_now(angel);
    assert!(o.is(CardType::Enchantment));
    assert!(!o.is_creature());
    assert!(o.chars.abilities.is_empty());
    // A noncreature spell doesn't end it.
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    t.resolve();
    assert!(!t.obj_now(angel).is_creature());
    // Next turn it still lasts; a creature spell ends it.
    t.lands(P0, "Forest", 2);
    let bear = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bear).go();
    assert!(t.obj_now(angel).is_creature());
}
