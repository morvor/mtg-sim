//! "For each time you've cast your commander from the command zone this game" (CR 903.8),
//! compiled by `oracle/patterns/misc_game_rules_commander.rs` and the value parsers: the
//! Commander Storm spells' "When you cast ~, copy it for each ...", anthems, and tokens.

use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

fn commander_game() -> TestGame {
    TestGame::with_config(
        2,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    )
}

/// Puts `name` into `p`'s command zone as one of their commanders.
fn commander(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.command(p, name);
    t.g.objects[id.0 as usize].is_commander = true;
    t.g.players[p.idx()].commander_names.push(name.into());
    id
}

/// Casts Isamaru ({W}) from the command zone, then sends it back there (its owner
/// choosing the command zone as it dies).
fn cast_and_return(t: &mut TestGame, cmdr: ObjectId, tax: usize) {
    let cmdr = t.g.current(cmdr);
    t.lands(P0, "Plains", 1 + tax);
    t.cast(P0, cmdr).go();
    t.resolve_all();
    let now = t.g.current(cmdr);
    assert!(t.on_battlefield(now));
    t.answer_yes(P0, true);
    t.g.destroy(now, None);
    t.settle();
    assert_eq!(t.zone(t.g.current(cmdr)), Zone::Command);
}

#[test]
fn copies_the_spell_for_each_time_the_commander_was_cast() {
    cr!("903.8", "707.10");
    compiles("Empyrial Storm");
    ruling!(
        "Empyrial Storm",
        "The copies are created on the stack, so they\u{2019}re not \u{201c}cast.\u{201d}"
    );
    let mut t = commander_game();
    let isamaru = commander(&mut t, P0, "Isamaru, Hound of Konda");
    cast_and_return(&mut t, isamaru, 0);
    cast_and_return(&mut t, isamaru, 2);
    t.lands(P0, "Plains", 6);
    let storm = t.hand(P0, "Empyrial Storm");
    t.cast(P0, storm).go();
    t.resolve_all();
    // The spell and two copies: three Angels. The copies weren't cast, so they don't
    // copy themselves.
    assert_eq!(t.named_on_battlefield("Angel Token").len(), 3);
}

#[test]
fn no_copies_without_commander_casts() {
    cr!("903.8", "707.10");
    let mut t = commander_game();
    commander(&mut t, P0, "Isamaru, Hound of Konda");
    t.lands(P0, "Plains", 6);
    let storm = t.hand(P0, "Empyrial Storm");
    t.cast(P0, storm).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Angel Token").len(), 1);
}

#[test]
fn creatures_get_plus_one_for_each_commander_cast_counting_both_commanders() {
    cr!("903.8", "702.124d");
    compiles("Commander's Insignia");
    ruling!(
        "Commander's Insignia",
        "Commander\u{2019}s Insignia counts the total number of times you\u{2019}ve cast each of your commanders"
    );
    let mut t = commander_game();
    let insignia = t.battlefield(P0, "Commander's Insignia");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let _ = insignia;
    t.settle();
    assert_eq!(t.pt(bears), (2, 2));
    // Kraum ({3}{U}{R}) and Tymna ({1}{W}{B}), partners.
    let kraum = commander(&mut t, P0, "Kraum, Ludevic's Opus");
    let tymna = commander(&mut t, P0, "Tymna the Weaver");
    t.lands(P0, "Island", 4);
    t.lands(P0, "Mountain", 1);
    t.cast(P0, kraum).go();
    t.resolve_all();
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Wastes", 1);
    t.cast(P0, tymna).go();
    t.resolve_all();
    t.settle();
    assert_eq!(t.pt(bears), (4, 4));
    // An opponent's commander casts don't count.
    let other = commander(&mut t, P1, "Isamaru, Hound of Konda");
    t.lands(P1, "Plains", 1);
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.cast(P1, other).go();
    t.resolve_all();
    t.settle();
    assert_eq!(t.pt(bears), (4, 4));
}

#[test]
fn creates_a_token_for_each_commander_cast() {
    cr!("903.8");
    compiles("Jyoti, Moag Ancient");
    let mut t = commander_game();
    let isamaru = commander(&mut t, P0, "Isamaru, Hound of Konda");
    cast_and_return(&mut t, isamaru, 0);
    cast_and_return(&mut t, isamaru, 2);
    t.enter(P0, "Jyoti, Moag Ancient");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Forest Dryad Token").len(), 2);
}

#[test]
fn a_double_faced_commander_is_counted_too() {
    cr!("903.8", "611.3a");
    ruling!(
        "Commander's Insignia",
        "counts each time you\u{2019}ve cast your commander, even the times it was countered or when your commander spell is still on the stack"
    );
    let mut t = commander_game();
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Commander's Insignia");
    // A modal double-faced commander, designated by its front face's name (as
    // `Game::designate_commander` does).
    let esika = commander(&mut t, P0, "Esika, God of the Tree");
    assert_eq!(t.g.obj(esika).chars.name, "Esika, God of the Tree");
    // {1}{G}{G}
    t.lands(P0, "Forest", 3);
    t.cast(P0, esika).go();
    // Counted as soon as it's cast, while the spell is still on the stack.
    t.settle();
    assert_eq!(t.pt(bears), (3, 3));
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn the_spell_is_copied_even_if_countered_before_the_trigger_resolves() {
    cr!("903.8", "707.10");
    ruling!(
        "Empyrial Storm",
        "can copy the Storm spell even if that spell is countered before that ability resolves"
    );
    let mut t = commander_game();
    let isamaru = commander(&mut t, P0, "Isamaru, Hound of Konda");
    cast_and_return(&mut t, isamaru, 0);
    t.lands(P0, "Plains", 6);
    let storm = t.hand(P0, "Empyrial Storm");
    let spell = t.cast(P0, storm).go();
    t.settle();
    // In response to the trigger, the spell itself is countered.
    assert!(t.g.counter(spell, None));
    assert!(t.in_graveyard(P0, "Empyrial Storm"));
    t.resolve_all();
    // Only the copy resolves.
    assert_eq!(t.named_on_battlefield("Angel Token").len(), 1);
}
