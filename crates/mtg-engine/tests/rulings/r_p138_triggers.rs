//! Rulings batch P138 — triggered abilities of edict cards: when they trigger and what
//! they see (CR 603.2, 603.4 intervening "if", 603.6 enters, 603.8 state triggers,
//! 603.10a look-back), targets chosen as they're put on the stack (CR 603.3d), and
//! Two-Headed Giant (CR 810).

use crate::r_p050_common::{team_lives, two_headed_giant};
use crate::r_p138_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

#[test]
fn sheoldred_cant_target_a_creature_another_upkeep_trigger_will_sacrifice() {
    cr!("603.3d", "503.1a", "115.1");
    ruling!(
        "Sheoldred, Whispering One",
        "If another triggered ability during your upkeep causes you to sacrifice a creature, such as that of Lord of the Pit, you won't be able to target it with Sheoldred's triggered ability as that ability is put on the stack."
    );
    supported("Sheoldred, Whispering One");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sheoldred, Whispering One");
    t.battlefield(P0, "Lord of the Pit");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_choose(P0, &[obj(bears)]);
    to_upkeep(&mut t, P0);
    // Both upkeep triggers went on the stack before either resolved: Sheoldred's target
    // was chosen while the Bears were still on the battlefield, so it's the Hill Giant,
    // the only creature card in the graveyard then.
    assert_eq!(t.stack_len(), 2);
    assert!(t.on_battlefield(bears));
    let targeted: Vec<Vec<Entity>> = t
        .g
        .stack
        .iter()
        .map(|id| crate::r_p016_common::stack_targets(&t, *id))
        .filter(|v| !v.is_empty())
        .collect();
    assert_eq!(targeted, vec![vec![obj(giant)]]);
    t.resolve_all();
    // The Bears' card stays in the graveyard.
    assert!(t.on_battlefield(giant));
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 0);
}

#[test]
fn upkeep_edicts_trigger_for_each_player_of_a_two_headed_giant_team() {
    cr!("810.2", "503.1a", "603.2");
    ruling!(
        "Sheoldred, Whispering One",
        "In a Two-Headed Giant game, the last ability will trigger twice at the beginning of the opposing team's upkeep, once for each player on that team. Each player sacrifices only a creature they control."
    );
    ruling!(
        "Call to the Grave",
        "In a Two-Headed Giant game, Call to the Grave will trigger twice during each team's upkeep, once for each player. Each of the two players will sacrifice a non-Zombie creature when the ability referring to that player resolves."
    );
    supported("Call to the Grave");
    for name in ["Sheoldred, Whispering One", "Call to the Grave"] {
        let mut t = two_headed_giant();
        t.battlefield(P0, name);
        let a = t.battlefield(P2, "Grizzly Bears");
        let b = t.battlefield(P3, "Grizzly Bears");
        let c = t.battlefield(P3, "Hill Giant");
        let zombie = t.battlefield(P3, "Walking Corpse");
        t.answer_choose(P3, &[obj(b)]);
        t.advance_to(P2, Step::Upkeep);
        t.settle();
        assert_eq!(triggers_on_stack(&t, "sacrifices"), 2, "{name}");
        t.resolve_all();
        assert!(!t.on_battlefield(a) && !t.on_battlefield(b), "{name}");
        assert!(t.on_battlefield(c) && t.on_battlefield(zombie), "{name}");
    }
}

#[test]
fn call_to_the_grave_checks_for_creatures_on_trigger_and_resolution() {
    cr!("603.4");
    ruling!(
        "Call to the Grave",
        "If there is at least one creature on the battlefield at the beginning of the end step, the last ability won't trigger. If it does trigger, but a creature enters before it resolves, the ability won't do anything when it resolves. Call to the Grave's controller won't sacrifice it."
    );
    // A creature on the battlefield: no trigger.
    let mut t = TestGame::new(2);
    let call = t.battlefield(P0, "Call to the Grave");
    t.battlefield(P1, "Grizzly Bears");
    to_end_step(&mut t);
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert!(t.on_battlefield(call));
    // None: it triggers; a creature enters in response: nothing happens.
    let mut t = TestGame::new(2);
    let call2 = t.battlefield(P0, "Call to the Grave");
    to_end_step(&mut t);
    assert_eq!(t.stack_len(), 1);
    t.battlefield(P1, "Grizzly Bears");
    t.resolve_all();
    assert!(t.on_battlefield(call2));
    // And without one, it's sacrificed.
    let mut t = TestGame::new(2);
    let call3 = t.battlefield(P0, "Call to the Grave");
    to_end_step(&mut t);
    t.resolve_all();
    assert!(!t.on_battlefield(call3));
}

#[test]
fn angrath_in_two_headed_giant() {
    cr!("810.9", "810.2");
    ruling!(
        "Angrath, the Flame-Chained",
        "In a Two-Headed Giant game, Angrath’s first ability causes the opposing team to lose 4 life, and each player on that team discards a card. His last ability causes the opposing team to lose life equal to the number of cards in both graveyards."
    );
    supported("Angrath, the Flame-Chained");
    let mut t = two_headed_giant();
    let angrath = t.battlefield(P0, "Angrath, the Flame-Chained");
    t.hand(P2, "Grizzly Bears");
    t.hand(P3, "Grizzly Bears");
    let before = team_lives(&t);
    lots(&mut t, P0);
    t.activate(P0, angrath, 0, &[]).expect("+1");
    t.resolve_all();
    assert_eq!(before.1 - team_lives(&t).1, 4);
    assert_eq!((t.hand_size(P2), t.hand_size(P3)), (0, 0));
    // −8: P2's graveyard has 1 card (the Bears) and P3's 1 + 3 more.
    for _ in 0..3 {
        t.graveyard(P3, "Hill Giant");
    }
    let n = t.graveyard_size(P2) + t.graveyard_size(P3);
    assert_eq!(n, 5);
    let have = t.counters(angrath, counters::LOYALTY);
    t.g.add_counters(obj(angrath), counters::LOYALTY, 8 - have.min(8), None);
    // (A loyalty ability once per turn: on P0's team's next turn.)
    let id = t.g.current(angrath);
    t.g.objects[id.0 as usize].activations_this_turn.clear();
    let before = team_lives(&t);
    t.activate(P0, angrath, 2, &[]).expect("−8");
    t.resolve_all();
    assert_eq!(before.1 - team_lives(&t).1, 5);
}

#[test]
fn witch_of_the_moors_needs_life_gained_this_turn() {
    cr!("603.4");
    ruling!(
        "Witch of the Moors",
        "If you haven't gained life by the time your end step begins, the triggered ability won't trigger at all."
    );
    for gained in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Witch of the Moors");
        t.battlefield(P1, "Grizzly Bears");
        if gained {
            crate::r_p009_common::gain(&mut t, P0, 1);
        }
        to_end_step(&mut t);
        assert_eq!(t.stack_len(), usize::from(gained), "gained {gained}");
    }
}

#[test]
fn instill_furor_triggers_on_its_controllers_turn_and_resolves_without_the_aura() {
    cr!("113.6", "603.2", "113.7a");
    ruling!(
        "Instill Furor",
        "Instill Furor grants a triggered ability that triggers at the end of the creature’s controller’s turn."
    );
    ruling!(
        "Instill Furor",
        "Once the ability triggers, nothing that happens to Instill Furor can prevent the ability from resolving."
    );
    supported("Instill Furor");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let furor = t.battlefield(P0, "Instill Furor");
    assert!(t.g.attach(furor, obj(bears)));
    // P0's end step: no trigger.
    to_end_step(&mut t);
    assert_eq!(t.stack_len(), 0);
    // P1's end step: it triggers; the Aura is destroyed in response.
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, furor);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn desolation_counts_lands_tapped_before_it_entered() {
    cr!("106.12", "603.2");
    ruling!(
        "Desolation",
        "It works even if it was not on the battlefield when the land was tapped for mana."
    );
    supported("Desolation");
    let mut t = TestGame::new(2);
    let plains = t.lands(P0, "Plains", 1)[0];
    let lions = t.hand(P0, "Savannah Lions");
    t.cast_with(P0, lions, &[]).unwrap();
    t.resolve_all();
    t.battlefield(P1, "Desolation");
    to_end_step(&mut t);
    t.resolve_all();
    assert!(!t.on_battlefield(plains));
    assert_eq!(t.life(P0), 18);
}

#[test]
fn cast_triggers_resolve_first_even_if_the_spell_is_countered() {
    cr!("603.2", "601.2i", "405.5");
    ruling!(
        "Kozilek, Butcher of Truth",
        "Kozilek's first ability triggers as you cast it, and that ability resolves before the spell itself. It resolves even if the spell is countered."
    );
    ruling!(
        "Nulldrifter",
        "Nulldrifter's first triggered ability will resolve before Nulldrifter does. If Nulldrifter is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    for (name, draws) in [("Kozilek, Butcher of Truth", 4usize), ("Nulldrifter", 2)] {
        let mut t = TestGame::new(2);
        let spell = cast(&mut t, P0, name, &[]);
        t.settle();
        assert_eq!(t.stack_len(), 2, "{name}");
        // P1 counters the spell in response to the trigger.
        let hand = t.hand_size(P0);
        cast(&mut t, P1, "Counterspell", &[&[obj(spell)]]);
        t.resolve();
        assert_ne!(t.zone(spell), Zone::Stack, "{name}: countered");
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + draws, "{name}");
    }
}

#[test]
fn kuon_counts_deaths_before_it_entered() {
    cr!("603.4", "710.2");
    ruling!(
        "Kuon, Ogre Ascendant // Kuon's Essence",
        "Kuon’s flip ability looks at everything that happened during that turn, even before Kuon entered the battlefield."
    );
    let name = "Kuon, Ogre Ascendant // Kuon's Essence";
    supported(name);
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        let b = t.battlefield(P1, "Grizzly Bears");
        kill(&mut t, b);
    }
    let kuon = t.battlefield(P0, name);
    to_end_step(&mut t);
    t.resolve_all();
    assert_eq!(t.obj_now(kuon).chars.name, "Kuon's Essence");
}

#[test]
fn mana_vortex_state_trigger_retriggers_only_after_leaving_the_stack() {
    cr!("603.8");
    ruling!(
        "Mana Vortex",
        "Mana Vortex’s third ability is a “state trigger.” Once a state trigger triggers, it won’t trigger again as long as the ability is on the stack. If the ability is countered and the trigger condition is still true, it will immediately trigger again."
    );
    supported("Mana Vortex");
    let mut t = TestGame::new(2);
    let vortex = t.battlefield(P0, "Mana Vortex");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.settle();
    assert_eq!(t.stack_len(), 1, "doesn't trigger again while on the stack");
    let ability = *t.g.stack.last().unwrap();
    cast(&mut t, P1, "Stifle", &[&[obj(ability)]]);
    t.resolve();
    assert!(t.on_battlefield(vortex));
    assert_eq!(t.stack_len(), 1, "triggered again");
    t.resolve_all();
    assert!(!t.on_battlefield(vortex));
}

#[test]
fn nefarox_trigger_resolves_before_blockers() {
    cr!("508.1m", "508.2", "509.1");
    ruling!(
        "Nefarox, Overlord of Grixis",
        "Nefarox's last ability will trigger and resolve before blockers are declared."
    );
    let mut t = TestGame::new(2);
    let nefarox = t.battlefield(P0, "Nefarox, Overlord of Grixis");
    let wall = t.battlefield(P1, "Wall of Denial");
    attack_with(&mut t, &[(nefarox, pl(P1))]);
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(t.stack_len() >= 1);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(!t.on_battlefield(wall), "gone before blockers");
}

#[test]
fn ruthless_deathfang_triggers_on_any_sacrifice_but_not_the_legend_rule() {
    cr!("704.5j", "701.21a", "603.10a");
    ruling!(
        "Ruthless Deathfang",
        "Notably, the “legend rule” does not cause any permanents to be sacrificed. They are simply put into their owners' graveyards."
    );
    ruling!(
        "Ruthless Deathfang",
        "Ruthless Deathfang's triggered ability will trigger whenever you sacrifice any creature, including Ruthless Deathfang itself. It will trigger whenever you sacrifice a creature for any reason, including to pay a cost or as the result of a spell or ability (such as an opposing Ruthless Deathfang's ability)."
    );
    supported("Ruthless Deathfang");
    // The legend rule.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ruthless Deathfang");
    t.battlefield(P0, "Isamaru, Hound of Konda");
    t.battlefield(P0, "Isamaru, Hound of Konda");
    t.battlefield(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 1);
    assert_eq!(t.stack_len(), 0);
    // Sacrificing itself to a cost, then a chain with an opposing Deathfang.
    let mut t = TestGame::new(2);
    let fang = t.battlefield(P0, "Ruthless Deathfang");
    let seer = t.battlefield(P0, "Viscera Seer");
    t.battlefield(P1, "Ruthless Deathfang");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(fang)]);
    t.answer_choose(P1, &[obj(bears)]);
    t.activate(P0, seer, 0, &[]).expect("sacrifice outlet");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "target opponent sacrifices"), 1);
    t.resolve_all();
    // P1 sacrificed the Bears (P0's trigger); P1's Deathfang then made P0 sacrifice the
    // Seer, which triggered nothing (P0's Deathfang was gone).
    assert!(!t.on_battlefield(bears));
    assert!(!t.on_battlefield(seer));
}

#[test]
fn oath_of_liliana_remembers_planeswalkers_that_entered_this_turn() {
    cr!("603.4", "603.2");
    ruling!(
        "Oath of Liliana",
        "Oath of Liliana’s last ability will trigger even if the planeswalker that entered the battlefield under your control is no longer on the battlefield, no longer under your control, or no longer a planeswalker. This is also true if Oath of Liliana wasn’t on the battlefield when the planeswalker entered."
    );
    let mut t = TestGame::new(2);
    let garruk = t.enter(P0, "Garruk Wildspeaker");
    t.settle();
    kill(&mut t, garruk);
    t.battlefield(P0, "Oath of Liliana");
    to_end_step(&mut t);
    t.resolve_all();
    let zombies = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(zombies.len(), 1);
}

#[test]
fn ogre_marauder_unblockable_unless_a_creature_is_sacrificed() {
    cr!("508.1m", "118.12");
    ruling!(
        "Ogre Marauder",
        "Ogre Marauder’s ability requires the defending player to sacrifice a creature as the ability resolves, or Ogre Marauder can’t be blocked for the rest of the turn."
    );
    for sac in [true, false] {
        let mut t = TestGame::new(2);
        let ogre = t.battlefield(P0, "Ogre Marauder");
        let b1 = t.battlefield(P1, "Grizzly Bears");
        let b2 = t.battlefield(P1, "Grizzly Bears");
        t.answer_yes(P1, sac);
        t.answer_choose(P1, &[obj(b1)]);
        attack_with(&mut t, &[(ogre, pl(P1))]);
        t.resolve_all();
        assert_eq!(t.on_battlefield(b1), !sac);
        let can_block = t.g.can_block(b2, ogre);
        assert_eq!(can_block, sac, "sacrificed {sac}");
    }
}

#[test]
fn savra_triggers_on_sacrifices_other_effects_cause() {
    cr!("603.2", "603.3b");
    ruling!(
        "Savra, Queen of the Golgari",
        "Sacrificing a creature that's both black and green will make both abilities trigger. You may put them on the stack in whichever order you want."
    );
    ruling!(
        "Savra, Queen of the Golgari",
        "Savra itself doesn't let you sacrifice creatures. Its abilities trigger whenever you sacrifice a black or green creature because some other spell, ability, or cost instructed you to do so."
    );
    supported("Savra, Queen of the Golgari");
    let mut t = TestGame::new(2);
    let savra = t.battlefield(P0, "Savra, Queen of the Golgari");
    assert_eq!(crate::r_p198_common::activated_count(&t, savra), 0);
    let mangler = t.battlefield(P0, "Dreg Mangler");
    let seer = t.battlefield(P0, "Viscera Seer");
    t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(mangler)]);
    let from = t.asked().len();
    t.activate(P0, seer, 0, &[]).expect("sacrifice outlet");
    t.settle();
    assert_eq!(t.stack_len(), 3, "the Seer's ability and both triggers");
    assert!(asked_since(&t, from)
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
}

#[test]
fn bearer_of_silence_payment_is_decided_on_resolution() {
    cr!("603.2", "608.2d");
    ruling!(
        "Bearer of Silence",
        "Players can respond to the triggered ability, but once it starts resolving and you decide whether to pay {1}{C}, it's too late for anyone to respond."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    cast(&mut t, P0, "Bearer of Silence", &[&[pl(P1)]]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert!(!asked_yes_no(&t, P0, from), "not asked while casting");
    t.answer_yes(P0, true);
    t.resolve();
    assert!(asked_yes_no(&t, P0, from));
    assert!(!t.on_battlefield(bears));
}

#[test]
fn inevitable_end_triggers_in_the_upkeep_and_can_be_responded_to() {
    cr!("502.4", "503.1a", "117.3a");
    ruling!(
        "Inevitable End",
        "Players may take actions before the triggered ability granted by Inevitable End resolves, but they can’t take any actions during a turn before that turn’s upkeep begins to try to stop the ability from triggering."
    );
    supported("Inevitable End");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let end = t.battlefield(P0, "Inevitable End");
    assert!(t.g.attach(end, obj(bears)));
    let steps = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::Priority { .. }),
        |g| g.turn.step,
    );
    to_upkeep(&mut t, P1);
    assert_eq!(t.stack_len(), 1);
    // Players get priority with it on the stack, but none in the untap step.
    assert_eq!(t.g.turn.priority, Some(P1));
    assert!(steps.lock().unwrap().iter().all(|s| *s != Step::Untap));
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn michiko_konda_one_permanent_per_damage_event() {
    cr!("120.3", "702.4b", "603.2");
    ruling!(
        "Michiko Konda, Truth Seeker",
        "One permanent is sacrificed each time an opponent’s source deals damage to Michiko Konda’s controller. The amount of damage doesn’t matter."
    );
    ruling!(
        "Michiko Konda, Truth Seeker",
        "Sources that deal damage multiple times (such as creatures with double strike) will trigger Michiko Konda multiple times."
    );
    supported("Michiko Konda, Truth Seeker");
    // 7 damage in one event: one permanent.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Michiko Konda, Truth Seeker");
    let giant = t.battlefield(P0, "Craw Wurm");
    t.lands(P0, "Forest", 3);
    attack_with(&mut t, &[(giant, pl(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(permanents_of(&t, P0).len(), 3, "one of four sacrificed");
    // Double strike: two events, two permanents.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Michiko Konda, Truth Seeker");
    let knight = t.battlefield(P0, "Fencing Ace");
    let forests = t.lands(P0, "Forest", 3);
    // P0 keeps the Ace (sacrificing Forests), so it deals its regular damage too.
    t.answer_choose(P0, &[obj(forests[0])]);
    t.answer_choose(P0, &[obj(forests[1])]);
    attack_with(&mut t, &[(knight, pl(P1))]);
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(permanents_of(&t, P0).len(), 2, "two of four sacrificed");
}

#[test]
fn michiko_konda_spliced_glacial_ray_triggers_twice() {
    cr!("702.47a", "120.3");
    ruling!(
        "Michiko Konda, Truth Seeker",
        "Spells that deal damage repeatedly trigger Michiko Konda’s ability each time they deal damage. For example, a Glacial Ray spliced onto another Glacial Ray triggers the ability twice (once for each “2 damage” event from the Glacial Ray)."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Michiko Konda, Truth Seeker");
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Wastes", 3);
    let ray = t.hand(P0, "Glacial Ray");
    t.hand(P0, "Glacial Ray");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, ray).target(pl(P1)).target(pl(P1)).go();
    t.resolve();
    assert_eq!(t.life(P1), 16);
    assert_eq!(triggers_on_stack(&t, "sacrifices"), 2);
}
