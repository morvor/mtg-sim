//! CR 608.2f, 603.10a: "each player ... sacrifices the rest" is processed for all players
//! simultaneously: a permanent that looks back at creatures dying sees the other players'
//! creatures that leave together with it.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn cataclysm_blood_artist_sees_the_creatures_sacrificed_with_it() {
    cr!("608.2f", "603.10a", "701.21a");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // "Whenever this creature or another creature dies, target player loses 1 life and
    // you gain 1 life."
    t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    let plains = t.lands(P0, "Plains", 4);
    // "Each player chooses from among the permanents they control an artifact, a creature,
    // an enchantment, and a land, then sacrifices the rest." P0 keeps the Bears (and a
    // Plains): Blood Artist is sacrificed together with one of P1's Bears.
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(plains[0])]);
    let c = t.hand(P0, "Cataclysm");
    t.cast(P0, c).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    // Two deaths seen: its own and P1's Bears.
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 22);
}
