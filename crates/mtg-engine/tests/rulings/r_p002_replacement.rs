//! Rulings batch P002 — replacement, prevention and redirection effects: Ravenous Slime's
//! "exile it instead" (CR 614.6) with several such effects and commanders (CR 616.1,
//! 903.9a), the order of replacement and prevention effects (CR 616.1, 615), redirection
//! of several simultaneous chunks of damage (Glarecaster), Channel Harm, Reverberation,
//! Riot Control and Selfless Squire.

use crate::r_p002_common::*;
use crate::r_p003_common::*;
use crate::r_s01_common::supported;
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Ravenous Slime: "If a creature an opponent controls would die, instead exile it and put
// a number of +1/+1 counters equal to that creature's power on this creature."
// ---------------------------------------------------------------------------------------

#[test]
fn ravenous_slime_exiles_creatures_dying_with_it_and_uses_their_last_known_power() {
    cr!("614.6", "608.2h");
    ruling!(
        "Ravenous Slime",
        "If Ravenous Slime dies at the same time as an opponent's creatures, the opponent's creatures are exiled."
    );
    ruling!(
        "Ravenous Slime",
        "The exiled creature's last known existence on the battlefield is checked to determine its power."
    );
    supported("Ravenous Slime");
    // Pyroclasm: 2 damage to each creature — the 1/1 Slime and P1's two Bears all die at once.
    let mut t = TestGame::new(2);
    let slime = add(&mut t, P0, "Ravenous Slime");
    add(&mut t, P1, "Grizzly Bears");
    add(&mut t, P1, "Grizzly Bears");
    add(&mut t, P0, "Llanowar Elves");
    cast_resolve(&mut t, P0, "Pyroclasm", "1R", &[]);
    assert!(!t.on_battlefield(slime));
    assert!(t.in_graveyard(P0, "Ravenous Slime"));
    assert_eq!(exiled(&t, "Grizzly Bears"), 2);
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
    // P0's own creature isn't affected.
    assert!(t.in_graveyard(P0, "Llanowar Elves"));
    // Last known power: Bears pumped by Giant Growth (5/5) is exiled, five counters.
    let mut t = TestGame::new(2);
    let slime = add(&mut t, P0, "Ravenous Slime");
    let bears = add(&mut t, P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    giant_growth(&mut t, bears);
    assert_eq!(t.pt(bears), (5, 5));
    kill(&mut t, bears);
    assert_eq!(exiled(&t, "Grizzly Bears"), 1);
    assert_eq!(t.counters(slime, counters::PLUS1), 5);
    assert_eq!(t.pt(slime), (6, 6));
}

#[test]
fn ravenous_slime_then_a_commander_may_go_to_the_command_zone() {
    cr!("903.9a", "614.6", "704.6d");
    ruling!(
        "Ravenous Slime",
        "If an opponent's commander would die, that player may choose to put it into the command zone from exile after Ravenous Slime's replacement effect has been applied and it has received the +1/+1 counters."
    );
    for to_command in [true, false] {
        let mut t = TestGame::with_config(2, GameConfig::commander_game());
        let slime = add(&mut t, P0, "Ravenous Slime");
        let cmdr = t.battlefield(P1, "Hill Giant");
        t.g.objects[cmdr.0 as usize].is_commander = true;
        t.g.players[1].commander_names.push("Hill Giant".into());
        t.answer_yes(P1, to_command);
        kill(&mut t, cmdr);
        // Slime got its counters either way (the commander was exiled first).
        assert_eq!(t.counters(slime, counters::PLUS1), 3);
        assert!(!t.in_graveyard(P1, "Hill Giant"));
        let in_command = t.g.find_in_zone(Zone::Command, "Hill Giant").len();
        assert_eq!(in_command, usize::from(to_command));
        assert_eq!(exiled(&t, "Hill Giant"), usize::from(!to_command));
    }
}

#[test]
fn two_ravenous_slimes_the_dying_creature_s_controller_picks_one() {
    cr!("616.1", "616.1e", "614.6");
    ruling!(
        "Ravenous Slime",
        "If more than one replacement effect tries to change where a dying creature ends up, that creature's controller chooses which effect applies. For example, if two players control a Ravenous Slime"
    );
    // P0 and P1 each control a Slime; P2's Hill Giant dies. P2 chooses either Slime.
    for chosen in [P0, P1] {
        let mut t = TestGame::new(3);
        let s0 = add(&mut t, P0, "Ravenous Slime");
        let s1 = add(&mut t, P1, "Ravenous Slime");
        let giant = add(&mut t, P2, "Hill Giant");
        prefer(
            &mut t,
            P2,
            if chosen == P0 {
                "controlled by P0"
            } else {
                "controlled by P1"
            },
        );
        let from = t.asked().len();
        kill(&mut t, giant);
        let choices = replacement_choices(&t, from);
        assert_eq!(choices.len(), 1, "{choices:?}");
        assert_eq!(choices[0].0, P2, "the dying creature's controller chooses");
        assert_eq!(choices[0].1.len(), 2);
        assert_ne!(choices[0].1[0], choices[0].1[1], "the options can be told apart");
        assert_eq!(exiled(&t, "Hill Giant"), 1);
        let (got, other) = if chosen == P0 { (s0, s1) } else { (s1, s0) };
        assert_eq!(t.counters(got, counters::PLUS1), 3);
        assert_eq!(t.counters(other, counters::PLUS1), 0);
    }
}

// ---------------------------------------------------------------------------------------
// The order of replacement and prevention effects (CR 616.1).
// ---------------------------------------------------------------------------------------

#[test]
fn tainted_remedy_and_alhammarret_s_archive_the_gaining_player_orders_them() {
    cr!("616.1", "616.1e", "614.1a");
    ruling!(
        "Tainted Remedy",
        "If more than one replacement effect tries to apply to a life gain event, the player who would gain life chooses the order in which they apply."
    );
    supported("Alhammarret's Archive");
    for (first, loss) in [("Alhammarret", 6), ("Tainted Remedy", 3)] {
        let mut t = TestGame::new(2);
        add(&mut t, P0, "Alhammarret's Archive");
        add(&mut t, P1, "Tainted Remedy");
        prefer(&mut t, P0, first);
        let from = t.asked().len();
        // Healing Salve: "Target player gains 3 life."
        let salve = t.hand(P0, "Healing Salve");
        mana(&mut t, P0, "W");
        t.cast(P0, salve).modes(&[0]).target(pl(P0)).go();
        t.resolve_all();
        assert_eq!(t.life(P0), 20 - loss, "{first} first");
        let choices = replacement_choices(&t, from);
        assert_eq!(choices.len(), 1, "{choices:?}");
        assert_eq!(choices[0].0, P0);
    }
}

#[test]
fn new_way_forward_and_another_prevention_effect_you_choose_the_order() {
    cr!("616.1", "615.7");
    ruling!(
        "New Way Forward",
        "If another effect (or effects) modifies how much damage the chosen source would deal to you this turn, you choose the order"
    );
    // P0: Healing Salve ("Prevent the next 3 damage that would be dealt to any target
    // this turn") on themself, then New Way Forward choosing P1's Hill Giant, which deals
    // 3 combat damage to P0.
    for (first, p1_loss, drawn) in [("New Way Forward", 3, 3), ("Healing Salve", 0, 0)] {
        let mut t = TestGame::new(2);
        let giant = add(&mut t, P1, "Hill Giant");
        let salve = t.hand(P0, "Healing Salve");
        mana(&mut t, P0, "W");
        t.cast(P0, salve).modes(&[1]).target(pl(P0)).go();
        t.resolve_all();
        t.answer_choose(P0, &[obj(giant)]);
        cast_resolve(&mut t, P0, "New Way Forward", "2WUR", &[]);
        prefer(&mut t, P0, first);
        let hand = t.hand_size(P0);
        let from = t.asked().len();
        t.set_step(P1, Step::PrecombatMain);
        t.attack(&[(giant, pl(P0))], &[]);
        t.resolve_all();
        let choices = replacement_choices(&t, from);
        assert_eq!(choices.len(), 1, "{choices:?}");
        assert_eq!(choices[0].0, P0);
        assert_eq!(t.life(P0), 20, "{first} first");
        assert_eq!(t.life(P1), 20 - p1_loss, "{first} first");
        assert_eq!(t.hand_size(P0), hand + drawn, "{first} first");
    }
}

#[test]
fn inkshield_and_another_prevention_effect_you_choose_the_order() {
    cr!("616.1", "615.7");
    ruling!(
        "Inkshield",
        "If multiple damage prevention effects would apply to combat damage that would be dealt to you, you get to choose the order"
    );
    // Craw Wurm (6/4) attacks P0, who has Healing Salve's "prevent the next 3" and
    // Inkshield: Inkshield first prevents all 6 (six Inklings); Salve first prevents 3
    // and Inkshield the other 3 (three Inklings).
    for (first, inklings) in [("Inkshield", 6), ("Healing Salve", 3)] {
        let mut t = TestGame::new(2);
        let wurm = add(&mut t, P1, "Craw Wurm");
        t.set_step(P1, Step::BeginningOfCombat);
        let salve = t.hand(P0, "Healing Salve");
        mana(&mut t, P0, "W");
        t.cast(P0, salve).modes(&[1]).target(pl(P0)).go();
        t.resolve_all();
        cast_resolve(&mut t, P0, "Inkshield", "3WB", &[]);
        prefer(&mut t, P0, first);
        let from = t.asked().len();
        t.attack(&[(wurm, pl(P0))], &[]);
        t.resolve_all();
        assert_eq!(replacement_choices(&t, from).len(), 1);
        assert_eq!(t.life(P0), 20);
        assert_eq!(
            crate::r_s01_common::tokens(&t, P0).len(),
            inklings,
            "{first} first"
        );
    }
}

// ---------------------------------------------------------------------------------------
// Glarecaster: "{5}{W}: The next time damage would be dealt to this creature and/or you
// this turn, that damage is dealt to any target instead."
// ---------------------------------------------------------------------------------------

#[test]
fn glarecaster_redirects_every_chunk_dealt_at_the_same_time() {
    cr!("614.9", "510.2");
    interp!("next-time-damage-batch");
    ruling!(
        "Glarecaster",
        "If both you and this card would be dealt damage at the same time, or if either you or this card would be dealt damage from multiple sources at the same time, the redirection will apply to both chunks of damage."
    );
    supported("Glarecaster");
    supported("Hurricane");
    // Hurricane for 2 (2 damage to each flier and each player): the 2 to Glarecaster and
    // the 2 to P0 both go to P1, who also takes 2 of their own.
    let mut t = TestGame::new(2);
    let glare = add(&mut t, P0, "Glarecaster");
    activate_resolve(&mut t, P0, glare, "next time", "5W", &[pl(P1)]);
    t.set_step(P1, Step::PrecombatMain);
    let h = t.hand(P1, "Hurricane");
    mana(&mut t, P1, "2G");
    t.cast(P1, h).x(2).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(dmg(&t, glare), 0);
    assert_eq!(t.life(P1), 14);
    // It applies once: a second Hurricane is dealt normally.
    let h = t.hand(P1, "Hurricane");
    mana(&mut t, P1, "1G");
    t.cast(P1, h).x(1).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 13);
    // Two unblocked attackers deal combat damage to P0 at once: both redirected (to one
    // of the attackers).
    let mut t = TestGame::new(2);
    let glare = add(&mut t, P0, "Glarecaster");
    let giant = add(&mut t, P1, "Hill Giant");
    let bears = add(&mut t, P1, "Grizzly Bears");
    let wurm = add(&mut t, P1, "Craw Wurm");
    activate_resolve(&mut t, P0, glare, "next time", "5W", &[obj(wurm)]);
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(giant, pl(P0)), (bears, pl(P0))], &[]);
    assert_eq!(t.life(P0), 20);
    assert!(!t.on_battlefield(wurm), "5 damage to the 6/4 Craw Wurm");
}

// ---------------------------------------------------------------------------------------
// Channel Harm: "Prevent all damage that would be dealt to you and permanents you control
// this turn by sources you don't control. If damage is prevented this way, you may have
// Channel Harm deal that much damage to target creature."
// ---------------------------------------------------------------------------------------

#[test]
fn channel_harm_with_an_illegal_target_does_nothing() {
    cr!("608.2b");
    ruling!(
        "Channel Harm",
        "If the target creature is an illegal target as Channel Harm tries to resolve, the entire spell doesn’t resolve. No damage will be prevented."
    );
    supported("Channel Harm");
    let mut t = TestGame::new(2);
    let bears = add(&mut t, P1, "Grizzly Bears");
    cast(&mut t, P0, "Channel Harm", "5W", &[obj(bears)]);
    kill(&mut t, bears);
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    bolt(&mut t, P1, pl(P0));
    assert_eq!(t.life(P0), 17);
}

#[test]
fn channel_harm_keeps_preventing_after_its_target_leaves() {
    cr!("608.2b", "615.3");
    ruling!(
        "Channel Harm",
        "Whether the target creature is still a legal target is not checked after Channel Harm resolves. Damage will still be prevented"
    );
    let mut t = TestGame::new(2);
    let bears = add(&mut t, P1, "Grizzly Bears");
    let mine = add(&mut t, P0, "Llanowar Elves");
    cast_resolve(&mut t, P0, "Channel Harm", "5W", &[obj(bears)]);
    kill(&mut t, bears);
    t.set_step(P1, Step::PrecombatMain);
    t.answer_yes(P0, true);
    bolt(&mut t, P1, pl(P0));
    bolt(&mut t, P1, obj(mine));
    assert_eq!(t.life(P0), 20);
    assert!(t.on_battlefield(mine));
}

#[test]
fn channel_harm_on_your_own_creature_deals_it_the_damage() {
    cr!("615.1", "615.5");
    ruling!(
        "Channel Harm",
        "You can choose a creature you control as the target. If you do, the damage Channel Harm deals to that creature won’t be prevented."
    );
    let mut t = TestGame::new(2);
    let giant = add(&mut t, P0, "Hill Giant");
    cast_resolve(&mut t, P0, "Channel Harm", "5W", &[obj(giant)]);
    t.set_step(P1, Step::PrecombatMain);
    t.answer_yes(P0, true);
    bolt(&mut t, P1, pl(P0));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert!(!t.on_battlefield(giant), "3 damage from Channel Harm");
}

#[test]
fn two_channel_harms_loop_until_a_player_declines() {
    cr!("615.1", "615.5");
    ruling!(
        "Channel Harm",
        "This forms a loop that either player can break by choosing to not have Channel Harm try to deal damage. It doesn’t matter which player breaks the loop"
    );
    // P0's Channel Harm targets P1's Craw Wurm; P1's targets P0's Hill Giant. P1 bolts
    // P0. Whoever stops the loop, no damage is dealt.
    // (P0's yeses, P1's yeses, times P0 and P1 are asked): the answers alternate, P0
    // first, until someone says no.
    for (p0_yes, p1_yes, asked) in [
        (1, 0, (1, 1)),
        (2, 1, (2, 2)),
        (1, 1, (2, 1)),
        (0, 0, (1, 0)),
    ] {
        let mut t = TestGame::new(2);
        let giant = add(&mut t, P0, "Hill Giant");
        let wurm = add(&mut t, P1, "Craw Wurm");
        cast_resolve(&mut t, P0, "Channel Harm", "5W", &[obj(wurm)]);
        cast_resolve(&mut t, P1, "Channel Harm", "5W", &[obj(giant)]);
        for _ in 0..p0_yes {
            t.answer_yes(P0, true);
        }
        t.answer_yes(P0, false);
        for _ in 0..p1_yes {
            t.answer_yes(P1, true);
        }
        t.answer_yes(P1, false);
        t.set_step(P1, Step::PrecombatMain);
        let from = t.asked().len();
        bolt(&mut t, P1, pl(P0));
        t.resolve_all();
        let yes_no = |p: PlayerId| {
            t.asked()[from..]
                .iter()
                .filter(|(q, d)| *q == p && matches!(d, mtg_engine::decision::Decision::YesNo { .. }))
                .count()
        };
        assert_eq!((yes_no(P0), yes_no(P1)), asked, "{p0_yes}/{p1_yes}");
        assert_eq!(t.life(P0), 20);
        assert_eq!(t.life(P1), 20);
        assert_eq!(dmg(&t, giant), 0, "{p0_yes}/{p1_yes}");
        assert_eq!(dmg(&t, wurm), 0, "{p0_yes}/{p1_yes}");
        assert!(t.g.stack.is_empty());
    }
}

// ---------------------------------------------------------------------------------------
// Reverberation, Riot Control, Selfless Squire.
// ---------------------------------------------------------------------------------------

#[test]
fn reverberation_sends_all_of_a_sorcery_s_damage_to_its_controller() {
    cr!("614.9");
    ruling!(
        "Reverberation",
        "If used on a sorcery which does damage to multiple creatures and/or players, it will cause that player to take damage equal to the sum of all damage inflicted by the spell."
    );
    supported("Reverberation");
    supported("Flame Rift");
    // Pyroclasm with three creatures out: 6 damage to its controller.
    let mut t = TestGame::new(2);
    let a = add(&mut t, P0, "Llanowar Elves");
    let b = add(&mut t, P1, "Grizzly Bears");
    let c = add(&mut t, P1, "Grizzly Bears");
    let pyro = cast(&mut t, P0, "Pyroclasm", "1R", &[]);
    cast_resolve(&mut t, P1, "Reverberation", "2UU", &[obj(pyro)]);
    assert_eq!(t.life(P0), 14);
    assert!(t.on_battlefield(a) && t.on_battlefield(b) && t.on_battlefield(c));
    // Flame Rift (4 to each player): 8 to its controller.
    let rift = cast(&mut t, P0, "Flame Rift", "1R", &[]);
    cast_resolve(&mut t, P1, "Reverberation", "2UU", &[obj(rift)]);
    assert_eq!(t.life(P0), 6);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn riot_control_prevents_only_damage_to_you() {
    cr!("615.1", "306.8", "120.3c");
    ruling!(
        "Riot Control",
        "Riot Control doesn't prevent combat damage that would be dealt to planeswalkers you control. It also doesn't prevent damage that would be dealt to creatures you control."
    );
    supported("Riot Control");
    let mut t = TestGame::new(2);
    let jace = add(&mut t, P0, "Jace Beleren");
    let elves = add(&mut t, P0, "Llanowar Elves");
    let giant = add(&mut t, P1, "Hill Giant");
    let bears = add(&mut t, P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Riot Control", "2W", &[]);
    assert_eq!(t.life(P0), 22, "1 life per opposing creature");
    t.set_step(P1, Step::BeginningOfCombat);
    t.attack(&[(giant, obj(jace)), (bears, pl(P0))], &[]);
    assert_eq!(t.life(P0), 22, "damage to P0 prevented");
    assert_eq!(t.counters(jace, counters::LOYALTY), 0);
    assert!(!t.on_battlefield(jace));
    bolt(&mut t, P1, obj(elves));
    assert!(!t.on_battlefield(elves));
}

#[test]
fn selfless_squire_s_prevention_outlasts_it() {
    cr!("611.2a", "615.3");
    ruling!(
        "Selfless Squire",
        "Selfless Squire's first triggered ability prevents all damage that would be dealt to you this turn after the triggered ability resolves, even if Selfless Squire leaves the battlefield."
    );
    let mut t = TestGame::new(2);
    let squire = enter(&mut t, P0, "Selfless Squire");
    assert_eq!(t.stack_len(), 1);
    // In response to the trigger: dealt normally.
    t.set_step(P1, Step::PrecombatMain);
    cast(&mut t, P1, "Lightning Bolt", "R", &[pl(P0)]);
    t.resolve();
    assert_eq!(t.life(P0), 17);
    t.resolve_all();
    let squire = t.g.current(squire);
    kill(&mut t, squire);
    bolt(&mut t, P1, pl(P0));
    assert_eq!(t.life(P0), 17);
}

#[test]
fn a_next_time_prevention_covers_all_of_a_source_s_simultaneous_damage() {
    cr!("615.8", "615.5", "510.2", "702.19b");
    interp!("next-time-damage-batch");
    supported("Awe Strike");
    // Awe Strike ("The next time target creature would deal damage this turn, prevent
    // that damage. You gain life equal to the damage prevented this way.") on a trampling
    // Colossal Dreadmaw (6/6) blocked by Grizzly Bears: 2 to the Bears and 4 to P0, all
    // prevented; P0 gains 6.
    let mut t = TestGame::new(2);
    let maw = add(&mut t, P1, "Colossal Dreadmaw");
    let bears = add(&mut t, P0, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    cast_resolve(&mut t, P0, "Awe Strike", "W", &[obj(maw)]);
    t.attack(&[(maw, pl(P0))], &[(bears, maw)]);
    assert!(t.on_battlefield(bears));
    assert_eq!(dmg(&t, bears), 0);
    assert_eq!(t.life(P0), 26);
    // It's used up: the next damage is dealt.
    t.set_step(P1, Step::PostcombatMain);
    t.g.deal_damage(maw, pl(P0), 6, false);
    t.settle();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_next_time_prevention_is_used_up_by_a_batch_part_of_which_can_t_be_prevented() {
    cr!("615.8", "615.12");
    interp!("next-time-damage-batch");
    supported("Whippoorwill");
    // As above, but the Bears were targeted by Whippoorwill ("Damage that would be dealt
    // to that creature this turn can't be prevented ..."): the 2 to the Bears are dealt,
    // the 4 to P0 prevented, and Awe Strike is used up all the same.
    let mut t = TestGame::new(2);
    let maw = add(&mut t, P1, "Colossal Dreadmaw");
    let bears = add(&mut t, P0, "Grizzly Bears");
    let whip = add(&mut t, P0, "Whippoorwill");
    t.set_step(P1, Step::BeginningOfCombat);
    activate_resolve(&mut t, P0, whip, "can't be prevented", "GG", &[obj(bears)]);
    cast_resolve(&mut t, P0, "Awe Strike", "W", &[obj(maw)]);
    t.attack(&[(maw, pl(P0))], &[(bears, maw)]);
    assert!(!t.on_battlefield(bears), "the 2 to the Bears can't be prevented");
    assert_eq!(t.life(P0), 24, "4 prevented, 4 gained");
    t.set_step(P1, Step::PostcombatMain);
    t.g.deal_damage(maw, pl(P0), 6, false);
    t.settle();
    assert_eq!(t.life(P0), 18, "Awe Strike was used up");
}
