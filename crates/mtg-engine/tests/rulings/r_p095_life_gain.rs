//! Rulings batch P095 — life gain and the permanents that care about it: "whenever you
//! gain life" triggers (one per life-gaining event, CR 119.9, 702.15b), replacement
//! effects that change the amount gained (CR 614.1a), statics that depend on the life
//! total (CR 611.3a, 613.1), intervening "if" clauses about life (CR 603.4), life gained
//! this turn, and Two-Headed Giant's shared life total (CR 810.9).

use crate::r_p095_common::*;
use crate::r_p096_common::{congregate, gain_one, one_trampling_lifelinker, two_lifelinkers};
use crate::r_s01_common::{supported, tokens, with_subtype};
use crate::r_s33_common::two_headed_giant;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn has_double_strike(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).has_keyword(KeywordKind::DoubleStrike)
}

// ---------------------------------------------------------------------------------------
// Twinblade Paladin
// ---------------------------------------------------------------------------------------

#[test]
fn twinblade_paladin_triggers_once_per_life_gaining_event() {
    cr!("119.9", "603.2c", "702.15b");
    ruling!(
        "Twinblade Paladin",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Twinblade Paladin's first ability will trigger twice. However, if a single creature you control with lifelink deals combat damage to multiple creatures, players, planeswalkers, and/or battles at the same time (perhaps because it has trample or was blocked by more than one creature), the ability will trigger only once."
    );
    ruling!(
        "Twinblade Paladin",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and Twinblade Paladin's first ability triggers only once."
    );
    ruling!(
        "Twinblade Paladin",
        "Twinblade Paladin's first ability triggers just once for each life-gaining event, whether it's 1 life from Dazzling Angel or 4 life from Apothecary Stomper."
    );
    supported("Twinblade Paladin");
    let mut t = TestGame::new(2);
    let paladin = t.battlefield(P0, "Twinblade Paladin");
    two_lifelinkers(&mut t);
    assert_eq!(t.counters(paladin, "+1/+1"), 2);
    let mut t = TestGame::new(2);
    let paladin = t.battlefield(P0, "Twinblade Paladin");
    one_trampling_lifelinker(&mut t);
    assert_eq!(t.counters(paladin, "+1/+1"), 1);
    // 1 life (Soul Warden), then 8 life for each creature (Congregate): one each.
    let mut t = TestGame::new(2);
    let paladin = t.battlefield(P0, "Twinblade Paladin");
    gain_one(&mut t, P0);
    assert_eq!(t.counters(paladin, "+1/+1"), 1);
    congregate(&mut t, P0);
    assert_eq!(abilities_from(&t, paladin), 1);
    t.resolve_all();
    assert_eq!(t.counters(paladin, "+1/+1"), 2);
}

#[test]
fn twinblade_paladin_gaining_double_strike_after_first_strike_damage() {
    cr!("510.4", "613.1f");
    ruling!(
        "Twinblade Paladin",
        "If Twinblade Paladin gains double strike after combat damage has already been dealt (most likely because a creature with lifelink dealt combat damage), Twinblade Paladin will deal only regular combat damage."
    );
    // P0 at 24 attacks with Paladin (3/3) and Knight of Meadowgrain (2/2 first strike,
    // lifelink). The Knight's first-strike damage brings P0 to 26: Paladin gets double
    // strike (and a +1/+1 counter) but deals only regular combat damage.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 24);
    let paladin = t.battlefield(P0, "Twinblade Paladin");
    let knight = t.battlefield(P0, "Knight of Meadowgrain");
    assert!(!has_double_strike(&t, paladin));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(paladin, Entity::Player(P1)), (knight, Entity::Player(P1))],
        &[],
    );
    assert!(has_double_strike(&t, paladin));
    assert_eq!(t.counters(paladin, "+1/+1"), 1);
    assert_eq!(t.life(P0), 26);
    assert_eq!(t.life(P1), 20 - 2 - 4);
}

#[test]
fn twinblade_paladin_counter_comes_too_late_to_save_it() {
    cr!("704.5g", "704.3", "603.3");
    ruling!(
        "Twinblade Paladin",
        "If Twinblade Paladin is dealt lethal damage at the same time that you gain life, it won't receive a counter from its first ability in time to save it."
    );
    // Paladin (3/3) and Child of Night (2/1 lifelink) attack; Hill Giant (3/3) blocks
    // Paladin.
    let mut t = TestGame::new(2);
    let paladin = t.battlefield(P0, "Twinblade Paladin");
    let child = t.battlefield(P0, "Child of Night");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(paladin, Entity::Player(P1)), (child, Entity::Player(P1))],
        &[(giant, paladin)],
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert!(t.in_graveyard(P0, "Twinblade Paladin"));
}

#[test]
fn twinblade_paladin_loses_double_strike_after_first_strike_damage_to_you() {
    cr!("510.4", "702.4c", "611.3a");
    ruling!(
        "Twinblade Paladin",
        "If you're dealt first-strike combat damage that brings your life total below 25, Twinblade Paladin will immediately lose double strike. It won't deal regular combat damage unless you bring your life total back above 25 before the regular combat damage step."
    );
    // P0 at 26. P1 attacks with White Knight (2/2 first strike, unblocked) and
    // Indomitable Ancients (2/10), which Paladin blocks. Paladin deals its first-strike
    // damage (3), then P0 is at 24 and Paladin deals no regular combat damage.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 26);
    let paladin = t.battlefield(P0, "Twinblade Paladin");
    let knight = t.battlefield(P1, "White Knight");
    let ancients = t.battlefield(P1, "Indomitable Ancients");
    assert!(has_double_strike(&t, paladin));
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(
        &[(knight, Entity::Player(P0)), (ancients, Entity::Player(P0))],
        &[(paladin, ancients)],
    );
    assert_eq!(t.life(P0), 24);
    assert!(!has_double_strike(&t, paladin));
    assert_eq!(crate::r_s29_common::damage_marked(&t, ancients), 3);
}

#[test]
fn twinblade_paladin_in_two_headed_giant() {
    cr!("810.9", "810.9a");
    ruling!(
        "Twinblade Paladin",
        "In a Two-Headed Giant game, Twinblade Paladin has double strike as long as your team has 25 or more life."
    );
    ruling!(
        "Twinblade Paladin",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Twinblade Paladin's first ability to trigger, even though it caused your team's life total to increase."
    );
    let mut t = two_headed_giant();
    let paladin = t.battlefield(P0, "Twinblade Paladin");
    crate::r_p096_common::set_team_life(&mut t, P0, 25);
    t.g.recompute();
    assert!(has_double_strike(&t, paladin));
    crate::r_p096_common::set_team_life(&mut t, P0, 24);
    t.g.recompute();
    assert!(!has_double_strike(&t, paladin));
    // P1 (P0's teammate) gains 4 life: the team is at 28, Paladin has double strike, but
    // its first ability doesn't trigger.
    cast_new(&mut t, P1, "Sacred Nectar", &[]);
    t.resolve();
    t.settle();
    assert_eq!(t.life(P0), 28);
    assert_eq!(abilities_from(&t, paladin), 0);
    assert!(has_double_strike(&t, paladin));
}

// ---------------------------------------------------------------------------------------
// "You gain that much life plus 1 / twice that much life instead"
// ---------------------------------------------------------------------------------------

#[test]
fn plus_one_replacements_apply_once_per_life_gaining_event() {
    cr!("614.1a", "614.5", "702.15b");
    ruling!(
        "Pest Rescuer",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Pest Rescuer's last ability will apply twice. However, if a single creature you control with lifelink deals combat damage to multiple creatures, players, planeswalkers, and/or battles at the same time (perhaps because it has trample or was blocked by more than one creature), the ability will apply only once."
    );
    ruling!(
        "Angel of Vitality",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Angel of Vitality's second ability will apply twice. However, if a single creature you control with lifelink deals combat damage to multiple creatures, players, planeswalkers, and/or battles at the same time (perhaps because it has trample or was blocked by more than one creature), the ability will apply only once."
    );
    ruling!(
        "Honor Troll",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Honor Troll's second ability will apply twice. However, if a single creature you control with lifelink deals combat damage to multiple creatures, players, and/or planeswalkers at the same time (perhaps because it has trample or was blocked by more than one creature), the ability will apply only once."
    );
    for card in ["Pest Rescuer", "Angel of Vitality", "Honor Troll"] {
        supported(card);
        // Two Child of Night: 2 + 1 twice.
        let mut t = TestGame::new(2);
        t.battlefield(P0, card);
        let from = t.g.turn_events.len();
        two_lifelinkers(&mut t);
        assert_eq!(gains_since(&t, from, P0), vec![3, 3], "{card}");
        // One trampling lifelinker dealing damage to a creature and a player: 3 + 1.
        let mut t = TestGame::new(2);
        t.battlefield(P0, card);
        let from = t.g.turn_events.len();
        one_trampling_lifelinker(&mut t);
        assert_eq!(gains_since(&t, from, P0), vec![4], "{card}");
    }
}

#[test]
fn pest_rescuer_stacks_and_applies_once_for_each() {
    cr!("614.5", "616.1");
    ruling!(
        "Pest Rescuer",
        "If you control two Pest Rescuers and you would gain life, you gain that much life plus 2. A third Pest Rescuer has you gain that much life plus 3, and so on."
    );
    ruling!(
        "Pest Rescuer",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and the last ability of Pest Rescuer applies only once."
    );
    for n in 1..=3 {
        let mut t = TestGame::new(2);
        for _ in 0..n {
            t.battlefield(P0, "Pest Rescuer");
        }
        gain(&mut t, P0, 4);
        assert_eq!(t.life(P0), 24 + n);
    }
    // Congregate with Pest Rescuer and three Bears: 2 × 4 = 8, plus 1 once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pest Rescuer");
    let from = t.g.turn_events.len();
    congregate(&mut t, P0);
    assert_eq!(gains_since(&t, from, P0), vec![9]);
}

#[test]
fn pest_rescuer_upkeep_trigger_checks_for_pests_twice() {
    cr!("603.4");
    ruling!(
        "Pest Rescuer",
        "Pest Rescuer's first ability checks at the moment it would trigger to see if you control any Pest creature tokens. If you do, the ability won't trigger at all. If it does trigger, the ability will check again as it tries to resolve. If you control any Pest creature tokens at that time, the ability won't resolve and none of its effects will happen."
    );
    // Two Pest Rescuers and no Pest: both trigger; the first makes a Pest, so the second
    // does nothing.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Pest Rescuer");
    let b = t.battlefield(P0, "Pest Rescuer");
    t.set_step(P1, Step::End);
    to_upkeep(&mut t, P0);
    assert_eq!(abilities_from(&t, a) + abilities_from(&t, b), 2);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Pest").len(), 1);
    // With a Pest token: no trigger at all (in P1's upkeep, "each upkeep").
    to_upkeep(&mut t, P1);
    assert_eq!(abilities_from(&t, a) + abilities_from(&t, b), 0);
}

#[test]
fn rhox_faithmender_doubles_the_life_gained_by_setting_a_life_total() {
    cr!("119.5", "614.1a");
    ruling!(
        "Rhox Faithmender",
        "If an effect sets your life total to a specific number, and that number is higher than your current life total, the effect will cause you to gain life equal to the difference. Rhox Faithmender will then double that number. For example, if you have 3 life and an effect says that your life total \"becomes 10,\" your life total will actually become 17."
    );
    supported("Rhox Faithmender");
    // Blessed Wind: "Target player's life total becomes 20." From 3: gain 17, doubled.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rhox Faithmender");
    set_life(&mut t, P0, 3);
    cast_new(&mut t, P0, "Blessed Wind", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 37);
}

#[test]
fn rhox_faithmender_two_headed_giant_teammate_isnt_affected() {
    cr!("810.9", "614.1a");
    ruling!(
        "Rhox Faithmender",
        "In a Two-Headed Giant game, only Rhox Faithmender's controller is affected by it. If that player's teammate gains life, Rhox Faithmender will have no effect, even when that life gain is applied to the shared team life total."
    );
    let mut t = two_headed_giant();
    t.battlefield(P0, "Rhox Faithmender");
    gain(&mut t, P1, 4);
    assert_eq!(t.life(P0), 34);
    gain(&mut t, P0, 4);
    assert_eq!(t.life(P0), 42);
}

#[test]
fn alhammarrets_archives_multiply() {
    cr!("614.5", "616.1");
    ruling!(
        "Alhammarret's Archive",
        "Because Alhammarret’s Archive is legendary, it’s unlikely that one player will control two. However, if that happens, life gained by that player will be multiplied by four. Three Archives will multiply that life gain by eight, and so on."
    );
    supported("Alhammarret's Archive");
    // Mirror Gallery: "The \"legend rule\" doesn't apply."
    for (n, life) in [(2, 24), (3, 28)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Mirror Gallery");
        for _ in 0..n {
            t.battlefield(P0, "Alhammarret's Archive");
        }
        t.settle();
        assert_eq!(t.named_on_battlefield("Alhammarret's Archive").len(), n);
        gain(&mut t, P0, 1);
        assert_eq!(t.life(P0), life);
    }
}

// ---------------------------------------------------------------------------------------
// Statics that depend on the life total
// ---------------------------------------------------------------------------------------

#[test]
fn damage_stays_marked_when_the_life_total_drops_below_25() {
    cr!("120.6", "704.5g", "611.3a");
    ruling!(
        "Angel of Vitality",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to Angel of Vitality may become lethal if your life total drops below 25 during that turn."
    );
    ruling!(
        "Honor Troll",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to Honor Troll may become lethal if your life total drops below 25 during that turn."
    );
    for card in ["Angel of Vitality", "Honor Troll"] {
        let mut t = TestGame::new(2);
        set_life(&mut t, P0, 25);
        let id = t.battlefield(P0, card);
        let shock = t.battlefield(P1, "Raging Goblin");
        damage(&mut t, shock, Entity::Object(id), 3);
        assert!(t.on_battlefield(id), "{card}");
        lose(&mut t, P0, 1);
        assert!(t.in_graveyard(P0, card), "{card}");
    }
}

#[test]
fn angel_of_vitality_life_gain_applies_before_lethal_damage_is_checked() {
    cr!("510.2", "704.3", "611.3a");
    ruling!(
        "Angel of Vitality",
        "If Angel of Vitality is dealt damage and your life total is increased above 25 before state-based actions are performed (most likely because Angel of Vitality is dealt combat damage at the same time that a creature you control with lifelink deals combat damage), Angel of Vitality's last ability applies before checking whether it's destroyed for having lethal damage marked on it."
    );
    // P0 at 22 attacks with Angel (2/2) and Child of Night (2/1 lifelink); Hill Giant
    // blocks the Angel: 3 damage to the Angel as P0 gains 2 + 1 (25 life).
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 22);
    let angel = t.battlefield(P0, "Angel of Vitality");
    let child = t.battlefield(P0, "Child of Night");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(angel, Entity::Player(P1)), (child, Entity::Player(P1))],
        &[(giant, angel)],
    );
    assert_eq!(t.life(P0), 25);
    assert!(t.on_battlefield(angel));
    assert_eq!(t.pt(angel), (4, 4));
}

// ---------------------------------------------------------------------------------------
// "Whenever you gain life" triggers
// ---------------------------------------------------------------------------------------

#[test]
fn gain_triggers_trigger_once_per_lifelink_creature() {
    cr!("119.9", "702.15b", "603.2c");
    ruling!(
        "Dawn of Hope",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Dawn of Hope's ability will trigger twice. However, if a single creature you control with lifelink deals combat damage to multiple creatures, players, planeswalkers, and/or battles at the same time (perhaps because it has trample or was blocked by more than one creature), the ability will trigger only once."
    );
    ruling!(
        "Wax-Wane Witness",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Wax-Wane Witness’s last ability will trigger twice. However, if a single creature you control with lifelink deals combat damage to multiple creatures, players, and/or planeswalkers at the same time (perhaps because it has trample or was blocked by more than one creature), the ability will trigger only once."
    );
    ruling!(
        "Dawn of Hope",
        "Dawn of Hope's first ability triggers just once for each life-gaining event, whether it's 1 life from Vicious Rumors or 3 life from Creeping Chill."
    );
    // Dawn of Hope: decline to pay; count the questions. Wax-Wane Witness: +1/+0 each.
    for (two, n) in [(true, 2), (false, 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Dawn of Hope");
        t.lands(P0, "Wastes", 4);
        let witness = t.battlefield(P0, "Wax-Wane Witness");
        let from = t.asked().len();
        if two {
            two_lifelinkers(&mut t);
        } else {
            one_trampling_lifelinker(&mut t);
        }
        let asked = t.asked()[from..]
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }))
            .count();
        assert_eq!(asked, n);
        assert_eq!(t.pt(witness), (2 + n as i32, 4));
    }
    // 1 life, then 3 life: one trigger each.
    let mut t = TestGame::new(2);
    let dawn = t.battlefield(P0, "Dawn of Hope");
    t.battlefield(P0, "Soul Warden");
    t.enter(P0, "Grizzly Bears");
    t.resolve();
    t.settle();
    assert_eq!(abilities_from(&t, dawn), 1);
    t.resolve_all();
    gain(&mut t, P0, 3);
    assert_eq!(abilities_from(&t, dawn), 1);
}

#[test]
fn cat_collector_triggers_only_for_the_first_life_gain_each_turn() {
    cr!("603.2c", "119.9");
    ruling!(
        "Cat Collector",
        "Cat Collector's last ability triggers just once and creates just one Cat token for the first life-gaining event on each of your turns, no matter how much life you gain."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cat Collector");
    gain(&mut t, P0, 5);
    t.resolve_all();
    gain(&mut t, P0, 1);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Cat").len(), 1);
    // Not on P1's turn; again on P0's next turn, once.
    t.advance_to(P1, Step::PrecombatMain);
    gain(&mut t, P0, 2);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Cat").len(), 1);
    t.advance_to(P0, Step::PrecombatMain);
    gain(&mut t, P0, 2);
    t.resolve_all();
    gain(&mut t, P0, 2);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Cat").len(), 2);
}

// ---------------------------------------------------------------------------------------
// Life gained this turn
// ---------------------------------------------------------------------------------------

#[test]
fn life_gained_this_turn_ignores_life_lost() {
    cr!("119.3", "603.4");
    ruling!(
        "Angelic Accord",
        "Angelic Accord’s ability checks how much life you’ve gained during the turn, not what your life total is compared to what it was when the turn began. For example, if you start the turn at 10 life, gain 6 life during the turn, then lose 6 life later that turn, the ability will trigger."
    );
    ruling!(
        "Haliya, Guided by Light",
        "Haliya, Guided by Light’s second ability checks how much life you’ve gained during the turn at the time it resolves, not what your life total is compared to what it was when the turn began. For example, if you start the turn at 10 life, gain 6 life during the turn, then lose 6 life later that turn, Haliya’s second ability will cause you to draw a card as it resolves."
    );
    ruling!(
        "Griffin Aerie",
        "Griffin Aerie's ability looks at how much life you've gained in the turn, even if it wasn't on the battlefield when you gained life. It doesn't care if you also lost life, even if you lost more life than you gained."
    );
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 10);
    gain(&mut t, P0, 6);
    lose(&mut t, P0, 8);
    assert_eq!(t.life(P0), 8);
    // These enter after the life was gained.
    t.battlefield(P0, "Angelic Accord");
    t.battlefield(P0, "Haliya, Guided by Light");
    t.battlefield(P0, "Griffin Aerie");
    let hand = t.hand_size(P0);
    to_end_step(&mut t, P0);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Angel").len(), 1);
    assert_eq!(with_subtype(&t, P0, "Griffin").len(), 1);
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------------------
// Intervening "if" clauses about life
// ---------------------------------------------------------------------------------------

#[test]
fn life_conditions_are_checked_on_trigger_and_on_resolution() {
    cr!("603.4");
    ruling!(
        "Felidar Sovereign",
        "Felidar Sovereign's triggered ability checks to see if you have 40 or more life as your upkeep begins. If you don't, the ability won't trigger at all. If you do, the ability will check again as it tries to resolve. If you don't have 40 or more life at that time, the ability won't do anything."
    );
    ruling!(
        "Convalescence",
        "The life total is checked when it triggers and the ability will be put on the stack only if you have 10 life or less. It checks again on resolution and does nothing if you have more life by then."
    );
    supported("Felidar Sovereign");
    supported("Convalescence");
    // Felidar Sovereign: 39 life, no trigger; 40 life, then 39 before it resolves.
    let mut t = TestGame::new(2);
    let felidar = t.battlefield(P0, "Felidar Sovereign");
    set_life(&mut t, P0, 39);
    to_upkeep(&mut t, P0);
    assert_eq!(abilities_from(&t, felidar), 0);
    set_life(&mut t, P0, 40);
    to_upkeep(&mut t, P0);
    assert_eq!(abilities_from(&t, felidar), 1);
    lose(&mut t, P0, 1);
    t.resolve_all();
    assert!(!t.has_lost(P1));
    // Convalescence: 11 life, no trigger; 10 life, then 11 before it resolves.
    let mut t = TestGame::new(2);
    let conv = t.battlefield(P0, "Convalescence");
    set_life(&mut t, P0, 11);
    to_upkeep(&mut t, P0);
    assert_eq!(abilities_from(&t, conv), 0);
    set_life(&mut t, P0, 10);
    to_upkeep(&mut t, P0);
    assert_eq!(abilities_from(&t, conv), 1);
    set_life(&mut t, P0, 11);
    t.resolve_all();
    assert_eq!(t.life(P0), 11);
    // ... and 10 at both times: 1 life.
    set_life(&mut t, P0, 10);
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P0), 11);
}

#[test]
fn glorious_enforcer_more_life_than_an_opponent() {
    cr!("603.4", "611.2a");
    ruling!(
        "Glorious Enforcer",
        "Glorious Enforcer's triggered ability checks whether you have more life than an opponent at the beginning of each combat to see if it triggers. If you don't, the ability won't trigger at all. The ability will check again as it tries to resolve. If you don't at that time, the ability won't do anything. You must have more life than an opponent at both times for Glorious Enforcer to get double strike. It does not, however, need to be the same opponent both times."
    );
    ruling!(
        "Glorious Enforcer",
        "After the triggered ability resolves, Glorious Enforcer will continue to have double strike until the end of the turn no matter what players' life totals are."
    );
    supported("Glorious Enforcer");
    // Three players: P0 20, P1 25, P2 15 at the beginning of combat (more than P2);
    // before it resolves P1 is at 10 and P2 at 30 (more than P1): double strike.
    let mut t = TestGame::new(3);
    let enforcer = t.battlefield(P0, "Glorious Enforcer");
    set_life(&mut t, P1, 25);
    set_life(&mut t, P2, 15);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(abilities_from(&t, enforcer), 1);
    set_life(&mut t, P1, 10);
    set_life(&mut t, P2, 30);
    t.resolve_all();
    assert!(has_double_strike(&t, enforcer));
    // Afterwards, whatever the life totals.
    set_life(&mut t, P0, 1);
    t.g.recompute();
    assert!(has_double_strike(&t, enforcer));
    // No opponent with less life: no trigger.
    let mut t = TestGame::new(2);
    let enforcer = t.battlefield(P0, "Glorious Enforcer");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(abilities_from(&t, enforcer), 0);
    // Triggered, but not more life on resolution: nothing.
    let mut t = TestGame::new(2);
    let enforcer = t.battlefield(P0, "Glorious Enforcer");
    set_life(&mut t, P1, 19);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(abilities_from(&t, enforcer), 1);
    set_life(&mut t, P1, 21);
    t.resolve_all();
    assert!(!has_double_strike(&t, enforcer));
}

#[test]
fn feudkillers_verdict_compares_with_any_opponent_without_targeting() {
    cr!("608.2c", "115.1");
    ruling!(
        "Feudkiller's Verdict",
        "Feudkiller's Verdict doesn't target an opponent. If you have more life than any opponent when that part of the effect happens, you'll put the token onto the battlefield."
    );
    supported("Feudkiller's Verdict");
    // P0 at 5, P1 at 30, P2 at 10: after gaining 10, P0 has more than P2.
    let mut t = TestGame::new(3);
    set_life(&mut t, P0, 5);
    set_life(&mut t, P1, 30);
    set_life(&mut t, P2, 10);
    let from = t.asked().len();
    cast_new(&mut t, P0, "Feudkiller's Verdict", &[]);
    t.resolve_all();
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    assert_eq!(t.life(P0), 15);
    assert_eq!(with_subtype(&t, P0, "Giant").len(), 1);
    // No opponent with less life after the gain: no token.
    let mut t = TestGame::new(2);
    set_life(&mut t, P0, 5);
    cast_new(&mut t, P0, "Feudkiller's Verdict", &[]);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
}

// ---------------------------------------------------------------------------------------
// Angel of Destiny
// ---------------------------------------------------------------------------------------

#[test]
fn angel_of_destiny_a_player_who_lost_to_first_strike_damage_gains_no_life() {
    cr!("510.4", "704.5a", "800.4a");
    ruling!(
        "Angel of Destiny",
        "All first-strike combat damage is dealt at the same time, and, separately, all regular combat damage is dealt at the same time. If one of those amounts of combat damage is enough to cause the player being dealt damage to lose the game, Angel of Destiny's first triggered ability has you gain life, but the other player has already lost the game and can't gain life."
    );
    supported("Angel of Destiny");
    // Three players; P1 at 2 life. Angel of Destiny (2/6 double strike) attacks P1.
    let mut t = TestGame::new(3);
    let angel = t.battlefield(P0, "Angel of Destiny");
    set_life(&mut t, P1, 2);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(angel, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.has_lost(P1));
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 0);
}

#[test]
fn angel_of_destiny_must_be_on_the_battlefield_at_your_end_step() {
    cr!("603.4", "113.6");
    ruling!(
        "Angel of Destiny",
        "If Angel of Destiny leaves the battlefield before your end step, its last ability won't trigger and won't cause a player to lose the game, even if Angel of Destiny attacked that player earlier."
    );
    for leaves in [false, true] {
        let mut t = TestGame::new(3);
        let angel = t.battlefield(P0, "Angel of Destiny");
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(angel, Entity::Player(P1))], &[]);
        t.resolve_all();
        set_life(&mut t, P0, 35);
        if leaves {
            bounce(&mut t, angel);
        }
        to_end_step(&mut t, P0);
        t.resolve_all();
        assert_eq!(t.has_lost(P1), !leaves);
        assert!(!t.has_lost(P2));
    }
}
