//! Rulings batch P208 — enchant (CR 303.4e, 704.5m): Auras with "enchant creature you
//! control" (or "an opponent controls") fall off when control of the Aura or the creature
//! changes, and Auras that return a creature they enchanted under your control.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::*;
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `aura` enchants P0's Grizzly Bears; P1 gains control of the Aura (`aura_moves`) or of
/// the Bears: the Aura is put into its owner's graveyard as a state-based action.
fn falls_off(aura: &str, aura_moves: bool) {
    supported(aura);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = attach_new(&mut t, P0, aura, bears);
    t.settle();
    assert!(t.on_battlefield(a), "{aura} is legal at first");
    give_control(&mut t, if aura_moves { a } else { bears }, P1);
    assert!(!t.on_battlefield(a), "{aura}");
    assert_eq!(t.zone(a), Zone::Graveyard(P0), "{aura}");
    assert!(t.on_battlefield(bears));
}

#[test]
fn enchant_creature_you_control_auras_fall_off_when_either_changes_control() {
    cr!("303.4e", "704.5m", "303.4a");
    ruling!(
        "Demonic Appetite",
        "If another player gains control of either Demonic Appetite or the enchanted creature (but not both), Demonic Appetite will be enchanting an illegal permanent."
    );
    ruling!(
        "Dying Wish",
        "If another player gains control of either Dying Wish or the enchanted creature (but not both), Dying Wish will be enchanting an illegal permanent."
    );
    ruling!(
        "Murder Investigation",
        "If another player gains control of either Murder Investigation or the enchanted creature (but not both), Murder Investigation will be enchanting an illegal permanent."
    );
    ruling!(
        "Inferno Fist",
        "If another player gains control of the enchanted creature, or if Inferno Fist becomes attached to a creature another player controls, Inferno Fist will be put into its owner's graveyard as a state-based action."
    );
    for aura in [
        "Demonic Appetite",
        "Dying Wish",
        "Murder Investigation",
        "Inferno Fist",
    ] {
        falls_off(aura, false);
        falls_off(aura, true);
    }
}

#[test]
fn enchant_creature_you_control_auras_fall_off_when_the_creature_changes_control() {
    cr!("303.4e", "704.5m");
    ruling!(
        "Flamespeaker's Will",
        "If another player gains control of the enchanted creature, Flamespeaker's Will will be put into its owner's graveyard."
    );
    ruling!(
        "Mortal Obstinacy",
        "If another player gains control of the enchanted creature, Mortal Obstinacy will be put into its owner's graveyard."
    );
    ruling!(
        "Journey to Eternity",
        "If another player gains control of the enchanted creature, Journey to Eternity will be put into your graveyard."
    );
    for aura in [
        "Flamespeaker's Will",
        "Mortal Obstinacy",
        "Journey to Eternity // Atzal, Cave of Eternity",
    ] {
        falls_off(aura, false);
    }
}

#[test]
fn betrayal_falls_off_a_creature_not_controlled_by_an_opponent() {
    cr!("303.4e", "704.5m");
    ruling!(
        "Betrayal",
        "If it ever enchants a creature that isn’t controlled by an opponent of Betrayal’s controller, then it’s put into the graveyard as a State-Based Action."
    );
    supported("Betrayal");
    // The creature's controller gains control of Betrayal.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let b = attach_new(&mut t, P0, "Betrayal", bears);
    t.settle();
    assert!(t.on_battlefield(b));
    give_control(&mut t, b, P1);
    assert!(t.in_graveyard(P0, "Betrayal"));
    // Betrayal's controller gains control of the creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Betrayal", bears);
    give_control(&mut t, bears, P0);
    assert!(t.in_graveyard(P0, "Betrayal"));
}

#[test]
fn journey_to_eternity_returns_a_creature_you_dont_own_under_your_control() {
    cr!("303.4e", "608.2c", "800.4a", "712.14a");
    ruling!(
        "Journey to Eternity",
        "If Journey to Eternity enchants a creature you control but don't own, the creature will return to the battlefield under your control from its owner's graveyard when it dies."
    );
    let journey = "Journey to Eternity // Atzal, Cave of Eternity";
    supported(journey);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    give_control(&mut t, bears, P0);
    attach_new(&mut t, P0, journey, bears);
    destroy(&mut t, bears);
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.g.obj(back[0]).controller, P0);
    assert_eq!(t.g.obj(back[0]).owner, P1);
    assert_eq!(t.named_on_battlefield("Atzal, Cave of Eternity").len(), 1);

    // Multiplayer: if the owner leaves the game, the creature leaves too; if you leave,
    // the creature you control from Journey's effect is exiled.
    for leaver in [P1, P0] {
        let mut t = TestGame::with_config(3, GameConfig::default());
        let bears = t.battlefield(P1, "Grizzly Bears");
        give_control(&mut t, bears, P0);
        attach_new(&mut t, P0, journey, bears);
        destroy(&mut t, bears);
        t.resolve_all();
        let back = t.named_on_battlefield("Grizzly Bears");
        assert_eq!(back.len(), 1);
        t.g.lose_game(leaver);
        t.settle();
        assert!(t.named_on_battlefield("Grizzly Bears").is_empty(), "{leaver:?} left");
        if leaver == P0 {
            assert!(t.in_exile("Grizzly Bears"));
        }
    }
}
