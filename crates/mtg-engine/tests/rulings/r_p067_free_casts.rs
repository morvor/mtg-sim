//! Rulings batch P067 — casting exiled cards "without paying their mana costs": X is 0
//! (CR 107.3b), its mana value in exile counts X as 0 (CR 202.3e), no alternative costs
//! but additional costs (CR 118.9a, 601.2b), and copies of cards (CR 707.12).

use crate::r_p067_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// Queues `p`'s answer of 3 for any X it's asked to choose.
fn answer_x3(t: &mut TestGame, p: PlayerId) {
    t.answer(p, DecisionKind::X, Answer::Number(3));
}

/// Whether `p` was asked to choose a casting method (an alternative cost) since `from`.
fn asked_method(t: &TestGame, from: usize) -> bool {
    t.asked()[from..].iter().any(|(_, d)| match d {
        Decision::ChooseCastingMethod { .. } => true,
        Decision::ChooseOption { prompt, .. } => prompt.contains("how to cast"),
        _ => false,
    })
}

// ---------------------------------------------------------------------------
// Alternative and additional costs
// ---------------------------------------------------------------------------

/// P1 casts Lightning Bolt at P0; P0 counters it with Tibalt's Trickery; P1's library has
/// `card` under three Forests. Returns the found card.
fn tibalt(t: &mut TestGame, card: &str) -> ObjectId {
    let lib = stack_library(t, P1, &["Forest", "Forest", "Forest", card, "Hill Giant"]);
    let bolt = t.hand(P1, "Lightning Bolt");
    mana(t, P1, ManaType::R, 1);
    t.g.turn.priority = Some(P1);
    let spell = t.cast(P1, bolt).target(P0).go();
    let tt = t.hand(P0, "Tibalt's Trickery");
    mana(t, P0, ManaType::R, 2);
    t.g.turn.priority = Some(P0);
    t.cast(P0, tt).target(spell).go();
    t.resolve();
    lib[3]
}

/// P0 casts Transforming Flourish (not copying it) at P1's Grizzly Bears; P1's library
/// has `card` under a Forest.
fn flourish(t: &mut TestGame, card: &str) -> ObjectId {
    let lib = stack_library(t, P1, &["Forest", card]);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let tf = t.hand(P0, "Transforming Flourish");
    mana(t, P0, ManaType::R, 3);
    t.answer_yes(P0, false);
    t.cast(P0, tf).target(bears).go();
    t.resolve_all_but_last();
    lib[1]
}

trait ResolveTop {
    fn resolve_all_but_last(&mut self);
}

impl ResolveTop for TestGame {
    /// Resolves the stack down to the original spell (the demonstrate trigger, then it).
    fn resolve_all_but_last(&mut self) {
        self.settle();
        while !self.g.stack.is_empty() {
            let top = *self.g.stack.last().unwrap();
            let is_spell = self.g.obj(top).chars.name == "Transforming Flourish";
            self.resolve();
            if is_spell {
                break;
            }
        }
    }
}

#[test]
fn free_casts_pay_kicker_and_mandatory_additional_costs_but_no_alternative_costs() {
    cr!("118.9", "118.9a", "601.2b", "601.2f");
    ruling!(
        "Tibalt's Trickery",
        "If a player casts a spell \"without paying its mana cost,\" they can't choose to cast it for any alternative costs. They can, however, pay additional costs, such as kicker costs. If the card has any mandatory additional costs, those must be paid to cast the spell."
    );
    ruling!(
        "Transforming Flourish",
        "If the destroyed permanent’s controller chooses to cast the exiled card without paying its mana cost, they can’t choose to cast it for any alternative costs. They can, however, pay additional costs, such as kicker costs. If the spell has any mandatory additional costs, those must be paid to cast the spell."
    );
    supported("Tibalt's Trickery");
    supported("Transforming Flourish");
    for which in ["tibalt", "flourish"] {
        let find = |t: &mut TestGame, card: &str| {
            if which == "tibalt" {
                tibalt(t, card)
            } else {
                flourish(t, card)
            }
        };
        // Burst Lightning's kicker {4} can be paid.
        let mut t = TestGame::new(2);
        t.lands(P1, "Mountain", 4);
        t.answer_yes(P1, true);
        t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(true));
        t.answer_targets(P1, &[Entity::Player(P0)]);
        let burst = find(&mut t, "Burst Lightning");
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Burst Lightning"), "{which}: {}", t.dump_log());
        assert_eq!(t.life(P0), 16, "{which}");
        assert_eq!(crate::r_s01_common::tapped_lands(&t, P1), 4, "{which}");
        let _ = burst;
        // Village Rites' sacrifice must be paid.
        let mut t = TestGame::new(2);
        let elf = t.battlefield(P1, "Llanowar Elves");
        t.answer_yes(P1, true);
        t.answer_choose(P1, &[o(elf)]);
        let hand = t.hand_size(P1);
        find(&mut t, "Village Rites");
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Llanowar Elves"), "{which}: {}", t.dump_log());
        assert_eq!(t.hand_size(P1), hand + 2, "{which}");
        // Invigorate: its alternative cost isn't offered.
        let mut t = TestGame::new(2);
        let elf = t.battlefield(P1, "Llanowar Elves");
        t.answer_yes(P1, true);
        t.answer_targets(P1, &[o(elf)]);
        let from = t.asked().len();
        find(&mut t, "Invigorate");
        t.resolve_all();
        assert!(!asked_method(&t, from), "{which}");
        assert_eq!(t.pt(elf), (5, 5), "{which}: {}", t.dump_log());
        assert_eq!(t.life(P0), if which == "tibalt" { 20 } else { 20 }, "{which}");
    }
}

// ---------------------------------------------------------------------------
// X is 0
// ---------------------------------------------------------------------------

#[test]
fn gix_spells_cast_this_way_have_x_0() {
    cr!("107.3b", "118.9");
    ruling!(
        "Gix, Yawgmoth Praetor",
        "If a spell you cast this way has an {X} in its mana cost, you must choose 0 for X."
    );
    supported("Gix, Yawgmoth Praetor");
    let mut t = TestGame::new(2);
    let gix = t.battlefield(P0, "Gix, Yawgmoth Praetor");
    let blaze = t.library_top(P1, "Blaze");
    let discard = t.hand(P0, "Hill Giant");
    t.lands(P0, "Swamp", 7);
    // X of the activation is 1 (one card discarded); Blaze is cast during the resolution,
    // with X answered 3.
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_choose(P0, &[o(discard)]);
    t.activate(P0, gix, 0, &[Entity::Player(P1)]).unwrap();
    t.answer_choose_moved(P0, &[o(blaze)]);
    t.answer_yes(P0, true);
    answer_x3(&mut t, P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(!exiled(&t, blaze), "{}", t.dump_log());
    assert!(t.in_graveyard(P1, "Blaze"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn improvisation_capstone_counts_x_as_0_and_casts_with_x_0() {
    cr!("107.3b", "202.3e");
    ruling!(
        "Improvisation Capstone",
        "If an exiled card has {X} in its mana cost, X is 0 for the purpose of determining its mana value and you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Improvisation Capstone");
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P0, &["Blaze", "Blaze", "Blaze", "Shock", "Hill Giant"]);
    let ic = t.hand(P0, "Improvisation Capstone");
    mana(&mut t, P0, ManaType::R, 7);
    t.answer_choose_moved(P0, &[o(lib[0])]);
    answer_x3(&mut t, P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, ic).go();
    t.resolve_all();
    // Mana values 1 + 1 + 1 + 1: four cards; Hill Giant stays in the library.
    assert_eq!(t.zone(lib[4]), Zone::Library(P0), "{}", t.dump_log());
    assert!(exiled(&t, lib[1]) && exiled(&t, lib[3]));
    assert!(t.in_graveyard(P0, "Blaze"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);
}

#[test]
fn exiled_x_cards_have_x_0_in_their_mana_value() {
    cr!("202.3e", "107.3b");
    ruling!(
        "Kotis, the Fangkeeper",
        "If an exiled card has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Maralen, Fae Ascendant",
        "If an exiled card has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Triple Triad",
        "If an exiled card has {X} in its mana cost, X is 0 for the purpose of determining its mana value."
    );
    ruling!(
        "Kotis, the Fangkeeper",
        "If any of the spells you cast has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Kotis, the Fangkeeper");
    supported("Maralen, Fae Ascendant");
    supported("Triple Triad");
    // Kotis (2 power) deals 2 combat damage: Blaze ({X}{R}, mana value 1) can be cast,
    // with X 0; Hill Giant (4) can't.
    let mut t = TestGame::new(2);
    let kotis = t.battlefield(P0, "Kotis, the Fangkeeper");
    let lib = stack_library(&mut t, P1, &["Blaze", "Hill Giant"]);
    let from = t.asked().len();
    t.answer_choose_moved(P0, &[o(lib[0])]);
    answer_x3(&mut t, P0);
    attack_with(&mut t, &[(kotis, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.settle();
    t.resolve_all();
    assert!(was_offered(&t, from, lib[0]), "{}", t.dump_log());
    assert!(!was_offered(&t, from, lib[1]));
    assert!(t.in_graveyard(P1, "Blaze"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 18, "only the combat damage");
    // Maralen (one Elf Faerie): Blaze (mana value 1) qualifies; Hill Giant (4) doesn't.
    let mut t = TestGame::new(2);
    let lib = stack_library(&mut t, P1, &["Blaze", "Hill Giant"]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Maralen, Fae Ascendant");
    t.resolve_all();
    assert!(exiled(&t, lib[0]) && exiled(&t, lib[1]), "{}", t.dump_log());
    assert!(castable(&mut t, P0, lib[0]));
    assert!(!castable(&mut t, P0, lib[1]));
    // Triple Triad: P0 exiles Shock (1), P1 exiles Blaze (1, not less) and P2 exiles
    // Endless One ({X}: 0, less).
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Triple Triad");
    let shock = t.library_top(P0, "Shock");
    let blaze = t.library_top(P1, "Blaze");
    let one = t.library_top(P2, "Endless One");
    t.set_step(P2, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(exiled(&t, shock) && exiled(&t, blaze) && exiled(&t, one), "{}", t.dump_log());
    t.set_step(P0, Step::PrecombatMain);
    assert!(castable(&mut t, P0, shock));
    assert!(castable(&mut t, P0, one));
    assert!(!castable(&mut t, P0, blaze));
}

#[test]
fn thranduils_decree_card_is_cast_with_x_0() {
    cr!("107.3b", "701.6a");
    ruling!(
        "Thranduil's Decree",
        "If the card has {X} in its mana cost, you must choose 0 as the value for X when casting it without paying its mana cost."
    );
    supported("Thranduil's Decree");
    let mut t = TestGame::new(2);
    let one = t.hand(P1, "Endless One");
    mana(&mut t, P1, ManaType::C, 3);
    t.set_step(P1, Step::PrecombatMain);
    let spell = t.cast(P1, one).x(3).go();
    let d = t.hand(P0, "Thranduil's Decree");
    mana(&mut t, P0, ManaType::U, 6);
    t.g.turn.priority = Some(P0);
    t.cast(P0, d).target(spell).go();
    t.resolve_all();
    assert!(exiled(&t, one), "{}", t.dump_log());
    t.set_step(P0, Step::PrecombatMain);
    answer_x3(&mut t, P0);
    cast_free(&mut t, P0, one);
    t.resolve_all();
    // Endless One entered with no counters: a 0/0, put into P1's graveyard.
    assert!(t.in_graveyard(P1, "Endless One"), "{}", t.dump_log());
}

#[test]
fn cosmium_catalyst_casts_the_crafted_card_with_x_0() {
    cr!("107.3b", "702.167a");
    ruling!(
        "Ore-Rich Stalactite // Cosmium Catalyst",
        "If the card has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    supported("Ore-Rich Stalactite // Cosmium Catalyst");
    let mut t = TestGame::new(2);
    let ore = t.battlefield(P0, "Ore-Rich Stalactite // Cosmium Catalyst");
    let blazes = crate::r_s07_common::graveyard_n(&mut t, P0, "Blaze", 4);
    t.lands(P0, "Mountain", 5);
    t.answer_choose(P0, &blazes.iter().map(|b| o(*b)).collect::<Vec<_>>());
    t.activate(P0, ore, 1, &[]).expect("craft");
    t.resolve_all();
    let cat = t.g.current(ore);
    assert!(t.on_battlefield(cat), "{}", t.dump_log());
    assert!(blazes.iter().all(|b| exiled(&t, *b)));
    t.g.obj_mut(cat).summoning_sick = false;
    mana(&mut t, P0, ManaType::R, 2);
    t.answer_yes(P0, true);
    answer_x3(&mut t, P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, cat, 0, &[]).expect("catalyst");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Blaze"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);
}

#[test]
fn knowledge_pool_card_is_cast_with_x_0() {
    cr!("107.3b");
    ruling!(
        "Knowledge Pool",
        "If the card you cast without paying its mana cost has an X in its mana cost, you must choose 0 as its value."
    );
    supported("Knowledge Pool");
    let mut t = TestGame::new(2);
    let blaze = t.library_top(P1, "Blaze");
    t.enter(P0, "Knowledge Pool");
    t.resolve_all();
    assert!(exiled(&t, blaze));
    let shock = t.hand(P0, "Shock");
    mana(&mut t, P0, ManaType::R, 1);
    t.answer_choose_moved(P0, &[o(blaze)]);
    t.answer_yes(P0, true);
    answer_x3(&mut t, P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, shock).target(P1).go();
    t.resolve();
    assert!(exiled(&t, shock), "{}", t.dump_log());
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Blaze"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);
}

#[test]
fn arcane_savant_copy_is_cast_with_x_0() {
    cr!("107.3b", "707.12");
    ruling!(
        "Arcane Savant",
        "If the copied card has {X} in its mana cost, you must choose 0 as its value when casting the copy."
    );
    supported("Arcane Savant");
    let mut t = TestGame::new(2);
    let side = t.g.add_to_sideboard(P0, vec![mtg_engine::card::card("Blaze")]);
    let x = t
        .g
        .move_object(side[0], Zone::Exile, mtg_engine::events::MoveCause::Effect, Some(P0))
        .unwrap();
    t.g.named_exiles.push((P0, "Arcane Savant".into(), x));
    t.answer_yes(P0, true);
    answer_x3(&mut t, P0);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Arcane Savant");
    t.resolve_all();
    assert!(exiled(&t, x));
    assert!(t.g.turn_events.iter().any(|e| matches!(e, mtg_engine::events::Event::SpellCast { .. })), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);
}

#[test]
fn elite_arcanist_copy_is_cast_with_x_0_whatever_the_activation_x() {
    cr!("107.3b", "707.12", "601.2f");
    ruling!(
        "Elite Arcanist",
        "If the copied card has {X} in its mana cost, you must choose 0 as its value when casting the copy. Don't confuse that {X} with the {X} in the activation cost of Elite Arcanist's ability."
    );
    let mut t = TestGame::new(2);
    let sog = t.hand(P0, "Stroke of Genius");
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[o(sog)]);
    let a = t.enter(P0, "Elite Arcanist");
    t.resolve_all();
    assert!(exiled(&t, sog));
    t.g.obj_mut(a).summoning_sick = false;
    // X of the activation: Stroke of Genius's mana value, 3.
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_targets(P0, &[Entity::Player(P0)]);
    let hand = t.hand_size(P0);
    t.activate(P0, a, 0, &[]).unwrap();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 3, "{}", t.dump_log());
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand, "the copy's X is 0: {}", t.dump_log());
}

#[test]
fn spellbinder_copy_of_a_split_card_casts_one_half_each_time() {
    cr!("709.3", "707.12", "709.3a");
    ruling!(
        "Spellbinder",
        "If the copied card is a split card, you choose which one side of it to cast, but you can't cast both sides. You can choose the other side later in the game, though."
    );
    supported("Spellbinder");
    let mut t = TestGame::new(2);
    let fi = t.hand(P0, "Fire // Ice");
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[o(fi)]);
    let sb = t.enter(P0, "Spellbinder");
    t.resolve_all();
    assert!(exiled(&t, fi), "{}", t.dump_log());
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sb = t.g.current(sb);
    assert!(t.g.attach(sb, o(bears)));
    t.g.recompute();
    // First combat: Fire, 2 damage to P1.
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![2]));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.settle();
    t.resolve_all();
    let methods: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseCastingMethod { options, .. } => Some(options.clone()),
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(t.life(P1), 16, "{methods:?}\n{}", t.dump_log());
    // Next turn: Ice, tapping P1's land and drawing a card.
    let land = t.battlefield(P1, "Forest");
    t.advance_to(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_targets(P0, &[o(land)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert!(t.obj_now(land).tapped, "{}", t.dump_log());
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn tecutlan_discovers_the_spells_mana_value_with_its_chosen_x() {
    cr!("202.3e", "701.57a", "107.3c");
    ruling!(
        "Brass's Tunnel-Grinder // Tecutlan, the Searing Rift",
        "If a permanent spell cast using mana produced by Tecutlan, the Searing Rift has {X} in its mana cost, use the value chosen for X when calculating that spell's mana value."
    );
    supported("Brass's Tunnel-Grinder // Tecutlan, the Searing Rift");
    let mut t = TestGame::new(2);
    let tec = crate::r_s17_common::enter_transformed(
        &mut t,
        P0,
        "Brass's Tunnel-Grinder // Tecutlan, the Searing Rift",
    );
    assert!(t.obj_now(tec).is(CardType::Land));
    t.g.obj_mut(t.g.current(tec)).summoning_sick = false;
    t.lands(P0, "Wastes", 1);
    let lib = stack_library(&mut t, P0, &["Hill Giant", "Shock"]);
    let one = t.hand(P0, "Endless One");
    // Endless One with X 2 (mana value 2): discover 2 passes Hill Giant and finds Shock.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, one).x(2).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Shock"), "{}", t.dump_log());
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.zone(lib[0]), Zone::Library(P0));
    assert_eq!(t.counters(one, "+1/+1"), 2);
}
