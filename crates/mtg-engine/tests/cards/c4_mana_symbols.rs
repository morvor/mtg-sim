//! Mana symbols as values, round 4 (cards4-costs-mana-replacements,
//! `oracle/patterns/mana_5.rs`): one mana for each colored mana symbol in a card's mana
//! cost, and an instruction repeated for each symbol of a color among revealed cards.

use mtg_engine::ability::AbilityKind;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort_by_key(|m| format!("{m:?}"));
    v
}

fn activate_first(t: &mut TestGame, p: PlayerId, src: ObjectId) {
    t.g.recompute();
    let uid =
        t.g.obj(src)
            .chars
            .abilities
            .iter()
            .find(|a| matches!(a.kind, AbilityKind::Activated(_)))
            .map(|a| a.uid)
            .expect("an activated ability");
    t.g.turn.priority = Some(p);
    t.g.activate_ability(p, src, uid).expect("activation");
    t.g.flush_events();
}

#[test]
fn charmed_pendant_adds_one_mana_for_each_colored_symbol_of_the_milled_card() {
    cr!("701.17a", "605.1a", "107.4e");
    assert_supported(&["Charmed Pendant"]);
    // Recoil {1}{U}{B}: {U}{B}.
    let mut t = TestGame::new(2);
    let pendant = t.battlefield(P0, "Charmed Pendant");
    t.library_top(P0, "Recoil");
    activate_first(&mut t, P0, pendant);
    // Milling is a library action: not a mana ability, it uses the stack (CR 605.1a).
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert!(t.in_graveyard(P0, "Recoil"));
    let mut want = vec![ManaType::U, ManaType::B];
    want.sort_by_key(|m| format!("{m:?}"));
    assert_eq!(pool(&t, P0), want);
}

#[test]
fn charmed_pendant_gives_one_mana_of_either_color_for_a_hybrid_symbol() {
    cr!("107.4e");
    ruling!(
        "Charmed Pendant",
        "you get one of either color mana, not both"
    );
    // Kitchen Finks: {1}{G/W}{G/W}: two mana, each green or white; nothing for {1}.
    let mut t = TestGame::new(2);
    let pendant = t.battlefield(P0, "Charmed Pendant");
    t.library_top(P0, "Kitchen Finks");
    activate_first(&mut t, P0, pendant);
    t.resolve();
    let got = pool(&t, P0);
    assert_eq!(got.len(), 2);
    assert!(got.iter().all(|m| matches!(m, ManaType::G | ManaType::W)));
}

#[test]
fn sanity_grinding_mills_a_card_for_each_blue_symbol_among_the_revealed_cards() {
    cr!("701.17a", "107.4e");
    ruling!(
        "Sanity Grinding",
        "Chroma abilities count hybrid mana symbols"
    );
    assert_supported(&["Sanity Grinding"]);
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    // Counterspell {U}{U}, Azorius Guildmage {W/U}{W/U} (hybrid symbols count), Lightning
    // Bolt: four blue symbols.
    t.library_top(P0, "Counterspell");
    t.library_top(P0, "Azorius Guildmage");
    t.library_top(P0, "Lightning Bolt");
    let lib0 = t.library_size(P0);
    let lib1 = t.library_size(P1);
    let spell = t.hand(P0, "Sanity Grinding");
    t.cast(P0, spell).target(P1).go();
    t.resolve_all();
    assert_eq!(t.library_size(P1), lib1 - 4);
    assert_eq!(t.graveyard_size(P1), 4);
    // The revealed cards go to the bottom of P0's library: none of them is on top.
    assert_eq!(t.library_size(P0), lib0);
    let top = t.g.library_top(P0).expect("a card");
    assert_eq!(t.g.obj(top).chars.name, "Filler");
}
