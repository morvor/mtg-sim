//! Rulings batch P121 — cards whose wording this batch made compile: Protean Hydra ("a
//! +1/+1 counter is removed": once per counter, however it's removed, CR 122.6, 704.5q),
//! Pelt Collector ("enters or dies": the dying creature as it last existed, CR 603.10a;
//! the entering one with its continuous effects, CR 603.6a), Rumbling Ruin ("count the
//! number of +1/+1 counters": counted once, CR 608.2h) and Braided Net ("for as long as
//! it remains tapped", CR 611.2b, 602.5).

use crate::r_p121_common::*;
use crate::r_s02_common::target_candidates;
use mtg_engine::testing::*;
use mtg_engine::*;

#[test]
fn protean_hydra_triggers_for_each_counter_removed_for_any_reason() {
    cr!("603.2c", "704.5q", "603.7a");
    ruling!(
        "Protean Hydra",
        "Protean Hydra’s last ability triggers whenever a +1/+1 counter is removed from it for any reason, not just when a +1/+1 counter is removed by its second ability."
    );
    supported("Protean Hydra");
    // Two counters removed by an effect: two triggers, four counters at the end step.
    let mut t = TestGame::new(2);
    let hydra = with_counters(&mut t, P0, "Protean Hydra", 3);
    remove_plus1(&mut t, hydra, 2);
    t.resolve_all();
    assert_eq!(plus1(&t, hydra), 1);
    end_step(&mut t, P0);
    assert_eq!(plus1(&t, hydra), 5);
    // A -1/-1 counter: the state-based action removes a +1/+1 counter with it.
    let mut t = TestGame::new(2);
    let hydra = with_counters(&mut t, P0, "Protean Hydra", 3);
    give_minus1(&mut t, hydra, 1);
    t.resolve_all();
    assert_eq!((plus1(&t, hydra), t.counters(hydra, MINUS1)), (2, 0));
    end_step(&mut t, P0);
    assert_eq!(plus1(&t, hydra), 4);
    // Damage: prevented, and that many counters removed by its own ability.
    let mut t = TestGame::new(2);
    let hydra = with_counters(&mut t, P0, "Protean Hydra", 3);
    cast_resolve(&mut t, P1, "Shock", &[obj(hydra)]);
    assert_eq!(plus1(&t, hydra), 1);
    end_step(&mut t, P0);
    assert_eq!(plus1(&t, hydra), 5);
}

#[test]
fn pelt_collector_uses_the_dying_creatures_last_known_power() {
    cr!("603.10a", "603.4");
    ruling!(
        "Pelt Collector",
        "To determine if Pelt Collector’s first ability triggers when a creature dies, use its power as it last existed on the battlefield."
    );
    supported("Pelt Collector");
    let mut t = TestGame::new(2);
    let pelt = with_counters(&mut t, P0, "Pelt Collector", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // A 2/2 dying doesn't beat the 3/3 Pelt Collector; pumped to 5/5, it does.
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(plus1(&t, pelt), 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Giant Growth", &[obj(bears)]);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(plus1(&t, pelt), 3);
}

#[test]
fn pelt_collector_uses_the_entering_creatures_power_with_static_abilities() {
    cr!("603.6a", "613.4c", "603.4");
    ruling!(
        "Pelt Collector",
        "To determine if Pelt Collector’s first ability triggers when a creature enters the battlefield, use the creature’s power after applying any static abilities (such as that of Trostani Discordant) that modify its power."
    );
    // Raging Goblin (1/1) entering doesn't beat the 1/1 Pelt Collector; with Goblin King
    // (other Goblins get +1/+1), it does.
    for king in [false, true] {
        let mut t = TestGame::new(2);
        let pelt = t.battlefield(P0, "Pelt Collector");
        if king {
            t.battlefield(P0, "Goblin King");
        }
        t.enter(P0, "Raging Goblin");
        t.resolve_all();
        assert_eq!(plus1(&t, pelt), u32::from(king), "Goblin King: {king}");
    }
}

#[test]
fn rumbling_ruin_counts_once_and_compares_power_continuously() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Rumbling Ruin",
        "The number of +1/+1 counters among creatures you control is counted only as Rumbling Ruin’s ability resolves. The set of creatures that can’t block is constantly updated as their power changes, but the number against which their power is compared doesn’t change later in the turn, even if the number of +1/+1 counters among your creatures changes."
    );
    supported("Rumbling Ruin");
    let can_block = |t: &mut TestGame, id: ObjectId| {
        t.g.recompute();
        let id = t.g.current(id);
        t.g.can_block_at_all(id)
    };
    let mut t = TestGame::new(2);
    with_counters(&mut t, P0, "Grizzly Bears", 1);
    with_counters(&mut t, P0, "Grizzly Bears", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.enter(P0, "Rumbling Ruin");
    t.settle();
    // In response, the counted number is still open: a third counter.
    let mine = with_counters(&mut t, P0, "Elite Vanguard", 1);
    t.resolve_all();
    // Three counters: the 2/2 and the 3/3 can't block.
    assert!(!can_block(&mut t, bears));
    assert!(!can_block(&mut t, giant));
    // More counters later don't change the number; a 4/4 can block.
    give_plus1(&mut t, mine, 3);
    let wurm = t.battlefield(P1, "Hill Giant");
    give_plus1(&mut t, wurm, 1);
    assert!(can_block(&mut t, wurm));
    // The Giant pumped above three can block; shrunk again, it can't.
    give_plus1(&mut t, giant, 1);
    assert!(can_block(&mut t, giant));
    give_minus1(&mut t, giant, 1);
    assert!(!can_block(&mut t, giant));
}

/// P0's Braided Net, with its net counters, ready to tap `target`.
fn net_taps(t: &mut TestGame, target: ObjectId) -> usize {
    let net = t.enter(P0, "Braided Net // Braided Quipu");
    let net = t.g.current(net);
    t.g.objects[net.0 as usize].summoning_sick = false;
    assert_eq!(t.counters(net, "net"), 3);
    let from = t.asked().len();
    t.activate(P0, net, 0, &[obj(target)]).expect("activate");
    t.resolve_all();
    assert_eq!(t.counters(net, "net"), 2);
    from
}

#[test]
fn braided_net_can_target_a_tapped_permanent() {
    cr!("115.1", "701.26a");
    ruling!(
        "Braided Net // Braided Quipu",
        "Braided Net's activated ability can target another nonland permanent that's already tapped."
    );
    supported("Braided Net // Braided Quipu");
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    let s = t.g.current(sorcerer);
    t.g.tap(s);
    let from = net_taps(&mut t, sorcerer);
    assert!(target_candidates(&t, P0, from)[0].contains(&obj(sorcerer)));
    assert!(t.obj_now(sorcerer).tapped);
    assert!(crate::r_s27_common::activatable(&mut t, P1, sorcerer).is_empty());
}

#[test]
fn braided_net_stops_untap_cost_abilities_until_it_untaps() {
    cr!("611.2b", "602.5", "107.6");
    ruling!(
        "Braided Net // Braided Quipu",
        "No activated abilities of the target nonland permanent can be activated for as long as it remains tapped, even if those activated abilities' costs involve untapping that permanent."
    );
    supported("Pili-Pala");
    let mut t = TestGame::new(2);
    let pili = t.battlefield(P1, "Pili-Pala");
    t.lands(P1, "Wastes", 4);
    net_taps(&mut t, pili);
    assert!(t.obj_now(pili).tapped);
    // "{2}, {Q}: Add one mana of any color." can't be activated while it stays tapped.
    assert!(crate::r_s27_common::activatable(&mut t, P1, pili).is_empty());
    // Once it untaps, the effect is over, even after it's tapped again.
    let p = t.g.current(pili);
    t.g.untap(p);
    t.g.flush_events();
    t.settle();
    t.g.tap(p);
    t.g.flush_events();
    t.settle();
    assert!(!crate::r_s27_common::activatable(&mut t, P1, pili).is_empty());
}

#[test]
fn rhuk_moves_all_or_none_of_the_equipment_when_it_attacks_or_dies() {
    cr!("603.10a", "701.3a");
    ruling!(
        "Rhuk, Hexgold Nabber",
        "If that creature has more than one Equipment attached to it, you may move all of those Equipment to Rhuk or none of those Equipment to Rhuk. You can't move some of them and leave others."
    );
    supported("Rhuk, Hexgold Nabber");
    for (dies, yes) in [(false, true), (true, true), (false, false), (true, false)] {
        let mut t = TestGame::new(2);
        let rhuk = t.battlefield(P0, "Rhuk, Hexgold Nabber");
        let bears = t.battlefield(P0, "Grizzly Bears");
        let a = crate::r_s06_common::attach_new(&mut t, P0, "Bonesplitter", bears);
        let b = crate::r_s06_common::attach_new(&mut t, P0, "Short Sword", bears);
        t.answer_yes(P0, yes);
        if dies {
            destroy(&mut t, bears);
        } else {
            attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        }
        t.resolve_all();
        let on = |t: &TestGame, e: ObjectId| crate::r_s06_common::attached_to(t, e);
        let want = if yes {
            Some(obj(rhuk))
        } else if dies {
            None
        } else {
            Some(obj(bears))
        };
        assert_eq!((on(&t, a), on(&t, b)), (want, want), "dies {dies}, yes {yes}");
    }
}
