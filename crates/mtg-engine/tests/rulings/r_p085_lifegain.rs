//! Rulings batch P085 — "players can't gain life" (Havoc Festival, Giant Cindermaw,
//! Rampaging Ferocidon): replacement effects on life gain, setting life totals, spells
//! that still resolve, and their other abilities.

use crate::r_p085_common::*;
use crate::r_s33_common::two_headed_giant;
use mtg_engine::ability::LibraryPosition;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const NO_GAIN: [&str; 3] = ["Havoc Festival", "Giant Cindermaw", "Rampaging Ferocidon"];

#[test]
fn cant_gain_life_beats_replacements() {
    cr!("614.17c", "119.10");
    ruling!(
        "Havoc Festival",
        "Effects that would replace gaining life with another effect won’t apply because it’s impossible for players to gain life."
    );
    supported("Havoc Festival");
    supported("Tainted Remedy");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Havoc Festival");
    t.battlefield(P0, "Tainted Remedy");
    let spell = t.hand(P1, "Revitalize");
    pool(&mut t, P1, &[(ManaType::W, 2)]);
    t.cast(P1, spell).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn set_life_total_higher_does_nothing() {
    cr!("119.5", "119.7");
    ruling!(
        "Giant Cindermaw",
        "If an effect says to set a player's life total to a number that's higher than the player's current life total while Giant Cindermaw is on the battlefield, the player's life total doesn't change."
    );
    ruling!(
        "Rampaging Ferocidon",
        "If an effect says to set a player's life total to a number that's higher than the player's current life total while Rampaging Ferocidon is on the battlefield, the player's life total doesn't change."
    );
    ruling!(
        "Havoc Festival",
        "If an effect says to set a player’s life total to a certain number and that number is higher than the player’s current life total, that part of the effect won’t do anything."
    );
    supported("Blessed Wind");
    for name in NO_GAIN {
        supported(name);
        for (start, end) in [(10, 10), (25, 20)] {
            let mut t = TestGame::new(2);
            t.battlefield(P1, name);
            t.g.player_mut(P0).life = start;
            let wind = t.hand(P0, "Blessed Wind");
            pool(&mut t, P0, &[(ManaType::W, 9)]);
            t.cast(P0, wind).target(P0).go();
            t.resolve_all();
            assert_eq!(t.life(P0), end, "{name} from {start}");
        }
    }
}

#[test]
fn life_gain_spells_still_resolve() {
    cr!("119.10", "608.2c");
    ruling!(
        "Giant Cindermaw",
        "Spells and abilities that cause players to gain life still resolve while Giant Cindermaw is on the battlefield."
    );
    ruling!(
        "Rampaging Ferocidon",
        "Spells and abilities that cause players to gain life still resolve while Rampaging Ferocidon is on the battlefield."
    );
    for name in ["Giant Cindermaw", "Rampaging Ferocidon"] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, name);
        let spell = t.hand(P0, "Revitalize");
        pool(&mut t, P0, &[(ManaType::W, 2)]);
        let hand = t.hand_size(P0);
        t.cast(P0, spell).go();
        t.resolve_all();
        assert_eq!(t.life(P0), 20, "{name}");
        // Revitalize also draws a card.
        assert_eq!(t.hand_size(P0), hand, "{name}");
        assert_eq!(t.zone(spell), Zone::Graveyard(P0));
    }
}

#[test]
fn rampaging_ferocidon_triggers() {
    cr!("603.6a", "603.10a");
    ruling!(
        "Rampaging Ferocidon",
        "If another creature enters the battlefield at the same time as Rampaging Ferocidon, its last ability triggers."
    );
    ruling!(
        "Rampaging Ferocidon",
        "Rampaging Ferocidon's last ability triggers whenever any player has a creature enter the battlefield, including you."
    );
    // Entering together with P1's Bears: P1 takes 1.
    let mut t = TestGame::new(2);
    let fero =
        t.g.create_card_object(card("Rampaging Ferocidon"), P0, Zone::Nowhere);
    let bears =
        t.g.create_card_object(card("Grizzly Bears"), P1, Zone::Nowhere);
    let mv = |obj, p| MoveEv {
        obj,
        to: Zone::Battlefield,
        pos: LibraryPosition::Top,
        cause: events::MoveCause::Effect,
        by: Some(p),
        etb: EtbInfo {
            controller: Some(p),
            ..Default::default()
        },
        source: None,
    };
    t.g.move_objects(vec![mv(fero, P0), mv(bears, P1)]);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 20);
    // P0's own creature: P0 takes 1.
    t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn havoc_festival_life_loss_on_resolution() {
    cr!("608.2h", "119.3");
    ruling!(
        "Havoc Festival",
        "The amount of life lost is calculated when the triggered ability resolves."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Havoc Festival");
    t.battlefield(P1, "Havoc Festival");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P1), 5);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn havoc_festival_two_headed_giant() {
    cr!("810.9", "608.2h");
    ruling!(
        "Havoc Festival",
        "In a Two-Headed Giant game, the last ability triggers for each player."
    );
    let mut t = two_headed_giant();
    t.battlefield(P0, "Havoc Festival");
    assert_eq!(t.life(P2), 30);
    // The P2/P3 team's upkeep: 30 - 15 - 8.
    t.advance_to(P2, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.life(P2), 7);
    assert_eq!(t.life(P3), 7);
    assert_eq!(t.life(P0), 30);
}
