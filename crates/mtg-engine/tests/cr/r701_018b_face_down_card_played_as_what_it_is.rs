//! CR 701.18b: "To play a card means to play that card as a land or to cast that card as
//! a spell, whichever is appropriate." A face-down card outside the battlefield that an
//! effect lets its player play (Praetor's Grasp) is played face up as what it is: a land
//! card as a land (one of the turn's land plays, CR 305.1, 701.18a), a nonland card as a
//! spell, never as a land.

use mtg_engine::decision::Action;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Casts Praetor's Grasp on P1, exiling the card `name` from the top of P1's library face
/// down, and gives P0 priority in their main phase with the exiled card's id.
fn grasp(t: &mut TestGame, name: &str) -> ObjectId {
    let card = t.library_top(P1, name);
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Forest", 2);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Object(card)]);
    let spell = t.hand(P0, "Praetor's Grasp");
    t.cast(P0, spell).target(P1).go();
    t.resolve_all();
    let card = t.g.current(card);
    assert_eq!(t.zone(card), Zone::Exile);
    assert!(t.g.obj(card).face_down);
    t.g.turn.priority = Some(P0);
    card
}

#[test]
fn a_face_down_exiled_nonland_card_is_cast_not_played_as_a_land() {
    cr!("701.18b", "305.1");
    ruling!(
        "Praetor's Grasp",
        "Playing a card exiled with Praetor’s Grasp follows all the normal rules for playing that card. You must pay its costs, and you must follow all timing restrictions, for example."
    );
    let mut t = TestGame::new(2);
    let bears = grasp(&mut t, "Grizzly Bears");
    let actions = t.g.legal_actions(P0);
    assert!(
        !actions.contains(&Action::PlayLand { card: bears }),
        "a creature card isn't a land to play: {actions:?}"
    );
    assert!(actions
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == bears)));
    assert!(t.g.play_land(P0, bears).is_err());
    assert_eq!(t.player(P0).lands_played_this_turn, 0);
    assert_eq!(t.zone(bears), Zone::Exile);
    // Cast, it's paid for ({1}{G}: the two Forests) and enters face up.
    t.cast(P0, bears).go();
    assert_eq!(
        t.g.battlefield
            .iter()
            .filter(|id| t.g.obj(**id).controller == P0
                && t.g.obj(**id).is(CardType::Land)
                && !t.g.obj(**id).tapped)
            .count(),
        0
    );
    t.resolve();
    let bears = t.g.current(bears);
    assert!(t.on_battlefield(bears));
    assert!(!t.obj_now(bears).face_down);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn a_face_down_exiled_land_card_is_played_as_a_land_face_up() {
    cr!("701.18b", "701.18a", "305.1");
    ruling!(
        "Praetor's Grasp",
        "Playing a card exiled with Praetor’s Grasp follows all the normal rules for playing that card. You must pay its costs, and you must follow all timing restrictions, for example."
    );
    let mut t = TestGame::new(2);
    let island = grasp(&mut t, "Island");
    let actions = t.g.legal_actions(P0);
    assert!(actions.contains(&Action::PlayLand { card: island }));
    assert!(!actions
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == island)));
    assert!(t.g.play_land(P0, island).is_ok());
    let island = t.g.current(island);
    assert!(t.on_battlefield(island));
    assert!(!t.obj_now(island).face_down);
    assert!(t.obj_now(island).is(CardType::Land));
    assert_eq!(t.player(P0).lands_played_this_turn, 1);
    // The turn's land play is used up: a second land can't be played.
    let forest = t.hand(P0, "Forest");
    assert!(t.g.play_land(P0, forest).is_err());
}
