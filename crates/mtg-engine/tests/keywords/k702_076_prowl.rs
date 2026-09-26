//! CR 702.76 Prowl.

use crate::common_k702_011_017::{assert_supported, attack_with};
use crate::common_k702_018_026::declare_blocks;
use crate::common_k702_027_037::can_cast;
use crate::common_k702_052_066::{destroy, run_effect};
use mtg_engine::ability::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const PROWL: CastMethod = CastMethod::Keyword(KeywordKind::Prowl);

/// `attacker` (controlled by the active player) attacks P1 unblocked; stops in the
/// postcombat main phase.
fn hit(t: &mut TestGame, attacker: ObjectId) {
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    declare_blocks(t, P1, &[]);
    let ap = t.g.turn.active;
    t.advance_to(ap, Step::PostcombatMain);
}

#[test]
fn prowl_is_available_after_a_rogue_you_control_dealt_combat_damage_to_a_player() {
    cr!("702.76", "702.76a");
    assert_supported("Morsel Theft");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let theft = t.hand(P0, "Morsel Theft");
    assert!(!can_cast(&mut t, P0, theft, PROWL));
    // Bane Alley Blackguard: a Human Rogue.
    let rogue = t.battlefield(P0, "Bane Alley Blackguard");
    hit(&mut t, rogue);
    assert_eq!(t.life(P1), 19);
    assert!(can_cast(&mut t, P0, theft, PROWL));
    let hand = t.hand_size(P0);
    t.cast(P0, theft).method(PROWL).target(P1).go();
    // Paid {1}{B} rather than {2}{B}{B}.
    assert_eq!(t.g.permanents().filter(|o| o.tapped && o.chars.is_land()).count(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 23);
    // "If ~'s prowl cost was paid, draw a card."
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn a_spell_cast_for_its_mana_cost_doesnt_count_as_prowled() {
    cr!("702.76a");
    let mut t = TestGame::new(2);
    let rogue = t.battlefield(P0, "Bane Alley Blackguard");
    hit(&mut t, rogue);
    t.lands(P0, "Swamp", 4);
    let theft = t.hand(P0, "Morsel Theft");
    let hand = t.hand_size(P0);
    t.cast(P0, theft).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.hand_size(P0), hand - 1);
}

#[test]
fn the_source_must_have_one_of_the_spells_creature_types() {
    cr!("702.76a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let theft = t.hand(P0, "Morsel Theft");
    let bears = t.battlefield(P0, "Grizzly Bears");
    hit(&mut t, bears);
    assert_eq!(t.life(P1), 18);
    assert!(!can_cast(&mut t, P0, theft, PROWL));
}

#[test]
fn a_changeling_has_every_creature_type_for_prowl() {
    cr!("702.76a", "702.73a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let theft = t.hand(P0, "Morsel Theft");
    let changeling = t.battlefield(P0, "Woodland Changeling");
    hit(&mut t, changeling);
    assert!(can_cast(&mut t, P0, theft, PROWL));
}

#[test]
fn the_source_must_have_been_under_your_control() {
    cr!("702.76a");
    let mut t = TestGame::new(2);
    let rogue = t.battlefield(P1, "Bane Alley Blackguard");
    t.set_step(P1, Step::PrecombatMain);
    attack_with(&mut t, &[(rogue, Entity::Player(P0))]);
    declare_blocks(&mut t, P0, &[]);
    t.advance_to(P1, Step::PostcombatMain);
    assert_eq!(t.life(P0), 19);
    // Thieves' Fortune: a Rogue instant with prowl. P0 can't prowl it; P1 can.
    assert_supported("Thieves' Fortune");
    let fortune = t.hand(P0, "Thieves' Fortune");
    t.lands(P0, "Island", 1);
    assert!(!can_cast(&mut t, P0, fortune, PROWL));
    t.lands(P1, "Island", 1);
    let theirs = t.hand(P1, "Thieves' Fortune");
    assert!(can_cast(&mut t, P1, theirs, PROWL));
}

#[test]
fn noncombat_damage_and_damage_to_creatures_dont_enable_prowl() {
    cr!("702.76a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let theft = t.hand(P0, "Morsel Theft");
    let rogue = t.battlefield(P0, "Krovikan Scoundrel");
    run_effect(
        &mut t,
        None,
        P0,
        Effect::DealDamage {
            source: Sel::Target(0),
            amount: Value::c(2),
            to: Sel::Players(PlayerRef::Player(P1)),
        },
        &[Entity::Object(rogue)],
    );
    assert_eq!(t.life(P1), 18);
    assert!(!can_cast(&mut t, P0, theft, PROWL));
    // Blocked: combat damage to a creature only.
    let wall = t.battlefield(P1, "Wall of Wood");
    attack_with(&mut t, &[(rogue, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(wall, rogue)]);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(!can_cast(&mut t, P0, theft, PROWL));
}

#[test]
fn what_counts_is_the_source_as_it_dealt_the_damage() {
    cr!("702.76a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let theft = t.hand(P0, "Morsel Theft");
    let rogue = t.battlefield(P0, "Krovikan Scoundrel");
    hit(&mut t, rogue);
    // It stops being a Rogue, then leaves the battlefield: it was a Rogue then.
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveAllCreatureTypes],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(rogue)],
    );
    assert!(!t.obj_now(rogue).chars.has_subtype("Rogue"));
    assert!(can_cast(&mut t, P0, theft, PROWL));
    destroy(&mut t, rogue);
    t.resolve_all();
    assert!(can_cast(&mut t, P0, theft, PROWL));
    // Not next turn.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!can_cast(&mut t, P0, theft, PROWL));
}

#[test]
fn a_prowled_creatures_enters_ability_sees_its_prowl_cost_paid() {
    cr!("702.76a");
    assert_supported("Latchkey Faerie");
    let mut t = TestGame::new(2);
    let rogue = t.battlefield(P0, "Krovikan Scoundrel");
    hit(&mut t, rogue);
    t.lands(P0, "Island", 3);
    let faerie = t.hand(P0, "Latchkey Faerie");
    let hand = t.hand_size(P0);
    t.cast(P0, faerie).method(PROWL).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Latchkey Faerie").len(), 1);
    // "When this creature enters, if its prowl cost was paid, draw a card."
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn prowl_works_only_from_a_zone_the_card_could_be_cast_from() {
    cr!("702.76a");
    let mut t = TestGame::new(2);
    let rogue = t.battlefield(P0, "Krovikan Scoundrel");
    hit(&mut t, rogue);
    t.lands(P0, "Swamp", 2);
    let in_gy = t.graveyard(P0, "Morsel Theft");
    assert!(!can_cast(&mut t, P0, in_gy, PROWL));
}
