//! Rulings batch P076 — casting a split card's half by "a spell with [characteristics]"
//! (CR 601.3e, 709.3), storm copies of an Aura becoming tokens (CR 111.13, 608.3f), and
//! green mana kept by Fangorn, Tree Shepherd (CR 500.5, 106.4, 106.6).

use crate::r_p076_common::*;
use crate::r_s01_common::{stack_library, supported};
use crate::r_s22_common::{attack_p1_unblocked, choose_named_when_offered};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const THERE: &str = "There // They're // Their";

#[test]
fn there_theyre_their_halves_with_mana_value_2_or_less_can_be_cast() {
    cr!("601.3e", "709.3a", "608.2g");
    ruling!(
        "There // They're // Their",
        "If an effect allows you to cast a spell with certain characteristics, consider only the characteristics of the spell you’re casting. For example, if an effect allows you to cast an instant or sorcery spell with mana value 2 or less from among cards in your graveyard, you could cast There or They’re this way, but There is right out."
    );
    supported("Sword of Once and Future");
    supported(THERE);
    // Sword of Once and Future: "... Then you may cast an instant or sorcery spell with
    // mana value 2 or less from your graveyard without paying its mana cost." The card has
    // mana value 6 in the graveyard (CR 709.4), but its halves There ({U}) and They're
    // ({1}{U}) have mana value 1 and 2; Their ({2}{U}) has 3.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Sword of Once and Future", bears);
    let lib = stack_library(&mut t, P0, &["Forest", "Forest"]);
    t.answer(P0, DecisionKind::Surveil, Answer::Split(lib, vec![]));
    t.graveyard(P0, THERE);
    choose_named_when_offered(&mut t, P0, THERE);
    // They're: "Up to three target creatures can't be blocked this turn."
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.answer_targets(P0, &[Entity::Object(bears)]);
    attack_p1_unblocked(&mut t, bears);
    let offered: Vec<Vec<String>> = t
        .asked()
        .into_iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } if options.iter().any(|o| o.contains("There")) => {
                Some(options)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        offered,
        vec![vec!["Cast There".to_string(), "Cast They're".to_string()]],
        "There and They're can be cast; Their can't"
    );
    // They're was cast (and exiled by the Sword's replacement effect).
    assert!(t.in_exile(THERE));
    let cast_theyre = t
        .g
        .history
        .spells_cast
        .iter()
        .any(|(_, s)| t.g.obj(*s).chars.name == "They're");
    assert!(cast_theyre);
}

#[test]
fn a_card_whose_halves_all_have_too_high_a_mana_value_cant_be_chosen() {
    cr!("601.3e");
    supported("Sword of Once and Future");
    // Fire // Ice: both halves have mana value 2 — castable. Commit // Memory: Commit {3}{U}
    // and Memory {4}{U}{U}: neither half qualifies.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s06_common::attach_new(&mut t, P0, "Sword of Once and Future", bears);
    let lib = stack_library(&mut t, P0, &["Forest", "Forest"]);
    t.answer(P0, DecisionKind::Surveil, Answer::Split(lib, vec![]));
    t.graveyard(P0, "Commit // Memory");
    choose_named_when_offered(&mut t, P0, "Commit // Memory");
    attack_p1_unblocked(&mut t, bears);
    assert!(t.in_graveyard(P0, "Commit // Memory"));
}

// ---------------------------------------------------------------------------------------
// Krosan Adaptation

#[test]
fn krosan_adaptation_storm_copy_token_is_not_created() {
    // Krosan Adaptation's 2019 ruling ("the token copies ... are considered to have been
    // “created” and those effects will apply") is superseded by CR 111.13 and 608.3f: a
    // copy of a permanent spell becomes a token as it resolves, and that token isn't
    // "created" for replacement effects such as Parallel Lives.
    cr!("111.13", "608.3f", "702.40a");
    supported("Krosan Adaptation");
    supported("Parallel Lives");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Parallel Lives");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // One spell cast before it: one storm copy.
    crate::r_s25_common::cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    crate::r_s25_common::cast_new(&mut t, P0, "Krosan Adaptation", &[Entity::Object(bears)]);
    t.resolve_all();
    let auras: Vec<&mtg_engine::object::GameObject> = t
        .g
        .permanents()
        .filter(|o| o.chars.name == "Krosan Adaptation")
        .collect();
    assert_eq!(auras.len(), 2, "the card and one token, not doubled");
    assert_eq!(auras.iter().filter(|o| o.is_token()).count(), 1);
    assert_eq!(t.pt(bears), (4, 2));
}

// ---------------------------------------------------------------------------------------
// Fangorn, Tree Shepherd

/// Fangorn's static ability ("You don't lose unspent green mana as steps and phases
/// end.") compiles; its mana trigger doesn't yet, and these rulings are about the static.
fn fangorn_static_supported() {
    let c = mtg_engine::card::card("Fangorn, Tree Shepherd");
    assert_eq!(
        c.unsupported_text(),
        vec!["Whenever one or more Treefolk you control attack, add twice that much {G}."]
    );
}

fn pool_count(t: &TestGame, ty: ManaType) -> usize {
    t.g.player(P0).mana_pool.count(ty)
}

#[test]
fn fangorn_green_mana_is_lost_after_it_leaves_at_the_end_of_the_step() {
    cr!("500.5", "106.4");
    ruling!(
        "Fangorn, Tree Shepherd",
        "Once Fangorn, Tree Shepherd leaves the battlefield, you have until the end of the current step or phase to spend your unspent green mana before it is lost as normal. There is no penalty associated with this other than the loss of the mana."
    );
    fangorn_static_supported();
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::Upkeep);
    let fangorn = t.battlefield(P0, "Fangorn, Tree Shepherd");
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::R, 1);
    t.advance_to(P0, Step::Draw);
    // The green mana stays; the red is lost.
    assert_eq!(pool_count(&t, ManaType::G), 2);
    assert_eq!(pool_count(&t, ManaType::R), 0);
    crate::r_s02_common::destroy(&mut t, fangorn);
    // Still there until the step ends ...
    assert_eq!(pool_count(&t, ManaType::G), 2);
    t.advance_to(P0, Step::PrecombatMain);
    // ... then lost, with no other penalty.
    assert_eq!(pool_count(&t, ManaType::G), 0);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn fangorn_kept_green_mana_keeps_its_restrictions() {
    cr!("106.6", "500.5");
    ruling!(
        "Fangorn, Tree Shepherd",
        "If a green mana you add has certain restrictions or riders associated with it (for example, if it was produced by Great Hall of the Citadel), they'll apply to that mana no matter when you spend it."
    );
    fangorn_static_supported();
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::Upkeep);
    t.battlefield(P0, "Fangorn, Tree Shepherd");
    // Pillar of the Paruns: "{T}: Add one mana of any color. Spend this mana only to cast
    // a multicolored spell." Green.
    let pillar = t.battlefield(P0, "Pillar of the Paruns");
    t.answer(P0, DecisionKind::Option, Answer::Index(4));
    t.activate(P0, pillar, 0, &[]).unwrap();
    assert_eq!(pool_count(&t, ManaType::G), 1);
    t.advance_to(P0, Step::PrecombatMain);
    assert_eq!(pool_count(&t, ManaType::G), 1);
    // It still can't pay for a monocolored spell ...
    let growth = t.hand(P0, "Giant Growth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(t.cast(P0, growth).target(bears).try_go().is_err());
    assert_eq!(pool_count(&t, ManaType::G), 1);
    // ... but pays the green part of a multicolored one.
    t.battlefield(P0, "Island");
    t.library_top(P0, "Forest");
    let spiral = t.hand(P0, "Growth Spiral");
    t.cast(P0, spiral).go();
    assert_eq!(pool_count(&t, ManaType::G), 0);
}
