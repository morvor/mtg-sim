//! Rulings batch P173 — copies of spells and triggered abilities (CR 707.10): the copy
//! keeps the original's modes, X, division of damage and additional or alternative cost
//! choices (CR 707.10, 707.10c, 601.2b), new targets keep the number of targets (CR
//! 115.7f), choices and costs on resolution are made separately for the copy (CR 608.2d,
//! 118.12), a copy isn't cast (CR 707.10, 603.2), and cast triggers resolve before their
//! spell, even if it was countered (CR 603.3, 608.2h).

use crate::r_p212_common::back_face;
use crate::r_s01_common::{attack_with, give_mana_for, supported, tokens, triggers_on_stack};
use crate::r_s04_common::add_mana;
use crate::r_s06_common::activate_containing;
use crate::r_s20_common::tap_for_mana;
use crate::r_s24_common::tapped;
use crate::r_s25_common::{
    abilities_from, change_copy_targets, keep_copy_targets, spell_copies, targets_of, x_of,
};
use crate::r_s26_common::modes_on_stack;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// The division of the stack object's first target slot.
fn division(t: &TestGame, id: ObjectId) -> Vec<u32> {
    t.g.obj(id)
        .stack
        .as_deref()
        .and_then(|si| si.chosen.first())
        .and_then(|c| c.divided.first().cloned())
        .unwrap_or_default()
}

/// Whether anyone was asked to divide something since `from`.
fn divided_since(t: &TestGame, from: usize) -> bool {
    crate::r_s01_common::asked_since(t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::Divide { .. }))
}

// --- Kirol, Attentive First-Year: copies of triggered abilities -----------------------------

/// P0's Kirol and two Grizzly Bears to tap for its ability.
fn kirol(t: &mut TestGame) -> (ObjectId, [ObjectId; 2]) {
    supported("Kirol, Attentive First-Year");
    let k = t.battlefield(P0, "Kirol, Attentive First-Year");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    (k, [a, b])
}

/// Kirol's ability, tapping the two Bears, targets the topmost object on the stack.
fn kirol_copies_top(t: &mut TestGame, k: ObjectId, bears: [ObjectId; 2]) {
    let top = *t.g.stack.last().expect("nothing to copy");
    t.answer_choose(P0, &[obj(bears[0]), obj(bears[1])]);
    t.answer_targets(P0, &[obj(top)]);
    activate_containing(t, P0, k, "Copy target triggered ability").expect("activates");
}

#[test]
fn kirols_copy_makes_its_choices_and_pays_its_costs_separately() {
    cr!("707.10", "608.2d", "118.12", "707.10c");
    ruling!(
        "Kirol, Attentive First-Year",
        "Any choices made when the ability resolves won't have been made yet when it's copied by Kirol's ability. Any such choices will be made separately when the copy resolves. If a triggered ability asks you to pay a cost, you pay that cost for the copy separately."
    );
    supported("Frenzied Goblin");
    // Frenzied Goblin: "Whenever this creature attacks, you may pay {R}. If you do, target
    // creature can't block this turn." The copy (at the Giant) is paid for; the original
    // (at the Wall) isn't.
    let mut t = TestGame::new(2);
    let (k, bears) = kirol(&mut t);
    let goblin = t.battlefield(P0, "Frenzied Goblin");
    let wall = t.battlefield(P1, "Wall of Wood");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(wall)]);
    attack_with(&mut t, &[(goblin, Entity::Player(P1))]);
    kirol_copies_top(&mut t, k, bears);
    change_copy_targets(&mut t, P0, &[Some(obj(giant))]);
    t.resolve();
    assert_eq!(abilities_from(&t, goblin).len(), 2);
    t.lands(P0, "Mountain", 2);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    let pays = crate::r_s01_common::asked_since(&t, from)
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .count();
    assert_eq!(pays, 2, "asked once for each");
    assert!(!t.g.can_block_at_all(giant));
    assert!(t.g.can_block_at_all(wall));
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 1);
}

#[test]
fn kirols_copy_keeps_the_division_and_the_number_of_targets() {
    cr!("707.10c", "601.2d", "115.7f");
    ruling!(
        "Kirol, Attentive First-Year",
        "If the ability copied by Kirol's ability divides damage or distributes counters among a number of targets, the division and number of targets can't be changed. If you choose new targets, you must choose the same number of targets."
    );
    supported("Bogardan Hellkite");
    let mut t = TestGame::new(2);
    let (k, bears) = kirol(&mut t);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Player(P1), obj(wurm)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 2]));
    let hellkite = t.enter(P0, "Bogardan Hellkite");
    t.settle();
    kirol_copies_top(&mut t, k, bears);
    change_copy_targets(&mut t, P0, &[None, Some(obj(giant))]);
    let from = t.asked().len();
    t.resolve();
    let both = abilities_from(&t, hellkite);
    assert_eq!(both.len(), 2);
    let copy = both[1];
    assert_eq!(targets_of(&t, copy), vec![Entity::Player(P1), obj(giant)]);
    assert_eq!(division(&t, copy), vec![3, 2]);
    assert!(!divided_since(&t, from));
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.obj_now(giant).damage, 2);
    assert_eq!(t.obj_now(wurm).damage, 2);
}

#[test]
fn kirols_copy_of_a_modal_trigger_keeps_the_mode() {
    cr!("707.10", "700.2g");
    ruling!(
        "Kirol, Attentive First-Year",
        "If the ability copied by Kirol's ability is modal (that is, if it says, \"Choose one —\" or similar), the mode is copied and can't be changed."
    );
    supported("Charming Prince");
    let mut t = TestGame::new(2);
    let (k, bears) = kirol(&mut t);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    let prince = t.enter(P0, "Charming Prince");
    t.settle();
    kirol_copies_top(&mut t, k, bears);
    t.resolve();
    let both = abilities_from(&t, prince);
    assert_eq!(both.len(), 2);
    assert!(both.iter().all(|a| modes_on_stack(&t, *a) == vec![1]));
    t.resolve_all();
    assert_eq!(t.life(P0), 26);
}

// --- Cast triggers resolve before their spell -------------------------------------------------

/// P1 counters the spell with Cancel.
fn p1_counters(t: &mut TestGame, spell: ObjectId) {
    t.lands(P1, "Island", 3);
    let c = t.hand(P1, "Cancel");
    t.cast(P1, c).target(spell).go();
    t.resolve();
    assert!(!t.g.stack.contains(&spell));
}

#[test]
fn aziza_copies_the_spell_before_it_resolves_even_if_it_was_countered() {
    cr!("603.3", "707.10", "608.2h", "118.12");
    ruling!(
        "Aziza, Mage Tower Captain",
        "Aziza's ability and the copy it creates both resolve before the spell that caused the ability to trigger. They resolve even if the spell is countered before the copy is created."
    );
    supported("Aziza, Mage Tower Captain");
    for counter in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Aziza, Mage Tower Captain");
        let cs: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
        add_mana(&mut t, P0, ManaType::R, 1);
        let b = t.hand(P0, "Lightning Bolt");
        let bolt = t.cast(P0, b).target(P1).go();
        t.settle();
        assert_eq!(triggers_on_stack(&t, "copy that spell"), 1);
        if counter {
            p1_counters(&mut t, bolt);
        }
        t.answer_yes(P0, true);
        t.answer_choose(P0, &cs.iter().map(|c| obj(*c)).collect::<Vec<_>>());
        keep_copy_targets(&mut t, P0);
        t.resolve();
        assert!(cs.iter().all(|c| tapped(&t, *c)));
        // The copy is above the original (if it's still there).
        let copies = spell_copies(&t);
        assert_eq!(copies.len(), 1, "counter: {counter}: {}", t.dump_log());
        assert_eq!(*t.g.stack.last().unwrap(), copies[0]);
        t.resolve();
        assert_eq!(t.life(P1), 17, "counter: {counter}");
        t.resolve_all();
        assert_eq!(t.life(P1), if counter { 17 } else { 14 });
    }
}

#[test]
fn raff_draws_before_the_spell_resolves_even_if_it_was_countered() {
    cr!("603.3", "118.12");
    ruling!(
        "Raff, Weatherlight Stalwart",
        "Raff, Weatherlight Stalwart’s triggered ability resolves before the spell that caused it to trigger. The ability will resolve even if that spell is countered."
    );
    supported("Raff, Weatherlight Stalwart");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raff, Weatherlight Stalwart");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    give_mana_for(&mut t, P0, "Divination");
    let d = t.hand(P0, "Divination");
    let spell = t.cast(P0, d).go();
    t.settle();
    assert_eq!(*t.g.stack.first().unwrap(), spell);
    p1_counters(&mut t, spell);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.resolve_all();
    assert!(tapped(&t, a) && tapped(&t, b));
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn barracks_of_the_thousand_token_comes_before_the_spell_resolves() {
    cr!("603.3", "601.2i");
    ruling!(
        "Thousand Moons Smithy",
        "Barracks of the Thousand's triggered ability will resolve before the spell that caused it to trigger."
    );
    supported("Thousand Moons Smithy // Barracks of the Thousand");
    let mut t = TestGame::new(2);
    let barracks = back_face(
        &mut t,
        P0,
        "Thousand Moons Smithy // Barracks of the Thousand",
    );
    t.battlefield(P0, "Forest");
    assert!(tap_for_mana(&mut t, P0, barracks, "Add {W}"));
    let c = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, c).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.g.stack, vec![spell], "the spell is still on the stack");
    let gnomes = tokens(&t, P0);
    assert_eq!(gnomes.len(), 1);
    // Barracks (an artifact) and the Gnome: 2/2; with the Bears, 3/3.
    assert_eq!(t.pt(gnomes[0]), (2, 2));
    t.resolve_all();
    assert_eq!(t.pt(gnomes[0]), (3, 3));
}

// --- Sorcerer Class --------------------------------------------------------------------------

fn sorcerer_class_3(t: &mut TestGame) -> ObjectId {
    supported("Sorcerer Class");
    let c = t.battlefield(P0, "Sorcerer Class");
    t.g.objects[c.0 as usize].class_level = 3;
    t.g.recompute();
    c
}

/// A creature P0 controls enchanted with Dual Casting.
fn dual_caster(t: &mut TestGame) -> ObjectId {
    supported("Dual Casting");
    let c = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Dual Casting");
    assert!(t.g.attach(aura, obj(c)));
    c
}

/// Dual Casting's granted ability (with a new Mountain for {R}) targets `spell`.
fn dual_cast(t: &mut TestGame, c: ObjectId, spell: ObjectId) {
    t.battlefield(P0, "Mountain");
    t.answer_targets(P0, &[obj(spell)]);
    activate_containing(t, P0, c, "Copy target instant or sorcery").expect("activates");
}

#[test]
fn sorcerer_class_ignores_copies_but_counts_cast_copies_of_cards() {
    cr!("707.10", "707.12", "603.2");
    ruling!(
        "Sorcerer Class",
        "Copying a spell won't cause Sorcerer Class's level 3 ability to trigger and won't be counted toward the number of spells you've cast during the turn. However, if an effect copies a card and you cast the copy, that will count for both of those things."
    );
    let mut t = TestGame::new(2);
    sorcerer_class_3(&mut t);
    let c = dual_caster(&mut t);
    add_mana(&mut t, P0, ManaType::R, 1);
    let s = t.hand(P0, "Shock");
    let shock = t.cast(P0, s).target(P1).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "deals damage to each opponent"), 1);
    dual_cast(&mut t, c, shock);
    keep_copy_targets(&mut t, P0);
    t.resolve(); // Dual Casting's ability: the copy
    assert_eq!(spell_copies(&t).len(), 1);
    assert_eq!(triggers_on_stack(&t, "deals damage to each opponent"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 20 - 2 - 2 - 1);
    // Isochron Scepter copies the imprinted card, and P0 casts the copy: it triggers, and
    // it's the second spell P0 cast this turn.
    supported("Isochron Scepter");
    let s2 = t.hand(P0, "Shock");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(s2)]);
    let scepter = t.enter(P0, "Isochron Scepter");
    t.resolve_all();
    assert!(t.in_exile("Shock"), "{}", t.dump_log());
    t.lands(P0, "Wastes", 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, scepter, "copy the exiled card").expect("activates");
    t.resolve();
    t.settle();
    assert_eq!(
        triggers_on_stack(&t, "deals damage to each opponent"),
        1,
        "{}",
        t.dump_log()
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 15 - 2 - 2, "{}", t.dump_log());
}

#[test]
fn sorcerer_class_damage_from_a_countered_spell_uses_last_known_information() {
    cr!("608.2h", "702.15b");
    ruling!(
        "Sorcerer Class",
        "If the spell that causes the last ability to trigger is countered or is otherwise no longer on the stack as the ability resolves, it will still deal damage. Last known information will be used to do determine the effects of that damage. For example, if the spell has lifelink, you will still gain life when it deals damage, even if the spell was countered."
    );
    supported("Firesong and Sunspeaker");
    let mut t = TestGame::new(2);
    sorcerer_class_3(&mut t);
    // "Red instant and sorcery spells you control have lifelink."
    t.battlefield(P0, "Firesong and Sunspeaker");
    add_mana(&mut t, P0, ManaType::R, 1);
    let s = t.hand(P0, "Shock");
    let shock = t.cast(P0, s).target(P1).go();
    t.settle();
    p1_counters(&mut t, shock);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Shock"));
    assert_eq!(t.life(P1), 19, "{}", t.dump_log());
    assert_eq!(t.life(P0), 21, "lifelink, from last known information");
}

// --- Copies of instants and sorceries ----------------------------------------------------------

/// P0's Sigil Tracer and another Wizard, with {1}{U} for the ability.
fn sigil_tracer(t: &mut TestGame) -> ObjectId {
    supported("Sigil Tracer");
    let s = t.battlefield(P0, "Sigil Tracer");
    t.battlefield(P0, "Prodigal Sorcerer");
    s
}

fn sigil_copy(t: &mut TestGame, tracer: ObjectId, spell: ObjectId) {
    t.battlefield(P0, "Island");
    t.battlefield(P0, "Wastes");
    t.answer_targets(P0, &[obj(spell)]);
    activate_containing(t, P0, tracer, "Copy target instant or sorcery").expect("activates");
}

#[test]
fn sigil_tracer_copies_untargeted_spells_and_kicker_choices() {
    cr!("707.10", "601.2b", "702.33d", "702.76a", "118.9");
    ruling!(
        "Sigil Tracer",
        "Sigil Tracer’s ability can copy an instant or sorcery spell that isn’t targeted."
    );
    let mut t = TestGame::new(2);
    let tr = sigil_tracer(&mut t);
    give_mana_for(&mut t, P0, "Divination");
    let d = t.hand(P0, "Divination");
    let spell = t.cast(P0, d).go();
    sigil_copy(&mut t, tr, spell);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4);

    ruling!(
        "Sigil Tracer",
        "If the controller of the original spell chose to pay an alternative cost (such as prowl) or an additional cost, that choice is also copied."
    );
    supported("Burst Lightning");
    let mut t = TestGame::new(2);
    let tr = sigil_tracer(&mut t);
    add_mana(&mut t, P0, ManaType::R, 5);
    let b = t.hand(P0, "Burst Lightning");
    let spell = t.cast(P0, b).kicked(true).target(P1).go();
    sigil_copy(&mut t, tr, spell);
    keep_copy_targets(&mut t, P0);
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 12, "both 4 damage");

    // An alternative cost: Morsel Theft cast for its prowl cost {1}{B} after a Rogue dealt
    // combat damage to P1 ("Target player loses 3 life and you gain 3 life. If Morsel
    // Theft's prowl cost was paid, draw a card."): the copy draws a card too.
    supported("Morsel Theft");
    let mut t = TestGame::new(2);
    let tr = sigil_tracer(&mut t);
    let rogue = t.battlefield(P0, "Bane Alley Blackguard");
    attack_with(&mut t, &[(rogue, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 19);
    t.lands(P0, "Swamp", 2);
    let theft = t.hand(P0, "Morsel Theft");
    let hand = t.hand_size(P0);
    let spell = t
        .cast(P0, theft)
        .method(CastMethod::Keyword(KeywordKind::Prowl))
        .target(P1)
        .go();
    sigil_copy(&mut t, tr, spell);
    keep_copy_targets(&mut t, P0);
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (26, 13));
    assert_eq!(t.hand_size(P0), hand - 1 + 2, "{}", t.dump_log());
}

#[test]
fn copies_of_modal_spells_keep_the_modes() {
    cr!("707.10", "700.2g");
    ruling!(
        "Sigil Tracer",
        "If the spell is modal (that is, it says “choose one —” or “choose two —”), the choice of mode(s) can’t be changed."
    );
    ruling!(
        "Dual Casting",
        "If the spell being copied is modal (that is, it says “Choose one —” or the like), the copy will have the same mode(s). You can’t choose different ones."
    );
    supported("Izzet Charm");
    for use_tracer in [true, false] {
        let mut t = TestGame::new(2);
        let copier = if use_tracer {
            sigil_tracer(&mut t)
        } else {
            dual_caster(&mut t)
        };
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        add_mana(&mut t, P0, ManaType::U, 1);
        add_mana(&mut t, P0, ManaType::R, 1);
        let c = t.hand(P0, "Izzet Charm");
        // "Izzet Charm deals 2 damage to target creature."
        let spell = t.cast(P0, c).modes(&[1]).target(bears).go();
        if use_tracer {
            sigil_copy(&mut t, copier, spell);
        } else {
            dual_cast(&mut t, copier, spell);
        }
        let from = t.asked().len();
        change_copy_targets(&mut t, P0, &[Some(obj(giant))]);
        t.resolve();
        let copy = spell_copies(&t)[0];
        assert_eq!(modes_on_stack(&t, copy), vec![1]);
        assert!(!crate::r_s01_common::asked_since(&t, from)
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseModes { .. })));
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Grizzly Bears"));
        assert_eq!(t.obj_now(giant).damage, 2);
    }
}

#[test]
fn dual_casting_copy_has_the_same_x() {
    cr!("707.10", "107.3");
    ruling!(
        "Dual Casting",
        "If the spell being copied has an X whose value was determined as it was cast (like Earthquake does), the copy has the same value of X."
    );
    supported("Earthquake");
    let mut t = TestGame::new(2);
    let c = dual_caster(&mut t);
    add_mana(&mut t, P0, ManaType::R, 3);
    let e = t.hand(P0, "Earthquake");
    let spell = t.cast(P0, e).x(2).go();
    dual_cast(&mut t, c, spell);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(x_of(&t, copy), Some(2));
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (16, 16));
}

// --- Wort, the Raidmother: conspire copies ---------------------------------------------------

/// P0's Wort with two creatures `helper` to tap for conspire.
fn wort(t: &mut TestGame, helper: &str) -> [ObjectId; 2] {
    supported("Wort, the Raidmother");
    t.battlefield(P0, "Wort, the Raidmother");
    [t.battlefield(P0, helper), t.battlefield(P0, helper)]
}

fn pay_conspire(t: &mut TestGame, bears: [ObjectId; 2]) {
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_choose(P0, &[obj(bears[0]), obj(bears[1])]);
}

#[test]
fn wort_conspire_copy_keeps_the_division() {
    cr!("702.78a", "707.10", "601.2d");
    ruling!(
        "Wort, the Raidmother",
        "If the spell has damage divided as it was cast (like Fiery Justice does), the division can't be changed (although the targets receiving that damage still can)."
    );
    supported("Fiery Justice");
    let mut t = TestGame::new(2);
    let bears = wort(&mut t, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    give_mana_for(&mut t, P0, "Fiery Justice");
    let f = t.hand(P0, "Fiery Justice");
    pay_conspire(&mut t, bears);
    t.answer_targets(P0, &[obj(wurm), Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![3, 2]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let spell = t.cast(P0, f).go();
    assert!(bears.iter().all(|b| tapped(&t, *b)));
    assert_eq!(division(&t, spell), vec![3, 2]);
    // The copy's 3 go to the Giant instead.
    change_copy_targets(&mut t, P0, &[Some(obj(giant)), None, None]);
    let from = t.asked().len();
    t.resolve(); // conspire
    let copy = spell_copies(&t)[0];
    assert_eq!(division(&t, copy), vec![3, 2]);
    assert!(!divided_since(&t, from));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"), "{}", t.dump_log());
    assert_eq!(t.obj_now(wurm).damage, 3);
    assert_eq!(t.life(P1), 20 - 2 - 2 + 5 + 5);
}

#[test]
fn wort_conspire_copy_has_the_same_x() {
    cr!("702.78a", "707.10", "107.3");
    ruling!(
        "Wort, the Raidmother",
        "If the spell that's copied has an X whose value was determined as it was cast (like Bonfire of the Damned does), the copy will have the same value of X."
    );
    supported("Bonfire of the Damned");
    let mut t = TestGame::new(2);
    // Red creatures for a red spell.
    let bears = wort(&mut t, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    add_mana(&mut t, P0, ManaType::R, 5);
    let b = t.hand(P0, "Bonfire of the Damned");
    pay_conspire(&mut t, bears);
    let spell = t.cast(P0, b).x(2).target(P1).go();
    keep_copy_targets(&mut t, P0);
    t.resolve();
    let copy = spell_copies(&t)[0];
    assert_eq!(x_of(&t, copy), Some(2));
    assert_eq!(x_of(&t, spell), Some(2));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert!(!t.on_battlefield(wurm));
}

// --- Swallow Whole ---------------------------------------------------------------------------

#[test]
fn swallow_whole_copy_counters_the_creature_tapped_for_the_original() {
    cr!("707.10", "601.2b", "608.2h");
    ruling!(
        "Swallow Whole",
        "If an effect copies Swallow Whole, the copy will put a +1/+1 counter on the creature tapped to pay the additional cost of the original spell."
    );
    supported("Swallow Whole");
    let mut t = TestGame::new(2);
    let c = dual_caster(&mut t);
    let eater = t.battlefield(P0, "Hill Giant");
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(x);
    t.g.tap(y);
    t.battlefield(P0, "Plains");
    let s = t.hand(P0, "Swallow Whole");
    t.answer_choose(P0, &[obj(eater)]);
    let spell = t.cast(P0, s).target(x).go();
    assert!(tapped(&t, eater) && !tapped(&t, c));
    dual_cast(&mut t, c, spell);
    change_copy_targets(&mut t, P0, &[Some(obj(y))]);
    t.resolve_all();
    assert_eq!(t.zone(x), Zone::Exile);
    assert_eq!(t.zone(y), Zone::Exile);
    assert_eq!(t.counters(eater, counters::PLUS1), 2, "{}", t.dump_log());
    assert_eq!(t.counters(c, counters::PLUS1), 0);
}
