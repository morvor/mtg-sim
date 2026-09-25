//! CR 108: cards — Oracle wording, what a "card" is, nontraditional cards, ownership and
//! control of cards.

use super::r105_util::*;
use super::r111_tokens::run_text;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::events::MoveCause;
use mtg_engine::game::{Game, GameConfig};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Puts a real card into a player's sideboard (outside the game).
fn sideboard(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    t.custom(p, (*card(name)).clone(), Zone::Outside(p))
}

#[test]
fn card_wording_comes_from_the_oracle_card_reference() {
    cr!("108.1");
    // Lord of Atlantis was printed "All Merfolk get +1/+1 and islandwalk"; its Oracle text
    // says "Other Merfolk", so it doesn't pump itself.
    let oracle = mtg_data::cards()
        .by_name("Lord of Atlantis")
        .unwrap()
        .oracle_text
        .clone()
        .unwrap();
    assert!(oracle.starts_with("Other Merfolk get +1/+1"));
    assert_eq!(
        card("Lord of Atlantis").front().chars.rules_text.as_ref(),
        oracle.as_str()
    );
    let mut t = TestGame::new(2);
    let lord = t.battlefield(P0, "Lord of Atlantis");
    let merfolk = t.battlefield(P0, "Merfolk of the Pearl Trident");
    t.g.recompute();
    assert_eq!(t.pt(lord), (2, 2));
    assert_eq!(t.pt(merfolk), (2, 2));
    assert!(t
        .obj(merfolk)
        .has_keyword(mtg_engine::keywords::KeywordKind::Landwalk));
}

#[test]
fn only_cards_count_as_cards() {
    cr!("108.2", "108.2b");
    ruling!(
        "Tarmogoyf",
        "Tarmogoyf counts card types, not cards."
    );
    let mut t = TestGame::new(2);
    let goyf = t.battlefield(P0, "Tarmogoyf");
    t.g.recompute();
    assert_eq!(t.pt(goyf), (0, 1));
    // A token in a graveyard (before it ceases to exist) isn't a card: Tarmogoyf doesn't
    // count it.
    let token = run_text(&mut t, P0, "Create a 1/1 white Soldier creature token.")[0];
    // (The Token Maker sorcery went to the graveyard: one card type so far.)
    t.g.recompute();
    assert_eq!(t.pt(goyf), (1, 2));
    let gone = t
        .g
        .move_object(token, Zone::Graveyard(P0), MoveCause::Effect, None)
        .unwrap();
    assert!(t.obj(gone).is_token());
    t.g.recompute();
    assert_eq!(t.pt(goyf), (1, 2));
    // A creature card does count.
    t.graveyard(P1, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(goyf), (2, 3));
}

#[test]
fn a_token_put_into_a_graveyard_isnt_a_card_put_into_a_graveyard() {
    cr!("108.2b");
    let mut t = TestGame::new(2);
    // Planar Void: "Whenever another card is put into a graveyard from anywhere, exile
    // that card."
    t.battlefield(P1, "Planar Void");
    let soldier = run_text(&mut t, P0, "Create a 1/1 white Soldier creature token.")[0];
    // The Token Maker sorcery was a card: it was exiled.
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0);
    let stack_before = t.stack_len();
    let murder = t.hand(P0, "Murder");
    t.lands(P0, "Swamp", 3);
    t.cast(P0, murder).target(soldier).go();
    t.resolve();
    // Only Murder (a card) triggers Planar Void; the token doesn't.
    assert_eq!(t.stack_len(), stack_before + 1);
    t.resolve_all();
    assert!(t.in_exile("Murder"));
}

#[test]
fn nontraditional_cards_arent_part_of_a_deck() {
    cr!("108.2a", "108.5");
    let mut deck = vec![card("Forest"); 20];
    deck.push(card("Bad Wolf Bay"));
    deck.push(card("What's Yours Is Now Mine"));
    deck.push(card("Titania"));
    deck.push(card("Lost Mine of Phandelver"));
    let decks = vec![deck, vec![card("Island"); 20]];
    let g = Game::new(GameConfig::default(), decks, vec![]);
    // Only the 20 traditional cards are in the library.
    assert_eq!(g.player(P0).library.len(), 20);
    assert!(g
        .player(P0)
        .library
        .iter()
        .all(|id| g.obj(*id).base.name.as_str() == "Forest"));
    // Planes and schemes start face down in the command zone (supplementary decks), a
    // vanguard face up; a dungeon starts outside the game.
    let find = |name: &str| {
        g.objects
            .iter()
            .find(|o| o.base.name.as_str() == name)
            .unwrap()
    };
    let plane = find("Bad Wolf Bay");
    assert_eq!(plane.zone, Zone::Command);
    assert!(plane.face_down);
    let scheme = find("What's Yours Is Now Mine");
    assert_eq!(scheme.zone, Zone::Command);
    assert!(scheme.face_down);
    let vanguard = find("Titania");
    assert_eq!(vanguard.zone, Zone::Command);
    assert!(!vanguard.face_down);
    let dungeon = find("Lost Mine of Phandelver");
    assert_eq!(dungeon.zone, Zone::Outside(P0));
    for o in [plane, scheme, vanguard, dungeon] {
        assert_eq!(o.owner, P0);
    }
}

#[test]
fn a_cards_owner_is_the_player_who_started_the_game_with_it() {
    cr!("108.3");
    let decks = vec![vec![card("Forest"); 10], vec![card("Grizzly Bears"); 10]];
    let g = Game::new(GameConfig::default(), decks, vec![]);
    assert!(g
        .player(P1)
        .library
        .iter()
        .all(|id| g.obj(*id).owner == P1));
    // Ownership doesn't change when another player controls it: the stolen creature goes
    // to its owner's graveyard.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let treason = t.hand(P0, "Act of Treason");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, treason).target(bears).go();
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, P0);
    let murder = t.hand(P0, "Murder");
    t.lands(P0, "Swamp", 3);
    t.cast(P0, murder).target(bears).go();
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.obj_now(bears).owner, P1);
}

#[test]
fn a_card_brought_in_from_the_sideboard_is_owned_by_the_player_who_started_with_it() {
    cr!("108.3", "108.3b");
    let mut t = TestGame::new(2);
    let axe = sideboard(&mut t, P0, "Lava Axe");
    let theirs = sideboard(&mut t, P1, "Stone Rain");
    let wish = t.hand(P0, "Burning Wish");
    t.lands(P0, "Mountain", 2);
    t.answer_yes(P0, true);
    t.cast(P0, wish).go();
    t.resolve();
    assert!(t.in_hand(P0, "Lava Axe"));
    let axe = t.g.current(axe);
    assert_eq!(t.obj(axe).owner, P0);
    // Only a card P0 owns could be chosen: P1's sideboard card stays outside the game.
    assert_eq!(t.zone(theirs), Zone::Outside(P1));
    // A second wish finds nothing: P1's sorcery isn't P0's.
    let wish2 = t.hand(P0, "Burning Wish");
    t.lands(P0, "Mountain", 2);
    t.answer_yes(P0, true);
    t.cast(P0, wish2).go();
    t.resolve();
    assert_eq!(t.zone(theirs), Zone::Outside(P1));
    assert!(!t.in_hand(P0, "Stone Rain"));
}

#[test]
fn a_card_that_isnt_a_permanent_or_spell_is_controlled_by_its_owner() {
    cr!("108.4", "108.4a", "109.4");
    let mut t = TestGame::new(2);
    // P0 steals P1's Grizzly Bears, then it dies.
    let bears = t.battlefield(P1, "Grizzly Bears");
    let treason = t.hand(P0, "Act of Treason");
    t.lands(P0, "Mountain", 3);
    t.cast(P0, treason).target(bears).go();
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, P0);
    let murder = t.hand(P0, "Murder");
    t.lands(P0, "Swamp", 3);
    t.cast(P0, murder).target(bears).go();
    t.resolve();
    let card_in_gy = t.g.current(bears);
    // "Target creature card's controller loses 2 life": it has no controller, so its
    // owner is used.
    let drain = card_with(
        "Grave Drain",
        "{0}",
        "Sorcery",
        None,
        vec![spell_ab(
            vec![TargetSpec::object(
                Filter::and(vec![
                    Filter::Type(CardType::Creature),
                    Filter::Card,
                    Filter::InZone(ZoneKind::Graveyard),
                ]),
                "target creature card in a graveyard",
            )],
            Effect::LoseLife {
                who: PlayerRef::ControllerOf(Box::new(Sel::Target(0))),
                n: Value::c(2),
            },
        )],
    );
    let d = put_in_hand(&mut t, P0, drain);
    t.cast(P0, d).target(card_in_gy).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn nontraditional_cards_cant_be_brought_in_from_outside_the_game() {
    cr!("108.5");
    let mut t = TestGame::new(2);
    // Death Wish: "You may put a card you own from outside the game into your hand."
    let plane = sideboard(&mut t, P0, "Bad Wolf Bay");
    let wish = t.hand(P0, "Death Wish");
    t.lands(P0, "Swamp", 3);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(plane)]);
    t.cast(P0, wish).go();
    t.resolve();
    assert_eq!(t.zone(plane), Zone::Outside(P0));
    assert!(!t.in_hand(P0, "Bad Wolf Bay"));
    // A dungeon card can be brought into the game (into the command zone).
    let dungeon = sideboard(&mut t, P0, "Lost Mine of Phandelver");
    let enter = card_with(
        "Enter the Dungeon",
        "{0}",
        "Sorcery",
        None,
        vec![spell_ab(
            vec![],
            Effect::Move {
                what: Sel::Choose {
                    chooser: PlayerRef::You,
                    filter: Filter::and(vec![
                        Filter::Type(CardType::Dungeon),
                        Filter::InZone(ZoneKind::Outside),
                        Filter::OwnedBy(PlayerRel::You),
                    ]),
                    count: Value::c(1),
                    up_to: false,
                    store: None,
                },
                to: Destination::zone(ZoneKind::Command),
            },
        )],
    );
    let e = put_in_hand(&mut t, P0, enter);
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, e).go();
    t.resolve();
    assert_eq!(t.zone(dungeon), Zone::Command);
}
