//! Rulings on open-ended "repeat this process" loops (CR 608.2c) and tokens whose
//! power/toughness is a value fixed as they're created (CR 111.3, 608.2h).

use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn yes_no_asked(t: &TestGame, p: PlayerId) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, Decision::YesNo { .. }))
        .count()
}

// --- Repeat this process ----------------------------------------------------------

#[test]
fn primal_surge_repeats_until_it_fails_and_triggers_wait() {
    ruling!(
        "Primal Surge",
        "Repeating the process includes the instruction to repeat the process"
    );
    ruling!(
        "Primal Surge",
        "those abilities will wait to go on the stack until Primal Surge has finished resolving"
    );
    supported("Primal Surge");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 10);
    let v2 = t.library_top(P0, "Elvish Visionary");
    let v1 = t.library_top(P0, "Elvish Visionary");
    let lib = t.library_size(P0);
    let hand = t.hand_size(P0);
    let spell = t.hand(P0, "Primal Surge");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.cast(P0, spell).go();
    t.g.resolve_top();
    // Both Visionaries entered; the third card (a non-permanent) ended the process. No
    // "draw a card" trigger went on the stack or resolved while Primal Surge resolved.
    assert!(t.on_battlefield(v1) && t.on_battlefield(v2));
    assert_eq!(t.library_size(P0), lib - 3);
    assert_eq!(t.stack_len(), 0);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn primal_surge_finishes_when_you_dont_put_the_card_onto_the_battlefield() {
    ruling!(
        "Primal Surge",
        "or one you don’t want to put onto the battlefield, Primal Surge finishes resolving"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 10);
    let giant = t.library_top(P0, "Hill Giant");
    let bears = t.library_top(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Primal Surge");
    t.answer_yes(P0, false);
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(giant), Zone::Library(P0));
}

#[test]
fn ad_nauseam_decides_after_each_card() {
    ruling!(
        "Ad Nauseam",
        "you decide whether to continue by revealing another card. You don't decide in advance"
    );
    supported("Ad Nauseam");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    t.library_top(P0, "Hill Giant");
    t.library_top(P0, "Grizzly Bears");
    let spell = t.hand(P0, "Ad Nauseam");
    t.answer_yes(P0, false);
    t.cast(P0, spell).go();
    t.resolve();
    // One card (Grizzly Bears, mana value 2), then a single choice not to continue.
    assert_eq!(t.life(P0), 18);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.in_hand(P0, "Hill Giant"));
    assert_eq!(yes_no_asked(&t, P0), 1);
}

#[test]
fn ad_nauseam_continues_at_zero_life_and_loses_when_it_stops() {
    ruling!(
        "Ad Nauseam",
        "even if your life total has been reduced to 0 or less"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    t.g.players[0].life = 3;
    t.library_top(P0, "Grizzly Bears");
    t.library_top(P0, "Hill Giant");
    let spell = t.hand(P0, "Ad Nauseam");
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.cast(P0, spell).go();
    t.g.resolve_top();
    // Hill Giant (4) took P0 to -1; they continued anyway, and Grizzly Bears (2) took
    // them to -3.
    assert_eq!(t.life(P0), -3);
    assert!(t.in_hand(P0, "Hill Giant") && t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.has_lost(P0));
    t.settle();
    assert!(t.has_lost(P0));
}

#[test]
fn cultivator_colossus_repeats_until_you_decline() {
    ruling!(
        "Cultivator Colossus",
        "The process is repeated until you decline to put a land card onto the battlefield."
    );
    ruling!(
        "Cultivator Colossus",
        "You repeat the process as part of Cultivator Colossus's triggered ability's resolution."
    );
    supported("Cultivator Colossus");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Steppe Lynx");
    t.hand(P0, "Forest");
    t.hand(P0, "Forest");
    t.hand(P0, "Forest");
    let colossus = t.hand(P0, "Cultivator Colossus");
    t.lands(P0, "Forest", 7);
    let library = t.library_size(P0);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.cast(P0, colossus).go();
    t.resolve(); // the Colossus
    t.g.resolve_top(); // its trigger
    // Two Forests put onto the battlefield tapped, a card drawn for each, and only then
    // the two landfall triggers.
    assert_eq!(t.library_size(P0), library - 2);
    assert_eq!(t.stack_len(), 0);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_eq!(yes_no_asked(&t, P0), 3);
}

// --- X/X tokens -----------------------------------------------------------------------

#[test]
fn gelatinous_genesis_x_three() {
    ruling!(
        "Gelatinous Genesis",
        "If X is 3, {6}{G} gets you three 3/3 Oozes"
    );
    supported("Gelatinous Genesis");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 7);
    let spell = t.hand(P0, "Gelatinous Genesis");
    t.cast(P0, spell).x(3).go();
    t.resolve_all();
    let oozes = t.named_on_battlefield("Ooze Token");
    assert_eq!(oozes.len(), 3);
    assert!(oozes.iter().all(|o| t.pt(*o) == (3, 3)));
}

#[test]
fn tumbleweed_rising_with_no_creatures_makes_a_0_0_token() {
    ruling!(
        "Tumbleweed Rising",
        "If you don’t control any creatures at that time, you’ll create a 0/0 Elemental token"
    );
    supported("Tumbleweed Rising");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 3);
    let spell = t.hand(P0, "Tumbleweed Rising");
    t.cast(P0, spell).go();
    t.g.resolve_top();
    let tok = t.named_on_battlefield("Elemental Token")[0];
    assert_eq!(t.pt(tok), (0, 0));
    t.settle();
    assert!(t.named_on_battlefield("Elemental Token").is_empty());
}

#[test]
fn formless_genesis_x_is_determined_once() {
    ruling!(
        "Formless Genesis",
        "The token's power and toughness won't change if the number of land cards in your graveyard later changes."
    );
    supported("Formless Genesis");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Forest");
    t.lands(P0, "Forest", 3);
    let spell = t.hand(P0, "Formless Genesis");
    t.cast(P0, spell).go();
    t.resolve_all();
    let tok = t.named_on_battlefield("Shapeshifter Token")[0];
    assert_eq!(t.pt(tok), (1, 1));
    t.graveyard(P0, "Island");
    t.graveyard(P0, "Swamp");
    assert_eq!(t.pt(tok), (1, 1));
}
