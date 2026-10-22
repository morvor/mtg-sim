//! Rulings batch S11 — melee (CR 702.121): "Whenever this creature attacks, it gets +1/+1
//! until end of turn for each opponent you attacked with a creature this combat."
//! Adriana, Captain of the Guard: "Melee. Other creatures you control have melee."

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s11_common::*;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn the_bonus_counts_opponents_attacked_as_melee_resolves() {
    cr!("702.121a", "800.4a");
    ruling!(
        "Adriana, Captain of the Guard",
        "You determine the size of the bonus as the melee ability resolves. Count each opponent that you attacked with one or more creatures. It doesn't matter if the attacking creatures are still attacking or even if they are still on the battlefield. It also doesn't matter if the opponent you attacked is still in the game."
    );
    supported("Adriana, Captain of the Guard");
    let mut t = TestGame::new(4);
    let adriana = t.battlefield(P0, "Adriana, Captain of the Guard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    attack_with(
        &mut t,
        &[
            (adriana, Entity::Player(P1)),
            (bears, Entity::Player(P2)),
            (elves, Entity::Player(P3)),
        ],
    );
    // Each attacker has melee: three triggers.
    assert_eq!(triggered_from(&t, adriana), 1);
    assert_eq!(triggered_from(&t, bears), 1);
    // Before they resolve, the Bears leave the battlefield, the Elves are removed from
    // combat, and P3 leaves the game.
    destroy(&mut t, bears);
    mtg_engine::combat::remove_from_combat(&mut t.g, elves);
    t.g.player_loses(P3);
    t.settle();
    t.resolve_all();
    // Three opponents were attacked.
    assert_eq!(t.pt(adriana), (7, 7));
}

#[test]
fn creatures_entering_attacking_dont_count_or_trigger_melee() {
    cr!("702.121a", "508.4", "702.116a");
    ruling!(
        "Adriana, Captain of the Guard",
        "Creatures that enter the battlefield attacking were never declared as attackers, so they won't count toward melee's effect. Similarly, if a creature with melee enters the battlefield attacking, melee won't trigger."
    );
    ruling!(
        "Skyhunter Strike Force",
        "Creatures that enter the battlefield attacking were never declared as attackers, so they won't count toward melee's effect."
    );
    supported("Warchief Giant");
    // Warchief Giant's myriad creates token copies attacking P2 and P3; with Adriana they
    // have melee too.
    let mut t = TestGame::new(4);
    let adriana = t.battlefield(P0, "Adriana, Captain of the Guard");
    let giant = t.battlefield(P0, "Warchief Giant");
    attack_with(
        &mut t,
        &[(adriana, Entity::Player(P1)), (giant, Entity::Player(P1))],
    );
    t.resolve_all();
    let copies = tokens(&t, P0);
    assert_eq!(copies.len(), 2);
    for c in &copies {
        assert!(t.obj(*c).chars.has_keyword(mtg_engine::keywords::KeywordKind::Melee));
        assert_eq!(triggered_from(&t, *c), 0);
        assert_eq!(t.pt(*c), (5, 3));
    }
    // Only P1 was attacked by a declared attacker.
    assert_eq!(t.pt(adriana), (5, 5));
    assert_eq!(t.pt(giant), (6, 4));
}
