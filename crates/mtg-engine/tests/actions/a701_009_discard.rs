//! CR 701.9: discard; CR 701.19: regenerate.

use crate::a701_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn a_card_discarded_into_a_hidden_zone_has_undefined_characteristics() {
    cr!("701.9c");
    ruling!(
        "Library of Leng",
        "The discard triggers anything else that triggers on discards."
    );
    supported("Library of Leng");
    supported("Megrim");
    supported("Mind Rot");
    let watcher = oracle_card(
        "Creature Watcher",
        "Enchantment",
        "{0}",
        None,
        "Whenever an opponent discards a creature card, you gain 3 life.",
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Library of Leng");
    t.battlefield(P0, "Megrim");
    t.custom(P0, watcher, Zone::Battlefield);
    let bears = t.hand(P1, "Grizzly Bears");
    let giant = t.hand(P1, "Hill Giant");
    t.lands(P0, "Swamp", 3);
    // P1 puts the Bears on top of their library (unrevealed), the Giant into the
    // graveyard.
    t.answer(
        P1,
        DecisionKind::Entities,
        Answer::Entities(vec![Entity::Object(bears), Entity::Object(giant)]),
    );
    let rot = t.hand(P0, "Mind Rot");
    // (Library of Leng's replacement is optional: P1 applies it only to the Bears.)
    t.answer_yes(P1, true);
    t.answer_yes(P1, false);
    t.cast(P0, rot).target(P1).go();
    t.resolve_all();
    let top = *t.g.player(P1).library.last().unwrap();
    assert_eq!(t.obj(top).chars.name.as_str(), "Grizzly Bears");
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Both were discarded: Megrim triggered twice.
    assert_eq!(t.life(P1), 16);
    // Only the creature card that went to the graveyard was seen as a creature card.
    assert_eq!(t.life(P0), 23);
}

#[test]
fn discarding_as_a_cost_isnt_caused_by_an_effect() {
    cr!("701.9a");
    ruling!(
        "Library of Leng",
        "You can't use the Library of Leng ability to place a discarded card on top of your library when you discard a card as a cost"
    );
    supported("Library of Leng");
    supported("Windcaller Aven");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Library of Leng");
    t.lands(P0, "Island", 1);
    let aven = t.hand(P0, "Windcaller Aven");
    t.answer_yes(P0, true);
    // Cycling: "{U}, Discard this card: Draw a card."
    let i =
        t.g.obj(aven)
            .chars
            .abilities
            .iter()
            .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
            .position(|a| a.text == "Cycling")
            .unwrap();
    t.activate(P0, aven, i, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Windcaller Aven"));
}

#[test]
fn discarding_to_pay_a_cost_as_an_ability_resolves_isnt_caused_by_an_effect() {
    cr!("118.12", "118.12a", "701.9a");
    ruling!(
        "Library of Leng",
        "You can't use the Library of Leng ability to place a discarded card on top of your library when you discard a card as a cost"
    );
    supported("Library of Leng");
    supported("Razormane Masticore");
    // "At the beginning of your upkeep, sacrifice this creature unless you discard a
    // card." The discard is a cost paid as the ability resolves (CR 118.12), not caused by
    // an effect, although no spell is being cast and no ability activated: Library of
    // Leng doesn't apply. (Were P0 asked about it, the second "yes" would put the card on
    // top of the library.)
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Library of Leng");
    let masticore = t.battlefield(P0, "Razormane Masticore");
    let forest = t.hand(P0, "Forest");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(masticore));
    assert_eq!(t.zone(forest), Zone::Graveyard(P0));
}

#[test]
fn a_static_regeneration_replaces_every_destruction() {
    cr!("701.19b");
    // "If this creature would be destroyed, regenerate it."
    assert!(card("Mossbridge Troll")
        .unsupported_text()
        .iter()
        .all(|u| !u.contains("regenerate")));
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Mossbridge Troll");
    for _ in 0..3 {
        t.g.untap(troll);
        t.g.deal_damage(troll, Entity::Object(troll), 5, false);
        t.settle();
        // Lethal damage destroys it (CR 704.5g): regenerated instead, each time.
        assert!(t.on_battlefield(troll));
        assert!(t.obj(troll).tapped);
        assert_eq!(t.obj(troll).damage, 0);
    }
    t.g.destroy(troll, None);
    assert!(t.on_battlefield(troll));
    // Sacrificing it isn't destroying it.
    t.g.sacrifice(troll, P0);
    assert!(t.in_graveyard(P0, "Mossbridge Troll"));
}
