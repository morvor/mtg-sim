//! Rulings batch S09 — imprint: cards exiled with a permanent, copied and cast later
//! (CR 607.2a, 707.12), or copied as tokens (CR 111.4, 707.2).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::{activate_containing, attach_new};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Spellbinder enters under P0's control, exiling `instant` from P0's hand; it's attached
/// to P0's Grizzly Bears, which are returned.
fn spellbinder_with(t: &mut TestGame, instant: &str) -> ObjectId {
    supported("Spellbinder");
    let card = t.hand(P0, instant);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(card)]);
    let sb = t.enter(P0, "Spellbinder");
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Exile);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.g.attach(sb, Entity::Object(bears)));
    t.g.recompute();
    bears
}

/// The equipped Bears deal combat damage to P1; P0 answers the copy and cast questions.
fn hit(t: &mut TestGame, bears: ObjectId, copy: bool, cast: bool) {
    attack_with(t, &[(bears, Entity::Player(P1))]);
    t.answer_yes(P0, copy);
    t.answer_yes(P0, cast);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    block_and_finish(t, P1, &[]);
    t.resolve_all();
}

#[test]
fn copying_the_imprinted_card_and_casting_the_copy_are_both_optional() {
    cr!("707.12", "607.2a", "704.5e");
    ruling!(
        "Spellbinder",
        "The creation of the copy and then the casting of the copy are both optional."
    );
    // Declining to copy it.
    let mut t = TestGame::new(2);
    let bears = spellbinder_with(&mut t, "Lightning Bolt");
    hit(&mut t, bears, false, true);
    assert_eq!(t.life(P1), 18);
    assert!(t.g.history.spells_cast.is_empty());
    assert!(t.in_exile("Lightning Bolt"));
    // Copying it but not casting the copy: the copy ceases to exist.
    let mut t = TestGame::new(2);
    let bears = spellbinder_with(&mut t, "Lightning Bolt");
    hit(&mut t, bears, true, false);
    assert_eq!(t.life(P1), 18);
    assert!(t.g.history.spells_cast.is_empty());
    assert_eq!(t.g.find_in_zone(Zone::Exile, "Lightning Bolt").len(), 1);
    // Copying and casting it: the copy deals 3 damage; the card stays exiled.
    let mut t = TestGame::new(2);
    let bears = spellbinder_with(&mut t, "Lightning Bolt");
    hit(&mut t, bears, true, true);
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.g.history.spells_cast.len(), 1);
    assert_eq!(t.g.find_in_zone(Zone::Exile, "Lightning Bolt").len(), 1);
}

#[test]
fn the_copy_is_cast_without_paying_its_mana_cost_x_is_zero_additional_costs_are_paid() {
    cr!("118.9", "107.3b", "601.2b");
    ruling!(
        "Spellbinder",
        "You don't pay the spell's mana cost. If the spell has X in its mana cost, X is 0. You do pay any additional costs for that spell."
    );
    supported("Stroke of Genius");
    supported("Fling");
    // Stroke of Genius ({X}{2}{U}, "Target player draws X cards."): X is 0.
    let mut t = TestGame::new(2);
    let bears = spellbinder_with(&mut t, "Stroke of Genius");
    let hand = t.hand_size(P1);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.g.history.spells_cast.len(), 1);
    assert_eq!(t.hand_size(P1), hand);
    assert_eq!(tapped_lands(&t, P0), 0);
    // Fling ("As an additional cost to cast this spell, sacrifice a creature."): the Hill
    // Giant is sacrificed, and Fling deals 3 damage.
    let mut t = TestGame::new(2);
    let bears = spellbinder_with(&mut t, "Fling");
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    assert_eq!(t.life(P1), 20 - 2 - 3);
}

#[test]
fn a_token_copy_of_the_exiled_card_has_its_enters_abilities() {
    cr!("111.4", "707.2", "603.6a", "614.1c");
    ruling!(
        "Mimic Vat",
        "Any \"enters\" abilities of the exiled card will trigger when the token is put onto the battlefield. Any \"as [this permanent] enters\" or \"[this permanent] enters with\" abilities of the exiled card will also work."
    );
    supported("Mimic Vat");
    supported("Servant of the Scale");
    // Mulldrifter: "When this creature enters, draw two cards."
    let mut t = TestGame::new(2);
    let vat = t.battlefield(P0, "Mimic Vat");
    let md = t.battlefield(P1, "Mulldrifter");
    t.answer_yes(P0, true);
    destroy(&mut t, md);
    t.resolve_all();
    assert!(t.in_exile("Mulldrifter"));
    t.lands(P0, "Wastes", 3);
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, vat, "Create a token").expect("activate");
    t.resolve_all();
    let token = tokens(&t, P0);
    assert_eq!(token.len(), 1);
    assert_eq!(t.g.obj(token[0]).chars.name, "Mulldrifter");
    assert_eq!(t.hand_size(P0), hand + 2);
    // Servant of the Scale: "This creature enters with a +1/+1 counter on it." (0/0).
    let mut t = TestGame::new(2);
    let vat = t.battlefield(P0, "Mimic Vat");
    let servant = t.battlefield(P0, "Servant of the Scale");
    t.answer_yes(P0, true);
    destroy(&mut t, servant);
    t.resolve_all();
    assert!(t.in_exile("Servant of the Scale"));
    t.lands(P0, "Wastes", 3);
    activate_containing(&mut t, P0, vat, "Create a token").expect("activate");
    t.resolve_all();
    let token = tokens(&t, P0);
    assert_eq!(token.len(), 1);
    assert_eq!(t.counters(token[0], counters::PLUS1), 1);
    assert_eq!(t.pt(token[0]), (1, 1));
}
