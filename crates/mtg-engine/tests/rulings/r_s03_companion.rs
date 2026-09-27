//! Rulings batch S03 — companion (CR 702.139): "If this card is your chosen companion, you
//! may put it into your hand from outside the game for {3} as a sorcery."

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use mtg_engine::card::{card, CardDef};
use mtg_engine::decision::{Action, Answer, Decision, SpecialAction};
use mtg_engine::game::GameConfig;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use std::sync::Arc;

const KAHEERA: &str = "Kaheera, the Orphanguard";

fn copies(name: &str, n: usize) -> Vec<Arc<CardDef>> {
    (0..n).map(|_| card(name)).collect()
}

/// A game (not started) with these decks, `starting` taking the first turn.
fn new_game(starting: PlayerId, decks: Vec<Vec<Arc<CardDef>>>) -> TestGame {
    use std::collections::VecDeque;
    use std::sync::Mutex;
    let n = decks.len();
    let script = Arc::new(Mutex::new(Script {
        queues: vec![VecDeque::new(); n],
        asked: vec![],
    }));
    let agents: Vec<Box<dyn mtg_engine::decision::Agent>> = (0..n)
        .map(|i| {
            Box::new(ScriptedAgent {
                player: PlayerId(i as u8),
                script: script.clone(),
            }) as Box<dyn mtg_engine::decision::Agent>
        })
        .collect();
    let config = GameConfig {
        starting_player: Some(starting),
        ..Default::default()
    };
    let mut g = mtg_engine::game::Game::new(config, decks, agents);
    g.logging = true;
    TestGame { g, script }
}

/// 60 cards whose creature cards are all Cats (Kaheera's condition), and whose nonland
/// cards all share a card type (Umori's).
fn cat_deck() -> Vec<Arc<CardDef>> {
    let mut d = copies("Savannah Lions", 20);
    d.extend(copies("Plains", 40));
    d
}

/// The companion reveal decisions asked: (player, candidates, max).
fn reveals(t: &TestGame) -> Vec<(PlayerId, Vec<Entity>, u32)> {
    t.asked()
        .into_iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities {
                prompt,
                candidates,
                max,
                ..
            } if prompt.contains("companion") => Some((p, candidates, max)),
            _ => None,
        })
        .collect()
}

/// P0's game with a Cat deck and Kaheera revealed as companion, in P0's first precombat
/// main phase. Returns the game and Kaheera.
fn kaheera_game() -> (TestGame, ObjectId) {
    supported(KAHEERA);
    let mut t = new_game(P0, vec![cat_deck(), copies("Forest", 60)]);
    let side = t.g.add_to_sideboard(P0, vec![card(KAHEERA)]);
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.g.start();
    assert_eq!(t.g.companion_of(P0), Some(side[0]));
    t.advance_to(P0, Step::PrecombatMain);
    (t, side[0])
}

/// P0 pays {3} to put their companion into their hand (the special action).
fn companion_to_hand(t: &mut TestGame, companion: ObjectId) -> bool {
    t.lands(P0, "Wastes", 3);
    t.g.turn.priority = Some(P0);
    t.g.perform_action(
        P0,
        Action::Special(SpecialAction::CompanionToHand { card: companion }),
    )
    .is_ok()
}

#[test]
fn a_companion_is_revealed_before_shuffling_and_begins_the_game_outside_it() {
    cr!("103.2b", "103.3", "702.139a");
    ruling!(
        "Kaheera, the Orphanguard",
        "Before shuffling your deck to become your library, you may reveal one card from outside the game to be your companion if your starting deck meets the requirements of the companion ability. You can't reveal more than one. It remains revealed outside the game as the game begins."
    );
    ruling!(
        "Kaheera, the Orphanguard",
        "Your companion begins the game outside the game. In tournament play, this means your sideboard. In casual play, it's simply a card you own that's not in your starting deck."
    );
    supported("Umori, the Collector");
    let mut t = new_game(P0, vec![cat_deck(), copies("Forest", 60)]);
    let deck = t.g.player(P0).library.clone();
    let side =
        t.g.add_to_sideboard(P0, vec![card(KAHEERA), card("Umori, the Collector")]);
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("companion")),
        |g| g.player(P0).library.clone(),
    );
    t.answer_choose(P0, &[Entity::Object(side[0])]);
    t.g.start();
    // One choice among both companions, of at most one card, before the deck was
    // shuffled.
    let r = reveals(&t);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].1.len(), 2);
    assert_eq!(r[0].2, 1);
    assert_eq!(seen.lock().unwrap().clone(), vec![deck.clone()]);
    let mut now = t.g.player(P0).library.clone();
    now.extend(t.g.player(P0).hand.iter().copied());
    assert_ne!(now, deck);
    // Kaheera is the companion, still outside the game; a second one can't be revealed.
    assert_eq!(t.g.companion_of(P0), Some(side[0]));
    assert!(!mtg_engine::kw::companion::choose_companion(
        &mut t.g, P0, side[1]
    ));
    for c in &side {
        assert_eq!(t.zone(*c), Zone::Outside(P0));
        assert!(!t.g.player(P0).library.contains(c));
        assert!(!t.g.player(P0).hand.contains(c));
    }
    assert_eq!(t.library_size(P0) + t.hand_size(P0), 60);
}

#[test]
fn companions_are_revealed_in_turn_order_starting_with_the_starting_player() {
    cr!("103.2b", "101.4e");
    ruling!(
        "Kaheera, the Orphanguard",
        "If more than one player wishes to reveal a companion, the starting player does so first, and players proceed in turn order. Once a player has chosen not to reveal a companion, that player can't change their mind."
    );
    // P1 starts; both could reveal Kaheera. P1 declines, then P0 reveals; P1 isn't asked
    // again.
    let mut t = new_game(P1, vec![cat_deck(), cat_deck()]);
    let mine = t.g.add_to_sideboard(P0, vec![card(KAHEERA)]);
    t.g.add_to_sideboard(P1, vec![card(KAHEERA)]);
    t.answer_choose(P1, &[]);
    t.answer_choose(P0, &[Entity::Object(mine[0])]);
    t.g.start();
    let order: Vec<PlayerId> = reveals(&t).into_iter().map(|(p, _, _)| p).collect();
    assert_eq!(order, vec![P1, P0]);
    assert_eq!(t.g.companion_of(P1), None);
    assert_eq!(t.g.companion_of(P0), Some(mine[0]));
}

#[test]
fn a_card_with_companion_in_the_starting_deck_is_an_ordinary_card() {
    cr!("702.139a", "100.2a");
    ruling!(
        "Kaheera, the Orphanguard",
        "The companion ability has no effect if the card is in your starting deck and creates no restriction on putting a card with a companion ability into your starting deck."
    );
    supported(KAHEERA);
    // Kaheera in a deck whose other creatures aren't Cats, Elementals, Nightmares,
    // Dinosaurs or Beasts: the deck is legal, and nobody is offered a companion.
    let mut deck = copies(KAHEERA, 1);
    for name in [
        "Grizzly Bears",
        "Hill Giant",
        "Llanowar Elves",
        "Elvish Mystic",
    ] {
        deck.extend(copies(name, 4));
    }
    deck.extend(copies("Centaur Courser", 3));
    deck.extend(copies("Forest", 40));
    assert_eq!(deck.len(), 60);
    assert!(mtg_engine::deck::check_constructed(&deck).is_empty());
    let mut t = new_game(P0, vec![deck, copies("Forest", 60)]);
    t.g.start();
    assert!(reveals(&t).is_empty());
    assert_eq!(t.g.companion_of(P0), None);
    t.advance_to(P0, Step::PrecombatMain);
    // It's cast like any other card.
    let k =
        t.g.player(P0)
            .library
            .iter()
            .chain(t.g.player(P0).hand.iter())
            .copied()
            .find(|c| t.obj(*c).chars.name == KAHEERA)
            .unwrap();
    if t.zone(k) != Zone::Hand(P0) {
        t.g.move_object(
            k,
            Zone::Hand(P0),
            mtg_engine::events::MoveCause::Effect,
            None,
        );
    }
    let k = t.g.current(k);
    t.lands(P0, "Plains", 3);
    t.cast(P0, k).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield(KAHEERA).len(), 1);
}

#[test]
fn a_companions_other_abilities_work_only_on_the_battlefield() {
    cr!("702.139a", "113.6", "400.1");
    ruling!(
        "Kaheera, the Orphanguard",
        "The companion's other abilities apply only if the creature is on the battlefield. They have no effect while the companion is outside the game."
    );
    // Kaheera: "Each other creature you control that's a Cat, Elemental, Nightmare,
    // Dinosaur, or Beast gets +1/+1 and has vigilance."
    let (mut t, kaheera) = kaheera_game();
    let lions = t.battlefield(P0, "Savannah Lions");
    assert_eq!(t.pt(lions), (2, 1));
    assert!(!t.obj(lions).has_keyword(KeywordKind::Vigilance));
    assert!(companion_to_hand(&mut t, kaheera));
    assert_eq!(t.pt(lions), (2, 1));
    t.lands(P0, "Plains", 3);
    let k = t.g.current(kaheera);
    t.cast(P0, k).go();
    t.resolve_all();
    assert_eq!(t.pt(lions), (3, 2));
    assert!(t.obj(lions).has_keyword(KeywordKind::Vigilance));
}

#[test]
fn a_companion_put_into_hand_is_like_any_other_card_brought_into_the_game() {
    cr!("702.139a", "116.2g", "701.6a");
    ruling!(
        "Kaheera, the Orphanguard",
        "For example, if it's discard, countered, or destroyed, it's put into your graveyard, remaining in the game."
    );
    ruling!(
        "Kaheera, the Orphanguard",
        "Once you put your companion into your hand, it behaves like any other card you’ve brought into the game. For example, if it’s countered or destroyed, it’s put into your graveyard, remaining in the game."
    );
    // Discarded.
    let (mut t, kaheera) = kaheera_game();
    assert!(companion_to_hand(&mut t, kaheera));
    let k = t.g.current(kaheera);
    for c in t.g.player(P0).hand.clone() {
        if c != k {
            t.g.move_object(
                c,
                Zone::Library(P0),
                mtg_engine::events::MoveCause::Effect,
                None,
            );
        }
    }
    t.advance_to(P1, Step::PrecombatMain);
    let rot = t.hand(P1, "Mind Rot");
    t.lands(P1, "Swamp", 3);
    t.cast(P1, rot).target(P0).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, KAHEERA));
    assert_eq!(t.zone(kaheera), Zone::Graveyard(P0));
    // Countered.
    let (mut t, kaheera) = kaheera_game();
    assert!(companion_to_hand(&mut t, kaheera));
    t.lands(P0, "Plains", 3);
    let spell = t.cast(P0, t.g.current(kaheera)).go();
    let scatter = t.hand(P1, "Essence Scatter");
    t.lands(P1, "Island", 2);
    t.cast(P1, scatter).target(spell).go();
    t.resolve_all();
    assert_eq!(t.zone(kaheera), Zone::Graveyard(P0));
    // Destroyed.
    let (mut t, kaheera) = kaheera_game();
    assert!(companion_to_hand(&mut t, kaheera));
    t.lands(P0, "Plains", 3);
    t.cast(P0, t.g.current(kaheera)).go();
    t.resolve_all();
    destroy(&mut t, kaheera);
    assert_eq!(t.zone(kaheera), Zone::Graveyard(P0));
    // It stays in the game: it isn't put back outside, nor can it be put into hand again.
    assert!(t
        .g
        .player(P0)
        .sideboard
        .iter()
        .all(|c| !t.g.is_live(*c) || t.obj(*c).chars.name != KAHEERA));
    t.g.turn.priority = Some(P0);
    assert!(!t
        .g
        .legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Special(SpecialAction::CompanionToHand { .. }))));
}

#[test]
fn putting_a_companion_into_hand_is_a_special_action_a_revoker_cant_stop() {
    cr!("702.139a", "116.2g", "116.1", "602.1");
    ruling!(
        "Kaheera, the Orphanguard",
        "Once per game, any time you could cast a sorcery (during your main phase when the stack is empty), you can pay {3} to put your companion from your sideboard into your hand. This is a special action, not an activated ability. It happens immediately and can’t be responded to. It can’t be countered or stopped by cards like Phyrexian Revoker."
    );
    supported("Phyrexian Revoker");
    let (mut t, kaheera) = kaheera_game();
    // Phyrexian Revoker naming Kaheera: "Activated abilities of sources with the chosen
    // name can't be activated."
    t.answer(P1, DecisionKind::Name, Answer::Text(KAHEERA.into()));
    t.enter(P1, "Phyrexian Revoker");
    t.resolve_all();
    let revoker = t.named_on_battlefield("Phyrexian Revoker")[0];
    assert_eq!(t.obj(revoker).choices.card_name.as_deref(), Some(KAHEERA));
    // It can't be cast from outside the game.
    t.lands(P0, "Plains", 3);
    t.g.turn.priority = Some(P0);
    assert!(!t
        .g
        .legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == kaheera)));
    assert!(companion_to_hand(&mut t, kaheera));
    assert!(t.in_hand(P0, KAHEERA));
    assert_eq!(t.stack_len(), 0);
    // Only once per game: back outside the game (as if), it can't be put into hand again.
    let k = t.g.current(kaheera);
    t.g.move_object(
        k,
        Zone::Outside(P0),
        mtg_engine::events::MoveCause::Effect,
        None,
    );
    t.g.turn.priority = Some(P0);
    assert!(!t
        .g
        .legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Special(SpecialAction::CompanionToHand { .. }))));
}
