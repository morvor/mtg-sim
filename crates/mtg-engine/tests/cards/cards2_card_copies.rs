//! Copies of cards cast as the ability resolves (CR 707.12): a card another player exiled
//! (Wildfire Devils), every card exiled with the source (Arcane Bombardment), a card in a
//! revealed hand (Reversal of Fortune).

use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn wildfire_devils_copies_the_card_the_random_player_exiles() {
    cr!("707.12", "608.2d");
    ruling!(
        "Wildfire Devils",
        "The copy of the card is created in exile and cast from exile, not from a graveyard."
    );
    ruling!(
        "Wildfire Devils",
        "The chosen player chooses which card to exile from their graveyard."
    );
    supported("Wildfire Devils");
    // Two players; only P1 has an instant or sorcery, so whoever is chosen at random,
    // either nothing happens (P0) or P1 exiles Opt.
    let mut copied = 0;
    for seed in 0..8 {
        let mut t = TestGame::new(2);
        t.g.rng = <rand_chacha::ChaCha8Rng as rand::SeedableRng>::seed_from_u64(seed);
        t.graveyard(P1, "Opt");
        t.answer_yes(P0, true);
        let hand = t.hand_size(P0);
        t.enter(P0, "Wildfire Devils");
        t.resolve_all();
        let exiled = t.in_exile("Opt");
        if exiled {
            copied += 1;
            // P0 cast the copy of P1's Opt: scry 1, draw a card.
            assert_eq!(t.hand_size(P0), hand + 1);
            assert!(!t.in_graveyard(P1, "Opt"));
        } else {
            assert_eq!(t.hand_size(P0), hand);
            assert!(t.in_graveyard(P1, "Opt"));
        }
    }
    assert!((1..8).contains(&copied), "{copied} of 8");
}

#[test]
fn arcane_bombardment_copies_every_card_exiled_with_it() {
    cr!("707.12", "607.2a");
    ruling!(
        "Arcane Bombardment",
        "You cast the desired copies as the triggered ability resolves."
    );
    supported("Arcane Bombardment");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Arcane Bombardment");
    t.graveyard(P0, "Divination");
    t.lands(P0, "Island", 1);
    let opt = t.hand(P0, "Opt");
    let hand = t.hand_size(P0);
    // Cast the copy of Divination.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.cast(P0, opt).go();
    t.resolve_all();
    assert!(t.in_exile("Divination"));
    // Opt left the hand (-1), Divination's copy drew two, Opt drew one.
    assert_eq!(t.hand_size(P0), hand - 1 + 2 + 1);
}

#[test]
fn reversal_of_fortune_casts_a_copy_of_a_card_in_the_opponents_hand() {
    cr!("707.12", "701.20");
    ruling!(
        "Reversal of Fortune",
        "Reversal of Fortune creates a copy of the card in the other player\u{2019}s hand, then allows you to cast the copy from that player\u{2019}s hand without paying its mana cost."
    );
    supported("Reversal of Fortune");
    let mut t = TestGame::new(2);
    t.hand(P1, "Lightning Bolt");
    t.lands(P0, "Mountain", 6);
    let r = t.hand(P0, "Reversal of Fortune");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.cast(P0, r).target(Entity::Player(P1)).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // The card itself stays in their hand.
    assert!(t.in_hand(P1, "Lightning Bolt"));
}
