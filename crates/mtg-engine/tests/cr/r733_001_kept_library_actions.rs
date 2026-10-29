//! What a reversal keeps (CR 733.1): "Players may not reverse actions that moved cards to a
//! library, moved cards from a library to any zone other than the stack, caused a library
//! to be shuffled, or caused cards from a library to be revealed." The engine restores the
//! game from before the attempt and performs those library actions again
//! (`reversal::keep_library_actions`). The in-game tests of each card that reaches the
//! exceptions are in `r733_001_reversal_reachability.rs` (`a_reversed_cast_keeps_...`).

use crate::r506_common::custom_card;
use mtg_engine::events::Event;
use mtg_engine::game::EventObserver;
use mtg_engine::mana::ManaCost;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;
use std::sync::{Arc, Mutex};

/// A custom instant with mana cost `cost`: "As an additional cost to cast this spell,
/// sacrifice a creature.\nYou gain 1 life."
fn creature_toll(t: &mut TestGame, cost: &str) -> ObjectId {
    let mut def = custom_card(
        "Creature Toll",
        "Instant",
        None,
        "As an additional cost to cast this spell, sacrifice a creature.\nYou gain 1 life.",
    );
    let cost = ManaCost::parse(cost).unwrap();
    def.faces[0].chars.colors = cost.colors();
    def.faces[0].chars.mana_cost = Some(cost);
    t.custom(P0, def, Zone::Hand(P0))
}

#[test]
fn a_draw_a_replacement_effect_makes_of_life_gained_for_mana_stays() {
    cr!("733.1", "605.1a", "121.1");
    interp!("illegal-action-reversal-restores-libraries");
    interp!("reversal-kept-library-actions-trigger");
    let mut t = TestGame::new(2);
    // "If you would gain life, draw that many cards instead."
    t.battlefield(P0, "Lich");
    // "{T}: Add {C}. You gain 1 life."
    let talisman = t.battlefield(P0, "Pristine Talisman");
    let life = t.g.player(P0).life;
    let spell = creature_toll(&mut t, "{1}");
    let top = t.g.library_top(P0).unwrap();
    let (hand, library) = (t.hand_size(P0), t.library_size(P0));
    let drawn = t.g.history.cards_drawn.get(&P0).copied().unwrap_or(0);
    // Paying {1} taps Pristine Talisman, whose life gain Lich makes a draw; P0 controls no
    // creature to sacrifice.
    let r = t.cast(P0, spell).method(CastMethod::Normal).try_go();
    assert!(r.is_err());
    // The draw stays (and is a draw: it counts, and "whenever you draw" abilities see it).
    let card = t.g.current(top);
    assert_eq!(t.zone(card), Zone::Hand(P0));
    assert_eq!(
        (t.hand_size(P0), t.library_size(P0)),
        (hand + 1, library - 1)
    );
    assert_eq!(t.g.history.cards_drawn.get(&P0).copied(), Some(drawn + 1));
    assert!(t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, Event::Drew { player, card: c, .. } if *player == P0 && *c == card)));
    // The mana ability is reversed: the Talisman untaps, no mana, no life change.
    assert!(!t.g.obj(talisman).tapped);
    assert!(t.g.player(P0).mana_pool.mana.is_empty());
    assert_eq!(t.g.player(P0).life, life);
    assert_eq!(t.zone(spell), Zone::Hand(P0));
}

/// The cards of a library, top last, named by id for the objects older than `known` and
/// by name for the others (a card moved during an attempt is a different object in the
/// attempt and after the reversal).
fn library_keys(g: &Game, p: PlayerId, known: usize) -> Vec<String> {
    g.player(p)
        .library
        .iter()
        .map(|id| {
            if (id.0 as usize) < known {
                format!("#{}", id.0)
            } else {
                g.obj(*id).chars.name.to_string()
            }
        })
        .collect()
}

/// Casts Fling with Ashnod's Altar and Darksteel Colossus (the Colossus is sacrificed for
/// mana and shuffled into its owner's library, then nothing is left to sacrifice), and
/// returns the library as the attempt shuffled it and as it is after the reversal.
fn colossus_shuffled() -> (Vec<String>, Vec<String>, TestGame) {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ashnod's Altar");
    t.battlefield(P0, "Darksteel Colossus");
    t.lands(P0, "Mountain", 1);
    let fling = t.hand(P0, "Fling");
    let known = t.g.objects.len();
    let shuffled: Arc<Mutex<Option<Vec<String>>>> = Arc::default();
    let s2 = shuffled.clone();
    t.g.observer = Some(EventObserver::new(move |g, ev| {
        if matches!(ev, Event::Shuffled { player } if *player == P0) {
            s2.lock().unwrap().get_or_insert(library_keys(g, P0, known));
        }
    }));
    let r = t
        .cast(P0, fling)
        .target(Entity::Player(P1))
        .method(CastMethod::Normal)
        .try_go();
    assert!(r.is_err());
    let during = shuffled
        .lock()
        .unwrap()
        .clone()
        .expect("the attempt shuffled");
    let after = library_keys(&t.g, P0, known);
    (during, after, t)
}

#[test]
fn a_shuffle_stays_with_the_order_it_gave_the_library() {
    cr!("733.1", "701.24a");
    interp!("illegal-action-reversal-restores-libraries");
    let (during, after, t) = colossus_shuffled();
    // The library is in the order the attempt's shuffle gave it, the Colossus among it.
    assert_eq!(after, during);
    assert!(after.iter().any(|k| k == "Darksteel Colossus"));
    assert_eq!(t.g.reversals.kept, 2);
    // The same game gives the same shuffle (the generator's state is the one the shuffle
    // left; determinism).
    let (_, again, t2) = colossus_shuffled();
    assert_eq!(again, after);
    assert_eq!(format!("{:?}", t.g.rng), format!("{:?}", t2.g.rng));
    assert_eq!(
        format!("{:?}", t.g.player_rngs),
        format!("{:?}", t2.g.player_rngs)
    );
}

#[test]
fn a_kept_move_triggers_as_a_zone_change_but_not_as_the_sacrifice_it_was_paid_as() {
    cr!("733.1", "603.2", "603.10a");
    interp!("illegal-action-reversal-restores-libraries");
    interp!("reversal-kept-library-actions-trigger");
    let mut t = TestGame::new(2);
    let leaves = custom_card(
        "Parting Watch",
        "Enchantment",
        None,
        "Whenever a creature you control leaves the battlefield, you gain 1 life.",
    );
    let leaves = t.custom(P0, leaves, Zone::Battlefield);
    let sacrificed = custom_card(
        "Offering Watch",
        "Enchantment",
        None,
        "Whenever you sacrifice a permanent, you gain 1 life.",
    );
    let sacrificed = t.custom(P0, sacrificed, Zone::Battlefield);
    t.battlefield(P0, "Ashnod's Altar");
    let colossus = t.battlefield(P0, "Darksteel Colossus");
    t.lands(P0, "Mountain", 1);
    let fling = t.hand(P0, "Fling");
    let r = t
        .cast(P0, fling)
        .target(Entity::Player(P1))
        .method(CastMethod::Normal)
        .try_go();
    assert!(r.is_err());
    assert_eq!(t.zone(t.g.current(colossus)), Zone::Library(P0));
    // The Colossus left the battlefield, which stays; its being sacrificed was the
    // payment, which is canceled (CR 733.1: no abilities trigger from an undone action).
    let sources: Vec<ObjectId> = t.g.pending_triggers.iter().map(|p| p.source).collect();
    assert_eq!(sources, vec![leaves]);
    assert!(!sources.contains(&sacrificed));
    t.settle();
    t.resolve_all();
    assert_eq!(t.g.player(P0).life, 21);
}

#[test]
fn a_library_card_revealed_while_attempting_an_action_stays_known() {
    cr!("733.1", "701.20a", "605.1b");
    interp!("illegal-action-reversal-restores-libraries");
    let mut t = TestGame::new(2);
    t.g.event_feed.enable();
    // A triggered mana ability that reveals a library card (CR 605.1b).
    let insight = custom_card(
        "Glimmering Insight",
        "Enchantment",
        None,
        "Whenever you tap a creature for mana, add an additional {G} and reveal the top card of your library.",
    );
    t.custom(P0, insight, Zone::Battlefield);
    // "{T}: Add {W}." — P0's only creature.
    let pilgrim = t.battlefield(P0, "Avacyn's Pilgrim");
    let mut def = custom_card(
        "Pilgrim Toll",
        "Instant",
        None,
        "As an additional cost to cast this spell, tap an untapped creature you control.\nYou gain 1 life.",
    );
    let cost = ManaCost::parse("{W}").unwrap();
    def.faces[0].chars.colors = cost.colors();
    def.faces[0].chars.mana_cost = Some(cost);
    let spell = t.custom(P0, def, Zone::Hand(P0));
    let top = t.g.library_top(P0).unwrap();
    let library = t.g.player(P0).library.clone();
    // Where a reader of the event feed was during the attempt (just after its reveal).
    let seen: Arc<Mutex<Option<usize>>> = Arc::default();
    let s2 = seen.clone();
    t.g.observer = Some(EventObserver::new(move |g, ev| {
        if matches!(ev, Event::Custom { name, .. } if name == reveal::REVEALED) {
            s2.lock().unwrap().get_or_insert(g.event_feed.len());
        }
    }));
    let r = t.cast(P0, spell).method(CastMethod::Normal).try_go();
    assert!(r.is_err());
    let cursor = seen.lock().unwrap().expect("the attempt revealed the card");
    // The card is revealed again on the restored game: the players know it, and a reader
    // of the event feed that read the attempt's events reads that reveal next (the feed's
    // positions don't go back).
    assert!(t.g.event_feed.len() > cursor);
    let revealed: Vec<ObjectId> =
        t.g.event_feed
            .since(cursor)
            .iter()
            .filter_map(|e| match e {
                Event::Custom {
                    name, obj: Some(o), ..
                } if name == reveal::REVEALED => Some(*o),
                _ => None,
            })
            .collect();
    assert_eq!(revealed, vec![top]);
    // Revealing doesn't move it; the rest is undone.
    assert_eq!(t.g.player(P0).library, library);
    assert!(!t.g.obj(pilgrim).tapped);
    assert!(t.g.player(P0).mana_pool.mana.is_empty());
    assert_eq!(t.zone(spell), Zone::Hand(P0));
}

#[test]
fn the_rollback_property_accepts_a_reversal_that_kept_library_actions() {
    cr!("733.1");
    interp!("illegal-action-reversal-restores-libraries");
    use mtg_engine::properties::agent::{rollback_view, undone_difference};
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ashnod's Altar");
    t.battlefield(P0, "Darksteel Colossus");
    let mountain = t.lands(P0, "Mountain", 1)[0];
    let fling = t.hand(P0, "Fling");
    let before = t.g.clone();
    let r = t
        .cast(P0, fling)
        .target(Entity::Player(P1))
        .method(CastMethod::Normal)
        .try_go();
    assert!(r.is_err());
    // The game differs (the Colossus is in the library), but not in what the reversal
    // must undo (docs/PROPERTIES.md, rollback identity).
    assert!(t.g.reversals.kept > before.reversals.kept);
    assert_ne!(rollback_view(&before), rollback_view(&t.g));
    assert_eq!(undone_difference(&before, &t.g), None);
    // A payment left in place would be noticed.
    let mut wrong = t.g.clone();
    wrong.objects[mountain.0 as usize].tapped = true;
    assert!(undone_difference(&before, &wrong).is_some());
    let mut wrong = t.g.clone();
    wrong.player_mut(P0).life -= 1;
    assert!(undone_difference(&before, &wrong).is_some());
}

#[test]
fn a_reversal_without_library_actions_restores_the_game_exactly() {
    cr!("733.1");
    interp!("reversal-undoes-mana-abilities");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let spell = creature_toll(&mut t, "{R}");
    let before = format!("{:?}", t.g.players);
    let objects = t.g.objects.len();
    let r = t.cast(P0, spell).method(CastMethod::Normal).try_go();
    assert!(r.is_err());
    assert_eq!(format!("{:?}", t.g.players), before);
    assert_eq!(t.g.objects.len(), objects);
    assert_eq!(t.g.reversals.kept, 0);
}
