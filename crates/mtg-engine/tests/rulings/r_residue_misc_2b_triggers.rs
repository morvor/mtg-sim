//! Item `rulings-residue-misc-2`, pass B — triggered abilities: state triggers (CR 603.8),
//! intervening "if" clauses (603.4), sources that leave while their abilities wait (603.10,
//! 608.2h, 113.7a) and the cards a delayed or end-step trigger can still find (400.7).

use crate::r_residue_misc_2b_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

// --- Bronze Bombshell ------------------------------------------------------------------------

#[test]
fn bronze_bombshell_is_a_state_trigger_that_waits_and_retriggers() {
    cr!("603.8", "603.3a");
    ruling!(
        "Bronze Bombshell",
        "This ability is a state trigger. Once it triggers, it won’t trigger again while the ability is on the stack. If the triggered ability leaves the stack and the creature is still on the battlefield under the control of a player other than its owner, the ability will immediately trigger again."
    );
    supported("Bronze Bombshell");
    supported("Stifle");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Bronze Bombshell");
    give_control(&mut t, b, P1);
    let trig = triggers_of(&t, "Bronze Bombshell");
    assert_eq!(trig.len(), 1, "{}", t.dump_log());
    assert_eq!(stack_controller(&t, trig[0]), P1);
    // Checked again: it doesn't trigger again while it waits.
    t.settle();
    t.settle();
    assert_eq!(triggers_of(&t, "Bronze Bombshell").len(), 1);
    // Countered: P1 still controls it, so it triggers again at once.
    stifle(&mut t, P0, trig[0]);
    t.settle();
    let again = triggers_of(&t, "Bronze Bombshell");
    assert_eq!(again.len(), 1, "{}", t.dump_log());
    assert_ne!(again[0], trig[0]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Bronze Bombshell"));
    assert_eq!(t.life(P1), 13);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn bronze_bombshell_deals_damage_only_if_its_ability_sacrificed_it() {
    cr!("603.8", "608.2c", "701.21a");
    interp!("that-opponent-fixed-as-it-triggers");
    ruling!(
        "Bronze Bombshell",
        "Bronze Bombshell deals 7 damage only if it was sacrificed as a result of its ability. If it leaves the battlefield or changes controllers before the ability resolves, it can’t be sacrificed and won’t deal damage."
    );
    // Back under its owner's control before the ability resolves: nothing happens.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Bronze Bombshell");
    give_control(&mut t, b, P1);
    assert_eq!(triggers_of(&t, "Bronze Bombshell").len(), 1);
    give_control(&mut t, b, P0);
    t.resolve_all();
    assert!(t.on_battlefield(b), "{}", t.dump_log());
    assert_eq!(controller(&t, b), P0);
    assert_eq!((t.life(P0), t.life(P1)), (20, 20));
    // To a third player: P1 can't sacrifice it (the new controller's own ability waits).
    let mut t = TestGame::new(3);
    let b = t.battlefield(P0, "Bronze Bombshell");
    give_control(&mut t, b, P1);
    let first = triggers_of(&t, "Bronze Bombshell")[0];
    give_control(&mut t, b, P2);
    // P1's ability resolves first only if it's on top: resolve just it.
    while t.g.stack.last() != Some(&first) {
        t.resolve();
    }
    t.resolve();
    assert_eq!(t.life(P1), 20, "{}", t.dump_log());
    // Destroyed in response: nothing is sacrificed, no damage.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Bronze Bombshell");
    give_control(&mut t, b, P1);
    destroy(&mut t, b);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Bronze Bombshell"));
    assert_eq!(t.life(P1), 20);
}

// --- Garruk Relentless -------------------------------------------------------------------------

#[test]
fn garruk_relentless_transform_is_a_state_trigger() {
    cr!("603.8");
    ruling!(
        "Garruk Relentless // Garruk, the Veil-Cursed",
        "Garruk Relentless's first ability is a state-triggered ability. It triggers once Garruk has two or fewer loyalty counters on him and it can't retrigger while that ability is on the stack."
    );
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Garruk Relentless // Garruk, the Veil-Cursed");
    t.settle();
    assert!(triggers_of(&t, "Garruk Relentless").is_empty());
    t.g.objects[g.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 2);
    t.settle();
    assert_eq!(triggers_of(&t, "Garruk Relentless").len(), 1);
    t.g.objects[g.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 1);
    t.settle();
    t.settle();
    assert_eq!(triggers_of(&t, "Garruk Relentless").len(), 1);
    t.resolve_all();
    assert_eq!(t.obj_now(g).chars.name, "Garruk, the Veil-Cursed");
    assert!(triggers_of(&t, "Garruk, the Veil-Cursed").is_empty());
}

// --- Krovikan Vampire ----------------------------------------------------------------------------

/// P0's Krovikan Vampire deals 2 damage to P1's Grizzly Bears, which die.
fn vampire_killed_bears() -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let kv = t.battlefield(P0, "Krovikan Vampire");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(kv, Entity::Object(bears), 2, false);
    t.settle();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    (t, kv, bears)
}

#[test]
fn krovikan_vampire_off_the_battlefield_at_the_end_step_doesnt_trigger() {
    cr!("603.4", "603.2");
    ruling!(
        "Krovikan Vampire",
        "If Krovikan Vampire isn’t on the battlefield at the beginning of the end step, its ability won’t trigger. No cards will be returned to the battlefield, even if Krovikan Vampire dealt damage to some creatures that died that turn."
    );
    supported("Krovikan Vampire");
    let (mut t, kv, _) = vampire_killed_bears();
    kill(&mut t, kv);
    t.advance_to(P0, Step::End);
    t.settle();
    assert!(triggers_of(&t, "Krovikan Vampire").is_empty());
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn krovikan_vampire_loses_track_of_a_card_that_left_the_graveyard() {
    cr!("400.7", "603.4");
    ruling!(
        "Krovikan Vampire",
        "If a creature Krovikan Vampire dealt damage to dies, then leaves the graveyard by some means, the game loses track of it. Krovikan Vampire’s ability won’t be able to return it to the battlefield, even if it’s put back into the graveyard before the end step."
    );
    let (mut t, _, bears) = vampire_killed_bears();
    // To the hand and back to the graveyard: a new object.
    let in_hand = mv(&mut t, bears, Zone::Hand(P1));
    mv(&mut t, in_hand, Zone::Graveyard(P1));
    t.settle();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn krovikan_vampire_returns_a_creature_that_died_in_response() {
    cr!("603.4", "608.2h");
    ruling!(
        "Krovikan Vampire",
        "If a creature that Krovikan Vampire dealt damage to that turn dies in response to the triggered ability, that creature card will be put onto the battlefield. Note that an appropriate creature must have died prior to the beginning of the end step in order for the ability to trigger in the first place."
    );
    // Only a damaged survivor: no trigger at all.
    let mut t = TestGame::new(2);
    let kv = t.battlefield(P0, "Krovikan Vampire");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(kv, Entity::Object(giant), 1, false);
    t.settle();
    t.advance_to(P0, Step::End);
    t.settle();
    assert!(triggers_of(&t, "Krovikan Vampire").is_empty());
    // A creature died first: it triggers, and the survivor dying in response returns too.
    let (mut t, kv, _) = vampire_killed_bears();
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(kv, Entity::Object(giant), 1, false);
    t.settle();
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(
        triggers_of(&t, "Krovikan Vampire").len(),
        1,
        "{}",
        t.dump_log()
    );
    kill(&mut t, giant);
    t.resolve_all();
    assert_eq!(n_named(&t, P0, "Hill Giant"), 1, "{}", t.dump_log());
    assert_eq!(n_named(&t, P0, "Grizzly Bears"), 1);
}

#[test]
fn krovikan_vampire_returns_cards_under_whoever_controls_it_as_it_triggers() {
    cr!("603.3a", "608.2c");
    ruling!(
        "Krovikan Vampire",
        "Whoever controls Krovikan Vampire when the ability triggers will return the applicable cards to the battlefield under their control. It doesn’t matter who controlled it at the time the damage was dealt."
    );
    let mut t = TestGame::new(3);
    let kv = t.battlefield(P0, "Krovikan Vampire");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(kv, Entity::Object(bears), 2, false);
    t.settle();
    give_control(&mut t, kv, P2);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(n_named(&t, P2, "Grizzly Bears"), 1, "{}", t.dump_log());
}

// --- Seraph -------------------------------------------------------------------------------

#[test]
fn seraph_triggers_if_it_dies_with_the_creature_but_not_after_leaving() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Seraph",
        "If Seraph isn’t on the battlefield at the time the other creature is put into a graveyard, the ability won’t trigger. However, the ability does trigger if Seraph and the other creature are put into the graveyard at the same time."
    );
    supported("Seraph");
    supported("Wrath of God");
    // Seraph gone first: nothing triggers when the damaged Giant dies.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Seraph");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(s, Entity::Object(giant), 1, false);
    t.settle();
    kill(&mut t, s);
    kill(&mut t, giant);
    assert!(triggers_of(&t, "Seraph").is_empty());
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    // Both die at once (Wrath of God): it triggers, and the Giant returns at the end step.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Seraph");
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.deal_damage(s, Entity::Object(giant), 1, false);
    t.settle();
    t.lands(P0, "Plains", 4);
    let w = t.hand(P0, "Wrath of God");
    t.cast(P0, w).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Seraph"));
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert_eq!(n_named(&t, P0, "Hill Giant"), 1, "{}", t.dump_log());
}

#[test]
fn seraph_returns_no_token_and_no_card_that_left_the_graveyard() {
    cr!("704.5d", "111.7", "400.7");
    // The token ceases to exist once it's in the graveyard (the ruling predates the
    // current wording, which puts the card onto the battlefield from the graveyard).
    ruling!(
        "Seraph",
        "If the other creature is a token, it will cease to exist after being exiled. It won’t be returned to the battlefield."
    );
    ruling!(
        "Seraph",
        "You do not get the creature back if the creature is not still in the graveyard at the end of the turn."
    );
    supported("Raise the Alarm");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Seraph");
    t.lands(P1, "Plains", 2);
    let ra = t.hand(P1, "Raise the Alarm");
    t.cast(P1, ra).go();
    t.resolve_all();
    let soldier = tokens(&t, P1)[0];
    t.g.deal_damage(s, Entity::Object(soldier), 1, false);
    t.settle();
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.deal_damage(s, Entity::Object(bears), 2, false);
    t.settle();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // The Bears leave the graveyard before the end step.
    mv(&mut t, bears, Zone::Exile);
    t.advance_to(P0, Step::End);
    t.settle();
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty(), "{}", t.dump_log());
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert!(t.in_exile("Grizzly Bears"));
}

// --- Kjeldoran Home Guard ------------------------------------------------------------------------

#[test]
fn kjeldoran_home_guard_killed_in_combat_doesnt_trigger() {
    cr!("603.4", "511.1");
    ruling!(
        "Kjeldoran Home Guard",
        "The ability will trigger only if Kjeldoran Home Guard is still on the battlefield at end of combat. If it’s been dealt lethal combat damage and destroyed during that combat (or has left the battlefield during that combat by any other means), its ability won’t trigger and you won’t get a token."
    );
    supported("Kjeldoran Home Guard");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let g = t.battlefield(P0, "Kjeldoran Home Guard");
    let giant = t.battlefield(P1, "Craw Wurm");
    attack_with(&mut t, &[(g, pl(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(giant, g)]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert!(t.in_graveyard(P0, "Kjeldoran Home Guard"));
    assert!(triggers_of(&t, "Kjeldoran Home Guard").is_empty());
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
}

// --- Spirit Mirror -------------------------------------------------------------------------

#[test]
fn spirit_mirror_checks_for_reflections_as_it_triggers_and_resolves() {
    cr!("603.4");
    ruling!(
        "Spirit Mirror",
        "It checks if there are any Reflection tokens on the battlefield at the beginning of upkeep, and if there are it will not trigger at all. It will check again on resolution and will do nothing if a Reflection is on the battlefield at that time."
    );
    supported("Spirit Mirror");
    // No Reflection: it triggers and makes one.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Spirit Mirror");
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(triggers_of(&t, "Spirit Mirror").len(), 1);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 1, "{}", t.dump_log());
    // With that Reflection on the battlefield next turn: no trigger at all.
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert!(triggers_of(&t, "Spirit Mirror").is_empty());
    // A Reflection that appears in response: the ability does nothing.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Spirit Mirror");
    t.advance_to(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    let trig = triggers_of(&t, "Spirit Mirror");
    assert_eq!(trig.len(), 1);
    // A Reflection token appears in response.
    let _ = m;
    create_token(&mut t, P1, "Reflection");
    assert_eq!(tokens(&t, P1).len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty(), "{}", t.dump_log());
}

// --- Dionus, Elvish Archdruid --------------------------------------------------------------

/// P0's turn, P0 controls `n` Dionus.
fn dionus(n: usize) -> (TestGame, Vec<ObjectId>) {
    supported("Dionus, Elvish Archdruid");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let d = (0..n)
        .map(|_| t.battlefield(P0, "Dionus, Elvish Archdruid"))
        .collect();
    (t, d)
}

#[test]
fn dionus_grants_itself_the_ability_but_an_elf_entering_tapped_doesnt_trigger_it() {
    cr!("603.2", "701.26a", "613.1f");
    ruling!(
        "Dionus, Elvish Archdruid",
        "Dionus's ability grants a triggered ability to itself as well as other Elves you control."
    );
    ruling!(
        "Dionus, Elvish Archdruid",
        "A permanent that enters tapped doesn't \"become tapped.\" As such, if it has the ability granted by Dionus, that ability won't trigger."
    );
    let (mut t, d) = dionus(1);
    tap(&mut t, d[0]);
    assert_eq!(triggers_of(&t, "Dionus, Elvish Archdruid").len(), 1);
    t.resolve_all();
    assert!(!t.obj_now(d[0]).tapped);
    assert_eq!(t.counters(d[0], counters::PLUS1), 1);
    let elf = enter_tapped(&mut t, P0, "Llanowar Elves");
    t.settle();
    assert!(t.obj_now(elf).tapped);
    assert!(t.g.stack.is_empty(), "{}", t.dump_log());
}

#[test]
fn dionus_granted_abilities_are_new_for_a_new_dionus_and_resolve_without_it() {
    cr!("400.7", "113.7a");
    ruling!(
        "Dionus, Elvish Archdruid",
        "If Dionus leaves the battlefield and returns to the battlefield in the same turn, or if one Dionus leaves and another Dionus enters, the ability granted by the old one is different from the ability granted by the new one. This means that an Elf that already untapped and got a +1/+1 counter from the previous granted triggered ability could untap and get another +1/+1 counter from the new granted triggered ability if it becomes tapped again later that turn."
    );
    ruling!(
        "Dionus, Elvish Archdruid",
        "If Dionus leaves the battlefield while one or more triggered abilities generated by the ability it grants are still on the stack, those abilities will still resolve as normal."
    );
    let (mut t, d) = dionus(1);
    let elf = t.battlefield(P0, "Llanowar Elves");
    tap(&mut t, elf);
    t.resolve_all();
    assert_eq!(t.counters(elf, counters::PLUS1), 1);
    // Once each turn: tapped again, it stays tapped.
    tap(&mut t, elf);
    assert!(t.g.stack.is_empty());
    // A new Dionus: the Elf has a new ability; tapped, it triggers, and Dionus leaving
    // while the ability waits doesn't stop it.
    kill(&mut t, d[0]);
    let e = t.g.current(elf);
    t.g.untap(e);
    t.battlefield(P0, "Dionus, Elvish Archdruid");
    tap(&mut t, elf);
    assert_eq!(
        triggers_of(&t, "Llanowar Elves").len(),
        1,
        "{}",
        t.dump_log()
    );
    let d2 = t.named_on_battlefield("Dionus, Elvish Archdruid")[0];
    kill(&mut t, d2);
    t.resolve_all();
    assert!(!t.obj_now(elf).tapped);
    assert_eq!(t.counters(elf, counters::PLUS1), 2);
}

#[test]
fn two_dionus_give_two_abilities_that_each_untap_and_add_a_counter() {
    cr!("113.2c", "603.2");
    ruling!(
        "Dionus, Elvish Archdruid",
        "If you somehow control multiple copies of Dionus, each permanent you control will have that many instances of the granted ability. These abilities are not redundant. For example, if you control two copies of Dionus and one becomes tapped during your turn, each of its granted abilities will trigger. When those abilities resolve, each one will untap that Dionus (if it's tapped at the time) and will put a +1/+1 counter on it regardless of whether it's tapped or untapped."
    );
    let (mut t, d) = dionus(2);
    tap(&mut t, d[0]);
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    t.resolve_all();
    assert!(!t.obj_now(d[0]).tapped);
    assert_eq!(t.counters(d[0], counters::PLUS1), 2);
}

// --- Deeproot Pilgrimage ----------------------------------------------------------------------

#[test]
fn deeproot_pilgrimage_triggers_once_for_merfolk_tapped_together() {
    cr!("603.2c");
    ruling!(
        "Deeproot Pilgrimage",
        "If more than one nontoken Merfolk you control becomes tapped at the same time (for example, to pay the cost of Adaptive Gemguard's activated ability), Deeproot Pilgrimage's ability will trigger only once."
    );
    supported("Deeproot Pilgrimage");
    supported("Adaptive Gemguard");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Deeproot Pilgrimage");
    let gem = t.battlefield(P0, "Adaptive Gemguard");
    let a = t.battlefield(P0, "Merfolk of the Pearl Trident");
    let b = t.battlefield(P0, "Merfolk of the Pearl Trident");
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.activate(P0, gem, 0, &[]).expect("Gemguard's ability");
    t.settle();
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    assert_eq!(
        triggers_of(&t, "Deeproot Pilgrimage").len(),
        1,
        "{}",
        t.dump_log()
    );
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Merfolk").len(), 3);
}

// --- Raiyuu, Storm's Edge ------------------------------------------------------------------

#[test]
fn raiyuu_needs_a_lone_attacker_and_gives_no_extra_main_phase() {
    cr!("506.5", "500.8");
    ruling!(
        "Raiyuu, Storm's Edge",
        "A creature attacks alone if it's the only creature declared as an attacker during the declare attackers step. For example, Raiyuu's triggered ability won't trigger if you attack with multiple creatures and all but one of them are removed from combat."
    );
    ruling!(
        "Raiyuu, Storm's Edge",
        "Notably, the triggered ability doesn't give you any additional main phases. This means that you will move directly from the end of combat step of one combat phase to the beginning of combat step of the next one."
    );
    supported("Raiyuu, Storm's Edge");
    // Two attackers: no trigger.
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let r = t.battlefield(P0, "Raiyuu, Storm's Edge");
    let b = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(r, pl(P1)), (b, pl(P1))]);
    assert!(triggers_of(&t, "Raiyuu, Storm's Edge").is_empty());
    // Alone: untapped, and the next combat follows the end of combat step directly.
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let r = t.battlefield(P0, "Raiyuu, Storm's Edge");
    attack_with(&mut t, &[(r, pl(P1))]);
    assert_eq!(triggers_of(&t, "Raiyuu, Storm's Edge").len(), 1);
    t.resolve_all();
    assert!(!t.obj_now(r).tapped);
    let mut steps = vec![];
    let ok = t.g.run_until(10_000, |g| {
        if steps.last() != Some(&g.turn.step) {
            steps.push(g.turn.step);
        }
        g.turn.step == Step::PostcombatMain
    });
    assert!(ok);
    let eoc = steps.iter().position(|s| *s == Step::EndOfCombat).unwrap();
    assert_eq!(steps[eoc + 1], Step::BeginningOfCombat, "{steps:?}");
    assert_eq!(t.g.turn.step, Step::PostcombatMain);
    assert_eq!(
        steps.iter().filter(|s| **s == Step::EndOfCombat).count(),
        2,
        "{steps:?}"
    );
}

// --- Another Round -----------------------------------------------------------------------------

#[test]
fn another_round_triggers_wait_until_it_finishes_and_go_on_the_stack_apnap() {
    cr!("603.3b", "603.3d", "101.4");
    ruling!(
        "Another Round",
        "Abilities that trigger “When [this creature] enters the battlefield” or “When [this creature] leaves the battlefield” will trigger each time that creature enters or leaves the battlefield as a result of Another Round’s effect. If those abilities require targets, you won’t choose those targets until after Another Round finishes resolving, when those abilities are put onto the stack."
    );
    ruling!(
        "Another Round",
        "All abilities that trigger while Another Round is resolving are put on the stack after Another Round is finished resolving. All such abilities controlled by the player whose turn it is are put on the stack first, then those controlled by the next player in turn order, and so on. For each player, they can put their abilities on the stack in the order of their choice. Abilities don’t have to be put on the stack in the order in which they triggered."
    );
    let mut t = TestGame::new(2);
    let fk = t.battlefield(P0, "Flametongue Kavu");
    t.battlefield(P1, "Soul Warden");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 4);
    let ar = t.hand(P0, "Another Round");
    // X = 1: exiled and returned twice.
    t.answer_choose_moved(P0, &[obj(fk)]);
    t.answer_choose_moved(P0, &[obj(fk)]);
    t.cast(P0, ar).x(1).go();
    t.settle();
    let from = t.asked().len();
    t.g.resolve_top();
    // No target was asked while Another Round resolved.
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseTargets { .. })));
    t.settle();
    let kavu = triggers_of(&t, "Flametongue Kavu");
    let warden = triggers_of(&t, "Soul Warden");
    assert_eq!((kavu.len(), warden.len()), (2, 2), "{}", t.dump_log());
    // P0's below P1's.
    let pos = |s: ObjectId| t.g.stack.iter().position(|x| *x == s).unwrap();
    assert!(kavu
        .iter()
        .all(|k| warden.iter().all(|w| pos(*k) < pos(*w))));
    let _ = bears;
}

// --- Lich-Knights' Conquest ------------------------------------------------------------------

#[test]
fn lich_knights_conquest_returns_a_sacrificed_artifact_creature_before_triggers_go_on_the_stack() {
    cr!("603.3", "608.2c");
    ruling!(
        "Lich-Knights' Conquest",
        "An artifact creature or enchantment creature sacrificed to Lich-Knights' Conquest may be one of the cards chosen to be returned."
    );
    ruling!(
        "Lich-Knights' Conquest",
        "If any abilities trigger as you sacrifice permanents, those abilities won't be put onto the stack until after you've returned creature cards from your graveyard to the battlefield."
    );
    supported("Lich-Knights' Conquest");
    supported("Perilous Myr");
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P0, "Perilous Myr");
    let myr2 = t.battlefield(P0, "Perilous Myr");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 4);
    let lk = t.hand(P0, "Lich-Knights' Conquest");
    // Sacrifice both Myr; "that many" (two) creature cards return: both Myr.
    t.answer_choose(P0, &[obj(myr), obj(myr2)]);
    t.answer_choose_moved(P0, &[obj(myr)]);
    t.cast(P0, lk).go();
    t.settle();
    let from = t.asked().len();
    t.g.resolve_top();
    assert_eq!(n_named(&t, P0, "Perilous Myr"), 2, "{}", t.dump_log());
    // The dies triggers are put on the stack now: targets were not chosen during
    // resolution, and the returned Myr can be chosen.
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseTargets { .. })));
    let returned = t.named_on_battlefield("Perilous Myr");
    t.answer_targets(P0, &[obj(returned[0])]);
    t.answer_targets(P0, &[obj(returned[0])]);
    t.settle();
    assert_eq!(triggers_of(&t, "Perilous Myr").len(), 2, "{}", t.dump_log());
    t.resolve_all();
    assert!(t.named_on_battlefield("Perilous Myr").len() < 2);
}

// --- Gluttonous Hellkite ---------------------------------------------------------------------

#[test]
fn gluttonous_hellkite_cast_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.3", "113.7a", "701.6a");
    ruling!(
        "Gluttonous Hellkite",
        "Gluttonous Hellkite's triggered ability will resolve before Gluttonous Hellkite does. If Gluttonous Hellkite is countered or otherwise leaves the stack in response to that triggered ability, the triggered ability will still resolve as normal."
    );
    supported("Gluttonous Hellkite");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 2);
    let h = t.hand(P0, "Gluttonous Hellkite");
    let spell = t.cast(P0, h).x(1).go();
    t.settle();
    assert_eq!(triggers_of(&t, "Gluttonous Hellkite").len(), 1);
    t.lands(P1, "Island", 2);
    let cs = t.hand(P1, "Counterspell");
    t.cast(P1, cs).target(obj(spell)).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Gluttonous Hellkite"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{}", t.dump_log());
}

// --- Hecatomb ------------------------------------------------------------------------------------

/// P0 controls `n` Grizzly Bears and casts Hecatomb; the spell resolves and its enters
/// ability waits on the stack.
fn hecatomb(n: usize) -> (TestGame, Vec<ObjectId>, ObjectId) {
    supported("Hecatomb");
    let mut t = TestGame::new(2);
    let bears = (0..n).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Wastes", 1);
    let h = t.hand(P0, "Hecatomb");
    t.cast(P0, h).go();
    t.resolve();
    t.settle();
    let h = t.named_on_battlefield("Hecatomb")[0];
    (t, bears, h)
}

#[test]
fn hecatomb_sacrifices_on_its_enters_trigger_all_four_or_none() {
    cr!("603.6a", "118.12");
    ruling!(
        "Hecatomb",
        "The sacrifice is done as a triggered ability just after it enters. It is not done on casting."
    );
    ruling!(
        "Hecatomb",
        "If you have less than 4 creatures, you do not partially sacrifice. You pay 4 creatures or none."
    );
    // Four: it waits on the stack with the creatures still there; paying keeps Hecatomb.
    let (mut t, bears, h) = hecatomb(4);
    assert_eq!(triggers_of(&t, "Hecatomb").len(), 1);
    assert!(bears.iter().all(|b| t.on_battlefield(*b)));
    t.answer_yes(P0, true);
    t.answer_choose(P0, &bears.iter().map(|b| obj(*b)).collect::<Vec<_>>());
    t.resolve_all();
    assert!(t.on_battlefield(h));
    assert!(
        bears.iter().all(|b| !t.on_battlefield(*b)),
        "{}",
        t.dump_log()
    );
    // Three: none are sacrificed, Hecatomb is.
    let (mut t, bears, h) = hecatomb(3);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(!t.on_battlefield(h));
    assert!(t.in_graveyard(P0, "Hecatomb"));
    assert!(bears.iter().all(|b| t.on_battlefield(*b)));
}

#[test]
fn hecatomb_choice_is_made_on_resolution_even_if_you_lost_control_of_it() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Hecatomb",
        "You choose whether to sacrifice creatures or not on resolution. If not, then you sacrifice Hecatomb. You can choose to not sacrifice creatures if you no longer control this card on resolution."
    );
    let (mut t, bears, h) = hecatomb(4);
    give_control(&mut t, h, P1);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert!(t.on_battlefield(h), "{}", t.dump_log());
    assert_eq!(controller(&t, h), P1);
    assert!(bears.iter().all(|b| t.on_battlefield(*b)));
}

// --- Phyrexian Dreadnought -----------------------------------------------------------------------

#[test]
fn phyrexian_dreadnought_gone_before_its_trigger_resolves_still_offers_both_options() {
    cr!("608.2c", "118.12");
    ruling!(
        "Phyrexian Dreadnought",
        "If the Dreadnought is no longer on the battlefield when the triggered ability resolves, you may still choose either option."
    );
    for pay in [true, false] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, "Colossal Dreadmaw");
        let b = t.battlefield(P0, "Colossal Dreadmaw");
        let dn = t.enter(P0, "Phyrexian Dreadnought");
        t.settle();
        assert_eq!(triggers_of(&t, "Phyrexian Dreadnought").len(), 1);
        destroy(&mut t, dn);
        t.answer_yes(P0, pay);
        t.answer(P0, DecisionKind::X, Answer::Number(2));
        t.answer_choose(P0, &[obj(a), obj(b)]);
        t.resolve_all();
        assert_eq!(
            !t.on_battlefield(a) && !t.on_battlefield(b),
            pay,
            "{}",
            t.dump_log()
        );
        assert!(t.in_graveyard(P0, "Phyrexian Dreadnought"));
    }
}

// --- Etrata, the Silencer -----------------------------------------------------------------------

/// P0's Etrata deals combat damage to P1, who controls `p1` creatures; with its trigger on
/// the stack (targeting P1's first creature, if any). Returns the game, Etrata and P1's
/// creatures.
fn etrata_hits(t: &mut TestGame, p1: &[&str]) -> (ObjectId, Vec<ObjectId>) {
    supported("Etrata, the Silencer");
    let e = t.battlefield(P0, "Etrata, the Silencer");
    let cs: Vec<ObjectId> = p1.iter().map(|n| t.battlefield(P1, n)).collect();
    t.set_step(P0, Step::PrecombatMain);
    if let Some(c) = cs.first() {
        t.answer_targets(P0, &[obj(*c)]);
    }
    attack_with(t, &[(e, pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    (e, cs)
}

fn in_library(t: &TestGame, p: PlayerId, name: &str) -> bool {
    t.g.player(p)
        .library
        .iter()
        .any(|c| t.g.obj(*c).chars.name == name)
}

#[test]
fn etrata_that_left_stays_where_it_went_and_the_library_is_still_shuffled() {
    cr!("608.2c", "400.7", "701.24a");
    ruling!(
        "Etrata, the Silencer",
        "If Etrata leaves the battlefield while its triggered ability is on the stack, it remains in its new zone and Etrata's owner shuffles their library."
    );
    let mut t = TestGame::new(2);
    let (e, _) = etrata_hits(&mut t, &["Grizzly Bears"]);
    assert_eq!(triggers_of(&t, "Etrata, the Silencer").len(), 1);
    kill(&mut t, e);
    let from = t.g.turn_events.len();
    t.resolve_all();
    assert!(
        t.in_graveyard(P0, "Etrata, the Silencer"),
        "{}",
        t.dump_log()
    );
    assert!(!in_library(&t, P0, "Etrata, the Silencer"));
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(shuffles_since(&t, P0, from), 1);
}

#[test]
fn etrata_does_nothing_without_a_legal_target() {
    cr!("608.2b", "603.3d");
    ruling!(
        "Etrata, the Silencer",
        "If the player controls no creatures, Etrata's ability doesn't resolve. Etrata remains on the battlefield and its owner doesn't shuffle their library. The same is true if the target creature is an illegal target when Etrata's ability tries to resolve."
    );
    // No creatures: nothing goes on the stack.
    let mut t = TestGame::new(2);
    let (e, _) = etrata_hits(&mut t, &[]);
    let from = t.g.turn_events.len();
    t.resolve_all();
    assert!(t.on_battlefield(e));
    assert_eq!(shuffles_since(&t, P0, from), 0);
    // The target leaves first: the ability doesn't resolve.
    let mut t = TestGame::new(2);
    let (e, cs) = etrata_hits(&mut t, &["Grizzly Bears"]);
    kill(&mut t, cs[0]);
    let from = t.g.turn_events.len();
    t.resolve_all();
    assert!(t.on_battlefield(e), "{}", t.dump_log());
    assert_eq!(shuffles_since(&t, P0, from), 0);
}

#[test]
fn etrata_exiling_a_token_doesnt_count_toward_three_hit_cards() {
    cr!("111.7", "704.5d");
    ruling!(
        "Etrata, the Silencer",
        "If Etrata's ability exiles a token creature, it won't count to determine whether three cards are exiled with hit counters."
    );
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        let c = t.exile(P1, "Hill Giant");
        t.g.objects[c.0 as usize].counters.insert("hit".into(), 1);
    }
    let soldier = create_token(&mut t, P1, "Soldier");
    supported("Etrata, the Silencer");
    let e = t.battlefield(P0, "Etrata, the Silencer");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[obj(soldier)]);
    attack_with(&mut t, &[(e, pl(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(soldier));
    assert!(!t.has_lost(P1), "{}", t.dump_log());
}

#[test]
fn etrata_as_a_commander_may_go_to_the_command_zone_and_the_library_is_shuffled() {
    cr!("903.9a", "608.2c");
    ruling!(
        "Etrata, the Silencer",
        "In a Commander game, if Etrata is your commander, you may put it into the command zone as its ability resolves. You'll still shuffle your library."
    );
    let mut t = commander_game(2);
    supported("Etrata, the Silencer");
    let e = t.battlefield(P0, "Etrata, the Silencer");
    make_commander(&mut t, e);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(e, pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    let from = t.g.turn_events.len();
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(
        t.g.find_in_zone(Zone::Command, "Etrata, the Silencer")
            .len(),
        1,
        "{}",
        t.dump_log()
    );
    assert!(!in_library(&t, P0, "Etrata, the Silencer"));
    assert_eq!(shuffles_since(&t, P0, from), 1);
}

// --- Timothar, Baron of Bats -----------------------------------------------------------------

/// P0 controls Timothar and an Arrogant Vampire, which dies; P0 pays {1} and exiles it
/// (unless `before` acts on the card while the ability waits). Returns the game, the
/// Vampire card and the Bats.
fn timothar(before: fn(&mut TestGame, ObjectId)) -> (TestGame, ObjectId, Vec<ObjectId>) {
    use mtg_engine::mana::ManaType;
    supported("Timothar, Baron of Bats");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Timothar, Baron of Bats");
    let v = t.battlefield(P0, "Arrogant Vampire");
    add(&mut t, P0, ManaType::C, 1);
    t.answer_yes(P0, true);
    destroy(&mut t, v);
    t.settle();
    assert_eq!(triggers_of(&t, "Timothar, Baron of Bats").len(), 1);
    before(&mut t, v);
    t.resolve_all();
    let bats = with_subtype(&t, P0, "Bat");
    for b in &bats {
        t.g.objects[b.0 as usize].summoning_sick = false;
    }
    (t, v, bats)
}

#[test]
fn timothar_cant_pay_if_the_card_cant_be_exiled() {
    cr!("608.2c", "400.7");
    ruling!(
        "Timothar, Baron of Bats",
        "If you can't exile the card, perhaps because another effect moved the card before the triggered ability started resolving, you can't pay {1} and won't create a token."
    );
    let (t, _, bats) = timothar(|t, v| {
        mv(t, v, Zone::Hand(P0));
    });
    assert!(bats.is_empty(), "{}", t.dump_log());
    assert!(t.in_hand(P0, "Arrogant Vampire"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 1, "the {{1}} wasn't paid");
}

#[test]
fn timothar_bat_is_sacrificed_even_if_the_card_left_exile() {
    cr!("608.2c", "607.2a");
    ruling!(
        "Timothar, Baron of Bats",
        "If the exiled card is somehow no longer in exile when the Bat token's triggered ability resolves, you will still sacrifice the Bat token."
    );
    let (mut t, v, bats) = timothar(|_, _| {});
    assert_eq!(bats.len(), 1);
    attack_with(&mut t, &[(bats[0], pl(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    mv(&mut t, v, Zone::Hand(P0));
    t.resolve_all();
    assert!(!t.on_battlefield(bats[0]));
    assert!(t.named_on_battlefield("Arrogant Vampire").is_empty());
    assert!(t.in_hand(P0, "Arrogant Vampire"));
}

#[test]
fn timothar_bat_ability_belongs_to_the_bats_controller_and_copies_dont_have_it() {
    cr!("603.3a", "707.2", "613.1a");
    ruling!(
        "Timothar, Baron of Bats",
        "The controller of the Bat token as it deals combat damage to a player will be the controller of its triggered ability, and the exiled card will be returned to the battlefield under that player's control."
    );
    ruling!(
        "Timothar, Baron of Bats",
        "If another creature enters the battlefield as a copy of the Bat token or another effect creates a copy of the Bat token, it will not have the triggered ability granted by Timothar."
    );
    supported("Cackling Counterpart");
    // A copy of the Bat (Cackling Counterpart) connects: nothing happens.
    let (mut t, _, bats) = timothar(|_, _| {});
    t.lands(P0, "Island", 3);
    let cc = t.hand(P0, "Cackling Counterpart");
    t.cast(P0, cc).target(obj(bats[0])).go();
    t.resolve_all();
    let copy = *with_subtype(&t, P0, "Bat")
        .iter()
        .find(|b| **b != bats[0])
        .expect("the copy");
    t.g.objects[copy.0 as usize].summoning_sick = false;
    attack_with(&mut t, &[(copy, pl(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(t.on_battlefield(copy));
    assert!(t.in_exile("Arrogant Vampire"));
    // P1 gains control of the Bat and connects with it: the card returns under P1's control.
    let (mut t, _, bats) = timothar(|_, _| {});
    give_control(&mut t, bats[0], P1);
    let bat = t.g.current(bats[0]);
    t.g.objects[bat.0 as usize].summoning_sick = false;
    t.advance_to(P1, Step::PrecombatMain);
    attack_with(&mut t, &[(bats[0], pl(P0))]);
    t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P1, Step::CombatDamage);
    t.settle();
    let trig = t.g.stack.last().copied().expect("the Bat's ability");
    assert_eq!(stack_controller(&t, trig), P1);
    t.resolve_all();
    assert_eq!(n_named(&t, P1, "Arrogant Vampire"), 1, "{}", t.dump_log());
}

// --- Zara, Renegade Recruiter; Hellkite Courser ------------------------------------------------

#[test]
fn zara_returns_the_creature_only_if_its_still_on_the_battlefield() {
    cr!("603.7c", "400.7");
    ruling!(
        "Zara, Renegade Recruiter",
        "The creature is returned to its owner's hand only if it's still on the battlefield as the delayed triggered ability resolves. If it's somewhere else at that time, it stays where it is."
    );
    supported("Zara, Renegade Recruiter");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let z = t.battlefield(P0, "Zara, Renegade Recruiter");
    let card = t.hand(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(card)]);
    attack_with(&mut t, &[(z, pl(P1))]);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    kill(&mut t, bears[0]);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"), "{}", t.dump_log());
    assert!(!t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn hellkite_courser_returns_the_commander_only_from_the_battlefield() {
    cr!("603.7c", "903.9a");
    ruling!(
        "Hellkite Courser",
        "You'll return the commander to the command zone only if it's still on the battlefield as the delayed triggered ability resolves. If it's in another zone, it will stay where it is."
    );
    supported("Hellkite Courser");
    let mut t = commander_game(2);
    let c = t.command(P0, "Hill Giant");
    make_commander(&mut t, c);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(c)]);
    t.enter(P0, "Hellkite Courser");
    t.resolve_all();
    let giant = t.named_on_battlefield("Hill Giant");
    assert_eq!(giant.len(), 1);
    // It dies; its owner leaves it in the graveyard (CR 903.9a).
    t.answer_yes(P0, false);
    kill(&mut t, giant[0]);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"), "{}", t.dump_log());
    assert!(t.g.find_in_zone(Zone::Command, "Hill Giant").is_empty());
}

// --- Noctis, Heir Apparent -------------------------------------------------------------------

#[test]
fn noctis_returns_after_attackers_are_declared_even_if_none_are() {
    cr!("508.1", "508.4", "603.7");
    ruling!(
        "Noctis, Heir Apparent",
        "If Noctis is exiled with its last ability, it will return during its owner's next declare attackers step after that player has finished declaring attackers. It will return even if its owner declares no attackers."
    );
    supported("Noctis, Heir Apparent");
    for with_bears in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let noctis = t.battlefield(P0, "Noctis, Heir Apparent");
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.lands(P0, "Mountain", 3);
        t.activate(P0, noctis, 0, &[]).unwrap();
        t.resolve_all();
        assert!(t.in_exile("Noctis, Heir Apparent"));
        let attackers = if with_bears {
            vec![(bears, pl(P1))]
        } else {
            vec![]
        };
        // Noctis isn't among the creatures that could be declared.
        let from = t.asked().len();
        t.answer(P0, DecisionKind::Attackers, Answer::Attackers(attackers));
        t.advance_to(P0, Step::DeclareAttackers);
        let declared = asked_since(&t, from)
            .into_iter()
            .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::DeclareAttackers { .. }));
        assert!(declared);
        t.resolve_all();
        let n = t.named_on_battlefield("Noctis, Heir Apparent");
        assert_eq!(n.len(), 1);
        assert!(t.g.is_attacking(n[0]));
        assert_eq!(t.g.is_attacking(bears), with_bears);
    }
}

// --- Stumpsquall Hydra -------------------------------------------------------------------------

#[test]
fn stumpsquall_hydra_distributes_among_commanders_on_the_battlefield() {
    cr!("608.2d", "903.3", "122.6");
    ruling!(
        "Stumpsquall Hydra",
        "If Stumpsquall Hydra leaves the battlefield before its triggered ability resolves, you can still distribute the counters among any number of commanders, including none of them."
    );
    ruling!(
        "Stumpsquall Hydra",
        "You can't put +1/+1 counters on commanders that aren't on the battlefield."
    );
    ruling!(
        "Stumpsquall Hydra",
        "You may put +1/+1 counters on commanders that aren't currently creatures (such as Tevesh Szat, Doom of Fools)."
    );
    supported("Stumpsquall Hydra");
    let mut t = commander_game(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    make_commander(&mut t, bears);
    // A noncreature commander (a planeswalker) and one in the command zone.
    let walker = t.battlefield(P0, "Tevesh Szat, Doom of Fools");
    make_commander(&mut t, walker);
    let away = t.command(P0, "Hill Giant");
    make_commander(&mut t, away);
    t.lands(P0, "Forest", 6);
    let card = t.hand(P0, "Stumpsquall Hydra");
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, card).x(3).go();
    t.resolve();
    let hydra = t.g.current(card);
    t.settle();
    assert_eq!(triggers_of(&t, "Stumpsquall Hydra").len(), 1);
    // The Hydra leaves first; the counters still go on the commanders.
    destroy(&mut t, hydra);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(bears), obj(walker)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 2, "{}", t.dump_log());
    assert_eq!(t.counters(walker, counters::PLUS1), 1);
    assert_eq!(t.counters(away, counters::PLUS1), 0);
    // The commander in the command zone was never a choice.
    let choices: Vec<Vec<Entity>> = asked_since(&t, from)
        .into_iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { candidates, .. } => Some(candidates),
            _ => None,
        })
        .collect();
    assert!(!choices.is_empty());
    assert!(choices.iter().all(|c| !c.contains(&obj(away))));
}

// --- Sengir, the Dark Baron -------------------------------------------------------------------

#[test]
fn sengir_triggers_however_the_player_lost() {
    cr!("104.3a", "104.3c", "104.3d", "704.5a", "704.5c");
    ruling!(
        "Sengir, the Dark Baron",
        "Sengir, the Dark Baron's third ability will trigger no matter how the player lost the game: due to a state-based action (having 0 or less life, trying to draw a card from an empty library, having 10 or more poison counters), because of a spell or effect that states they lose the game, because your awesome might has left them with no choice other than concession, or for any other reason."
    );
    supported("Sengir, the Dark Baron");
    for how in 0..3 {
        let mut t = TestGame::new(3);
        // A turn that began (so life totals as it began are recorded).
        t.advance_to(P1, Step::Upkeep);
        t.advance_to(P0, Step::PrecombatMain);
        t.battlefield(P0, "Sengir, the Dark Baron");
        match how {
            0 => {
                t.g.players[P1.idx()]
                    .counters
                    .insert(counters::POISON.into(), 10);
                t.settle();
            }
            1 => concede(&mut t, P1),
            _ => {
                t.g.players[P1.idx()].life = 0;
                t.settle();
            }
        }
        assert!(t.has_lost(P1));
        t.resolve_all();
        assert_eq!(t.life(P0), 40, "case {how}: {}", t.dump_log());
    }
}

// --- Khabál Ghoul, Scavenging Ghoul -------------------------------------------------------------

#[test]
fn ghouls_count_every_creature_that_died_this_turn_even_before_they_entered() {
    cr!("700.4", "111.7");
    ruling!(
        "Khabál Ghoul",
        "The effect counts all creatures put into all graveyards from the battlefield during the turn. This includes creature tokens put into a graveyard as well as creatures put into a graveyard before Khabál Ghoul entered the battlefield."
    );
    ruling!(
        "Scavenging Ghoul",
        "The Ghoul still gets to claim counters even if it enters after the creatures died."
    );
    supported("Khabál Ghoul");
    supported("Scavenging Ghoul");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let tok = create_token(&mut t, P0, "Soldier");
    kill(&mut t, bears);
    kill(&mut t, tok);
    let kg = t.battlefield(P0, "Khabál Ghoul");
    let sg = t.battlefield(P0, "Scavenging Ghoul");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.counters(kg, counters::PLUS1), 2, "{}", t.dump_log());
    assert_eq!(t.counters(sg, "corpse"), 2);
}

// --- Indominus Rex, Alpha; Nissa, Ascended Animist --------------------------------------------

#[test]
fn indominus_rex_and_nissa_use_their_last_known_counters() {
    cr!("608.2h", "113.7a", "606.3");
    ruling!(
        "Indominus Rex, Alpha",
        "If Indominus Rex isn't on the battlefield as its last ability resolves, use the number of counters that were on it when it was last on the battlefield to determine how many cards to draw."
    );
    ruling!(
        "Nissa, Ascended Animist",
        "If Nissa is no longer on the battlefield at the time her first ability resolves, use the number of loyalty counters she had when she last existed on the battlefield to determine the value of X."
    );
    supported("Indominus Rex, Alpha");
    supported("Nissa, Ascended Animist");
    // Rex discards Serra Angel (flying and vigilance counters) and is destroyed in response.
    let mut t = TestGame::new(2);
    let angel = t.hand(P0, "Serra Angel");
    t.answer_choose(P0, &[obj(angel)]);
    let rex = t.enter(P0, "Indominus Rex, Alpha");
    t.settle();
    assert_eq!(t.counters(rex, "flying") + t.counters(rex, "vigilance"), 2);
    destroy(&mut t, rex);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2, "{}", t.dump_log());
    // Nissa with 7 loyalty activates +1 (8) and is destroyed in response: an 8/8.
    let mut t = TestGame::new(2);
    let n = t.battlefield(P0, "Nissa, Ascended Animist");
    t.g.objects[n.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 7);
    t.activate(P0, n, 0, &[]).expect("+1");
    destroy(&mut t, n);
    t.resolve_all();
    let horrors = with_subtype(&t, P0, "Horror");
    assert_eq!(horrors.len(), 1, "{}", t.dump_log());
    assert_eq!(t.pt(horrors[0]), (8, 8));
}

// --- Boreal Outrider ---------------------------------------------------------------------------

#[test]
fn boreal_outrider_gives_one_counter_per_trigger() {
    cr!("603.2", "614.1c");
    ruling!(
        "Boreal Outrider",
        "A creature affected by Boreal Outrider’s triggered ability will enter with only one additional +1/+1 counter on it, even if you spent more than one snow mana of the appropriate color(s) to cast it."
    );
    ruling!(
        "Boreal Outrider",
        "If you control multiple Boreal Outriders, each of their triggered abilities will trigger separately. A creature could enter the battlefield with multiple +1/+1 counters on it this way, even if you spent only a single snow mana of the appropriate color(s) to cast the creature spell."
    );
    supported("Boreal Outrider");
    for (outriders, snow_forests, expected) in [(1, 2, 1), (2, 1, 2)] {
        let mut t = TestGame::new(2);
        for _ in 0..outriders {
            t.battlefield(P0, "Boreal Outrider");
        }
        t.lands(P0, "Snow-Covered Forest", snow_forests);
        t.lands(P0, "Forest", 2 - snow_forests);
        let bears = t.hand(P0, "Grizzly Bears");
        t.cast(P0, bears).go();
        t.resolve_all();
        assert_eq!(
            t.counters(t.g.current(bears), counters::PLUS1),
            expected,
            "{outriders} Outriders, {snow_forests} snow"
        );
    }
}
