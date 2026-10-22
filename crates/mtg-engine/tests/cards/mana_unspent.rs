//! Unspent mana (CR 500.5, 703.4q): effects that keep it as steps and phases end ("You
//! don't lose unspent red mana as steps and phases end", "Players don't lose unspent
//! mana ...") and replacement effects that change it instead ("If you would lose unspent
//! mana, that mana becomes colorless instead", CR 614.1a).

use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let c = card(n);
        assert!(
            c.unsupported_text().is_empty(),
            "{n} has unsupported text: {:?}",
            c.unsupported_text()
        );
    }
}

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort();
    v
}

#[test]
fn unspent_mana_cards_compile() {
    assert_supported(&["Upwelling", "Horizon Stone", "Kruphix, God of Horizons"]);
    // The unspent-mana abilities of cards with other, unrelated unsupported text.
    for n in [
        "Electro, Assaulting Battery",
        "Leyline Tyrant",
        "Ashling, Flame Dancer",
        "Omnath, Locus of the Void",
        "Omnath, Locus of All",
        "Ozai, the Phoenix King",
        "Fangorn, Tree Shepherd",
        "The Last Agni Kai",
    ] {
        let bad: Vec<String> = card(n)
            .unsupported_text()
            .into_iter()
            .filter(|u| u.contains("lose unspent"))
            .map(|u| u.to_string())
            .collect();
        assert!(bad.is_empty(), "{n}: {bad:?}");
    }
}

#[test]
fn only_red_mana_is_kept() {
    cr!("500.5", "106.4");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline Tyrant");
    let mountain = t.battlefield(P0, "Mountain");
    let forest = t.battlefield(P0, "Forest");
    t.activate(P0, mountain, 0, &[]).unwrap();
    t.activate(P0, forest, 0, &[]).unwrap();
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    // Into the next turn, too.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    // The kept mana pays for a spell later.
    t.advance_to(P0, Step::PrecombatMain);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn players_dont_lose_unspent_mana() {
    cr!("500.5");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Upwelling");
    let forest = t.battlefield(P0, "Forest");
    t.activate(P0, forest, 0, &[]).unwrap();
    t.advance_to(P1, Step::Draw);
    assert_eq!(pool(&t, P0), vec![ManaType::G]);
}

#[test]
fn mana_is_lost_once_the_effect_is_gone() {
    cr!("500.5");
    let mut t = TestGame::new(2);
    let upwelling = t.battlefield(P0, "Upwelling");
    let forest = t.battlefield(P0, "Forest");
    t.activate(P0, forest, 0, &[]).unwrap();
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(pool(&t, P0), vec![ManaType::G]);
    t.g.destroy(upwelling, None);
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn unspent_mana_becomes_colorless_instead() {
    cr!("500.5", "614.1a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Horizon Stone");
    let forest = t.battlefield(P0, "Forest");
    t.activate(P0, forest, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::G]);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(pool(&t, P0), vec![ManaType::C]);
    // It's only this player's mana.
    let theirs = t.battlefield(P1, "Forest");
    t.activate(P1, theirs, 0, &[]).unwrap();
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(pool(&t, P1).is_empty());
    assert_eq!(pool(&t, P0), vec![ManaType::C]);
}

#[test]
fn until_end_of_turn_keep_red_mana() {
    cr!("500.5", "514.2");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    let agni = t.hand(P0, "The Last Agni Kai");
    t.cast(P0, agni).targets(&[mine.into(), theirs.into()]).go();
    t.resolve();
    let mountain = t.battlefield(P0, "Mountain");
    t.activate(P0, mountain, 0, &[]).unwrap();
    t.advance_to(P0, Step::End);
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    t.advance_to(P1, Step::Upkeep);
    assert!(pool(&t, P0).is_empty());
}
