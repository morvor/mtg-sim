//! Rulings batch S12 — opus (an ability word): "Whenever you cast an instant or sorcery
//! spell, [effect]. If five or more mana was spent to cast that spell, [effect] [instead]."

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
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

/// Hero tokens `p` controls (Tellah, Great Sage's).
fn heroes(t: &TestGame, p: PlayerId) -> usize {
    with_subtype(t, p, "Hero").len()
}

#[test]
fn tellah_deals_damage_equal_to_the_mana_spent_even_if_its_gone() {
    cr!("601.2h", "608.2h", "701.21a");
    ruling!(
        "Tellah, Great Sage",
        "If Tellah is no longer on the battlefield when its ability resolves and eight or more mana was spent to cast the spell that caused its ability to trigger, the ability will still deal damage to each opponent."
    );
    supported("Tellah, Great Sage");
    supported("Blaze");
    // Tellah, Great Sage: "Whenever you cast a noncreature spell, create a 1/1 colorless
    // Hero creature token. If four or more mana was spent to cast that spell, draw two
    // cards. If eight or more mana was spent to cast that spell, sacrifice Tellah and it
    // deals that much damage to each opponent."
    // Blaze with X = 2 (three mana): only a Hero.
    let mut t = TestGame::new(2);
    let tellah = t.battlefield(P0, "Tellah, Great Sage");
    t.lands(P0, "Mountain", 3);
    let blaze = t.hand(P0, "Blaze");
    let hand = t.hand_size(P0);
    t.cast(P0, blaze).x(2).target(P1).go();
    t.resolve_all();
    assert_eq!(heroes(&t, P0), 1);
    assert_eq!(t.hand_size(P0), hand - 1);
    assert_eq!(t.life(P1), 18);
    assert!(t.on_battlefield(tellah));
    // X = 7 with Thalia, Guardian of Thraben's {1} more: nine mana spent (Blaze's mana
    // value is 8). A Hero, two cards, then Tellah is sacrificed and deals 9 damage.
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Mountain", 9);
    let blaze = t.hand(P0, "Blaze");
    let hand = t.hand_size(P0);
    t.cast(P0, blaze).x(7).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(heroes(&t, P0), 2);
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    assert!(t.in_graveyard(P0, "Tellah, Great Sage"));
    assert_eq!(t.life(P1), 18 - 9);
    t.resolve_all();
    assert_eq!(t.life(P1), 18 - 9 - 7);
    // Tellah destroyed in response to its trigger: the ability still deals the damage.
    let mut t = TestGame::new(2);
    let tellah = t.battlefield(P0, "Tellah, Great Sage");
    t.lands(P0, "Mountain", 8);
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(7).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    destroy(&mut t, tellah);
    assert!(t.in_graveyard(P0, "Tellah, Great Sage"));
    t.resolve();
    assert_eq!(heroes(&t, P0), 1);
    assert_eq!(t.life(P1), 12);
    t.resolve_all();
    assert_eq!(t.life(P1), 5);
}

#[test]
fn tellahs_ability_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "608.2h");
    ruling!(
        "Tellah, Great Sage",
        "Tellah's ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tellah, Great Sage");
    t.lands(P0, "Mountain", 8);
    let blaze = t.hand(P0, "Blaze");
    let hand = t.hand_size(P0);
    let blaze = t.cast(P0, blaze).x(7).target(P1).go();
    t.settle();
    // The trigger is above Blaze.
    assert_eq!(t.stack_len(), 2);
    assert_ne!(*t.g.stack.last().unwrap(), blaze);
    // Blaze is countered in response: the trigger still resolves, with the eight mana
    // spent to cast it.
    let cs = in_hand_with_mana(&mut t, P1, "Counterspell");
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(blaze).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Blaze"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(heroes(&t, P0), 1);
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    assert!(t.in_graveyard(P0, "Tellah, Great Sage"));
    assert_eq!(t.life(P1), 12);
}
