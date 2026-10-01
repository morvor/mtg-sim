//! Rulings batch S35 — miscellaneous shared rulings: searching for a card with a stated
//! quality (CR 701.23b), modes in printed order (CR 700.2), creature spells, "sacrifice it
//! unless" after it left, the monarch (CR 725), paying energy once, mana added by a
//! triggered ability, cumulative cost reductions, attack costs in Two-Headed Giant, and
//! friend or foe.

use crate::r_s01_common::{supported, watch, with_subtype};
use crate::r_s02_common::{can_cast, destroy};
use crate::r_s05_common::enter;
use crate::r_s09_common::declare;
use crate::r_s13_common::add;
use crate::r_s24_common::pool;
use crate::r_s28_common::energy;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::GameConfig;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn a_search_for_a_basic_land_card_may_find_nothing() {
    cr!("701.23b");
    ruling!(
        "Rampant Growth",
        "Because the \"search\" requires you to find a card with certain characteristics, you don't have to find the card if you don't want to."
    );
    supported("Rampant Growth");
    // "Search your library for a basic land card, put that card onto the battlefield
    // tapped, then shuffle."
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    t.lands(P0, "Forest", 2);
    t.answer_choose(P0, &[]);
    let from = t.asked().len();
    let growth = t.hand(P0, "Rampant Growth");
    t.cast(P0, growth).go();
    t.resolve_all();
    let searched: Vec<(u32, usize)> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities {
                min, candidates, ..
            } => Some((*min, candidates.len())),
            _ => None,
        })
        .collect();
    assert_eq!(searched, vec![(0, 1)]);
    assert!(!t.on_battlefield(forest));
    assert!(t.in_graveyard(P0, "Rampant Growth"));
    // Choosing to find it puts it onto the battlefield tapped.
    let mut t = TestGame::new(2);
    let forest = t.library_top(P0, "Forest");
    t.lands(P0, "Forest", 2);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    let growth = t.hand(P0, "Rampant Growth");
    t.cast(P0, growth).go();
    t.resolve_all();
    assert!(t.on_battlefield(forest));
    assert!(t.obj_now(forest).tapped);
}

#[test]
fn modes_are_performed_in_printed_order() {
    cr!("700.2", "608.2c");
    ruling!(
        "Collective Resistance",
        "No matter which combination of modes you choose, you always follow the instructions in the order they are written."
    );
    supported("Collective Resistance");
    // "Escalate {G}. Choose one or more — • Destroy target artifact. • Destroy target
    // enchantment. • Target creature gains hexproof and indestructible until end of turn."
    // Choosing the third mode, then the first, for the same artifact creature: it's
    // destroyed first, before it would gain indestructible.
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    t.lands(P0, "Forest", 3);
    let spell = t.hand(P0, "Collective Resistance");
    t.cast(P0, spell)
        .modes(&[2, 0])
        .target(thopter)
        .target(thopter)
        .go();
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Ornithopter"));
}

#[test]
fn an_artifact_creature_spell_is_a_creature_spell() {
    cr!("205.2a", "601.2i");
    ruling!(
        "Skittering Skirge",
        "A \"creature spell\" is any spell with the type Creature, even if it has other types such as Artifact or Enchantment. Older cards of type Summon are also Creature spells."
    );
    supported("Skittering Skirge");
    // "When you cast a creature spell, sacrifice this creature." A noncreature artifact
    // spell doesn't trigger it; an artifact creature spell does.
    let mut t = TestGame::new(2);
    let skirge = t.battlefield(P0, "Skittering Skirge");
    t.lands(P0, "Plains", 1);
    let ring = t.hand(P0, "Sol Ring");
    t.cast(P0, ring).go();
    t.resolve_all();
    assert!(t.on_battlefield(skirge));
    let thopter = t.hand(P0, "Ornithopter");
    t.cast(P0, thopter).go();
    t.resolve_all();
    assert!(!t.on_battlefield(skirge));
    assert!(t.in_graveyard(P0, "Skittering Skirge"));
}

#[test]
fn the_action_may_still_be_performed_after_the_creature_left() {
    cr!("118.12a");
    ruling!(
        "Rogue Elephant",
        "If the creature is no longer on the battlefield when the ability resolves, you may still perform the action if you want."
    );
    supported("Rogue Elephant");
    // "When this creature enters, sacrifice it unless you sacrifice a Forest." It's
    // destroyed in response; P0 may still sacrifice a Forest.
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P0, "Forest");
    let elephant = enter(&mut t, P0, "Rogue Elephant");
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, elephant);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
    assert!(!t.on_battlefield(forest));
    assert!(t.in_graveyard(P0, "Forest"));
}

#[test]
fn there_is_at_most_one_monarch() {
    cr!("725.1", "725.3");
    ruling!(
        "Court of Embereth",
        "The game starts with no monarch. As a player becomes the monarch, the current monarch (if any) ceases being the monarch. There is never more than one monarch at a time."
    );
    supported("Court of Embereth");
    supported("Grave Venerations");
    // Court of Embereth: "When this enchantment enters, you become the monarch."
    let mut t = TestGame::new(3);
    assert_eq!(t.g.monarch, None);
    enter(&mut t, P0, "Court of Embereth");
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P0));
    enter(&mut t, P1, "Grave Venerations");
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P1));
    enter(&mut t, P2, "Court of Embereth");
    t.resolve_all();
    assert_eq!(t.g.monarch, Some(P2));
}

#[test]
fn an_energy_payment_is_made_once_as_the_ability_resolves() {
    cr!("107.14", "118.12a", "603.12");
    ruling!(
        "Saheeli, Radiant Creator",
        "Some spells and abilities say that you “may pay” a certain amount of {E}. You can’t pay that amount multiple times to multiply the effect. You simply choose whether or not to pay that amount of {E} as the ability resolves."
    );
    supported("Saheeli, Radiant Creator");
    // "At the beginning of combat on your turn, you may pay {E}{E}{E}. When you do,
    // create a token that's a copy of target permanent you control, except it's a 5/5
    // artifact creature in addition to its other types and has haste. Sacrifice it at the
    // beginning of the next end step." With seven energy: one payment, one token.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Saheeli, Radiant Creator");
    let bears = t.battlefield(P0, "Grizzly Bears");
    add(&mut t, P0, "energy", 7);
    // When P0 decides whether to pay: the energy then, and the step.
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| (g.player(P0).counter("energy"), g.turn.step),
    );
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    // One choice, made as the ability resolved in the beginning of combat step: one
    // payment of three.
    assert_eq!(
        seen.lock().unwrap().clone(),
        vec![(7, Step::BeginningOfCombat)]
    );
    assert_eq!(energy(&t, P0), 4);
    let copies: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.chars.name == "Grizzly Bears")
        .map(|o| o.id)
        .collect();
    assert_eq!(copies.len(), 1);
    assert_eq!(t.pt(copies[0]), (5, 5));
    assert!(t
        .obj_now(copies[0])
        .chars
        .card_types
        .contains(mtg_engine::types::CardType::Artifact));
    // It's sacrificed at the beginning of the next end step.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(copies[0]));
}

#[test]
fn mana_added_by_a_triggered_ability_empties_at_the_end_of_the_step() {
    cr!("106.4", "500.5");
    ruling!(
        "Priest of Gix",
        "You get the mana whether you want it or not. If you don't spend it, it will disappear at the end of the current step (or phase)."
    );
    supported("Priest of Gix");
    // "When this creature enters, add {B}{B}{B}."
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    enter(&mut t, P0, "Priest of Gix");
    t.resolve_all();
    assert_eq!(pool(&t, P0, ManaType::B), 3);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(pool(&t, P0, ManaType::B), 0);
}

#[test]
fn two_helms_of_awakening_reduce_costs_by_two() {
    cr!("601.2f");
    ruling!("Helm of Awakening", "The effect is cumulative.");
    supported("Helm of Awakening");
    // "Spells cost {1} less to cast." Hill Giant costs {3}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Helm of Awakening");
    t.lands(P0, "Mountain", 2);
    let giant = t.hand(P0, "Hill Giant");
    assert!(!can_cast(&mut t, P0, giant, CastMethod::Normal));
    t.battlefield(P0, "Helm of Awakening");
    assert!(can_cast(&mut t, P0, giant, CastMethod::Normal));
    t.cast(P0, giant).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn propaganda_taxes_a_creature_once_in_two_headed_giant() {
    cr!("508.1h", "805.10b", "810.7");
    ruling!(
        "Propaganda",
        "In the Two-Headed Giant format, you still only have to pay once per creature."
    );
    supported("Propaganda");
    // P0 and P1 against P2 and P3, who each control Propaganda ("Creatures can't attack
    // you unless their controller pays {2} for each creature they control that's
    // attacking you."). A creature attacks one player of the team: {2}, not {4}.
    let thg = || TestGame::with_config(4, GameConfig::two_headed_giant(vec![0, 0, 1, 1]));
    let mut t = thg();
    t.battlefield(P2, "Propaganda");
    t.battlefield(P3, "Propaganda");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let plains = t.lands(P0, "Plains", 3);
    let attacks = declare(&mut t, P0, &[(bears, Entity::Player(P2))]);
    assert_eq!(attacks, vec![(bears, Entity::Player(P2))]);
    let tapped = plains.iter().filter(|l| t.obj_now(**l).tapped).count();
    assert_eq!(tapped, 2);
    // Two creatures, one attacking each defending player: {2} each.
    let mut t = thg();
    t.battlefield(P2, "Propaganda");
    t.battlefield(P3, "Propaganda");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let plains = t.lands(P0, "Plains", 4);
    let attacks = declare(
        &mut t,
        P0,
        &[(a, Entity::Player(P2)), (b, Entity::Player(P3))],
    );
    assert_eq!(attacks.len(), 2);
    assert!(plains.iter().all(|l| t.obj_now(*l).tapped));
}

#[test]
fn you_may_call_yourself_a_foe() {
    cr!("608.2c");
    ruling!(
        "Pir's Whim",
        "You make this choice for yourself as well as each other player. In some rare cases, you may wish to call yourself (or your teammate in a Two-Headed Giant game) a foe. You can do that."
    );
    supported("Pir's Whim");
    // "For each player, choose friend or foe. Each friend searches their library for a
    // land card, puts it onto the battlefield tapped, then shuffles. Each foe sacrifices an
    // artifact or enchantment of their choice." P0 calls itself a foe (option 1) and P1 a
    // friend (option 0).
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    let forest = t.library_top(P1, "Forest");
    t.lands(P0, "Forest", 4);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    let whim = t.hand(P0, "Pir's Whim");
    t.cast(P0, whim).go();
    t.resolve_all();
    assert!(!t.on_battlefield(ring));
    assert!(t.in_graveyard(P0, "Sol Ring"));
    assert!(t.on_battlefield(forest));
    assert!(with_subtype(&t, P0, "Forest").len() == 4);
}
