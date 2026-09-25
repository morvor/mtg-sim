//! CR 107.1–107.3: numbers, negative values, and X.

use super::r105_util::*;
use mtg_engine::ability::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// "Target player loses half their life, rounded up." (a custom sorcery).
fn halving_spell(up: bool) -> CardDef {
    let text = if up {
        "Target player loses half their life, rounded up."
    } else {
        "Target player loses half their life, rounded down."
    };
    card_from_text("Halver", "{0}", "Sorcery", None, text)
}

#[test]
fn fractions_are_rounded_as_the_text_says() {
    cr!("107.1", "107.1a");
    let mut t = TestGame::new(2);
    t.g.players[1].life = 15;
    let s = put_in_hand(&mut t, P0, halving_spell(true));
    t.cast(P0, s).target(P1).go();
    t.resolve();
    // Half of 15 is 7.5: rounded up, 8 life is lost.
    assert_eq!(t.life(P1), 7);
    let s = put_in_hand(&mut t, P0, halving_spell(false));
    t.cast(P0, s).target(P1).go();
    t.resolve();
    // Half of 7 rounded down is 3.
    assert_eq!(t.life(P1), 4);
}

#[test]
fn aspect_of_wolf_rounds_x_down_and_y_up() {
    cr!("107.1a", "107.3p");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 3);
    let aspect = t.battlefield(P0, "Aspect of Wolf");
    assert!(t.g.attach(aspect, Entity::Object(bears)));
    t.g.recompute();
    // Three Forests: X = 1 (half, rounded down), Y = 2 (half, rounded up).
    assert_eq!(t.pt(bears), (3, 4));
}

#[test]
fn power_can_be_negative_but_deals_no_damage() {
    cr!("107.1b");
    let mut t = TestGame::new(2);
    // A 3/4 creature gets -5/-0: it's a -2/4 creature.
    let c = put(&mut t, P0, vanilla("Three Four", "{3}", 3, 4));
    let shrink = card_with(
        "Shrink",
        "{0}",
        "Instant",
        None,
        vec![spell_ab(
            vec![TargetSpec::object(Filter::creature(), "target creature")],
            Effect::Modify {
                what: Sel::Target(0),
                mods: vec![Modification::ModifyPT(Value::c(-5), Value::c(0))],
                duration: Duration::EndOfTurn,
            },
        )],
    );
    let s = put_in_hand(&mut t, P0, shrink);
    t.cast(P0, s).target(c).go();
    t.resolve();
    assert_eq!(t.pt(c), (-2, 4));
    // It doesn't assign combat damage.
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(c, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 20);
    // Giving it +3/+0 raises its power to 1.
    t.lands(P0, "Forest", 1);
    let pump = card_with(
        "Pump",
        "{0}",
        "Instant",
        None,
        vec![spell_ab(
            vec![TargetSpec::object(Filter::creature(), "target creature")],
            Effect::Modify {
                what: Sel::Target(0),
                mods: vec![Modification::ModifyPT(Value::c(3), Value::c(0))],
                duration: Duration::EndOfTurn,
            },
        )],
    );
    let s = put_in_hand(&mut t, P0, pump);
    t.cast(P0, s).target(c).go();
    t.resolve();
    assert_eq!(t.pt(c), (1, 4));
}

#[test]
fn negative_calculation_yields_zero_viridian_joiner() {
    cr!("107.1b");
    let mut t = TestGame::new(2);
    // Viridian Joiner: "{T}: Add an amount of {G} equal to this creature's power."
    let joiner = t.battlefield(P0, "Viridian Joiner");
    t.g.effects.clear();
    let e = Effect::Modify {
        what: Sel::Target(0),
        mods: vec![Modification::ModifyPT(Value::c(-2), Value::c(0))],
        duration: Duration::EndOfTurn,
    };
    let s = put_in_hand(
        &mut t,
        P0,
        card_with(
            "Weaken",
            "{0}",
            "Instant",
            None,
            vec![spell_ab(
                vec![TargetSpec::object(Filter::creature(), "t")],
                e,
            )],
        ),
    );
    t.cast(P0, s).target(joiner).go();
    t.resolve();
    assert_eq!(t.pt(joiner), (-1, 2));
    t.activate(P0, joiner, 0, &[]).unwrap();
    // The ability adds no mana (and certainly doesn't remove any).
    assert_eq!(pool_total(&t, P0), 0);
}

#[test]
fn negative_calculation_yields_zero_chameleon_colossus() {
    cr!("107.1b", "107.3c");
    let mut t = TestGame::new(2);
    // Chameleon Colossus: "{2}{G}{G}: This creature gets +X/+X until end of turn, where X
    // is its power." With -6/-0 it's a -2/4; activating keeps it -2/4 (not -4/2).
    let colossus = t.battlefield(P0, "Chameleon Colossus");
    let e = Effect::Modify {
        what: Sel::Target(0),
        mods: vec![Modification::ModifyPT(Value::c(-6), Value::c(0))],
        duration: Duration::EndOfTurn,
    };
    let s = put_in_hand(
        &mut t,
        P0,
        card_with(
            "Weaken",
            "{0}",
            "Instant",
            None,
            vec![spell_ab(
                vec![TargetSpec::object(Filter::creature(), "t")],
                e,
            )],
        ),
    );
    t.cast(P0, s).target(colossus).go();
    t.resolve();
    assert_eq!(t.pt(colossus), (-2, 4));
    t.lands(P0, "Forest", 4);
    t.activate(P0, colossus, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.pt(colossus), (-2, 4));
}

#[test]
fn cant_gain_negative_life() {
    cr!("107.1b");
    let mut t = TestGame::new(2);
    // Infernal Reckoning: "Exile target colorless creature. You gain life equal to its
    // power." against a colorless creature with -2 power.
    let thopter = t.battlefield(P1, "Ornithopter");
    let e = Effect::Modify {
        what: Sel::Target(0),
        mods: vec![Modification::ModifyPT(Value::c(-2), Value::c(0))],
        duration: Duration::EndOfTurn,
    };
    let s = put_in_hand(
        &mut t,
        P0,
        card_with(
            "Weaken",
            "{0}",
            "Instant",
            None,
            vec![spell_ab(
                vec![TargetSpec::object(Filter::creature(), "t")],
                e,
            )],
        ),
    );
    t.cast(P0, s).target(thopter).go();
    t.resolve();
    assert_eq!(t.pt(thopter), (-2, 2));
    t.lands(P0, "Swamp", 1);
    let r = t.hand(P0, "Infernal Reckoning");
    t.cast(P0, r).target(thopter).go();
    t.resolve();
    assert!(t.in_exile("Ornithopter"));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn setting_power_to_a_negative_value_is_allowed() {
    cr!("107.1b");
    let mut t = TestGame::new(2);
    // An effect that sets base power to a calculated value keeps a negative result:
    // "Target creature has base power X, where X is the power of another target creature."
    let small = put(&mut t, P1, vanilla("Weakling", "{1}", 1, 3));
    let big = put(&mut t, P1, vanilla("Big", "{4}", 4, 4));
    let weaken = Effect::Modify {
        what: Sel::Target(0),
        mods: vec![Modification::ModifyPT(Value::c(-3), Value::c(0))],
        duration: Duration::EndOfTurn,
    };
    let s = put_in_hand(
        &mut t,
        P0,
        card_with(
            "Weaken",
            "{0}",
            "Instant",
            None,
            vec![spell_ab(
                vec![TargetSpec::object(Filter::creature(), "t")],
                weaken,
            )],
        ),
    );
    t.cast(P0, s).target(small).go();
    t.resolve();
    assert_eq!(t.pt(small).0, -2);
    let set = Effect::Modify {
        what: Sel::Target(1),
        mods: vec![Modification::SetPT(
            Some(Value::PowerOf(Box::new(Sel::Target(0)))),
            None,
        )],
        duration: Duration::EndOfTurn,
    };
    let s = put_in_hand(
        &mut t,
        P0,
        card_with(
            "Copy Power",
            "{0}",
            "Instant",
            None,
            vec![spell_ab(
                vec![
                    TargetSpec::object(Filter::creature(), "first"),
                    TargetSpec::object(Filter::creature(), "second"),
                ],
                set,
            )],
        ),
    );
    t.cast(P0, s)
        .targets(&[Entity::Object(small)])
        .target(big)
        .go();
    t.resolve();
    assert_eq!(t.pt(big), (-2, 4));
}

#[test]
fn any_number_means_zero_or_more() {
    cr!("107.1c");
    let mut t = TestGame::new(2);
    // X is "any number": a negative announcement isn't accepted.
    t.lands(P0, "Wastes", 2);
    let one = t.hand(P0, "Endless One");
    let id = t.cast(P0, one).x(-3).go();
    let x = t.g.obj(id).stack.as_ref().unwrap().x.unwrap();
    assert!(x >= 0, "X = {x}");
    t.resolve();
    // A "choose a number" with no upper bound still can't be negative.
    let chooser = card_with(
        "Number Chooser",
        "{0}",
        "Sorcery",
        None,
        vec![spell_ab(
            vec![],
            Effect::Choose {
                who: PlayerRef::You,
                kind: ChoiceKind::Number { min: 0, max: 99 },
            },
        )],
    );
    let s = put_in_hand(&mut t, P0, chooser);
    let id = t.cast(P0, s).go();
    t.answer(P0, DecisionKind::Number, Answer::Number(-4));
    t.resolve();
    assert!(t.g.obj(id).choices.number.unwrap() >= 0);
}

#[test]
fn undeterminable_numbers_are_zero() {
    cr!("107.2");
    let mut t = TestGame::new(2);
    // "You gain life equal to the greatest power among creatures you control" with no
    // creatures: that number can't be determined, so 0.
    let spell = card_with(
        "Power Gain",
        "{0}",
        "Sorcery",
        None,
        vec![spell_ab(
            vec![],
            Effect::GainLife {
                who: PlayerRef::You,
                n: Value::GreatestPower(Filter::creature().you_control()),
            },
        )],
    );
    let s = put_in_hand(&mut t, P0, spell);
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.life(P0), 20);
    // "Deals damage equal to target permanent's power" to a permanent with no power.
    let rock = t.battlefield(P1, "Sol Ring");
    let spell = card_with(
        "Power Burn",
        "{0}",
        "Sorcery",
        None,
        vec![spell_ab(
            vec![TargetSpec::object(Filter::Permanent, "target permanent")],
            Effect::DealDamage {
                source: Sel::This,
                amount: Value::PowerOf(Box::new(Sel::Target(0))),
                to: Sel::Players(PlayerRef::EachOpponent),
            },
        )],
    );
    let s = put_in_hand(&mut t, P0, spell);
    t.cast(P0, s).target(rock).go();
    t.resolve();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn x_is_announced_while_casting_and_is_that_value_on_the_stack() {
    cr!("107.3", "107.3a", "107.3i");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let blaze = t.hand(P0, "Blaze");
    let id = t.cast(P0, blaze).x(3).target(P1).go();
    // X was announced; the spell's mana value on the stack includes it.
    assert!(t
        .asked()
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseX { .. })));
    assert_eq!(t.g.mana_value_of(id), 4);
    // {X}{R} with X = 3 cost four mana.
    assert_eq!(
        t.g.battlefield
            .iter()
            .filter(|b| t.g.obj(**b).tapped)
            .count(),
        4
    );
    t.resolve();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn x_in_an_additional_cost_is_announced() {
    cr!("107.3a");
    let mut t = TestGame::new(2);
    // Fire Covenant: "As an additional cost to cast this spell, pay X life. Fire Covenant
    // deals X damage divided as you choose among any number of target creatures."
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    t.lands(P0, "Mountain", 1);
    let cov = t.hand(P0, "Fire Covenant");
    t.cast(P0, cov).x(2).target(bears).go();
    assert_eq!(t.life(P0), 18);
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn x_in_an_activation_cost_is_announced() {
    cr!("107.3a");
    let mut t = TestGame::new(2);
    // "{X}: Put X tower counters on this enchantment." (Helix Pinnacle)
    let helix = t.battlefield(P0, "Helix Pinnacle");
    t.lands(P0, "Forest", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, helix, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.counters(helix, "tower"), 2);
    // Each activation has its own X.
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, helix, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.counters(helix, "tower"), 3);
}

#[test]
fn activation_cost_x_is_independent_of_the_objects_x() {
    cr!("107.3k");
    let mut t = TestGame::new(2);
    // A permanent cast with X = 3 whose activated ability has {X} in its cost.
    let def = card_from_text(
        "X Tower",
        "{X}{G}",
        "Enchantment",
        None,
        "{X}: Put X tower counters on ~.",
    );
    t.lands(P0, "Forest", 4);
    let c = put_in_hand(&mut t, P0, def);
    t.cast(P0, c).x(3).go();
    t.resolve();
    let tower = t.named_on_battlefield("X Tower")[0];
    t.lands(P0, "Forest", 1);
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.activate(P0, tower, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.counters(tower, "tower"), 1);
}

#[test]
fn casting_without_paying_the_mana_cost_forces_x_to_zero() {
    cr!("107.3b");
    let mut t = TestGame::new(2);
    let blaze = t.hand(P0, "Blaze");
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![blaze],
        Duration::EndOfTurn,
        true,
        None,
    );
    let asked = t.asked().len();
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    let id = t.cast(P0, blaze).method(CastMethod::Free).target(P1).go();
    assert!(
        !t.asked()[asked..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseX { .. })),
        "the only legal choice for X is 0"
    );
    assert_eq!(t.g.obj(id).stack.as_ref().unwrap().x, Some(0));
    t.resolve();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn cost_reductions_dont_force_x_to_zero() {
    cr!("107.3b");
    let mut t = TestGame::new(2);
    // "Instant and sorcery spells you cast cost {1} less to cast."
    let reducer = card_with(
        "Reducer",
        "{2}",
        "Artifact",
        None,
        vec![static_ab(StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::Spells(Filter::Or(vec![
                Filter::Type(CardType::Instant),
                Filter::Type(CardType::Sorcery),
            ])),
            who: PlayerRel::You,
            change: CostChange::ReduceGeneric(Value::c(1)),
        }))],
    );
    put(&mut t, P0, reducer);
    t.lands(P0, "Mountain", 2);
    let blaze = t.hand(P0, "Blaze");
    // X = 2: {2}{R} reduced by {1} costs {1}{R}.
    t.cast(P0, blaze).x(2).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn x_defined_by_the_text_is_not_chosen_and_can_change() {
    cr!("107.3c");
    let mut t = TestGame::new(2);
    let colossus = t.battlefield(P0, "Chameleon Colossus");
    t.lands(P0, "Forest", 5);
    let before = t.asked().len();
    t.activate(P0, colossus, 0, &[]).unwrap();
    assert!(!t.asked()[before..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseX { .. })));
    // In response, it gets +3/+3: X is its power as the ability resolves (7).
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(colossus).go();
    t.resolve();
    assert_eq!(t.pt(colossus), (7, 7));
    t.resolve();
    assert_eq!(t.pt(colossus), (14, 14));
}

#[test]
fn x_of_another_object_uses_that_objects_value() {
    cr!("107.3e");
    let mut t = TestGame::new(2);
    // Zaxara: "Whenever you cast a spell with {X} in its mana cost, create a 0/0 green
    // Hydra creature token, then put X +1/+1 counters on it."
    t.battlefield(P0, "Zaxara, the Exemplary");
    t.lands(P0, "Mountain", 4);
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(3).target(P1).go();
    t.settle();
    t.resolve();
    let hydras = t.named_on_battlefield("Hydra Token");
    assert_eq!(hydras.len(), 1);
    assert_eq!(t.pt(hydras[0]), (3, 3));
}

#[test]
fn x_only_in_the_text_is_chosen_as_the_cost_is_paid() {
    cr!("107.3f");
    let mut t = TestGame::new(2);
    // Flameblast Dragon: "Whenever this creature attacks, you may pay {X}{R}. If you do,
    // it deals X damage to any target."
    let dragon = t.battlefield(P0, "Flameblast Dragon");
    t.lands(P0, "Mountain", 4);
    t.answer(
        P0,
        DecisionKind::Targets,
        Answer::Entities(vec![Entity::Player(P1)]),
    );
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(dragon, Entity::Player(P1))], &[]);
    // 3 from the ability, 5 combat damage.
    assert_eq!(t.life(P1), 12);
}

#[test]
fn x_chosen_as_an_optional_cost_is_paid_and_if_you_dont() {
    cr!("107.3f");
    let mut t = TestGame::new(2);
    // X appears only in the cost the ability asks for; declining to pay does the
    // "If you don't" part instead.
    let def = card_from_text(
        "Toll Shrine",
        "{1}",
        "Enchantment",
        None,
        "At the beginning of your upkeep, you may pay {X}. If you do, you gain X life. If you don't, you lose 2 life.",
    );
    put(&mut t, P0, def);
    t.lands(P0, "Plains", 3);
    t.set_step(P0, Step::Untap);
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.advance_to(P0, Step::Upkeep);
    t.resolve();
    assert_eq!(t.life(P0), 22);
    assert_eq!(
        t.g.battlefield
            .iter()
            .filter(|b| t.g.obj(**b).tapped)
            .count(),
        2
    );
    // Declined: no X is chosen and the other branch happens.
    t.set_step(P0, Step::Untap);
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(false));
    t.advance_to(P0, Step::Upkeep);
    t.resolve();
    assert_eq!(t.life(P0), 20);
}

#[test]
fn x_in_a_mana_cost_is_zero_outside_the_stack() {
    cr!("107.3g");
    let mut t = TestGame::new(2);
    let blaze = t.hand(P0, "Blaze");
    assert_eq!(t.g.mana_value_of(blaze), 1);
    let gy = t.graveyard(P0, "Blaze");
    assert_eq!(t.g.mana_value_of(gy), 1);
    // Endless One cast with X = 4 is a permanent with mana value 0.
    t.lands(P0, "Wastes", 4);
    let one = t.hand(P0, "Endless One");
    t.cast(P0, one).x(4).go();
    t.resolve();
    let perm = t.named_on_battlefield("Endless One")[0];
    assert_eq!(t.g.mana_value_of(perm), 0);
    assert!(matches(
        &t,
        perm,
        &Filter::ManaValue(Cmp::Eq, Box::new(Value::c(0))),
        P0
    ));
}

#[test]
fn paying_an_objects_mana_cost_with_x() {
    cr!("107.3h");
    let mut t = TestGame::new(2);
    // A permanent with {X} in its cost: "At the beginning of your upkeep, sacrifice this
    // creature unless you pay its mana cost." X is 0: it costs {G}.
    let hydra = card_from_text(
        "Upkeep Hydra",
        "{X}{G}",
        "Creature — Hydra",
        Some((3, 3)),
        "At the beginning of your upkeep, sacrifice ~ unless you pay its mana cost.",
    );
    let h = put(&mut t, P0, hydra);
    t.lands(P0, "Forest", 1);
    t.set_step(P0, Step::Untap);
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.advance_to(P0, Step::Upkeep);
    t.resolve();
    assert!(t.on_battlefield(h));
    assert_eq!(
        t.g.battlefield
            .iter()
            .filter(|b| t.g.obj(**b).tapped)
            .count(),
        1
    );

    // A spell on the stack: X is the value chosen as it was cast. "Counter target spell
    // unless its controller pays its mana cost" against Blaze with X = 2 costs {2}{R}.
    let mut t = TestGame::new(2);
    t.lands(P1, "Mountain", 3);
    let blaze = t.hand(P1, "Blaze");
    t.g.turn.priority = Some(P1);
    t.set_step(P1, Step::PrecombatMain);
    let spell = t.cast(P1, blaze).x(2).target(P0).go();
    t.lands(P1, "Mountain", 2); // only two more mana: {2}{R} can't be paid
    let tax = card_with(
        "Tax",
        "{0}",
        "Instant",
        None,
        vec![spell_ab(
            vec![TargetSpec {
                what: TargetKind::Spell(Filter::Spell),
                ..TargetSpec::object(Filter::Spell, "target spell")
            }],
            Effect::PayOptional {
                who: PlayerRef::ControllerOf(Box::new(Sel::Target(0))),
                cost: Cost {
                    mana: None,
                    parts: vec![CostPart::PayManaCostOf(Box::new(Sel::Target(0)))],
                },
                then: Box::new(Effect::Noop),
                otherwise: Box::new(Effect::CounterSpell {
                    what: Sel::Target(0),
                }),
            },
        )],
    );
    let tx = put_in_hand(&mut t, P0, tax);
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.cast(P0, tx).target(spell).go();
    t.resolve();
    assert!(
        t.in_graveyard(P1, "Blaze"),
        "couldn't pay {{2}}{{R}}, so it's countered"
    );
}

#[test]
fn a_gained_ability_uses_its_own_x_or_zero() {
    cr!("107.3j");
    let mut t = TestGame::new(2);
    // A creature cast with X = 3 gains "At the beginning of your upkeep, you gain X life"
    // (which doesn't define X): X is 0 for that ability. It also gains "... where X is its
    // power", which defines X.
    let hydra = card_with(
        "X Hydra",
        "{X}{G}",
        "Creature — Hydra",
        Some((2, 2)),
        vec![],
    );
    t.lands(P0, "Forest", 4);
    let h = put_in_hand(&mut t, P0, hydra);
    t.cast(P0, h).x(3).go();
    t.resolve();
    let perm = t.g.current(h);
    let undefined = triggered_ab(
        TriggerCond::BeginningOf {
            step: TriggerStep::Upkeep,
            whose: PlayerRel::You,
        },
        Effect::GainLife {
            who: PlayerRef::You,
            n: Value::X,
        },
    );
    let defined = triggered_ab(
        TriggerCond::BeginningOf {
            step: TriggerStep::Upkeep,
            whose: PlayerRel::You,
        },
        Effect::GainLife {
            who: PlayerRef::You,
            n: Value::PowerOf(Box::new(Sel::This)),
        },
    );
    let grant = card_with(
        "Grant",
        "{0}",
        "Sorcery",
        None,
        vec![spell_ab(
            vec![TargetSpec::object(Filter::creature(), "target creature")],
            Effect::Modify {
                what: Sel::Target(0),
                mods: vec![
                    Modification::AddAbility(undefined),
                    Modification::AddAbility(defined),
                ],
                duration: Duration::Permanent,
            },
        )],
    );
    let g = put_in_hand(&mut t, P0, grant);
    t.cast(P0, g).target(perm).go();
    t.resolve();
    t.set_step(P0, Step::Untap);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    // 0 from the first ability, 2 from the second.
    assert_eq!(t.life(P0), 22);
}

#[test]
fn enters_ability_uses_the_spells_x() {
    cr!("107.3m");
    let mut t = TestGame::new(2);
    // "When this creature enters, you gain X life." — X is the spell's X; the
    // permanent's own X is 0.
    let def = card_from_text(
        "X Healer",
        "{X}{W}",
        "Creature — Cleric",
        Some((1, 1)),
        "When ~ enters, you gain X life.",
    );
    t.lands(P0, "Plains", 5);
    let c = put_in_hand(&mut t, P0, def);
    t.cast(P0, c).x(4).go();
    t.resolve();
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    let perm = t.named_on_battlefield("X Healer")[0];
    assert_eq!(t.g.mana_value_of(perm), 1);
    // A replacement effect ("enters with X +1/+1 counters") also uses the spell's X.
    t.lands(P0, "Wastes", 3);
    let one = t.hand(P0, "Endless One");
    t.cast(P0, one).x(3).go();
    t.resolve();
    let e = t.named_on_battlefield("Endless One")[0];
    assert_eq!(t.counters(e, "+1/+1"), 3);
}

#[test]
fn an_enters_ability_of_a_spell_without_x_chooses_its_own_x() {
    cr!("107.3f", "107.3m");
    let mut t = TestGame::new(2);
    // Squealing Devil ({1}{R}): "When this creature enters, you may pay {X}. If you do,
    // target creature gets +X/+0 until end of turn." No X was chosen for the spell, so
    // the ability's X is chosen as its cost is paid.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 5);
    let devil = t.hand(P0, "Squealing Devil");
    t.cast(P0, devil).go();
    t.answer(
        P0,
        DecisionKind::Targets,
        Answer::Entities(vec![Entity::Object(bears)]),
    );
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    t.resolve();
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 2));
}

#[test]
fn delayed_trigger_uses_the_x_of_the_spell_that_created_it() {
    cr!("107.3n");
    let mut t = TestGame::new(2);
    // "{X}{W}: At the beginning of the next end step, you gain X life."
    let def = card_with(
        "Delayed Gift",
        "{X}{W}",
        "Sorcery",
        None,
        vec![spell_ab(
            vec![],
            Effect::AtNext {
                step: TriggerStep::End,
                effect: Box::new(Effect::GainLife {
                    who: PlayerRef::You,
                    n: Value::X,
                }),
            },
        )],
    );
    t.lands(P0, "Plains", 4);
    let s = put_in_hand(&mut t, P0, def);
    t.cast(P0, s).x(3).go();
    t.resolve();
    assert_eq!(t.life(P0), 20);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    let _ = (ManaType::W, Zone::Battlefield);
}
