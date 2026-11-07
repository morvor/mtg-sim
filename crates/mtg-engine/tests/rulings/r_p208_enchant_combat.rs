//! Rulings batch P208 — enchant (CR 506–510): Auras about blocking requirements,
//! evasion, first strike, flanking and lifelink.

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::*;
use crate::r_s21_common::legal_blocks;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn setons_desire_forces_only_creatures_able_to_block() {
    cr!("509.1c", "702.5");
    ruling!(
        "Seton's Desire",
        "If a creature the defending player controls can't block the enchanted creature for any reason (such as being tapped), then it doesn't block that creature."
    );
    supported("Seton's Desire");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Seton's Desire", bears);
    for _ in 0..7 {
        t.graveyard(P0, "Grizzly Bears");
    }
    let giant = t.battlefield(P1, "Hill Giant");
    let tapped = t.battlefield(P1, "Grizzly Bears");
    crate::r_p209_common::tap(&mut t, tapped);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    // The untapped Hill Giant must block; the tapped Bears can't.
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(giant, bears)]));
    assert!(!legal_blocks(&mut t, P1, &[(giant, bears), (tapped, bears)]));
    // With only tapped creatures, no block is required.
    crate::r_p209_common::tap(&mut t, giant);
    assert!(legal_blocks(&mut t, P1, &[]));

    // Blocking costs {1} (Archangel of Tithes is attacking): the defending player isn't
    // forced to pay, so the untapped Hill Giant needn't block.
    supported("Archangel of Tithes");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Seton's Desire", bears);
    for _ in 0..7 {
        t.graveyard(P0, "Grizzly Bears");
    }
    let angel = t.battlefield(P0, "Archangel of Tithes");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Wastes", 1);
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (angel, Entity::Player(P1))],
    );
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(giant, bears)]));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 0);
    assert!(t.on_battlefield(giant));
}

#[test]
fn predatory_impetus_creature_is_unblocked_when_nothing_can_block() {
    cr!("509.1c", "701.15b");
    ruling!(
        "Predatory Impetus",
        "If each creature the defending player controls can't block for any reason (such as being tapped), then the enchanted creature isn't blocked."
    );
    supported("Predatory Impetus");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Predatory Impetus", bears);
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    // An untapped creature must block it.
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(giant, bears)]));
    // A tapped one can't: the creature isn't blocked.
    crate::r_p209_common::tap(&mut t, giant);
    assert!(legal_blocks(&mut t, P1, &[]));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 15);

    // Blocking costs {1} (Archangel of Tithes is attacking): the defending player isn't
    // forced to pay, so the creature needn't be blocked.
    supported("Archangel of Tithes");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Predatory Impetus", bears);
    let angel = t.battlefield(P0, "Archangel of Tithes");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Wastes", 1);
    attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (angel, Entity::Player(P1))],
    );
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(giant, bears)]));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 0);
    assert_eq!(t.life(P1), 20 - 5 - 3);
}

#[test]
fn gutter_skulker_stays_blocked_when_the_other_attacker_is_removed() {
    cr!("509.1h", "506.4", "506.4b");
    ruling!(
        "Gutter Skulker",
        "If Gutter Skulker or the creature enchanted by Gutter Shortcut is attacking with another creature and becomes blocked, removing the other creature from combat doesn't cause it to become unblocked."
    );
    let skulker_card = "Gutter Skulker // Gutter Shortcut";
    supported(skulker_card);
    let mut t = TestGame::new(2);
    let skulker = t.battlefield(P0, skulker_card);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    to_blockers(
        &mut t,
        &[(skulker, Entity::Player(P1)), (bears, Entity::Player(P1))],
        &[(giant, skulker)],
    );
    assert_eq!(crate::r_s21_common::blocks_now(&t), vec![(giant, skulker)]);
    mtg_engine::combat::remove_from_combat(&mut t.g, bears);
    t.g.recompute();
    t.advance_to(P0, Step::EndOfCombat);
    // The Skulker stayed blocked: no damage to P1.
    assert_eq!(t.life(P1), 20);
    // The two 3/3s traded damage.
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(!t.on_battlefield(skulker));
}

#[test]
fn felidar_umbra_moved_in_the_first_strike_step_gives_lifelink_to_both() {
    cr!("510.4", "702.15b", "702.7b");
    ruling!(
        "Felidar Umbra",
        "If Felidar Umbra is enchanting a creature with first strike, you can activate its ability and attach it to a creature without first strike during the first combat damage step. Each of those creatures will have lifelink when dealing combat damage and you'll gain life accordingly."
    );
    supported("Felidar Umbra");
    supported("White Knight");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "White Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let umbra = attach_new(&mut t, P0, "Felidar Umbra", knight);
    t.lands(P0, "Plains", 2);
    attack_with(
        &mut t,
        &[(knight, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::FirstStrikeDamage);
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 18);
    t.activate(P0, umbra, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(attached_to(&t, umbra), Some(Entity::Object(bears)));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn mardu_runemark_losing_first_strike_doesnt_deal_damage_twice() {
    cr!("510.4", "702.7c");
    ruling!(
        "Mardu Runemark",
        "If the enchanted creature assigns combat damage during the first combat damage step, and then it loses first strike before the second combat damage step, it won’t assign combat damage again in the second combat damage step."
    );
    supported("Mardu Runemark");
    supported("Savannah Lions");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Savannah Lions");
    attach_new(&mut t, P0, "Mardu Runemark", bears);
    assert!(has_kw(&t, bears, mtg_engine::keywords::KeywordKind::FirstStrike));
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 16);
    destroy(&mut t, lions);
    assert!(!has_kw(&t, bears, mtg_engine::keywords::KeywordKind::FirstStrike));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn agility_flanking_is_cumulative_with_existing_flanking() {
    cr!("702.25b", "702.25a");
    ruling!(
        "Agility",
        "If the enchanted creature already has Flanking, its effect is cumulative."
    );
    supported("Agility");
    supported("Suq'Ata Lancer");
    let mut t = TestGame::new(2);
    let lancer = t.battlefield(P0, "Suq'Ata Lancer");
    attach_new(&mut t, P0, "Agility", lancer);
    let giant = t.battlefield(P1, "Hill Giant");
    to_blockers(&mut t, &[(lancer, Entity::Player(P1))], &[(giant, lancer)]);
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
}
