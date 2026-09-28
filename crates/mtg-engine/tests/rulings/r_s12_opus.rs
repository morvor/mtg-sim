//! Rulings batch S12 — opus (an ability word): "Whenever you cast an instant or sorcery
//! spell, [effect]. If five or more mana was spent to cast that spell, [effect] [instead]."

use crate::r_s01_common::*;
use crate::r_s03_common::in_hand_with_mana;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn an_opus_ability_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5", "608.2h");
    ruling!(
        "Tackle Artist",
        "An opus ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Tackle Artist");
    supported("Blaze");
    // Tackle Artist: "Opus — Whenever you cast an instant or sorcery spell, put a +1/+1
    // counter on this creature. If five or more mana was spent to cast that spell, put two
    // +1/+1 counters on this creature instead."
    let mut t = TestGame::new(2);
    let artist = t.battlefield(P0, "Tackle Artist");
    // Lightning Bolt: the trigger is above it and resolves while the Bolt waits.
    let bolt = in_hand_with_mana(&mut t, P0, "Lightning Bolt");
    let bolt = t.cast(P0, bolt).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_ne!(*t.g.stack.last().unwrap(), bolt);
    t.resolve();
    assert_eq!(t.counters(artist, counters::PLUS1), 1);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Blaze with X = 4 (five mana), countered in response to the trigger: the trigger
    // still resolves, and five mana was spent to cast that spell.
    t.lands(P0, "Mountain", 5);
    let blaze = t.hand(P0, "Blaze");
    let blaze = t.cast(P0, blaze).x(4).target(P1).go();
    t.settle();
    let cs = in_hand_with_mana(&mut t, P1, "Counterspell");
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(blaze).go();
    // Counterspell resolves first, then the opus trigger.
    t.resolve();
    assert!(t.in_graveyard(P0, "Blaze"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(artist, counters::PLUS1), 3);
    assert_eq!(t.life(P1), 17);
    // A creature spell doesn't trigger it.
    let bears = in_hand_with_mana(&mut t, P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve_all();
    assert_eq!(t.counters(artist, counters::PLUS1), 3);
}

#[test]
fn opus_mills_more_if_five_or_more_mana_was_spent() {
    cr!("601.2h", "608.2c");
    supported("Exhibition Tidecaller");
    supported("Mind Spring");
    // Exhibition Tidecaller: "target player mills three cards. If five or more mana was
    // spent to cast that spell, that player mills ten cards instead."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Exhibition Tidecaller");
    // Mind Spring with X = 2 (four mana): three cards.
    t.lands(P0, "Island", 4);
    let ms = t.hand(P0, "Mind Spring");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, ms).x(2).go();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 3);
    // X = 3 (five mana): ten cards.
    t.lands(P0, "Island", 5);
    let ms = t.hand(P0, "Mind Spring");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, ms).x(3).go();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P1), 13);
}
