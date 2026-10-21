//! Secret choices (`choices.rs`, `oracle/patterns/choice_grammar_hidden.rs`): "As ~
//! enters, secretly choose an opponent." with the cost "Reveal the player you chose",
//! "each opponent secretly chooses ..., then the choices are revealed". (What other
//! players see of a secret choice is tested through mtg-api observations in
//! `crates/mtg-api/tests/hidden_info.rs`.)

use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

#[test]
fn guardian_archon_secret_opponent_revealed_as_a_cost_gives_protection_from_them() {
    cr!("702.16a", "702.16b", "118.3");
    compiles("Guardian Archon");
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    let archon = t.enter(P0, "Guardian Archon");
    t.settle();
    let ch = &t.g.obj(archon).choices;
    assert_eq!(ch.player, Some(P2));
    assert_eq!(ch.secret, vec![P0], "chosen secretly");
    t.activate(P0, archon, 0, &[Entity::Object(archon)]).unwrap();
    assert!(
        t.g.obj(archon).choices.secret.is_empty(),
        "the cost reveals the choice"
    );
    t.resolve_all();
    // P2's spells can't target P0 or the Archon; P1's can.
    t.lands(P2, "Mountain", 1);
    t.lands(P1, "Mountain", 1);
    let bolt2 = t.hand(P2, "Lightning Bolt");
    let _ = t.cast(P2, bolt2).target(Entity::Player(P0)).try_go();
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "P0 can't be the target of P2's spells");
    assert_eq!(t.g.obj(archon).damage, 0);
    let bolt1 = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt1).target(Entity::Player(P0)).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 17, "only the chosen player");
}

#[test]
fn guardian_archon_without_a_chosen_opponent_the_reveal_cost_cant_be_paid() {
    cr!("118.3");
    ruling!(
        "Guardian Archon",
        "you can’t pay the cost to activate the last ability"
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    // Put onto the battlefield without entering through its replacement effect: no
    // opponent was chosen.
    let archon = t.battlefield(P0, "Guardian Archon");
    assert!(t.activate(P0, archon, 0, &[Entity::Object(archon)]).is_err());
}

#[test]
fn stalking_leonin_each_one_has_its_own_secret_opponent() {
    cr!("607.2d");
    ruling!(
        "Stalking Leonin",
        "you may choose a different opponent for each one"
    );
    let mut t = TestGame::new(3);
    t.set_step(P0, Step::PrecombatMain);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let a = t.enter(P0, "Stalking Leonin");
    t.resolve_all();
    t.answer_choose(P0, &[Entity::Player(P2)]);
    let b = t.enter(P0, "Stalking Leonin");
    t.resolve_all();
    assert_eq!(t.g.obj(a).choices.player, Some(P1));
    assert_eq!(t.g.obj(b).choices.player, Some(P2));
    assert_eq!(t.g.obj(a).choices.secret, vec![P0]);
}

#[test]
fn prisoners_dilemma_choices_stay_secret_until_all_are_made_then_are_revealed() {
    cr!("608.2c");
    let mut t = TestGame::new(3);
    t.lands(P0, "Mountain", 5);
    let spell = t.hand(P0, "Prisoner's Dilemma");
    t.advance_to(P0, Step::PrecombatMain);
    t.answer(P1, DecisionKind::Option, Answer::Index(1));
    t.answer(P2, DecisionKind::Option, Answer::Index(1));
    let id = t.cast(P0, spell).go();
    t.resolve_all();
    let ch = &t.g.obj(id).choices;
    assert_eq!(ch.by_player.len(), 2);
    assert!(ch.secret.is_empty(), "revealed once both chose");
    assert!(t.dump_log().contains("reveals their choice"));
}
