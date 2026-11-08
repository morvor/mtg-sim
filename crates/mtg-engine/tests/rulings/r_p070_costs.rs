//! Rulings batch P070 — costs of cards cast from exile: no alternative cost can be combined
//! with "without paying its mana cost" or another alternative cost (CR 118.9a, 601.2b),
//! optional additional costs may be paid and mandatory ones must be (CR 601.2b, 601.2f),
//! X is 0 when casting without paying the mana cost (CR 107.3b); the total cost is the
//! mana cost or alternative cost, plus increases, less reductions (CR 601.2f), while the
//! mana value comes from the mana cost only (CR 202.3); tapping creatures for a cost that
//! isn't {T} (CR 302.6); the six types of mana and snow costs (CR 106.1b, 107.4h).

use crate::r_p070_common::*;
use mtg_engine::card::card;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The mana value of the object `id` (followed across zone changes) as it is now.
fn mv(t: &TestGame, id: ObjectId) -> u32 {
    let id = t.g.current(id);
    mtg_engine::mana_value::mana_value_with(&t.g, id, &t.obj_now(id).chars)
}

/// The number of `p`'s permanents that are tapped lands.
fn tapped_lands(t: &TestGame, p: PlayerId) -> usize {
    crate::r_s01_common::tapped_lands(t, p)
}

// ---------------------------------------------------------------------------
// Eye of the Storm
// ---------------------------------------------------------------------------

#[test]
fn eye_of_the_storm_copies_get_choices_and_additional_costs_x_0_no_alternative_costs() {
    cr!("707.12", "107.3b", "118.9a", "601.2b");
    ruling!(
        "Eye of the Storm",
        "The player chooses modes, pays additional costs, chooses targets, and so on for the copies when casting them. Any X in the mana cost of a copy cast this way will be 0. Alternative costs can't be paid."
    );
    supported("Eye of the Storm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eye of the Storm");
    // Blaze with X 3: exiled; its copy is cast with X 0 (no damage).
    let blaze = t.hand(P0, "Blaze");
    mana(&mut t, P0, R, 4);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, blaze).x(3).target(P1).go();
    t.resolve_all();
    assert!(exiled(&t, blaze), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20, "the copy's X is 0: {}", t.dump_log());
    // Burst Lightning (not kicked): exiled; copies of Blaze and of Burst Lightning are
    // cast, the latter kicked (its additional cost paid as it's cast), each at a target
    // chosen then.
    let burst = t.hand(P0, "Burst Lightning");
    mana(&mut t, P0, R, 1);
    mana(&mut t, P0, C, 4);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, burst).target(P1).go();
    t.resolve_all();
    assert!(exiled(&t, burst), "{}", t.dump_log());
    assert_eq!(t.life(P1), 16, "kicked copy: {}", t.dump_log());
    // Fireblast (alternative cost: sacrifice two Mountains): its copy is cast without
    // paying its mana cost; no alternative cost is offered.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eye of the Storm");
    let mtns = t.lands(P0, "Mountain", 2);
    let fb = t.hand(P0, "Fireblast");
    mana(&mut t, P0, R, 2);
    mana(&mut t, P0, C, 4);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, fb).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
    assert!(!crate::r_p068_common::asked_method(&t, from));
    assert!(mtns.iter().all(|m| t.on_battlefield(*m)));
}

#[test]
fn eye_of_the_storm_copy_of_a_replicate_spell_can_be_replicated() {
    cr!("702.56a", "707.12", "601.2b");
    ruling!(
        "Eye of the Storm",
        "When a spell with Replicate is copied by Eye of the Storm, you have the opportunity to pay additional costs, so you may Replicate as much as you like (assuming you have the mana available)."
    );
    supported("Eye of the Storm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eye of the Storm");
    let tot = t.hand(P0, "Train of Thought");
    mana(&mut t, P0, U, 1);
    mana(&mut t, P0, C, 1);
    // Cast without replicating; its copy is replicated twice ({1}{U} each).
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(0));
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Number(2));
    let hand = t.hand_size(P0);
    t.cast(P0, tot).go();
    mana(&mut t, P0, U, 2);
    mana(&mut t, P0, C, 2);
    t.resolve_all();
    assert!(exiled(&t, tot), "{}", t.dump_log());
    // The copy and its two replicate copies: three cards (Train of Thought itself left
    // the hand).
    assert_eq!(t.hand_size(P0), hand - 1 + 3, "{}", t.dump_log());
}

// ---------------------------------------------------------------------------
// Copies of exiled cards cast without paying their mana costs
// ---------------------------------------------------------------------------

/// Elite Arcanist enters exiling `name` from P0's hand; returns it ready to activate and
/// the exiled card.
fn elite_arcanist(t: &mut TestGame, name: &str) -> (ObjectId, ObjectId) {
    let c = t.hand(P0, name);
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[o(c)]);
    let a = t.enter(P0, "Elite Arcanist");
    t.resolve_all();
    assert!(exiled(t, c), "{}", t.dump_log());
    t.g.obj_mut(a).summoning_sick = false;
    (a, c)
}

#[test]
fn elite_arcanist_copy_pays_additional_costs_but_no_alternative_cost() {
    cr!("118.9a", "601.2b", "601.2f", "707.12");
    ruling!(
        "Elite Arcanist",
        "You can't pay any alternative costs such as overload costs. You can pay additional costs such as kicker costs. If the spell has any mandatory additional costs, you must pay those."
    );
    supported("Elite Arcanist");
    // Kicker: paid ({4}) for 4 damage.
    let mut t = TestGame::new(2);
    let (a, _) = elite_arcanist(&mut t, "Burst Lightning");
    mana(&mut t, P0, C, 5);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
    // Fling: its mandatory additional cost (sacrificing a creature) is paid.
    let mut t = TestGame::new(2);
    let (a, _) = elite_arcanist(&mut t, "Fling");
    let giant = t.battlefield(P0, "Hill Giant");
    mana(&mut t, P0, C, 2);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(giant)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 17);
    // Cyclonic Rift: not overloaded (only the target returns).
    let mut t = TestGame::new(2);
    let (a, _) = elite_arcanist(&mut t, "Cyclonic Rift");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Hill Giant");
    mana(&mut t, P0, C, 2);
    mana(&mut t, P0, U, 7);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[o(b1)]);
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(b1), "{}", t.dump_log());
    assert!(t.on_battlefield(b2), "not overloaded");
    assert!(!crate::r_p068_common::asked_method(&t, from));
}

#[test]
fn elite_arcanist_x_is_the_exiled_cards_mana_value() {
    cr!("107.3c", "602.2b");
    ruling!(
        "Elite Arcanist",
        "You don't choose the value of {X} in the activation cost of Elite Arcanist's ability. It's defined by the mana value of the exiled instant card."
    );
    supported("Elite Arcanist");
    for (name, n) in [("Lightning Bolt", 1), ("Cancel", 3)] {
        let mut t = TestGame::new(2);
        let (a, _) = elite_arcanist(&mut t, name);
        t.lands(P0, "Wastes", 5);
        let from = t.asked().len();
        // Even answering another value, X is the mana value.
        t.answer(P0, DecisionKind::X, Answer::Number(0));
        t.answer_yes(P0, false);
        t.activate(P0, a, 0, &[]).unwrap();
        assert_eq!(tapped_lands(&t, P0), n, "{name}: {}", t.dump_log());
        // The only value offered, if any, is the mana value.
        for (_, d) in &t.asked()[from..] {
            if let mtg_engine::decision::Decision::ChooseX { min, max, .. } = d {
                assert_eq!((*min, *max), (n as i64, n as i64), "{name}");
            }
        }
        t.resolve_all();
    }
}

/// Puts `name` (a card P0 drafted, from outside the game) into exile as exiled with an
/// Arcane Savant, and has Arcane Savant enter.
fn arcane_savant(t: &mut TestGame, name: &str) {
    let side = t.g.add_to_sideboard(P0, vec![card(name)]);
    let x =
        t.g.move_object(
            side[0],
            Zone::Exile,
            mtg_engine::events::MoveCause::Effect,
            Some(P0),
        )
        .unwrap();
    t.g.named_exiles.push((P0, "Arcane Savant".into(), x));
    t.enter(P0, "Arcane Savant");
}

#[test]
fn arcane_savant_copy_pays_additional_costs_but_no_alternative_cost() {
    cr!("118.9a", "601.2b", "601.2f", "707.12");
    ruling!(
        "Arcane Savant",
        "You can't pay any alternative costs the card has. You can pay additional costs, and if the spell has any mandatory additional costs, you must pay those."
    );
    supported("Arcane Savant");
    // Kicker paid.
    let mut t = TestGame::new(2);
    mana(&mut t, P0, C, 4);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    arcane_savant(&mut t, "Burst Lightning");
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
    // Bone Splinters: a creature is sacrificed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let target = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(bears)]);
    t.answer_targets(P0, &[o(target)]);
    arcane_savant(&mut t, "Bone Splinters");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Hill Giant"));
    // Fireblast: no alternative cost offered; the Mountains stay.
    let mut t = TestGame::new(2);
    let mtns = t.lands(P0, "Mountain", 2);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    arcane_savant(&mut t, "Fireblast");
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "{}", t.dump_log());
    assert!(!crate::r_p068_common::asked_method(&t, from));
    assert!(mtns.iter().all(|m| t.on_battlefield(*m)));
}

// ---------------------------------------------------------------------------
// Valgavoth: an alternative cost of life
// ---------------------------------------------------------------------------

/// Begins casting the card `c` (followed across zone changes) exiled with Valgavoth, with
/// the only method its permission offers (paying life).
fn valgavoth_cast(t: &mut TestGame, c: ObjectId) -> CastBuilder<'_> {
    let methods = crate::r_s07_common::cast_methods(t, P0, c);
    assert_eq!(methods.len(), 1, "{methods:?}");
    let c = t.g.current(c);
    t.cast(P0, c).method(methods[0].clone())
}

#[test]
fn valgavoth_pays_life_for_the_mana_cost_and_additional_costs_as_usual() {
    cr!("118.9", "601.2b", "601.2f", "702.33d");
    ruling!(
        "Valgavoth, Terror Eater",
        "The spells you cast using the permission granted by Valgavoth's last ability have an alternative cost that's paying life equal to their mana value. You can't choose to pay their mana cost and may not pay any other alternative costs. You may still pay for additional costs, such as kicker costs, and you may pay mana for those as normal. If the spell has mandatory additional costs, you must pay those."
    );
    supported("Valgavoth, Terror Eater");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Valgavoth, Terror Eater");
    let to_gy = |t: &mut TestGame, name: &str| {
        let c = t.hand(P1, name);
        move_to(t, c, Zone::Graveyard(P1))
    };
    let burst = to_gy(&mut t, "Burst Lightning");
    let splinters = to_gy(&mut t, "Bone Splinters");
    let fb = to_gy(&mut t, "Fireblast");
    assert!(exiled(&t, burst) && exiled(&t, splinters) && exiled(&t, fb));
    // Burst Lightning kicked: 1 life and {4}.
    let lands = t.lands(P0, "Wastes", 4);
    let life = t.life(P0);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    let spell = valgavoth_cast(&mut t, burst).target(P1).go();
    assert_eq!(t.life(P0), life - 1, "{}", t.dump_log());
    assert!(
        lands.iter().all(|l| t.obj_now(*l).tapped),
        "kicker paid with mana"
    );
    let _ = spell;
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Bone Splinters: 1 life and a creature sacrificed.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P0, &[o(bears)]);
    valgavoth_cast(&mut t, splinters).target(giant).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"), "{}", t.dump_log());
    assert_eq!(t.life(P0), life - 2);
    // Fireblast: only for 6 life, never its own alternative cost; with no mana at all the
    // Mountains stay.
    let mtns = t.lands(P0, "Mountain", 2);
    let from = t.asked().len();
    valgavoth_cast(&mut t, fb).target(P1).go();
    assert_eq!(t.life(P0), life - 8, "{}", t.dump_log());
    assert!(mtns
        .iter()
        .all(|m| t.on_battlefield(*m) && !t.obj_now(*m).tapped));
    assert!(!crate::r_p068_common::asked_method(&t, from));
}

// ---------------------------------------------------------------------------
// Total cost and mana value
// ---------------------------------------------------------------------------

#[test]
fn urianger_reduction_applies_after_kicker_and_mana_value_is_unchanged() {
    cr!("601.2f", "202.3", "702.33d");
    ruling!(
        "Urianger Augurelt",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying (such as a flashback cost), add any cost increases (such as kicker costs), then apply any cost reductions (such as that of Urianger Augurelt's last ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Urianger Augurelt");
    let mut t = TestGame::new(2);
    let u = t.battlefield(P0, "Urianger Augurelt");
    let burst = t.library_top(P0, "Burst Lightning");
    t.answer_yes(P0, true);
    t.activate(P0, u, 0, &[]).unwrap();
    t.resolve_all();
    assert!(exiled(&t, burst), "{}", t.dump_log());
    t.g.untap(u);
    t.activate(P0, u, 1, &[]).unwrap();
    t.resolve_all();
    // Kicked: {R} + {4} - {2} = {2}{R}.
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    let spell = t.cast(P0, t.g.current(burst)).target(P1).go();
    assert_eq!(tapped_lands(&t, P0), 3, "{}", t.dump_log());
    assert_eq!(mv(&t, spell), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn dirgur_focusmage_reduction_applies_after_kicker_and_mana_value_is_unchanged() {
    cr!("601.2f", "202.3", "702.33d");
    ruling!(
        "Dirgur Focusmage // Braingeyser",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Dirgur Focusmage's first ability). The mana value of the spell is determined by only its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Dirgur Focusmage // Braingeyser");
    // Kicked Burst Lightning: {R} + {4} - {1}: four mana.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dirgur Focusmage // Braingeyser");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    let burst = t.hand(P0, "Burst Lightning");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    let spell = t.cast(P0, burst).target(P1).go();
    assert_eq!(tapped_lands(&t, P0), 4, "{}", t.dump_log());
    assert_eq!(mv(&t, spell), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Overrun ({2}{G}{G}{G}) for four mana is still mana value 5: Dirgur becomes prepared.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Dirgur Focusmage // Braingeyser");
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Wastes", 1);
    let ov = t.hand(P0, "Overrun");
    let spell = t.cast(P0, ov).go();
    assert_eq!(tapped_lands(&t, P0), 4);
    assert_eq!(mv(&t, spell), 5);
    t.resolve_all();
    assert!(t.obj_now(d).prepared.is_some(), "{}", t.dump_log());
}

#[test]
fn uvilda_reduction_applies_after_kicker_and_mana_value_is_unchanged() {
    cr!("601.2f", "202.3", "702.33d");
    ruling!(
        "Uvilda, Dean of Perfection // Nassari, Dean of Expression",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you’re paying, add any cost increases, then apply any cost reductions (such as that of Uvilda, Dean of Perfection). The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast the spell was."
    );
    supported("Uvilda, Dean of Perfection // Nassari, Dean of Expression");
    let mut t = TestGame::new(2);
    let uvilda = t.battlefield(
        P0,
        "Uvilda, Dean of Perfection // Nassari, Dean of Expression",
    );
    let burst = t.hand(P0, "Burst Lightning");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(burst)]);
    t.activate(P0, uvilda, 0, &[]).unwrap();
    t.resolve_all();
    let ex = t.g.current(burst);
    assert_eq!(t.counters(ex, "refine"), 3, "{}", t.dump_log());
    // Kicked: {R} + {4} - {4} = {R}: one Mountain.
    let mtn = t.lands(P0, "Mountain", 1)[0];
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, mtg_engine::decision::Decision::ChooseTargets { .. }),
        |g| {
            g.stack
                .iter()
                .map(|s| mtg_engine::mana_value::mana_value_with(g, *s, &g.obj(*s).chars))
                .collect::<Vec<_>>()
        },
    );
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.remove_counters(Entity::Object(ex), "refine", 3);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 16, "kicked: {}", t.dump_log());
    assert!(t.obj_now(mtn).tapped);
    // As its target was chosen, the spell (on the stack) had mana value 1.
    let seen = seen.lock().unwrap();
    assert!(seen.iter().any(|v| v.contains(&1)), "{seen:?}");
}

// ---------------------------------------------------------------------------
// Tapping creatures for costs
// ---------------------------------------------------------------------------

#[test]
fn harmonized_trio_taps_creatures_that_just_arrived() {
    cr!("302.6", "602.2b");
    ruling!(
        "Harmonized Trio // Brainstorm",
        "You can tap any untapped creatures you control to pay the cost of Harmonized Trio's activated ability, even ones you haven't controlled continuously since the beginning of your most recent turn."
    );
    supported("Harmonized Trio // Brainstorm");
    let mut t = TestGame::new(2);
    let trio = t.battlefield(P0, "Harmonized Trio // Brainstorm");
    let a = t.battlefield_sick(P0, "Grizzly Bears");
    let b = t.battlefield_sick(P0, "Llanowar Elves");
    t.answer_choose(P0, &[o(a), o(b)]);
    t.activate(P0, trio, 0, &[])
        .expect("summoning-sick creatures pay the cost");
    assert!(t.obj_now(a).tapped && t.obj_now(b).tapped);
    t.resolve_all();
    assert!(t.obj_now(trio).prepared.is_some(), "{}", t.dump_log());
}

#[test]
fn caparocti_sunborn_taps_creatures_that_just_arrived() {
    cr!("302.6", "701.57a");
    ruling!(
        "Caparocti Sunborn",
        "You can tap any two untapped artifacts and/or creatures you control, including creatures you haven't controlled continuously since the beginning of your most recent turn, for Caparocti Sunborn's ability."
    );
    supported("Caparocti Sunborn");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Caparocti Sunborn");
    let a = t.battlefield_sick(P0, "Grizzly Bears");
    let orni = t.battlefield_sick(P0, "Ornithopter");
    let shock = t.library_top(P0, "Shock");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(a), o(orni)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(c, Entity::Player(P1))]);
    t.resolve_all();
    assert!(
        t.obj_now(a).tapped && t.obj_now(orni).tapped,
        "{}",
        t.dump_log()
    );
    assert!(t.zone(shock) != Zone::Library(P0), "discovered");
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------
// The Ruinous Powers: mana value on the stack, with X
// ---------------------------------------------------------------------------

#[test]
fn the_ruinous_powers_uses_the_spells_mana_value_with_x_even_if_it_left_the_stack() {
    cr!("202.3e", "608.2h", "113.7a");
    ruling!(
        "The Ruinous Powers",
        "Use the mana value of the spell as it exists on the stack (or last existed on the stack, if it's been removed) to determine how much life is lost. If there is an {X} in that spell's cost, include the value chosen for X to determine its mana value."
    );
    supported("The Ruinous Powers");
    for countered in [false, true] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "The Ruinous Powers");
        let blaze = t.library_top(P1, "Blaze");
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert!(exiled(&t, blaze), "{}", t.dump_log());
        t.set_step(P0, Step::PrecombatMain);
        // Blaze with X 3 at P0 (mana of any color): mana value 4.
        mana(&mut t, P0, B, 4);
        let spell = t.cast(P0, t.g.current(blaze)).x(3).target(P0).go();
        t.settle();
        assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
        if countered {
            let cs = t.hand(P0, "Counterspell");
            mana(&mut t, P0, U, 2);
            t.cast(P0, cs).target(spell).go();
            t.resolve();
            assert!(t.in_graveyard(P1, "Blaze"), "{}", t.dump_log());
        }
        t.resolve();
        assert_eq!(t.life(P1), 16, "countered {countered}: {}", t.dump_log());
    }
}

// ---------------------------------------------------------------------------
// Mana types and snow
// ---------------------------------------------------------------------------

#[test]
fn azula_spends_colorless_mana_one_of_the_six_types() {
    cr!("106.1b", "609.4b");
    ruling!(
        "Azula, Cunning Usurper",
        "The six types of mana are white, blue, black, red, green, and colorless."
    );
    supported("Azula, Cunning Usurper");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let div = t.graveyard(P1, "Divination");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P1, &[o(bears)]);
    t.answer_choose(P1, &[o(div)]);
    t.enter(P0, "Azula, Cunning Usurper");
    t.resolve_all();
    assert!(exiled(&t, bears) && exiled(&t, div), "{}", t.dump_log());
    // Colorless mana is a type: {C}{C} casts the Bears ({1}{G}).
    mana(&mut t, P0, C, 2);
    t.cast(P0, t.g.current(bears)).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(bears)), "{}", t.dump_log());
}

/// Asserts that P0, with a Mountain, can't cast the exiled Icehide Golem ({S}), and can
/// with a Snow-Covered Mountain.
fn snow_needs_a_snow_source(t: &mut TestGame, golem: ObjectId) {
    assert!(exiled(t, golem), "{}", t.dump_log());
    t.g.stack.clear();
    t.set_step(P0, Step::PostcombatMain);
    t.lands(P0, "Mountain", 1);
    assert!(
        t.cast(P0, t.g.current(golem)).try_go().is_err(),
        "nonsnow mana"
    );
    t.lands(P0, "Snow-Covered Mountain", 1);
    t.cast(P0, t.g.current(golem)).go();
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(golem)), "{}", t.dump_log());
}

#[test]
fn mana_of_any_type_doesnt_pay_snow_costs() {
    cr!("107.4h", "106.1b", "609.4b");
    ruling!(
        "Blightwing Bandit",
        "The six types of mana are white, blue, black, red, green, and colorless. Snow mana is not a type of mana. Blightwing Bandit won't let you pay a snow cost using mana produced by a nonsnow source."
    );
    ruling!(
        "Court of Locthwain",
        "The six types of mana are white, blue, black, red, green, and colorless. Snow mana is not a type of mana. Court of Locthwain won't let you pay a snow cost using mana produced by a nonsnow source."
    );
    ruling!(
        "Gonti, Lord of Luxury",
        "The six types of mana are white, blue, black, red, green, and colorless. Snow mana is not a type of mana. Gonti, Lord of Luxury won't let you pay a snow cost using mana produced by a nonsnow source."
    );
    ruling!(
        "Ian Malcolm, Chaotician",
        "The six types of mana are white, blue, black, red, green, and colorless. Snow mana is not a type of mana. Ian Malcolm, Chaotician won't let you pay a snow cost using mana produced by a nonsnow source."
    );
    for name in [
        "Blightwing Bandit",
        "Court of Locthwain",
        "Gonti, Lord of Luxury",
        "Ian Malcolm, Chaotician",
    ] {
        supported(name);
    }
    // Blightwing Bandit: P0's first spell during P1's turn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blightwing Bandit");
    let golem = t.library_top(P1, "Icehide Golem");
    t.set_step(P1, Step::PrecombatMain);
    let opt = t.hand(P0, "Opt");
    mana(&mut t, P0, U, 1);
    t.g.turn.priority = Some(P0);
    t.cast(P0, opt).go();
    t.resolve_all();
    snow_needs_a_snow_source(&mut t, golem);
    // Court of Locthwain (P0 isn't the monarch): P0's upkeep.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Court of Locthwain");
    let golem = t.library_top(P1, "Icehide Golem");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    snow_needs_a_snow_source(&mut t, golem);
    // Gonti, Lord of Luxury.
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P1, &["Icehide Golem", "Forest", "Forest", "Forest"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[o(lib[0])]);
    t.enter(P0, "Gonti, Lord of Luxury");
    t.resolve_all();
    snow_needs_a_snow_source(&mut t, lib[0]);
    // Ian Malcolm: P1 draws its second card this turn and exiles Icehide Golem.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ian Malcolm, Chaotician");
    t.g.draw_cards(P1, 1);
    let golem = t.library_top(P1, "Icehide Golem");
    t.library_top(P1, "Forest");
    t.g.draw_cards(P1, 1);
    t.settle();
    t.resolve_all();
    snow_needs_a_snow_source(&mut t, golem);
}
