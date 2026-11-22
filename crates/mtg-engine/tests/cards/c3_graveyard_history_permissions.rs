//! Permissions over graveyard cards qualified by how they got there this turn
//! (cards3-cast-zones): "milled this turn" (Raul, Trouble Shooter), "put there from your
//! library this turn" (Kagha, Shadow Archdruid), "put there from anywhere other than the
//! battlefield this turn" (Banon, the Returners' Leader), "a land and a spell from among
//! cards in graveyards that were put there from libraries this turn" (Coram, the
//! Undertaker), "cards in your graveyard you've surveilled this turn" (Eye of Duskmantle).

use mtg_engine::decision::Answer;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn can_cast(t: &mut TestGame, p: PlayerId, card: ObjectId, targets: &[Entity]) -> bool {
    can_cast_as(t, p, card, targets, CastMethod::Normal)
}

/// For the life the permission's alternative cost requires (CR 118.9b).
const PAY_LIFE: CastMethod = CastMethod::Alternative(mtg_engine::casting::PERMISSION_COST);

fn can_cast_as(
    t: &mut TestGame,
    p: PlayerId,
    card: ObjectId,
    targets: &[Entity],
    method: CastMethod,
) -> bool {
    let c = t.g.current(card);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.g.turn.priority = Some(p);
    let ok = t.g.cast_spell(p, c, method).is_ok();
    t.g.flush_events();
    if ok {
        t.resolve_all();
    } else {
        t.clear_answers();
    }
    ok
}

#[test]
fn raul_casts_one_spell_milled_this_turn_each_of_your_turns() {
    cr!("601.3", "701.17a");
    ruling!(
        "Raul, Trouble Shooter",
        "You must follow the normal timing permissions and restrictions of the spell you cast from your graveyard."
    );
    assert_supported("Raul, Trouble Shooter");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let raul = t.battlefield(P0, "Raul, Trouble Shooter");
    t.lands(P0, "Mountain", 2);
    let old = t.graveyard(P0, "Shock");
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.activate(P0, raul, 0, &[]).unwrap();
    t.resolve_all();
    let bolt_x = t.g.current(bolt);
    assert_eq!(t.zone(bolt_x), Zone::Graveyard(P0));
    // A card that was already in the graveyard: no.
    assert!(!can_cast(&mut t, P0, old, &[Entity::Player(P1)]));
    assert!(can_cast(&mut t, P0, bolt_x, &[Entity::Player(P1)]));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn raul_permission_is_used_once_each_turn_and_not_for_other_ways_to_the_graveyard() {
    cr!("601.3");
    ruling!(
        "Raul, Trouble Shooter",
        "If you are instructed to put a card into your graveyard from your library without using the word “mill,”"
    );
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Raul, Trouble Shooter");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Island", 2);
    let shock = t.library_top(P0, "Shock");
    let bolt = t.library_top(P0, "Lightning Bolt");
    let scour = t.hand(P0, "Thought Scour");
    t.cast(P0, scour).target(P0).go();
    t.resolve_all();
    // Both milled; one may be cast.
    assert!(can_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
    assert!(!can_cast(&mut t, P0, shock, &[Entity::Player(P1)]));
    // A card surveilled into the graveyard isn't milled.
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Raul, Trouble Shooter");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Island", 1);
    let bolt = t.library_top(P0, "Lightning Bolt");
    let consider = t.hand(P0, "Consider");
    t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![], vec![bolt]));
    t.cast(P0, consider).go();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Graveyard(P0));
    assert!(!can_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
}

#[test]
fn kagha_plays_a_land_or_casts_a_permanent_spell_put_there_from_your_library() {
    cr!("601.3", "305.1");
    ruling!(
        "Kagha, Shadow Archdruid",
        "You must follow the normal timing rules of the card you play from your graveyard."
    );
    assert_supported("Kagha, Shadow Archdruid");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let kagha = t.battlefield(P0, "Kagha, Shadow Archdruid");
    t.lands(P0, "Forest", 2);
    let bears = t.library_top(P0, "Grizzly Bears");
    let forest = t.library_top(P0, "Forest");
    let old = t.graveyard(P0, "Forest");
    t.attack(&[(kagha, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    let forest_x = t.g.current(forest);
    assert_eq!(t.zone(forest_x), Zone::Graveyard(P0));
    // Not a card put there on an earlier turn.
    assert!(t.play_land(P0, old).is_err());
    // A land, then nothing more this turn (once).
    t.play_land(P0, forest_x).expect("play the milled Forest");
    assert!(!can_cast(&mut t, P0, bears, &[]));
}

#[test]
fn kagha_doesnt_cast_nonpermanent_spells() {
    cr!("601.3");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let kagha = t.battlefield(P0, "Kagha, Shadow Archdruid");
    t.lands(P0, "Mountain", 1);
    let bolt = t.library_top(P0, "Lightning Bolt");
    t.library_top(P0, "Forest");
    t.attack(&[(kagha, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Graveyard(P0));
    assert!(!can_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
}

#[test]
fn banon_casts_a_creature_spell_put_there_from_anywhere_but_the_battlefield() {
    cr!("601.3");
    ruling!(
        "Banon, the Returners' Leader",
        "You pay all costs and follow all timing rules for cards played with the permission granted by Banon's first ability."
    );
    assert_supported("Banon, the Returners' Leader");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Banon, the Returners' Leader");
    t.lands(P0, "Forest", 4);
    // A creature that died this turn: no.
    let died = t.battlefield(P0, "Grizzly Bears");
    t.g.destroy(died, None);
    t.settle();
    assert!(!can_cast(&mut t, P0, died, &[]));
    // A creature card discarded this turn: yes.
    let discarded = t.hand(P0, "Grizzly Bears");
    t.g.discard(P0, discarded, None);
    t.settle();
    assert_eq!(t.zone(t.g.current(discarded)), Zone::Graveyard(P0));
    assert!(can_cast(&mut t, P0, discarded, &[]));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn coram_plays_a_land_and_casts_a_spell_from_cards_milled_this_turn() {
    cr!("601.3", "305.1");
    ruling!(
        "Coram, the Undertaker",
        "You pay all costs and follow all timing rules for lands played and spells cast this way."
    );
    assert_supported("Coram, the Undertaker");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    let coram = t.battlefield(P0, "Coram, the Undertaker");
    t.lands(P0, "Mountain", 2);
    let bolt = t.library_top(P1, "Lightning Bolt");
    let forest = t.library_top(P0, "Forest");
    t.attack(&[(coram, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    // A land from your graveyard and a spell from an opponent's.
    let forest_x = t.g.current(forest);
    t.play_land(P0, forest_x).expect("play the milled Forest");
    assert!(can_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
    // Its owner's graveyard.
    assert!(t.in_graveyard(P1, "Lightning Bolt"));
}

#[test]
fn coram_one_spell_each_turn() {
    cr!("601.3");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Coram, the Undertaker");
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Island", 1);
    let shock = t.library_top(P1, "Shock");
    let bolt = t.library_top(P1, "Lightning Bolt");
    let scour = t.hand(P0, "Thought Scour");
    t.cast(P0, scour).target(P1).go();
    t.resolve_all();
    assert!(can_cast(&mut t, P0, bolt, &[Entity::Player(P1)]));
    assert!(!can_cast(&mut t, P0, shock, &[Entity::Player(P1)]));
}

#[test]
fn eye_of_duskmantle_casts_surveilled_cards_for_life() {
    cr!("601.3", "118.9", "701.25a");
    ruling!(
        "Eye of Duskmantle",
        "You may only cast spells this way by paying the appropriate amount of life."
    );
    ruling!(
        "Eye of Duskmantle",
        "“Cards in your graveyard you’ve surveilled this turn” includes cards you put into your graveyard while surveilling."
    );
    assert_supported("Eye of Duskmantle");
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Eye of Duskmantle");
    t.lands(P0, "Island", 1);
    let bears = t.library_top(P0, "Grizzly Bears");
    let consider = t.hand(P0, "Consider");
    t.answer(P0, DecisionKind::Surveil, Answer::Split(vec![], vec![bears]));
    t.cast(P0, consider).go();
    t.resolve_all();
    let bears_x = t.g.current(bears);
    assert_eq!(t.zone(bears_x), Zone::Graveyard(P0));
    // Not for its mana cost; for 2 life.
    t.lands(P0, "Forest", 2);
    assert!(!can_cast(&mut t, P0, bears_x, &[]));
    assert!(can_cast_as(&mut t, P0, bears_x, &[], PAY_LIFE));
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn eye_of_duskmantle_doesnt_cast_cards_seen_while_surveilling_and_milled_later() {
    cr!("601.3");
    ruling!(
        "Eye of Duskmantle",
        "It does not include cards you saw while surveilling, left on top of your library, and then put into your graveyard later that turn by other means."
    );
    let mut t = TestGame::new(2);
    t.advance_to(P0, Step::PrecombatMain);
    t.battlefield(P0, "Eye of Duskmantle");
    t.lands(P0, "Island", 2);
    let shock = t.library_top(P0, "Shock");
    let bolt = t.library_top(P0, "Lightning Bolt");
    let bears = t.library_top(P0, "Grizzly Bears");
    let gaze = t.hand(P0, "Otherworldly Gaze");
    let scour = t.hand(P0, "Thought Scour");
    // Surveil 3: the Bears into the graveyard, the Bolt and the Shock left on top.
    t.answer(
        P0,
        DecisionKind::Surveil,
        Answer::Split(vec![bolt, shock], vec![bears]),
    );
    t.cast(P0, gaze).go();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bears)), Zone::Graveyard(P0));
    // Then milled: not surveilled into the graveyard.
    t.cast(P0, scour).target(P0).go();
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bolt)), Zone::Graveyard(P0));
    assert!(!can_cast_as(&mut t, P0, bolt, &[Entity::Player(P1)], PAY_LIFE));
    assert!(can_cast_as(&mut t, P0, bears, &[], PAY_LIFE));
    assert_eq!(t.life(P0), 18);
}
