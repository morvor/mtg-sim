//! CR 800: multiplayer rules — general, and leaving the game.

use super::r800_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::game::{AttackSide, GameConfig, Variant};
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn ffa(n: usize) -> TestGame {
    TestGame::with_config(n, GameConfig::free_for_all())
}

/// Gives `controller` control of `id` indefinitely, as a resolved "gain control of"
/// effect of theirs would.
fn steal(t: &mut TestGame, controller: PlayerId, id: ObjectId, duration: Duration) {
    apply(
        t,
        controller,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration,
        },
        &[id],
    );
}

// ---------------------------------------------------------------------------
// General
// ---------------------------------------------------------------------------

#[test]
fn a_game_that_begins_with_more_than_two_players_is_multiplayer() {
    cr!("800.1", "800.6");
    // The first mulligan a player takes in a multiplayer game is free.
    let mut t = super::r100_common::pregame(
        GameConfig {
            starting_player: Some(P0),
            ..GameConfig::free_for_all()
        },
        vec![fillers(40), fillers(40), fillers(40)],
    );
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    t.answer(P1, DecisionKind::Mulligan, Answer::Bool(true));
    t.answer(P1, DecisionKind::Mulligan, Answer::Bool(true));
    t.g.start();
    assert!(t.g.is_multiplayer());
    assert_eq!(t.hand_size(P0), 7, "the first mulligan doesn't count");
    assert_eq!(t.hand_size(P1), 6, "later mulligans count as normal");
    assert_eq!(t.hand_size(P2), 7);
    // It's still a multiplayer game after a player leaves.
    t.g.player_loses(P2);
    assert!(t.g.is_multiplayer());
    // A two-player game isn't.
    let mut t = super::r100_common::pregame(
        GameConfig {
            starting_player: Some(P0),
            ..GameConfig::default()
        },
        vec![fillers(40), fillers(40)],
    );
    t.answer(P0, DecisionKind::Mulligan, Answer::Bool(true));
    t.g.start();
    assert!(!t.g.is_multiplayer());
    assert_eq!(t.hand_size(P0), 6);
}

fn fillers(n: usize) -> Vec<std::sync::Arc<mtg_engine::card::CardDef>> {
    super::r100_common::fillers(n)
}

#[test]
fn a_game_may_use_several_options_but_one_variant() {
    cr!("800.2");
    // A Free-for-All game with both the limited range of influence and the attack left
    // options: each applies.
    let config = GameConfig {
        range_of_influence: Some(1),
        attack_side: Some(AttackSide::Left),
        attack_multiple_players: false,
        ..GameConfig::free_for_all()
    };
    assert_eq!(config.validate(6), Ok(()));
    let mut t = TestGame::with_config(6, config);
    let a = bear(&mut t, P0);
    // Attack left: only P1 (and range 1 allows P1 and P5).
    assert_eq!(targets_of(&attack_choices(&mut t, P0), a), vec![Entity::Player(P1)]);
    t.set_step(P0, Step::PrecombatMain);
    t.g.combat = None;
    // Range 1 limits targets to P1 and P5.
    let c = bolt_candidates(&mut t, P0);
    assert!(c.contains(&Entity::Player(P1)) && c.contains(&Entity::Player(P5)));
    assert!(!c.contains(&Entity::Player(P2)));
    // The configuration names exactly one variant.
    assert_eq!(t.g.config.variant, Variant::FreeForAll);
}

#[test]
fn seating_is_as_the_players_agree_unless_the_variant_says_otherwise() {
    cr!("800.5");
    // Without a variant that prescribes seating, the players stay where they were.
    let decks: Vec<_> = (0..4)
        .map(|i| {
            vec![std::sync::Arc::new(mtg_engine::card::CardDef::custom(
                mtg_engine::object::Characteristics {
                    name: format!("Card of participant {i}").into(),
                    rules_text: std::sync::Arc::from(""),
                    ..Default::default()
                },
            ))]
        })
        .collect();
    let g = mtg_engine::multiplayer::setup::new_seated(GameConfig::default(), decks, vec![]);
    assert_eq!(g.multiplayer.seats, vec![0, 1, 2, 3]);
    for i in 0..4 {
        let top = g.players[i].library[0];
        assert_eq!(
            g.obj(top).chars.name.as_str(),
            format!("Card of participant {i}")
        );
    }
}

#[test]
fn the_starting_player_draws_on_their_first_turn_except_in_two_headed_giant() {
    cr!("800.7");
    let mut t = super::r100_common::pregame(
        GameConfig {
            starting_player: Some(P0),
            skip_mulligans: true,
            ..GameConfig::free_for_all()
        },
        vec![fillers(40), fillers(40), fillers(40)],
    );
    t.g.start();
    to_step(&mut t, P0, Step::PrecombatMain);
    assert!(t.g.turn.step_log.contains(&Step::Draw));
    assert_eq!(t.hand_size(P0), 8);
    // In Two-Headed Giant, the team that plays first skips the draw step of its first turn.
    let mut t = super::r100_common::pregame(
        GameConfig {
            starting_player: Some(P0),
            skip_mulligans: true,
            ..GameConfig::two_headed_giant(vec![0, 0, 1, 1])
        },
        (0..4).map(|_| fillers(40)).collect(),
    );
    t.g.start();
    to_step(&mut t, P0, Step::PrecombatMain);
    assert!(!t.g.turn.step_log.contains(&Step::Draw));
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (7, 7));
}

// ---------------------------------------------------------------------------
// Leaving the game (CR 800.4)
// ---------------------------------------------------------------------------

#[test]
fn a_multiplayer_game_continues_after_a_player_leaves() {
    cr!("800.4");
    let mut t = ffa(3);
    concede(&mut t, P1);
    assert!(t.g.player(P1).left_game);
    assert_eq!(t.g.result, None);
    // The turn after P0's is P2's.
    to_turn_of(&mut t, P2);
    assert_eq!(t.g.turn.previous_active, Some(P0));
}

#[test]
fn a_leaving_players_objects_leave_and_what_they_control_is_exiled() {
    cr!("800.4a");
    let mut t = ffa(4);
    // P1's creature, which P0 controls: it leaves the game with its owner.
    let theirs = bear(&mut t, P1);
    steal(&mut t, P0, theirs, Duration::Permanent);
    // P0's creature, which P1 controls through an effect: the effect ends, and it returns
    // to P0.
    let mine = t.battlefield(P0, "Hill Giant");
    steal(&mut t, P1, mine, Duration::Permanent);
    assert_eq!(t.obj_now(mine).controller, P1);
    // Reanimate (P1): "Put target creature card from a graveyard onto the battlefield
    // under your control." P2's creature card: P1 still controls it once those effects
    // end, so it's exiled.
    let dead = t.graveyard(P2, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Swamp", 1);
    let reanimate = t.hand(P1, "Reanimate");
    t.cast(P1, reanimate).target(dead).go();
    t.resolve();
    let reanimated = t.g.current(dead);
    assert!(on_bf(&t, reanimated));
    assert_eq!(t.obj_now(reanimated).controller, P1);
    // P1 activates an ability; it's on the stack when P1 leaves, while P1 has priority.
    let pyro = t.battlefield(P1, "Prodigal Pyromancer");
    t.activate(P1, pyro, 0, &[Entity::Player(P0)]).unwrap();
    assert_eq!(t.stack_len(), 1);
    t.g.turn.priority = Some(P1);
    t.g.take_action(P1, Action::Concede);
    // This happened immediately, not as a state-based action.
    assert!(!t.g.is_live(t.g.current(theirs)) || t.zone(theirs) == Zone::Nowhere);
    assert!(t.g.find_in_zone(Zone::Battlefield, "Prodigal Pyromancer").is_empty());
    assert!(!t.in_exile("Prodigal Pyromancer"), "left the game, not exiled");
    assert_eq!(t.obj_now(mine).controller, P0);
    assert!(on_bf(&t, mine));
    assert_eq!(t.zone(reanimated), Zone::Exile);
    assert_eq!(t.stack_len(), 0, "the ability ceased to exist");
    // P1 had priority: the next player in turn order gets it.
    assert_eq!(t.g.turn.priority, Some(P2));
}

#[test]
fn a_static_control_effect_of_a_leaving_player_ends_before_their_objects_are_exiled() {
    cr!("800.4a");
    // P1 controls a Mind Control ("You control enchanted creature.") that P2 owns — it
    // entered under P1's control — enchanting P3's creature.
    let mut t = ffa(4);
    let bears = bear(&mut t, P3);
    let mc = t.battlefield(P2, "Mind Control");
    t.g.objects[mc.0 as usize].base_controller = P1;
    assert!(t.g.attach(mc, Entity::Object(bears)));
    t.g.recompute();
    assert_eq!(t.obj_now(mc).controller, P1);
    assert_eq!(t.obj_now(bears).controller, P1);
    // P1 leaves. The effect giving P1 control of the creature ends first: it returns to P3
    // and stays. Then the Aura, which P1 still controls, is exiled.
    concede(&mut t, P1);
    assert!(on_bf(&t, bears));
    assert_eq!(t.obj_now(bears).controller, P3);
    assert_eq!(t.zone(mc), Zone::Exile);
}

#[test]
fn nothing_changes_to_the_control_of_a_player_who_left() {
    cr!("800.4b");
    let mut t = ffa(4);
    let bear0 = bear(&mut t, P0);
    concede(&mut t, P1);
    // Control of an object doesn't change to P1.
    apply(
        &mut t,
        P0,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::Player(P1),
            duration: Duration::Permanent,
        },
        &[bear0],
    );
    assert_eq!(t.obj_now(bear0).controller, P0);
    // No token is created under P1's control.
    let spec = mtg_engine::replacement::TokenCreate {
        chars: mtg_engine::object::Characteristics {
            name: "Soldier".into(),
            card_types: [CardType::Creature].into_iter().collect(),
            power: Some(1),
            toughness: Some(1),
            rules_text: std::sync::Arc::from(""),
            ..Default::default()
        },
        card: None,
        tapped: false,
        attacking: None,
        copy_of: None,
        copy_exceptions: vec![],
    };
    assert!(t.g.create_tokens(P1, spec, 1, None).is_empty());
    assert!(t.named_on_battlefield("Soldier").is_empty());
    // A card that would be put onto the battlefield under P1's control stays where it is.
    let card = t.graveyard(P0, "Grizzly Bears");
    let moved = t.g.move_object_ev(MoveEv {
        obj: card,
        to: Zone::Battlefield,
        pos: LibraryPosition::Top,
        cause: mtg_engine::events::MoveCause::Effect,
        by: Some(P0),
        etb: EtbInfo {
            controller: Some(P1),
            ..Default::default()
        },
        source: None,
    });
    assert_eq!(moved, None);
    assert_eq!(t.zone(card), Zone::Graveyard(P0));
    // A player isn't controlled by a player who left: P2 activates Mindslaver on P3, then
    // leaves; P3 makes their own decisions during their turn.
    let slaver = t.battlefield(P2, "Mindslaver");
    t.lands(P2, "Wastes", 4);
    t.set_step(P2, Step::PrecombatMain);
    t.activate(P2, slaver, 0, &[Entity::Player(P3)]).unwrap();
    t.resolve();
    concede(&mut t, P2);
    to_turn_of(&mut t, P3);
    assert_eq!(mtg_engine::player_control::decider(&t.g, P3), P3);
}

#[test]
fn an_object_whose_default_controller_left_is_exiled_when_a_control_effect_ends() {
    cr!("800.4c");
    let mut t = ffa(4);
    // P1 reanimates P2's creature: P1 controls it by default.
    let dead = t.graveyard(P2, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Swamp", 1);
    let reanimate = t.hand(P1, "Reanimate");
    t.cast(P1, reanimate).target(dead).go();
    t.resolve();
    let giant = t.g.current(dead);
    // P3 gains control of it until end of turn.
    steal(&mut t, P3, giant, Duration::EndOfTurn);
    assert_eq!(t.obj_now(giant).controller, P3);
    // P1 leaves: P3 still controls it, so it stays.
    concede(&mut t, P1);
    assert!(on_bf(&t, giant));
    // When P3's control effect ends, no one in the game controls it: it's exiled.
    to_turn_of(&mut t, P2);
    assert_eq!(t.zone(giant), Zone::Exile);
}

#[test]
fn nothing_is_created_for_or_triggers_for_a_player_who_left() {
    cr!("800.4d");
    let mut t = ffa(4);
    // Blood Tyrant (P1): "Whenever a player loses the game, put five +1/+1 counters on
    // this creature." P1 conceding makes P1 lose, but P1's triggered ability isn't put on
    // the stack.
    t.battlefield(P1, "Blood Tyrant");
    concede(&mut t, P1);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // A token that P1 would own isn't created.
    let spec = mtg_engine::replacement::TokenCreate {
        chars: mtg_engine::object::Characteristics {
            name: "Spirit".into(),
            card_types: [CardType::Creature].into_iter().collect(),
            power: Some(1),
            toughness: Some(1),
            rules_text: std::sync::Arc::from(""),
            ..Default::default()
        },
        card: None,
        tapped: false,
        attacking: None,
        copy_of: None,
        copy_exceptions: vec![],
    };
    assert!(t.g.create_tokens(P1, spec, 2, None).is_empty());
    assert!(t.named_on_battlefield("Spirit").is_empty());
}

#[test]
fn combat_damage_isnt_assigned_to_a_player_who_left() {
    cr!("800.4e");
    let mut t = ffa(4);
    // Vampire Nighthawk has lifelink: if it dealt damage, P0 would gain life.
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let other = bear(&mut t, P0);
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, &[(hawk, Entity::Player(P1)), (other, Entity::Player(P2))]);
    go_to(&mut t, Step::DeclareBlockers);
    concede(&mut t, P1);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20, "no damage was assigned to P1");
    assert_eq!(t.life(P2), 18);
}

#[test]
fn a_player_who_left_doesnt_pay_costs() {
    cr!("800.4f");
    // "Whenever an opponent casts a spell, draw a card unless that player pays {1}."
    let mut t = ffa(3);
    let study = custom_with(
        "Tax Collector's Study",
        "Enchantment",
        None,
        vec![triggered(
            TriggerCond::CastSpell {
                who: PlayerRel::Opponent,
                filter: Filter::Any,
            },
            Effect::PayOptional {
                who: PlayerRef::TriggerPlayer,
                cost: Cost::mana(mtg_engine::mana::ManaCost::parse("{1}").unwrap()),
                then: Box::new(Effect::Noop),
                otherwise: Box::new(Effect::Draw {
                    who: PlayerRef::You,
                    n: Value::c(1),
                }),
            },
        )],
    );
    bf(&mut t, P0, study);
    t.set_step(P1, Step::PrecombatMain);
    let spell = t.custom(P1, super::r114_common::free_sorcery("Nothing Much"), Zone::Hand(P1));
    t.cast(P1, spell).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // P1 has mana to pay and would pay, but leaves the game before the ability resolves.
    // Everyone else would also choose to pay if they were asked in P1's place.
    t.g.players[1]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::C, 1);
    for p in [P0, P1, P2] {
        t.answer_yes(p, true);
    }
    concede(&mut t, P1);
    t.script.lock().unwrap().asked.clear();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1, "the cost wasn't paid, so P0 draws");
    // No one was asked whether to pay it in P1's place.
    assert!(
        !t.asked()
            .iter()
            .any(|(_, d)| matches!(d, Decision::YesNo { .. })),
        "{:?}",
        t.asked()
    );
}

#[test]
fn another_player_makes_a_choice_an_object_requires_of_a_player_who_left() {
    cr!("800.4g");
    // "At the beginning of each opponent's upkeep, that player may have you gain 3 life."
    let mut t = ffa(4);
    let gift = custom_with(
        "Tribute Collector",
        "Enchantment",
        None,
        vec![triggered(
            TriggerCond::BeginningOf {
                step: TriggerStep::Upkeep,
                whose: PlayerRel::Opponent,
            },
            Effect::May {
                who: PlayerRef::TriggerPlayer,
                effect: Box::new(Effect::GainLife {
                    who: PlayerRef::You,
                    n: Value::c(3),
                }),
            },
        )],
    );
    bf(&mut t, P0, gift);
    t.set_step(P0, Step::End);
    to_step(&mut t, P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    // P1 leaves with the ability on the stack. Its controller, P0, chooses another
    // opponent to make the choice.
    concede(&mut t, P1);
    t.script.lock().unwrap().asked.clear();
    t.answer_choose(P0, &[Entity::Player(P3)]);
    t.answer_yes(P3, true);
    t.resolve();
    assert_eq!(t.life(P0), 23, "P3 chose to have P0 gain life");
    let asked = t.asked();
    let substitute = asked.iter().find_map(|(p, d)| match d {
        Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.clone()),
        _ => None,
    });
    assert_eq!(
        substitute,
        Some(vec![Entity::Player(P2), Entity::Player(P3)]),
        "another opponent"
    );
    assert!(asked
        .iter()
        .any(|(p, d)| *p == P3 && matches!(d, Decision::YesNo { .. })));
    assert!(!asked.iter().any(|(p, _)| *p == P1));
}

#[test]
fn the_next_player_makes_a_choice_a_rule_requires_of_a_player_who_left() {
    cr!("800.4h");
    // Without the attack multiple players option, the active player chooses the defending
    // player as combat begins (CR 506.2a). P0 leaves during their main phase; the turn
    // continues, and the next player in turn order makes that choice.
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            attack_multiple_players: false,
            ..GameConfig::default()
        },
    );
    concede(&mut t, P0);
    t.script.lock().unwrap().asked.clear();
    go_to_any(&mut t, Step::BeginningOfCombat);
    let chooser = t
        .asked()
        .into_iter()
        .find(|(_, d)| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.contains("defending player")))
        .map(|(p, _)| p);
    assert_eq!(chooser, Some(P1));
}

#[test]
fn information_about_a_player_who_left_is_last_known() {
    cr!("800.4i");
    // P1 cast a spell this turn, then left: that action can still be found. Grapeshot's
    // storm count includes P1's spell.
    let mut t = ffa(3);
    let spell = t.custom(P1, super::r114_common::free_instant("Quick Thought"), Zone::Hand(P1));
    t.cast(P1, spell).go();
    t.resolve();
    concede(&mut t, P1);
    t.lands(P0, "Mountain", 2);
    let shot = t.hand(P0, "Grapeshot");
    t.answer_targets(P0, &[Entity::Player(P2)]);
    t.cast(P0, shot).target(P2).go();
    t.resolve_all();
    assert_eq!(t.life(P2), 18, "Grapeshot and one copy");
    // "That player's hand": the number of cards they had as they left.
    let mut t = ffa(3);
    // "At the beginning of each opponent's upkeep, you gain life equal to the number of
    // cards in that player's hand."
    let e = custom_with(
        "Counting Eye",
        "Enchantment",
        None,
        vec![triggered(
            TriggerCond::BeginningOf {
                step: TriggerStep::Upkeep,
                whose: PlayerRel::Opponent,
            },
            Effect::GainLife {
                who: PlayerRef::You,
                n: Value::HandSize(PlayerRef::TriggerPlayer),
            },
        )],
    );
    t.custom(P0, e, Zone::Battlefield);
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    t.set_step(P0, Step::End);
    to_step(&mut t, P1, Step::Upkeep);
    let n = t.hand_size(P1) as i32;
    assert!(n >= 3);
    concede(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.life(P0), 20 + n);
}

#[test]
fn a_turn_continues_without_an_active_player() {
    cr!("800.4j");
    let mut t = ffa(3);
    // P0 leaves during their precombat main phase.
    concede(&mut t, P0);
    assert_eq!(t.g.turn.active, P0);
    assert_eq!(t.g.turn.priority, Some(P1), "the next player gets priority");
    // The turn continues through its remaining steps; where the active player would
    // receive priority, the next player in turn order does.
    let order = priority_order_in(&mut t, Step::End);
    assert_eq!(t.g.turn.active, P0);
    assert_eq!(order, vec![P1, P2]);
    to_turn_of(&mut t, P1);
    assert_eq!(t.g.turn.previous_active, Some(P0));
}

#[test]
fn a_player_who_left_doesnt_begin_a_turn() {
    cr!("800.4k");
    // Time Warp: "Target player takes an extra turn after this one." P0 gives P1 an extra
    // turn, then P1 leaves.
    let mut t = ffa(3);
    t.lands(P0, "Island", 5);
    let warp = t.hand(P0, "Time Warp");
    t.cast(P0, warp).target(P1).go();
    t.resolve();
    assert_eq!(t.g.extra_turns, vec![P1]);
    concede(&mut t, P1);
    // Neither the extra turn nor P1's normal turn begins.
    to_turn_of(&mut t, P2);
    assert_eq!(t.g.turn.previous_active, Some(P0));
    to_turn_of(&mut t, P0);
    to_turn_of(&mut t, P2);
    assert_eq!(t.g.turn.previous_active, Some(P0));
}

#[test]
fn until_your_next_turn_effects_last_until_that_turn_would_have_begun() {
    cr!("800.4m");
    // Hag of Inner Weakness (P1): "At the beginning of your upkeep, target creature an
    // opponent controls gets -2/-1 until your next turn."
    let mut t = ffa(4);
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P1, "Hag of Inner Weakness");
    t.set_step(P0, Step::End);
    t.answer_targets(P1, &[Entity::Object(giant)]);
    to_step(&mut t, P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 2));
    // P1 leaves during P2's turn. The effect doesn't end immediately...
    to_turn_of(&mut t, P2);
    concede(&mut t, P1);
    t.g.recompute();
    assert_eq!(t.pt(giant), (1, 2));
    to_turn_of(&mut t, P3);
    assert_eq!(t.pt(giant), (1, 2));
    to_turn_of(&mut t, P0);
    assert_eq!(t.pt(giant), (1, 2));
    // ...nor last indefinitely: it ends when P1's turn would have begun.
    to_turn_of(&mut t, P2);
    t.g.recompute();
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn ante_cards_dont_leave_with_their_owner() {
    cr!("800.4n");
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            ante: true,
            ..GameConfig::free_for_all()
        },
    );
    let anted = t.custom(P1, vanilla("Wagered Bear", 2, 2), Zone::Ante);
    let hand = t.hand(P1, "Grizzly Bears");
    concede(&mut t, P1);
    assert_eq!(t.zone(anted), Zone::Ante);
    assert_eq!(t.g.ante, vec![anted]);
    assert_eq!(t.zone(hand), Zone::Nowhere);
}

#[test]
fn the_planar_controller_passes_on_before_leaving() {
    cr!("800.4p");
    // P1's Krosa is face up during P2's turn: P2 is the planar controller.
    let mut t = TestGame::with_config(
        3,
        GameConfig {
            variant: Variant::Planechase,
            ..GameConfig::default()
        },
    );
    let deck = super::r107_planechase::add_planar_deck(&mut t, P1, &["Krosa"]);
    mtg_engine::variants::turn_face_up_in_command(&mut t.g, deck[0]);
    let b = bear(&mut t, P0);
    t.set_step(P2, Step::PrecombatMain);
    assert_eq!(mtg_engine::planechase::planar_controller(&t.g), Some(P2));
    assert_eq!(t.obj(deck[0]).controller, P2);
    // P2 leaves: P0, the next player in turn order, becomes the planar controller first,
    // so the plane isn't exiled as something P2 controlled.
    concede(&mut t, P2);
    t.g.recompute();
    assert_eq!(mtg_engine::planechase::planar_controller(&t.g), Some(P0));
    assert_eq!(t.zone(deck[0]), Zone::Command);
    assert_eq!(t.obj(deck[0]).controller, P0);
    assert_eq!(t.pt(b), (4, 4));
}
