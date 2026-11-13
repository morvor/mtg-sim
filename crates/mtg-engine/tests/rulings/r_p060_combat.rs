//! Rulings batch P060 — draws tied to damage and combat: "whenever this is dealt damage"
//! triggers once per damage event (CR 603.2c, 510.2), even for lethal damage; damage
//! counted after it's dealt (Grothama) or redirected (Diviner Spirit); prevention that
//! can't happen (Sokrates); attack and block restrictions and requirements (CR 508.1,
//! 509.1) for bribery counters, Rowan Kenrith and Nessian Boar.

use crate::r_p060_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, triggers_on_stack};
use crate::r_s02_common::can_attack;
use crate::r_s21_common::{blocks_now, legal_blocks};
use crate::r_s06_common::attach_new;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn flumph_triggers_once_per_damage_event_even_if_lethal() {
    cr!("603.2c", "510.2", "603.3", "603.3d");
    ruling!(
        "Flumph",
        "Flumph's ability only triggers once each time it is dealt damage, even if multiple sources are dealing damage to it at once."
    );
    ruling!("Flumph", "Flumph's ability will trigger even if it is dealt lethal damage.");
    ruling!(
        "Flumph",
        "Flumph's triggered ability is not optional. You must target an opponent if able."
    );
    supported("Flumph");
    // Brave the Sands lets Flumph (0/4) block both of P1's attacking Hill Giants: 6 damage
    // from two sources at once, lethal. One trigger.
    let mut t = TestGame::new(2);
    let flumph = t.battlefield(P0, "Flumph");
    t.battlefield(P0, "Brave the Sands");
    let g1 = t.battlefield(P1, "Hill Giant");
    let g2 = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(g1, pl(P0)), (g2, pl(P0))]);
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(flumph, g1), (flumph, g2)]),
    );
    // Scripted to decline a target: the ability isn't optional, so P1 is targeted anyway.
    t.answer_targets(P0, &[]);
    let from = t.asked().len();
    let before = libraries(&t);
    t.advance_to(P1, Step::CombatDamage);
    t.settle();
    assert!(t.in_graveyard(P0, "Flumph"), "lethal damage");
    assert_eq!(triggers_on_stack(&t, "each draw a card"), 1, "one trigger");
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![1, 1]);
    // It wasn't asked whether to use the ability.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::YesNo { .. })));
}

/// P0's Hill Giant gains Requiem Monolith's ability (P0 declines the 1 damage).
fn monolith_giant(t: &mut TestGame) -> ObjectId {
    supported("Requiem Monolith");
    let giant = t.battlefield(P0, "Hill Giant");
    let m = t.battlefield(P0, "Requiem Monolith");
    t.answer_yes(P0, false);
    t.activate(P0, m, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    giant
}

#[test]
fn requiem_monolith_triggers_once_for_simultaneous_damage_and_for_lethal_damage() {
    cr!("603.2c", "510.2", "603.3");
    ruling!(
        "Requiem Monolith",
        "If multiple sources deal damage to a creature that has been granted the listed ability by Requiem Monolith’s ability (probably because multiple creatures blocked that creature), the granted ability triggers only once. When it resolves, the creature’s controller draws cards and loses life equal to the total amount of damage dealt to that creature by those sources."
    );
    ruling!(
        "Requiem Monolith",
        "If lethal damage is dealt to a creature that has been granted the listed ability by Requiem Monolith’s ability, the granted ability will still trigger."
    );
    // Two Grizzly Bears block the Giant: 4 damage at once, lethal. One trigger: P0 draws
    // four and loses 4 life.
    let mut t = TestGame::new(2);
    let giant = monolith_giant(&mut t);
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(giant, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(b1, giant), (b2, giant)]),
    );
    let before = libraries(&t);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(triggers_on_stack(&t, "draw that many"), 1);
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![4, 0]);
    assert_eq!(t.life(P0), 16);
    // Lethal damage from one source (Lightning Bolt): it still triggers.
    let mut t = TestGame::new(2);
    let giant = monolith_giant(&mut t);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(giant).go();
    let before = libraries(&t);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(drawn_since(&t, &before), vec![3, 0]);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn grothama_counts_damage_from_any_source_not_only_fights() {
    cr!("603.10a", "608.2h");
    ruling!(
        "Grothama, All-Devouring",
        "Grothama's last ability triggers even if it dies without being fought by attacking creatures. For example, if you use Blaze to deal 10 damage to Grothama, you draw ten cards."
    );
    supported("Grothama, All-Devouring");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Grothama, All-Devouring");
    t.lands(P0, "Mountain", 11);
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(10).target(g).go();
    let before = libraries(&t);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grothama, All-Devouring"));
    assert_eq!(drawn_since(&t, &before), vec![10, 0]);
}

#[test]
fn diviner_spirit_damage_redirected_to_its_controller_draws_twice() {
    cr!("510.2", "603.2", "614.9");
    ruling!(
        "Diviner Spirit",
        "If Diviner Spirit deals combat damage to its controller (perhaps because damage was redirected), that player will draw twice that number of cards."
    );
    supported("Diviner Spirit");
    supported("Harm's Way");
    // P1's Harm's Way: the next 2 damage Diviner Spirit (2/4) would deal to P1 is dealt to
    // P0 instead. All its combat damage is dealt to P0, its controller: "you and that
    // player each draw that many cards" has P0 draw 4 (one instruction, interpretation
    // `player-named-twice-draws-twice`).
    let mut t = TestGame::new(2);
    let ds = t.battlefield(P0, "Diviner Spirit");
    attack_with(&mut t, &[(ds, pl(P1))]);
    t.lands(P1, "Plains", 1);
    let hw = t.hand(P1, "Harm's Way");
    t.answer_choose(P1, &[obj(ds)]);
    t.cast(P1, hw).target(pl(P0)).go();
    t.resolve();
    let before = libraries(&t);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (18, 20));
    assert_eq!(drawn_since(&t, &before), vec![4, 0]);
    // Its combat damage dealt to P1: each player draws two.
    let mut t = TestGame::new(2);
    let ds = t.battlefield(P0, "Diviner Spirit");
    attack_with(&mut t, &[(ds, pl(P1))]);
    let before = libraries(&t);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
    assert_eq!(drawn_since(&t, &before), vec![2, 2]);
}

#[test]
fn a_player_named_twice_draws_twice_as_one_instruction() {
    cr!("121.2", "121.2a", "614.9", "119.9", "119.10");
    ruling!(
        "Diviner Spirit",
        "If Diviner Spirit deals combat damage to its controller (perhaps because damage was redirected), that player will draw twice that number of cards."
    );
    ruling!(
        "Angel of Destiny",
        "If a creature you control somehow deals combat damage to you (most likely because its combat damage was redirected), you gain twice that much life as a single life-gaining event."
    );
    ruling!(
        "Alms Collector",
        "To determine whether a player is instructed to draw multiple once or instructed multiple times to draw one card, count how many times the word “draw” is used."
    );
    interp!("player-named-twice-draws-twice");
    supported("Alms Collector");
    supported("Angel of Destiny");
    supported("Leyline of Hope");
    // P1's Alms Collector: "If an opponent would draw two or more cards, instead you and
    // that player each draw a card." Diviner Spirit's 2 combat damage, redirected to P0
    // (P1's Harm's Way), has P0, both "you" and "that player", draw four cards as one
    // instruction (one word "draw"): it's replaced once, so P0 and P1 each draw one card
    // (two instructions to draw two would have them each draw two).
    let mut t = TestGame::new(2);
    let ds = t.battlefield(P0, "Diviner Spirit");
    t.battlefield(P1, "Alms Collector");
    attack_with(&mut t, &[(ds, pl(P1))]);
    t.lands(P1, "Plains", 1);
    let hw = t.hand(P1, "Harm's Way");
    t.answer_choose(P1, &[obj(ds)]);
    t.cast(P1, hw).target(pl(P0)).go();
    t.resolve();
    let before = libraries(&t);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    assert_eq!(drawn_since(&t, &before), vec![1, 1]);
    // Not redirected: P1 draws its two cards, and P0's are replaced (one each).
    let mut t = TestGame::new(2);
    let ds = t.battlefield(P0, "Diviner Spirit");
    t.battlefield(P1, "Alms Collector");
    attack_with(&mut t, &[(ds, pl(P1))]);
    let before = libraries(&t);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(drawn_since(&t, &before), vec![1, 3]);
    // Life, the same way: Angel of Destiny ("Whenever a creature you control deals combat
    // damage to a player, you and that player each gain that much life.") with P0's
    // Leyline of Hope ("If you would gain life, you gain that much life plus 1 instead."):
    // Grizzly Bears' 2 combat damage redirected to P0 has P0 gain 4 life as one event,
    // replaced once: 5 (two gains of 2 would be 3 each), and Ajani's Pridemate ("Whenever
    // you gain life, put a +1/+1 counter on this creature.") triggers once.
    supported("Ajani's Pridemate");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Angel of Destiny");
    t.battlefield(P0, "Leyline of Hope");
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.lands(P1, "Plains", 1);
    let hw = t.hand(P1, "Harm's Way");
    t.answer_choose(P1, &[obj(bears)]);
    t.cast(P1, hw).target(pl(P0)).go();
    t.resolve();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20 - 2 + 5, 20));
    assert_eq!(t.counters(pridemate, "+1/+1"), 1);
}

#[test]
fn a_player_named_twice_in_any_group_of_single_players_gets_one_instruction() {
    cr!("121.2a", "119.10");
    interp!("player-named-twice-draws-twice");
    // Not tied to one wording: "you and the controller of those creatures each ..." (Nelly
    // Borca's wording, compiled as a group of players) names P0 twice when those creatures
    // are P0's. P0 draws four cards as one instruction (P1's Alms Collector replaces it
    // once: each player draws one) and gains 4 life as one event (Leyline of Hope adds 1
    // once).
    let each = |verb: &str| {
        crate::r_s01_common::custom_card(
            "Shared Spoils",
            "Creature — Spirit",
            "{2}",
            Some((2, 2)),
            &format!(
                "Whenever one or more creatures you control deal combat damage to a player, \
                 you and the controller of those creatures each {verb}."
            ),
        )
    };
    // (instruction, P1's Alms Collector, P0's Leyline of Hope, cards drawn, P0's life)
    for (verb, alms, leyline, drawn, life) in [
        ("draw two cards", false, false, vec![4, 0], 20),
        ("draw two cards", true, false, vec![1, 1], 20),
        ("gain 2 life", false, false, vec![0, 0], 24),
        ("gain 2 life", false, true, vec![0, 0], 25),
    ] {
        let mut t = TestGame::new(2);
        let spirit = t.custom(P0, each(verb), mtg_engine::object::Zone::Battlefield);
        if alms {
            t.battlefield(P1, "Alms Collector");
        }
        if leyline {
            t.battlefield(P0, "Leyline of Hope");
        }
        attack_with(&mut t, &[(spirit, pl(P1))]);
        let before = libraries(&t);
        t.advance_to(P0, Step::EndOfCombat);
        t.resolve_all();
        assert_eq!(t.life(P1), 18, "{verb}");
        assert_eq!(drawn_since(&t, &before), drawn, "{verb} {alms}");
        assert_eq!(t.life(P0), life, "{verb} {leyline}");
    }
}

#[test]
fn sokrates_players_draw_half_even_if_the_damage_cant_be_prevented() {
    cr!("615.12", "615.1", "510.2");
    ruling!(
        "Sokrates, Athenian Teacher",
        "If damage dealt by a creature with the ability granted by Sokrates's last ability can't be prevented for some reason, the appropriate players will still draw half that many cards, rounded down."
    );
    supported("Sokrates, Athenian Teacher");
    supported("Leyline of Punishment");
    // Serra Angel (4 power) attacks P1: with the damage prevented, no damage and each
    // draws two; with Leyline of Punishment, P1 takes 4 and each still draws two.
    for unpreventable in [false, true] {
        let mut t = TestGame::new(2);
        let angel = t.battlefield(P0, "Serra Angel");
        let s = t.battlefield(P0, "Sokrates, Athenian Teacher");
        if unpreventable {
            t.battlefield(P1, "Leyline of Punishment");
        }
        t.activate(P0, s, 0, &[obj(angel)]).unwrap();
        t.resolve_all();
        let before = libraries(&t);
        attack_with(&mut t, &[(angel, pl(P1))]);
        t.advance_to(P0, Step::EndOfCombat);
        t.resolve_all();
        let loss = if unpreventable { 4 } else { 0 };
        assert_eq!(t.life(P1), 20 - loss, "unpreventable: {unpreventable}");
        assert_eq!(drawn_since(&t, &before), vec![2, 2], "{unpreventable}");
    }
}

#[test]
fn bribery_counters_stop_attacks_and_blocks_only_while_a_gwafa_is_there() {
    cr!("508.1c", "509.1b", "506.4", "122.1");
    ruling!(
        "Gwafa Hazid, Profiteer",
        "Creatures with bribery counters on them can't be declared as attackers or blockers. However, if a bribery counter is put on a creature that's already been declared as an attacker or blocker, that creature will continue to attack or block that combat."
    );
    ruling!(
        "Gwafa Hazid, Profiteer",
        "If Gwafa Hazid leaves the battlefield, any bribery counters will remain where they are. However, without a Gwafa Hazid on the battlefield, creatures with bribery counters on them can attack and block as normal. If another Gwafa Hazid enters, its ability will affect all creatures with bribery counters on them, including the ones that received counters from a different Gwafa Hazid."
    );
    supported("Gwafa Hazid, Profiteer");
    let bribe = |t: &mut TestGame, gwafa: ObjectId, c: ObjectId| {
        t.lands(P0, "Plains", 1);
        t.lands(P0, "Island", 1);
        let before = libraries(t);
        t.activate(P0, gwafa, 0, &[obj(c)]).unwrap();
        t.resolve();
        assert_eq!(t.counters(c, "bribery"), 1);
        assert_eq!(drawn_since(t, &before)[1], 1, "its controller draws");
    };
    // A bribed creature can't block.
    let mut t = TestGame::new(2);
    let gwafa = t.battlefield(P0, "Gwafa Hazid, Profiteer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    let raider = t.battlefield(P0, "Hill Giant");
    bribe(&mut t, gwafa, bears);
    attack_with(&mut t, &[(raider, pl(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[(bears, raider)]), "can't block");
    assert!(legal_blocks(&mut t, P1, &[(other, raider)]));
    // Gwafa leaves: the counter stays, and the creature can block.
    destroy(&mut t, gwafa);
    assert_eq!(t.counters(bears, "bribery"), 1);
    assert!(legal_blocks(&mut t, P1, &[(bears, raider)]), "blocks as normal");
    // A bribed creature can't attack.
    let mut t = TestGame::new(2);
    let gwafa = t.battlefield(P0, "Gwafa Hazid, Profiteer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Grizzly Bears");
    bribe(&mut t, gwafa, bears);
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, bears), "can't attack");
    assert!(can_attack(&mut t, other));
    // Gwafa leaves: the counter stays, and the creature can attack.
    destroy(&mut t, gwafa);
    assert_eq!(t.counters(bears, "bribery"), 1);
    assert!(can_attack(&mut t, bears));
    // Another Gwafa Hazid enters: the creature can't attack again.
    t.battlefield(P0, "Gwafa Hazid, Profiteer");
    assert!(!can_attack(&mut t, bears));
    // Bribed after it was declared as an attacker: it keeps attacking and deals damage.
    let mut t = TestGame::new(2);
    let gwafa = t.battlefield(P0, "Gwafa Hazid, Profiteer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, pl(P0))]);
    bribe(&mut t, gwafa, bears);
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.life(P0), 18);
    // Bribed after it was declared as a blocker: it keeps blocking.
    let mut t = TestGame::new(2);
    let gwafa = t.battlefield(P0, "Gwafa Hazid, Profiteer");
    let raider = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(raider, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(bears, raider)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    bribe(&mut t, gwafa, bears);
    assert_eq!(blocks_now(&t), vec![(bears, raider)]);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "the Giant stayed blocked");
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn rowans_attack_requirement_doesnt_force_creatures_that_cant_attack_or_costs() {
    cr!("508.1d", "508.1h", "302.6");
    // Misattached: the data files this ruling under Will Kenrith; it describes Rowan
    // Kenrith's first ability ("+2: During target player's next turn, each creature that
    // player controls attacks if able."). Will Kenrith takes part as a planeswalker the
    // creatures may attack (free of Ghostly Prison's cost).
    ruling!(
        "Will Kenrith",
        "If a creature affected by Rowan's first ability can't attack for any reason (such as being tapped or having come under that player's control that turn), then it doesn't attack. If there's a cost associated with having it attack, the player isn't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    supported("Rowan Kenrith");
    supported("Will Kenrith");
    let setup = |prison: bool, will: bool| {
        let mut t = TestGame::new(2);
        let rowan = pw(&mut t, P0, "Rowan Kenrith", 4);
        let will = will.then(|| pw(&mut t, P0, "Will Kenrith", 4));
        if prison {
            t.battlefield(P0, "Ghostly Prison");
        }
        let bears = t.battlefield(P1, "Grizzly Bears");
        crate::r_p007_common::loyalty(&mut t, P0, rowan, 0, &[pl(P1)]);
        t.resolve_all();
        if prison {
            // Rowan leaves (the requirement stays): attacking him would cost nothing.
            destroy(&mut t, rowan);
        }
        t.set_step(P0, Step::End);
        t.advance_to(P1, Step::PrecombatMain);
        (t, bears, will)
    };
    use crate::r_s09_common::legal_attack;
    // The untapped Bears must attack (P0 or Will Kenrith); the Elves tapped for mana and
    // the Giant that came under P1's control this turn don't.
    let (mut t, bears, will) = setup(false, true);
    let will = will.unwrap();
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.g.tap(elves);
    let giant = t.battlefield_sick(P1, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[]), "the Bears must attack");
    assert!(legal_attack(&mut t, &[(bears, pl(P0))]));
    assert!(legal_attack(&mut t, &[(bears, obj(will))]));
    assert!(!can_attack(&mut t, elves) && !can_attack(&mut t, giant));
    // With Ghostly Prison (attacking P0 costs {2} per creature) and no planeswalker to
    // attack, P1 isn't forced to pay: no attack is required.
    let (mut t, bears, _) = setup(true, false);
    t.lands(P1, "Wastes", 2);
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(legal_attack(&mut t, &[]), "no attack required with a cost");
    assert!(can_attack(&mut t, bears), "it may still attack and pay");
    // With Will Kenrith there too, attacking him costs nothing: the Bears must attack.
    let (mut t, bears, will) = setup(true, true);
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!legal_attack(&mut t, &[]));
    assert!(legal_attack(&mut t, &[(bears, obj(will.unwrap()))]));
}

#[test]
fn will_kenrith_creatures_keep_abilities_gained_afterward() {
    cr!("613.1f", "613.7", "613.4b", "613.4c");
    ruling!(
        "Will Kenrith",
        "If an affected creature gains an ability after Will's first ability resolves, it will keep that ability."
    );
    supported("Will Kenrith");
    let mut t = TestGame::new(2);
    let will = pw(&mut t, P0, "Will Kenrith", 4);
    // Serra Angel (flying, vigilance) becomes a 0/3 with no abilities.
    let angel = t.battlefield(P1, "Serra Angel");
    crate::r_p007_common::loyalty(&mut t, P0, will, 0, &[obj(angel)]);
    t.resolve_all();
    let flying = |t: &TestGame| {
        crate::r_s07_common::has_kw(t, angel, mtg_engine::keywords::KeywordKind::Flying)
    };
    assert_eq!(t.pt(angel), (0, 3));
    assert!(!flying(&t));
    // Jump gives it flying afterward: it has flying. Giant Growth applies too.
    t.lands(P0, "Island", 1);
    t.lands(P0, "Forest", 1);
    let jump = t.hand(P0, "Jump");
    t.cast(P0, jump).target(angel).go();
    t.resolve_all();
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(angel).go();
    t.resolve_all();
    assert!(flying(&t));
    assert_eq!(t.pt(angel), (3, 6));
}

#[test]
fn nessian_boar_block_requirement_and_costs() {
    cr!("509.1c", "509.1d");
    ruling!(
        "Nessian Boar",
        "If a creature can’t block Nessian Boar for any reason (such as being tapped), then it doesn’t block. If there’s a cost associated with having it block, the player isn’t forced to pay that cost, so it doesn’t have to block in that case either. If there’s a cost associated with having it block Nessian Boar, but not to block other attacking creatures, that creature is free to block one of those other creatures or to not block at all."
    );
    supported("Nessian Boar");
    supported("Awesome Presence");
    // Without costs: the untapped Bears must block the Boar; the tapped Giant can't.
    let mut t = TestGame::new(2);
    let boar = t.battlefield(P0, "Nessian Boar");
    let other = t.battlefield(P0, "Grizzly Bears");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.tap(giant);
    attack_with(&mut t, &[(boar, pl(P1)), (other, pl(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[]), "the Bears must block");
    assert!(!legal_blocks(&mut t, P1, &[(bears, other)]));
    assert!(legal_blocks(&mut t, P1, &[(bears, boar)]));
    assert!(!legal_blocks(&mut t, P1, &[(bears, boar), (giant, boar)]));
    // Awesome Presence on the Boar: blocking it costs {3}. P1 may block the other
    // attacker or not block at all.
    let mut t = TestGame::new(2);
    let boar = t.battlefield(P0, "Nessian Boar");
    let other = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Awesome Presence", obj(boar));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Wastes", 3);
    attack_with(&mut t, &[(boar, pl(P1)), (other, pl(P1))]);
    assert!(legal_blocks(&mut t, P1, &[]), "not forced to pay");
    assert!(legal_blocks(&mut t, P1, &[(bears, other)]));
    block_and_finish(&mut t, P1, &[(bears, other)]);
    assert_eq!(t.life(P1), 20 - 10, "the Boar (10/6) was unblocked");
}

#[test]
fn nessian_boar_draws_for_each_blocker_before_damage() {
    cr!("509.3d", "509.2", "509.2a");
    ruling!(
        "Nessian Boar",
        "Nessian Boar’s second ability triggers for each creature blocking it."
    );
    ruling!(
        "Nessian Boar",
        "Nessian Boar’s second ability resolves before combat damage is dealt, and players have priority to cast spells (such as the cards the defending player just drew) after the ability has resolved but before combat damage is dealt."
    );
    let mut t = TestGame::new(2);
    let boar = t.battlefield(P0, "Nessian Boar");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    // P1's next two cards: Giant Growth, then a filler.
    let gg = t.library_top(P1, "Giant Growth");
    t.lands(P1, "Forest", 1);
    attack_with(&mut t, &[(boar, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(b1, boar), (b2, boar)]),
    );
    let before = libraries(&t);
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draws a card"), 2, "one per blocker");
    t.resolve_all();
    assert_eq!(drawn_since(&t, &before), vec![0, 2]);
    // Still before combat damage: P1 casts the Giant Growth just drawn on a blocker.
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    let gg = t.g.current(gg);
    assert_eq!(t.zone(gg), mtg_engine::object::Zone::Hand(P1));
    t.cast(P1, gg).target(b1).go();
    t.resolve_all();
    // The blockers deal 5 + 2 damage: the Boar (10/6) dies.
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.in_graveyard(P0, "Nessian Boar"));
}
