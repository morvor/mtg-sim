//! Rulings batch P048 — "attacks (or blocks) each combat if able", given to the opponent
//! by a token (Pursued Whale, Goblin Spymaster) or on the creature itself (Iron Golem):
//! a creature that can't attack (tapped, summoning sick, "can't attack") doesn't, and a
//! cost to attack is never forced (CR 508.1d, 509.1c); cost increases add up before
//! reductions (CR 601.2f).

use crate::r_p048_common::*;
use crate::r_s06_common::give_control;
use crate::r_s09_common::legal_attack;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// What keeps P1's Hill Giant (under a "must attack" requirement) from attacking.
#[derive(Clone, Copy, Debug)]
enum Obstacle {
    None,
    Tapped,
    /// It came under P1's control this turn.
    NewlyControlled,
    /// P0 controls Ghostly Prison ("Creatures can't attack you unless their controller
    /// pays {2} for each creature they control that's attacking you.").
    AttackCost,
    /// Pacifism ("Enchanted creature can't attack or block.").
    CantAttack,
}

const OBSTACLES: [Obstacle; 5] = [
    Obstacle::None,
    Obstacle::Tapped,
    Obstacle::NewlyControlled,
    Obstacle::AttackCost,
    Obstacle::CantAttack,
];

#[test]
fn creatures_that_must_attack_dont_when_they_cant_or_it_costs() {
    cr!("508.1d", "302.6");
    ruling!(
        "Pursued Whale",
        "If a creature affected by the Pirate token can't attack for any reason (such as being tapped or having come under that player's control that turn), then it doesn't attack."
    );
    ruling!(
        "Pursued Whale",
        "If there's a cost associated with having it attack, the player isn't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    ruling!(
        "Goblin Spymaster",
        "If, during your declare attackers step, a creature you control is tapped or is affected by a spell or ability that says it can't attack, then it doesn't attack, even if you control one of Goblin Spymaster's tokens."
    );
    ruling!(
        "Goblin Spymaster",
        "If there's a cost associated with having a creature attack, you aren't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    ruling!(
        "Iron Golem",
        "If the Iron Golem can't attack or block for any reason (such as being tapped), then it doesn't attack or block."
    );
    ruling!(
        "Iron Golem",
        "If there's a cost associated with having it attack or block, its controller isn't forced to pay that cost, so it doesn't have to attack or block in that case either."
    );
    supported("Pursued Whale");
    supported("Goblin Spymaster");
    supported("Iron Golem");
    type Setup = fn(&mut TestGame);
    let cases: [(&str, &str, Setup); 3] = [
        // P0's Pursued Whale enters: P1 gets the Pirate token.
        ("Pursued Whale", "Hill Giant", |t| {
            t.enter(P0, "Pursued Whale");
            t.resolve_all();
            assert_eq!(tokens_of(t, P1, "Pirate"), 1);
        }),
        // P0's Goblin Spymaster: P1 gets the Goblin token in P1's end step (and then
        // P1's next turn begins).
        ("Goblin Spymaster", "Hill Giant", |t| {
            t.battlefield(P0, "Goblin Spymaster");
            t.advance_to(P1, Step::End);
            t.resolve_all();
            assert_eq!(tokens_of(t, P1, "Goblin"), 1);
            t.advance_to(P0, Step::PrecombatMain);
        }),
        // Iron Golem itself, P1's.
        ("Iron Golem", "Iron Golem", |_| {}),
    ];
    for (card, creature, setup) in cases {
        for obstacle in OBSTACLES {
            let skip = may_skip_attack(creature, obstacle, setup);
            let expect = !matches!(obstacle, Obstacle::None);
            assert_eq!(skip, expect, "{card}: {obstacle:?}");
        }
    }
}

/// In P1's turn, P1's `creature` (made by `setup`, which gives P1 the requirement) faces
/// `obstacle`; returns whether P1 may declare no attackers. Any token P1 got (which must
/// attack too) is tapped first.
fn may_skip_attack(creature: &str, obstacle: Obstacle, setup: fn(&mut TestGame)) -> bool {
    let mut t = TestGame::new(2);
    setup(&mut t);
    t.advance_to(P1, Step::Upkeep);
    let tokens: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.controller == P1 && o.is_token())
            .map(|o| o.id)
            .collect();
    for tok in tokens {
        tap(&mut t, tok);
    }
    let c = match obstacle {
        Obstacle::NewlyControlled => {
            let c = t.battlefield(P0, creature);
            give_control(&mut t, c, P1);
            c
        }
        _ => t.battlefield(P1, creature),
    };
    match obstacle {
        Obstacle::Tapped => tap(&mut t, c),
        Obstacle::AttackCost => {
            t.battlefield(P0, "Ghostly Prison");
        }
        Obstacle::CantAttack => {
            attach_new(&mut t, P0, "Pacifism", obj(c));
        }
        _ => {}
    }
    // Lands to pay the attack cost with: P1 still isn't forced to pay it.
    t.lands(P1, "Wastes", 2);
    to_combat(&mut t, P1);
    legal_attack(&mut t, &[])
}

#[test]
fn iron_golem_blocks_if_able_but_not_while_tapped() {
    cr!("509.1c");
    ruling!(
        "Iron Golem",
        "If the Iron Golem can't attack or block for any reason (such as being tapped), then it doesn't attack or block."
    );
    for tapped in [false, true] {
        let mut t = TestGame::new(2);
        let golem = t.battlefield(P1, "Iron Golem");
        let bears = t.battlefield(P0, "Grizzly Bears");
        if tapped {
            tap(&mut t, golem);
        }
        to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
        assert_eq!(blocks_now(&t).is_empty(), tapped, "tapped: {tapped}");
    }
}

#[test]
fn pursued_whales_tax_is_added_before_cost_reductions() {
    cr!("601.2f", "202.3");
    ruling!(
        "Pursued Whale",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases (such as that of Pursued Whale), then apply any cost reductions."
    );
    ruling!(
        "Pursued Whale",
        "The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Baral, Chief of Compliance");
    // P1's Lightning Bolt targeting P0's Whale costs {R} + {3} - {1} (Baral, Chief of
    // Compliance: "Instant and sorcery spells you cast cost {1} less to cast.") = {2}{R};
    // the reduction applied first would find no generic mana to reduce ({3}{R}). Either
    // permanent may have arrived first.
    for baral_first in [false, true] {
        let mut t = TestGame::new(2);
        if baral_first {
            t.battlefield(P1, "Baral, Chief of Compliance");
        }
        let whale = t.battlefield(P0, "Pursued Whale");
        if !baral_first {
            t.battlefield(P1, "Baral, Chief of Compliance");
        }
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Mountain", 1);
        t.lands(P1, "Wastes", 3);
        let bolt = t.hand(P1, "Lightning Bolt");
        let spell = t.cast_with(P1, bolt, &[obj(whale)]).expect("cast");
        let tapped =
            t.g.permanents()
                .filter(|o| o.controller == P1 && o.tapped)
                .count();
        assert_eq!(tapped, 3, "Baral first: {baral_first}");
        assert_eq!(t.g.mana_value_of(spell), 1);
    }
}
