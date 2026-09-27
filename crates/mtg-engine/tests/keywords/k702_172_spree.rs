//! CR 702.172 Spree.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_140_152::*;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::{ManaCost, ManaType};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::*;

const ACCIDENT: &str = "Unfortunate Accident";

/// The modal choice of a spell with spree, as compiled.
fn spree_modes(name: &str) -> Modal {
    card(name)
        .front()
        .chars
        .abilities
        .iter()
        .find_map(|a| match &a.kind {
            AbilityKind::Spell(s) => s.body.modal.clone(),
            _ => None,
        })
        .expect("modal spell ability")
}

/// The modes `p` was offered for the most recent modal choice.
fn modes_offered(t: &TestGame, p: PlayerId) -> (u32, u32, usize) {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseModes {
                min, max, modes, ..
            } if q == p => Some((min, max, modes.len())),
            _ => None,
        })
        .expect("modes asked")
}

#[test]
fn spree_chooses_one_or_more_modes_paying_each_modes_cost() {
    cr!("702.172", "702.172a");
    assert_supported(ACCIDENT);
    // Unfortunate Accident: {B} instant, spree, "+ {2}{B} — Destroy target creature." "+
    // {1} — Create a 1/1 red Mercenary creature token with ..."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let accident = t.hand(P0, ACCIDENT);
    add_mana(&mut t, P0, ManaType::B, 5);
    let spell = t.cast(P0, accident).modes(&[0, 1]).target(bears).go();
    // One or more of its two modes.
    assert_eq!(modes_offered(&t, P0), (1, 2, 2));
    // {B} + {2}{B} + {1}; its mana value is still 1.
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(creature_tokens(&t, P0).len(), 1);
    // Only the second mode: {B} + {1}.
    let mut t = TestGame::new(2);
    let accident = t.hand(P0, ACCIDENT);
    add_mana(&mut t, P0, ManaType::B, 5);
    t.cast(P0, accident).modes(&[1]).go();
    assert_eq!(pool(&t, P0), 3);
    t.resolve_all();
    assert_eq!(creature_tokens(&t, P0).len(), 1);
}

#[test]
fn a_mode_can_be_chosen_only_if_its_cost_and_targets_are_available() {
    cr!("702.172a");
    ruling!(
        "Unfortunate Accident",
        "If a mode requires a target, you can select that mode only if there’s a legal target available. Ignore the targeting requirements for modes you don’t choose."
    );
    ruling!(
        "Unfortunate Accident",
        "You must choose at least one of the listed modes and pay its associated additional cost in order to cast a spell with spree."
    );
    ruling!(
        "Unfortunate Accident",
        "You can’t choose the same mode more than once."
    );
    // No creature to destroy: the destroy mode can't be chosen (asking for it, or for no
    // mode, or for a mode twice, gets the one mode that can be chosen).
    for answer in [vec![0], vec![], vec![1, 1]] {
        let mut t = TestGame::new(2);
        let accident = t.hand(P0, ACCIDENT);
        add_mana(&mut t, P0, ManaType::B, 4);
        let spell = t.cast(P0, accident).modes(&answer).go();
        let chosen: Vec<Option<usize>> = t
            .obj(spell)
            .stack
            .as_ref()
            .unwrap()
            .chosen
            .iter()
            .map(|c| c.mode)
            .collect();
        assert_eq!(chosen, vec![Some(1)], "answer {answer:?}");
        // {B} + {1}.
        assert_eq!(pool(&t, P0), 2);
        t.resolve_all();
        assert_eq!(creature_tokens(&t, P0).len(), 1);
    }
    // {B} alone pays for no mode: it can't be cast.
    let mut t = TestGame::new(2);
    let accident = t.hand(P0, ACCIDENT);
    add_mana(&mut t, P0, ManaType::B, 1);
    assert!(t.cast(P0, accident).modes(&[1]).try_go().is_err());
    assert!(t.in_hand(P0, ACCIDENT));
}

#[test]
fn casting_without_paying_the_mana_cost_still_pays_the_mode_costs() {
    cr!("702.172a");
    ruling!(
        "Unfortunate Accident",
        "If an effect allows you to cast a spell with spree “without paying its mana cost,” you must still choose at least one mode and pay the associated additional costs."
    );
    ruling!(
        "Unfortunate Accident",
        "The mana value of a spell with spree is determined only by its mana cost (in the upper right corner of the card). It doesn’t matter which modes you choose or which additional costs you pay, including any additional costs imposed by other effects."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let accident = t.hand(P0, ACCIDENT);
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![accident],
        Duration::EndOfTurn,
        true,
        None,
    );
    add_mana(&mut t, P0, ManaType::B, 3);
    let spell = t
        .cast(P0, accident)
        .method(CastMethod::Free)
        .modes(&[0])
        .target(bears)
        .go();
    // The mana cost {B} wasn't paid; the mode's {2}{B} was.
    assert_eq!(pool(&t, P0), 0);
    assert_eq!(t.g.mana_value_of(spell), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn modes_are_chosen_as_its_cast_and_performed_in_printed_order() {
    cr!("702.172a");
    assert_supported("Requisition Raid");
    ruling!(
        "Requisition Raid",
        "No matter which modes you choose, you always follow the instructions in the order they are written."
    );
    ruling!(
        "Requisition Raid",
        "If all targets for the chosen modes become illegal before a spell with spree resolves, the spell won't resolve and none of its effects will happen. If at least one target is still legal, the spell will resolve but will have no effect on any illegal targets."
    );
    // Requisition Raid: {W} sorcery, spree, "+ {1} — Destroy target artifact." "+ {1} —
    // Destroy target enchantment." "+ {1} — Put a +1/+1 counter on each creature target
    // player controls."
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let raid = t.hand(P0, "Requisition Raid");
    add_mana(&mut t, P0, ManaType::W, 4);
    t.cast(P0, raid)
        .modes(&[2, 0])
        .targets(&[Entity::Object(thopter)])
        .targets(&[Entity::Player(P0)])
        .go();
    // The artifact leaves before it resolves: the other mode still happens.
    crate::common_k702_052_066::destroy(&mut t, thopter);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.named_on_battlefield("Glorious Anthem"), vec![anthem]);
}

#[test]
fn the_plus_signs_are_reminders_of_the_additional_costs() {
    cr!("702.172b");
    ruling!(
        "Unfortunate Accident",
        "Each additional cost and associated mode in the text box is also preceded with a + indicator. These symbols also have no rules meaning and serve only to remind players that the listed costs are additional costs."
    );
    // The "+ [cost] — [effect]" lines are the spell's modes, each with its additional
    // cost; the spell has the spree keyword.
    let c = card(ACCIDENT);
    assert!(c
        .front()
        .chars
        .abilities
        .iter()
        .any(|a| matches!(&a.kind, AbilityKind::Keyword(k) if k.kind == KeywordKind::Spree)));
    let modal = spree_modes(ACCIDENT);
    assert!(modal.per_mode_cost);
    let costs: Vec<Option<ManaCost>> = modal
        .modes
        .iter()
        .map(|m| m.cost.as_ref().and_then(|c| c.mana.clone()))
        .collect();
    assert_eq!(
        costs,
        vec![ManaCost::parse("{2}{B}"), ManaCost::parse("{1}")]
    );
    // The mana cost printed with the + sign is just its mana cost.
    assert_eq!(c.front().chars.mana_cost, ManaCost::parse("{B}"));
    // Nothing is asked about the + symbols: casting asks only for modes.
    let mut t = TestGame::new(2);
    let accident = t.hand(P0, ACCIDENT);
    add_mana(&mut t, P0, ManaType::B, 2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.cast(P0, accident).go();
    assert!(t
        .asked()
        .iter()
        .all(|(_, d)| !matches!(d, Decision::OptionalCost { .. })));
}
