//! Market Gnome (hand-written, `src/cards/market_gnome.rs`).

use mtg_engine::ability::AbilityKind;
use mtg_engine::testing::*;
use mtg_engine::*;

fn craft_uid(t: &mut TestGame, src: ObjectId) -> u64 {
    t.g.recompute();
    t.g.obj(src)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)) && a.text.starts_with("Craft"))
        .map(|a| a.uid)
        .unwrap()
}

#[test]
fn triggers_when_exiled_as_a_craft_material() {
    cr!("702.167a", "603.10a");
    ruling!("Market Gnome", "you'll gain 1 life and draw a card before the transformed card returns");
    let mut t = TestGame::new(2);
    let landmark = t.battlefield(P0, "Oteclan Landmark");
    let gnome = t.battlefield(P0, "Market Gnome");
    t.lands(P0, "Plains", 3);
    let uid = craft_uid(&mut t, landmark);
    t.answer_choose(P0, &[Entity::Object(gnome)]);
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, landmark, uid).unwrap();
    t.g.flush_events();
    t.settle();
    // The Gnome's trigger is above the craft ability.
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.hand_size(P0), 1);
    assert!(t.named_on_battlefield("Oteclan Levitator").is_empty());
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Oteclan Levitator").len(), 1);
}

#[test]
fn no_trigger_when_exiled_otherwise() {
    cr!("603.2");
    let mut t = TestGame::new(2);
    let gnome = t.battlefield(P0, "Market Gnome");
    t.lands(P0, "Plains", 2);
    let s = t.hand(P0, "Swords to Plowshares");
    t.cast(P0, s).target(gnome).go();
    t.resolve_all();
    assert!(t.in_exile("Market Gnome"));
    // Swords: its controller gains life equal to its power (0); no card drawn.
    assert_eq!(t.hand_size(P0), 0);
}
