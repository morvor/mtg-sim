//! The worked examples of CR 903 (the Commander variant), played out as tests.

use crate::r703_common::supported;
use mtg_engine::card::{card, CardDef};
use mtg_engine::commander_rules::{commanders_of, computed_color_identity};
use mtg_engine::deck::{check_commander, DeckProblem};
use mtg_engine::game::GameConfig;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

const ISAMARU: &str = "Isamaru, Hound of Konda";

fn commander_game(n: usize) -> TestGame {
    TestGame::with_config(n, GameConfig::commander_game())
}

/// Makes `id` a commander (the designation of its card, CR 903.3).
fn make_commander(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].is_commander = true;
    let owner = t.obj(id).owner;
    let name = t.obj(id).card.as_ref().unwrap().name.clone();
    t.g.players[owner.idx()].commander_names.push(name);
}

fn colors(cs: &[Color]) -> ColorSet {
    let mut s = ColorSet::NONE;
    for c in cs {
        s.insert(*c);
    }
    s
}

/// A Commander deck: the commander, `extra`, and basic lands up to 100 cards.
fn deck(commander: &Arc<CardDef>, extra: &[&str], land: &str) -> Vec<Arc<CardDef>> {
    let mut d = vec![commander.clone()];
    d.extend(extra.iter().map(|n| card(n)));
    while d.len() < 100 {
        d.push(card(land));
    }
    d
}

/// The cards of `extra` the deck check rejects for their colors or mana.
fn rejected(commander: &str, extra: &[&str], land: &str) -> Vec<String> {
    let c = card(commander);
    let mut v: Vec<String> = check_commander(&deck(&c, extra, land), &c, &[], false)
        .into_iter()
        .filter_map(|p| match p {
            DeckProblem::OutsideColorIdentity { name } => Some(name.to_string()),
            DeckProblem::ManaOutsideColorIdentity { name } => Some(name.to_string()),
            _ => None,
        })
        .collect();
    v.sort();
    v.dedup();
    v
}

#[test]
fn face_down_and_copying_commanders_are_commanders_but_a_copy_of_one_isnt() {
    cr!("903.9a", "708.2");
    cr_example!("903.3", "Ixidron");
    supported("Ixidron");
    supported("Cytoshape");
    supported("Body Double");
    // Turned face down by Ixidron ("As this creature enters, turn all other nontoken
    // creatures face down."): still a commander; it returns to the command zone.
    let mut t = commander_game(2);
    let cmdr = t.battlefield(P0, ISAMARU);
    make_commander(&mut t, cmdr);
    t.enter(P1, "Ixidron");
    t.g.recompute();
    let cmdr = t.g.current(cmdr);
    assert!(t.obj(cmdr).face_down);
    assert_eq!(commanders_of(&t.g, P0), vec![cmdr]);
    t.answer_yes(P0, true);
    t.g.destroy(cmdr, None);
    t.settle();
    assert_eq!(t.g.find_in_zone(Zone::Command, ISAMARU).len(), 1);

    // Copying another creature because of Cytoshape: still a commander.
    let mut t = commander_game(2);
    t.set_step(P0, Step::PrecombatMain);
    let cmdr = t.battlefield(P0, ISAMARU);
    make_commander(&mut t, cmdr);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Tropical Island", 3);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    let cyto = t.hand(P0, "Cytoshape");
    t.cast(P0, cyto).target(cmdr).go();
    t.resolve_all();
    assert_eq!(t.obj(cmdr).chars.name, "Grizzly Bears");
    assert_eq!(commanders_of(&t.g, P0), vec![cmdr]);
    t.answer_yes(P0, true);
    t.g.destroy(cmdr, None);
    t.settle();
    assert_eq!(t.g.find_in_zone(Zone::Command, ISAMARU).len(), 1);

    // Body Double copying a commander in a graveyard: not a commander.
    let mut t = commander_game(2);
    let cmdr = t.graveyard(P0, ISAMARU);
    make_commander(&mut t, cmdr);
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(cmdr)]);
    let double = t.enter(P1, "Body Double");
    let double = t.g.current(double);
    assert_eq!(t.obj(double).chars.name, ISAMARU);
    assert!(!t.obj(double).is_commander);
    assert!(commanders_of(&t.g, P1).is_empty());
    assert_eq!(commanders_of(&t.g, P0), vec![cmdr]);
    // When it dies, it isn't put into the command zone.
    t.answer_yes(P1, true);
    t.g.destroy(double, None);
    t.settle();
    assert!(t.in_graveyard(P1, "Body Double"));
    assert!(t.g.find_in_zone(Zone::Command, ISAMARU).is_empty());
}

#[test]
fn bosh_iron_golems_color_identity_is_red() {
    cr!("903.5c");
    cr_example!("903.4", "Bosh, Iron Golem");
    let bosh = card("Bosh, Iron Golem");
    // Mana cost {8}: a colorless card, but {3}{R} in its rules text.
    assert_eq!(bosh.front().chars.colors, ColorSet::NONE);
    assert_eq!(computed_color_identity(&bosh), colors(&[Color::Red]));
    // So a Bosh deck can have red cards, but not green ones.
    assert_eq!(
        rejected("Bosh, Iron Golem", &["Lightning Bolt", "Giant Growth"], "Mountain"),
        vec!["Giant Growth"]
    );
}

#[test]
fn civilized_scholars_color_identity_is_blue_and_red() {
    cr!("903.4");
    cr_example!("903.4d", "Civilized Scholar");
    let scholar = card("Civilized Scholar // Homicidal Brute");
    // The front face ({2}{U}) is blue; Homicidal Brute has a red color indicator.
    assert_eq!(scholar.front().chars.colors, colors(&[Color::Blue]));
    assert_eq!(
        computed_color_identity(&scholar),
        colors(&[Color::Blue, Color::Red])
    );
    // It can't be in a mono-blue deck.
    assert_eq!(
        rejected("Niv-Mizzet, Parun", &["Civilized Scholar // Homicidal Brute"], "Island"),
        Vec::<String>::new()
    );
    let mono_blue = "Talrand, Sky Summoner";
    assert_eq!(
        computed_color_identity(&card(mono_blue)),
        colors(&[Color::Blue])
    );
    assert_eq!(
        rejected(mono_blue, &["Civilized Scholar // Homicidal Brute"], "Island"),
        vec!["Civilized Scholar // Homicidal Brute"]
    );
}

#[test]
fn a_wort_deck_has_only_red_green_or_colorless_cards_and_mana_symbols() {
    cr!("903.4");
    cr_example!("903.5c", "Wort, the Raidmother");
    let wort = card("Wort, the Raidmother");
    // {4}{R/G}{R/G}: red and green.
    assert_eq!(
        computed_color_identity(&wort),
        colors(&[Color::Red, Color::Green])
    );
    // Only red (Lightning Bolt), only green (Giant Growth), red and green (Rhythm of the
    // Wild), colorless (Sol Ring, and Gruul Signet's {R}{G} symbols): fine.
    // A card with a white hybrid symbol (Figure of Destiny, {R/W}), a white card (Savannah
    // Lions), and a colorless card with a blue mana symbol in its rules text (Izzet
    // Signet: "{1}, {T}: Add {U}{R}."): not.
    assert_eq!(
        rejected(
            "Wort, the Raidmother",
            &[
                "Lightning Bolt",
                "Giant Growth",
                "Rhythm of the Wild",
                "Sol Ring",
                "Gruul Signet",
                "Figure of Destiny",
                "Savannah Lions",
                "Izzet Signet",
            ],
            "Mountain"
        ),
        vec!["Figure of Destiny", "Izzet Signet", "Savannah Lions"]
    );
}

#[test]
fn a_wort_deck_may_have_mountains_and_forests_but_no_plains_islands_or_swamps() {
    cr!("903.4");
    cr_example!("903.5d", "Mountain and/or Forest");
    assert_eq!(
        rejected(
            "Wort, the Raidmother",
            &[
                "Taiga",
                "Stomping Ground",
                "Forest",
                "Plateau",
                "Volcanic Island",
                "Badlands",
                "Plains",
                "Island",
                "Swamp",
            ],
            "Mountain"
        ),
        vec!["Badlands", "Island", "Plains", "Plateau", "Swamp", "Volcanic Island"]
    );
}
