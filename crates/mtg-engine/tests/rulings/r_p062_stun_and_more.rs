//! Rulings batch P062 — stun counters (CR 122.1d) put by tapping effects on creatures that
//! may already be tapped; costs that untap permanents; votes for creatures; "your second
//! spell each turn"; clashing; skipping an untap step in Two-Headed Giant; "target creature
//! blocks this creature this turn if able"; and abilities activated while another of the
//! source's abilities is on the stack.

use crate::r_p062_common::*;
use crate::r_s01_common::{attack_with, supported, triggers_on_stack};
use crate::r_s02_common::create_token;
use crate::r_s05_common::enter;
use crate::r_s06_common::{activate_containing, has_kw};
use crate::r_s21_common::legal_blocks;
use crate::r_s29_common::{cast_and_resolve, put_counters};
use mtg_engine::ability::{Duration, Effect, Filter, Modification, Sel};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

const STUN: &str = "stun";

/// The enters ability of the real creature `name` (entering under P0's control) targets
/// P1's tapped Bears: they just get a stun counter; P1's next untap step removes it
/// instead of untapping them.
fn stuns_tapped_bears(t: &mut TestGame, name: &str) {
    supported(name);
    let bears = tapped_creature(t, P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    enter(t, P0, name);
    t.resolve_all();
    assert!(is_tapped(t, bears));
    assert_eq!(t.counters(bears, STUN), 1);
    through_untap_step(t, P1);
    assert!(is_tapped(t, bears));
    assert_eq!(t.counters(bears, STUN), 0);
    untaps_next(t, bears, P1);
}

#[test]
fn protocol_knights_ability_can_target_a_tapped_creature() {
    cr!("122.1d", "701.26a");
    ruling!(
        "Protocol Knight",
        "Protocol Knight’s ability can target a creature that’s already tapped."
    );
    // P0 controls another Knight.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Protocol Knight");
    stuns_tapped_bears(&mut t, "Protocol Knight");
}

#[test]
fn referee_squads_ability_can_target_a_tapped_creature() {
    cr!("122.1d", "701.26a");
    ruling!(
        "Referee Squad",
        "Referee Squad's enters-the-battlefield ability can target a creature that's already tapped."
    );
    let mut t = TestGame::new(2);
    stuns_tapped_bears(&mut t, "Referee Squad");
}

#[test]
fn waylaying_pirates_on_a_tapped_permanent_just_puts_a_stun_counter() {
    cr!("122.1d", "701.26a", "603.4");
    ruling!(
        "Waylaying Pirates",
        "The enters-the-battlefield ability can target an artifact or creature that's already tapped. If so, you'll just put a stun counter on that artifact or creature."
    );
    // P0 controls an artifact.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ornithopter");
    stuns_tapped_bears(&mut t, "Waylaying Pirates");
}

#[test]
fn out_cold_isnt_countered_by_ward() {
    cr!("702.21a", "101.2");
    ruling!(
        "Out Cold",
        "If you target a creature with ward, you may still pay the ward cost, but Out Cold won't be countered even if you don't."
    );
    supported("Out Cold");
    // Tomakul Honor Guard: "Ward {2}". P0 has no mana left to pay it.
    let mut t = TestGame::new(2);
    let guard = t.battlefield(P1, "Tomakul Honor Guard");
    t.answer_yes(P0, false);
    let spell = crate::r_s25_common::cast_new(&mut t, P0, "Out Cold", &[obj(guard)]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Ward") + triggers_on_stack(&t, "counter"), 1);
    t.resolve_all();
    assert!(!t.g.is_live(spell));
    assert!(t.in_graveyard(P0, "Out Cold"));
    assert!(is_tapped(&t, guard));
    assert_eq!(t.counters(guard, STUN), 1);
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Clue").len(), 1);
}

/// P1 casts Involuntary Cooldown on P0's `targets` (P0's turn resumes afterwards): they're
/// tapped with two stun counters each.
fn cooldown(t: &mut TestGame, targets: &[ObjectId]) {
    supported("Involuntary Cooldown");
    t.set_step(P1, Step::PrecombatMain);
    crate::r_s25_common::lands_for_cost(t, P1, "Involuntary Cooldown");
    let card = t.hand(P1, "Involuntary Cooldown");
    let es: Vec<Entity> = targets.iter().map(|o| obj(*o)).collect();
    t.answer_targets(P1, &es);
    t.cast_with(P1, card, &[]).expect("cast");
    t.resolve_all();
    t.set_step(P0, Step::PrecombatMain);
    for o in targets {
        assert!(is_tapped(t, *o));
        assert_eq!(t.counters(*o, STUN), 2);
    }
}

#[test]
fn a_cost_untapping_two_permanents_cant_untap_one_stunned_permanent_twice() {
    cr!("122.1d", "118.3", "601.2h");
    ruling!(
        "Involuntary Cooldown",
        "On the other hand, if untapping multiple permanents is part of a cost (such as that of Halo Fountain's last two abilities), you can't \"untap\" the same permanent more than once to pay that cost."
    );
    supported("Halo Fountain");
    // Halo Fountain: "{W}{W}, {T}, Untap two tapped creatures you control: Draw a card."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cooldown(&mut t, &[bears]);
    let fountain = t.battlefield(P0, "Halo Fountain");
    t.lands(P0, "Plains", 2);
    assert!(activate_containing(&mut t, P0, fountain, "Draw a card").is_err());
    assert_eq!(t.counters(bears, STUN), 2);
    // With a second tapped creature, the cost can be paid.
    let giant = tapped_creature(&mut t, P0, "Hill Giant");
    t.answer_choose(P0, &[obj(bears), obj(giant)]);
    activate_containing(&mut t, P0, fountain, "Draw a card").expect("cost paid");
    assert_eq!(t.counters(bears, STUN), 1);
    assert!(is_tapped(&t, bears));
    assert!(!is_tapped(&t, giant));
}

#[test]
fn stun_counters_arent_abilities_and_losing_abilities_doesnt_matter() {
    cr!("122.1d", "122.1b", "613.1f");
    ruling!(
        "Involuntary Cooldown",
        "Stun counters are not keyword counters, and they don't cause the permanents they're on to gain any abilities. They won't be affected by effects that cause permanents to lose all abilities."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cooldown(&mut t, &[bears]);
    assert!(t.obj_now(bears).chars.abilities.is_empty());
    // The Bears lose all abilities: the stun counters still keep them tapped.
    crate::r_s05_common::run_from(
        &mut t,
        P1,
        None,
        Effect::Modify {
            what: Sel::All(Filter::Objects(vec![bears])),
            mods: vec![Modification::RemoveAllAbilities],
            duration: Duration::Permanent,
        },
        &[],
    );
    through_untap_step(&mut t, P0);
    assert!(is_tapped(&t, bears));
    assert_eq!(t.counters(bears, STUN), 1);
}

#[test]
fn a_permanent_with_no_stun_counters_untaps_as_normal() {
    cr!("122.1d", "502.3");
    ruling!(
        "Involuntary Cooldown",
        "Stun counters exist independently of the effects that created them. If a permanent has no stun counters on it, it will untap as normal."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    cooldown(&mut t, &[bears, giant]);
    // The Giant's stun counters are removed (by some effect).
    let g = t.g.current(giant);
    t.g.remove_counters(Entity::Object(g), STUN, 2);
    through_untap_step(&mut t, P0);
    assert!(!is_tapped(&t, giant));
    assert!(is_tapped(&t, bears));
    assert_eq!(t.counters(bears, STUN), 1);
}

/// Votes of the players for the creature `id` (P0 first).
fn both_vote_for(t: &mut TestGame, id: ObjectId) {
    t.answer_choose(P0, &[obj(id)]);
    t.answer_choose(P1, &[obj(id)]);
}

#[test]
fn trap_the_trespassers_votes_can_be_for_a_hexproof_creature() {
    cr!("701.38a", "702.11b", "115.1");
    ruling!(
        "Trap the Trespassers",
        "None of the creatures are targeted. Players may vote for a creature with hexproof, protection from blue, or similar."
    );
    supported("Trap the Trespassers");
    // Gladecover Scout: "Hexproof".
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    t.battlefield(P1, "Grizzly Bears");
    both_vote_for(&mut t, scout);
    cast_and_resolve(&mut t, P0, "Trap the Trespassers", &[]);
    assert_eq!(t.counters(scout, STUN), 2);
    assert!(is_tapped(&t, scout));
}

#[test]
fn trap_the_trespassers_votes_can_be_for_a_tapped_creature() {
    cr!("701.38a", "122.1d");
    ruling!(
        "Trap the Trespassers",
        "Players may vote for creatures that are already tapped."
    );
    supported("Trap the Trespassers");
    let mut t = TestGame::new(2);
    let bears = tapped_creature(&mut t, P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    both_vote_for(&mut t, bears);
    cast_and_resolve(&mut t, P0, "Trap the Trespassers", &[]);
    assert_eq!(t.counters(bears, STUN), 2);
    assert!(is_tapped(&t, bears));
}

#[test]
fn shackle_slinger_counts_spells_cast_before_it_entered() {
    cr!("603.2", "601.2i");
    ruling!(
        "Shackle Slinger",
        "Spells that were cast before Shackle Slinger entered the battlefield count. If Shackle Slinger was the first spell you cast this turn, the next spell you cast this turn is your second spell."
    );
    supported("Shackle Slinger");
    // Shackle Slinger is P0's first spell; Grizzly Bears is the second: P1's untapped
    // Hill Giant is tapped.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    cast_and_resolve(&mut t, P0, "Shackle Slinger", &[]);
    t.answer_targets(P0, &[obj(giant)]);
    crate::r_s25_common::cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "second spell"), 1);
    t.resolve_all();
    assert!(is_tapped(&t, giant));
    assert_eq!(t.counters(giant, STUN), 0);
    // A spell cast before it entered counts: the first spell, then Shackle Slinger put
    // onto the battlefield, then the second spell; the tapped Giant gets a stun counter.
    let mut t = TestGame::new(2);
    let giant = tapped_creature(&mut t, P1, "Hill Giant");
    cast_and_resolve(&mut t, P0, "Grizzly Bears", &[]);
    t.battlefield(P0, "Shackle Slinger");
    t.answer_targets(P0, &[obj(giant)]);
    cast_and_resolve(&mut t, P0, "Grizzly Bears", &[]);
    assert_eq!(t.counters(giant, STUN), 1);
}

#[test]
fn entangling_trap_triggers_on_a_clash_an_opponent_started_and_you_can_win_it() {
    cr!("701.30a", "701.30b", "603.2");
    ruling!(
        "Entangling Trap",
        "If you clash because of a spell or ability an opponent controls, the ability will still trigger. Likewise, you can still win the clash even if you weren’t the player to initiate it."
    );
    supported("Entangling Trap");
    supported("Broken Ambitions");
    // P0 casts Grizzly Bears; P1 casts Broken Ambitions (X = 0) on it and clashes with
    // P0. P0 reveals Craw Wurm (mana value 6), P1 a mana value 0 card: P0 wins.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Entangling Trap");
    let giant = t.battlefield(P1, "Hill Giant");
    t.library_top(P0, "Craw Wurm");
    let bears_spell = crate::r_s25_common::cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.lands(P1, "Island", 1);
    let ba = t.hand(P1, "Broken Ambitions");
    t.cast(P1, ba).x(0).target(bears_spell).go();
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "clash"), 1);
    t.resolve_all();
    assert!(is_tapped(&t, giant));
    // P0 won: the Giant doesn't untap during P1's next untap step.
    misses_one_untap(&mut t, giant, P1);
}

#[test]
fn in_two_headed_giant_skipping_your_next_untap_step_skips_the_teams() {
    cr!("805.8", "614.10");
    ruling!(
        "Dovin, Architect of Law",
        "In a Two-Headed Giant game, if a player is instructed to skip their next untap step, the entire team skips that step."
    );
    supported("Dovin, Architect of Law");
    let mut t = crate::r_s33_common::two_headed_giant();
    let dovin = t.battlefield(P0, "Dovin, Architect of Law");
    let n = t.counters(dovin, "loyalty");
    put_counters(&mut t, dovin, "loyalty", 9 - n);
    let theirs = t.battlefield(P2, "Grizzly Bears");
    let teammates = tapped_creature(&mut t, P3, "Hill Giant");
    t.answer_targets(P0, &[Entity::Player(P2)]);
    activate_containing(&mut t, P0, dovin, "skips").expect("activate -9");
    t.resolve_all();
    assert!(is_tapped(&t, theirs));
    // The P2/P3 team's next turn: neither player's permanents untap.
    let team_upkeep = |t: &mut TestGame| {
        let turn = t.g.turn.number;
        assert!(t.g.run_until(10_000, |g| g.turn.number != turn
            && g.is_active_player(P2)
            && g.turn.step == Step::Upkeep
            && g.turn.stage == Stage::Priority));
    };
    team_upkeep(&mut t);
    assert!(is_tapped(&t, theirs));
    assert!(is_tapped(&t, teammates));
    // The following one: they untap.
    team_upkeep(&mut t);
    assert!(!is_tapped(&t, theirs));
    assert!(!is_tapped(&t, teammates));
}

/// P0's Maraleaf Rider, with a Food to sacrifice.
fn rider(t: &mut TestGame) -> ObjectId {
    supported("Maraleaf Rider");
    let rider = t.battlefield(P0, "Maraleaf Rider");
    create_token(t, P0, "Food");
    rider
}

/// P0 activates the Rider's ability targeting `target` (sacrificing a Food).
fn must_block_rider(t: &mut TestGame, rider: ObjectId, target: ObjectId) {
    let food = crate::r_s01_common::with_subtype(t, P0, "Food")[0];
    t.answer_targets(P0, &[obj(target)]);
    t.answer_choose(P0, &[obj(food)]);
    activate_containing(t, P0, rider, "Sacrifice a Food").expect("activate");
    t.resolve_all();
}

#[test]
fn maraleaf_riders_target_that_cant_block_it_is_free_to_block_others() {
    cr!("509.1c");
    ruling!(
        "Maraleaf Rider",
        "If the target creature can block other creatures but can't block Maraleaf Rider, it's free to block any other creature or to not block at all."
    );
    // Cloud Sprite: "Flying. This creature can block only creatures with flying." P0
    // attacks with the Rider and Wind Drake (flying).
    supported("Cloud Sprite");
    let mut t = TestGame::new(2);
    let r = rider(&mut t);
    let drake = t.battlefield(P0, "Wind Drake");
    let sprite = t.battlefield(P1, "Cloud Sprite");
    must_block_rider(&mut t, r, sprite);
    attack_with(
        &mut t,
        &[(r, Entity::Player(P1)), (drake, Entity::Player(P1))],
    );
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(sprite, drake)]));
    assert!(!legal_blocks(&mut t, P1, &[(sprite, r)]));
}

#[test]
fn maraleaf_riders_target_that_cant_block_or_must_pay_doesnt_block() {
    cr!("509.1c", "509.1d");
    ruling!(
        "Maraleaf Rider",
        "If the target creature can't block for any reason (such as being tapped), then it doesn't block. If there's a cost associated with having it block, its controller isn't forced to pay that cost, so it doesn't have to block in that case either."
    );
    // Untapped, with no cost: it must block the Rider.
    let mut t = TestGame::new(2);
    let r = rider(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    must_block_rider(&mut t, r, bears);
    attack_with(&mut t, &[(r, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(bears, r)]));
    // Tapped: it doesn't block.
    let mut t = TestGame::new(2);
    let r = rider(&mut t);
    let bears = t.battlefield(P1, "Grizzly Bears");
    must_block_rider(&mut t, r, bears);
    tap(&mut t, bears);
    attack_with(&mut t, &[(r, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[]));
    // A cost to block (Archangel of Tithes attacking: "creatures can't block unless their
    // controller pays {1} for each of those creatures"): it doesn't have to block.
    supported("Archangel of Tithes");
    let mut t = TestGame::new(2);
    let r = rider(&mut t);
    let angel = t.battlefield(P0, "Archangel of Tithes");
    let bears = t.battlefield(P1, "Grizzly Bears");
    must_block_rider(&mut t, r, bears);
    attack_with(
        &mut t,
        &[(r, Entity::Player(P1)), (angel, Entity::Player(P1))],
    );
    assert!(legal_blocks(&mut t, P1, &[]));
}

/// P0's Wicked Wolf ("Sacrifice a Food: Put a +1/+1 counter on this creature. It gains
/// indestructible until end of turn. Tap it.") activated, sacrificing a new Food.
fn feed_wolf(t: &mut TestGame, wolf: ObjectId) {
    let food = create_token(t, P0, "Food");
    t.answer_choose(P0, &[obj(food)]);
    activate_containing(t, P0, wolf, "Sacrifice a Food").expect("activate");
    t.clear_answers();
}

#[test]
fn wicked_wolfs_last_ability_can_be_activated_while_tapped() {
    cr!("602.2");
    ruling!(
        "Wicked Wolf",
        "You can activate Wicked Wolf's last ability even if it's already tapped."
    );
    supported("Wicked Wolf");
    let mut t = TestGame::new(2);
    let wolf = tapped_creature(&mut t, P0, "Wicked Wolf");
    feed_wolf(&mut t, wolf);
    t.resolve_all();
    assert_eq!(t.counters(wolf, "+1/+1"), 1);
    assert!(has_kw(&t, wolf, KeywordKind::Indestructible));
    assert!(is_tapped(&t, wolf));
}

#[test]
fn wicked_wolfs_last_ability_can_be_activated_repeatedly_before_it_fights() {
    cr!("602.2", "701.14a", "117.3c");
    ruling!(
        "Wicked Wolf",
        "You can activate Wicked Wolf's last ability any number of times while its first ability is on the stack."
    );
    supported("Wicked Wolf");
    // The Wolf (3/3) enters, its fight ability targeting Craw Wurm (6/4); in response it's
    // fed twice: a 5/5 indestructible Wolf kills the Wurm and survives.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm)]);
    let wolf = enter(&mut t, P0, "Wicked Wolf");
    assert_eq!(triggers_on_stack(&t, "fights"), 1);
    feed_wolf(&mut t, wolf);
    feed_wolf(&mut t, wolf);
    t.resolve_all();
    assert_eq!(t.counters(wolf, "+1/+1"), 2);
    assert!(t.on_battlefield(wolf));
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}

#[test]
fn thalias_geistcaller_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "702.34a", "701.6a");
    ruling!(
        "Thalia's Geistcaller",
        "The triggered ability of Thalia's Geistcaller will resolve before the spell that caused it to trigger but after targets have been chosen for that spell. It resolves even if that spell is countered."
    );
    supported("Thalia's Geistcaller");
    supported("Think Twice");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thalia's Geistcaller");
    let tt = t.graveyard(P0, "Think Twice");
    t.lands(P0, "Island", 3);
    t.cast(P0, tt)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    t.settle();
    // The trigger is above the spell.
    let items = crate::r_s04_common::stack_items(&t);
    assert_eq!(items.len(), 2);
    assert_eq!(items[0], "Think Twice");
    assert!(items[1].contains("Spirit"), "{items:?}");
    // P1 counters Think Twice; the trigger still resolves.
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    let spell = t.g.stack[0];
    t.cast_with(P1, cs, &[obj(spell)]).expect("cast Counterspell");
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(t.in_exile("Think Twice"));
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Spirit").len(), 1);
}
