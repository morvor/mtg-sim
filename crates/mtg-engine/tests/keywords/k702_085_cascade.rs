//! CR 702.85 Cascade.

use crate::common_k702_011_017::{assert_supported, give_mana_for};
use crate::common_k702_018_026::triggers_on_stack;
use mtg_engine::ability::*;
use mtg_engine::decision::{Agent, Answer, Decision, PassiveAgent};
use mtg_engine::game::Game;
use mtg_engine::object::{CastMethod, FaceState, StackKind};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Puts real cards on top of `p`'s library, the first one on top.
fn stack_library(t: &mut TestGame, p: PlayerId, top_first: &[&str]) -> Vec<ObjectId> {
    top_first
        .iter()
        .rev()
        .map(|n| t.library_top(p, n))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect()
}

/// Casts the real card `name` from `p`'s hand (with the mana for it) and returns the spell.
fn cast_from_hand(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    give_mana_for(t, p, name);
    let card = t.hand(p, name);
    t.cast(p, card).go()
}

/// Names of the spells on the stack, bottom first.
fn spell_names(t: &TestGame) -> Vec<String> {
    t.g.stack
        .iter()
        .filter(|s| t.g.obj(**s).is_spell())
        .map(|s| t.g.obj(*s).chars.name.to_string())
        .collect()
}

/// Names of the bottom `n` cards of `p`'s library (bottom first).
fn bottom_names(t: &TestGame, p: PlayerId, n: usize) -> Vec<String> {
    t.g.player(p)
        .library
        .iter()
        .take(n)
        .map(|c| t.g.obj(*c).chars.name.to_string())
        .collect()
}

#[test]
fn cascade_exiles_until_a_cheaper_nonland_card_and_casts_it_for_free() {
    cr!("702.85", "702.85a");
    ruling!(
        "Averna, the Chaos Bloom",
        "Cascade triggers when you cast the spell, meaning that it resolves before that spell. If you end up casting the exiled card, it will go on the stack above the spell with cascade."
    );
    ruling!(
        "Averna, the Chaos Bloom",
        "You exile the cards face up. All players will be able to see them."
    );
    assert_supported("Bloodbraid Elf");
    let mut t = TestGame::new(2);
    stack_library(
        &mut t,
        P0,
        &["Forest", "Hill Giant", "Mountain", "Grizzly Bears", "Llanowar Elves"],
    );
    let lib = t.library_size(P0);
    let elf = cast_from_hand(&mut t, P0, "Bloodbraid Elf");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 1);
    // Resolve the cascade trigger (it's above the spell).
    t.resolve();
    // Grizzly Bears (mana value 2 < 4) was cast without paying its mana cost, above the
    // Elf; Hill Giant (4) doesn't have a lesser mana value.
    assert_eq!(spell_names(&t), vec!["Bloodbraid Elf", "Grizzly Bears"]);
    let bears = *t.g.stack.last().unwrap();
    let cast = &t.g.obj(bears).stack.as_ref().unwrap().cast;
    assert!(cast.was_cast);
    assert_eq!(cast.method, CastMethod::Free);
    // The other exiled cards are on the bottom of the library, in some order; Llanowar
    // Elves stays on top.
    let mut bottom = bottom_names(&t, P0, 3);
    bottom.sort();
    assert_eq!(bottom, vec!["Forest", "Hill Giant", "Mountain"]);
    assert_eq!(t.library_size(P0), lib - 1);
    let top = *t.g.player(P0).library.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name, "Llanowar Elves");
    assert!(t.g.exile.is_empty());
    t.resolve_all();
    assert!(t.on_battlefield(elf));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn a_card_not_cast_goes_to_the_bottom_with_the_rest() {
    cr!("702.85a");
    ruling!(
        "Averna, the Chaos Bloom",
        "When the cascade ability resolves, you must exile cards. The only optional part of the ability is whether or not you cast the last card exiled."
    );
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Forest", "Grizzly Bears"]);
    cast_from_hand(&mut t, P0, "Bloodbraid Elf");
    t.answer_yes(P0, false);
    t.resolve();
    assert_eq!(spell_names(&t), vec!["Bloodbraid Elf"]);
    let mut bottom = bottom_names(&t, P0, 2);
    bottom.sort();
    assert_eq!(bottom, vec!["Forest", "Grizzly Bears"]);
}

#[test]
fn a_countered_spell_still_cascades() {
    cr!("702.85a");
    ruling!(
        "Averna, the Chaos Bloom",
        "If a spell with cascade is countered, the cascade ability will still resolve normally."
    );
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Grizzly Bears"]);
    let elf = cast_from_hand(&mut t, P0, "Bloodbraid Elf");
    t.settle();
    assert!(t.g.counter(elf, None));
    t.resolve();
    assert!(t.in_graveyard(P0, "Bloodbraid Elf"));
    assert_eq!(spell_names(&t), vec!["Grizzly Bears"]);
}

#[test]
fn a_split_card_counts_its_combined_mana_value_and_either_half_can_be_cast() {
    cr!("702.85a");
    ruling!(
        "Averna, the Chaos Bloom",
        "The mana value of a split card is determined by the combined mana cost of its two halves. If cascade allows you to cast a split card, you may cast either half but not both halves."
    );
    // Bloodbraid Elf (4): Fire // Ice has mana value 4, so it's skipped.
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Fire // Ice", "Grizzly Bears"]);
    cast_from_hand(&mut t, P0, "Bloodbraid Elf");
    t.resolve();
    assert_eq!(spell_names(&t), vec!["Bloodbraid Elf", "Grizzly Bears"]);
    // Enlisted Wurm (6): it's hit, and either half may be cast.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    stack_library(&mut t, P0, &["Fire // Ice"]);
    cast_from_hand(&mut t, P0, "Enlisted Wurm");
    // Choose the second option (Ice), targeting the Bears.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve();
    let options = t
        .asked()
        .into_iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options),
            _ => None,
        })
        .expect("a choice of halves");
    assert_eq!(options.len(), 3);
    assert!(options[0].contains("Fire"));
    assert!(options[1].contains("Ice"));
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name, "Ice");
    assert_eq!(t.g.obj(top).face, FaceState::Half(1));
}

#[test]
fn the_resulting_spell_must_have_a_lesser_mana_value() {
    cr!("702.85a");
    ruling!(
        "Averna, the Chaos Bloom",
        "not only do you stop exiling cards if you exile a nonland card with lesser mana value than the spell with cascade, but the resulting spell you cast must also have lesser mana value."
    );
    // Esika, God of the Tree (front {1}{G}{G}, 3) // The Prismatic Bridge (back {W}{U}{B}{R}{G}, 5).
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Esika, God of the Tree"]);
    cast_from_hand(&mut t, P0, "Bloodbraid Elf");
    t.resolve();
    // Only the front face could be cast: a yes/no question rather than a choice of faces.
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseOption { .. })));
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name, "Esika, God of the Tree");
}

#[test]
fn a_card_with_x_is_cast_with_x_zero() {
    cr!("702.85a", "107.3b");
    ruling!(
        "Averna, the Chaos Bloom",
        "If the card has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Blaze"]);
    cast_from_hand(&mut t, P0, "Bloodbraid Elf");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name, "Blaze");
    assert_eq!(t.g.obj(top).stack.as_ref().unwrap().x, Some(0));
    t.resolve();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn each_instance_of_cascade_triggers_separately() {
    cr!("702.85c");
    ruling!(
        "Maelstrom Wanderer",
        "Each instance of cascade triggers and resolves separately. The spell you cast due to the first cascade ability will go on the stack on top of the second cascade ability. That spell will resolve before you exile cards for the second cascade ability."
    );
    ruling!(
        "Maelstrom Wanderer",
        "the second cascade trigger will look for a spell with mana value less than Maelstrom Wanderer's mana value of 8."
    );
    assert_supported("Maelstrom Wanderer");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Hill Giant", "Enlisted Wurm"]);
    cast_from_hand(&mut t, P0, "Maelstrom Wanderer");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 2);
    // First cascade: Hill Giant goes on the stack above the second trigger.
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 1);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name, "Hill Giant");
    t.resolve();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    // Second cascade: Enlisted Wurm (6 < 8) is cast, and cascades itself.
    t.resolve();
    let top = *t.g.stack.last().unwrap();
    assert!(matches!(
        t.g.obj(top).stack.as_ref().unwrap().kind,
        StackKind::Triggered { .. }
    ));
    assert_eq!(
        spell_names(&t),
        vec!["Maelstrom Wanderer", "Enlisted Wurm"]
    );
}

#[test]
fn as_you_cascade_a_land_can_be_put_onto_the_battlefield() {
    cr!("702.85b");
    ruling!(
        "Averna, the Chaos Bloom",
        "Averna's ability isn't the same as playing a land. You may do this even if you've already played a land during your turn and even if it isn't your turn at all."
    );
    assert_supported("Averna, the Chaos Bloom");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Averna, the Chaos Bloom");
    stack_library(&mut t, P0, &["Forest", "Island", "Grizzly Bears"]);
    t.g.players[0].lands_played_this_turn = 1;
    cast_from_hand(&mut t, P0, "Bloodbraid Elf");
    pick_land(&mut t, P0, "Island");
    t.answer_yes(P0, false);
    t.resolve();
    let island = t.named_on_battlefield("Island");
    assert_eq!(island.len(), 1);
    assert!(t.g.obj(island[0]).tapped);
    // The land choice happens before deciding whether to cast Grizzly Bears.
    let asked = t.asked();
    let land_q = asked
        .iter()
        .position(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
        .expect("land choice");
    let cast_q = asked
        .iter()
        .position(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .expect("cast choice");
    assert!(land_q < cast_q);
    let mut bottom = bottom_names(&t, P0, 2);
    bottom.sort();
    assert_eq!(bottom, vec!["Forest", "Grizzly Bears"]);
    assert_eq!(t.g.player(P0).lands_played_this_turn, 1);
}

#[test]
fn as_you_cascade_works_even_without_a_card_to_cast() {
    cr!("702.85b");
    ruling!(
        "Averna, the Chaos Bloom",
        "If you don't cast the nonland card you reveal, or if you don't reveal any nonland cards with lesser mana value, you can still put a land card onto the battlefield before you finish cascading."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Averna, the Chaos Bloom");
    // A library of lands only: everything is exiled.
    t.g.players[0].library.clear();
    stack_library(&mut t, P0, &["Forest", "Plains"]);
    cast_from_hand(&mut t, P0, "Bloodbraid Elf");
    pick_land(&mut t, P0, "Plains");
    t.resolve();
    assert_eq!(t.named_on_battlefield("Plains").len(), 1);
    assert_eq!(bottom_names(&t, P0, 1), vec!["Forest"]);
    assert_eq!(t.library_size(P0), 1);
}

/// Answers `p`'s "as you cascade" land choice with the exiled card named `name`, leaving
/// every other decision to the scripted agent.
struct PickLandByName {
    inner: Box<dyn Agent>,
    name: &'static str,
}

impl Agent for PickLandByName {
    fn decide(&mut self, g: &Game, p: PlayerId, d: &Decision) -> Answer {
        if let Decision::ChooseEntities {
            prompt, candidates, ..
        } = d
        {
            if prompt.contains("cascade") {
                let pick: Vec<Entity> = candidates
                    .iter()
                    .copied()
                    .filter(|e| {
                        e.object()
                            .is_some_and(|o| g.obj(o).chars.name == self.name)
                    })
                    .take(1)
                    .collect();
                // Keep the scripted agent's log of decisions.
                let _ = self.inner.decide(g, p, d);
                return Answer::Entities(pick);
            }
        }
        self.inner.decide(g, p, d)
    }
}

fn pick_land(t: &mut TestGame, p: PlayerId, name: &'static str) {
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(&mut agents[p.idx()], Box::new(PassiveAgent));
    agents[p.idx()] = Box::new(PickLandByName { inner, name });
}

#[test]
fn spells_given_cascade_by_an_effect_cascade() {
    cr!("702.85a");
    assert_supported("The First Sliver");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "The First Sliver");
    stack_library(&mut t, P0, &["Llanowar Elves"]);
    cast_from_hand(&mut t, P0, "Muscle Sliver");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 1);
    t.resolve();
    assert_eq!(spell_names(&t), vec!["Muscle Sliver", "Llanowar Elves"]);
    // A non-Sliver spell doesn't have cascade.
    t.resolve_all();
    cast_from_hand(&mut t, P0, "Hill Giant");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 0);
}

#[test]
fn the_next_spell_given_cascade_cascades() {
    cr!("702.85a");
    assert_supported("Dark Apostle");
    let mut t = TestGame::new(2);
    let apostle = t.battlefield(P0, "Dark Apostle");
    t.lands(P0, "Wastes", 3);
    // "{3}, {T}: The next noncreature spell you cast this turn has cascade."
    let text = t
        .obj_now(apostle)
        .chars
        .abilities
        .iter()
        .find(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .unwrap()
        .text
        .clone();
    crate::common_k702_027_037::activate_named(&mut t, P0, apostle, &text, 0).expect("activate");
    t.resolve();
    stack_library(&mut t, P0, &["Grizzly Bears", "Shock"]);
    // A creature spell doesn't get cascade.
    cast_from_hand(&mut t, P0, "Hill Giant");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 0);
    t.resolve_all();
    // The next noncreature spell does: Divination (3) cascades into Grizzly Bears (2).
    cast_from_hand(&mut t, P0, "Divination");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Cascade"), 1);
    t.resolve();
    assert_eq!(spell_names(&t), vec!["Divination", "Grizzly Bears"]);
}
