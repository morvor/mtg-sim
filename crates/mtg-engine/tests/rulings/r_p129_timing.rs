//! Rulings batch P129: reflexive triggered abilities ("When you do, ...", CR 603.12) go on
//! the stack after the payment / sacrifice / discard as the first ability resolves, and
//! players may respond before their effect happens; a cost paid "this way" is paid at most
//! once; an ability whose targets are all illegal doesn't resolve (CR 608.2b).

use crate::r_p129_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Untapped lands `p` controls.
fn untapped(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.is(CardType::Land) && !o.tapped)
        .count()
}

#[test]
fn ancestral_katana_attaches_when_the_reflexive_ability_resolves() {
    cr!("603.12", "301.5c");
    ruling!(
        "Ancestral Katana",
        "If you choose to pay {1} as Ancestral Katana's ability resolves, a second triggered ability triggers and players may respond to it."
    );
    supported("Ancestral Katana");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 1);
    let katana = t.battlefield(P0, "Ancestral Katana");
    let warrior = t.battlefield(P0, "Elvish Warrior");
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(warrior, Entity::Player(P1))]);
    resolve_into_reflexive(&mut t, &[]);
    assert_eq!(untapped(&t, P0), 0);
    assert_eq!(t.obj_now(katana).attached_to, None);
    t.resolve_all();
    assert_eq!(t.obj_now(katana).attached_to, Some(obj(warrior)));
}

#[test]
fn anointer_of_valor_reflexive_counter_and_one_payment() {
    cr!("603.12", "122.1");
    ruling!(
        "Anointer of Valor",
        "Players may respond to that reflexive triggered ability before the +1/+1 counter is put on the creature."
    );
    ruling!(
        "Anointer of Valor",
        "You can't pay {3} more than once to put more than one +1/+1 counter on the attacking creature."
    );
    supported("Anointer of Valor");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 6);
    t.battlefield(P0, "Anointer of Valor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    resolve_into_reflexive(&mut t, &[]);
    assert_eq!(t.counters(bears, counters::PLUS1), 0);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(untapped(&t, P0), 3);
}

#[test]
fn biting_palm_ninja_reveals_and_exiles_when_the_reflexive_ability_resolves() {
    cr!("603.12", "510.3a");
    ruling!(
        "Biting-Palm Ninja",
        "If you choose to remove a menace counter from Biting-Palm Ninja as its triggered ability resolves, a second triggered ability will go on the stack"
    );
    supported("Biting-Palm Ninja");
    let mut t = TestGame::new(2);
    let ninja = t.battlefield(P0, "Biting-Palm Ninja");
    put_counters(&mut t, ninja, "menace", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    attack_with(&mut t, &[(ninja, Entity::Player(P1))]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bolt)]);
    to_combat_damage_triggers(&mut t);
    resolve_into_reflexive(&mut t, &[]);
    assert_eq!(t.counters(ninja, "menace"), 0);
    assert!(t.in_hand(P1, "Lightning Bolt"));
    t.resolve_all();
    assert!(t.in_exile("Lightning Bolt"));
}

#[test]
fn behemoth_of_vault_0_targets_on_dying_and_pays_on_resolution() {
    cr!("603.12", "603.3d", "107.14");
    ruling!(
        "Behemoth of Vault 0",
        "You choose the target of Behemoth of Vault 0’s last ability when it triggers, but you don’t have to decide whether or not to pay until the ability resolves."
    );
    ruling!(
        "Behemoth of Vault 0",
        "a second “reflexive” ability triggers and is put onto the stack. This ability does not have a target."
    );
    supported("Behemoth of Vault 0");
    let mut t = TestGame::new(2);
    let behemoth = t.battlefield(P0, "Behemoth of Vault 0");
    give_energy(&mut t, P0, 4);
    let giant = t.battlefield(P1, "Hill Giant");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    destroy(&mut t, behemoth);
    assert_eq!(targets_of(&t, top(&t)), vec![obj(giant)]);
    assert!(target_candidates(&t, P0, from).len() == 1);
    assert_eq!(energy(&t, P0), 4);
    resolve_into_reflexive(&mut t, &[]);
    assert_eq!(energy(&t, P0), 0);
    assert!(t.on_battlefield(giant));
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
}

#[test]
fn caesars_attacking_tokens_dont_trigger_attack_abilities() {
    cr!("603.12", "508.4", "508.3a");
    ruling!(
        "Caesar, Legion's Emperor",
        "since they weren't declared as attacking creatures, no abilities that trigger whenever a creature attacks or whenever you attack"
    );
    supported("Caesar, Legion's Emperor");
    let mut t = TestGame::new(2);
    let caesar = t.battlefield(P0, "Caesar, Legion's Emperor");
    let thopter = t.battlefield(P0, "Ornithopter");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(thopter)]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1]));
    attack_with(&mut t, &[(caesar, Entity::Player(P1))]);
    t.resolve_all();
    let soldiers: Vec<_> = tokens(&t, P0);
    assert_eq!(soldiers.len(), 2);
    for s in &soldiers {
        assert!(t.g.is_attacking(*s));
    }
    // Caesar's own "whenever you attack" didn't trigger again.
    assert_eq!(t.stack_len(), 0);
    let yes_no = t
        .asked()
        .iter()
        .filter(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::YesNo { .. }))
        .count();
    assert_eq!(yes_no, 1);
}

#[test]
fn optional_costs_this_way_are_paid_at_most_once() {
    cr!("603.12", "117.3");
    ruling!(
        "Cavalier of Night",
        "While resolving Cavalier of Night’s enters-the-battlefield ability, you can’t sacrifice more than one creature."
    );
    ruling!(
        "Pyroclastic Hellion",
        "While resolving Pyroclastic Hellion’s ability, you can’t return more than one land to have it deal more damage."
    );
    ruling!(
        "Skyrider Patrol",
        "While resolving the triggered ability of Skyrider Patrol, you can’t pay {G}{U} multiple times to give more than one +1/+1 counter."
    );
    ruling!(
        "Tolarian Kraken",
        "While resolving Tolarian Kraken's ability, you can't pay more than {1} to tap or untap more than one creature."
    );
    // Cavalier of Night: offered two creatures, only one is sacrificed.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Ornithopter");
    let giant = t.battlefield(P1, "Hill Giant");
    t.enter(P0, "Cavalier of Night");
    t.settle();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    t.answer_targets(P0, &[obj(giant)]);
    t.resolve_all();
    assert_eq!(
        [a, b].iter().filter(|x| t.on_battlefield(**x)).count(),
        1,
        "exactly one creature sacrificed"
    );
    assert!(!t.on_battlefield(giant));
    // Pyroclastic Hellion: one land returned, 2 damage.
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Mountain", 2);
    t.enter(P0, "Pyroclastic Hellion");
    t.settle();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lands[0]), obj(lands[1])]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.life(P1), 18);
    // Skyrider Patrol: one counter, {G}{U} paid once.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 2);
    t.lands(P0, "Island", 2);
    t.battlefield(P0, "Skyrider Patrol");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    assert_eq!(untapped(&t, P0), 2);
    // Tolarian Kraken: {1} paid once, one creature tapped.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 3);
    t.battlefield(P0, "Tolarian Kraken");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_yes(P0, true);
    let draw = mtg_engine::ability::Effect::Draw {
        who: mtg_engine::ability::PlayerRef::You,
        n: mtg_engine::ability::Value::c(1),
    };
    run_with(&mut t, P0, draw, &[]);
    t.settle();
    t.resolve_all();
    assert!(t.obj_now(giant).tapped);
    assert_eq!(untapped(&t, P0), 2);
}

#[test]
fn electro_and_leyline_tyrant_keep_red_mana_while_they_stay() {
    cr!("106.4", "500.4");
    ruling!(
        "Electro, Assaulting Battery",
        "You can keep unspent red mana indefinitely while Electro is under your control."
    );
    ruling!(
        "Electro, Assaulting Battery",
        "Once Electro leaves the battlefield, you have until the end of the current step or phase to spend whatever red mana you have"
    );
    ruling!(
        "Leyline Tyrant",
        "You can keep unspent red mana indefinitely while Leyline Tyrant is on the battlefield."
    );
    ruling!(
        "Leyline Tyrant",
        "Once Leyline Tyrant leaves the battlefield, you have until the end of the current step or phase to spend your unspent red mana"
    );
    for name in ["Electro, Assaulting Battery", "Leyline Tyrant"] {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        add_mana(&mut t, P0, ManaType::R, 2);
        add_mana(&mut t, P0, ManaType::G, 1);
        t.advance_to(P0, Step::End);
        t.advance_to(P1, Step::Upkeep);
        let p = pool(&t, P0);
        assert_eq!((p[3], p[4]), (2, 0), "{name}: red kept, green lost");
        // It leaves: the red mana is lost as the step ends.
        t.answer_yes(P0, false);
        t.answer(P0, DecisionKind::Number, Answer::Number(0));
        destroy(&mut t, src);
        t.resolve_all();
        assert_eq!(pool(&t, P0)[3], 2, "{name}: kept until the step ends");
        t.advance_to(P1, Step::Draw);
        assert_eq!(pool(&t, P0)[3], 0, "{name}: lost afterward");
    }
}

#[test]
fn leyline_tyrant_pays_red_mana_then_the_reflexive_ability_targets() {
    cr!("603.12", "603.10a");
    ruling!(
        "Leyline Tyrant",
        "Leyline Tyrant's last ability goes on the stack without a target."
    );
    let mut t = TestGame::new(2);
    let tyrant = t.battlefield(P0, "Leyline Tyrant");
    add_mana(&mut t, P0, ManaType::R, 3);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::Number, Answer::Number(3));
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, tyrant);
    untargeted_then_reflexive(&mut t, P0, from, &[Entity::Player(P1)]);
    assert_eq!(pool(&t, P0)[3], 0);
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn blaster_hulk_divides_damage_as_the_reflexive_ability_goes_on_the_stack() {
    cr!("603.12", "601.2d", "107.14");
    ruling!(
        "Blaster Hulk",
        "You don't choose a target for Blaster Hulk's last ability at the time it triggers."
    );
    ruling!(
        "Blaster Hulk",
        "You choose how many targets Blaster Hulk's last ability has and how the damage is divided"
    );
    // (Its first ability, a cost reduction, isn't supported; the triggered ability is.)
    let mut t = TestGame::new(2);
    let hulk = t.battlefield(P0, "Blaster Hulk");
    give_energy(&mut t, P0, 6);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1), obj(bears)]);
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![6, 2]));
    attack_with(&mut t, &[(hulk, Entity::Player(P1))]);
    untargeted_then_reflexive(&mut t, P0, from, &[Entity::Player(P1), obj(bears)]);
    assert_eq!(energy(&t, P0), 0);
    assert_eq!(t.life(P1), 20);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 14);
}
