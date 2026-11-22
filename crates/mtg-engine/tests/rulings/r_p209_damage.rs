//! Rulings batch P209 — enchant (CR 303.4, 702.5): Auras about damage — fights and
//! damage-dealing abilities whose enchanted creature or target leaves, redirection
//! (Pariah, Saving Grace, Ward of Piety), damage triggers, regeneration and lifelink.

use crate::r_p209_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s03_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::*;
use crate::r_s06_common::*;
use crate::r_s07_common::damage_on;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Fights and "deals damage equal to its power" with something gone
// ---------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum Gone {
    Nothing,
    Target,
    Enchanted,
}

/// P0 casts `aura` on its Hill Giant; its enters trigger targets P1's Grizzly Bears. Before
/// the trigger resolves, `gone` leaves the battlefield. Returns (Giant damage, Bears
/// damage or None if the Bears died / left).
fn fight_aura(aura: &str, gone: Gone) -> (TestGame, ObjectId, ObjectId) {
    supported(aura);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let a = in_hand_with_mana(&mut t, P0, aura);
    t.cast(P0, a).target(giant).go();
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.answer_yes(P0, true);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1, "{aura}'s trigger");
    match gone {
        Gone::Nothing => {}
        Gone::Target => {
            move_to(&mut t, bears, Zone::Hand(P1));
        }
        Gone::Enchanted => {
            move_to(&mut t, giant, Zone::Hand(P0));
        }
    }
    t.resolve_all();
    (t, giant, bears)
}

fn fight_aura_cases(aura: &str) {
    let (t, giant, _) = fight_aura(aura, Gone::Nothing);
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{aura}");
    assert_eq!(damage_on(&t, giant), 2, "{aura}");
    // The target left: the ability doesn't resolve, the Giant isn't dealt damage.
    let (t, giant, _) = fight_aura(aura, Gone::Target);
    assert_eq!(damage_on(&t, giant), 0, "{aura}");
    // The enchanted creature left: no fight, the Bears aren't dealt damage.
    let (t, _, bears) = fight_aura(aura, Gone::Enchanted);
    assert!(t.on_battlefield(bears), "{aura}");
    assert_eq!(damage_on(&t, bears), 0, "{aura}");
}

#[test]
fn warbriar_blessing_fight_with_a_creature_gone() {
    cr!("701.14b", "608.2b", "701.14a");
    ruling!(
        "Warbriar Blessing",
        "If the target creature is an illegal target when Warbriar Blessing's triggered ability tries to resolve, the ability doesn't resolve. If the enchanted creature is no longer on the battlefield, the target creature won't deal or be dealt damage."
    );
    fight_aura_cases("Warbriar Blessing");
}

#[test]
fn cartouche_of_strength_fight_with_a_creature_gone() {
    cr!("701.14b", "608.2b", "701.14a");
    ruling!(
        "Cartouche of Strength",
        "If the triggered ability's target is illegal when it tries to resolve, or if the enchanted creature has left the battlefield, no creature will deal or be dealt damage."
    );
    fight_aura_cases("Cartouche of Strength");
}

#[test]
fn pain_for_all_uses_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Pain for All",
        "If the enchanted creature leaves the battlefield before Pain for All’s second ability resolves, use its power as it last existed on the battlefield to determine how much damage is dealt."
    );
    supported("Pain for All");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    let a = in_hand_with_mana(&mut t, P0, "Pain for All");
    t.cast(P0, a).target(giant).go();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let from = t.asked().len();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // "Any other target": not the enchanted creature.
    let cands = target_candidates(&t, P0, from);
    assert!(cands.iter().all(|c| !c.contains(&Entity::Object(giant))), "{cands:?}");
    move_to(&mut t, giant, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

/// P0's Hill Giant enchanted by Predatory Urge activates the ability targeting P1's
/// Grizzly Bears; `respond` runs while it's on the stack.
fn predatory_urge(respond: impl FnOnce(&mut TestGame, ObjectId, ObjectId, ObjectId)) -> (TestGame, ObjectId, ObjectId) {
    supported("Predatory Urge");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let urge = attach_new(&mut t, P0, "Predatory Urge", giant);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, giant, "deals damage equal to its power").unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    respond(&mut t, giant, bears, urge);
    t.resolve_all();
    (t, giant, bears)
}

#[test]
fn predatory_urge_ability_doesnt_care_about_the_aura_leaving() {
    cr!("113.7a", "602.2");
    ruling!(
        "Predatory Urge",
        "If the enchanted creature’s ability is activated, that creature is the one that will deal and be dealt damage when the ability resolves. It doesn’t matter if Predatory Urge leaves the battlefield"
    );
    let (t, giant, _) = predatory_urge(|t, _, _, urge| destroy(t, urge));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(damage_on(&t, giant), 2);
}

#[test]
fn predatory_urge_ability_with_an_illegal_target_doesnt_resolve() {
    cr!("608.2b");
    ruling!(
        "Predatory Urge",
        "If the targeted creature leaves the battlefield (or otherwise becomes an illegal target) before the ability resolves, the ability doesn’t resolve. The enchanted creature isn’t dealt damage."
    );
    let (t, giant, _) = predatory_urge(|t, _, bears, _| {
        move_to(t, bears, Zone::Hand(P1));
    });
    assert_eq!(damage_on(&t, giant), 0);
}

#[test]
fn predatory_urge_creature_gone_still_deals_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Predatory Urge",
        "On the other hand, if the enchanted creature leaves the battlefield before the ability resolves, the ability continues to resolve. The enchanted creature deals damage to the targeted creature equal to the power the enchanted creature had as it last existed on the battlefield."
    );
    let (t, _, _) = predatory_urge(|t, giant, _, _| {
        move_to(t, giant, Zone::Hand(P0));
    });
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

// ---------------------------------------------------------------------------------------
// Damage triggers
// ---------------------------------------------------------------------------------------

#[test]
fn pain_for_all_triggers_once_for_simultaneous_damage() {
    cr!("603.2c", "510.2", "120.3");
    ruling!(
        "Pain for All",
        "If the enchanted creature is dealt damage by multiple sources at once, such as by two creatures blocking it, Pain for All’s last ability triggers only once."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Craw Wurm");
    attach_new(&mut t, P0, "Pain for All", giant);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    to_blockers(&mut t, &[(giant, Entity::Player(P1))], &[(b1, giant), (b2, giant)]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(on_stack(&t, "dealt damage"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn mire_blight_triggers_on_noncombat_damage() {
    cr!("603.2", "120.3");
    ruling!(
        "Mire Blight",
        "Mire Blight’s ability triggers when the enchanted creature is dealt any kind of damage, not just combat damage."
    );
    supported("Mire Blight");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    attach_new(&mut t, P0, "Mire Blight", giant);
    cast_spell(&mut t, P0, "Shock", &[Entity::Object(giant)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
}

#[test]
fn eternal_thirst_counter_comes_too_late_for_simultaneous_death() {
    cr!("704.3", "603.3");
    ruling!(
        "Eternal Thirst",
        "If the enchanted creature is dealt lethal damage at the same time as a creature an opponent controls, they’re destroyed at the same time."
    );
    supported("Eternal Thirst");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Eternal Thirst", mine);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    to_blockers(&mut t, &[(mine, Entity::Player(P1))], &[(theirs, mine)]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn trollhide_one_regeneration_shield_handles_lethal_and_deathtouch_damage() {
    cr!("701.19a", "704.5g", "704.5h", "704.3");
    ruling!(
        "Trollhide",
        "If the enchanted creature is dealt lethal damage and is dealt damage by a source with deathtouch during the same combat damage step, a single regeneration shield will save it."
    );
    supported("Trollhide");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Trollhide", giant);
    let rats = t.battlefield(P1, "Typhoid Rats");
    let wurm = t.battlefield(P1, "Craw Wurm");
    to_blockers(
        &mut t,
        &[(giant, Entity::Player(P1))],
        &[(rats, giant), (wurm, giant)],
    );
    add_mana(&mut t, P0, ManaType::G, 2);
    activate_containing(&mut t, P0, giant, "Regenerate").unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    assert!(t.obj_now(giant).tapped);
    assert_eq!(damage_on(&t, giant), 0);
}

#[test]
fn consecrated_by_blood_sacrificed_creatures_deal_no_combat_damage() {
    cr!("510.1", "701.21a", "118.3");
    ruling!(
        "Consecrated by Blood",
        "If the regeneration ability is activated before combat damage is dealt, the two creatures you sacrifice won’t deal combat damage."
    );
    supported("Consecrated by Blood");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Consecrated by Blood", bears);
    let v1 = t.battlefield(P0, "Elite Vanguard");
    let v2 = t.battlefield(P0, "Elite Vanguard");
    to_blockers(
        &mut t,
        &[
            (bears, Entity::Player(P1)),
            (v1, Entity::Player(P1)),
            (v2, Entity::Player(P1)),
        ],
        &[],
    );
    t.answer_choose(P0, &[Entity::Object(v1), Entity::Object(v2)]);
    activate_containing(&mut t, P0, bears, "Regenerate").unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(v1) && !t.on_battlefield(v2));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

// ---------------------------------------------------------------------------------------
// Redirection
// ---------------------------------------------------------------------------------------

/// P0 controls two `aura`s on two Colossal Dreadmaws; P1's Lava Axe targets P0. P0 picks
/// the second replacement: all 5 damage goes to one creature.
fn two_redirections(aura: &str, cast: bool) {
    supported(aura);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let d1 = t.battlefield(P0, "Colossal Dreadmaw");
    let d2 = t.battlefield(P0, "Colossal Dreadmaw");
    for d in [d1, d2] {
        if cast {
            cast_aura(&mut t, P0, aura, d).unwrap();
        } else {
            attach_new(&mut t, P0, aura, d);
        }
    }
    t.answer(P0, DecisionKind::Replacement, Answer::Index(1));
    t.set_step(P1, Step::PrecombatMain);
    cast_spell(&mut t, P1, "Lava Axe", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "{aura}");
    let mut dmg = [damage_on(&t, d1), damage_on(&t, d2)];
    dmg.sort();
    assert_eq!(dmg, [0, 5], "{aura}");
}

#[test]
fn pariah_two_redirections_the_controller_chooses_one() {
    cr!("616.1", "616.1e", "614.1a");
    ruling!(
        "Pariah",
        "If you control multiple Pariahs enchanting different creatures, you choose which redirection effect to apply. You can't divide damage dealt by one source"
    );
    two_redirections("Pariah", false);
}

#[test]
fn saving_grace_two_redirections_the_controller_chooses_one() {
    cr!("616.1", "616.1e", "614.1a");
    ruling!(
        "Saving Grace",
        "If you have more than one Saving Grace enter the battlefield in one turn, all damage that would be dealt at once to you and/or permanents you control is dealt to one of the enchanted creatures of your choice."
    );
    two_redirections("Saving Grace", true);
}

#[test]
fn saving_grace_can_redirect_more_than_toughness_at_once() {
    cr!("614.1a", "510.2", "704.5g");
    ruling!(
        "Saving Grace",
        "More damage can be redirected to the enchanted creature than it has toughness, as long as that damage is all dealt at once"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let v = t.battlefield(P0, "Elite Vanguard");
    let giant = t.battlefield(P0, "Hill Giant");
    cast_aura(&mut t, P0, "Saving Grace", bears).unwrap();
    assert_eq!(t.pt(bears), (2, 5));
    t.set_step(P1, Step::PrecombatMain);
    cast_spell(&mut t, P1, "Pyroclasm", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(v));
    assert_eq!(damage_on(&t, giant), 0);
}

// ---------------------------------------------------------------------------------------
// Combat damage and life
// ---------------------------------------------------------------------------------------

#[test]
fn sunbond_counts_life_gained_even_if_the_total_decreases() {
    cr!("119.10", "702.15b", "510.2");
    ruling!(
        "Sunbond",
        "In some unusual cases, you can gain life even though your life total actually decreases."
    );
    supported("Sunbond");
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    attach_new(&mut t, P0, "Sunbond", hawk);
    let a: Vec<_> = (0..3).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
    t.set_step(P1, Step::BeginningOfCombat);
    let attacks: Vec<_> = a.iter().map(|b| (*b, Entity::Player(P0))).collect();
    to_blockers(&mut t, &attacks, &[(hawk, a[0])]);
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.counters(hawk, "+1/+1"), 2);
}

#[test]
fn staggering_insight_twice_lifelink_redundant_triggers_separate() {
    cr!("702.15f", "603.2");
    ruling!(
        "Staggering Insight",
        "Multiple instances of lifelink on the same creature are redundant. On the other hand, multiple instances of the triggered ability trigger separately"
    );
    supported("Staggering Insight");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Staggering Insight", bears);
    attach_new(&mut t, P0, "Staggering Insight", bears);
    stack_library(&mut t, P0, &["Forest", "Forest", "Forest"]);
    let hand = t.hand_size(P0);
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn celestial_mantle_doubles_a_negative_life_total() {
    cr!("119.5", "104.3b");
    ruling!(
        "Celestial Mantle",
        "If the enchanted creature’s controller has a life total below 0"
    );
    supported("Celestial Mantle");
    supported("Platinum Angel");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Platinum Angel");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Celestial Mantle", bears);
    t.g.players[P0.idx()].life = -4;
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P0), -8);
    assert!(!t.has_lost(P0));
}

/// P0 controls `aura` on P1's Grizzly Bears, which attack P0: P0 draws nothing. On P0's
/// own Bears attacking P1, P0 draws.
fn draw_on_damage_to_an_opponent(aura: &str) {
    supported(aura);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, aura, bears);
    stack_library(&mut t, P0, &["Forest", "Forest"]);
    stack_library(&mut t, P1, &["Forest", "Forest"]);
    t.answer_yes(P0, true);
    t.answer_yes(P1, true);
    t.set_step(P1, Step::BeginningOfCombat);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    to_blockers(&mut t, &[(bears, Entity::Player(P0))], &[]);
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (h0, h1), "{aura}");
    // Positive control.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, aura, bears);
    stack_library(&mut t, P0, &["Forest", "Forest"]);
    t.answer_yes(P0, true);
    let h0 = t.hand_size(P0);
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 1, "{aura}");
}

#[test]
fn keen_sense_and_curiosity_on_an_opponents_creature_damaging_you() {
    cr!("603.2", "102.2");
    ruling!(
        "Keen Sense",
        "If this Aura enchants an opponent's creature, you won't draw a card when that creature damages you."
    );
    ruling!(
        "Curiosity",
        "If you control Curiosity and it's enchanting an opponent's creature, you won't draw a card when that creature deals damage to you."
    );
    draw_on_damage_to_an_opponent("Keen Sense");
    draw_on_damage_to_an_opponent("Curiosity");
}

#[test]
fn dwindle_destroyed_blocker_leaves_the_attacker_blocked() {
    cr!("509.1h", "510.1c", "702.19c");
    ruling!(
        "Dwindle",
        "Once the enchanted creature is destroyed, the attacking creature won’t assign or deal combat damage unless it has trample or is being blocked by another creature."
    );
    supported("Dwindle");
    for (attacker, loss) in [("Hill Giant", 0), ("Colossal Dreadmaw", 6)] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, attacker);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P1, "Dwindle", bears);
        t.set_step(P1, Step::BeginningOfCombat);
        to_blockers(&mut t, &[(a, Entity::Player(P0))], &[(bears, a)]);
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Grizzly Bears"));
        t.advance_to(P1, Step::EndOfCombat);
        assert_eq!(t.life(P0), 20 - loss, "{attacker}");
    }
}

#[test]
fn breath_of_fury_with_no_creature_to_move_to_does_nothing() {
    cr!("303.4d", "704.5m", "608.2c");
    ruling!(
        "Breath of Fury",
        "If there isn’t a legal creature to attach Breath of Fury to after the enchanted creature is sacrificed, you don’t untap your creatures or get an additional combat phase"
    );
    supported("Breath of Fury");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Breath of Fury", bears);
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Breath of Fury"));
    assert!(!t.g.turn.schedule.contains(&Step::DeclareAttackers));
    // Positive control: with another creature, there's an additional combat.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    t.g.tap(other);
    attach_new(&mut t, P0, "Breath of Fury", bears);
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    t.answer_choose(P0, &[Entity::Object(other)]);
    t.resolve_all();
    assert!(!t.obj_now(other).tapped);
    assert!(t.g.turn.schedule.contains(&Step::DeclareAttackers));
}

#[test]
fn any_other_target_excludes_the_damage_source() {
    cr!("115.1", "115.4");
    // Regression for Pain for All's "any other target" (see
    // `pain_for_all_uses_last_known_power`): a source dealing damage "to any other target"
    // can't target itself.
    // (Matoc's other ability isn't supported; the activated one is compiled.)
    let mut t = TestGame::new(2);
    let matoc = t.battlefield(P0, "Matoc, Lavamancer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let cands = ability_targets(&mut t, matoc, 0);
    assert!(!cands.contains(&Entity::Object(matoc)));
    assert!(cands.contains(&Entity::Object(bears)));
    assert!(cands.contains(&Entity::Player(P1)));
}
