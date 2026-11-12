//! Rulings batch P139 — triggered abilities around sacrifices: when they trigger and go on
//! the stack (Ob Nixilis, Unshackled, Ulamog, the Infinite Gyre, attack triggers that
//! resolve before blockers are declared), what they trigger on (any sacrifice, any Angel),
//! and delayed "sacrifice it at the beginning of the next end step" abilities (Angrath,
//! Slave of Bolas, Spinal Embrace).

use crate::r_p137_common::obj;
use crate::r_s01_common::{attack_with, give_mana_for, supported};
use crate::r_s02_common::destroy;
use crate::r_s04_common::{add_mana, top_of_stack};
use crate::r_s06_common::give_control;
use crate::r_s17_common::become_copy;
use crate::r_s25_common::cast_new;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn controller_of(t: &TestGame, id: ObjectId) -> PlayerId {
    t.g.obj(id).controller
}

/// `p` activates Evolving Wilds (sacrificing it to search for a basic land card).
fn fetch(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let wilds = t.battlefield(p, "Evolving Wilds");
    t.library_top(p, "Forest");
    t.activate(p, wilds, 0, &[]).unwrap();
    wilds
}

#[test]
fn ob_nixilis_triggers_after_the_search_resolves_with_the_other_triggers_in_apnap_order() {
    cr!("603.3b", "701.23a", "101.4");
    ruling!(
        "Ob Nixilis, Unshackled",
        "The first triggered ability won't be put on the stack until after the spell or ability causing the opponent to search their library finishes resolving. Notably, if that spell or ability causes any other abilities to trigger (for example, if the opponent searched for a creature card and put it onto the battlefield), those abilities and Ob Nixilis's triggered ability will go on the stack together. The active player puts all of their abilities on the stack in any order, then each other player in turn order does the same. The last ability put onto the stack this way will be the first to resolve, and so on."
    );
    supported("Ob Nixilis, Unshackled");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Ob Nixilis, Unshackled");
    let lynx = t.battlefield(P1, "Steppe Lynx");
    fetch(&mut t, P1);
    t.settle();
    assert_eq!(t.stack_len(), 1, "only the Evolving Wilds ability");
    t.resolve();
    // The landfall trigger (P1, the active player) and Ob Nixilis's (P0) went on the
    // stack together: Ob Nixilis's on top.
    assert_eq!(t.stack_len(), 2);
    let top = top_of_stack(&t);
    assert_eq!(controller_of(&t, top), P0);
    assert_eq!(controller_of(&t, t.g.stack[0]), P1);
    t.resolve();
    assert_eq!(t.life(P1), 10);
    assert!(!t.on_battlefield(lynx));
}

#[test]
fn sacrifice_triggers_trigger_on_any_sacrifice() {
    cr!("701.21a", "603.2");
    ruling!(
        "Nazgûl Battle-Mace",
        "The last ability granted by Nazgûl Battle-Mace triggers whenever an opponent sacrifices a nontoken permanent for any reason, not just due to the annihilator ability."
    );
    supported("Nazgûl Battle-Mace");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mace = t.battlefield(P0, "Nazgûl Battle-Mace");
    t.g.attach(mace, Entity::Object(bears));
    // P1 sacrifices Evolving Wilds to its own ability and doesn't pay 3 life.
    t.answer_yes(P1, false);
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(false));
    fetch(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Evolving Wilds").len(), 1);
    let wilds = t.named_on_battlefield("Evolving Wilds")[0];
    assert_eq!(controller_of(&t, wilds), P0);
    assert_eq!(t.life(P1), 20);

    ruling!(
        "It That Betrays",
        "When the second ability resolves, you must return the card to the battlefield, even if you don’t want to."
    );
    supported("It That Betrays");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "It That Betrays");
    // P0 would decline if asked.
    t.answer_yes(P0, false);
    fetch(&mut t, P1);
    t.resolve_all();
    let wilds = t.named_on_battlefield("Evolving Wilds");
    assert_eq!(wilds.len(), 1);
    assert_eq!(controller_of(&t, wilds[0]), P0);

    ruling!(
        "Thraximundar",
        "Thraximundar's last ability triggers whenever any player (including you) sacrifices a creature for any reason. It doesn't trigger only when a creature is sacrificed due to its middle ability. Note that Thraximundar itself doesn't allow you to sacrifice any creatures."
    );
    supported("Thraximundar");
    let mut t = TestGame::new(2);
    let thrax = t.battlefield(P0, "Thraximundar");
    for p in [P0, P1] {
        let altar = t.battlefield(p, "Ashnod's Altar");
        let victim = t.battlefield(p, "Grizzly Bears");
        t.answer_choose(p, &[obj(victim)]);
        t.answer_yes(P0, true);
        t.activate(p, altar, 0, &[]).unwrap();
        t.resolve_all();
    }
    assert_eq!(t.counters(thrax, "+1/+1"), 2);
}

#[test]
fn rampage_of_the_valkyries_triggers_for_any_angel_you_control() {
    cr!("603.6c", "700.4");
    ruling!(
        "Rampage of the Valkyries",
        "The last ability of Rampage of the Valkyries will trigger whenever any Angel you control dies, not just the one created by the first ability."
    );
    supported("Rampage of the Valkyries");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rampage of the Valkyries");
    let angel = t.battlefield(P0, "Serra Angel");
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, angel);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn attack_triggers_resolve_before_blockers_are_declared() {
    cr!("508.1m", "508.2", "509.1");
    ruling!(
        "Thraximundar",
        "Thraximundar's middle ability resolves during the declare attackers step. The defending player sacrifices a creature before it would get a chance to block."
    );
    supported("Thraximundar");
    let mut t = TestGame::new(2);
    let thrax = t.battlefield(P0, "Thraximundar");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, false);
    attack_with(&mut t, &[(thrax, Entity::Player(P1))]);
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(!t.on_battlefield(bears));

    ruling!(
        "Vaevictis Asmadi, the Dire",
        "Vaevictis's triggered ability resolves during the declare attackers step, before blockers are chosen. A creature targeted by its ability won't be around to block, and creatures put onto the battlefield with its ability may be able to do so."
    );
    supported("Vaevictis Asmadi, the Dire");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Vaevictis Asmadi, the Dire");
    let land = t.battlefield(P0, "Swamp");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.library_top(P1, "Giant Spider");
    t.answer_targets(P0, &[obj(land)]);
    t.answer_targets(P0, &[obj(blocker)]);
    attack_with(&mut t, &[(dragon, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareAttackers);
    assert!(!t.on_battlefield(blocker));
    let spider = t.named_on_battlefield("Giant Spider");
    assert_eq!(spider.len(), 1);
    // The Giant Spider (reach) blocks the Dragon.
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(spider[0], dragon)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(crate::r_s10_common::blocking(&t, spider[0]));
}

#[test]
fn ulamogs_cast_trigger_resolves_even_if_the_spell_is_countered() {
    cr!("603.2", "601.2i", "701.6a");
    ruling!(
        "Ulamog, the Infinite Gyre",
        "Ulamog's ability triggers as you cast it, and that ability resolves before the spell itself. It resolves even if that spell is countered."
    );
    supported("Ulamog, the Infinite Gyre");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    add_mana(&mut t, P0, ManaType::C, 11);
    let card = t.hand(P0, "Ulamog, the Infinite Gyre");
    t.answer_targets(P0, &[obj(ring)]);
    let ulamog = t.cast(P0, card).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // P1 counters Ulamog with the trigger still on the stack.
    give_mana_for(&mut t, P1, "Counterspell");
    let cs = t.hand(P1, "Counterspell");
    t.cast_with(P1, cs, &[obj(ulamog)]).unwrap();
    t.resolve();
    assert!(t.in_graveyard(P0, "Ulamog, the Infinite Gyre") || t.library_size(P0) > 30);
    assert!(t.on_battlefield(ring), "the trigger hasn't resolved yet");
    t.resolve_all();
    assert!(!t.on_battlefield(ring));
    assert!(t.named_on_battlefield("Ulamog, the Infinite Gyre").is_empty());
}

// ---------------------------------------------------------------------------------------
// Witch of the Moors
// ---------------------------------------------------------------------------------------

fn gain_life(t: &mut TestGame, p: PlayerId, n: i32) {
    t.g.gain_life(p, n as u32);
    t.g.flush_events();
}

#[test]
fn witch_of_the_moors_looks_at_the_whole_turn_and_triggers_once() {
    cr!("603.4", "119.3");
    ruling!(
        "Witch of the Moors",
        "The triggered ability looks at whether you've gained life in the turn, even if Witch of the Moors wasn't on the battlefield when you gained life. It doesn't care if you also lost life, even if you lost more life than you gained."
    );
    ruling!(
        "Witch of the Moors",
        "The triggered ability triggers just once, no matter how much life you've gained."
    );
    supported("Witch of the Moors");
    let mut t = TestGame::new(2);
    gain_life(&mut t, P0, 2);
    gain_life(&mut t, P0, 3);
    t.g.lose_life(P0, 10);
    t.g.flush_events();
    t.battlefield(P0, "Witch of the Moors");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1, "one trigger");
    t.resolve_all();
    assert_eq!(
        usize::from(t.on_battlefield(a)) + usize::from(t.on_battlefield(b)),
        1,
        "one creature sacrificed"
    );
}

#[test]
fn witch_of_the_moors_needs_no_target_but_an_illegal_one_stops_everything() {
    cr!("608.2b", "115.1");
    ruling!(
        "Witch of the Moors",
        "You don't have to choose a target creature card at all. If you do and the target card is an illegal target by the time the ability tries to resolve, the ability won't resolve. Your opponents won't sacrifice creatures."
    );
    supported("Witch of the Moors");
    // No target.
    let mut t = TestGame::new(2);
    gain_life(&mut t, P0, 1);
    t.battlefield(P0, "Witch of the Moors");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // A target that's gone: nothing happens.
    let mut t = TestGame::new(2);
    gain_life(&mut t, P0, 1);
    t.battlefield(P0, "Witch of the Moors");
    let card = t.graveyard(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(card)]);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    crate::r_s05_common::move_to(&mut t, card, Zone::Exile);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(!t.in_hand(P0, "Hill Giant"));
}

#[test]
fn azlask_x_is_fixed_as_the_ability_resolves() {
    cr!("608.2h", "611.2c");
    ruling!(
        "Azlask, the Swelling Scourge",
        "The value of X is determined only once, as Azlask's last ability resolves."
    );
    supported("Azlask, the Swelling Scourge");
    let mut t = TestGame::new(2);
    let azlask = t.battlefield(P0, "Azlask, the Swelling Scourge");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s13_common::add(&mut t, Entity::Player(P0), "experience", 2);
    for ty in [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G] {
        add_mana(&mut t, P0, ty, 1);
    }
    t.activate(P0, azlask, 0, &[]).unwrap();
    t.settle();
    // More experience with the ability on the stack: counted.
    crate::r_s13_common::add(&mut t, Entity::Player(P0), "experience", 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    // More experience afterwards: not counted.
    crate::r_s13_common::add(&mut t, Entity::Player(P0), "experience", 4);
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.pt(azlask), (5, 5));
}

// ---------------------------------------------------------------------------------------
// Delayed "sacrifice it" triggers
// ---------------------------------------------------------------------------------------

/// P0's Angrath (with 5 loyalty) uses its −3 on `target`.
fn angrath_minus_three(t: &mut TestGame, target: ObjectId) {
    let angrath = t.battlefield(P0, "Angrath, the Flame-Chained");
    t.g.objects[angrath.0 as usize]
        .counters
        .insert("loyalty".into(), 5);
    t.activate(P0, angrath, 1, &[obj(target)]).unwrap();
    t.resolve_all();
}

#[test]
fn angrath_checks_mana_value_only_at_the_end_step() {
    cr!("603.7", "608.2h", "707.2");
    ruling!(
        "Angrath, the Flame-Chained",
        "Whether the target creature has mana value 3 or less is checked only as the delayed triggered ability from Angrath’s second ability resolves during the end step."
    );
    supported("Angrath, the Flame-Chained");
    // Hill Giant (mana value 4) becomes a copy of Grizzly Bears (mana value 2): sacrificed.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let model = t.battlefield(P1, "Grizzly Bears");
    angrath_minus_three(&mut t, giant);
    assert_eq!(controller_of(&t, giant), P0);
    become_copy(&mut t, giant, model);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    // Grizzly Bears becomes a copy of Hill Giant: not sacrificed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let model = t.battlefield(P1, "Hill Giant");
    angrath_minus_three(&mut t, bears);
    become_copy(&mut t, bears, model);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn angrath_and_slave_of_bolas_can_target_your_own_or_untapped_creatures() {
    cr!("115.1", "701.26b");
    ruling!(
        "Angrath, the Flame-Chained",
        "You can target and gain control of an untapped creature with Angrath’s second ability. You can also untap a creature you already control and give it haste."
    );
    supported("Angrath, the Flame-Chained");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Hill Giant");
    angrath_minus_three(&mut t, theirs);
    assert_eq!(controller_of(&t, theirs), P0);
    let mine = t.battlefield_sick(P0, "Hill Giant");
    t.g.objects[mine.0 as usize].tapped = true;
    angrath_minus_three(&mut t, mine);
    assert!(!t.obj_now(mine).tapped);
    assert!(crate::r_s06_common::has_kw(&t, mine, mtg_engine::keywords::KeywordKind::Haste));

    ruling!("Slave of Bolas", "You may target a creature you already control.");
    supported("Slave of Bolas");
    let mut t = TestGame::new(2);
    let mine = t.battlefield_sick(P0, "Grizzly Bears");
    t.g.objects[mine.0 as usize].tapped = true;
    cast_new(&mut t, P0, "Slave of Bolas", &[obj(mine)]);
    t.resolve_all();
    assert!(!t.obj_now(mine).tapped);
    assert!(crate::r_s06_common::has_kw(&t, mine, mtg_engine::keywords::KeywordKind::Haste));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn spinal_embrace_sacrifices_only_if_you_still_control_it() {
    cr!("603.7", "701.21a");
    ruling!(
        "Spinal Embrace",
        "You must sacrifice the creature if you still control it as the delayed triggered ability resolves."
    );
    supported("Spinal Embrace");
    for keep in [true, false] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::BeginningOfCombat);
        let giant = t.battlefield(P1, "Hill Giant");
        cast_new(&mut t, P0, "Spinal Embrace", &[obj(giant)]);
        t.resolve_all();
        assert_eq!(controller_of(&t, giant), P0);
        if !keep {
            give_control(&mut t, giant, P1);
        }
        // P0 would decline if asked.
        t.answer_yes(P0, false);
        t.advance_to(P0, Step::End);
        t.resolve_all();
        assert_eq!(t.on_battlefield(giant), !keep);
        assert_eq!(t.life(P0), if keep { 23 } else { 20 });
    }
}

#[allow(dead_code)]
fn asked_kinds(t: &TestGame, from: usize) -> Vec<String> {
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| !matches!(d, Decision::Priority { .. }))
        .map(|(p, d)| format!("{p:?} {d:?}"))
        .collect()
}
