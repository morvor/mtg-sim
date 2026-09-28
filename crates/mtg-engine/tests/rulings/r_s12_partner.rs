//! Rulings batch S12 — partner (CR 702.124): partner abilities let a player have two
//! commanders (Commander, CR 903; Commander Draft, CR 903.13).

use crate::r_s01_common::*;
use crate::r_s04_common::untapped_lands;
use mtg_engine::card::{card, CardDef};
use mtg_engine::commander_rules::check_commander_draft_deck;
use mtg_engine::deck::DeckProblem;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::kw::partner::{check_commander_deck, commanders_problem};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;
use std::sync::Arc;

fn cards(names: &[&str]) -> Vec<Arc<CardDef>> {
    names.iter().map(|n| card(n)).collect()
}

fn can_pair(a: &str, b: &str) -> bool {
    commanders_problem(&[&card(a), &card(b)]).is_none()
}

/// A Commander deck: the commanders plus `n` copies of `land`.
fn deck(commanders: &[&str], land: &str, n: usize) -> Vec<Arc<CardDef>> {
    commanders
        .iter()
        .map(|c| card(c))
        .chain((0..n).map(|_| card(land)))
        .collect()
}

/// A two-player Commander game already under way.
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

/// A game that hasn't begun, with the given decks.
fn pregame(config: GameConfig, decks: Vec<Vec<Arc<CardDef>>>) -> TestGame {
    crate::r_s12_common::pregame(config, decks)
}

#[test]
fn partners_can_share_colors_and_draft_allows_two_of_the_same() {
    cr!("702.124c", "702.124d", "702.124h", "903.13f", "903.8");
    ruling!(
        "Rograkh, Son of Rohgahh",
        "You can choose two commanders with partner that are the same color or colors. In Commander Draft, you can even choose two of the same commander with partner if you drafted them. If you do this, make sure you keep the number of times you've cast each from the command zone clear for \"commander tax\" purposes."
    );
    supported("Rograkh, Son of Rohgahh");
    // Rograkh and Kediss, Emberclaw Familiar are both red.
    assert!(can_pair("Rograkh, Son of Rohgahh", "Kediss, Emberclaw Familiar"));
    let pair = cards(&["Rograkh, Son of Rohgahh", "Kediss, Emberclaw Familiar"]);
    let d = deck(
        &["Rograkh, Son of Rohgahh", "Kediss, Emberclaw Familiar"],
        "Mountain",
        98,
    );
    assert!(check_commander_deck(&d, &pair, &[], false).is_empty());
    // Commander Draft: two drafted Rograkhs can both be commanders.
    let two = cards(&["Rograkh, Son of Rohgahh", "Rograkh, Son of Rohgahh"]);
    let draft_deck = deck(
        &["Rograkh, Son of Rohgahh", "Rograkh, Son of Rohgahh"],
        "Mountain",
        58,
    );
    assert_eq!(
        check_commander_draft_deck(&draft_deck, &two, &two, &[]),
        vec![]
    );
    // In a game, each has its own commander tax: casting one doesn't make the other cost
    // {2} more. Both begin in the command zone.
    let mut lib = draft_deck.clone();
    lib.rotate_left(2);
    let mut t = pregame(
        GameConfig {
            variant: Variant::Commander,
            skip_mulligans: true,
            starting_player: Some(P0),
            ..Default::default()
        },
        vec![lib, deck(&[], "Island", 60)],
    );
    assert!(t.g.designate_commander(P0, "Rograkh, Son of Rohgahh"));
    assert!(t.g.designate_commander(P0, "Rograkh, Son of Rohgahh"));
    t.g.start();
    let ids = t.g.find_in_zone(Zone::Command, "Rograkh, Son of Rohgahh");
    assert_eq!(ids.len(), 2);
    t.advance_to(P0, mtg_engine::turn::Step::PrecombatMain);
    let (a, b) = (ids[0], ids[1]);
    // Rograkh costs {0}: the first one needs no mana. It dies and returns to the command
    // zone.
    t.cast(P0, a).go();
    t.resolve_all();
    assert!(t.on_battlefield(a));
    t.answer_yes(P0, true);
    t.g.destroy(t.g.current(a), None);
    t.settle();
    assert_eq!(t.zone(a), Zone::Command);
    let a = t.g.current(a);
    assert_eq!(mtg_engine::kw::partner::commander_tax(&t.g, P0, a), 2);
    // The second costs no more: it was never cast from the command zone.
    assert_eq!(mtg_engine::kw::partner::commander_tax(&t.g, P0, b), 0);
    let spell = t.cast(P0, b).try_go();
    assert!(spell.is_ok(), "{spell:?}");
    t.resolve_all();
    assert!(t.on_battlefield(b));
    // The first one still costs {2} more (not {4}): the second one's cast didn't add to
    // its count.
    assert_eq!(mtg_engine::kw::partner::commander_tax(&t.g, P0, a), 2);
    t.lands(P0, "Mountain", 3);
    t.cast(P0, a).go();
    assert_eq!(untapped_lands(&t, P0), 1);
}

#[test]
fn losing_partner_during_the_game_doesnt_end_being_a_commander() {
    cr!("702.124a", "903.3", "903.9a");
    ruling!(
        "Rograkh, Son of Rohgahh",
        "To have two commanders, both must have the partner ability as the game begins. Losing the ability during the game doesn't cause either to cease to be your commander."
    );
    ruling!(
        "Donatello, the Brains",
        "To have two commanders, both must have appropriate partner abilities as the game begins. Losing the ability during the game doesn't cause either to cease to be your commander."
    );
    supported("Humility");
    // Without partner as the game begins, they can't both be commanders.
    assert!(!can_pair("Rograkh, Son of Rohgahh", "Isamaru, Hound of Konda"));
    for (a, b) in [
        ("Rograkh, Son of Rohgahh", "Kraum, Ludevic's Opus"),
        ("Donatello, the Brains", "Splinter, the Mentor"),
    ] {
        assert!(can_pair(a, b), "{a} and {b}");
        let mut t = commander_game();
        let x = commander(&mut t, P0, a);
        commander(&mut t, P0, b);
        give_mana_for(&mut t, P0, a);
        t.cast(P0, x).go();
        t.resolve_all();
        assert!(t.on_battlefield(x));
        // Humility: "All creatures lose all abilities and have base power and toughness
        // 1/1." It no longer has partner, but it's still a commander: when it dies, it may
        // go to the command zone, and it's taxed for its earlier cast.
        t.battlefield(P1, "Humility");
        assert!(!t
            .obj_now(x)
            .has_keyword(mtg_engine::keywords::KeywordKind::Partner));
        assert!(t.obj_now(x).is_commander);
        t.answer_yes(P0, true);
        t.g.destroy(t.g.current(x), None);
        t.settle();
        assert_eq!(t.zone(x), Zone::Command, "{a}");
        let x = t.g.current(x);
        assert!(t.obj(x).is_commander);
        assert_eq!(
            mtg_engine::kw::partner::commander_tax(&t.g, P0, x),
            2,
            "{a}"
        );
    }
}

#[test]
fn both_commanders_start_in_the_command_zone_the_other_98_are_the_library() {
    cr!("702.124b", "903.6", "903.7");
    ruling!(
        "Pippin, Warden of Isengard",
        "Both commanders start in the command zone, and the remaining 98 cards of your deck are shuffled to become your library."
    );
    supported("Pippin, Warden of Isengard");
    supported("Merry, Warden of Isengard");
    let pair = ["Pippin, Warden of Isengard", "Merry, Warden of Isengard"];
    let d = deck(&pair, "Forest", 98);
    assert!(check_commander_deck(&d, &cards(&pair), &[], false).is_empty());
    let mut lib = d.clone();
    lib.rotate_left(1);
    let mut t = pregame(
        GameConfig {
            skip_mulligans: true,
            ..GameConfig::commander_game()
        },
        vec![lib, deck(&[], "Island", 100)],
    );
    for c in pair {
        assert!(t.g.designate_commander(P0, c));
    }
    t.g.start();
    for c in pair {
        let ids = t.g.find_in_zone(Zone::Command, c);
        assert_eq!(ids.len(), 1, "{c} in the command zone");
        assert!(t.obj(ids[0]).is_commander);
    }
    // The other 98 cards: 91 in the library after drawing seven.
    assert_eq!(t.library_size(P0) + t.hand_size(P0), 98);
    assert_eq!(t.hand_size(P0), 7);
    let all_forests = t
        .g
        .player(P0)
        .library
        .iter()
        .chain(t.g.player(P0).hand.iter())
        .all(|id| t.obj(*id).chars.name.as_str() == "Forest");
    assert!(all_forests);
}

#[test]
fn two_commanders_combined_color_identity_leonardo_allows_every_color() {
    cr!("702.124c", "903.4", "903.5c");
    ruling!(
        "Donatello, the Brains",
        "If Donatello, the Brains and Splinter, the Mentor are your commanders, your deck may contain cards with blue and/or black in their color identity, but not cards with red, green, or white. (If Leonardo, the Balance is one of your commanders, your deck may contain cards with any color or combination of colors in their color identity, regardless of who your other commander is.)"
    );
    supported("Leonardo, the Balance");
    // Donatello (blue) and Splinter (black).
    let pair = cards(&["Donatello, the Brains", "Splinter, the Mentor"]);
    let mut d = deck(&["Donatello, the Brains", "Splinter, the Mentor"], "Island", 96);
    d.extend(cards(&["Swamp", "Dimir Signet"]));
    assert!(check_commander_deck(&d, &pair, &[], false).is_empty());
    for other in ["Lightning Bolt", "Giant Growth", "Swords to Plowshares"] {
        let mut bad = d.clone();
        bad.pop();
        bad.push(card(other));
        assert!(
            check_commander_deck(&bad, &pair, &[], false)
                .contains(&DeckProblem::OutsideColorIdentity { name: other.into() }),
            "{other}"
        );
    }
    // Leonardo, the Balance (color identity WUBRG) and Donatello: any colors.
    let pair = cards(&["Leonardo, the Balance", "Donatello, the Brains"]);
    let mut d = deck(&["Leonardo, the Balance", "Donatello, the Brains"], "Island", 95);
    d.extend(cards(&["Lightning Bolt", "Giant Growth", "Swords to Plowshares"]));
    assert!(check_commander_deck(&d, &pair, &[], false).is_empty());
}

#[test]
fn partner_character_select_pairs_only_with_partner_character_select() {
    cr!("702.124a", "702.124f", "702.124i");
    ruling!(
        "Donatello, the Brains",
        "You can have two commanders if each one has \"Partner—Character select.\" You can't mix and match them with cards that have other partner abilities."
    );
    supported("April O'Neil, Live on the Scene");
    assert!(can_pair("Donatello, the Brains", "Splinter, the Mentor"));
    assert!(can_pair("April O'Neil, Live on the Scene", "Michelangelo, the Heart"));
    // Not with plain partner, "partner with", or "Partner—Friends forever".
    assert!(!can_pair("Donatello, the Brains", "Kraum, Ludevic's Opus"));
    assert!(!can_pair("Kraum, Ludevic's Opus", "Donatello, the Brains"));
    assert!(!can_pair("Splinter, the Mentor", "Pippin, Warden of Isengard"));
    assert!(!can_pair("Splinter, the Mentor", "Wernog, Rider's Chaplain"));
}
