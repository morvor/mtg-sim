//! Quoted abilities of tokens that name the card creating them (`src/granted_by.rs`, gap
//! R3): "create a green Ooze creature token with \"This token's power and toughness are
//! each equal to the number of slime counters on Gutter Grime.\"" refers to the specific
//! object that created the token (CR 201.5a), as part of the token's copiable values
//! (CR 111.3, 707.2); "a card named Rekindling Phoenix" is a name (CR 201.2).

use mtg_engine::testing::*;
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn kill(t: &mut TestGame, id: ObjectId) {
    t.g.destroy(t.g.current(id), None);
    t.g.flush_events();
    t.settle();
}

#[test]
fn gutter_grime_each_ooze_counts_the_grime_that_created_it() {
    cr!("201.5a", "111.3", "604.3a", "707.2");
    ruling!("Gutter Grime", "each Ooze token remembers which one created it");
    ruling!(
        "Gutter Grime",
        "the power and toughness of each Ooze token it created will become 0"
    );
    ruling!(
        "Gutter Grime",
        "The power and toughness of the Ooze tokens will constantly update"
    );
    assert_supported(&["Gutter Grime", "Saproling Burst"]);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Gutter Grime");
    let b = t.battlefield(P0, "Gutter Grime");
    t.g.add_counters(Entity::Object(a), "slime", 2, None);
    let bears = t.battlefield(P0, "Grizzly Bears");
    kill(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.counters(a, "slime"), 3);
    assert_eq!(t.counters(b, "slime"), 1);
    let mut oozes = t.named_on_battlefield("Ooze Token");
    assert_eq!(oozes.len(), 2);
    oozes.sort_by_key(|o| t.pt(*o).0);
    let (small, big) = (oozes[0], oozes[1]);
    assert_eq!(t.pt(small), (1, 1));
    assert_eq!(t.pt(big), (3, 3));
    // More counters on one Grime: only its Ooze grows.
    t.g.add_counters(Entity::Object(b), "slime", 2, None);
    t.g.recompute();
    assert_eq!(t.pt(small), (3, 3));
    assert_eq!(t.pt(big), (3, 3));
    // A copy of an Ooze refers to the same Grime (Cackling Counterpart: "Create a token
    // that's a copy of target creature you control.").
    t.lands(P0, "Island", 3);
    let cc = t.hand(P0, "Cackling Counterpart");
    t.cast(P0, cc).target(small).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ooze Token").len(), 3);
    // The Grime that created the bigger Ooze leaves: that Ooze is 0/0 and dies.
    let grime_of_big = a;
    kill(&mut t, grime_of_big);
    assert!(!t.on_battlefield(big));
    let left = t.named_on_battlefield("Ooze Token");
    assert_eq!(left.len(), 2);
    for o in left {
        assert_eq!(t.pt(o), (3, 3));
    }
}

#[test]
fn saproling_burst_tokens_count_their_own_burst() {
    cr!("201.5a", "604.3a");
    ruling!(
        "Saproling Burst",
        "Each token knows which Saproling Burst it came from."
    );
    // Fading 7: two Bursts with seven fade counters each; one makes two Saprolings.
    let mut t = TestGame::new(2);
    let a = t.enter(P0, "Saproling Burst");
    let b = t.enter(P0, "Saproling Burst");
    assert_eq!(t.counters(a, "fade"), 7);
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    t.activate(P0, b, 0, &[]).unwrap();
    t.resolve_all();
    let mut saps: Vec<(i32, i32)> = t
        .named_on_battlefield("Saproling Token")
        .into_iter()
        .map(|s| t.pt(s))
        .collect();
    saps.sort();
    assert_eq!(saps, vec![(5, 5), (5, 5), (6, 6)]);
}

#[test]
fn rekindling_phoenix_returns_a_card_named_rekindling_phoenix() {
    cr!("201.2", "603.3d");
    ruling!(
        "Rekindling Phoenix",
        "If you don't have a card named Rekindling Phoenix in your graveyard"
    );
    assert_supported(&["Rekindling Phoenix", "Deathpact Angel", "Ozox, the Clattering King"]);
    let mut t = TestGame::new(2);
    let phoenix = t.battlefield(P0, "Rekindling Phoenix");
    kill(&mut t, phoenix);
    t.resolve_all();
    let tokens = t.named_on_battlefield("Elemental Token");
    assert_eq!(tokens.len(), 1);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    assert!(!t.on_battlefield(tokens[0]));
    let back = t.named_on_battlefield("Rekindling Phoenix");
    assert_eq!(back.len(), 1);
    assert!(t
        .obj_now(back[0])
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    // No Phoenix in the graveyard (exiled meanwhile): the token stays.
    let mut t = TestGame::new(2);
    let phoenix = t.battlefield(P0, "Rekindling Phoenix");
    kill(&mut t, phoenix);
    t.resolve_all();
    let token = t.named_on_battlefield("Elemental Token")[0];
    let card = t.g.current(phoenix);
    t.g.exile_object(card, None);
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(token));
}

#[test]
fn ozox_jumblebones_returns_ozox() {
    cr!("201.2");
    let mut t = TestGame::new(2);
    let ozox = t.battlefield(P0, "Ozox, the Clattering King");
    kill(&mut t, ozox);
    t.resolve_all();
    let jb = t.named_on_battlefield("Jumblebones");
    assert_eq!(jb.len(), 1);
    kill(&mut t, jb[0]);
    t.resolve_all();
    assert!(t.in_hand(P0, "Ozox, the Clattering King"));
}

#[test]
fn deathpact_angel_cleric_returns_a_card_named_deathpact_angel() {
    cr!("201.2");
    ruling!(
        "Deathpact Angel",
        "doesn't target any card in your graveyard"
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Deathpact Angel");
    kill(&mut t, angel);
    t.resolve_all();
    let cleric = t.named_on_battlefield("Cleric Token")[0];
    // (Not summoning sick by P0's next turn.)
    t.advance_to(P0, mtg_engine::turn::Step::Upkeep);
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Swamp", 4);
    t.activate(P0, cleric, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(cleric));
    assert_eq!(t.named_on_battlefield("Deathpact Angel").len(), 1);
}
