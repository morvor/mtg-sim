//! CR 314: schemes (the Archenemy variant) — nontraditional cards that stay in the
//! command zone, their abilities while face up, ownership and control, returning to the
//! scheme deck, and "this scheme".

use crate::r100_common::{fillers, pregame};
use crate::r300_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::variants;
use mtg_engine::*;

fn set_in_motion(t: &mut TestGame) {
    keyword_action(t, P0, KeywordAction::SetInMotion, 1);
}

fn real(name: &str) -> CardDef {
    (*card(name)).clone()
}

#[test]
fn schemes_are_nontraditional_cards_used_in_archenemy() {
    cr!("314.1");
    assert!(variants::is_nontraditional(&card("Roots of All Evil")));
    // In an Archenemy game the archenemy sets schemes in motion; in another game, no one
    // does.
    let mut t = archenemy_game();
    add_scheme_deck(&mut t, P0, vec![real("Roots of All Evil")]);
    set_in_motion(&mut t);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Saproling Token").len(), 5);
    let mut t = TestGame::new(2);
    add_scheme_deck(&mut t, P0, vec![real("Roots of All Evil")]);
    set_in_motion(&mut t);
    t.resolve_all();
    assert!(t.named_on_battlefield("Saproling Token").is_empty());
}

#[test]
fn scheme_cards_remain_in_the_command_zone() {
    cr!("314.2");
    let mut t = archenemy_game();
    let deck = add_scheme_deck(
        &mut t,
        P0,
        vec![real("Roots of All Evil"), real("Look Skyward and Despair")],
    );
    set_in_motion(&mut t);
    // Face up or in the scheme deck: not permanents, can't be cast, and they stay in the
    // command zone if they would leave it.
    for id in deck.clone() {
        assert_eq!(t.zone(id), Zone::Command);
        assert!(t.g.permanents().all(|o| o.id != id));
        t.g.turn.priority = Some(P0);
        assert!(t.g.cast_spell(P0, id, CastMethod::Normal).is_err());
        run_effect(
            &mut t,
            P1,
            None,
            Effect::Move {
                what: Sel::Target(0),
                to: Destination::zone(ZoneKind::Hand),
            },
            &[Entity::Object(id)],
        );
        assert_eq!(t.g.current(id), id);
        assert_eq!(t.zone(id), Zone::Command);
    }
    assert_eq!(variants::face_up_schemes(&t.g), vec![deck[0]]);
}

#[test]
fn schemes_have_no_subtypes() {
    cr!("314.3");
    for s in ["Roots of All Evil", "The Very Soil Shall Shake"] {
        assert!(subtypes_of(s).is_empty(), "{s}");
    }
}

#[test]
fn a_face_up_schemes_abilities_function_from_the_command_zone() {
    cr!("314.4");
    let mut t = archenemy_game();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let tapper = oracle_card(
        "Endless Toll",
        "Ongoing Scheme",
        "",
        None,
        "{1}: You gain 1 life.",
    );
    let deck = add_scheme_deck(
        &mut t,
        P0,
        vec![real("The Very Soil Shall Shake"), tapper],
    );
    // In the scheme deck, face down: no effect.
    assert_eq!(t.pt(bears), (2, 2));
    // Face up: its static ability applies.
    set_in_motion(&mut t);
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    assert!(t.obj(bears).has_keyword(mtg_engine::keywords::KeywordKind::Trample));
    // Its activated abilities can be activated (and a face-down one's can't).
    t.lands(P0, "Wastes", 2);
    t.g.turn.priority = Some(P0);
    assert!(t
        .g
        .activatable_abilities(P0)
        .iter()
        .all(|(src, _)| *src != deck[1]));
    set_in_motion(&mut t);
    t.activate(P0, deck[1], 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 41);
    // Its triggered abilities trigger: "When a creature you control dies, abandon this
    // scheme."
    t.g.destroy(bears, None);
    t.settle();
    t.resolve_all();
    assert!(t.obj(deck[0]).face_down);
}

#[test]
fn a_scheme_is_owned_and_controlled_by_the_player_who_started_with_it() {
    cr!("314.5");
    let mut deck = fillers(40);
    deck.push(card("The Very Soil Shall Shake"));
    let mut t = pregame(
        GameConfig {
            variant: Variant::Archenemy,
            teams: Some(vec![0, 1, 1]),
            skip_mulligans: true,
            ..Default::default()
        },
        vec![deck, fillers(40), fillers(40)],
    );
    t.g.start();
    let scheme = variants::scheme_deck(&t.g, P0)[0];
    assert_eq!((t.obj(scheme).owner, t.obj(scheme).controller), (P0, P0));
    // Set in motion, then an opponent tries to gain control of it.
    t.set_step(P0, Step::PrecombatMain);
    set_in_motion(&mut t);
    run_effect(
        &mut t,
        P1,
        None,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[Entity::Object(scheme)],
    );
    assert_eq!(t.obj(scheme).controller, P0);
    let archenemy_bears = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(archenemy_bears), (4, 4));
    assert_eq!(t.pt(theirs), (2, 2));
}

#[test]
fn a_non_ongoing_scheme_returns_to_the_bottom_of_the_scheme_deck() {
    cr!("314.6");
    let mut t = archenemy_game();
    let deck = add_scheme_deck(
        &mut t,
        P0,
        vec![real("Look Skyward and Despair"), real("Roots of All Evil")],
    );
    set_in_motion(&mut t);
    // While its "set in motion" ability waits, it stays face up.
    t.settle();
    assert!(!t.obj(deck[0]).face_down);
    t.resolve();
    // Then it's turned face down and put on the bottom of its owner's scheme deck.
    assert!(t.obj(deck[0]).face_down);
    assert_eq!(variants::scheme_deck(&t.g, P0), vec![deck[1], deck[0]]);
    assert_eq!(t.named_on_battlefield("Dragon Token").len(), 1);
}

#[test]
fn this_scheme_means_the_scheme_card_that_is_the_abilitys_source() {
    cr!("314.7");
    let mut t = archenemy_game();
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Two face-up ongoing schemes; only the one whose ability resolves is abandoned.
    let other = oracle_card(
        "Other Plot",
        "Ongoing Scheme",
        "",
        None,
        "Creatures you control have vigilance.",
    );
    let deck = add_scheme_deck(
        &mut t,
        P0,
        vec![real("The Very Soil Shall Shake"), other],
    );
    keyword_action(&mut t, P0, KeywordAction::SetInMotion, 2);
    t.resolve_all();
    assert_eq!(variants::face_up_schemes(&t.g).len(), 2);
    t.g.destroy(bears, None);
    t.settle();
    t.resolve_all();
    assert!(t.obj(deck[0]).face_down, "this scheme was abandoned");
    assert!(!t.obj(deck[1]).face_down, "the other scheme stays");
    let _ = CardType::Scheme;
}
