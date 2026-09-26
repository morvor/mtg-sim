//! "If that spell is countered this way, [put it somewhere] instead of [into its owner's
//! graveyard]" — self-replacement effects of counterspells (CR 614.15, 701.6a; patterns in
//! `src/oracle/patterns/replacements_counter.rs`).

use mtg_engine::events::Event;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_compiles(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

#[test]
fn counter_elsewhere_cards_compile() {
    assert_compiles(&[
        "Dissipate",
        "Assert Authority",
        "Memory Lapse",
        "Lapse of Certainty",
        "Remand",
        "Spell Crumple",
        "Syncopate",
        "Spell Shrivel",
        "Liquify",
        "Deny Existence",
        "Faerie Trickery",
        "Horribly Awry",
        "No Escape",
        "Reject",
        "Deny the Divine",
        "Defabricate",
    ]);
}

/// P0 casts Grizzly Bears; P1 answers with `counter` targeting it and everything resolves.
fn counter_bears(counter: &str, p1_lands: usize) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P1, "Island", p1_lands);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    let c = t.hand(P1, counter);
    t.cast(P1, c).target(spell).go();
    t.resolve_all();
    (t, bears)
}

#[test]
fn dissipate_exiles_the_countered_spell() {
    cr!("614.15", "701.6a");
    ruling!(
        "Dissipate",
        "The card does not go to the graveyard before being exiled."
    );
    let (t, _) = counter_bears("Dissipate", 3);
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    // The spell moved from the stack straight to exile: no zone change put it into a
    // graveyard on the way.
    let moves: Vec<(Zone, Zone)> = t
        .g
        .turn_events
        .iter()
        .filter_map(|e| match e {
            Event::ZoneChange { new, from, to, .. }
                if t.g.obj(*new).chars.name == "Grizzly Bears" =>
            {
                Some((*from, *to))
            }
            _ => None,
        })
        .collect();
    assert!(
        moves.contains(&(Zone::Stack, Zone::Exile)),
        "moves: {moves:?}"
    );
    assert!(
        !moves.iter().any(|(_, to)| matches!(to, Zone::Graveyard(_))),
        "moves: {moves:?}"
    );
    // Dissipate itself goes to the graveyard as usual.
    assert!(t.in_graveyard(P1, "Dissipate"));
}

#[test]
fn dissipate_doesnt_exile_a_spell_that_cant_be_countered() {
    cr!("614.15", "701.6a");
    ruling!(
        "Dissipate",
        "If the spell is not countered (because the spell it targets can't be countered), then it does not get exiled."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P1, "Island", 3);
    let victim = t.battlefield(P1, "Ornithopter");
    let decay = t.hand(P0, "Abrupt Decay");
    let spell = t.cast(P0, decay).target(victim).go();
    let d = t.hand(P1, "Dissipate");
    t.cast(P1, d).target(spell).go();
    t.resolve_all();
    // Abrupt Decay resolved and went to its owner's graveyard, not to exile.
    assert!(!t.on_battlefield(victim));
    assert!(t.in_graveyard(P0, "Abrupt Decay"));
    assert!(!t.in_exile("Abrupt Decay"));
}

#[test]
fn memory_lapse_puts_the_spell_on_top_of_its_owners_library() {
    cr!("614.15", "701.6a");
    let (t, _) = counter_bears("Memory Lapse", 2);
    let top = t.g.library_top(P0).expect("P0's library is empty");
    assert_eq!(t.g.obj(top).chars.name, "Grizzly Bears");
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn remand_returns_the_spell_to_its_owners_hand_and_draws() {
    cr!("614.15", "701.6a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P1, "Island", 2);
    t.library_top(P1, "Island");
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    let r = t.hand(P1, "Remand");
    let h1 = t.hand_size(P1);
    t.cast(P1, r).target(spell).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    // Remand left P1's hand and P1 drew a card.
    assert_eq!(t.hand_size(P1), h1);
}

#[test]
fn remand_on_a_spell_that_cant_be_countered_still_draws() {
    cr!("614.15");
    ruling!(
        "Remand",
        "Remand can target a spell that can't be countered. That spell won't be countered or returned to its owner's hand, but you'll draw a card."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P1, "Island", 2);
    t.library_top(P1, "Island");
    let victim = t.battlefield(P1, "Ornithopter");
    let decay = t.hand(P0, "Abrupt Decay");
    let spell = t.cast(P0, decay).target(victim).go();
    let r = t.hand(P1, "Remand");
    let h1 = t.hand_size(P1);
    t.cast(P1, r).target(spell).go();
    t.resolve_all();
    assert!(!t.in_hand(P0, "Abrupt Decay"));
    assert!(t.in_graveyard(P0, "Abrupt Decay"));
    assert_eq!(t.hand_size(P1), h1);
}

#[test]
fn spell_crumple_puts_both_spells_on_the_bottom() {
    cr!("614.15", "701.6a");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Forest");
    t.library_top(P1, "Island");
    t.lands(P0, "Forest", 2);
    t.lands(P1, "Island", 3);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    let c = t.hand(P1, "Spell Crumple");
    t.cast(P1, c).target(spell).go();
    t.resolve_all();
    // The bottom of a library is its first card.
    let bottom = |t: &TestGame, p: PlayerId| {
        let lib = &t.g.player(p).library;
        t.g.obj(lib[0]).chars.name.to_string()
    };
    assert_eq!(bottom(&t, P0), "Grizzly Bears");
    assert_eq!(bottom(&t, P1), "Spell Crumple");
    assert_eq!(t.g.obj(t.g.library_top(P0).unwrap()).chars.name, "Forest");
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn spell_crumple_on_a_spell_that_cant_be_countered_still_goes_to_the_bottom() {
    cr!("614.15", "701.6a");
    ruling!(
        "Spell Crumple",
        "If the targeted spell can’t be countered, it won’t be put onto the bottom of its owner’s library. Spell Crumple will still be put on the bottom of its owner’s library."
    );
    let mut t = TestGame::new(2);
    t.library_top(P0, "Forest");
    t.library_top(P1, "Island");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P1, "Island", 3);
    let victim = t.battlefield(P1, "Ornithopter");
    let decay = t.hand(P0, "Abrupt Decay");
    let spell = t.cast(P0, decay).target(victim).go();
    let c = t.hand(P1, "Spell Crumple");
    t.cast(P1, c).target(spell).go();
    t.resolve_all();
    // Abrupt Decay resolved and went to its owner's graveyard, not to the library.
    assert!(!t.on_battlefield(victim));
    assert!(t.in_graveyard(P0, "Abrupt Decay"));
    let lib0 = &t.g.player(P0).library;
    assert!(lib0.iter().all(|o| t.g.obj(*o).chars.name != "Abrupt Decay"));
    // Spell Crumple is on the bottom of P1's library all the same.
    let lib1 = &t.g.player(P1).library;
    assert_eq!(t.g.obj(lib1[0]).chars.name, "Spell Crumple");
    assert!(!t.in_graveyard(P1, "Spell Crumple"));
}

#[test]
fn syncopate_exiles_only_if_the_controller_doesnt_pay() {
    cr!("614.15", "701.6a");
    // P0 can't pay {1}: exiled.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P1, "Island", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    let s = t.hand(P1, "Syncopate");
    t.cast(P1, s).target(spell).x(1).go();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // P0 pays: the spell resolves.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    t.lands(P1, "Island", 2);
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, bears).go();
    let s = t.hand(P1, "Syncopate");
    t.cast(P1, s).target(spell).x(1).go();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(!t.in_exile("Grizzly Bears"));
}
