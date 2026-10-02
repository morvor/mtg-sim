//! Rulings batch S32 — targets that must have a relationship with each other (see
//! `src/target_groups.rs`): "two target cards from an opponent's graveyard" are cards in
//! one opponent's graveyard, and "two target creature cards that share a creature type"
//! still share it on resolution, using the last known information of one that left.

use crate::r_s01_common::*;
use crate::r_s04_common::add_mana;
use mtg_engine::events::MoveCause;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn advocate_needs_two_cards_in_a_single_opponents_graveyard() {
    cr!("115.1", "601.2c", "602.2b");
    ruling!(
        "Nullmage Advocate",
        "You can’t activate this ability unless a single opponent has at least two cards in their graveyard to target."
    );
    supported("Nullmage Advocate");

    // Nullmage Advocate: each of two opponents has one card in their graveyard, so
    // there's no legal choice of two targets.
    let mut t = TestGame::new(3);
    let advocate = t.battlefield(P0, "Nullmage Advocate");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let bears = t.graveyard(P1, "Grizzly Bears");
    let elves = t.graveyard(P2, "Llanowar Elves");
    assert!(t.activate(P0, advocate, 0, &[]).is_err());
    assert_eq!(t.stack_len(), 0);

    // A second card in one opponent's graveyard: now it can be activated, but only with
    // both cards from that graveyard. Choosing one from each graveyard isn't allowed.
    t.graveyard(P2, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(bears), Entity::Object(elves)]);
    t.answer_targets(P0, &[Entity::Object(anthem)]);
    t.activate(P0, advocate, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.in_hand(P2, "Llanowar Elves"));
    assert!(t.in_hand(P2, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Glorious Anthem"));
}

#[test]
fn creature_cards_that_share_a_type_one_leaves_the_other_still_returns() {
    cr!("608.2b", "601.2c", "700.2a");
    ruling!(
        "Raise the Draugr",
        "you’ll still return the other card to your hand as long as it has a creature type that the other card had as it left your graveyard"
    );
    ruling!(
        "Return from Extinction",
        "you’ll still return the other card to your hand as long as it has a creature type that the other card had as it left your graveyard"
    );
    for name in ["Raise the Draugr", "Return from Extinction"] {
        supported(name);
        // Two Bears that share a creature type are targeted; one leaves the graveyard
        // before the spell resolves: the other is still returned.
        let mut t = TestGame::new(2);
        let spell = t.hand(P0, name);
        let bears = t.graveyard(P0, "Grizzly Bears");
        let runeclaw = t.graveyard(P0, "Runeclaw Bear");
        add_mana(&mut t, P0, ManaType::B, 2);
        t.cast(P0, spell)
            .modes(&[1])
            .targets(&[Entity::Object(bears), Entity::Object(runeclaw)])
            .go();
        t.g.move_object(runeclaw, Zone::Exile, MoveCause::Effect, Some(P1));
        t.resolve();
        assert!(t.in_hand(P0, "Grizzly Bears"), "{name}");
        assert!(t.in_exile("Runeclaw Bear"), "{name}");

        // Two creature cards that share no creature type can't be chosen together: a
        // pair that does is chosen instead.
        let mut t = TestGame::new(2);
        let spell = t.hand(P0, name);
        let bears = t.graveyard(P0, "Grizzly Bears");
        let elves = t.graveyard(P0, "Llanowar Elves");
        t.graveyard(P0, "Runeclaw Bear");
        add_mana(&mut t, P0, ManaType::B, 2);
        t.cast(P0, spell)
            .modes(&[1])
            .targets(&[Entity::Object(bears), Entity::Object(elves)])
            .go();
        t.resolve();
        assert!(t.in_hand(P0, "Grizzly Bears"), "{name}");
        assert!(t.in_hand(P0, "Runeclaw Bear"), "{name}");
        assert!(t.in_graveyard(P0, "Llanowar Elves"), "{name}");

        // With no two creature cards sharing a creature type, the second mode can't be
        // chosen: asking for it, P0 gets the first mode (one card returned) instead.
        let mut t = TestGame::new(2);
        let spell = t.hand(P0, name);
        let bears = t.graveyard(P0, "Grizzly Bears");
        let elves = t.graveyard(P0, "Llanowar Elves");
        add_mana(&mut t, P0, ManaType::B, 2);
        t.cast(P0, spell)
            .modes(&[1])
            .targets(&[Entity::Object(bears), Entity::Object(elves)])
            .try_go()
            .unwrap_or_else(|e| panic!("{name}: the first mode is still castable: {e:?}"));
        t.resolve();
        assert_eq!(t.hand_size(P0), 1, "{name}: only one card returned");
        assert_eq!(
            t.graveyard_size(P0),
            2,
            "{name}: the spell and the other card"
        );
    }
}
