//! Trigger events (cards3-triggers-durations): events caused by a spell or ability
//! (CR 603.2, 608.2), exact amounts, untap-step untaps, objects and players linked to the
//! source (CR 607.2a), and the other trigger events of `trigger_grammar_4.rs`.

use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s25_common::cast_new;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn psychogenic_probe_shuffles_caused_by_abilities() {
    cr!("603.2", "701.24a");
    ruling!(
        "Psychogenic Probe",
        "This ability will trigger even if the player’s library is empty at the time they are supposed to shuffle it."
    );
    supported("Psychogenic Probe");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Psychogenic Probe");
    // An activated ability's shuffle (P0's library is empty).
    let elder = t.battlefield(P0, "Sakura-Tribe Elder");
    t.activate(P0, elder, 0, &[]).expect("activate");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    // A shuffle no spell or ability performs (a mulligan's, say) doesn't trigger it.
    t.g.shuffle_library(P0);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), 18);
}

#[test]
fn widespread_panic_only_its_controllers_shuffles() {
    cr!("603.2");
    ruling!(
        "Widespread Panic",
        "Widespread Panic’s ability is put onto the stack after the library is shuffled."
    );
    supported("Widespread Panic");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Widespread Panic");
    // P0's Path to Exile makes P1 search and shuffle: not its controller's.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.library_top(P1, "Forest");
    t.answer_yes(P1, true);
    cast_new(&mut t, P0, "Path to Exile", &[Entity::Object(bears)]);
    t.resolve();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "top of their library"), 0);
    // P1's own ability makes P1 shuffle: P1 puts a card from hand on top.
    t.hand(P1, "Island");
    let elder = t.battlefield(P1, "Sakura-Tribe Elder");
    t.activate(P1, elder, 0, &[]).expect("activate");
    t.settle();
    t.resolve();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "top of their library"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 0);
}

#[test]
fn sacred_ground_returns_lands_an_opponent_destroys() {
    cr!("603.2", "701.8a");
    supported("Sacred Ground");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Sacred Ground");
    t.battlefield(P0, "Forest");
    let forest = t.named_on_battlefield("Forest")[0];
    cast_new(&mut t, P1, "Stone Rain", &[Entity::Object(forest)]);
    t.resolve();
    t.settle();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Forest").len(), 1);
    // P0's own spell: it stays in the graveyard.
    t.set_step(P0, Step::PrecombatMain);
    let forest = t.named_on_battlefield("Forest")[0];
    cast_new(&mut t, P0, "Stone Rain", &[Entity::Object(forest)]);
    t.resolve();
    t.settle();
    t.resolve_all();
    assert!(t.named_on_battlefield("Forest").is_empty());
    assert!(t.in_graveyard(P0, "Forest"));
}

#[test]
fn firesong_and_sunspeaker_white_spell_life_gain() {
    cr!("603.2", "119.3");
    ruling!(
        "Firesong and Sunspeaker",
        "Firesong and Sunspeaker's last ability doesn't trigger if a triggered ability of a white instant or sorcery spell or card causes you to gain life, such as the triggered ability of Renewed Faith when it's cycled."
    );
    supported("Firesong and Sunspeaker");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Firesong and Sunspeaker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    cast_new(&mut t, P0, "Renewed Faith", &[]);
    t.resolve();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "3 damage"), 1);
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Cycling it: its triggered ability's life gain doesn't trigger Firesong.
    let faith = t.hand(P0, "Renewed Faith");
    t.lands(P0, "Plains", 2);
    t.library_top(P0, "Island");
    t.answer_yes(P0, true);
    t.activate(P0, faith, 0, &[]).expect("cycle");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 28);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn ghyrson_starn_exactly_one_damage() {
    cr!("603.2", "603.2c");
    ruling!(
        "Ghyrson Starn, Kelermorph",
        "If another source deals exactly 1 damage to a permanent or player multiple times or deals exactly 1 damage to multiple permanents and/or players simultaneously"
    );
    supported("Ghyrson Starn, Kelermorph");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ghyrson Starn, Kelermorph");
    // "{T}: This creature deals 1 damage to any target."
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).expect("ping");
    t.settle();
    t.resolve_all();
    // 1 from the Sorcerer, 2 from Ghyrson (whose own 2 damage doesn't trigger it).
    assert_eq!(t.life(P1), 17);
    // A permanent: Ghyrson deals its 2 damage to that creature.
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.obj_mut(sorcerer).tapped = false;
    t.activate(P0, sorcerer, 0, &[Entity::Object(giant)]).expect("ping");
    t.settle();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P1), 17);
    // Not exactly 1: no trigger.
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn ob_nixilis_opponents_each_lose_exactly_one() {
    cr!("603.2c", "119.3");
    ruling!(
        "Ob Nixilis, Captive Kingpin",
        "Ob Nixilis's last ability will trigger if an opponent is dealt exactly 1 damage, pays exactly 1 life, or loses exactly 1 life some other way."
    );
    supported("Ob Nixilis, Captive Kingpin");
    let mut t = TestGame::new(3);
    let ob = t.battlefield(P0, "Ob Nixilis, Captive Kingpin");
    for _ in 0..3 {
        t.library_top(P0, "Island");
    }
    // "Whenever this creature or another creature you control dies, each opponent loses
    // 1 life and you gain 1 life.": both opponents lose exactly 1 at once: once.
    t.battlefield(P0, "Zulaport Cutthroat");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s02_common::destroy(&mut t, bears);
    t.resolve();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "+1/+1 counter"), 1);
    t.resolve_all();
    assert_eq!(t.counters(ob, "+1/+1"), 1);
    // 3 life at once: no trigger.
    t.set_step(P0, Step::PrecombatMain);
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.counters(ob, "+1/+1"), 1);
}

#[test]
fn millennium_calendar_counts_permanents_untapped_in_your_untap_step() {
    cr!("502.3", "603.2c");
    ruling!(
        "The Millennium Calendar",
        "The Millenium Calendar's first ability triggers during the untap step."
    );
    supported("The Millennium Calendar");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::End);
    let cal = t.battlefield(P0, "The Millennium Calendar");
    for l in t.lands(P0, "Forest", 3) {
        t.g.obj_mut(l).tapped = true;
    }
    // An opponent's tapped permanent isn't untapped by you.
    let theirs = t.battlefield(P1, "Forest");
    t.g.obj_mut(theirs).tapped = true;
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(cal, "time"), 3);
    // Untapping outside the untap step doesn't trigger it.
    let land = t.named_on_battlefield("Forest")[0];
    t.g.obj_mut(land).tapped = true;
    t.g.untap(land);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn ninth_doctor_additional_upkeep_after_the_upkeep() {
    cr!("502.3", "500.9");
    ruling!(
        "The Ninth Doctor",
        "The additional upkeep is taken after that step, not after the untap step."
    );
    supported("The Ninth Doctor");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::End);
    let doc = t.battlefield(P0, "The Ninth Doctor");
    t.g.obj_mut(doc).tapped = true;
    t.advance_to(P0, Step::Draw);
    // The trigger resolved in the upkeep: a second upkeep followed it, in the same turn.
    assert_eq!(t.g.turn.upkeeps, 2);
    assert_eq!(t.g.turn.step_log.iter().filter(|s| **s == Step::Upkeep).count(), 2);
    // An untapped Doctor doesn't become untapped: one upkeep the next turn.
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.g.turn.upkeeps, 1);
}

#[test]
fn diabolic_servitude_tracks_the_creature_it_returned() {
    cr!("607.2a", "603.10a", "400.7");
    supported("Diabolic Servitude");
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.enter(P0, "Diabolic Servitude");
    t.settle();
    t.resolve_all();
    let returned = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(returned.len(), 1);
    // Another creature dying doesn't trigger it.
    let other = t.battlefield(P0, "Hill Giant");
    crate::r_s02_common::destroy(&mut t, other);
    assert_eq!(t.stack_len(), 0);
    // The returned creature dies: it's exiled and Servitude returns to its owner's hand.
    crate::r_s02_common::destroy(&mut t, returned[0]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_hand(P0, "Diabolic Servitude"));
}

#[test]
fn diabolic_servitude_leaving_exiles_the_creature() {
    cr!("607.2a");
    ruling!(
        "Diabolic Servitude",
        "If the creature is on the battlefield and is returned to its owner’s hand or is exiled, this card loses track of the creature and has no further effects in the future."
    );
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let servitude = t.enter(P0, "Diabolic Servitude");
    t.settle();
    t.resolve_all();
    crate::r_s02_common::destroy(&mut t, servitude);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // Returned to hand and recast: a new object Servitude doesn't track.
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    let servitude = t.enter(P0, "Diabolic Servitude");
    t.settle();
    t.resolve_all();
    let g = t.named_on_battlefield("Hill Giant")[0];
    let c = t.g.current(g);
    t.g.move_object(c, mtg_engine::object::Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None);
    let g2 = t.hand(P0, "Hill Giant");
    let _ = g2;
    t.settle();
    crate::r_s02_common::destroy(&mut t, servitude);
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
}

#[test]
fn havengul_mystery_transforms_back_when_its_creature_leaves() {
    cr!("607.2a", "701.27a");
    supported("Havengul Laboratory");
    let mut t = TestGame::new(2);
    let lab = t.battlefield(P0, "Havengul Laboratory");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    assert!(mtg_engine::dfc::transform(&mut t.g, lab));
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(lab).chars.name.as_str(), "Havengul Mystery");
    let returned = t.named_on_battlefield("Grizzly Bears")[0];
    // It leaves the battlefield (bounced): Havengul Mystery transforms.
    let c = t.g.current(returned);
    t.g.move_object(c, mtg_engine::object::Zone::Hand(P0), mtg_engine::events::MoveCause::Effect, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(lab).chars.name.as_str(), "Havengul Laboratory");
}

#[test]
fn voltaic_visionary_transforms_when_you_play_the_exiled_card() {
    cr!("607.2a", "305.1");
    ruling!(
        "Voltaic Visionary",
        "Playing a card this way follows all the normal timing rules for that card."
    );
    supported("Voltaic Visionary");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let v = t.battlefield(P0, "Voltaic Visionary");
    let top = t.library_top(P0, "Forest");
    t.activate(P0, v, 0, &[]).expect("activate");
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    let exiled = t.g.current(top);
    assert_eq!(t.zone(exiled), mtg_engine::object::Zone::Exile);
    t.play_land(P0, exiled).expect("play the exiled land");
    t.settle();
    t.resolve_all();
    assert_eq!(t.obj_now(v).chars.name.as_str(), "Volt-Charged Berserker");
}

#[test]
fn voltaic_visionary_not_for_other_cards() {
    cr!("607.2a");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let v = t.battlefield(P0, "Voltaic Visionary");
    let island = t.hand(P0, "Island");
    t.play_land(P0, island).expect("play");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.obj_now(v).chars.name.as_str(), "Voltaic Visionary");
}

#[test]
fn prowl_draws_and_converts_when_the_exiled_card_is_played() {
    cr!("607.2a", "601.2");
    ruling!(
        "Prowl, Stoic Strategist",
        "Players must pay all costs and follow all normal timing rules to play cards they own that were exiled by Prowl, Stoic Strategist."
    );
    supported("Prowl, Stoic Strategist");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let prowl = t.battlefield(P0, "Prowl, Stoic Strategist");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.obj_mut(bears).tapped = true;
    t.library_top(P0, "Island");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.attack(&[(prowl, Entity::Player(P1))], &[]);
    t.resolve_all();
    let exiled = t.g.current(bears);
    assert_eq!(t.zone(exiled), mtg_engine::object::Zone::Exile);
    // P1 casts it on their turn.
    let hand = t.hand_size(P0);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Forest", 2);
    t.cast(P1, exiled).go();
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.obj_now(prowl).chars.name.as_str(), "Prowl, Pursuit Vehicle");
}

#[test]
fn war_doctor_phasing_and_exile() {
    cr!("603.2c", "702.26b");
    interp!("batched-alternatives-one-batch");
    ruling!(
        "The War Doctor",
        "If The War Doctor phases out at the same time as any other permanents, its first ability won't trigger."
    );
    supported("The War Doctor");
    let mut t = TestGame::new(2);
    let doc = t.battlefield(P0, "The War Doctor");
    // Two permanents phasing out at once: one trigger.
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![a, b]);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(doc, "time"), 1);
    // Cards put into exile from a graveyard.
    let c = t.graveyard(P1, "Island");
    t.g.move_object(c, mtg_engine::object::Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(doc, "time"), 2);
    // Both at the same time: one trigger.
    let d = t.battlefield(P1, "Forest");
    let e = t.graveyard(P1, "Island");
    t.g.batch_hold += 1;
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![d]);
    t.g.move_object(e, mtg_engine::object::Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    t.g.batch_hold -= 1;
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(doc, "time"), 3);
    // The Doctor phasing out with another permanent: no trigger.
    let f = t.battlefield(P1, "Mountain");
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![doc, f]);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn rendmaw_card_with_two_or_more_card_types() {
    cr!("603.2", "205.2a");
    ruling!(
        "Rendmaw, Creaking Nest",
        "The card types in Magic include artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery."
    );
    supported("Rendmaw, Creaking Nest");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Rendmaw, Creaking Nest");
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    cast_new(&mut t, P0, "Ornithopter", &[]);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Bird"), 1);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
    assert_eq!(crate::r_s01_common::tokens(&t, P1).len(), 1);
    // A land with two card types played: it triggers too.
    let arbor = t.hand(P0, "Dryad Arbor");
    t.play_land(P0, arbor).expect("play");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Bird"), 1);
}

#[test]
fn historians_boon_final_chapter_triggers() {
    cr!("714.2c", "603.3b");
    ruling!(
        "Historian's Boon",
        "The final chapter ability of a Saga is the ability with the greatest chapter number among chapter abilities that Saga has."
    );
    supported("Historian's Boon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Historian's Boon");
    // History of Benalia: I, II — a Knight token; III — Knights get +2/+1.
    let saga = t.battlefield(P0, "History of Benalia");
    t.g.obj_mut(saga).counters.insert("lore".into(), 1);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    // Chapter II: no Angel.
    assert_eq!(t.named_on_battlefield("Angel").len(), 0);
    assert_eq!(t.counters(saga, "lore"), 2);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
    let angels = crate::r_s01_common::with_subtype(&t, P0, "Angel");
    assert_eq!(angels.len(), 1);
    assert_eq!(t.pt(angels[0]), (4, 4));
}

#[test]
fn watcher_of_hours_surveils_as_time_counters_are_removed_in_exile() {
    cr!("702.62a", "113.6b");
    ruling!(
        "Watcher of Hours",
        "Suspend is a keyword that represents three abilities."
    );
    supported("Watcher of Hours");
    let surveils = |t: &TestGame| {
        t.asked()
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::Surveil { .. }))
            .count()
    };
    let mut t = TestGame::new(2);
    for _ in 0..4 {
        t.library_top(P0, "Island");
    }
    let w = t.exile(P0, "Watcher of Hours");
    t.g.obj_mut(w).counters.insert("time".into(), 6);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Draw);
    assert_eq!(t.counters(w, "time"), 5);
    assert_eq!(surveils(&t), 1);
    // On the battlefield, removing a time counter doesn't trigger it.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Island");
    let w = t.battlefield(P0, "Watcher of Hours");
    t.g.obj_mut(w).counters.insert("time".into(), 2);
    t.g.remove_counters_by(Entity::Object(w), "time", 1, Some(P0));
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn risky_move_gaining_control_from_another_player() {
    cr!("705.2", "108.4");
    ruling!("Risky Move", "This card does not target any creatures or players.");
    supported("Risky Move");
    for opp in [P0, P2] {
        let mut t = TestGame::new(3);
        let rm = t.battlefield(P0, "Risky Move");
        let bears = t.battlefield(P1, "Grizzly Bears");
        // P1 gains control of it in their upkeep, chooses the Bears and that opponent, and
        // loses the flip.
        t.answer_choose(P1, &[Entity::Object(bears)]);
        t.answer_choose(P1, &[Entity::Player(opp)]);
        t.g.dice.loaded_coins.push_back(false);
        t.set_step(P0, Step::End);
        t.advance_to(P1, Step::Draw);
        t.resolve_all();
        assert_eq!(t.obj_now(rm).controller, P1);
        assert_eq!(t.obj_now(bears).controller, opp);
    }
}

#[test]
fn infinite_reflection_enters_attached_to_a_creature() {
    cr!("303.4", "707.2");
    ruling!(
        "Infinite Reflection",
        "If Infinite Reflection enters attached to a creature and then becomes attached to a different creature before its \"enters\" trigger resolves, each other nontoken creature you control will become a copy of the first creature."
    );
    supported("Infinite Reflection");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P1, "Serra Angel");
    cast_new(&mut t, P0, "Infinite Reflection", &[Entity::Object(giant)]);
    t.resolve();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "becomes a copy"), 1);
    // Moved to another creature before the trigger resolves: still a copy of the Giant.
    let aura = t.named_on_battlefield("Infinite Reflection")[0];
    assert!(t.g.attach(aura, Entity::Object(other)));
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.obj_now(giant).chars.name.as_str(), "Hill Giant");
}

#[test]
fn teamwork_triggers_virtual_assistant_and_maria_hill() {
    cr!("702.194a", "702.194b");
    supported("Agent Maria Hill");
    supported("Virtual Assistant");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 8);
    t.library_top(P0, "Island");
    let hill = t.battlefield(P0, "Agent Maria Hill");
    t.battlefield(P0, "Virtual Assistant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let power = t.pt(hill).0;
    assert!(power >= 2);
    let hand = t.hand_size(P0);
    // Repulsor Blast (teamwork 2) cast using teamwork, tapping Agent Maria Hill.
    t.answer_choose(P0, &[Entity::Object(hill)]);
    let c = t.hand(P0, "Repulsor Blast");
    t.cast(P0, c).kicked(true).target(wurm).go();
    assert!(t.obj_now(hill).tapped);
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(hill, "+1/+1"), 1);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Robot").len(), 1);
    // Without teamwork: neither triggers.
    let wurm = t.battlefield(P1, "Craw Wurm");
    let c = t.hand(P0, "Repulsor Blast");
    t.cast(P0, c).kicked(false).target(wurm).go();
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(hill, "+1/+1"), 1);
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Robot").len(), 1);
}

#[test]
fn hero_of_bretagard_counts_cards_exiled_from_hand_or_by_your_spells() {
    cr!("603.2c", "603.2");
    supported("Hero of Bretagard");
    supported("Ranar the Ever-Watchful");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let hero = t.battlefield(P0, "Hero of Bretagard");
    // Two cards exiled from P0's hand at once.
    let a = t.hand(P0, "Island");
    let b = t.hand(P0, "Forest");
    t.g.batch_hold += 1;
    for c in [a, b] {
        t.g.move_object(c, mtg_engine::object::Zone::Exile, mtg_engine::events::MoveCause::Effect, None);
    }
    t.g.batch_hold -= 1;
    t.g.end_event_batch();
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(hero, "+1/+1"), 2);
    // P0's spell exiles a permanent.
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Swords to Plowshares", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.counters(hero, "+1/+1"), 3);
    // An opponent's spell exiling a permanent: no.
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Swords to Plowshares", &[Entity::Object(giant)]);
    t.resolve_all();
    assert_eq!(t.counters(hero, "+1/+1"), 3);
}

#[test]
fn ranar_creates_a_spirit_when_your_spell_exiles_permanents() {
    cr!("603.2c");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Ranar the Ever-Watchful");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Swords to Plowshares", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
}

#[test]
fn zurgo_and_ojutai_once_per_player_or_battle_dealt_damage() {
    cr!("603.2c", "510.2");
    ruling!(
        "Zurgo and Ojutai",
        "The triggered ability will trigger once for each player or battle dealt combat damage by Dragons you control, no matter how many Dragons were involved."
    );
    supported("Zurgo and Ojutai");
    // Two Dragons attacking one player: once.
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let z = t.battlefield(P0, "Zurgo and Ojutai");
    let s = t.battlefield(P0, "Shivan Dragon");
    for _ in 0..6 {
        t.library_top(P0, "Island");
    }
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    // The card from the top three (the default), then the Dragon to return.
    t.answer(P0, DecisionKind::Entities, Answer::Default);
    t.answer_choose(P0, &[Entity::Object(s)]);
    t.attack(&[(z, Entity::Player(P1)), (s, Entity::Player(P1))], &[]);
    t.resolve_all();
    // One card from the top three, and Shivan Dragon returned to hand.
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(t.in_hand(P0, "Shivan Dragon"));
    // Two players dealt damage: twice.
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    let z = t.battlefield(P0, "Zurgo and Ojutai");
    let s = t.battlefield(P0, "Shivan Dragon");
    for _ in 0..6 {
        t.library_top(P0, "Island");
    }
    let hand = t.hand_size(P0);
    t.answer_yes(P0, false);
    t.answer_yes(P0, false);
    t.attack(&[(z, Entity::Player(P1)), (s, Entity::Player(P2))], &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(!t.in_hand(P0, "Shivan Dragon"));
}

#[test]
fn imodane_spell_targeting_only_a_creature_dealing_damage_to_it() {
    cr!("115.9c", "603.2");
    supported("Imodane, the Pyrohammer");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Imodane, the Pyrohammer");
    let giant = t.battlefield(P1, "Hill Giant");
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Object(giant)]);
    t.resolve_all();
    // 3 damage to each opponent.
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 17);
    // Targeting a player: no.
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.life(P2), 17);
}

#[test]
fn zenos_chosen_creature_and_chosen_player() {
    cr!("607.2d", "701.27a", "104.2a");
    ruling!(
        "Zenos yae Galvus // Shinryu, Transcendent Rival",
        "In the case where you can't choose a creature an opponent controls for Zenos's first ability (probably because your opponents control no creatures), other creatures will still get -2/-2 until end of turn when Zenos's first ability resolves."
    );
    supported("Zenos yae Galvus");
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P1, "Hill Giant");
    let angel = t.battlefield(P2, "Serra Angel");
    t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let zenos = t.enter(P0, "Zenos yae Galvus");
    t.settle();
    t.resolve_all();
    // Creatures other than Zenos and the chosen creature get -2/-2.
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.pt(angel), (2, 2));
    assert_eq!(t.pt(giant), (3, 3));
    // Another creature leaving doesn't transform it; the chosen one does.
    crate::r_s02_common::destroy(&mut t, angel);
    t.resolve_all();
    assert_eq!(t.obj_now(zenos).chars.name.as_str(), "Zenos yae Galvus");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    crate::r_s02_common::destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.obj_now(zenos).chars.name.as_str(), "Shinryu, Transcendent Rival");
    // The chosen player loses: you win the game.
    t.g.players[P1.idx()].life = 0;
    t.settle();
    t.resolve_all();
    assert!(t.has_lost(P1));
    assert!(t.has_lost(P2));
    assert!(!t.has_lost(P0));
}

#[test]
fn zenos_no_creature_to_choose() {
    cr!("607.2d");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.enter(P0, "Zenos yae Galvus");
    t.settle();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}
