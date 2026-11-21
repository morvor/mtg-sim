//! CR 106.6: mana with a spending restriction ("Spend this mana only to cast an instant or
//! sorcery spell") can pay only for what the restriction allows.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// P0 has Vodalian Arcanist ("{T}: Add {C}. Spend this mana only to cast an instant or
/// sorcery spell.") and a Forest; returns the game and the Arcanist's mana in P0's pool.
fn arcanist_mana() -> TestGame {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let arcanist = t.battlefield(P0, "Vodalian Arcanist");
    t.lands(P0, "Forest", 1);
    t.activate(P0, arcanist, 0, &[]).unwrap();
    assert_eq!(t.g.players[0].mana_pool.total(), 1);
    t
}

#[test]
fn instant_or_sorcery_mana_pays_for_an_instant_but_not_a_creature_spell() {
    cr!("106.6");
    let mut t = arcanist_mana();
    let bears = t.hand(P0, "Grizzly Bears");
    assert!(t.cast(P0, bears).try_go().is_err());
    let mut t = arcanist_mana();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let growth = t.hand(P0, "Titanic Growth");
    assert!(t
        .cast(P0, growth)
        .target(Entity::Object(bears))
        .try_go()
        .is_ok());
}
