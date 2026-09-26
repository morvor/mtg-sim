//! CR 405: the stack — what goes on it, its order, simultaneous objects, the
//! characteristics and controllers of objects on it, resolution, and things that don't use
//! it.

use crate::r114_common::*;
use crate::r703_common::{oracle_card, run_effect, supported};
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::object::{CastMethod, ObjKind, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

fn cast_action(card: ObjectId) -> Action {
    Action::Cast {
        card,
        method: CastMethod::Normal,
    }
}

/// The given objects, which are in a zone of this kind.
fn in_zone(zone: ZoneKind, ids: Vec<ObjectId>) -> Sel {
    Sel::All(Filter::and(vec![Filter::InZone(zone), Filter::Objects(ids)]))
}

// ---------------------------------------------------------------------------
// 405.1–405.2
// ---------------------------------------------------------------------------

#[test]
fn a_cast_spell_is_the_card_on_the_stack_and_an_ability_has_no_card() {
    cr!("405.1");
    supported("Prodigal Sorcerer");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let card = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, card).go();
    let o = t.obj(spell);
    assert_eq!(o.zone, Zone::Stack);
    assert_eq!(o.kind, ObjKind::Card);
    assert_eq!(o.card.as_ref().unwrap().name, "Grizzly Bears");
    assert_eq!(t.obj(card).next, Some(spell), "the card itself moved to the stack");
    t.resolve();
    // An activated ability goes on top of the stack without any card.
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    let ab = t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).unwrap().unwrap();
    let o = t.obj(ab);
    assert_eq!(o.kind, ObjKind::StackAbility);
    assert!(o.card.is_none());
    assert_eq!(t.g.stack.last(), Some(&ab));
}

#[test]
fn each_object_is_put_on_top_of_the_stack_and_the_last_one_resolves_first() {
    cr!("405.2");
    let mut t = TestGame::new(2);
    let a = t.custom(P0, free_instant("First"), Zone::Hand(P0));
    let b = t.custom(P0, free_instant("Second"), Zone::Hand(P0));
    let c = t.custom(P1, free_instant("Third"), Zone::Hand(P1));
    let a = t.cast(P0, a).go();
    let b = t.cast(P0, b).go();
    let c = t.cast(P1, c).go();
    assert_eq!(t.g.stack, vec![a, b, c]);
    t.g.resolve_top();
    assert_eq!(t.g.stack, vec![a, b]);
    assert_eq!(t.life(P1), 21, "the last one added resolved");
}

// ---------------------------------------------------------------------------
// 405.3: objects put onto the stack at the same time
// ---------------------------------------------------------------------------

#[test]
fn simultaneous_objects_go_on_the_stack_active_player_lowest_then_apnap() {
    cr!("405.3");
    ruling!(
        "Hive Mind",
        "First the player whose turn it is (or, if that’s the player who cast the original spell, the player to that player’s left) puts their copy on the stack"
    );
    supported("Hive Mind");
    // Three players; it's P1's turn. P0 controls Hive Mind; P2 casts Lightning Bolt.
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Hive Mind");
    t.lands(P2, "Mountain", 1);
    let bolt = t.hand(P2, "Lightning Bolt");
    let spell = t.cast(P2, bolt).target(P1).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // The copies: P1's (the active player's) lowest, then P0's; none for P2.
    assert_eq!(t.stack_len(), 3);
    assert_eq!(t.g.stack[0], spell);
    let controllers: Vec<PlayerId> = t.g.stack[1..].iter().map(|c| t.obj(*c).controller).collect();
    assert_eq!(controllers, vec![P1, P0]);
    assert!(t.g.stack[1..]
        .iter()
        .all(|c| t.obj(*c).kind == ObjKind::SpellCopy));
}

#[test]
fn a_player_with_several_simultaneous_objects_chooses_their_order() {
    cr!("405.3");
    // Copies of a spell for each other object it could target, all controlled by P0: P0
    // orders them.
    let bolt_def = oracle_card(
        "Zap",
        "Instant",
        "{0}",
        None,
        "Zap deals 1 damage to target creature.",
    );
    for order in [vec![0, 1], vec![1, 0]] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Grizzly Bears");
        let b = t.battlefield(P1, "Hill Giant");
        let c = t.battlefield(P1, "Craw Wurm");
        let zap = t.custom(P0, bolt_def.clone(), Zone::Hand(P0));
        let spell = t.cast(P0, zap).target(a).go();
        t.answer(P0, DecisionKind::Order, Answer::Indices(order.clone()));
        run_effect(
            &mut t,
            P0,
            None,
            Effect::CopySpellRetargeted {
                what: in_zone(ZoneKind::Stack, vec![spell]),
                target: None,
            },
            &[],
        );
        assert!(t
            .asked()
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::Order { .. })));
        let targets: Vec<Entity> = t.g.stack[1..]
            .iter()
            .map(|id| t.obj(*id).stack.as_ref().unwrap().chosen[0].targets[0][0])
            .collect();
        let expected = if order == [0, 1] {
            vec![Entity::Object(b), Entity::Object(c)]
        } else {
            vec![Entity::Object(c), Entity::Object(b)]
        };
        assert_eq!(targets, expected);
    }
}

// ---------------------------------------------------------------------------
// 405.4: characteristics and controllers
// ---------------------------------------------------------------------------

#[test]
fn a_spell_has_its_cards_characteristics_and_an_ability_only_its_text() {
    cr!("405.4");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    let card = t.hand(P0, "Grizzly Bears");
    let spell = t.cast(P0, card).go();
    let c = &t.obj(spell).chars;
    assert_eq!(c.name, "Grizzly Bears");
    assert!(c.is(CardType::Creature) && c.has_subtype("Bear"));
    assert!(c.colors.contains(Color::Green));
    assert_eq!((c.power, c.toughness), (Some(2), Some(2)));
    assert_eq!(c.mana_cost.as_ref().unwrap().to_string(), "{1}{G}");
    // The controller of a spell is the player who cast it.
    assert_eq!(t.obj(spell).controller, P0);
    t.resolve();
    // An activated ability: the player who activated it controls it; it has no card types,
    // colors, mana cost, or power and toughness.
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    let ab = t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).unwrap().unwrap();
    let o = t.obj(ab);
    assert_eq!(o.controller, P0);
    assert!(o.chars.card_types.is_empty());
    assert_eq!(o.chars.colors, ColorSet::NONE);
    assert!(o.chars.mana_cost.is_none());
    assert_eq!(o.chars.power, None);
    t.resolve();
}

#[test]
fn a_triggered_ability_is_controlled_by_its_sources_controller_when_it_triggered() {
    cr!("405.4");
    supported("Soul Warden");
    let mut t = TestGame::new(2);
    // P1 owns Soul Warden but P0 controls it.
    let warden = t.battlefield(P1, "Soul Warden");
    t.g.objects[warden.0 as usize].base_controller = P0;
    t.g.recompute();
    t.battlefield(P1, "Grizzly Bears");
    let bears = t.hand(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 2);
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, bears).go();
    t.g.resolve_top();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let trig = t.g.stack[0];
    assert_eq!(t.obj(trig).controller, P0);
    // Even if control of the source changes before it resolves.
    t.g.objects[warden.0 as usize].base_controller = P1;
    t.g.recompute();
    assert_eq!(t.obj(trig).controller, P0);
    t.resolve();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.life(P1), 20);
}

// ---------------------------------------------------------------------------
// 405.5: resolving
// ---------------------------------------------------------------------------

#[test]
fn when_all_players_pass_the_top_object_resolves_or_the_step_ends() {
    cr!("405.5");
    let mut t = TestGame::new(2);
    let a = t.custom(P0, free_instant("First"), Zone::Hand(P0));
    let b = t.custom(P0, free_instant("Second"), Zone::Hand(P0));
    t.g.take_action(P0, cast_action(a));
    t.g.take_action(P0, cast_action(b));
    assert_eq!(t.stack_len(), 2);
    t.g.take_action(P0, Action::Pass);
    t.g.take_action(P1, Action::Pass);
    // Only the top one resolved.
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 21);
    t.g.take_action(P0, Action::Pass);
    t.g.take_action(P1, Action::Pass);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.turn.stage, Stage::Priority);
    // With the stack empty, the step ends.
    t.g.take_action(P0, Action::Pass);
    t.g.take_action(P1, Action::Pass);
    assert_eq!(t.turn.stage, Stage::End);
}

// ---------------------------------------------------------------------------
// 405.6: things that don't use the stack
// ---------------------------------------------------------------------------

#[test]
fn effects_dont_use_the_stack_but_delayed_triggers_do() {
    cr!("405.6", "405.6a");
    let mut t = TestGame::new(2);
    let def = oracle_card(
        "Slow Healing",
        "Instant",
        "{0}",
        None,
        "You gain 2 life. At the beginning of the next end step, you gain 3 life.",
    );
    let card = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, card).go();
    t.resolve();
    // The effect happened as the spell resolved: nothing new on the stack.
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.stack_len(), 0);
    // The delayed triggered ability goes on the stack when it triggers.
    crate::r703_common::to_step_start(&mut t, P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.obj(t.g.stack[0]).kind, ObjKind::StackAbility);
    t.resolve();
    assert_eq!(t.life(P0), 25);
}

#[test]
fn static_abilities_apply_without_using_the_stack() {
    cr!("405.6b");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let goyf = t.battlefield(P0, "Tarmogoyf");
    t.lands(P0, "Plains", 3);
    let anthem = t.hand(P0, "Glorious Anthem");
    t.cast(P0, anthem).go();
    t.resolve();
    // Glorious Anthem's effect applies at once; nothing was put on the stack.
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.pt(bears), (3, 3));
    // Nor does a characteristic-defining ability: Tarmogoyf's power and toughness follow
    // the cards in graveyards (an instant, now), with the Anthem's bonus on top.
    t.graveyard(P1, "Lightning Bolt");
    t.g.recompute();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.pt(goyf), (2, 3));
}

#[test]
fn mana_abilities_resolve_immediately_with_all_their_effects() {
    cr!("405.6c");
    supported("Shivan Reef");
    let mut t = TestGame::new(2);
    let reef = t.battlefield(P0, "Shivan Reef");
    assert_eq!(t.turn.priority, Some(P0));
    // "{T}: Add {U} or {R}. This land deals 1 damage to you."
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.activate(P0, reef, 1, &[]).unwrap();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.player(P0).mana_pool.total(), 1);
    assert_eq!(t.life(P0), 19, "the other effect happened immediately too");
    // The player who had priority still has it.
    assert_eq!(t.turn.priority, Some(P0));
}

#[test]
fn special_actions_happen_immediately() {
    cr!("405.6d");
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    t.g.take_action(P0, Action::PlayLand { card: forest });
    assert!(t.on_battlefield(forest));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.turn.priority, Some(P0));
}

#[test]
fn turn_based_actions_dont_use_the_stack() {
    cr!("405.6e");
    let mut t = TestGame::new(2);
    let hand = t.hand_size(P1);
    crate::r703_common::to_step_start(&mut t, P1, Step::Draw);
    // The draw happened as the step began, before anyone received priority.
    assert_eq!(t.hand_size(P1), hand + 1);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn state_based_actions_dont_use_the_stack() {
    cr!("405.6f");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.obj_mut(bears).damage = 2;
    t.settle();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn a_player_who_concedes_leaves_the_game_immediately() {
    cr!("405.6g");
    ruling!(
        "Platinum Angel",
        "You can concede a game while Platinum Angel on the battlefield"
    );
    supported("Platinum Angel");
    let mut t = TestGame::new(3);
    // P1 can't lose the game ("You can't lose the game and your opponents can't win the
    // game")...
    t.battlefield(P1, "Platinum Angel");
    t.g.lose_game(P1);
    assert!(!t.has_lost(P1), "Platinum Angel is in effect");
    let q = t.custom(P0, free_instant("Quick"), Zone::Hand(P0));
    t.g.take_action(P0, cast_action(q));
    assert_eq!(t.stack_len(), 1);
    t.g.take_action(P0, Action::Pass);
    // ...but P1 can still concede, while the spell is on the stack: they're out at once.
    t.g.take_action(P1, Action::Concede);
    assert!(t.has_lost(P1));
    assert!(!t.player(P1).in_game());
    assert_eq!(t.stack_len(), 1);
}

#[test]
fn a_player_leaving_a_multiplayer_game_takes_their_objects_with_them_immediately() {
    cr!("405.6h");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let q = t.custom(P1, free_instant("Quick"), Zone::Hand(P1));
    t.g.take_action(P0, Action::Pass);
    t.g.take_action(P1, cast_action(q));
    assert_eq!(t.stack_len(), 1);
    t.g.player_loses(P1);
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.stack_len(), 0);
}
