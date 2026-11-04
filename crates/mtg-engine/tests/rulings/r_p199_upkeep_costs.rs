//! Rulings batch P199 — upkeep costs: "At the beginning of your upkeep, sacrifice this
//! unless you [pay / sacrifice / return / exile] ...", cumulative upkeep with unusual
//! costs, and other upkeep triggers. The choices are made as the trigger resolves (CR
//! 608.2), the amounts are determined then (CR 608.2h), and "unless" costs can be declined
//! even when the permanent is gone (CR 118.12).

use crate::r_s01_common::*;
use crate::r_s03_common::choice_candidates;
use crate::r_s04_common::*;
use crate::r_s06_common::give_control;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn pay_questions(t: &TestGame, p: PlayerId, from: usize) -> usize {
    asked_of_since(
        t,
        p,
        from,
        |d| matches!(d, Decision::YesNo { prompt, .. } if prompt.starts_with("Pay")),
    )
}

fn yes_no_questions(t: &TestGame, p: PlayerId, from: usize) -> usize {
    asked_of_since(t, p, from, |d| matches!(d, Decision::YesNo { .. }))
}

#[test]
fn fettergeist_counts_the_other_creatures_as_the_trigger_resolves() {
    cr!("608.2h", "118.12");
    ruling!(
        "Fettergeist",
        "The amount of mana you need to pay is determined when the triggered ability resolves."
    );
    supported("Fettergeist");
    let mut t = TestGame::new(2);
    let geist = t.battlefield(P0, "Fettergeist");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 4);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "unless you pay"), 1);
    // A second other creature arrives with the trigger on the stack: {2}, not {1}.
    t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.on_battlefield(geist));
    assert_eq!(untapped_lands(&t, P0), 2);
}

/// Loads coin results (true = heads).
fn load(t: &mut TestGame, heads: &[bool]) {
    t.g.dice.loaded_coins.extend(heads.iter().copied());
}

#[test]
fn karplusan_minotaur_wins_only_flips_its_controller_flipped() {
    cr!("705.2", "702.24a");
    ruling!(
        "Karplusan Minotaur",
        "The coin flip rules have changed. You now win or lose a flip only if *you* flipped the coin."
    );
    supported("Stitch in Time");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Karplusan Minotaur");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    // P1 flips and loses (calls tails, heads comes up): P0 doesn't win that flip.
    t.lands(P1, "Island", 2);
    t.lands(P1, "Mountain", 2);
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    load(&mut t, &[true]);
    let stitch = t.hand(P1, "Stitch in Time");
    t.cast(P1, stitch).go();
    t.resolve_all();
    let lost: Vec<_> = t
        .turn_events
        .iter()
        .filter(|e| matches!(e, Event::CoinFlipped { player, lost: true, .. } if *player == P1))
        .collect();
    assert_eq!(lost.len(), 1);
    assert_eq!(triggers_on_stack(&t, "win a coin flip"), 0);
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // P0 flips for the cumulative upkeep and wins (calls heads, heads): the win trigger.
    let mut t = TestGame::new(2);
    let mino = t.battlefield(P0, "Karplusan Minotaur");
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    load(&mut t, &[true]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.on_battlefield(mino));
    assert_eq!(t.life(P1), 19);
}

#[test]
fn justice_deals_its_damage_with_a_trigger_after_the_red_source() {
    cr!("603.2", "603.3");
    ruling!(
        "Justice",
        "The damage done by Justice is done as a triggered ability, which is added to the stack just after the red creature or spell deals its damage."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Justice");
    let shock = t.hand(P1, "Shock");
    t.lands(P1, "Mountain", 1);
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    t.resolve();
    // Shock has dealt its damage; Justice's ability is now on the stack.
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn yawgmoth_demon_may_keep_its_artifacts() {
    cr!("603.5", "608.2c");
    ruling!(
        "Yawgmoth Demon",
        "The sacrificing of an artifact is not mandatory. You can choose not to sacrifice an artifact but will pay the consequences."
    );
    supported("Yawgmoth Demon");
    let mut t = TestGame::new(2);
    let demon = t.battlefield(P0, "Yawgmoth Demon");
    let rock = t.battlefield(P0, "Mind Stone");
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.on_battlefield(rock));
    assert!(t.obj_now(demon).tapped);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn ritual_of_subdual_changes_the_type_of_mana_but_not_the_amount() {
    cr!("106.12", "614.1a");
    ruling!(
        "Ritual of Subdual",
        "This effect changes the type of mana produced, but not the amount."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ritual of Subdual");
    // Boros Garrison would add {R}{W}: it adds {C}{C}.
    let garrison = t.battlefield(P0, "Boros Garrison");
    t.activate(P0, garrison, 0, &[]).unwrap();
    let pool = &t.g.player(P0).mana_pool;
    assert_eq!(pool.total(), 2);
    assert_eq!(pool.count(ManaType::C), 2);
}

#[test]
fn xathrid_demon_must_sacrifice_another_creature_if_it_can() {
    cr!("701.21a", "608.2c");
    ruling!(
        "Xathrid Demon",
        "When the ability resolves, if you control any other creatures, you must sacrifice one of them. You lose life only if no creature can be sacrificed."
    );
    // Another creature: it's sacrificed and P1 loses 2 life; P0 keeps their life.
    let mut t = TestGame::new(2);
    let demon = t.battlefield(P0, "Xathrid Demon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert_eq!((t.life(P0), t.life(P1)), (20, 18));
    assert!(!t.obj_now(demon).tapped);
    // No other creature: tapped and 7 life lost.
    let mut t = TestGame::new(2);
    let demon = t.battlefield(P0, "Xathrid Demon");
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.on_battlefield(demon));
    assert!(t.obj_now(demon).tapped);
    assert_eq!((t.life(P0), t.life(P1)), (13, 20));
}

#[test]
fn circling_vultures_is_sacrificed_unless_a_creature_card_is_exiled() {
    cr!("118.12", "608.2c");
    ruling!(
        "Circling Vultures",
        "When the ability resolves, you choose whether to sacrifice the creature or perform the other action. If you can't perform the other action, then you must sacrifice the creature."
    );
    for (in_gy, exile, stays) in [(true, true, true), (true, false, false), (false, true, false)] {
        let mut t = TestGame::new(2);
        let v = t.battlefield(P0, "Circling Vultures");
        let bears = in_gy.then(|| t.graveyard(P0, "Grizzly Bears"));
        next_upkeep(&mut t, P0);
        let from = t.asked().len();
        t.answer_yes(P0, exile);
        t.resolve_all();
        assert_eq!(t.on_battlefield(v), stays, "{in_gy} {exile}");
        if let Some(b) = bears {
            assert_eq!(t.zone(b) == Zone::Exile, stays);
        } else {
            // Nothing to exile: not even asked.
            assert_eq!(yes_no_questions(&t, P0, from), 0);
        }
    }
}

#[test]
fn wayward_angel_with_threshold_is_only_black() {
    cr!("105.3", "613.1e");
    ruling!(
        "Wayward Angel",
        "When you have seven or more cards in your graveyard, this card is black. It is not white and black."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Wayward Angel");
    assert_eq!(t.obj_now(angel).chars.colors, ColorSet::single(Color::White));
    for _ in 0..7 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.g.recompute();
    assert_eq!(t.obj_now(angel).chars.colors, ColorSet::single(Color::Black));
    assert_eq!(t.pt(angel), (7, 7));
}

#[test]
fn charnel_troll_can_discard_a_creature_card_before_its_upkeep_trigger_resolves() {
    cr!("117.3a", "608.2c");
    ruling!(
        "Charnel Troll",
        "You can activate Charnel Troll’s last ability during your upkeep before its triggered ability resolves."
    );
    ruling!(
        "Charnel Troll",
        "You can’t choose not to exile a creature card from your graveyard if you have one to exile."
    );
    supported("Charnel Troll");
    let mut t = TestGame::new(2);
    let troll = t.battlefield(P0, "Charnel Troll");
    let bears = t.hand(P0, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    // In response, discard the Bears: then the trigger has a creature card to exile.
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, troll, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.counters(troll, counters::PLUS1), 1);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    let from = t.asked().len();
    t.resolve_all();
    // The Bears were exiled without a choice to keep them, and the Troll stays.
    assert_eq!(yes_no_questions(&t, P0, from), 0);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.on_battlefield(troll));
    assert_eq!(t.counters(troll, counters::PLUS1), 2);
}

#[test]
fn liege_of_the_pit_cant_sacrifice_itself_but_can_sacrifice_another_liege() {
    cr!("109.5", "701.21a");
    ruling!(
        "Liege of the Pit",
        "You can’t sacrifice Liege of the Pit to its own ability. You can sacrifice one Liege of the Pit to another Liege of the Pit."
    );
    // Alone: it can't be sacrificed to its own ability, so it deals 7 damage to P0.
    let mut t = TestGame::new(2);
    let liege = t.battlefield(P0, "Liege of the Pit");
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.on_battlefield(liege));
    assert_eq!(t.life(P0), 13);
    // Two: the first trigger to resolve may sacrifice the other Liege.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Liege of the Pit");
    let b = t.battlefield(P0, "Liege of the Pit");
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 2);
    let from = t.asked().len();
    let top = top_of_stack(&t);
    let src = match &t.g.obj(top).stack.as_ref().unwrap().kind {
        mtg_engine::object::StackKind::Triggered { source, .. } => Some(*source),
        _ => None,
    };
    let other = if src == Some(a) { b } else { a };
    t.answer_choose(P0, &[obj(other)]);
    t.resolve();
    assert!(!t.on_battlefield(other));
    let cands = choice_candidates(&t, from, "");
    assert!(cands.iter().all(|c| !c.contains(&obj(src.unwrap()))));
    assert!(t.on_battlefield(src.unwrap()));
}

#[test]
fn two_stampeding_serows_each_return_a_creature() {
    cr!("608.2c", "603.3b");
    ruling!(
        "Stampeding Serow",
        "You choose a creature and return it all during resolution of the triggered ability. This means that if you control two of these, there is no way to end up returning only one creature."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Stampeding Serow");
    let b = t.battlefield(P0, "Stampeding Serow");
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Hand(P0));
    assert_eq!(t.zone(b), Zone::Hand(P0));
}

#[test]
fn two_waterspout_djinns_must_return_different_islands() {
    cr!("608.2c", "118.12");
    ruling!(
        "Waterspout Djinn",
        "You choose a land to return and return it during resolution of the upkeep triggered ability. This means if you have two Waterspout Djinns, they must each return different Islands."
    );
    supported("Waterspout Djinn");
    for (islands, kept) in [(2, 2), (1, 1)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Waterspout Djinn");
        t.battlefield(P0, "Waterspout Djinn");
        t.lands(P0, "Island", islands);
        next_upkeep(&mut t, P0);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Waterspout Djinn").len(), kept);
        assert_eq!(t.named_on_battlefield("Island").len(), 0);
        assert_eq!(hand_names(&t, P0).iter().filter(|n| *n == "Island").count(), islands);
    }
}

/// The upkeep trigger of `name` is on the stack when its controller loses control of it;
/// as it resolves, the old controller is still asked whether to pay `name`'s cost, and
/// declining it doesn't sacrifice the permanent (they don't control it).
fn declines_after_losing_control(name: &str, setup: fn(&mut TestGame)) {
    let mut t = TestGame::new(2);
    let id = t.battlefield(P0, name);
    setup(&mut t);
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1, "{name}");
    give_control(&mut t, id, P1);
    let from = t.asked().len();
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(yes_no_questions(&t, P0, from) >= 1, "{name}: asked");
    assert!(t.on_battlefield(id), "{name}");
    assert_eq!(t.obj_now(id).controller, P1, "{name}");
}

#[test]
fn unless_costs_are_chosen_on_resolution_even_without_control() {
    cr!("118.12", "608.2c");
    ruling!(
        "Child of Gaea",
        "You choose whether to pay or not on resolution. If not, then you sacrifice Child of Gaea. You can choose to not pay even if you no longer control Child of Gaea on resolution."
    );
    ruling!(
        "Drifting Djinn",
        "You choose whether to pay or not on resolution. If not, then you sacrifice Drifting Djinn. You can choose to not pay if you no longer control this card on resolution."
    );
    ruling!(
        "Contamination",
        "You choose whether to sacrifice a creature or not on resolution. If not, then you sacrifice Contamination. You can choose to not sacrifice even if you no longer control Contamination on resolution."
    );
    ruling!(
        "Endless Wurm",
        "You choose whether to sacrifice an enchantment or not on resolution. If not, then you sacrifice this card. You can choose to not sacrifice an enchantment if you no longer control this card on resolution."
    );
    declines_after_losing_control("Child of Gaea", |t| {
        t.lands(P0, "Forest", 2);
    });
    declines_after_losing_control("Drifting Djinn", |t| {
        t.lands(P0, "Island", 2);
    });
    declines_after_losing_control("Contamination", |t| {
        t.battlefield(P0, "Grizzly Bears");
    });
    declines_after_losing_control("Endless Wurm", |t| {
        t.battlefield(P0, "Glorious Anthem");
    });
    // While still controlled, declining sacrifices it; paying keeps it.
    for (name, land) in [("Child of Gaea", "Forest"), ("Drifting Djinn", "Island")] {
        for pay in [true, false] {
            let mut t = TestGame::new(2);
            let id = t.battlefield(P0, name);
            t.lands(P0, land, 2);
            next_upkeep(&mut t, P0);
            let from = t.asked().len();
            t.answer_yes(P0, pay);
            t.resolve_all();
            assert_eq!(pay_questions(&t, P0, from), 1);
            assert_eq!(t.on_battlefield(id), pay, "{name} {pay}");
        }
    }
    for (name, fodder) in [("Contamination", "Grizzly Bears"), ("Endless Wurm", "Glorious Anthem")] {
        for pay in [true, false] {
            let mut t = TestGame::new(2);
            let id = t.battlefield(P0, name);
            let f = t.battlefield(P0, fodder);
            next_upkeep(&mut t, P0);
            t.answer_yes(P0, pay);
            t.resolve_all();
            assert_eq!(t.on_battlefield(id), pay, "{name} {pay}");
            assert_eq!(t.on_battlefield(f), !pay, "{name} {pay}");
        }
    }
}

#[test]
fn smothering_abomination_chooses_the_sacrifice_as_its_trigger_resolves() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Smothering Abomination",
        "You choose which creature you control to sacrifice as the first triggered ability resolves. Players can respond to this ability triggering, but once you choose a creature to sacrifice, it's too late for anyone to respond."
    );
    supported("Smothering Abomination");
    let mut t = TestGame::new(2);
    let abom = t.battlefield(P0, "Smothering Abomination");
    next_upkeep(&mut t, P0);
    assert_eq!(t.stack_len(), 1);
    // A creature that arrives in response can be the one sacrificed.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert!(t.on_battlefield(abom));
    // The draw trigger follows.
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1);
}

#[test]
fn tombstone_stairwell_tokens_are_controlled_by_each_player() {
    cr!("603.3a", "111.2");
    ruling!(
        "Tombstone Stairwell",
        "You control the triggered ability, but each player controls the token creature they put onto the battlefield due to the ability's effect."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tombstone Stairwell");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P1, "Grizzly Bears");
    // P1's upkeep: only the token ability (controlled by P0) triggers.
    t.set_step(P0, mtg_engine::turn::Step::End);
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let top = top_of_stack(&t);
    assert_eq!(t.g.obj(top).controller, P0);
    t.resolve_all();
    let spawn = t.named_on_battlefield("Tombspawn");
    assert_eq!(spawn.len(), 2);
    let mut ctl: Vec<_> = spawn.iter().map(|id| t.g.obj(*id).controller).collect();
    ctl.sort();
    assert_eq!(ctl, vec![P0, P1]);
}

#[test]
fn sheltering_ancient_counters_may_go_on_any_opponents_creatures() {
    cr!("702.24a", "118.3");
    ruling!(
        "Sheltering Ancient",
        "You may choose a different creature to put each counter on, but you can also choose the same creature multiple times. The creatures don’t have to be controlled by the same opponent."
    );
    let mut t = TestGame::new(3);
    let anc = t.battlefield(P0, "Sheltering Ancient");
    t.g.add_counters(obj(anc), "age", 2, None);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P2, "Grizzly Bears");
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a)]);
    t.answer_choose(P0, &[obj(a)]);
    t.answer_choose(P0, &[obj(b)]);
    t.resolve_all();
    assert!(t.on_battlefield(anc));
    assert_eq!(t.counters(a, counters::PLUS1), 2);
    assert_eq!(t.counters(b, counters::PLUS1), 1);
}

#[test]
fn forsaken_city_may_exile_a_card_while_untapped() {
    cr!("603.5", "502.3");
    ruling!(
        "Forsaken City",
        "You may choose to exile a card in your hand even if Forsaken City is already untapped."
    );
    supported("Forsaken City");
    let mut t = TestGame::new(2);
    let city = t.battlefield(P0, "Forsaken City");
    let bears = t.hand(P0, "Grizzly Bears");
    next_upkeep(&mut t, P0);
    assert!(!t.obj_now(city).tapped);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(!t.obj_now(city).tapped);
}

#[test]
fn herald_of_leshrac_takes_a_different_land_for_each_age_counter() {
    cr!("702.24a", "118.3");
    ruling!(
        "Herald of Leshrac",
        "You must choose a different land you don't control for each age counter on Herald of Leshrac."
    );
    // Two age counters and two lands to take: both are taken.
    let mut t = TestGame::new(2);
    let herald = t.battlefield(P0, "Herald of Leshrac");
    t.g.add_counters(obj(herald), "age", 1, None);
    let lands = t.lands(P1, "Mountain", 2);
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(herald));
    for l in &lands {
        assert_eq!(t.obj_now(*l).controller, P0);
    }
    assert_eq!(t.pt(herald), (4, 6));
    // Two age counters but only one land P0 doesn't control: it can't be paid.
    let mut t = TestGame::new(2);
    let herald = t.battlefield(P0, "Herald of Leshrac");
    t.g.add_counters(obj(herald), "age", 1, None);
    let land = t.battlefield(P1, "Mountain");
    next_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.on_battlefield(herald));
    assert_eq!(t.obj_now(land).controller, P1);
}
