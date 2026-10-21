//! The worked examples of CR 616 (interaction of replacement and/or prevention effects),
//! played out as tests. The CR 616.2 example is cited by a test in
//! `r609_616_interaction.rs`.

use crate::r600_examples_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The options of the first replacement-effect choice asked, and who was asked.
fn replacement_choice(t: &TestGame) -> (PlayerId, Vec<String>) {
    t.asked()
        .iter()
        .find_map(|(p, d)| match d {
            Decision::ChooseReplacement { options } => Some((*p, options.clone())),
            _ => None,
        })
        .expect("a replacement choice")
}

#[test]
fn the_destroyed_creatures_controller_picks_exile_or_shuffle_and_the_other_does_nothing() {
    cr!("616.1", "616.1e");
    cr_example!("616.1f", "instead shuffle it into its owner’s library");
    let mut outcomes = vec![];
    for pick in [0usize, 1] {
        let mut t = TestGame::new(2);
        // The enchantment is P1's; the creature is P0's, so P0 chooses.
        t.custom(
            P1,
            custom_costed(
                "Exile Field",
                "Enchantment",
                "{2}",
                "If a card would be put into a graveyard from anywhere, instead exile it.",
            ),
            Zone::Battlefield,
        );
        let c = t.custom(
            P0,
            custom_card(
                "Homing Beast",
                "Creature — Beast",
                Some((2, 2)),
                "If this creature would die, instead shuffle it into its owner's library.",
            ),
            Zone::Battlefield,
        );
        let lib = t.library_size(P0);
        t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
        t.lands(P1, "Swamp", 3);
        let murder = t.hand(P1, "Murder");
        t.cast(P1, murder).target(c).go();
        t.answer(P0, DecisionKind::Replacement, Answer::Index(pick));
        t.resolve();
        let (chooser, options) = replacement_choice(&t);
        assert_eq!(chooser, P0);
        assert_eq!(options.len(), 2);
        let exiled = options[pick].contains("Exile Field");
        // The one applied first did its thing; the other did nothing.
        assert_eq!(t.in_exile("Homing Beast"), exiled);
        assert_eq!(t.library_size(P0), if exiled { lib } else { lib + 1 });
        assert!(!t.in_graveyard(P0, "Homing Beast"));
        // Murder itself is exiled by the enchantment.
        assert!(t.in_exile("Murder"));
        outcomes.push(exiled);
    }
    outcomes.sort();
    assert_eq!(outcomes, vec![false, true]);
}

#[test]
fn rusted_sentinel_enters_as_an_untapped_copy_of_essence_of_the_wild() {
    cr!("616.1c", "614.1c");
    cr_example!("616.1f", "Essence of the Wild reads");
    supported("Essence of the Wild");
    supported("Rusted Sentinel");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Essence of the Wild");
    t.lands(P0, "Plains", 4);
    let s = t.hand(P0, "Rusted Sentinel");
    t.cast(P0, s).go();
    t.resolve();
    let essences = t.named_on_battlefield("Essence of the Wild");
    assert_eq!(essences.len(), 2);
    assert!(t.named_on_battlefield("Rusted Sentinel").is_empty());
    let copy = t.g.current(s);
    assert!(essences.contains(&copy));
    assert!(!t.obj_now(copy).tapped);
    assert_eq!(t.pt(copy), (6, 6));
}

#[test]
fn doubling_season_applies_before_each_voice_of_all_token_chooses_a_color() {
    cr!("616.1", "614.12");
    cr_example!("616.1g", "create a token that’s a copy of Voice of All");
    supported("Doubling Season");
    supported("Voice of All");
    supported("Cackling Counterpart");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    let voice = t.battlefield(P0, "Voice of All");
    t.lands(P0, "Island", 3);
    // Blue for one token, red for the other.
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    let cc = t.hand(P0, "Cackling Counterpart");
    t.cast(P0, cc).target(voice).go();
    t.resolve();
    let tokens: Vec<ObjectId> = t
        .named_on_battlefield("Voice of All")
        .into_iter()
        .filter(|v| t.obj_now(*v).is_token())
        .collect();
    assert_eq!(tokens.len(), 2);
    let mut colors: Vec<Option<Color>> =
        tokens.iter().map(|v| t.obj_now(*v).choices.color).collect();
    colors.sort();
    assert_eq!(colors, vec![Some(Color::Blue), Some(Color::Red)]);
}
