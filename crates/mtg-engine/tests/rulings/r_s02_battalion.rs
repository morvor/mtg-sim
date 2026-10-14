//! Rulings batch S02 — battalion (an ability word, CR 207.2c): "Whenever this creature and
//! at least two other creatures attack, ..."

use crate::r_s01_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_battalion_trigger_resolves_even_if_fewer_creatures_are_still_attacking() {
    cr!("207.2c", "603.2", "506.4");
    ruling!(
        "Boros Elite",
        "Once a battalion ability has triggered, it doesn't matter how many creatures are still attacking when that ability resolves."
    );
    supported("Boros Elite");
    let mut t = TestGame::new(2);
    // "Battalion — Whenever this creature and at least two other creatures attack, this
    // creature gets +2/+2 until end of turn."
    let elite = t.battlefield(P0, "Boros Elite");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let p1 = Entity::Player(P1);
    attack_with(&mut t, &[(elite, p1), (a, p1), (b, p1)]);
    assert_eq!(triggers_on_stack(&t, "at least two other creatures attack"), 1);
    // Both other attackers leave combat (one is destroyed) before the trigger resolves.
    t.g.destroy(a, None);
    mtg_engine::combat::remove_from_combat(&mut t.g, b);
    t.settle();
    assert!(!t.on_battlefield(a));
    assert!(!t.g.is_attacking(b));
    t.resolve_all();
    assert_eq!(t.pt(elite), (3, 3));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn battalion_attackers_can_attack_different_players_and_planeswalkers() {
    cr!("207.2c", "508.1b");
    ruling!(
        "Boros Elite",
        "The three attacking creatures don't have to be attacking the same player, planeswalker, or battle."
    );
    supported("Boros Elite");
    let mut t = TestGame::new(3);
    let elite = t.battlefield(P0, "Boros Elite");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    attack_with(
        &mut t,
        &[
            (elite, Entity::Player(P1)),
            (a, Entity::Object(jace)),
            (b, Entity::Player(P2)),
        ],
    );
    assert_eq!(triggers_on_stack(&t, "at least two other creatures attack"), 1);
    t.resolve_all();
    assert_eq!(t.pt(elite), (3, 3));
}

#[test]
fn paladin_elizabeth_taggerdys_battalion_counts_attackers_of_different_defenders() {
    cr!("207.2c", "508.1b");
    ruling!(
        "Paladin Elizabeth Taggerdy",
        "The three attacking creatures don’t have to be attacking the same player, planeswalker, or battle."
    );
    supported("Paladin Elizabeth Taggerdy");
    let mut t = TestGame::new(2);
    // "Battalion — Whenever Paladin Elizabeth Taggerdy and at least two other creatures
    // attack, draw a card, then you may put a creature card with mana value X or less from
    // your hand onto the battlefield tapped and attacking, where X is Paladin Elizabeth
    // Taggerdy's power."
    let paladin = t.battlefield(P0, "Paladin Elizabeth Taggerdy");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    let hand = t.hand_size(P0);
    attack_with(
        &mut t,
        &[
            (paladin, Entity::Player(P1)),
            (a, Entity::Object(jace)),
            (b, Entity::Player(P1)),
        ],
    );
    assert_eq!(triggers_on_stack(&t, "at least two other creatures attack"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn battalion_doesnt_trigger_with_only_two_attackers() {
    cr!("207.2c", "603.2");
    let mut t = TestGame::new(2);
    let elite = t.battlefield(P0, "Boros Elite");
    let a = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(elite, Entity::Player(P1)), (a, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "at least two other creatures attack"), 0);
    t.resolve_all();
    assert_eq!(t.pt(elite), (1, 1));
}
