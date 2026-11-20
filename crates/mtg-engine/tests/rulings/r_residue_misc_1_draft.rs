//! Item `rulings-residue-misc-1`, pass B — Conspiracy draft rulings (batches P064, P074:
//! Animus of Predation, Noble Banneret; CR 905.2) and Crovax the Cursed's upkeep.

use crate::r_residue_misc_1_common::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::decision::Decision;
use mtg_engine::draft::{Draft, DraftError, DraftInfo, DraftOption, DraftStyle};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

/// A one-player draft of a single pack of the cards named.
fn draft_of(names: &[&str]) -> Draft {
    let pack: Vec<Arc<CardDef>> = names.iter().map(|n| card(n)).collect();
    Draft::new(vec![vec![pack]], DraftStyle::Conspiracy, 1)
}

fn index_of(d: &Draft, name: &str) -> usize {
    d.pack(P0).iter().position(|c| c.name == name).unwrap()
}

fn game_with(info: DraftInfo) -> TestGame {
    let mut t = TestGame::new(2);
    t.g.start.draft = info;
    t.set_step(P0, Step::PrecombatMain);
    t
}

const ANIMUS: &str = "Animus of Predation";

#[test]
fn animus_of_predation_removing_a_card_that_works_as_you_draft_it() {
    cr!("905.2", "905.2b", "905.2c");
    ruling!(
        "Animus of Predation",
        "If a card you remove requires you to reveal it as you draft it, do so and follow any additional instructions it may have related to drafting it. However, if a card you remove instructs you to draft it face up, it has no effect while face up and can't be turned face down to use an ability."
    );
    // Lore Seeker: "Reveal this card as you draft it. After you draft this card, you may
    // add a booster pack to the draft." Removed with Animus: revealed, and P0 may add a pack.
    let mut d = draft_of(&[ANIMUS, "Lore Seeker", "Grizzly Bears", "Shock"]);
    d.pick(P0, &[0]).unwrap();
    let with: smol_str::SmolStr = ANIMUS.into();
    let seeker = index_of(&d, "Lore Seeker");
    d.pick_with(
        P0,
        &[seeker],
        Some(DraftOption::Remove { with: with.clone() }),
    )
    .unwrap();
    assert!(d.drafted(P0).iter().all(|c| c.card.name != "Lore Seeker"));
    assert!(d.can_add_booster(P0));
    // Cogwork Grinder ("Draft this card face up. As you draft a card, you may remove it
    // from the draft face down."): removed, it can't remove anything.
    let mut d = draft_of(&[ANIMUS, "Cogwork Grinder", "Grizzly Bears", "Shock"]);
    d.pick(P0, &[0]).unwrap();
    let grinder = index_of(&d, "Cogwork Grinder");
    d.pick_with(P0, &[grinder], Some(DraftOption::Remove { with }))
        .unwrap();
    let bears = index_of(&d, "Grizzly Bears");
    assert_eq!(
        d.pick_with(
            P0,
            &[bears],
            Some(DraftOption::Remove {
                with: "Cogwork Grinder".into()
            })
        ),
        Err(DraftError::InvalidOption)
    );
}

#[test]
fn animus_of_predation_removed_cards_arent_in_the_card_pool() {
    cr!("905.2", "905.2b");
    ruling!(
        "Animus of Predation",
        "Cards you removed from the draft with Animus of Predation can't be exiled this way because they aren't in your card pool."
    );
    let mut d = draft_of(&[ANIMUS, "Serra Angel", "Grizzly Bears"]);
    d.pick(P0, &[0]).unwrap();
    let serra = index_of(&d, "Serra Angel");
    d.pick_with(
        P0,
        &[serra],
        Some(DraftOption::Remove {
            with: ANIMUS.into(),
        }),
    )
    .unwrap();
    d.pick(P0, &[0]).unwrap();
    let (pools, info) = d.finish();
    assert!(pools[0].iter().all(|c| c.name != "Serra Angel"));
    // Caller of the Untamed in the deck ("Before you shuffle your deck to start the game,
    // you may reveal this card from your deck and exile a creature card you drafted that
    // isn't in your deck."); the rest of the pool is the sideboard.
    let mut t = game_with(info);
    t.library_top(P0, "Caller of the Untamed");
    let side: Vec<Arc<CardDef>> = pools[0]
        .iter()
        .filter(|c| c.name != ANIMUS)
        .cloned()
        .collect();
    t.g.add_to_sideboard(P0, side);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    mtg_engine::opening_hand::before_shuffle_actions(&mut t.g);
    let offered: Vec<String> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .filter_map(|e| e.object())
        .map(|o| t.obj_now(o).chars.name.to_string())
        .collect();
    assert_eq!(offered, vec!["Grizzly Bears".to_string()]);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_exile("Serra Angel"));
}

#[test]
fn animus_of_predation_has_the_keywords_only_on_the_battlefield() {
    cr!("113.6", "905.2b");
    ruling!(
        "Animus of Predation",
        "The last ability of Animus of Predation functions only if Animus of Predation is on the battlefield. For example, Animus of Predation is not a creature card with flying in your library or graveyard."
    );
    let mut d = draft_of(&[ANIMUS, "Serra Angel"]);
    d.pick(P0, &[0]).unwrap();
    d.pick_with(
        P0,
        &[0],
        Some(DraftOption::Remove {
            with: ANIMUS.into(),
        }),
    )
    .unwrap();
    let (_, info) = d.finish();
    let mut t = game_with(info);
    let on = t.battlefield(P0, ANIMUS);
    let in_gy = t.graveyard(P0, ANIMUS);
    let in_lib = t.library_top(P0, ANIMUS);
    t.g.recompute();
    assert!(t.obj_now(on).chars.has_keyword(KeywordKind::Flying));
    assert!(!t.obj_now(in_gy).chars.has_keyword(KeywordKind::Flying));
    assert!(!t.obj_now(in_lib).chars.has_keyword(KeywordKind::Flying));
}

#[test]
fn noble_banneret_can_note_another_noble_banneret() {
    cr!("905.2", "905.2b", "905.2c");
    ruling!(
        "Noble Banneret",
        "The creature card you reveal during the draft can be another Noble Banneret. That second Noble Banneret will then be face up for a future creature card to be revealed. If Noble Banneret is one of the names you noted, a Noble Banneret that's the only creature you control will get +1/+1 and have lifelink."
    );
    let with: smol_str::SmolStr = "Noble Banneret".into();
    let mut d = draft_of(&["Noble Banneret", "Noble Banneret", "Grizzly Bears"]);
    d.pick(P0, &[0]).unwrap();
    // The second Banneret, revealed and noted with the first (which turns face down).
    d.pick_with(
        P0,
        &[0],
        Some(DraftOption::RevealAndNote { with: with.clone() }),
    )
    .unwrap();
    let up: Vec<bool> = d.drafted(P0).iter().map(|c| c.face_up).collect();
    assert_eq!(up, vec![false, true]);
    // The second is face up: it can note Grizzly Bears.
    d.pick_with(P0, &[0], Some(DraftOption::RevealAndNote { with }))
        .unwrap();
    let (_, info) = d.finish();
    let mut t = game_with(info);
    let banneret = t.battlefield(P0, "Noble Banneret");
    assert_eq!(t.pt(banneret), (4, 4));
    assert!(t.obj_now(banneret).chars.has_keyword(KeywordKind::Lifelink));
}

#[test]
fn crovax_the_cursed_declined_with_no_counters() {
    cr!("603.5", "122.8");
    ruling!(
        "Crovax the Cursed",
        "If you choose not to sacrifice a creature and he has no +1/+1 counters, then nothing bad happens."
    );
    let mut t = TestGame::new(2);
    let crovax = t.battlefield(P0, "Crovax the Cursed");
    // Glorious Anthem keeps the 0/0 Crovax alive.
    t.battlefield(P0, "Glorious Anthem");
    t.battlefield(P0, "Grizzly Bears");
    assert_eq!(p1(&t, crovax), 0);
    t.set_step(P0, Step::Untap);
    t.answer_yes(P0, false);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(crovax));
    assert_eq!(p1(&t, crovax), 0);
    assert_eq!(t.pt(crovax), (1, 1));
    assert_eq!(creatures(&t, P0).len(), 2);
}

#[test]
fn crovax_the_cursed_one_sacrifice_each_turn() {
    cr!("603.2", "701.21a");
    ruling!(
        "Crovax the Cursed",
        "You can only sacrifice one creature to it each turn."
    );
    let mut t = TestGame::new(2);
    let crovax = t.enter(P0, "Crovax the Cursed");
    t.resolve_all();
    assert_eq!(p1(&t, crovax), 4);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.set_step(P0, Step::Untap);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a)]);
    let from = t.asked().len();
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(p1(&t, crovax), 5);
    // One creature is chosen to sacrifice, and the ability triggers once.
    let maxes: Vec<u32> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseEntities { max, .. } => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(maxes, vec![1]);
    assert_eq!(
        [a, b].iter().filter(|x| t.on_battlefield(**x)).count(),
        1,
        "one creature sacrificed"
    );
    // No other way to sacrifice to it this turn: no activated ability does.
    assert!(t
        .obj_now(crovax)
        .chars
        .abilities
        .iter()
        .filter(|ab| matches!(ab.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .all(|ab| !ab.text.contains("acrifice")));
}
