//! Policies the engine applies where the rules leave something to players or to procedure,
//! and its known deviations (docs/interpretations/): automatic mana payment, the
//! optimistic list of castable spells, conceding, the simulation's turn and action
//! limits, loops the engine doesn't recognize, and scaled combat taxes that come to
//! nothing.

use mtg_engine::decision::{Action, Answer};
use mtg_engine::game::{GameConfig, GameResult};
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn tapped(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).tapped
}

#[test]
fn mana_is_paid_from_the_cheapest_least_flexible_sources() {
    cr!("601.2g", "601.2h", "605.3a");
    interp!("automatic-mana-payment");
    // {R}: a Mountain rather than a Volcanic Island.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let island = t.battlefield(P0, "Volcanic Island");
    let mountain = t.battlefield(P0, "Mountain");
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(Entity::Player(P1)).go();
    assert!(tapped(&t, mountain));
    assert!(!tapped(&t, island));
    // {G}: a Forest rather than a creature's mana ability.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let elves = t.battlefield(P0, "Llanowar Elves");
    let forest = t.battlefield(P0, "Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    assert!(tapped(&t, forest));
    assert!(!tapped(&t, elves));
}

#[test]
fn a_spell_whose_reduction_depends_on_its_targets_is_listed_as_castable() {
    cr!("601.2f", "733.1");
    interp!("castability-is-optimistic");
    // Roadside Blowout ({2}{U}) "costs {2} less to cast if it targets a permanent with
    // mana value 1". With one Island it's listed even though no permanent with mana value
    // 1 is there; trying to cast it fails and is reversed.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 1);
    let blowout = t.hand(P0, "Roadside Blowout");
    let cast = Action::Cast {
        card: blowout,
        method: CastMethod::Normal,
    };
    assert!(t.g.legal_actions(P0).contains(&cast));
    assert!(t.cast(P0, blowout).target(giant).try_go().is_err());
    assert!(t.in_hand(P0, "Roadside Blowout"));
    assert!(t.on_battlefield(giant));
}

#[test]
fn a_player_concedes_by_answering_a_priority_decision() {
    cr!("104.3a");
    interp!("concede-with-priority");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Concession isn't among the engine's listed actions, but it's accepted with priority.
    assert!(!t.g.legal_actions(P0).contains(&Action::Concede));
    t.answer(P0, DecisionKind::Priority, Answer::Action(Action::Concede));
    t.g.advance();
    assert!(t.has_lost(P0));
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P1])));
}

#[test]
fn a_game_past_the_turn_or_action_limit_is_a_draw() {
    cr!("104.4a");
    interp!("simulation-limits");
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            max_turns: 3,
            ..GameConfig::default()
        },
    );
    assert_eq!(t.g.run(), GameResult::Draw);
    assert!(t.g.turn.number > 3);
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            max_actions: 40,
            ..GameConfig::default()
        },
    );
    assert_eq!(t.g.run(), GameResult::Draw);
    assert!(t.g.actions_taken > 40);
}

#[test]
fn a_loop_that_grows_the_game_state_runs_until_the_action_limit() {
    cr!("104.4b", "732.6");
    interp!("growing-loops-run-to-the-limit");
    // Two Charismatic Conquerors on each side ("Whenever an artifact or creature an
    // opponent controls enters untapped, they may tap that permanent. If they don't, you
    // create a 1/1 white Vampire creature token with flying."): only one trigger per
    // entering token can tap it, so each token makes another for the other side, without
    // end. The state never repeats (there are more tokens each time), so the engine never
    // recognizes the loop: the game is a draw only when it reaches the action limit.
    let mut t = TestGame::with_config(
        2,
        GameConfig {
            max_actions: 300,
            ..GameConfig::default()
        },
    );
    for p in [P0, P1] {
        t.battlefield(p, "Charismatic Conqueror");
        t.battlefield(p, "Charismatic Conqueror");
    }
    t.set_step(P0, Step::PrecombatMain);
    t.enter(P1, "Grizzly Bears");
    assert_eq!(t.g.run(), GameResult::Draw);
    assert!(t.g.actions_taken > 300, "ended by the action limit");
    assert!(t.named_on_battlefield("Vampire Token").len() > 10);
}

#[test]
fn a_scaled_attack_tax_that_comes_to_nothing_isnt_a_cost() {
    cr!("508.1d", "508.1h", "118.5");
    interp!("combat-tax-scaled-to-nothing");
    // Collective Restraint with no basic land types among P1's lands: X = 0, so the tax
    // is nothing, and Bloodrock Cyclops ("attacks each combat if able") must attack.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Collective Restraint");
    let cyclops = t.battlefield(P0, "Bloodrock Cyclops");
    t.set_step(P0, Step::BeginningOfCombat);
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(t.g.is_attacking(cyclops));
    // With a Forest, X = 1: a real cost, so the Cyclops doesn't have to attack.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Collective Restraint");
    t.battlefield(P1, "Forest");
    let cyclops = t.battlefield(P0, "Bloodrock Cyclops");
    t.lands(P0, "Mountain", 1);
    t.set_step(P0, Step::BeginningOfCombat);
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(!t.g.is_attacking(cyclops));
}
