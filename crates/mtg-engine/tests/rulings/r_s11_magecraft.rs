//! Rulings batch S11 — magecraft (ability word, CR 207.2c): "Magecraft — Whenever you cast
//! or copy an instant or sorcery spell, [effect]." A copy of a spell put onto the stack
//! isn't cast (CR 707.10), so the "copy" half triggers for it; a copy of a card in
//! another zone (CR 707.12) triggers only once it's cast.

use crate::r_s01_common::*;
use crate::r_s02_common::can_cast;
use crate::r_s03_common::*;
use crate::r_s11_common::*;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Base power and toughness 8/8 from Octavia's magecraft ability.
fn is_8_8(t: &TestGame, id: ObjectId) -> bool {
    t.pt(id) == (8, 8)
}

#[test]
fn a_copy_of_an_instant_or_sorcery_spell_triggers_magecraft() {
    cr!("707.10", "603.2");
    ruling!(
        "Archmage Emeritus",
        "If an effect creates a copy of an instant or sorcery spell, this will also cause the magecraft ability to trigger."
    );
    supported("Archmage Emeritus");
    supported("Twincast");
    // Archmage Emeritus: "Magecraft — Whenever you cast or copy an instant or sorcery
    // spell, draw a card."
    let mut t = TestGame::new(2);
    let emeritus = t.battlefield(P0, "Archmage Emeritus");
    let bolt = in_hand_with_mana(&mut t, P0, "Lightning Bolt");
    let twincast = in_hand_with_mana(&mut t, P0, "Twincast");
    let hand = t.hand_size(P0);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    // Twincast ("Copy target instant or sorcery spell.") in response.
    t.cast(P0, twincast).target(bolt).go();
    t.resolve_all();
    assert_eq!(spells_copied(&t), 1);
    // Three triggers: casting the Bolt, casting Twincast, and the copy of the Bolt.
    assert_eq!(triggered_from(&t, emeritus), 3);
    assert_eq!(t.hand_size(P0), hand - 2 + 3);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn octavias_magecraft_also_triggers_for_a_copy() {
    cr!("707.10", "603.2");
    ruling!(
        "Octavia, Living Thesis",
        "If an effect creates a copy of an instant or sorcery spell, this will also cause the magecraft ability to trigger."
    );
    supported("Octavia, Living Thesis");
    // Octavia: "Magecraft — Whenever you cast or copy an instant or sorcery spell, target
    // creature has base power and toughness 8/8 until end of turn." Each trigger targets
    // a different Bears.
    let mut t = TestGame::new(2);
    let octavia = t.battlefield(P0, "Octavia, Living Thesis");
    let bears: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let bolt = in_hand_with_mana(&mut t, P0, "Lightning Bolt");
    let twincast = in_hand_with_mana(&mut t, P0, "Twincast");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.answer_targets(P0, &[Entity::Object(bears[0])]);
    t.settle();
    t.cast(P0, twincast).target(bolt).go();
    t.answer_targets(P0, &[Entity::Object(bears[1])]);
    t.settle();
    // Twincast resolves: the copy's trigger targets the third Bears.
    t.answer_yes(P0, false);
    t.answer_targets(P0, &[Entity::Object(bears[2])]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(triggered_from(&t, octavia), 3);
    assert!(is_8_8(&t, bears[0]) && is_8_8(&t, bears[1]) && is_8_8(&t, bears[2]));
    assert_eq!(t.pt(bears[3]), (2, 2));
}

#[test]
fn copying_a_card_in_another_zone_doesnt_trigger_magecraft_but_casting_the_copy_does() {
    cr!("707.12", "707.10", "702.99a");
    ruling!(
        "Archmage Emeritus",
        "Some effects instruct you to copy an instant or sorcery card in a zone other than the stack. These copies do not cause magecraft abilities to trigger. However, most effects that do this also allow you to cast the copy, and casting the copy will cause magecraft abilities to trigger."
    );
    ruling!(
        "Octavia, Living Thesis",
        "Some effects instruct you to copy an instant or sorcery card in a zone other than the stack. These copies do not cause magecraft abilities to trigger."
    );
    supported("Last Thoughts");
    // Last Thoughts ("Draw a card. Cipher") encoded on Hill Giant: when the Giant deals
    // combat damage to a player, the encoded card (in exile) is copied and the copy may
    // be cast.
    let mut t = TestGame::new(2);
    let emeritus = t.battlefield(P0, "Archmage Emeritus");
    let octavia = t.battlefield(P0, "Octavia, Living Thesis");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let thoughts = in_hand_with_mana(&mut t, P0, "Last Thoughts");
    t.cast(P0, thoughts).go();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.resolve_all();
    assert_eq!(triggered_from(&t, emeritus), 1);
    assert_eq!(triggered_from(&t, octavia), 1);
    assert_eq!(t.g.exile.len(), 1, "Last Thoughts encoded");
    let hand = t.hand_size(P0);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.attack(&[(giant, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Only the cast copy triggered: one more trigger each, not two.
    assert_eq!(spells_copied(&t), 0, "a copy of a card isn't a copied spell");
    assert_eq!(triggered_from(&t, emeritus), 2);
    assert_eq!(triggered_from(&t, octavia), 2);
    // Archmage Emeritus drew a card, and the copy drew one.
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn magecraft_triggers_once_for_each_copy_an_effect_creates() {
    cr!("702.40a", "707.10", "603.2c");
    ruling!(
        "Archmage Emeritus",
        "If an effect creates multiple copies of an instant or sorcery spell, magecraft abilities trigger once for each copy created by the effect."
    );
    ruling!(
        "Octavia, Living Thesis",
        "If an effect creates multiple copies of an instant or sorcery spell, magecraft abilities trigger once for each copy created by the effect."
    );
    supported("Grapeshot");
    supported("Opt");
    // Two Opts, then Grapeshot: its storm trigger copies it twice.
    let mut t = TestGame::new(2);
    let emeritus = t.battlefield(P0, "Archmage Emeritus");
    let octavia = t.battlefield(P0, "Octavia, Living Thesis");
    let bears: Vec<ObjectId> = (0..6).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    for b in &bears[..2] {
        let opt = in_hand_with_mana(&mut t, P0, "Opt");
        t.cast(P0, opt).go();
        t.answer_targets(P0, &[Entity::Object(*b)]);
        t.resolve_all();
    }
    let shot = in_hand_with_mana(&mut t, P0, "Grapeshot");
    let hand = t.hand_size(P0);
    t.cast(P0, shot).target(Entity::Player(P1)).go();
    t.answer_targets(P0, &[Entity::Object(bears[2])]);
    t.settle();
    assert_eq!(triggered_from(&t, emeritus), 3);
    // The storm trigger resolves: two copies (keeping their target), each triggering
    // both magecraft abilities.
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.answer_targets(P0, &[Entity::Object(bears[3])]);
    t.answer_targets(P0, &[Entity::Object(bears[4])]);
    t.resolve_all();
    assert_eq!(spells_copied(&t), 2);
    assert_eq!(t.life(P1), 17);
    // One trigger for casting Grapeshot and one for each of its two copies.
    assert_eq!(triggered_from(&t, emeritus), 5);
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
    assert_eq!(triggered_from(&t, octavia), 5);
    assert!(bears[..5].iter().all(|b| is_8_8(&t, *b)));
    assert_eq!(t.pt(bears[5]), (2, 2));
}

#[test]
fn magecraft_abilities_share_a_trigger_condition_but_differ_in_effect() {
    cr!("207.2c", "603.2", "603.3b");
    ruling!(
        "Archmage Emeritus",
        "Each magecraft ability has a different effect, although they all have the same trigger condition, whenever you cast or copy an instant or sorcery spell."
    );
    ruling!(
        "Octavia, Living Thesis",
        "Each magecraft ability has a different effect, although they all have the same trigger condition, whenever you cast or copy an instant or sorcery spell."
    );
    supported("Witherbloom Apprentice");
    supported("Storm-Kiln Artist");
    let mut t = TestGame::new(2);
    let emeritus = t.battlefield(P0, "Archmage Emeritus");
    let octavia = t.battlefield(P0, "Octavia, Living Thesis");
    let apprentice = t.battlefield(P0, "Witherbloom Apprentice");
    let artist = t.battlefield(P0, "Storm-Kiln Artist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let opt = in_hand_with_mana(&mut t, P0, "Opt");
    let hand = t.hand_size(P0);
    t.cast(P0, opt).go();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.settle();
    // All four trigger on the one spell.
    for src in [emeritus, octavia, apprentice, artist] {
        assert_eq!(triggered_from(&t, src), 1);
    }
    t.resolve_all();
    // Draw a card (and Opt's own draw); drain 1; base 8/8; a Treasure.
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    assert_eq!((t.life(P0), t.life(P1)), (21, 19));
    assert!(is_8_8(&t, bears));
    assert_eq!(with_subtype(&t, P0, "Treasure").len(), 1);
}

#[test]
fn casting_an_instant_or_sorcery_triggers_archmage_emeritus() {
    cr!("207.2c", "603.2", "601.2i");
    ruling!(
        "Witherbloom Apprentice",
        "For example, if you control Archmage Emeritus and cast an instant or sorcery spell, Archmage Emeritus’s magecraft ability will trigger and you will draw a card."
    );
    ruling!(
        "Octavia, Living Thesis",
        "For example, if you control Archmage Emeritus and cast an instant or sorcery spell, Archmage Emeritus's magecraft ability will trigger and you will draw a card."
    );
    ruling!(
        "Sedgemoor Witch",
        "For example, if you control Archmage Emeritus and cast an instant or sorcery spell, Archmage Emeritus's magecraft ability will trigger and you will draw a card."
    );
    supported("Sedgemoor Witch");
    supported("Divination");
    let mut t = TestGame::new(2);
    let emeritus = t.battlefield(P0, "Archmage Emeritus");
    t.battlefield(P0, "Witherbloom Apprentice");
    t.battlefield(P0, "Sedgemoor Witch");
    // A sorcery (Divination: "Draw two cards.") and an instant (Lightning Bolt).
    let div = in_hand_with_mana(&mut t, P0, "Divination");
    let bolt = in_hand_with_mana(&mut t, P0, "Lightning Bolt");
    let hand = t.hand_size(P0);
    t.cast(P0, div).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 1 + 2);
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(triggered_from(&t, emeritus), 2);
    assert_eq!(t.hand_size(P0), hand - 2 + 2 + 2);
    // The other magecraft abilities triggered too: two drains, two Pests.
    assert_eq!(t.life(P1), 20 - 2 - 3);
    assert_eq!(t.life(P0), 22);
    assert_eq!(with_subtype(&t, P0, "Pest").len(), 2);
    // A creature spell doesn't trigger magecraft.
    let bears = in_hand_with_mana(&mut t, P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(triggered_from(&t, emeritus), 2);
}

#[test]
fn octavia_costs_8_less_with_eight_instant_and_sorcery_cards_in_your_graveyard() {
    cr!("601.2f");
    // "This spell costs {8} less to cast if you have eight or more instant and/or sorcery
    // cards in your graveyard." ({8}{U}{U})
    for (instants, sorceries, castable) in [(4, 4, true), (4, 3, false), (0, 8, true)] {
        let mut t = TestGame::new(2);
        for _ in 0..instants {
            t.graveyard(P0, "Lightning Bolt");
        }
        for _ in 0..sorceries {
            t.graveyard(P0, "Divination");
        }
        // Creature cards don't count.
        t.graveyard(P0, "Grizzly Bears");
        t.lands(P0, "Island", 2);
        let octavia = t.hand(P0, "Octavia, Living Thesis");
        assert_eq!(
            can_cast(&mut t, P0, octavia, CastMethod::Normal),
            castable,
            "{instants} instants, {sorceries} sorceries"
        );
    }
}
