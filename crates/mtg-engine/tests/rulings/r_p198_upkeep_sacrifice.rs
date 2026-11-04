//! Rulings batch P198 — "at the beginning of your upkeep" abilities that make you
//! sacrifice something, pay, or give the permanent away: what they do when their source
//! is gone (CR 603.4, 608.2), when nothing can be sacrificed (701.21), who pays after a
//! control change, and choices made as they resolve.

use crate::r_p194_common::damage_events;
use crate::r_p198_common::*;
use crate::r_s01_common::stack_library;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn demonic_taskmaster_sacrifices_even_if_it_left_and_two_sacrifice_each_other() {
    cr!("603.4", "608.2b", "701.21a");
    ruling!(
        "Demonic Taskmaster",
        "If Demonic Taskmaster leaves the battlefield after its ability triggers but before it resolves, you’ll still have to sacrifice a creature. Notably, if you control two Demonic Taskmasters and no other creatures, each one will force you to sacrifice the other."
    );
    ruling!(
        "Demonic Taskmaster",
        "If you control no other creatures, you won’t sacrifice anything."
    );
    supported("Demonic Taskmaster");
    // Bounced with its trigger on the stack: the Bears are still sacrificed.
    let mut t = TestGame::new(2);
    let dt = t.battlefield(P0, "Demonic Taskmaster");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    bounce(&mut t, dt);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // Two Taskmasters and nothing else: both die.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Demonic Taskmaster");
    let b = t.battlefield(P0, "Demonic Taskmaster");
    to_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    // Alone: nothing is sacrificed.
    let mut t = TestGame::new(2);
    let dt = t.battlefield(P0, "Demonic Taskmaster");
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.on_battlefield(dt));
}

#[test]
fn lord_of_the_pits_can_sacrifice_each_other_and_lifelink_offsets_its_damage() {
    cr!("701.21a", "702.15b", "120.3f", "119.3");
    ruling!(
        "Lord of the Pit",
        "If you have two Lords of the Pit, you can sacrifice them to each other."
    );
    ruling!(
        "Lord of the Pit",
        "If Lord of the Pit has gained lifelink, being dealt 7 damage by it doesn’t cause your life total to change, even though you gain and lose life. If your life total is less than 7, you won’t lose the game."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Lord of the Pit");
    let b = t.battlefield(P0, "Lord of the Pit");
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!(t.life(P0), 20);
    // Alone, with lifelink from Whip of Erebos, at 5 life.
    let mut t = TestGame::new(2);
    let lord = t.battlefield(P0, "Lord of the Pit");
    t.battlefield(P0, "Whip of Erebos");
    t.g.players[0].life = 5;
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.on_battlefield(lord));
    assert!(damage_events(&t)
        .iter()
        .any(|(s, to, n)| *s == lord && *to == pl(P0) && *n == 7));
    assert_eq!(t.life(P0), 5);
    assert!(!t.has_lost(P0));
}

#[test]
fn liege_of_the_pit_must_sacrifice_another_creature_if_it_can() {
    cr!("701.21a", "608.2c");
    ruling!(
        "Liege of the Pit",
        "If you have another creature on the battlefield when the ability resolves, you must sacrifice it. You can’t choose to be dealt 7 damage instead."
    );
    let mut t = TestGame::new(2);
    let liege = t.battlefield(P0, "Liege of the Pit");
    let bears = t.battlefield(P0, "Grizzly Bears");
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(liege));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn two_xathrid_demons_sacrifice_each_other_or_you_lose_seven() {
    cr!("701.21a", "603.4", "608.2b");
    ruling!(
        "Xathrid Demon",
        "If you have two Xathrid Demons, both abilities will trigger at the beginning of your upkeep. When the first Demon’s ability resolves, you can sacrifice the second Demon to satisfy the requirement (“Sacrifice a creature other than Xathrid Demon” really means “Sacrifice a creature other than this creature”). When the second Demon’s ability resolves, you must sacrifice another creature. If you can’t (because the first Demon has somehow left the battlefield and you control no other creatures), you lose 7 life, regardless of whether the second Demon is still on the battlefield at this time."
    );
    supported("Xathrid Demon");
    // Both are sacrificed, each to the other's ability: the opponent loses 7 (a Demon's power) twice.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Xathrid Demon");
    let b = t.battlefield(P0, "Xathrid Demon");
    to_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!(t.life(P1), 20 - 7 - 7);
    assert_eq!(t.life(P0), 20);
    // The first one's ability sacrifices the second (whose ability is still on the stack)
    // after the first left: nothing to sacrifice, you lose 7.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Xathrid Demon");
    let b = t.battlefield(P0, "Xathrid Demon");
    to_upkeep(&mut t, P0);
    t.resolve();
    let left = [a, b].into_iter().find(|d| t.on_battlefield(*d)).unwrap();
    assert_eq!(t.life(P1), 13);
    bounce(&mut t, left);
    t.resolve_all();
    assert_eq!(t.life(P0), 13);
}

#[test]
fn archdemon_of_unx_sacrifices_itself_if_needed_and_makes_a_zombie_regardless() {
    cr!("701.21a", "608.2c");
    ruling!(
        "Archdemon of Unx",
        "If Archdemon of Unx is the only non-Zombie creature you control when the triggered ability resolves, you’ll have to sacrifice it. You’ll still get a Zombie token."
    );
    ruling!(
        "Archdemon of Unx",
        "If you somehow control no non-Zombie creatures when the triggered ability resolves, you won’t have to sacrifice anything. You’ll still get a Zombie token."
    );
    supported("Archdemon of Unx");
    let mut t = TestGame::new(2);
    let arch = t.battlefield(P0, "Archdemon of Unx");
    let gd = t.battlefield(P0, "Gravedigger");
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(!t.on_battlefield(arch));
    assert!(t.on_battlefield(gd));
    assert_eq!(with_subtype(&t, P0, "Zombie").len(), 2);
    // Bounced with the trigger on the stack; only Zombies left.
    let mut t = TestGame::new(2);
    let arch = t.battlefield(P0, "Archdemon of Unx");
    let gd = t.battlefield(P0, "Gravedigger");
    to_upkeep(&mut t, P0);
    bounce(&mut t, arch);
    t.resolve_all();
    assert!(t.on_battlefield(gd));
    assert_eq!(with_subtype(&t, P0, "Zombie").len(), 2);
}

#[test]
fn smothering_abomination_sacrifices_itself_and_draws_for_it() {
    cr!("701.21a", "603.10a");
    ruling!(
        "Smothering Abomination",
        "If Smothering Abomination is the only creature you control as its first triggered ability resolves, you must sacrifice it."
    );
    ruling!(
        "Smothering Abomination",
        "Sacrificing Smothering Abomination will cause its last ability to trigger."
    );
    let mut t = TestGame::new(2);
    let sa = t.battlefield(P0, "Smothering Abomination");
    to_upkeep(&mut t, P0);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert!(!t.on_battlefield(sa));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn rust_elemental_sacrifices_an_artifact_if_it_can() {
    cr!("701.21a");
    ruling!(
        "Rust Elemental",
        "Sacrificing an artifact isn't optional. If you control an artifact other than Rust Elemental, you must sacrifice it. Otherwise, you must tap Rust Elemental and lose 4 life."
    );
    let mut t = TestGame::new(2);
    let re = t.battlefield(P0, "Rust Elemental");
    let ring = t.battlefield(P0, "Sol Ring");
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(!t.on_battlefield(ring));
    assert!(!t.obj_now(re).tapped);
    assert_eq!(t.life(P0), 20);
    let mut t = TestGame::new(2);
    let re = t.battlefield(P0, "Rust Elemental");
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.obj_now(re).tapped);
    assert_eq!(t.life(P0), 16);
}

#[test]
fn stampeding_serow_returns_without_targeting_and_itself_if_needed() {
    cr!("115.10a", "608.2c", "702.18a");
    ruling!(
        "Stampeding Serow",
        "The ability that returns a creature doesn’t target anything. You can return a creature with shroud or protection from green, for example. You don’t decide which creature to return until the ability resolves. If you control no other green creatures, you must return Stampeding Serow itself."
    );
    let mut t = TestGame::new(2);
    let serow = t.battlefield(P0, "Stampeding Serow");
    let ae = t.battlefield(P0, "Argothian Enchantress");
    assert!(t.obj_now(ae).has_keyword(KeywordKind::Shroud));
    to_upkeep(&mut t, P0);
    t.answer_choose(P0, &[obj(ae)]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Argothian Enchantress"));
    assert!(t.on_battlefield(serow));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stampeding Serow");
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.in_hand(P0, "Stampeding Serow"));
}

#[test]
fn trusted_advisor_and_null_profusion_apply_in_timestamp_order() {
    cr!("402.2", "613.7", "613.11");
    ruling!(
        "Trusted Advisor",
        "If multiple effects modify your hand size, apply them in timestamp order."
    );
    supported("Null Profusion");
    for (first, second, size) in [
        ("Null Profusion", "Trusted Advisor", 4),
        ("Trusted Advisor", "Null Profusion", 2),
    ] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, first);
        t.battlefield(P0, second);
        t.g.recompute();
        assert_eq!(
            t.g.player(P0).max_hand_size,
            Some(size),
            "{first} then {second}"
        );
    }
}

#[test]
fn rohgahh_taps_and_gives_away_every_kobold_to_an_opponent_chosen_on_resolution() {
    cr!("608.2c", "613.1b");
    ruling!(
        "Rohgahh of Kher Keep",
        "If you don't pay {R}{R}{R} as the first ability resolves, all creatures named Kobolds of Kher Keep become tapped (even the ones you don't control), as does Rohgahh. The chosen opponent gains control of Rohgahh and all creatures named Kobolds of Kher Keep regardless of whether you control them or whether they were tapped by the ability."
    );
    ruling!(
        "Rohgahh of Kher Keep",
        "Rohgahh's first ability isn't targeted. You don't choose the opponent that will gain control of the creatures until the ability resolves."
    );
    let mut t = TestGame::new(3);
    let r = t.battlefield(P0, "Rohgahh of Kher Keep");
    let mine = t.battlefield(P0, "Kobolds of Kher Keep");
    let theirs = t.battlefield(P2, "Kobolds of Kher Keep");
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[pl(P2)]);
    t.resolve_all();
    let asked = &t.asked()[from..];
    assert!(asked
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { .. })));
    for k in [r, mine, theirs] {
        assert!(t.obj_now(k).tapped);
        assert_eq!(ctl(&t, k), P2);
    }
}

#[test]
fn rogue_skycaptain_makes_its_choices_on_resolution() {
    cr!("608.2c", "603.3");
    ruling!(
        "Rogue Skycaptain",
        "No choices are made when this ability triggers. After the wage counter is put on Rogue Skycaptain, then you choose whether to pay mana and, if you don’t, which opponent gains control of it."
    );
    supported("Rogue Skycaptain");
    let mut t = TestGame::new(3);
    let rs = t.battlefield(P0, "Rogue Skycaptain");
    t.g.add_counters(obj(rs), "wage", 1, None);
    t.lands(P0, "Wastes", 4);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.counters(rs, "wage"), 1);
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[pl(P2)]);
    t.resolve_all();
    // Asked on resolution: pay first, then the opponent.
    let asked: Vec<_> = t.asked()[from..]
        .iter()
        .filter(|(p, _)| *p == P0)
        .map(|(_, d)| d.clone())
        .collect();
    let pay = asked
        .iter()
        .position(|d| matches!(d, Decision::YesNo { .. }))
        .unwrap();
    let who = asked
        .iter()
        .position(|d| matches!(d, Decision::ChooseEntities { .. }))
        .unwrap();
    assert!(pay < who);
    assert_eq!(ctl(&t, rs), P2);
    assert_eq!(t.counters(rs, "wage"), 0);
    // Paying {2} per wage counter (two now) keeps it.
    let mut t = TestGame::new(2);
    let rs = t.battlefield(P0, "Rogue Skycaptain");
    t.g.add_counters(obj(rs), "wage", 1, None);
    t.lands(P0, "Wastes", 4);
    t.advance_to(P0, Step::Upkeep);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(ctl(&t, rs), P0);
    assert_eq!(t.counters(rs, "wage"), 2);
}

#[test]
fn child_of_gaea_stays_if_you_no_longer_control_it() {
    cr!("701.21a", "608.2b");
    ruling!(
        "Child of Gaea",
        "If you do choose not to pay, but no longer control Child of Gaea, you cannot sacrifice. It will remain on the battlefield under its current controller’s control."
    );
    let mut t = TestGame::new(2);
    let child = t.battlefield(P0, "Child of Gaea");
    to_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    cast_at(&mut t, P1, "Ray of Command", &[obj(child)]);
    t.resolve();
    assert_eq!(ctl(&t, child), P1);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.on_battlefield(child));
    assert_eq!(ctl(&t, child), P1);
}

#[test]
fn primordial_ooze_is_tapped_even_if_the_damage_is_prevented() {
    cr!("615.1", "608.2c");
    ruling!(
        "Primordial Ooze",
        "It becomes tapped even if the damage is prevented."
    );
    supported("Primordial Ooze");
    supported("Personal Sanctuary");
    let mut t = TestGame::new(2);
    let ooze = t.battlefield(P0, "Primordial Ooze");
    t.battlefield(P0, "Personal Sanctuary");
    to_upkeep(&mut t, P0);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.counters(ooze, "+1/+1"), 1);
    assert!(t.obj_now(ooze).tapped);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn sheltering_ancient_is_sacrificed_if_no_opponent_controls_a_creature() {
    cr!("702.24a", "118.3");
    ruling!(
        "Sheltering Ancient",
        "If no opponent controls a creature, you can’t pay the upkeep and must sacrifice Sheltering Ancient."
    );
    let mut t = TestGame::new(2);
    let sa = t.battlefield(P0, "Sheltering Ancient");
    to_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.on_battlefield(sa));
    // With a creature to put the counter on, it stays.
    let mut t = TestGame::new(2);
    let sa = t.battlefield(P0, "Sheltering Ancient");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(sa));
    assert_eq!(t.counters(bears, "+1/+1"), 1);
}

#[test]
fn sacred_mesa_can_make_a_pegasus_in_the_upkeep_to_sacrifice() {
    cr!("117.3a", "503.1");
    ruling!(
        "Sacred Mesa",
        "It can be used during upkeep to create a Pegasus token in time for it to be sacrificed."
    );
    let mut t = TestGame::new(2);
    let mesa = t.battlefield(P0, "Sacred Mesa");
    to_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    lots(&mut t, P0);
    t.activate(P0, mesa, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(with_subtype(&t, P0, "Pegasus").len(), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(mesa));
    assert!(with_subtype(&t, P0, "Pegasus").is_empty());
}

#[test]
fn gurzigosts_unblocked_damage_goes_to_the_planeswalker_it_attacks() {
    cr!("510.1c", "506.4");
    ruling!(
        "Gurzigost",
        "If this creature is attacking a planeswalker, assigning its damage as though it weren’t blocked means the damage is assigned to the planeswalker, not to the defending player."
    );
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gurzigost");
    let pw = t.battlefield(P1, "Archangel Elspeth");
    t.g.objects[pw.0 as usize]
        .counters
        .insert("loyalty".into(), 10);
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.hand(P0, "Island");
    t.set_step(P0, Step::BeginningOfCombat);
    crate::r_s01_common::attack_with(&mut t, &[(g, obj(pw))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(blocker, g)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    lots(&mut t, P0);
    t.answer_yes(P0, true);
    t.activate(P0, g, 0, &[]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.obj_now(pw).counter("loyalty"), 4);
    assert_eq!(t.life(P1), 20);
    assert!(t.on_battlefield(blocker));
}

#[test]
fn charnel_troll_can_use_untapped_lands_in_the_upkeep_before_the_draw() {
    cr!("502.3", "503.1", "504.1");
    ruling!(
        "Charnel Troll",
        "Because your upkeep step is after your untap step, your lands will be untapped to help you activate Charnel Troll’s last ability during your upkeep. However, because your upkeep step is before your draw step, you won’t have drawn a card for the turn yet."
    );
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Charnel Troll");
    let lands = [t.battlefield(P0, "Swamp"), t.battlefield(P0, "Forest")];
    t.hand(P0, "Hill Giant");
    for l in lands {
        t.g.tap(l);
    }
    let top = stack_library(&mut t, P0, &["Grizzly Bears"])[0];
    to_upkeep(&mut t, P0);
    assert!(lands.iter().all(|l| !t.obj_now(*l).tapped));
    // The card for the turn hasn't been drawn: only Hill Giant can be discarded.
    assert!(!t.in_hand(P0, "Grizzly Bears"));
    let _ = top;
    t.activate(P0, troll, 0, &[]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(t.counters(troll, "+1/+1"), 1);
    // Its upkeep trigger exiles that creature card from the graveyard.
    t.resolve_all();
    assert_eq!(t.counters(troll, "+1/+1"), 2);
    assert!(t.in_exile("Hill Giant"));
}

#[test]
fn breeding_pit_must_be_on_the_battlefield_at_the_end_step() {
    cr!("603.2", "603.4");
    ruling!(
        "Breeding Pit",
        "Breeding Pit must actually be on the battlefield at the beginning of the endstep in order for you to get a Thrull. The getting of the Thrull is not part of paying the upkeep, it is part of having the card on the battlefield."
    );
    // Paid, then destroyed before the end step: no Thrull.
    let mut t = TestGame::new(2);
    let pit = t.battlefield(P0, "Breeding Pit");
    t.lands(P0, "Swamp", 2);
    to_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(pit));
    let thrulls = with_subtype(&t, P0, "Thrull").len();
    kill(&mut t, pit);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Thrull").len(), thrulls);
    // Put onto the battlefield after the upkeep (nothing paid): a Thrull.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Breeding Pit");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Thrull").len(), 1);
}

#[test]
fn icy_prison_leaving_early_exiles_the_creature_forever() {
    cr!("603.6a", "610.3c");
    ruling!(
        "Icy Prison",
        "If Icy Prison leaves the battlefield before its first ability has resolved, its third ability will trigger and do nothing. Then its first ability will resolve and exile the targeted creature forever."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let ip = t.enter(P0, "Icy Prison");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, ip);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    pass_turn(&mut t);
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn static_prison_leaving_early_exiles_nothing() {
    cr!("610.3c");
    ruling!(
        "Static Prison",
        "If Static Prison leaves the battlefield before its first triggered ability resolves, the target permanent won't be exiled."
    );
    supported("Static Prison");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let sp = t.enter(P0, "Static Prison");
    t.settle();
    kill(&mut t, sp);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn illusions_of_grandeurs_life_loss_goes_with_its_controller() {
    cr!("603.3a", "603.10a");
    ruling!(
        "Illusions of Grandeur",
        "If a player takes control of this card away from you, you do not lose 20 life because the loss of 20 life is a separate ability. That player is now subject to the loss."
    );
    supported("Confiscate");
    let mut t = TestGame::new(2);
    let iog = t.enter(P0, "Illusions of Grandeur");
    t.resolve_all();
    assert_eq!(t.life(P0), 40);
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Confiscate", &[obj(iog)]);
    t.resolve_all();
    assert_eq!(ctl(&t, iog), P1);
    kill(&mut t, iog);
    t.resolve_all();
    assert_eq!(t.life(P0), 40);
    assert_eq!(t.life(P1), 0);
}
