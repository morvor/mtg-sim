//! Rulings batch P034 — triggered abilities of "counters matter" cards: cast triggers
//! that resolve before (and without) their spell, life-gain events, intervening "if"
//! clauses (CR 603.4), simultaneous entries (CR 603.6a), and each player's steps.

use crate::r_p034_common::*;
use crate::r_p038_common::{add, kill, tap};
use crate::r_s01_common::{
    attack_with, block_and_finish, supported, tokens, triggers_on_stack, watch,
};
use mtg_engine::decision::Decision;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn plus1(t: &TestGame, id: ObjectId) -> u32 {
    t.counters(id, counters::PLUS1)
}

/// `by` casts Cancel at `spell` (with exactly {1}{U}{U}) and only Cancel resolves.
fn counter_spell(t: &mut TestGame, by: PlayerId, spell: ObjectId) {
    let c = cast_card(t, by, "Cancel", &[obj(spell)]);
    assert_eq!(t.g.stack.last(), Some(&c));
    t.resolve();
    assert!(!t.g.stack.contains(&spell), "the spell was countered");
}

#[test]
fn cast_triggers_resolve_first_even_if_the_spell_is_countered() {
    cr!("603.3", "601.2i");
    ruling!(
        "Firemind's Research",
        "The triggered ability of Firemind’s Research resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Herald of Ilharg",
        "Herald of Ilharg's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered."
    );
    ruling!(
        "Curse of Vengeance",
        "Curse of Vengeance's first triggered ability resolves before the spell that caused it to trigger."
    );
    ruling!(
        "Ramos, Dragon Engine",
        "In that case, use its last known information to determine what colors it was."
    );
    supported("Firemind's Research");
    supported("Curse of Vengeance");
    // Firemind's Research: a charge counter for a countered Opt.
    let mut t = TestGame::new(2);
    let fr = t.battlefield(P0, "Firemind's Research");
    let opt = cast_card(&mut t, P0, "Opt", &[]);
    assert!(top_is_trigger(&t));
    counter_spell(&mut t, P1, opt);
    t.resolve_all();
    assert_eq!(t.counters(fr, counters::CHARGE), 1);

    // Herald of Ilharg: two counters and two damage for a countered Craw Wurm (its mana
    // value as it last existed on the stack).
    let mut t = TestGame::new(2);
    let herald = t.battlefield(P0, "Herald of Ilharg");
    let wurm = cast_card(&mut t, P0, "Craw Wurm", &[]);
    assert!(top_is_trigger(&t));
    counter_spell(&mut t, P1, wurm);
    t.resolve_all();
    assert_eq!(plus1(&t, herald), 2);
    assert_eq!(t.life(P1), 18);

    // Curse of Vengeance (on P1, controlled by P0): P1's countered spell still adds a
    // spite counter.
    let mut t = TestGame::new(2);
    let curse = t.battlefield(P0, "Curse of Vengeance");
    t.g.attach(curse, Entity::Player(P1));
    t.g.recompute();
    let opt = cast_card(&mut t, P1, "Opt", &[]);
    assert!(top_is_trigger(&t));
    counter_spell(&mut t, P0, opt);
    t.resolve_all();
    assert_eq!(t.counters(curse, "spite"), 1);

    // Ramos: a countered Lightning Helix still gives two counters (it was red and white).
    let mut t = TestGame::new(2);
    let ramos = t.battlefield(P0, "Ramos, Dragon Engine");
    let helix = cast_card(&mut t, P0, "Lightning Helix", &[Entity::Player(P1)]);
    assert!(top_is_trigger(&t));
    counter_spell(&mut t, P1, helix);
    t.resolve_all();
    assert_eq!(plus1(&t, ramos), 2);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn ramos_counts_the_spells_colors() {
    cr!("105.2", "202.2");
    ruling!(
        "Ramos, Dragon Engine",
        "If you cast a colorless spell, Ramos's triggered ability triggers, but it won't get any +1/+1 counters."
    );
    ruling!(
        "Ramos, Dragon Engine",
        "Ramos's triggered ability counts the number of colors a spell has (from zero to five)"
    );
    let mut t = TestGame::new(2);
    let ramos = t.battlefield(P0, "Ramos, Dragon Engine");
    // A colorless spell paid for with red and green mana: it triggers, no counters.
    pool(&mut t, P0, ManaType::R, 1);
    pool(&mut t, P0, ManaType::G, 1);
    let ms = t.hand(P0, "Mind Stone");
    t.cast(P0, ms).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "for each of that spell's colors"), 1);
    t.resolve_all();
    assert_eq!(plus1(&t, ramos), 0);
    // Ball Lightning ({R}{R}{R}): one color.
    cast_card(&mut t, P0, "Ball Lightning", &[]);
    t.resolve_all();
    assert_eq!(plus1(&t, ramos), 1);
    // Lightning Helix: two.
    cast_card(&mut t, P0, "Lightning Helix", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(plus1(&t, ramos), 3);
}

/// P0 attacks with `attackers` (P1 blocking as given), and combat ends.
fn fight(t: &mut TestGame, attackers: &[ObjectId], blocks: &[(ObjectId, ObjectId)]) {
    let a: Vec<_> = attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    attack_with(t, &a);
    block_and_finish(t, P1, blocks);
}

#[test]
fn each_lifelink_creatures_combat_damage_is_one_life_gain_event() {
    cr!("702.15b", "119.9");
    ruling!(
        "Karlov of the Ghost Council",
        "if a single creature with lifelink deals combat damage to multiple players or permanents at the same time"
    );
    ruling!(
        "Qala, Ajani's Pridemate",
        "if a single creature you control with lifelink deals combat damage to multiple creatures, players, planeswalkers"
    );
    ruling!(
        "Karlov of the Ghost Council",
        "The ability triggers just once for each life-gaining event, whether it's 1 life from an attacking creature with lifelink or 4 life from Faith's Fetters."
    );
    supported("Karlov of the Ghost Council");
    supported("Qala, Ajani's Pridemate");
    supported("Lifelink");
    for (name, per) in [
        ("Karlov of the Ghost Council", 2),
        ("Qala, Ajani's Pridemate", 1),
    ] {
        // Two lifelinkers deal combat damage at once: two triggers.
        let mut t = TestGame::new(2);
        let k = t.battlefield(P0, name);
        let a = t.battlefield(P0, "Vault Skirge");
        let b = t.battlefield(P0, "Vault Skirge");
        fight(&mut t, &[a, b], &[]);
        assert_eq!(t.life(P0), 22);
        assert_eq!(plus1(&t, k), 2 * per, "{name}");
        // One lifelinker blocked by two creatures: one trigger.
        let mut t = TestGame::new(2);
        let k = t.battlefield(P0, name);
        let giant = t.battlefield(P0, "Hill Giant");
        let ll = t.battlefield(P0, "Lifelink");
        t.g.attach(ll, obj(giant));
        t.g.recompute();
        let e1 = t.battlefield(P1, "Llanowar Elves");
        let e2 = t.battlefield(P1, "Llanowar Elves");
        fight(&mut t, &[giant], &[(e1, giant), (e2, giant)]);
        assert!(!t.on_battlefield(e1) && !t.on_battlefield(e2));
        assert_eq!(t.life(P0), 23);
        assert_eq!(plus1(&t, k), per, "{name}");
    }
    // Faith's Fetters: 4 life, one trigger.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Karlov of the Ghost Council");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_card(&mut t, P0, "Faith's Fetters", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(plus1(&t, k), 2);
}

#[test]
fn qala_dealt_lethal_damage_while_you_gain_life_dies() {
    cr!("704.3", "603.3", "510.2");
    ruling!(
        "Qala, Ajani's Pridemate",
        "it won't receive a counter from its second ability in time to save it"
    );
    let mut t = TestGame::new(2);
    let q = t.battlefield(P0, "Qala, Ajani's Pridemate");
    let skirge = t.battlefield(P0, "Vault Skirge");
    let giant = t.battlefield(P1, "Hill Giant");
    fight(&mut t, &[q, skirge], &[(giant, q)]);
    assert_eq!(t.life(P0), 21);
    assert!(t.in_graveyard(P0, "Qala, Ajani's Pridemate"));
}

#[test]
fn end_step_intervening_if_triggers_check_as_the_step_begins_and_on_resolution() {
    cr!("603.4");
    ruling!(
        "Sunstar Chaplain",
        "If you don’t control two or more tapped creatures when the ability resolves, the ability won’t do anything."
    );
    ruling!(
        "Fractal Tender",
        "It's not possible to put a counter on Fractal Tender during an end step in time to have the ability trigger."
    );
    supported("Sunstar Chaplain");
    supported("Fractal Tender");
    // Sunstar Chaplain: two tapped creatures, one untapped in response: nothing.
    for untap in [false, true] {
        let mut t = TestGame::new(2);
        let sc = add(&mut t, P0, "Sunstar Chaplain");
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Grizzly Bears");
        t.advance_to(P0, Step::PostcombatMain);
        tap(&mut t, a);
        tap(&mut t, b);
        t.answer_targets(P0, &[obj(sc)]);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "two or more tapped creatures"), 1);
        if untap {
            let b = t.g.current(b);
            t.g.untap(b);
        }
        t.resolve_all();
        assert_eq!(plus1(&t, sc), if untap { 0 } else { 1 }, "untap: {untap}");
    }
    // One tapped creature: it doesn't trigger, and tapping a second one during the end
    // step is too late.
    let mut t = TestGame::new(2);
    let sc = add(&mut t, P0, "Sunstar Chaplain");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.advance_to(P0, Step::PostcombatMain);
    tap(&mut t, a);
    t.answer_targets(P0, &[obj(sc)]);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "two or more tapped creatures"), 0);
    tap(&mut t, b);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "two or more tapped creatures"), 0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(plus1(&t, sc), 0);

    // Fractal Tender: a counter put on it earlier in the turn: a Fractal. A counter put
    // on it only during the end step: none.
    let mut t = TestGame::new(2);
    let ft = add(&mut t, P0, "Fractal Tender");
    put(&mut t, ft, counters::PLUS1, 1);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    let fractals = crate::r_p038_common::tokens_of(&t, P0, "Fractal");
    assert_eq!(fractals.len(), 1);
    assert_eq!(plus1(&t, fractals[0]), 3);
    let mut t = TestGame::new(2);
    let ft = add(&mut t, P0, "Fractal Tender");
    t.advance_to(P0, Step::End);
    put(&mut t, ft, counters::PLUS1, 1);
    t.resolve_all();
    t.advance_to(P1, Step::Upkeep);
    assert!(crate::r_p038_common::tokens_of(&t, P0, "Fractal").is_empty());
}

#[test]
fn delta_bloodflies_checks_for_a_creature_with_a_counter_twice() {
    cr!("603.4");
    ruling!(
        "Delta Bloodflies",
        "If you don’t control a creature with a counter on it at that time, the ability won’t resolve"
    );
    supported("Delta Bloodflies");
    // No counters: no trigger.
    let mut t = TestGame::new(2);
    let db = t.battlefield(P0, "Delta Bloodflies");
    attack_with(&mut t, &[(db, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "each opponent loses 1 life"), 0);
    // A counter on Grizzly Bears, removed in response: no life loss.
    for remove in [false, true] {
        let mut t = TestGame::new(2);
        let db = t.battlefield(P0, "Delta Bloodflies");
        let bears = t.battlefield(P0, "Grizzly Bears");
        put(&mut t, bears, "stun", 1);
        attack_with(&mut t, &[(db, Entity::Player(P1))]);
        assert_eq!(triggers_on_stack(&t, "each opponent loses 1 life"), 1);
        if remove {
            t.g.remove_counters(obj(bears), "stun", 1);
        }
        t.resolve_all();
        assert_eq!(t.life(P1), if remove { 20 } else { 19 });
    }
}

#[test]
fn charging_cinderhorn_triggers_at_each_end_step_if_no_creatures_attacked() {
    cr!("603.4", "603.2");
    ruling!(
        "Charging Cinderhorn",
        "Charging Cinderhorn’s triggered ability doesn’t trigger at all if a creature attacked this turn."
    );
    ruling!(
        "Charging Cinderhorn",
        "including yours, and even if it wasn’t on the battlefield during combat"
    );
    // A creature attacked this turn (P1's Bears, before the Cinderhorn was there): no
    // trigger at P1's end step.
    let mut t = TestGame::new(2);
    t.advance_to(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    t.advance_to(P1, Step::PostcombatMain);
    let ox = add(&mut t, P0, "Charging Cinderhorn");
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "fury counter"), 0);
    assert_eq!(t.counters(ox, "fury"), 0);
    // No attack in P0's own turn, the Cinderhorn put onto the battlefield after combat:
    // it gets a fury counter and deals 1 damage to P0.
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PostcombatMain);
    let ox = add(&mut t, P0, "Charging Cinderhorn");
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(ox, "fury"), 1);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn charging_cinderhorn_still_gets_a_counter_if_the_active_player_lost() {
    cr!("800.4j");
    ruling!(
        "Charging Cinderhorn",
        "If a player loses the game during that player’s own turn, the turn continues to completion."
    );
    let mut t = TestGame::new(3);
    t.advance_to(P1, Step::PostcombatMain);
    let ox = add(&mut t, P0, "Charging Cinderhorn");
    t.g.lose_life(P1, 20);
    t.g.flush_events();
    t.settle();
    assert!(t.has_lost(P1));
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == Step::End
            && g.turn.stage == mtg_engine::turn::Stage::Priority
            && !g.stack.is_empty()
    });
    assert!(ok, "P1's end step happened with the trigger");
    assert_eq!(t.g.turn.active, P1);
    t.resolve_all();
    assert_eq!(t.counters(ox, "fury"), 1);
    assert_eq!((t.life(P0), t.life(P2)), (20, 20));
}

#[test]
fn traveling_plague_gets_a_counter_at_each_players_upkeep() {
    cr!("503.1a");
    ruling!(
        "Traveling Plague",
        "A counter is placed on every player's upkeep, not just the controller's upkeep."
    );
    supported("Traveling Plague");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let tp = t.battlefield(P0, "Traveling Plague");
    t.g.attach(tp, obj(giant));
    t.g.recompute();
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(tp, "plague"), 1);
    assert_eq!(t.pt(giant), (2, 2));
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(tp, "plague"), 2);
}

#[test]
fn permanents_entering_together_trigger_each_others_abilities() {
    cr!("603.6a", "603.2");
    ruling!(
        "Kyler, Sigardian Emissary",
        "If Kyler enters the battlefield at the same time as other Humans you control, Kyler's first ability will trigger once for each"
    );
    ruling!(
        "Surge Conductor",
        "Surge Conductor’s ability will trigger for each of those nontoken artifacts."
    );
    supported("Kyler, Sigardian Emissary");
    supported("Surge Conductor");
    supported("Eerie Interlude");
    // Kyler and two Humans flickered by Eerie Interlude return together: two counters.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kyler, Sigardian Emissary");
    let a = t.battlefield(P0, "Elite Vanguard");
    let b = t.battlefield(P0, "Elite Vanguard");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_card(
        &mut t,
        P0,
        "Eerie Interlude",
        &[obj(k), obj(a), obj(b), obj(bears)],
    );
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    let k = t.named_on_battlefield("Kyler, Sigardian Emissary")[0];
    assert_eq!(plus1(&t, k), 2, "the two Humans, not the Bears");
    for v in t.named_on_battlefield("Elite Vanguard") {
        assert_eq!(t.pt(v), (4, 3));
    }
    // Surge Conductor and Mind Stone flickered together: Mind Stone's entry proliferates.
    let mut t = TestGame::new(2);
    let sc = t.battlefield(P0, "Surge Conductor");
    let ms = t.battlefield(P0, "Mind Stone");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 1);
    cast_card(&mut t, P0, "Ghostly Flicker", &[obj(sc), obj(ms)]);
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(plus1(&t, bears), 2);
}

#[test]
fn kilos_trigger_resolves_before_the_station_ability_it_was_tapped_for() {
    cr!("603.3", "702.184a");
    ruling!(
        "Kilo, Apogee Mind",
        "its last ability will resolve before the station ability resolves"
    );
    supported("Kilo, Apogee Mind");
    supported("Wedgelight Rammer");
    // The Spacecraft has no counters as Kilo's proliferate resolves, then gets three.
    let mut t = TestGame::new(2);
    let kilo = t.battlefield(P0, "Kilo, Apogee Mind");
    let ship = t.battlefield(P0, "Wedgelight Rammer");
    t.answer_choose(P0, &[obj(kilo)]);
    t.activate(P0, ship, 0, &[]).unwrap();
    t.settle();
    assert!(t.obj_now(kilo).tapped);
    assert!(top_is_trigger(&t));
    t.answer_choose(P0, &[obj(ship)]);
    t.resolve_all();
    assert_eq!(t.counters(ship, counters::CHARGE), 3, "not 3 + 1");
}

#[test]
fn midnight_oil_draws_an_extra_card_whatever_its_hour_counters() {
    cr!("603.2");
    ruling!(
        "Midnight Oil",
        "If Midnight Oil has no hour counters on it, its first triggered ability will continue to trigger"
    );
    ruling!(
        "Midnight Oil",
        "If Midnight Oil has only one hour counter on it, its first triggered ability will remove that one counter."
    );
    supported("Midnight Oil");
    for hours in [0, 1] {
        let mut t = TestGame::new(2);
        let mo = add(&mut t, P0, "Midnight Oil");
        put(&mut t, mo, "hour", hours);
        t.advance_to(P0, Step::Draw);
        let hand = t.hand_size(P0);
        t.settle();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "the additional card ({hours})");
        assert_eq!(t.counters(mo, "hour"), 0);
    }
}

#[test]
fn black_markets_mana_isnt_optional() {
    cr!("603.5", "106.4");
    ruling!("Black Market", "Getting the mana is not optional.");
    supported("Black Market");
    let mut t = TestGame::new(2);
    let bm = add(&mut t, P0, "Black Market");
    put(&mut t, bm, counters::CHARGE, 2);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    t.settle();
    let from = t.asked().len();
    t.resolve_all();
    assert!(!crate::r_s01_common::asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
    assert_eq!(t.g.player(P0).mana_pool.total(), 2);
}

#[test]
fn decimator_beetle_targets_and_generous_patron() {
    cr!("608.2b", "122.1");
    ruling!(
        "Decimator Beetle",
        "It’ll put a -1/-1 counter on the opponent’s creature even if there isn’t one to remove from your creature."
    );
    ruling!(
        "Decimator Beetle",
        "If one target of Decimator Beetle’s second ability becomes illegal, the other will still be affected."
    );
    ruling!(
        "Generous Patron",
        "Generous Patron's last ability triggers even if the counters you put on another player's creatures aren't +1/+1 counters."
    );
    supported("Decimator Beetle");
    supported("Generous Patron");
    // A creature of P0's without -1/-1 counters; a -1/-1 counter still goes on P1's Hill
    // Giant, and Generous Patron draws a card for it.
    let mut t = TestGame::new(2);
    let beetle = t.battlefield(P0, "Decimator Beetle");
    let patron = t.battlefield(P0, "Generous Patron");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(patron)]);
    t.answer_targets(P0, &[obj(giant)]);
    attack_with(&mut t, &[(beetle, Entity::Player(P1))]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.counters(giant, counters::MINUS1), 1);
    assert_eq!(t.hand_size(P0), hand + 1, "Generous Patron");
    // P0's target leaves in response: P1's Hill Giant still gets the counter; P1's Giant
    // leaves: P0's Bears still loses its counter.
    for gone in ["mine", "theirs"] {
        let mut t = TestGame::new(2);
        let beetle = t.battlefield(P0, "Decimator Beetle");
        let bears = t.battlefield(P0, "Grizzly Bears");
        put(&mut t, bears, counters::MINUS1, 1);
        let giant = t.battlefield(P1, "Hill Giant");
        t.answer_targets(P0, &[obj(bears)]);
        t.answer_targets(P0, &[obj(giant)]);
        attack_with(&mut t, &[(beetle, Entity::Player(P1))]);
        kill(&mut t, if gone == "mine" { bears } else { giant });
        t.resolve_all();
        if gone == "mine" {
            assert_eq!(t.counters(giant, counters::MINUS1), 1);
        } else {
            assert_eq!(t.counters(bears, counters::MINUS1), 0);
        }
    }
}

#[test]
fn a_crime_is_committed_by_the_initial_targets_only() {
    cr!("700.13", "115.7");
    ruling!(
        "Patrolling Peacemaker",
        "Only the initial targets chosen for that spell or ability are used to determine whether its controller committed a crime."
    );
    supported("Patrolling Peacemaker");
    supported("Redirect");
    // P1 targets its own Bears; P0 redirects the spell to the Peacemaker: P1 committed
    // no crime.
    let mut t = TestGame::new(2);
    let pp = t.battlefield(P0, "Patrolling Peacemaker");
    put(&mut t, pp, counters::PLUS1, 2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let gg = cast_card(&mut t, P1, "Giant Growth", &[obj(bears)]);
    assert_eq!(t.stack_len(), 1);
    cast_card(&mut t, P0, "Redirect", &[obj(gg)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(pp)]);
    t.resolve();
    t.resolve_all();
    assert_eq!(t.pt(pp), (5, 5), "the Giant Growth went to the Peacemaker");
    assert_eq!(plus1(&t, pp), 2, "no proliferate");
    // P1 targets the Peacemaker: a crime.
    let mut t = TestGame::new(2);
    let pp = t.battlefield(P0, "Patrolling Peacemaker");
    put(&mut t, pp, counters::PLUS1, 2);
    cast_card(&mut t, P1, "Giant Growth", &[obj(pp)]);
    t.answer_choose(P0, &[obj(pp)]);
    t.resolve_all();
    assert_eq!(plus1(&t, pp), 3);
}

#[test]
fn together_forever_cant_be_activated_before_the_counters_kill_the_creature() {
    cr!("704.3", "117.5");
    ruling!(
        "Together Forever",
        "Together Forever's ability can't be activated before state-based actions put that creature into its owner's graveyard."
    );
    supported("Together Forever");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Together Forever");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::Priority { .. }),
        |g| {
            g.battlefield
                .iter()
                .any(|o| g.obj(*o).counter(counters::MINUS1) > 0)
        },
    );
    t.answer_targets(P0, &[obj(elves)]);
    crate::r_p038_common::enter(&mut t, P0, "Decimator Beetle");
    assert!(top_is_trigger(&t));
    let ok =
        t.g.run_until(10_000, |g| g.turn.step != Step::PrecombatMain);
    assert!(ok);
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    let seen = seen.lock().unwrap().clone();
    assert!(!seen.is_empty());
    assert!(
        seen.iter().all(|x| !x),
        "no priority with the 0/0 Elves around"
    );
}

#[test]
fn sporogenesis_ignores_a_noncreature_with_fungus_counters() {
    cr!("700.4", "603.10a");
    ruling!(
        "Sporogenesis",
        "If a non-creature with fungus counters on it leaves the battlefield, there is no effect."
    );
    supported("Sporogenesis");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sporogenesis");
    let ms = t.battlefield(P0, "Mind Stone");
    put(&mut t, ms, "fungus", 2);
    kill(&mut t, ms);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    let bears = t.battlefield(P1, "Grizzly Bears");
    put(&mut t, bears, "fungus", 2);
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
}

#[test]
fn tesak_counts_the_attacking_creatures_as_its_ability_resolves() {
    cr!("608.2h", "506.4");
    ruling!(
        "Tesak, Judith's Hellhound",
        "Count the number of attacking creatures when Tesak, Judith's Hellhound's last ability resolves"
    );
    supported("Tesak, Judith's Hellhound");
    for killed in ["none", "bears", "tesak"] {
        let mut t = TestGame::new(2);
        let tesak = t.battlefield(P0, "Tesak, Judith's Hellhound");
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Grizzly Bears");
        attack_with(
            &mut t,
            &[
                (tesak, Entity::Player(P1)),
                (a, Entity::Player(P1)),
                (b, Entity::Player(P1)),
            ],
        );
        assert!(top_is_trigger(&t));
        match killed {
            "bears" => kill(&mut t, a),
            "tesak" => kill(&mut t, tesak),
            _ => {}
        }
        t.resolve();
        let want = if killed == "none" { 3 } else { 2 };
        assert_eq!(t.g.player(P0).mana_pool.total(), want, "{killed}");
    }
}
