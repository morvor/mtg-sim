//! CR 116: special actions.

use super::r114_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::events::MoveCause;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn add_mana(t: &mut TestGame, p: PlayerId, ty: ManaType, n: u32) {
    t.g.players[p.idx()].mana_pool.add_type(ty, n);
}

fn specials(t: &mut TestGame, p: PlayerId) -> Vec<SpecialAction> {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .into_iter()
        .filter_map(|a| match a {
            Action::Special(s) => Some(s),
            _ => None,
        })
        .collect()
}

/// The special action granted by a static ability of `source`.
fn static_action(t: &mut TestGame, p: PlayerId, source: ObjectId) -> Option<SpecialAction> {
    specials(t, p)
        .into_iter()
        .find(|s| matches!(s, SpecialAction::Static { source: s2, .. } if *s2 == source))
}

/// Takes a special action through the priority system.
fn take(t: &mut TestGame, p: PlayerId, sa: SpecialAction) {
    t.g.turn.priority = Some(p);
    t.g.take_action(p, Action::Special(sa));
    t.g.flush_events();
}

fn face_down(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.g.create_card_object(card(name), p, Zone::Nowhere);
    t.g.move_object_ev(MoveEv {
        obj: id,
        to: Zone::Battlefield,
        pos: LibraryPosition::Top,
        cause: MoveCause::Effect,
        by: Some(p),
        etb: EtbInfo {
            controller: Some(p),
            face_down: Some(KeywordKind::Morph),
            ..Default::default()
        },
        source: None,
    })
    .unwrap()
}

#[test]
fn special_actions_dont_use_the_stack_and_the_player_gets_priority_afterward() {
    cr!("116.1", "116.3");
    ruling!(
        "Leonin Arbiter",
        "Paying {2} to ignore Leonin Arbiter's effect is a special action. Any player may take this special action any time they have priority. It doesn't use the stack and can't be responded to."
    );
    let mut t = TestGame::new(2);
    let arbiter = t.battlefield(P0, "Leonin Arbiter");
    add_mana(&mut t, P1, ManaType::C, 2);
    // P0 passes; P1 takes the special action and gets priority again, and the passes
    // in succession start over.
    t.g.turn.priority = Some(P0);
    t.g.take_action(P0, Action::Pass);
    assert_eq!(t.g.turn.priority, Some(P1));
    let sa = static_action(&mut t, P1, arbiter).expect("special action available");
    take(&mut t, P1, sa);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.turn.priority, Some(P1));
    assert_eq!(t.g.turn.passes, 0);
    assert_eq!(t.player(P1).mana_pool.total(), 0);
}

#[test]
fn playing_a_land_is_a_special_action_taken_in_your_main_phase() {
    cr!("116.2a");
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    let island = t.hand(P0, "Island");
    // Not during the upkeep.
    t.set_step(P0, Step::Upkeep);
    assert!(t.play_land(P0, forest).is_err());
    // Not while the stack isn't empty.
    t.set_step(P0, Step::PrecombatMain);
    let s = t.custom(P0, free_instant("Quick Gift"), Zone::Hand(P0));
    t.cast(P0, s).go();
    assert!(t.play_land(P0, forest).is_err());
    t.resolve_all();
    // In the main phase with an empty stack: it doesn't use the stack.
    t.play_land(P0, forest).unwrap();
    assert!(t.on_battlefield(t.g.current(forest)));
    assert_eq!(t.stack_len(), 0);
    // Only once each turn by default.
    assert!(t.play_land(P0, island).is_err());
    // Not during another player's turn.
    let mut t = TestGame::new(2);
    let forest = t.hand(P1, "Forest");
    t.set_step(P0, Step::PrecombatMain);
    assert!(t.play_land(P1, forest).is_err());
}

#[test]
fn turning_a_face_down_creature_face_up_any_time_you_have_priority() {
    cr!("116.2b");
    let mut t = TestGame::new(2);
    let angel = face_down(&mut t, P0, "Exalted Angel");
    assert!(t.obj(angel).face_down);
    // During the opponent's turn, with a spell on the stack.
    t.set_step(P1, Step::PrecombatMain);
    let s = t.custom(P1, free_instant("Quick Gift"), Zone::Hand(P1));
    t.cast(P1, s).go();
    // Without the morph cost, it can't be done.
    assert!(!specials(&mut t, P0).contains(&SpecialAction::TurnFaceUp { obj: angel }));
    add_mana(&mut t, P0, ManaType::W, 2);
    add_mana(&mut t, P0, ManaType::C, 2);
    assert!(specials(&mut t, P0).contains(&SpecialAction::TurnFaceUp { obj: angel }));
    take(&mut t, P0, SpecialAction::TurnFaceUp { obj: angel });
    assert!(!t.obj(angel).face_down);
    assert_eq!(t.obj(angel).chars.name, "Exalted Angel");
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    // It didn't use the stack: only the spell is there.
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn an_effect_may_let_a_player_take_an_action_later() {
    cr!("116.2c");
    // Guardian Angel: "Prevent the next X damage that would be dealt to any target this
    // turn. Until end of turn, you may pay {1} any time you could cast an instant. If you
    // do, prevent the next 1 damage that would be dealt to that permanent or player this
    // turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let angel = t.hand(P0, "Guardian Angel");
    add_mana(&mut t, P0, ManaType::W, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, angel).x(1).target(bears).go();
    t.resolve_all();
    // The opponent can't take the action; P0 can, each time paying {1}.
    add_mana(&mut t, P1, ManaType::C, 1);
    assert!(!specials(&mut t, P1)
        .iter()
        .any(|s| matches!(s, SpecialAction::Offer { .. })));
    add_mana(&mut t, P0, ManaType::C, 2);
    for _ in 0..2 {
        let sa = specials(&mut t, P0)
            .into_iter()
            .find(|s| matches!(s, SpecialAction::Offer { .. }))
            .unwrap();
        take(&mut t, P0, sa);
    }
    assert_eq!(t.stack_len(), 0);
    // 1 + 2 damage prevented: Lightning Bolt deals none.
    t.set_step(P1, Step::PrecombatMain);
    add_mana(&mut t, P1, ManaType::R, 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(bears).go();
    t.resolve_all();
    assert_eq!(t.obj(bears).damage, 0);
    // The action is only available until end of turn.
    add_mana(&mut t, P0, ManaType::C, 1);
    t.advance_to(P1, Step::Upkeep);
    assert!(!specials(&mut t, P0)
        .iter()
        .any(|s| matches!(s, SpecialAction::Offer { .. })));
}

#[test]
fn a_player_may_take_an_action_to_ignore_a_static_abilitys_effect() {
    cr!("116.2d");
    ruling!(
        "Leonin Arbiter",
        "If a player pays {2}, that enables only them to ignore Leonin Arbiter's effect that turn. Each other player is still affected by it."
    );
    let mut t = TestGame::new(2);
    let arbiter = t.battlefield(P0, "Leonin Arbiter");
    t.library_top(P0, "Forest");
    t.library_top(P1, "Forest");
    let tutor = |t: &mut TestGame, p: PlayerId| {
        t.set_step(p, Step::PrecombatMain);
        add_mana(t, p, ManaType::G, 1);
        add_mana(t, p, ManaType::C, 1);
        let s = t.hand(p, "Sylvan Scrying");
        t.cast(p, s).go();
        t.resolve_all();
    };
    // Nobody can search.
    tutor(&mut t, P0);
    assert!(!t.in_hand(P0, "Forest"));
    // P0 pays {2}: P0 ignores the effect this turn; P1 is still affected.
    add_mana(&mut t, P0, ManaType::C, 2);
    let sa = static_action(&mut t, P0, arbiter).unwrap();
    take(&mut t, P0, sa);
    // It can't be taken again while P0 is ignoring the effect.
    add_mana(&mut t, P0, ManaType::C, 2);
    assert!(static_action(&mut t, P0, arbiter).is_none());
    tutor(&mut t, P0);
    assert!(t.in_hand(P0, "Forest"));
    t.g.players[0].mana_pool.empty();
    t.g.turn.active = P1;
    tutor(&mut t, P1);
    assert!(!t.in_hand(P1, "Forest"));
    // Lost in Thought: the enchanted creature's controller exiles three cards from their
    // graveyard to ignore "can't attack or block" until end of turn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = t.custom(
        P0,
        card("Lost in Thought").as_ref().clone(),
        Zone::Battlefield,
    );
    t.g.attach(aura, Entity::Object(bears));
    for _ in 0..3 {
        t.graveyard(P1, "Island");
    }
    t.g.recompute();
    // Only the creature's controller can take the action.
    assert!(static_action(&mut t, P0, aura).is_none());
    let sa = static_action(&mut t, P1, aura).unwrap();
    t.set_step(P1, Step::PrecombatMain);
    assert!(!t.g.can_attack(bears));
    take(&mut t, P1, sa);
    assert_eq!(t.graveyard_size(P1), 0);
    assert!(t.g.can_attack(bears));
    // Next turn the effect applies again.
    t.advance_to(P0, Step::Upkeep);
    t.set_step(P1, Step::PrecombatMain);
    assert!(!t.g.can_attack(bears));
}

#[test]
fn circling_vultures_may_be_discarded_any_time_you_could_cast_an_instant() {
    cr!("116.2e");
    ruling!(
        "Circling Vultures",
        "Circling Vultures's discard ability is a static ability, not an activated ability."
    );
    let mut t = TestGame::new(2);
    let vultures = t.hand(P0, "Circling Vultures");
    // During the opponent's turn, with a spell on the stack.
    t.set_step(P1, Step::PrecombatMain);
    let s = t.custom(P1, free_instant("Quick Gift"), Zone::Hand(P1));
    t.cast(P1, s).go();
    assert!(static_action(&mut t, P1, vultures).is_none());
    let sa = static_action(&mut t, P0, vultures).unwrap();
    take(&mut t, P0, sa);
    assert!(t.in_graveyard(P0, "Circling Vultures"));
    assert_eq!(t.stack_len(), 1);
    // Only from the hand.
    let on_bf = t.battlefield(P0, "Circling Vultures");
    assert!(static_action(&mut t, P0, on_bf).is_none());
}

#[test]
fn turning_a_face_down_conspiracy_face_up_is_a_special_action() {
    cr!("116.2j");
    ruling!(
        "Secret Summoning",
        "As a special action, you may turn a face-down conspiracy face up. You may do so any time you have priority."
    );
    let mut t = TestGame::new(2);
    let c = t.command(P0, "Secret Summoning");
    t.g.objects[c.0 as usize].face_down = true;
    t.g.recompute();
    // Its owner may turn it face up, the opponent may not; it gets a new timestamp.
    assert!(!specials(&mut t, P1).contains(&SpecialAction::TurnFaceUp { obj: c }));
    assert!(specials(&mut t, P0).contains(&SpecialAction::TurnFaceUp { obj: c }));
    let before = t.obj(c).timestamp;
    take(&mut t, P0, SpecialAction::TurnFaceUp { obj: c });
    assert!(!t.obj(c).face_down);
    assert!(t.obj(c).timestamp > before);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn a_special_actions_hybrid_or_phyrexian_cost_is_chosen_before_paying() {
    cr!("118.13c");
    // "Any player may pay {W/P} for that player to ignore this effect until end of turn."
    let arbiter = compile_def(
        "Phyrexian Arbiter",
        "Creature — Cat Cleric",
        "{1}{W}",
        "Players can't search libraries. Any player may pay {W/P} for that player to ignore this effect until end of turn.",
    );
    let mut t = TestGame::new(2);
    let a = t.custom(P0, arbiter, Zone::Battlefield);
    add_mana(&mut t, P1, ManaType::W, 1);
    let sa = static_action(&mut t, P1, a).unwrap();
    // P1 chooses to pay 2 life rather than {W}, though white mana is available.
    t.answer(P1, DecisionKind::Option, Answer::Index(2));
    take(&mut t, P1, sa);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.player(P1).mana_pool.total(), 1);
}
