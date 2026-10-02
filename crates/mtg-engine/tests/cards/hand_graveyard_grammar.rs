//! Card actions on hands, graveyards and libraries (grammar in
//! `src/oracle/patterns/hand_graveyard_grammar.rs`): exiling cards from graveyards with
//! counts and sources, discard/draw variants, revealing from a hand, moving cards between
//! a hand and a library, "A or B" instructions, and statics that grant abilities to cards
//! in a graveyard or hand.

use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

/// The ability of `name` containing `piece` compiles (other abilities may not).
fn assert_compiles(cards: &[(&str, &str)]) {
    for (n, piece) in cards {
        let c = card(n);
        let all: String = c.faces.iter().map(|f| f.chars.rules_text.to_string()).collect();
        assert!(
            all.to_lowercase().contains(&piece.to_lowercase()),
            "{n} has no text {piece:?}"
        );
        for u in c.unsupported_text() {
            assert!(
                !u.to_lowercase().contains(&piece.to_lowercase()),
                "{n}: {piece:?} is unsupported: {u}"
            );
        }
    }
}

/// The candidates of the most recent "choose entities" decision asked of `p`.
fn last_choice_of(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseEntities { candidates, .. } if q == p => Some(candidates),
            _ => None,
        })
        .expect("no choice was asked")
}

fn objs(ids: &[ObjectId]) -> Vec<Entity> {
    ids.iter().map(|o| Entity::Object(*o)).collect()
}

// ---------------------------------------------------------------------------
// Exiling cards from graveyards
// ---------------------------------------------------------------------------

#[test]
fn exile_family_texts_compile() {
    assert_compiles(&[
        ("Kaya, Orzhov Usurper", "Exile up to two target cards from a single graveyard"),
        ("Ashiok, Nightmare Weaver", "Exile all cards from all opponents' hands and graveyards"),
        ("Summon: Esper Valigarmanda", "Exile an instant or sorcery card from each graveyard"),
    ]);
}

#[test]
fn exile_family_compiles() {
    assert_supported(&[
        "Decompose",
        "Griffnaut Tracker",
        "Faerie Macabre",
        "Worldfire",
        "Identity Crisis",
        "Thraben Charm",
        "Titania's Command",
        "Shred Memory",
        "Rats' Feast",
        "Pestilent Cauldron // Restorative Burst",
        "Decree of Annihilation",
        "Ultimate Nullification",
        "Thought Distortion",
        "Aegis Sculptor",
        "Bloodcurdler",
    ]);
}

#[test]
fn decompose_targets_up_to_three_cards_in_a_single_graveyard() {
    cr!("115.1", "115.3");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Lightning Bolt");
    let c = t.graveyard(P1, "Shock");
    let mine = t.graveyard(P0, "Forest");
    // Cards from two graveyards can't be chosen together (the choice is fitted to a
    // single graveyard).
    let mut t2 = TestGame::new(2);
    t2.lands(P0, "Swamp", 2);
    let x = t2.graveyard(P1, "Grizzly Bears");
    let y = t2.graveyard(P0, "Forest");
    let d = t2.hand(P0, "Decompose");
    t2.cast(P0, d).targets(&objs(&[x, y])).go();
    t2.resolve();
    assert!(t2.zone(x) != Zone::Exile || t2.zone(y) != Zone::Exile);
    let d = t.hand(P0, "Decompose");
    t.cast(P0, d).targets(&objs(&[a, b, c])).go();
    t.resolve();
    for x in [a, b, c] {
        assert_eq!(t.zone(x), Zone::Exile);
    }
    assert_eq!(t.zone(mine), Zone::Graveyard(P0));
}

#[test]
fn faerie_macabre_exiles_from_different_graveyards() {
    cr!("115.1");
    let mut t = TestGame::new(2);
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P0, "Forest");
    let fm = t.hand(P0, "Faerie Macabre");
    t.answer_targets(P0, &objs(&[a, b]));
    t.activate(P0, fm, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    assert!(t.in_graveyard(P0, "Faerie Macabre"));
}

#[test]
fn worldfire_exiles_every_hand_and_graveyard() {
    cr!("406.1");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 9);
    t.hand(P0, "Shock");
    t.hand(P1, "Grizzly Bears");
    t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P0, "Forest");
    let wf = t.hand(P0, "Worldfire");
    t.cast(P0, wf).go();
    t.resolve();
    assert_eq!(t.hand_size(P0), 0);
    assert_eq!(t.hand_size(P1), 0);
    // Worldfire itself goes to the graveyard after it resolves.
    assert_eq!(t.graveyard_size(P1), 0);
    assert!(t.in_exile("Lightning Bolt") && t.in_exile("Forest") && t.in_exile("Shock"));
    assert_eq!(t.life(P0), 1);
    assert_eq!(t.life(P1), 1);
}

#[test]
fn identity_crisis_exiles_a_hand_and_graveyard() {
    cr!("406.1");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Swamp", 4);
    t.hand(P1, "Grizzly Bears");
    t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P0, "Forest");
    let ic = t.hand(P0, "Identity Crisis");
    t.cast(P0, ic).target(P1).go();
    t.resolve();
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.graveyard_size(P1), 0);
    assert!(t.in_graveyard(P0, "Forest"));
}

#[test]
fn titanias_command_life_for_each_card_exiled_this_way() {
    cr!("406.1");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 6);
    for _ in 0..3 {
        t.graveyard(P1, "Grizzly Bears");
    }
    let tc = t.hand(P0, "Titania's Command");
    t.cast(P0, tc).modes(&[0, 2]).target(P1).go();
    t.resolve();
    assert_eq!(t.graveyard_size(P1), 0);
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.named_on_battlefield("Bear Token").len(), 2, "{}", t.dump_log());
}

#[test]
fn aegis_sculptor_may_exile_two_cards_only_with_two() {
    cr!("603.5");
    let text = card("Aegis Sculptor").faces[0].chars.rules_text.to_string();
    assert!(text.contains("you may exile two cards from your graveyard"));
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Aegis Sculptor");
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Shock");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &objs(&[a, b]));
    t.advance_to(P1, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    assert_eq!(t.counters(s, "+1/+1"), 1);
}
