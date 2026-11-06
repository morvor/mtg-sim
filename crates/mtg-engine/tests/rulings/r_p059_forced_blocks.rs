//! Rulings batch P059 — "blocks this turn if able" requirements (CR 509.1c): a creature
//! that's tapped or can't block, or whose controller isn't being attacked, doesn't block,
//! and one whose block has a cost isn't forced to pay it (CR 509.1d); the defending player
//! chooses which attacker it blocks; Predatory Rampage's requirement applies to every
//! creature its opponents control, but its bonus only to the creatures it affected as it
//! resolved (CR 611.2c).

use crate::r_p058_common::mana;
use crate::r_p059_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s06_common::{attach_new, give_control};
use crate::r_s09_common::declare;
use mtg_engine::decision::Answer;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Unable {
    No,
    Tapped,
    /// Enchanted by Pacifism.
    CantBlock,
    /// No creature is attacking its controller or that player's planeswalkers (P0
    /// attacks P2 instead).
    NotAttacked,
    /// Its controller is attacked only through a planeswalker (it still must block).
    WalkerAttacked,
    /// War Cadence for 1 ("This turn, creatures can't block unless their controller pays
    /// {X} for each blocking creature they control."); its controller has the mana.
    Cost,
}
use Unable::*;

/// P0 applies `name`'s block requirement to P1's Grizzly Bears in P0's precombat main
/// phase (P0 casts or activates it; Courtly Provocateur's second ability).
fn apply(t: &mut TestGame, name: &str, c: ObjectId) {
    supported(name);
    mana(t, P0, 3);
    match name {
        "Courtly Provocateur" => {
            let s = t.battlefield(P0, name);
            let i = crate::r_p040_common::ability(t, s, "{T}: Target creature blocks");
            t.activate(P0, s, i, &[o(c)]).unwrap();
        }
        "Predatory Rampage" => {
            cast(t, P0, name, &[]);
        }
        "Hustle // Bustle" => {
            let s = t.hand(P0, name);
            t.cast(P0, s).method(CastMethod::Half(0)).target(c).go();
        }
        _ => {
            cast(t, P0, name, &[o(c)]);
        }
    }
    t.resolve_all();
}

/// Whether P1's Grizzly Bears blocks P0's Hill Giant (attacking P1, or P1's Garruk
/// Wildspeaker) when P1 declares no blockers, with `name`'s requirement on the Bears.
fn blocks(name: &str, why: Unable) -> bool {
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let walker = t.battlefield(P1, "Garruk Wildspeaker");
    t.lands(P1, "Plains", 2);
    apply(&mut t, name, bears);
    match why {
        Tapped => {
            t.g.tap(bears);
        }
        CantBlock => {
            attach_new(&mut t, P0, "Pacifism", bears);
        }
        Cost => {
            let cadence = t.battlefield(P0, "War Cadence");
            t.answer(P0, DecisionKind::X, Answer::Number(1));
            t.activate(P0, cadence, 0, &[]).unwrap();
            t.resolve_all();
        }
        _ => {}
    }
    let attackers = match why {
        NotAttacked => vec![(giant, pl(P2))],
        WalkerAttacked => vec![(giant, o(walker))],
        _ => vec![(giant, pl(P1))],
    };
    attack_with(&mut t, &attackers);
    block(&mut t, P1, &[]);
    t.g.is_blocking(bears)
}

fn check(name: &str, cases: &[Unable]) {
    assert!(blocks(name, No), "{name}: blocks when able");
    assert!(blocks(name, WalkerAttacked), "{name}: its planeswalker attacked");
    for why in cases {
        assert!(!blocks(name, *why), "{name}: {why:?}");
    }
}

#[test]
fn forced_blockers_dont_block_when_unable_or_when_it_costs() {
    cr!("509.1c", "509.1d", "509.1a");
    ruling!(
        "Courtly Provocateur",
        "After the second ability resolves, the creature blocks only if it’s able to do so as the declare blockers step begins. If, at that time, the creature is tapped, is affected by a spell or ability that says it can’t block, or no creatures are attacking that player or a planeswalker controlled by that player, then it doesn’t block. If there’s a cost associated with having the creature block, the player isn’t forced to pay that cost, so it doesn’t block in that case either."
    );
    check("Courtly Provocateur", &[Tapped, CantBlock, NotAttacked, Cost]);
    ruling!(
        "Culling Mark",
        "The creature blocks only if it’s able to do so as the declare blockers step begins. If, at that time, the creature is tapped, it’s affected by a spell or ability that says it can’t block, or no creatures are attacking its controller or a planeswalker controlled by that player, then it doesn’t block. If there’s a cost associated with having the creature block, the player isn’t forced to pay that cost. If that cost isn’t paid, the creature won’t block."
    );
    check("Culling Mark", &[Tapped, CantBlock, NotAttacked, Cost]);
    ruling!(
        "Predatory Rampage",
        "Each creature controlled by an opponent blocks only if it's able to do so as the declare blockers step begins. If, at that time, the creature is tapped, is affected by a spell or ability that says it can't block, or no creatures are attacking that player, a planeswalker controlled by that player, or a battle that player protects, then it doesn't block. If there's a cost associated with having the creature block, the player isn't forced to pay that cost, so it doesn't block in that case either."
    );
    check("Predatory Rampage", &[Tapped, CantBlock, NotAttacked, Cost]);
    // Hustle on an opponent's creature during the caster's turn: the block side of the
    // ruling tested with the attack side in r_p059_forced_attacks.rs.
    check("Hustle // Bustle", &[Tapped, CantBlock, Cost]);
    // Provoke untaps the creature as it resolves; tapped again afterwards, it doesn't
    // block.
    check("Provoke", &[Tapped, CantBlock, NotAttacked]);
}

#[test]
fn the_blocking_player_chooses_which_attacker_it_blocks() {
    cr!("509.1a", "509.1c");
    ruling!(
        "Courtly Provocateur",
        "The controller of the creature targeted by the second ability chooses which attacking creature that creature blocks."
    );
    ruling!(
        "Predatory Rampage",
        "Your opponent still chooses which attacking creature each creature they control blocks."
    );
    for name in ["Courtly Provocateur", "Predatory Rampage"] {
        for pick_second in [false, true] {
            let mut t = TestGame::new(2);
            let bears = t.battlefield(P1, "Grizzly Bears");
            let giant = t.battlefield(P0, "Hill Giant");
            let ogre = t.battlefield(P0, "Gray Ogre");
            apply(&mut t, name, bears);
            attack_with(&mut t, &[(giant, pl(P1)), (ogre, pl(P1))]);
            let attacker = if pick_second { ogre } else { giant };
            block(&mut t, P1, &[(bears, attacker)]);
            let c = t.g.combat.as_ref().unwrap();
            assert_eq!(c.blocking(bears), vec![attacker], "{name}");
        }
    }
}

#[test]
fn predatory_rampage_forces_every_opposing_creature_to_block() {
    cr!("509.1c", "611.2c");
    ruling!(
        "Predatory Rampage",
        "Each creature your opponents control blocks if able, even if that creature wasn't on the battlefield or wasn't controlled by an opponent when Predatory Rampage resolved."
    );
    ruling!(
        "Predatory Rampage",
        "Only creatures you control when Predatory Rampage resolves will get +3/+3. Creatures that enter or that you gain control of later in the turn are unaffected."
    );
    ruling!("Predatory Rampage", "Predatory Rampage doesn't force any creature to attack.");
    supported("Predatory Rampage");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let given = t.battlefield(P0, "Gray Ogre");
    let stolen = t.battlefield(P1, "Craw Wurm");
    cast_resolve(&mut t, P0, "Predatory Rampage", &[]);
    assert_eq!(t.pt(giant), (6, 6));
    assert_eq!(t.pt(given), (5, 5));
    // After it resolved: P1 gets a new creature and P0's Gray Ogre; P0 gets P1's Craw Wurm
    // and a new Raging Goblin.
    let late = t.enter(P1, "Llanowar Elves");
    give_control(&mut t, given, P1);
    give_control(&mut t, stolen, P0);
    let goblin = t.enter(P0, "Raging Goblin");
    t.resolve_all();
    assert_eq!(t.pt(stolen), (6, 4), "the Wurm doesn't get +3/+3");
    assert_eq!(t.pt(goblin), (1, 1), "nor does the Goblin");
    // Nothing is forced to attack.
    assert!(declare(&mut t, P0, &[]).is_empty());
    // The two creatures P1 has now must block.
    t.g.combat = None;
    attack_with(&mut t, &[(giant, pl(P1)), (goblin, pl(P1))]);
    block(&mut t, P1, &[]);
    assert!(t.g.is_blocking(late), "{}", t.dump_log());
    assert!(t.g.is_blocking(given));
}

#[test]
fn provoke_can_target_an_untapped_creature() {
    cr!("115.1", "509.1c");
    ruling!("Provoke", "You can target an untapped creature.");
    // "Untap target creature you don't control. That creature blocks this turn if able.
    // Draw a card."
    supported("Provoke");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let candidates = crate::r_s04_common::spell_targets(&mut t, P0, "Provoke");
    assert!(candidates.contains(&o(bears)));
    let library = t.library_size(P0);
    let giant = t.battlefield(P0, "Hill Giant");
    apply(&mut t, "Provoke", bears);
    assert_eq!(t.library_size(P0), library - 1, "drew a card");
    assert!(!t.obj_now(bears).tapped);
    attack_with(&mut t, &[(giant, pl(P1))]);
    block(&mut t, P1, &[]);
    assert!(t.g.is_blocking(bears));
}
