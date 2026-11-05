//! Rulings batch P051 — drain triggers and the life a player gained or lost this turn:
//! "if you gained life this turn" counts life gained before the permanent arrived and
//! ignores life lost (CR 603.4, 119.9), "lost 4 or more life this turn" is checked as the
//! end step begins (CR 603.4), "for the first time each turn" triggers once (CR 603.2),
//! triggered abilities of a spell's caster resolve first (CR 603.3), the order of APNAP
//! triggers (CR 603.3b), and "you may pay {1}" once per resolution (CR 603.5).

use crate::r_p044_common::player_counters;
use crate::r_p046_common::{fill_pool, pool};
use crate::r_p051_common::*;
use crate::r_p057_common::{into_beginning_of_combat, pump};
use crate::r_s01_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const BEARS: &str = "Grizzly Bears";

/// `p` gains `gain` life, then loses `loss` life.
fn gain_then_lose(t: &mut TestGame, p: PlayerId, gain: u32, loss: u32) {
    t.g.gain_life(p, gain);
    t.g.lose_life(p, loss);
    t.g.flush_events();
    t.settle();
}

#[test]
fn gained_life_this_turn_counts_before_the_permanent_arrived_and_ignores_losses() {
    cr!("603.4", "119.3");
    ruling!(
        "Syndicate Heavy",
        "Syndicate Heavy's last ability looks at how much life you've gained in the turn, even if it wasn't on the battlefield when you gained life. It doesn't care if you also lost life, even if you lost more life than you gained."
    );
    ruling!(
        "Restless Bloodseeker // Bloodsoaked Reveler",
        "The first ability of both Restless Bloodseeker and Bloodsoaked Reveler will trigger if you gained life at any point during the turn, even if you also lost that much or more life during the same turn."
    );
    supported("Syndicate Heavy");
    supported("Restless Bloodseeker // Bloodsoaked Reveler");
    for gain in [3, 4] {
        let mut t = TestGame::new(2);
        gain_then_lose(&mut t, P0, gain, 10);
        t.battlefield(P0, "Syndicate Heavy");
        t.battlefield(P0, "Restless Bloodseeker // Bloodsoaked Reveler");
        t.advance_to(P0, Step::End);
        t.resolve_all();
        let clues = with_subtype(&t, P0, "Clue").len();
        assert_eq!(clues, (gain == 4) as usize, "gained {gain}");
        assert_eq!(with_subtype(&t, P0, "Blood").len(), 1, "gained {gain}");
    }
    // No life gained: no Blood.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Restless Bloodseeker // Bloodsoaked Reveler");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Blood").is_empty());
}

#[test]
fn yshtola_checks_life_lost_as_the_end_step_begins() {
    cr!("603.4");
    ruling!(
        "Y'shtola, Night's Blessed",
        "Y'shtola's second ability checks at the moment it would trigger to see if a player lost 4 or more life this turn. If none did, the ability won't trigger at all. Once an end step begins, it's too late to cause a player to lose life in order to cause this ability to trigger."
    );
    supported("Y'shtola, Night's Blessed");
    for lost in [3, 4] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Y'shtola, Night's Blessed");
        gain_then_lose(&mut t, P1, 0, lost);
        let hand = t.hand_size(P0);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "draw a card"), (lost == 4) as usize);
        // Too late now: P1 loses another life during the end step.
        gain_then_lose(&mut t, P1, 0, 1);
        assert_eq!(triggers_on_stack(&t, "draw a card"), (lost == 4) as usize);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + (lost == 4) as usize, "lost {lost}");
    }
}

#[test]
fn yshtola_cast_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5", "701.6a");
    ruling!(
        "Y'shtola, Night's Blessed",
        "Y'shtola's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack."
    );
    supported("Y'shtola, Night's Blessed");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Y'shtola, Night's Blessed");
    let spell = cast_pool(&mut t, P0, "Divination", &[]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "2 damage to each opponent"), 1);
    // P1 counters Divination in response.
    t.set_step(P0, Step::PrecombatMain);
    fill_pool(&mut t, P1, 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(obj(spell)).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Divination"));
    assert_eq!(lives(&t, 2), vec![20, 20]);
    t.resolve();
    assert_eq!(lives(&t, 2), vec![22, 18]);
    // Without a counterspell, the trigger resolves before the spell.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Y'shtola, Night's Blessed");
    let hand = t.hand_size(P0);
    cast_pool(&mut t, P0, "Divination", &[]);
    t.resolve();
    assert_eq!(lives(&t, 2), vec![22, 18]);
    assert_eq!(t.hand_size(P0), hand, "Divination hasn't resolved yet");
}

#[test]
fn gontis_machinations_gives_one_energy_each_turn() {
    cr!("603.2");
    ruling!(
        "Gonti's Machinations",
        "The triggered ability gives you at most one energy counter each turn, regardless of how much life you lost."
    );
    supported("Gonti's Machinations");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gonti's Machinations");
    gain_then_lose(&mut t, P0, 0, 5);
    t.resolve_all();
    gain_then_lose(&mut t, P0, 0, 1);
    t.resolve_all();
    assert_eq!(player_counters(&t, P0, "energy"), 1);
    // The next turn, again.
    t.advance_to(P1, Step::Upkeep);
    gain_then_lose(&mut t, P0, 0, 2);
    t.resolve_all();
    assert_eq!(player_counters(&t, P0, "energy"), 2);
}

#[test]
fn garrulous_sycophant_checks_the_monarch_as_the_end_step_begins_and_on_resolution() {
    cr!("603.4", "725.1");
    ruling!(
        "Garrulous Sycophant",
        "The last ability of Garrulous Sycophant checks to see if you're the monarch as your end step begins. If you're not, the ability won't trigger at all. You won't be able to do anything that would make you the monarch during your end step in time to have that ability trigger. The ability will also check to see if you're the monarch as it tries to resolve. If you're not the monarch at that time, the ability will have no effect."
    );
    supported("Garrulous Sycophant");
    // Not the monarch as the end step begins; the monarch during it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garrulous Sycophant");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "each opponent loses 1 life"), 0);
    mtg_engine::designations::become_monarch(&mut t.g, P0);
    t.settle();
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![20, 20]);
    // The monarch as it begins, not as the ability resolves.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garrulous Sycophant");
    mtg_engine::designations::become_monarch(&mut t.g, P0);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "each opponent loses 1 life"), 1);
    mtg_engine::designations::become_monarch(&mut t.g, P1);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![20, 20]);
    // The monarch throughout.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Garrulous Sycophant");
    mtg_engine::designations::become_monarch(&mut t.g, P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![21, 19]);
}

#[test]
fn revenge_of_ravens_resolves_before_the_attackers_triggers() {
    cr!("603.3b", "104.3b", "800.4a");
    ruling!(
        "Revenge of Ravens",
        "The triggered ability of Revenge of Ravens resolves before any abilities the attacking player controls. If this causes the attacking player to lose the game, that player's triggered abilities won't resolve and no combat damage will be dealt."
    );
    supported("Revenge of Ravens");
    supported("Brutal Hordechief");
    // Three players, so the game goes on without the attacking player.
    let mut t = TestGame::new(3);
    t.battlefield(P1, "Revenge of Ravens");
    let chief = t.battlefield(P0, "Brutal Hordechief");
    t.g.players[0].life = 1;
    attack_with(&mut t, &[(chief, pl(P1))]);
    t.resolve();
    assert!(t.has_lost(P0));
    t.resolve_all();
    t.advance_to_step(Step::End);
    // Revenge of Ravens drained P0; Brutal Hordechief's trigger didn't resolve and no
    // combat damage was dealt.
    assert_eq!(t.life(P1), 21);
}

#[test]
fn night_market_lookout_triggers_when_it_becomes_tapped_by_attacking() {
    cr!("603.2", "508.1f");
    ruling!(
        "Night Market Lookout",
        "This is a triggered ability, not an activated ability. It doesn't allow you to tap Night Market Lookout whenever you want; rather, you need some other way of tapping it, such as by attacking."
    );
    supported("Night Market Lookout");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Night Market Lookout");
    assert!(!t
        .obj_now(l)
        .chars
        .abilities
        .iter()
        .any(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_))));
    attack_with(&mut t, &[(l, pl(P1))]);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![21, 19]);
}

#[test]
fn wispdrinker_vampire_checks_power_as_the_creature_enters() {
    cr!("603.2", "603.6a", "122.6");
    ruling!(
        "Wispdrinker Vampire",
        "Wispdrinker Vampire's triggered ability checks the power of a creature only at the moment it enters the battlefield. If it enters with counters, those counters are included. If that creature's power is 2 or less when it enters the battlefield but becomes greater than 2 after the ability triggers, each opponent will still lose 1 life and you'll still gain 1 life."
    );
    supported("Wispdrinker Vampire");
    // A 2/2 that grows after the ability triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wispdrinker Vampire");
    let bears = t.enter(P0, BEARS);
    t.settle();
    pump(&mut t, bears, 3, 0);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![21, 19]);
    // A 0/0 entering with three +1/+1 counters (Walking Ballista for X = 3) is a 3/3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wispdrinker Vampire");
    let ballista = t.hand(P0, "Walking Ballista");
    fill_pool(&mut t, P0, 6);
    t.cast(P0, ballista).x(3).go();
    t.resolve_all();
    assert_eq!(t.pt(t.named_on_battlefield("Walking Ballista")[0]), (3, 3));
    assert_eq!(lives(&t, 2), vec![20, 20]);
}

#[test]
fn wispdrinker_vampire_grants_only_to_creatures_controlled_on_resolution() {
    cr!("611.2c");
    ruling!(
        "Wispdrinker Vampire",
        "Wispdrinker Vampire's activated ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won't get deathtouch or lifelink."
    );
    supported("Wispdrinker Vampire");
    use mtg_engine::keywords::KeywordKind;
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Wispdrinker Vampire");
    let bears = t.battlefield(P0, BEARS);
    assert!(activate_text(&mut t, P0, w, "deathtouch", &[]));
    t.resolve_all();
    let later = t.battlefield(P0, BEARS);
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Lifelink));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Deathtouch));
    assert!(!t.obj_now(later).has_keyword(KeywordKind::Lifelink));
    assert!(!t.obj_now(later).has_keyword(KeywordKind::Deathtouch));
}

/// Resolves a "you may pay {1}. If you do, ..." drain trigger with plenty of mana,
/// answering yes to every payment question. Returns the payment questions P0 was asked
/// and the mana spent.
fn pay_once(t: &mut TestGame) -> (usize, usize) {
    fill_pool(t, P0, 5);
    let before = pool(t, P0);
    let from = t.asked().len();
    for _ in 0..5 {
        t.answer_yes(P0, true);
    }
    t.resolve_all();
    let asked = t.asked()[from..]
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }))
        .count();
    t.clear_answers();
    (asked, before - pool(t, P0))
}

#[test]
fn may_pay_one_drains_can_be_paid_only_once() {
    cr!("603.5", "118.1");
    ruling!(
        "Underhanded Designs",
        "You can pay {1} only once each time Underhanded Designs's triggered ability resolves. You can't pay more to cause your opponents to lose more life."
    );
    supported("Underhanded Designs");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Underhanded Designs");
    t.enter(P0, "Ornithopter");
    t.settle();
    assert_eq!(pay_once(&mut t), (1, 1));
    assert_eq!(lives(&t, 2), vec![21, 19]);

    ruling!(
        "Faith of the Devoted",
        "While resolving the triggered ability of Faith of the Devoted, you can't pay {1} multiple times to multiply the effect."
    );
    supported("Faith of the Devoted");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Faith of the Devoted");
    let card = t.hand(P0, BEARS);
    t.g.discard(P0, card, None);
    t.g.flush_events();
    t.settle();
    assert_eq!(pay_once(&mut t), (1, 1));
    assert_eq!(lives(&t, 2), vec![22, 18]);
}

#[test]
fn sinister_monolith_triggers_with_no_creatures_on_the_battlefield() {
    cr!("506.1");
    ruling!(
        "Sinister Monolith",
        "The beginning of combat step happens every turn, even if there are no creatures on the battlefield that could attack."
    );
    supported("Sinister Monolith");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sinister Monolith");
    assert!(creatures(&t, P0).is_empty() && creatures(&t, P1).is_empty());
    into_beginning_of_combat(&mut t, P0);
    t.resolve_all();
    assert_eq!(lives(&t, 2), vec![21, 19]);
}
