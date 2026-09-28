//! Rulings batch S20 — attacking: phased-out permanents are treated as though they don't
//! exist (CR 702.26b): they can't be targeted, their static abilities don't apply, their
//! triggered abilities don't trigger, and they can't attack or block.

use crate::r_s01_common::*;
use crate::r_s04_common::{add_mana, spell_targets};
use crate::r_s09_common::legal_attack;
use crate::r_s20_common::*;
use mtg_engine::combat::block_options;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn phased_out(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).phased_out
}

/// Whether P1 could choose `id` as the target of a Lightning Bolt now.
fn bolt_can_target(t: &mut TestGame, id: ObjectId) -> bool {
    spell_targets(t, P1, "Lightning Bolt").contains(&Entity::Object(id))
}

/// P1 attacks P0 with `attacker` in P1's turn: whether `blocker` could block it.
fn could_block(t: &mut TestGame, attacker: ObjectId, blocker: ObjectId) -> bool {
    t.g.combat = None;
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(t, &[(attacker, Entity::Player(P0))]);
    block_options(&t.g, &[P0])
        .iter()
        .any(|(b, attackers)| *b == blocker && attackers.contains(&attacker))
}

#[test]
fn a_creature_phased_out_by_slip_out_the_back_is_treated_as_though_it_doesnt_exist() {
    cr!("702.26b");
    ruling!(
        "Slip Out the Back",
        "Phased-out permanents are treated as though they don't exist. They can't be the target of spells or abilities, their static abilities have no effect on the game, their triggered abilities can't trigger, they can't attack or block, and so on."
    );
    supported("Slip Out the Back");
    let mut t = TestGame::new(2);
    // Benalish Marshal: "Other creatures you control get +1/+1."
    let marshal = t.battlefield(P0, "Benalish Marshal");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (3, 3));
    assert!(bolt_can_target(&mut t, marshal));
    // "Put a +1/+1 counter on target creature. It phases out."
    add_mana(&mut t, P0, ManaType::U, 1);
    let slip = t.hand(P0, "Slip Out the Back");
    t.cast(P0, slip).target(marshal).go();
    t.resolve_all();
    assert!(phased_out(&t, marshal));
    // Its static ability has no effect.
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
    // It can't attack.
    to_beginning_of_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[(marshal, Entity::Player(P1))]));
    // It can't be the target of spells.
    assert!(!bolt_can_target(&mut t, marshal));
    // It can't block in P1's turn.
    let giant = t.battlefield(P1, "Hill Giant");
    assert!(could_block(&mut t, giant, bears));
    assert!(!could_block(&mut t, giant, marshal));
}

#[test]
fn king_of_the_oathbreakers_phased_out_cant_be_targeted_or_block() {
    cr!("702.26b");
    ruling!(
        "King of the Oathbreakers",
        "Phased-out permanents are treated as though they don't exist. They can't be the target of spells or abilities"
    );
    supported("King of the Oathbreakers");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let king = t.battlefield(P0, "King of the Oathbreakers");
    // "Whenever King of the Oathbreakers or another Spirit you control becomes the target
    // of a spell, it phases out." P1's Lightning Bolt is countered on resolution: its
    // target doesn't exist.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(king).go();
    t.resolve_all();
    assert!(phased_out(&t, king));
    assert_eq!(t.obj_now(king).damage, 0);
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
    // It can't be targeted again, and it can't block P1's attacker.
    assert!(!bolt_can_target(&mut t, king));
    let giant = t.battlefield(P1, "Hill Giant");
    assert!(!could_block(&mut t, giant, king));
}

#[test]
fn renegade_silent_phased_out_cant_be_targeted_or_block() {
    cr!("702.26b");
    ruling!(
        "Renegade Silent",
        "Phased-out permanents are treated as though they don't exist. They can't be the targets of spells or abilities, their static abilities have no effect on the game, their triggered abilities can't trigger, they can't attack or block, and so on."
    );
    supported("Renegade Silent");
    let mut t = TestGame::new(2);
    // "At the beginning of your end step, goad up to one target creature you don't
    // control and put a +1/+1 counter on this creature. This creature phases out."
    let silent = t.battlefield(P0, "Renegade Silent");
    t.answer_targets(P0, &[]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(phased_out(&t, silent));
    // In P1's turn it can't be targeted and can't block.
    t.advance_to(P1, Step::PrecombatMain);
    assert!(phased_out(&t, silent));
    assert!(!bolt_can_target(&mut t, silent));
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(could_block(&mut t, giant, bears));
    assert!(!could_block(&mut t, giant, silent));
}

#[test]
fn permanents_phased_out_by_unite_the_coalition_dont_exist() {
    cr!("702.26b");
    ruling!(
        "Unite the Coalition",
        "Phased-out permanents are treated as though they don’t exist. They can’t be the targets of spells or abilities, their static abilities have no effect on the game, their triggered abilities can’t trigger, they can’t attack or block, and so on."
    );
    supported("Unite the Coalition");
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let warden = t.battlefield(P0, "Soul Warden");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert_eq!(t.pt(bears), (3, 3));
    // Three times "Target permanent phases out", and "Target player draws a card" twice.
    for ty in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
    ] {
        add_mana(&mut t, P0, ty, 1);
    }
    add_mana(&mut t, P0, ManaType::C, 2);
    let unite = t.hand(P0, "Unite the Coalition");
    t.cast(P0, unite)
        .modes(&[0, 0, 0, 1, 1])
        .targets(&[Entity::Object(anthem)])
        .targets(&[Entity::Object(warden)])
        .targets(&[Entity::Object(bears)])
        .targets(&[Entity::Player(P0)])
        .targets(&[Entity::Player(P0)])
        .go();
    t.resolve_all();
    for p in [anthem, warden, bears] {
        assert!(phased_out(&t, p));
    }
    // The Anthem's static ability doesn't apply: a new creature is 2/2.
    let life = t.life(P0);
    let ogre = t.enter(P0, "Gray Ogre");
    t.settle();
    t.resolve_all();
    assert_eq!(t.pt(ogre), (2, 2));
    // Soul Warden's ability ("Whenever another creature enters, you gain 1 life.")
    // didn't trigger.
    assert_eq!(t.life(P0), life);
    // The Bears can't attack.
    to_beginning_of_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[(bears, Entity::Player(P1))]));
}
