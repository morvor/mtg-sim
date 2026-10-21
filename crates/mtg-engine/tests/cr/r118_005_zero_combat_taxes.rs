//! CR 118.5 with 508.1d, 508.1h, 509.1c and 509.1d: a tax to attack or block that comes to
//! {0} is still a cost. Requirements don't force a creature under it to attack or block,
//! and a player who attacks or blocks with it acknowledges paying {0}. The {0} can be a
//! scaled amount whose multiplier is 0 (Collective Restraint without basic land types,
//! Myr Prototype without counters, Cowed by Wisdom with an empty hand), a written {0}, or
//! a chosen X = 0 (War Tax, War Cadence).

use crate::r506_common::*;
use mtg_engine::ability::*;
use mtg_engine::combat::{
    attack_declaration_legal, attack_options, block_costs_by_player, block_declaration_legal,
    block_options, required_attack_cost, required_block_cost,
};
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaCost;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn legal_attack(t: &mut TestGame, decl: &[(ObjectId, Entity)]) -> bool {
    t.g.recompute();
    let options = attack_options(&t.g);
    attack_declaration_legal(&t.g, &options, decl)
}

fn legal_blocks(t: &mut TestGame, decl: &[(ObjectId, ObjectId)]) -> bool {
    t.g.recompute();
    let defenders = t.g.combat.as_ref().unwrap().defending_players.clone();
    let options = block_options(&t.g, &defenders);
    block_declaration_legal(&t.g, &options, decl)
}

/// The prompts of the yes/no questions `p` was asked.
fn yes_no_prompts(t: &TestGame, p: PlayerId) -> Vec<String> {
    t.asked()
        .into_iter()
        .filter_map(|(q, d)| match d {
            Decision::YesNo { prompt, .. } if q == p => Some(prompt),
            _ => None,
        })
        .collect()
}

/// "Creatures can't attack you unless their controller pays {0} for each creature they
/// control that's attacking you", for `p` until end of turn: a written {0}.
fn written_zero_tax(t: &mut TestGame, p: PlayerId) {
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    let cost = Cost {
        mana: ManaCost::parse("{0}"),
        ..Cost::default()
    };
    assert!(cost.mana.is_some(), "{{0}} parses");
    t.g.exec(
        &Effect::AddRestriction {
            restriction: Restriction::AttackCost {
                attackers: Filter::Any,
                defender: PlayerFilter::You,
                planeswalkers: false,
                cost,
            },
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
}

#[test]
fn an_attack_tax_of_zero_is_a_cost_however_it_comes_to_zero() {
    cr!("118.5", "508.1d");
    interp!("combat-tax-of-zero-is-a-cost");
    // Juggernaut "attacks each combat if able". Each tax comes to {0}: it's still a cost,
    // so not attacking is legal (and so is attacking).
    for case in ["scaled", "written", "none"] {
        let mut t = TestGame::new(2);
        let jugg = t.battlefield(P0, "Juggernaut");
        match case {
            // Domain with no basic land types among P1's lands: X = 0.
            "scaled" => {
                t.battlefield(P1, "Collective Restraint");
            }
            "written" => written_zero_tax(&mut t, P1),
            _ => {}
        }
        t.set_step(P0, Step::BeginningOfCombat);
        let p1 = Entity::Player(P1);
        let cost = required_attack_cost(&t.g, jugg, p1);
        assert_eq!(
            cost.as_ref().map(|c| c.is_free()),
            (case != "none").then_some(true),
            "{case}"
        );
        assert_eq!(legal_attack(&mut t, &[]), case != "none", "{case}");
        assert!(legal_attack(&mut t, &[(jugg, p1)]), "{case}");
    }
}

#[test]
fn a_creature_with_a_counter_scaled_tax_and_no_counters_has_a_zero_cost() {
    cr!("118.5", "508.1d", "509.1c");
    interp!("combat-tax-of-zero-is-a-cost");
    // Myr Prototype: "This creature can't attack or block unless you pay {1} for each
    // +1/+1 counter on it." With no counters: {0} to attack and {0} to block.
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P1, "Myr Prototype");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lure = t.battlefield(P0, "Lure");
    assert!(t.g.attach(lure, Entity::Object(bears)));
    assert!(required_attack_cost(&t.g, myr, Entity::Player(P0)).is_some_and(|c| c.is_free()));
    assert!(required_block_cost(&t.g, myr).is_some_and(|c| c.is_free()));
    // Lure: "All creatures able to block enchanted creature do so." The Myr can't block
    // unless its controller pays {0}, so it doesn't have to (CR 509.1c).
    to_combat(&mut t, P0);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(legal_blocks(&mut t, &[]));
    assert!(legal_blocks(&mut t, &[(myr, bears)]));
    // A Grizzly Bears under Lure's requirement is still forced, for comparison.
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P1, "Wall of Wood");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lure = t.battlefield(P0, "Lure");
    assert!(t.g.attach(lure, Entity::Object(bears)));
    to_combat(&mut t, P0);
    declare(&mut t, &[(bears, Entity::Player(P1))]);
    go_to(&mut t, Step::DeclareAttackers);
    assert!(!legal_blocks(&mut t, &[]));
    assert!(legal_blocks(&mut t, &[(wall, bears)]));
}

#[test]
fn cowed_by_wisdom_with_an_empty_hand_still_taxes_attacks() {
    cr!("118.5", "508.1d");
    interp!("combat-tax-of-zero-is-a-cost");
    // Cowed by Wisdom: "Enchanted creature can't attack or block unless its controller pays
    // {1} for each card in your hand." P1 (its controller) has no cards in hand.
    let mut t = TestGame::new(2);
    let cyclops = t.battlefield(P0, "Bloodrock Cyclops");
    let aura = t.battlefield(P1, "Cowed by Wisdom");
    assert!(t.g.attach(aura, Entity::Object(cyclops)));
    assert_eq!(t.hand_size(P1), 0);
    t.set_step(P0, Step::BeginningOfCombat);
    let p1 = Entity::Player(P1);
    assert!(required_attack_cost(&t.g, cyclops, p1).is_some_and(|c| c.is_free()));
    assert!(legal_attack(&mut t, &[]));
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(!t.g.is_attacking(cyclops));
}

#[test]
fn attacking_with_a_creature_taxed_zero_means_acknowledging_paying_zero() {
    cr!("118.5", "508.1h");
    interp!("combat-tax-of-zero-is-a-cost");
    // Master Warcraft: P1 chooses which creatures attack. P1 chooses P0's Myr Prototype
    // (no counters: a cost of {0} to attack). P0 is asked whether to pay the {0}: paying
    // isn't automatic. Declining means the Myr doesn't attack; accepting, it does.
    for pay in [false, true] {
        let mut t = TestGame::new(2);
        let myr = t.battlefield(P0, "Myr Prototype");
        to_combat(&mut t, P0);
        t.lands(P1, "Mountain", 4);
        let mw = t.hand(P1, "Master Warcraft");
        t.cast(P1, mw).go();
        t.resolve_all();
        t.answer(
            P1,
            DecisionKind::Attackers,
            Answer::Attackers(vec![(myr, Entity::Player(P1))]),
        );
        t.answer_yes(P0, pay);
        go_to(&mut t, Step::DeclareAttackers);
        let prompts = yes_no_prompts(&t, P0);
        assert_eq!(prompts.len(), 1, "{prompts:?}");
        assert!(prompts[0].contains("attacks"), "{prompts:?}");
        assert_eq!(t.g.is_attacking(myr), pay, "pay: {pay}");
    }
}

#[test]
fn blocking_with_a_creature_taxed_zero_means_acknowledging_paying_zero() {
    cr!("118.5", "509.1d");
    interp!("combat-tax-of-zero-is-a-cost");
    // Master Warcraft: P0 (attacking) chooses how P1's creatures block, and has P1's Myr
    // Prototype (no counters: a cost of {0} to block) block. P1 is asked whether to pay
    // the {0}; declining means the Myr doesn't block.
    for pay in [false, true] {
        let mut t = TestGame::new(2);
        let myr = t.battlefield(P1, "Myr Prototype");
        let bears = t.battlefield(P0, "Grizzly Bears");
        to_combat(&mut t, P0);
        t.lands(P0, "Mountain", 4);
        let mw = t.hand(P0, "Master Warcraft");
        t.cast(P0, mw).go();
        t.resolve_all();
        declare(&mut t, &[(bears, Entity::Player(P1))]);
        go_to(&mut t, Step::DeclareAttackers);
        assert_eq!(
            block_costs_by_player(&t.g, &[(myr, bears)])
                .iter()
                .map(|(p, c)| (*p, c.is_free()))
                .collect::<Vec<_>>(),
            vec![(P1, true)]
        );
        block(&mut t, P0, &[(myr, bears)]);
        t.answer_yes(P1, pay);
        go_to(&mut t, Step::DeclareBlockers);
        let prompts = yes_no_prompts(&t, P1);
        assert!(
            prompts.iter().any(|p| p.contains("blocks")),
            "{prompts:?}"
        );
        assert_eq!(t.g.is_blocking(myr), pay, "pay: {pay}");
    }
}
