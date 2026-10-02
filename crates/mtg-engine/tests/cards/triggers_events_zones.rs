//! Zone-change trigger events (CR 603.6, 603.10a, 700.4): "is put into [a / your / an
//! opponent's / a player's] graveyard [from ...]", "is put into exile from ...", "is put
//! into a library from anywhere", "is returned to [your / a player's] hand", "leaves an
//! opponent's graveyard", "leaves the battlefield without dying", and subjects such as
//! "another creature you control or a land you control", "~ dies or another artifact
//! ...".

use mtg_engine::ability::LibraryPosition;
use mtg_engine::events::MoveCause;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

/// Moves several objects at the same time (one event batch, CR 603.2c).
fn move_all(t: &mut TestGame, objs: &[ObjectId], to: impl Fn(PlayerId) -> Zone, cause: MoveCause) {
    let moves = objs
        .iter()
        .map(|o| MoveEv {
            obj: *o,
            to: to(t.g.obj(*o).owner),
            pos: LibraryPosition::Top,
            cause,
            by: Some(P0),
            etb: EtbInfo::default(),
            source: None,
        })
        .collect();
    t.g.move_objects(moves);
}

fn count_subtype(t: &TestGame, subtype: &str) -> usize {
    t.g.battlefield
        .iter()
        .filter(|id| t.g.obj(**id).chars.has_subtype(subtype))
        .count()
}

#[test]
fn a_permanent_is_put_into_an_opponents_graveyard() {
    cr!("700.4", "603.10a");
    supported("Patron of the Nezumi");
    ruling!(
        "Patron of the Nezumi",
        "triggers even if an event puts this and multiple other permanents"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Patron of the Nezumi");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(mine, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "not an opponent's graveyard");
    t.g.destroy(theirs, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Wrath of God: it sees the opponent's creature die with it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Patron of the Nezumi");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn a_nontoken_permanent_is_put_into_a_players_graveyard() {
    cr!("700.4");
    supported("Liability");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Liability");
    let bear = t.battlefield(P1, "Grizzly Bears");
    t.g.destroy(bear, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 19, "that player: the graveyard's owner");
    assert_eq!(t.life(P0), 20);
    // A token isn't a nontoken permanent.
    let goblin = t.hand(P0, "Dragon Fodder");
    t.lands(P0, "Mountain", 2);
    t.cast(P0, goblin).go();
    t.resolve_all();
    let tokens = t.named_on_battlefield("Goblin Token");
    let tokens = if tokens.is_empty() {
        t.g.battlefield
            .iter()
            .copied()
            .filter(|id| t.g.obj(*id).chars.has_subtype("Goblin"))
            .collect()
    } else {
        tokens
    };
    assert_eq!(tokens.len(), 2);
    t.g.destroy(tokens[0], None);
    t.resolve_all();
    // (Dragon Fodder went to the graveyard from the stack.)
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_permanent_owned_by_another_player_dies() {
    cr!("700.4", "108.3");
    supported("Kothophed, Soul Hoarder");
    ruling!(
        "Kothophed, Soul Hoarder",
        "It doesn't matter who controlled the permanent"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kothophed, Soul Hoarder");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    // Gain control of it first: it's still owned by the other player.
    t.lands(P0, "Mountain", 3);
    let treason = t.hand(P0, "Act of Treason");
    t.cast(P0, treason).target(theirs).go();
    t.resolve_all();
    assert_eq!(t.obj_now(theirs).controller, P0);
    let hand = t.hand_size(P0);
    t.g.destroy(t.g.current(theirs), None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 19);
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(mine, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
}

#[test]
fn another_creature_or_a_land_you_control_dies() {
    cr!("700.4");
    supported("Long Feng, Grand Secretariat");
    let mut t = TestGame::new(2);
    let feng = t.battlefield(P0, "Long Feng, Grand Secretariat");
    let land = t.battlefield(P0, "Forest");
    let theirs = t.battlefield(P1, "Forest");
    t.answer_targets(P0, &[Entity::Object(feng)]);
    t.g.destroy(land, None);
    t.resolve_all();
    assert_eq!(t.counters(feng, "+1/+1"), 1);
    t.g.destroy(theirs, None);
    t.resolve_all();
    assert_eq!(t.counters(feng, "+1/+1"), 1, "an opponent's land");
}

#[test]
fn this_dies_or_another_artifact_is_put_into_a_graveyard() {
    cr!("700.4", "603.10a");
    supported("Scrap Trawler");
    ruling!("Scrap Trawler", "triggers for each of them");
    let mut t = TestGame::new(2);
    let trawler = t.battlefield(P0, "Scrap Trawler");
    let ring = t.battlefield(P0, "Sol Ring");
    let a = t.graveyard(P0, "Ornithopter");
    let b = t.graveyard(P0, "Ornithopter");
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.g.destroy_all(vec![trawler, ring], None, false);
    t.resolve_all();
    // Each trigger returns an artifact card with lesser mana value: both Ornithopters.
    let in_hand = t.g.find_in_zone(Zone::Hand(P0), "Ornithopter").len();
    assert_eq!(in_hand, 2);
    // Sol Ring's own trigger can't return a card with mana value 1 (not lesser).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Scrap Trawler");
    let ring = t.battlefield(P0, "Sol Ring");
    let other = t.graveyard(P0, "Sol Ring");
    t.answer_targets(P0, &[Entity::Object(other)]);
    t.g.destroy(ring, None);
    t.resolve_all();
    assert!(!t.in_hand(P0, "Sol Ring"));
}

#[test]
fn artifacts_into_your_graveyard_from_the_battlefield_or_elsewhere() {
    cr!("700.4", "701.17a");
    supported("Ultron's Auxiliary");
    let mut t = TestGame::new(2);
    let aux = t.battlefield(P0, "Ultron's Auxiliary");
    let ring = t.battlefield(P0, "Sol Ring");
    t.g.destroy(ring, None);
    t.resolve_all();
    assert_eq!(t.counters(aux, "+1/+1"), 1);
    t.library_top(P0, "Sol Ring");
    t.g.mill(P0, 1);
    t.resolve_all();
    assert_eq!(t.counters(aux, "+1/+1"), 2);
    let card = t.hand(P0, "Sol Ring");
    t.g.discard(P0, card, None);
    t.resolve_all();
    assert_eq!(t.counters(aux, "+1/+1"), 3);
    // A nonartifact card: no.
    t.library_top(P0, "Island");
    t.g.mill(P0, 1);
    t.resolve_all();
    assert_eq!(t.counters(aux, "+1/+1"), 3);
}

#[test]
fn a_creature_dies_or_a_creature_card_is_put_into_a_graveyard_from_a_library() {
    cr!("700.4", "701.17a");
    supported("Dreadhound");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dreadhound");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bear, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    t.library_top(P1, "Grizzly Bears");
    t.g.mill(P1, 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    let card = t.hand(P1, "Grizzly Bears");
    t.g.discard(P1, card, None);
    t.resolve_all();
    assert_eq!(t.life(P1), 18, "not from a library");
}

#[test]
fn put_into_your_graveyard_from_anywhere_other_than_the_battlefield() {
    cr!("603.6c");
    supported("Disa the Restless");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Disa the Restless");
    let goyf = t.library_top(P0, "Tarmogoyf");
    t.g.mill(P0, 1);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Tarmogoyf").len(), 1);
    let _ = goyf;
    // From the battlefield: it stays in the graveyard.
    let goyf = t.named_on_battlefield("Tarmogoyf")[0];
    t.g.destroy(goyf, None);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Tarmogoyf"));
}

#[test]
fn one_or_more_cards_put_into_exile_from_your_graveyard() {
    cr!("603.2c");
    supported("Rakshasa Vizier");
    let mut t = TestGame::new(2);
    let vizier = t.battlefield(P0, "Rakshasa Vizier");
    let a = t.graveyard(P0, "Island");
    let b = t.graveyard(P0, "Island");
    move_all(&mut t, &[a, b], |_| Zone::Exile, MoveCause::Effect);
    t.resolve_all();
    assert_eq!(t.counters(vizier, "+1/+1"), 2, "that many");
    // From the hand: no.
    let c = t.hand(P0, "Island");
    move_all(&mut t, &[c], |_| Zone::Exile, MoveCause::Effect);
    t.resolve_all();
    assert_eq!(t.counters(vizier, "+1/+1"), 2);
}

#[test]
fn one_or_more_cards_put_into_exile_from_your_library_and_or_graveyard() {
    cr!("603.2c");
    supported("Laelia, the Blade Reforged");
    ruling!(
        "Laelia, the Blade Reforged",
        "no matter how many cards were exiled at the same time"
    );
    let mut t = TestGame::new(2);
    let laelia = t.battlefield(P0, "Laelia, the Blade Reforged");
    let a = t.graveyard(P0, "Island");
    let b = t.library_top(P0, "Island");
    move_all(&mut t, &[a, b], |_| Zone::Exile, MoveCause::Effect);
    t.resolve_all();
    assert_eq!(t.counters(laelia, "+1/+1"), 1);
}

#[test]
fn one_or_more_cards_put_into_a_library_from_anywhere() {
    cr!("603.2c");
    supported("Dutiful Knowledge Seeker");
    let mut t = TestGame::new(2);
    let seeker = t.battlefield(P0, "Dutiful Knowledge Seeker");
    let a = t.graveyard(P1, "Island");
    let b = t.hand(P0, "Island");
    move_all(&mut t, &[a, b], Zone::Library, MoveCause::Effect);
    t.resolve_all();
    assert_eq!(t.counters(seeker, "+1/+1"), 1);
}

#[test]
fn a_permanent_is_returned_to_your_hand() {
    cr!("603.10a");
    supported("Azorius Aethermage");
    ruling!(
        "Azorius Aethermage",
        "including a token creature or Azorius Aethermage itself"
    );
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Azorius Aethermage");
    t.lands(P0, "Island", 2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    move_all(&mut t, &[bear], Zone::Hand, MoveCause::Return);
    t.resolve_all();
    // The Bears and a card drawn.
    assert_eq!(t.hand_size(P0), hand + 2);
    // An opponent's permanent returned to their hand: no.
    let theirs = t.battlefield(P1, "Grizzly Bears");
    move_all(&mut t, &[theirs], Zone::Hand, MoveCause::Return);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Itself.
    t.answer_yes(P0, true);
    move_all(&mut t, &[mage], Zone::Hand, MoveCause::Return);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 4);
}

#[test]
fn a_permanent_is_returned_to_a_players_hand() {
    cr!("603.10a");
    supported("Warped Devotion");
    ruling!("Warped Devotion", "can trigger on itself being returned");
    let mut t = TestGame::new(2);
    let devotion = t.battlefield(P0, "Warped Devotion");
    t.hand(P1, "Island");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    move_all(&mut t, &[theirs], Zone::Hand, MoveCause::Return);
    t.resolve_all();
    // That player (the owner) discards a card.
    assert_eq!(t.graveyard_size(P1), 1);
    assert_eq!(t.hand_size(P1), 1);
    move_all(&mut t, &[devotion], Zone::Hand, MoveCause::Return);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0, "returned it, then discarded it");
}

#[test]
fn this_or_another_creature_returned_to_your_hand() {
    cr!("603.10a");
    supported("Stormfront Riders");
    ruling!(
        "Stormfront Riders",
        "returned to your hand at the same time, its last ability triggers for each"
    );
    let mut t = TestGame::new(2);
    let riders = t.battlefield(P0, "Stormfront Riders");
    let bear = t.battlefield(P0, "Grizzly Bears");
    move_all(&mut t, &[riders, bear], Zone::Hand, MoveCause::Return);
    t.resolve_all();
    assert_eq!(count_subtype(&t, "Soldier"), 2);
}

#[test]
fn noncreature_permanents_returned_to_hand() {
    cr!("603.2c");
    supported("Tameshi, Reality Architect");
    ruling!(
        "Tameshi, Reality Architect",
        "returned to hand from other zones do not cause this ability to trigger"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tameshi, Reality Architect");
    let ring = t.graveyard(P0, "Sol Ring");
    let hand = t.hand_size(P0);
    move_all(&mut t, &[ring], Zone::Hand, MoveCause::Return);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "from the graveyard: no draw");
    let a = t.battlefield(P1, "Sol Ring");
    let b = t.battlefield(P0, "Forest");
    let bear = t.battlefield(P0, "Grizzly Bears");
    move_all(&mut t, &[a, b, bear], Zone::Hand, MoveCause::Return);
    t.resolve_all();
    // Forest and Bears to my hand, plus one card drawn.
    assert_eq!(t.hand_size(P0), hand + 4);
}

#[test]
fn a_creature_card_leaves_an_opponents_graveyard() {
    cr!("603.10a", "113.6");
    supported("Erebos's Titan");
    ruling!(
        "Erebos's Titan",
        "triggers only if Erebos's Titan is in your graveyard"
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Erebos's Titan");
    t.hand(P0, "Island");
    let theirs = t.graveyard(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.g.exile_object(theirs, None);
    t.resolve_all();
    assert!(t.in_hand(P0, "Erebos's Titan"));
    // On the battlefield: no.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Erebos's Titan");
    t.hand(P0, "Island");
    let theirs = t.graveyard(P1, "Grizzly Bears");
    t.g.exile_object(theirs, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn another_creature_leaves_the_battlefield_without_dying() {
    cr!("603.6c", "700.4");
    supported("Imperial Cosmographer");
    let mut t = TestGame::new(2);
    let cosmo = t.battlefield(P0, "Imperial Cosmographer");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.g.exile_object(a, None);
    t.resolve_all();
    assert_eq!(t.counters(cosmo, "+1/+1"), 2);
    t.g.destroy(b, None);
    t.resolve_all();
    assert_eq!(t.counters(cosmo, "+1/+1"), 2, "it died");
}

#[test]
fn one_or_more_creatures_leave_without_dying() {
    cr!("603.6c", "603.2c", "603.10a");
    supported("Dour Port-Mage");
    ruling!("Dour Port-Mage", "its first ability will still trigger");
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Dour Port-Mage");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    move_all(&mut t, &[mage, a, b], Zone::Hand, MoveCause::Return);
    t.resolve_all();
    assert_eq!(
        t.hand_size(P0),
        hand + 3 + 1,
        "one card drawn for the batch"
    );
}

#[test]
fn this_or_another_creature_leaves_without_dying() {
    cr!("603.6c", "603.10a");
    supported("Three Tree Scribe");
    ruling!("Three Tree Scribe", "including itself");
    let mut t = TestGame::new(2);
    let scribe = t.battlefield(P0, "Three Tree Scribe");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(other)]);
    t.answer_targets(P0, &[Entity::Object(other)]);
    move_all(&mut t, &[scribe, bear], |_| Zone::Exile, MoveCause::Exile);
    t.resolve_all();
    assert_eq!(t.counters(other, "+1/+1"), 2);
}

#[test]
fn one_or_more_creature_cards_into_your_graveyard_during_your_turn() {
    cr!("603.2c", "603.6c");
    supported("Crawling Infestation");
    let insects = |t: &TestGame| count_subtype(t, "Insect");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Crawling Infestation");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(bear, None);
    t.resolve_all();
    assert_eq!(insects(&t), 1, "a creature card from the battlefield");
    let card = t.hand(P0, "Grizzly Bears");
    t.g.discard(P0, card, None);
    t.resolve_all();
    assert_eq!(insects(&t), 1, "only once each turn");
    // "From anywhere" is never a leaves-the-battlefield ability: it doesn't look back in
    // time, so destroyed together with the creature it doesn't see the creature card
    // arrive.
    let mut t = TestGame::new(2);
    let infestation = t.battlefield(P0, "Crawling Infestation");
    let bear = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy_all(vec![infestation, bear], None, false);
    t.resolve_all();
    assert_eq!(insects(&t), 0);
}
