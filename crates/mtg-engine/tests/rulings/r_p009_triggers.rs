//! Rulings batch P009 — triggered abilities of burn cards: "first spell each turn"
//! (CR 603.2, 603.4), last known information (CR 608.2h, 113.7a), reflexive triggers
//! (CR 603.12), effects created by a chapter that outlive the Saga (CR 603.7, 714),
//! simultaneous damage triggering once (CR 603.2c), "at the beginning of the end step"
//! intervening-if checks (CR 603.4), and costs paid while resolving (CR 608.2d).

use crate::r_p009_common::*;
use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::{cast_new, targets_of};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// --- Vial Smasher the Fierce --------------------------------------------------------------

#[test]
fn vial_smasher_triggers_on_your_first_spell_on_any_turn() {
    cr!("603.2", "601.2i");
    ruling!(
        "Vial Smasher the Fierce",
        "Vial Smasher's triggered ability triggers when you cast your first spell each turn, regardless of whose turn it is."
    );
    supported("Vial Smasher the Fierce");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vial Smasher the Fierce");
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // The second spell that turn doesn't trigger it.
    cast_new(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn vial_smasher_uses_a_countered_spells_last_known_mana_value() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Vial Smasher the Fierce",
        "Vial Smasher's triggered ability resolves before the spell that caused it to trigger. If Vial Smasher's ability resolves and the spell that caused it to trigger has been countered, use that spell's mana value as it last existed on the stack to determine how much damage is dealt."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vial Smasher the Fierce");
    let div = cast_new(&mut t, P0, "Divination", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2, "the trigger is above Divination");
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.g.turn.priority = Some(P1);
    t.cast(P1, cs).target(div).go();
    t.resolve(); // Counterspell
    assert!(t.in_graveyard(P0, "Divination"));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn vial_smasher_must_be_on_the_battlefield_as_the_spell_is_cast() {
    cr!("603.2", "601.2i", "603.10");
    ruling!(
        "Vial Smasher the Fierce",
        "Vial Smasher has to be on the battlefield at the moment you cast your first spell. If that spell causes Vial Smasher to leave the battlefield as an additional cost to cast it, Vial Smasher's ability can't trigger. If that spell is Vial Smasher itself, Vial Smasher's ability can't trigger."
    );
    // Casting Vial Smasher itself.
    let mut t = TestGame::new(2);
    cast_new(&mut t, P0, "Vial Smasher the Fierce", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // Sacrificing it to cast Fling.
    let mut t = TestGame::new(2);
    let smasher = t.battlefield(P0, "Vial Smasher the Fierce");
    t.answer_choose(P0, &[obj(smasher)]);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Fling", &[obj(bears)]);
    assert!(!t.on_battlefield(smasher));
    t.settle();
    assert_eq!(t.stack_len(), 1, "no trigger");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn vial_smasher_chooses_the_opponent_as_it_resolves() {
    cr!("608.2c", "603.3d");
    ruling!(
        "Vial Smasher the Fierce",
        "The opponent to be dealt damage is chosen at random while the triggered ability is resolving. After that opponent is chosen, you choose whether damage will be dealt to that player or to a planeswalker they control, and if so, which planeswalker."
    );
    // Three players: P2 leaves the game while the trigger is on the stack, so P1 is the
    // only opponent left to choose from as it resolves.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Vial Smasher the Fierce");
    cast_new(&mut t, P0, "Divination", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.g.players[2].life = 0;
    t.settle();
    assert!(t.has_lost(P2));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Choosing a planeswalker that player controls instead.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vial Smasher the Fierce");
    let pw = t.battlefield(P1, "Chandra Nalaar");
    t.answer(
        P0,
        DecisionKind::Option,
        mtg_engine::decision::Answer::Index(1),
    );
    t.answer_choose(P0, &[obj(pw)]);
    cast_new(&mut t, P0, "Divination", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.counters(pw, counters::LOYALTY), 3);
}

// --- Other triggers ---------------------------------------------------------------------

#[test]
fn rampaging_raptor_triggers_only_on_damage_to_an_opponent() {
    cr!("603.2", "510.2");
    ruling!(
        "Rampaging Raptor",
        "Unlike many similar abilities, Rampaging Raptor's triggered ability triggers whenever it deals combat damage to an opponent, not to a player. If it happens to deal combat damage to you (usually due to a redirection effect, which would be unusual), the ability won't trigger."
    );
    supported("Rampaging Raptor");
    let mut t = TestGame::new(2);
    let raptor = t.battlefield(P0, "Rampaging Raptor");
    let pw = t.battlefield(P1, "Chandra Nalaar");
    t.answer_targets(P0, &[obj(pw)]);
    t.g.deal_damage(raptor, pl(P1), 4, true);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(pw, counters::LOYALTY), 2);
    // Combat damage to its controller: no trigger.
    let mut t = TestGame::new(2);
    let raptor = t.battlefield(P0, "Rampaging Raptor");
    t.battlefield(P1, "Chandra Nalaar");
    t.g.deal_damage(raptor, pl(P0), 4, true);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn battle_of_frost_and_fire_chapter_three_outlives_the_saga() {
    cr!("714.4", "603.7c");
    ruling!(
        "Battle of Frost and Fire",
        "The triggered ability created by the chapter III ability may trigger multiple times during the turn, even though Battle of Frost and Fire will likely no longer be on the battlefield."
    );
    supported("Battle of Frost and Fire");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Battle of Frost and Fire");
    t.g.add_counters(obj(saga), counters::LORE, 3, None);
    t.g.flush_events();
    t.resolve_all();
    assert!(!t.on_battlefield(saga), "sacrificed after chapter III");
    for _ in 0..2 {
        t.hand(P0, "Grizzly Bears");
        let hand = t.hand_size(P0);
        cast_new(&mut t, P0, "Colossal Dreadmaw", &[]);
        t.settle();
        assert_eq!(t.stack_len(), 2);
        t.resolve_all();
        // Draw two, discard one.
        assert_eq!(t.hand_size(P0), hand + 1);
    }
}

#[test]
fn annie_joins_up_doubles_a_legendary_creatures_when_trigger() {
    cr!("603.1", "603.2d");
    ruling!(
        "Annie Joins Up",
        "Triggered abilities use the word “when,” “whenever,” or “at.” They’re often written as “[Trigger condition], [effect].”"
    );
    supported("Annie Joins Up");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Annie Joins Up");
    let a = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(a)]);
    t.answer_targets(P0, &[obj(a)]);
    t.enter(P0, "Dragonlord Atarka");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
}

#[test]
fn leyline_of_lightning_pays_one_once() {
    cr!("608.2d");
    ruling!(
        "Leyline of Lightning",
        "While resolving Leyline of Lightning's second ability, you can't pay {1} multiple times to deal damage multiple times."
    );
    supported("Leyline of Lightning");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Lightning");
    t.lands(P0, "Wastes", 3);
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    cast_new(&mut t, P0, "Divination", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn ashling_counts_resolutions_not_activations() {
    cr!("602.2", "608.2");
    ruling!(
        "Ashling the Pilgrim",
        "Ashling the Pilgrim's ability counts resolutions, not activations. Any such abilities that are still on the stack won't count toward the total."
    );
    supported("Ashling the Pilgrim");
    let mut t = TestGame::new(2);
    let ashling = t.battlefield(P0, "Ashling the Pilgrim");
    t.lands(P0, "Mountain", 8);
    t.activate(P0, ashling, 0, &[]).unwrap();
    t.resolve();
    t.activate(P0, ashling, 0, &[]).unwrap();
    t.activate(P0, ashling, 0, &[]).unwrap();
    t.resolve();
    // Three activations, two resolutions: nothing happens yet.
    assert_eq!(t.counters(ashling, counters::PLUS1), 2);
    assert_eq!(t.life(P1), 20);
    t.resolve();
    // The third resolution: remove the three counters and deal 3 damage to everything.
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.counters(ashling, counters::PLUS1), 0);
}

#[test]
fn pestilence_activations_are_separate_one_damage_effects() {
    cr!("602.2", "608.2");
    ruling!(
        "Pestilence",
        "Each activation is considered a new damage effect. An activation can only be 1 point of damage."
    );
    ruling!(
        "Pyrohemia",
        "Each activation is considered a new damage effect. An activation can only be 1 point of damage."
    );
    for (card, land) in [("Pestilence", "Swamp"), ("Pyrohemia", "Mountain")] {
        supported(card);
        let mut t = TestGame::new(2);
        let p = t.battlefield(P0, card);
        let wurm = t.battlefield(P1, "Craw Wurm");
        t.lands(P0, land, 2);
        t.activate(P0, p, 0, &[]).unwrap();
        t.resolve();
        assert_eq!(dmg(&t, wurm), 1, "{card}");
        assert_eq!(t.life(P1), 19, "{card}");
        t.activate(P0, p, 0, &[]).unwrap();
        t.resolve();
        assert_eq!(dmg(&t, wurm), 2, "{card}");
    }
}

#[test]
fn pestilence_stays_if_a_creature_was_there_as_the_end_step_began() {
    cr!("603.4", "513.2");
    ruling!(
        "Pestilence",
        "It will stay on the battlefield if there is a creature that is put into the graveyard during the end step. This is because this ability will not trigger at all if there is at least one creature on the battlefield as the end step begins."
    );
    ruling!(
        "Pyrohemia",
        "It will stay on the battlefield if there is a creature that is put into the graveyard during the end step. This is because this ability will not trigger at all if there is at least one creature on the battlefield as the end step begins."
    );
    for card in ["Pestilence", "Pyrohemia"] {
        let mut t = TestGame::new(2);
        let p = t.battlefield(P0, card);
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.set_step(P0, Step::PostcombatMain);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(t.stack_len(), 0, "{card}");
        kill(&mut t, bears);
        t.advance_to(P1, Step::Upkeep);
        assert!(t.on_battlefield(p), "{card}");
        // With no creatures as the end step begins, it's sacrificed.
        t.advance_to(P1, Step::End);
        t.settle();
        t.resolve_all();
        assert!(!t.on_battlefield(p), "{card}");
    }
}

#[test]
fn volatile_rig_triggers_once_for_simultaneous_damage() {
    cr!("603.2c", "510.2");
    ruling!(
        "Volatile Rig",
        "If Volatile Rig is dealt damage by multiple sources at the same time (for example, multiple blocking creatures), its first triggered ability will trigger only once."
    );
    supported("Volatile Rig");
    let mut t = TestGame::new(2);
    let rig = t.battlefield(P0, "Volatile Rig");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage_batch(vec![(a, obj(rig), 1), (b, obj(rig), 1)], true);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "flip a coin"), 1);
}

#[test]
fn heartfire_immolator_uses_its_last_known_power() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Heartfire Immolator",
        "Use Heartfire Immolator's power as it last existed on the battlefield to determine how much damage its last ability deals."
    );
    supported("Heartfire Immolator");
    let mut t = TestGame::new(2);
    let imm = t.battlefield(P0, "Heartfire Immolator");
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    cast_new(&mut t, P0, "Giant Growth", &[obj(imm)]);
    t.resolve_all();
    assert_eq!(t.pt(imm).0, 6, "prowess and Giant Growth");
    t.lands(P0, "Mountain", 1);
    t.answer_targets(P0, &[obj(wurm)]);
    activate_containing(&mut t, P0, imm, "Sacrifice").unwrap();
    assert!(!t.on_battlefield(imm));
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
}

// --- Reflexive triggers and choices while resolving ---------------------------------------

#[test]
fn serum_core_chimera_targets_with_a_reflexive_trigger() {
    cr!("603.12");
    ruling!(
        "Serum-Core Chimera",
        "You don't choose a target for Serum-Core Chimera's last ability at the time you activate it. Rather, a second \"reflexive\" ability triggers when you discard a card this way. You choose a target for that ability as it goes on the stack. Each player may respond to this triggered ability as normal."
    );
    supported("Serum-Core Chimera");
    let mut t = TestGame::new(2);
    let chimera = t.battlefield(P0, "Serum-Core Chimera");
    t.g.add_counters(obj(chimera), "oil", 3, None);
    let card = t.hand(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let asked = t.asked().len();
    activate_containing(&mut t, P0, chimera, "oil").unwrap();
    assert!(
        !t.asked()[asked..]
            .iter()
            .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseTargets { .. })),
        "no target on activation"
    );
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(card)]);
    t.answer_targets(P0, &[obj(wurm)]);
    t.resolve();
    // The reflexive trigger is on the stack, with its target, and can be responded to.
    assert_eq!(t.stack_len(), 1);
    let top = t.g.stack[0];
    assert_eq!(targets_of(&t, top), vec![obj(wurm)]);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 3);
}

#[test]
fn yotia_declares_war_taps_then_targets_reflexively() {
    cr!("603.12", "714.2b");
    ruling!(
        "Yotia Declares War",
        "You don't choose the target creature or planeswalker as you put the second chapter ability of Yotia Declares War on the stack. First, you tap any number of artifacts. Then, a second \"reflexive\" ability triggers (denoted by the phrase \"when you do\") that requires a target. Players may respond to the reflexive triggered ability as normal, and they will know how many artifacts were tapped when they do."
    );
    supported("Yotia Declares War");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Yotia Declares War");
    let a = t.battlefield(P0, "Ornithopter");
    let b = t.battlefield(P0, "Ornithopter");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.g.add_counters(obj(saga), counters::LORE, 2, None);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let chapter = t.g.stack[0];
    assert!(targets_of(&t, chapter).is_empty());
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.answer_targets(P0, &[obj(wurm)]);
    t.resolve();
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    assert_eq!(t.stack_len(), 1, "the reflexive trigger");
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 2);
}

#[test]
fn torrent_sculptor_chooses_the_card_as_its_trigger_resolves() {
    cr!("608.2c", "603.3d");
    ruling!(
        "Torrent Sculptor // Flamethrower Sonata",
        "You choose which instant or sorcery card to exile as Torrent Sculptor’s enters-the-battlefield ability resolves. You must exile one if able."
    );
    supported("Torrent Sculptor // Flamethrower Sonata");
    let mut t = TestGame::new(2);
    let sculptor = t.enter(P0, "Torrent Sculptor // Flamethrower Sonata");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // Divination arrives in the graveyard in response; it's exiled (mandatory).
    t.graveyard(P0, "Divination");
    t.resolve_all();
    assert!(t.in_exile("Divination"));
    assert_eq!(t.counters(sculptor, counters::PLUS1), 2);
}

#[test]
fn nahiri_creates_and_equips_while_resolving() {
    cr!("608.2", "117.1");
    ruling!(
        "Nahiri, Heir of the Ancients",
        "You create a Kor Warrior token and attach an Equipment to it if you wish all while Nahiri's first ability is resolving."
    );
    supported("Nahiri, Heir of the Ancients");
    let mut t = TestGame::new(2);
    let nahiri = t.battlefield(P0, "Nahiri, Heir of the Ancients");
    let eq = t.battlefield(P0, "Bonesplitter");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(eq)]);
    t.activate(P0, nahiri, 0, &[]).unwrap();
    t.resolve();
    let token = crate::r_s01_common::tokens(&t, P0)[0];
    assert_eq!(t.obj_now(eq).attached_to, Some(obj(token)));
    assert_eq!(t.pt(token), (3, 1));
    assert_eq!(t.zone(eq), Zone::Battlefield);
}


#[test]
fn impatience_counts_a_countered_spell_as_cast() {
    cr!("603.4", "601.2i", "701.6a");
    ruling!(
        "Impatience",
        "If a spell is countered, it still counts as having been cast."
    );
    supported("Impatience");
    // P1 casts Divination, which is countered: no damage at P1's end step.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Impatience");
    t.set_step(P1, Step::PrecombatMain);
    let div = cast_new(&mut t, P1, "Divination", &[]);
    t.lands(P0, "Island", 2);
    let cs = t.hand(P0, "Counterspell");
    t.g.turn.priority = Some(P0);
    t.cast(P0, cs).target(div).go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Divination"));
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // P0 cast Counterspell (on P1's turn) but no spell on its own turn: 2 damage at P0's
    // end step.
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn you_didnt_cast_a_spell_this_turn() {
    cr!("603.4");
    supported("Nightpack Ambusher");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nightpack Ambusher");
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 1);
    // Having cast a spell: no Wolf.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nightpack Ambusher");
    t.set_step(P0, Step::PostcombatMain);
    cast_new(&mut t, P0, "Divination", &[]);
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(crate::r_s01_common::tokens(&t, P0).is_empty());
}
