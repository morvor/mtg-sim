//! CR 702.144 Demonstrate.

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_140_152::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::ObjKind;
use mtg_engine::testing::*;
use mtg_engine::*;

/// The spells on the stack from bottom to top, with their controllers and whether each
/// is a copy.
fn spells_on_stack(t: &TestGame) -> Vec<(PlayerId, bool)> {
    t.g.stack
        .iter()
        .filter(|s| matches!(t.g.obj(**s).kind, ObjKind::Card | ObjKind::SpellCopy))
        .map(|s| (t.g.obj(*s).controller, t.g.obj(*s).kind == ObjKind::SpellCopy))
        .collect()
}

#[test]
fn you_may_copy_it_and_then_an_opponent_you_choose_copies_it() {
    cr!("702.144", "702.144a");
    assert_supported("Excavation Technique");
    ruling!(
        "Replication Technique",
        "This means that if you cast a spell with demonstrate and both you and an opponent copy it, the opponent's copy will resolve first, then your copy will resolve, and finally the original spell will resolve."
    );
    ruling!(
        "Incarnation Technique",
        "Similarly, the opponent you chose to create a copy may choose new targets for that copy as it’s created."
    );
    ruling!(
        "Incarnation Technique",
        "You choose whether to make a copy as the demonstrate ability resolves. This happens before the original spell resolves. Your copy goes on the stack above the original spell."
    );
    ruling!(
        "Incarnation Technique",
        "If you copy a spell with demonstrate, you then immediately choose an opponent. If they copy the spell, it goes on top of the stack."
    );
    let mut t = TestGame::new(3);
    // Excavation Technique: "Demonstrate. Destroy target nonland permanent. Its controller
    // creates two Treasure tokens."
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Grizzly Bears");
    let c = t.battlefield(P0, "Sol Ring");
    let spell = t.hand(P0, "Excavation Technique");
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::C, 3);
    t.cast(P0, spell).target(a).go();
    // P0 copies it and chooses a new target for the copy, then chooses P2, who copies it
    // and chooses a new target too.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer_yes(P2, true);
    t.answer_targets(P2, &[Entity::Object(c)]);
    t.settle();
    // The demonstrate trigger resolves first.
    t.resolve();
    assert_eq!(
        spells_on_stack(&t),
        vec![(P0, false), (P0, true), (P2, true)]
    );
    // P2's copy resolves first: the Sol Ring is destroyed and P0 gets two Treasures.
    t.resolve();
    assert!(!t.g.is_live(c));
    assert!(t.g.is_live(a) && t.g.is_live(b));
    t.resolve_all();
    assert!(!t.g.is_live(a) && !t.g.is_live(b));
    let treasures = |p: PlayerId| {
        t.named_on_battlefield("Treasure Token")
            .into_iter()
            .filter(|x| t.obj(*x).controller == p)
            .count()
    };
    assert_eq!((treasures(P0), treasures(P1), treasures(P2)), (2, 4, 0));
}

#[test]
fn if_you_dont_copy_it_no_opponent_copies_it() {
    cr!("702.144a");
    ruling!(
        "Incarnation Technique",
        "If you cast the spell and choose not to copy it, no opponent will get to copy it either."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Excavation Technique");
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::C, 3);
    t.cast(P0, spell).target(a).go();
    t.answer_yes(P0, false);
    t.settle();
    t.resolve();
    assert_eq!(spells_on_stack(&t), vec![(P0, false)]);
    t.resolve_all();
    assert!(!t.g.is_live(a));
    assert_eq!(t.named_on_battlefield("Treasure Token").len(), 2);
}

#[test]
fn copies_of_a_creature_spell_with_demonstrate_become_tokens() {
    cr!("702.144a");
    assert_supported("Silverquill Lecturer");
    ruling!(
        "Silverquill Lecturer",
        "Resolving copies of permanent spells become tokens as they enter the battlefield."
    );
    let mut t = TestGame::new(2);
    // Silverquill Lecturer: "Creature spells you cast have demonstrate."
    t.battlefield(P0, "Silverquill Lecturer");
    let bears = t.hand(P0, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::G, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, bears).go();
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![Entity::Player(P1)]));
    t.resolve_all();
    let bears_of = |p: PlayerId| {
        t.named_on_battlefield("Grizzly Bears")
            .into_iter()
            .filter(|x| t.obj(*x).controller == p)
            .map(|x| t.obj(x).is_token())
            .collect::<Vec<bool>>()
    };
    let mut mine = bears_of(P0);
    mine.sort();
    assert_eq!(mine, vec![false, true]);
    assert_eq!(bears_of(P1), vec![true]);
}

#[test]
fn only_spells_you_cast_with_that_quality_have_demonstrate() {
    cr!("702.144a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Silverquill Lecturer");
    // A noncreature spell doesn't get demonstrate.
    let a = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Murder");
    add_mana(&mut t, P0, ManaType::B, 3);
    t.cast(P0, spell).target(a).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Demonstrate"), 0);
    t.resolve_all();
    // An opponent's creature spell doesn't either.
    let bears = t.hand(P1, "Grizzly Bears");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    add_mana(&mut t, P1, ManaType::G, 2);
    t.cast(P1, bears).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Demonstrate"), 0);
}
