//! Round-trip follow-up `roundtrip-tail-3` (cards M–R): in-game tests for compiler
//! misreads the round trip found (each shows the behavior the corrected compilation has
//! and the old one didn't), and renderer checks.

use mtg_engine::card::card;
use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn enchant_artifact_or_non_aura_enchantment_enchants_a_plain_artifact() {
    cr!("702.5a", "303.4a");
    // Puppet Crafting: "Enchant artifact or non-Aura enchantment". The quality was read
    // as "artifact enchantment or non-Aura enchantment", so a plain artifact couldn't be
    // enchanted.
    supported("Puppet Crafting");
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Mind Stone");
    let aura = t.hand(P0, "Puppet Crafting");
    t.lands(P0, "Forest", 2);
    t.cast(P0, aura).target(stone).go();
    t.resolve();
    assert!(t.named_on_battlefield("Puppet Crafting").len() == 1);
    let o = t.obj_now(stone);
    assert!(o.is_creature(), "Mind Stone should be a creature");
    assert_eq!(t.pt(stone), (5, 5));
}

#[test]
fn for_each_card_drawn_a_choice_is_offered_for_each() {
    cr!("118.12a", "608.2c");
    // Read the Runes: "Draw X cards. For each card drawn this way, discard a card unless
    // you sacrifice a permanent." Each card drawn is its own choice; it was one choice
    // (sacrificing one permanent spared every discard).
    supported("Read the Runes");
    let mut t = TestGame::new(2);
    for _ in 0..4 {
        t.library_top(P0, "Island");
    }
    let spell = t.hand(P0, "Read the Runes");
    t.lands(P0, "Island", 3);
    t.battlefield(P0, "Memnite");
    // Sacrifice for the first card, not for the second.
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.cast(P0, spell).x(2).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 1, "one of the two cards drawn is discarded");
    // Three Islands and Memnite: one of them was sacrificed.
    let permanents = t.g.battlefield.len();
    assert_eq!(permanents, 3);
}

#[test]
fn damage_divided_among_x_targets_needs_exactly_x_targets() {
    cr!("601.2c", "601.2d");
    // Meteor Swarm: "deals 8 damage divided as you choose among X target creatures
    // and/or planeswalkers". The count was read as one to eight targets whatever X was,
    // so X = 1 could still split the damage among two creatures.
    supported("Meteor Swarm");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Meteor Swarm");
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Forest", 2);
    let two = [Entity::Object(a), Entity::Object(b)];
    // X = 1: asking for two targets isn't a legal choice; one target gets all 8.
    t.cast(P0, spell).x(1).targets(&two).go();
    t.resolve();
    let dead = [a, b].iter().filter(|o| !t.on_battlefield(**o)).count();
    assert_eq!(dead, 1, "exactly one creature was a target");
}

#[test]
fn never_more_targets_than_the_damage_divided() {
    cr!("601.2d");
    // Meteor Swarm: "8 damage divided as you choose among X target creatures and/or
    // planeswalkers". Each target gets at least 1 of the 8 damage, so X = 9 can't be
    // chosen (the targets can't be), while X = 8 gives each of eight targets 1.
    for (x, legal) in [(9, false), (8, true)] {
        let mut t = TestGame::new(2);
        for _ in 0..9 {
            t.battlefield(P1, "Grizzly Bears");
        }
        let spell = t.hand(P0, "Meteor Swarm");
        t.lands(P0, "Mountain", 4);
        t.lands(P0, "Forest", 8);
        let r = t.cast(P0, spell).x(x).try_go();
        assert_eq!(r.is_ok(), legal, "X = {x}");
        if legal {
            t.resolve();
            let bears = t.named_on_battlefield("Grizzly Bears");
            assert_eq!(bears.len(), 9);
            let damaged = bears.iter().filter(|b| t.obj_now(**b).damage == 1).count();
            assert_eq!(damaged, 8);
        }
    }
}
