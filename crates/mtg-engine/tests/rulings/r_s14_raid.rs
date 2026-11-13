//! Rulings batch S14 — raid: "if you attacked this turn" (an ability word, CR 207.2c). The
//! condition looks at the whole turn (CR 508.1): whether its controller declared one or
//! more attacking creatures this turn, whatever happened to them or to what they attacked.

use crate::r_s01_common::*;
use crate::r_s05_common::tokens_with_subtype;
use crate::r_s14_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 casts Storm Fleet Spy ("Raid — When this creature enters, if you attacked this turn,
/// draw a card.") and everything resolves. Returns the number of cards drawn.
fn spy_draws(t: &mut TestGame) -> usize {
    let before = t.hand_size(P0);
    cast_from_hand(t, P0, "Storm Fleet Spy", &[]);
    t.resolve_all();
    t.hand_size(P0) - before
}

#[test]
fn raid_looks_back_even_if_the_attacker_and_what_it_attacked_are_gone() {
    cr!("508.1", "603.4", "104.3b");
    ruling!(
        "Storm Fleet Spy",
        "Raid abilities evaluate the entire turn to see if you attacked with a creature. That creature doesn't have to still be on the battlefield. Similarly, the player, planeswalker, or battle it attacked doesn't have to still be in the game or on the battlefield."
    );
    supported("Storm Fleet Spy");
    // Hill Giant attacks P1's Jace Beleren (loyalty 3), which is dealt 3 damage and put
    // into the graveyard; then the Giant is destroyed.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let jace = t.battlefield(P1, "Jace Beleren");
    t.attack(&[(giant, Entity::Object(jace))], &[]);
    assert!(t.in_graveyard(P1, "Jace Beleren"));
    crate::r_s02_common::destroy(&mut t, giant);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(spy_draws(&mut t), 1);
    // In a three-player game, the Giant attacks P2, who loses the game; then it's
    // destroyed.
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.players[P2.idx()].life = 3;
    t.attack(&[(giant, Entity::Player(P2))], &[]);
    assert!(t.has_lost(P2));
    crate::r_s02_common::destroy(&mut t, giant);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(spy_draws(&mut t), 1);
}

#[test]
fn raid_cares_only_that_you_attacked_with_a_creature() {
    cr!("508.1", "603.4");
    ruling!(
        "Storm Fleet Spy",
        "Raid abilities care only that you attacked with a creature. It doesn't matter how many creatures you attacked with or which player, planeswalker, or battle those creatures attacked."
    );
    // No attack: no card.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(spy_draws(&mut t), 0);
    // One creature attacking a planeswalker: one card.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    t.attack(&[(bears, Entity::Object(jace))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(spy_draws(&mut t), 1);
    // Three creatures attacking the player: still one card.
    let mut t = TestGame::new(2);
    let attackers: Vec<(ObjectId, Entity)> = (0..3)
        .map(|_| (t.battlefield(P0, "Grizzly Bears"), Entity::Player(P1)))
        .collect();
    t.attack(&attackers, &[]);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(spy_draws(&mut t), 1);
}

#[test]
fn an_end_step_raid_trigger_counts_attacks_made_before_it_entered() {
    cr!("508.1", "603.4", "603.2");
    ruling!(
        "Searslicer Goblin",
        "Some raid abilities trigger at the beginning of your end step. These abilities trigger if you attacked with a creature that turn, even if the permanent with that raid ability wasn't on the battlefield when you attacked."
    );
    supported("Searslicer Goblin");
    // Searslicer Goblin: "Raid — At the beginning of your end step, if you attacked this
    // turn, create a 1/1 red Goblin creature token." Cast after combat.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    cast_from_hand(&mut t, P0, "Searslicer Goblin", &[]);
    t.resolve_all();
    assert!(tokens_with_subtype(&t, P0, "Goblin").is_empty());
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Goblin").len(), 1);
    // Without an attack, nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Searslicer Goblin");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(tokens_with_subtype(&t, P0, "Goblin").is_empty());
}
