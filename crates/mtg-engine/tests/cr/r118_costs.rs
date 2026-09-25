//! CR 118: costs.

use super::r114_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, Decision};
use mtg_engine::mana::{ManaCost, ManaType};
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn pool(t: &mut TestGame, p: PlayerId, types: &[ManaType]) {
    for ty in types {
        t.g.players[p.idx()].mana_pool.add_type(*ty, 1);
    }
}

fn pool_count(t: &TestGame, p: PlayerId, ty: ManaType) -> usize {
    t.player(p).mana_pool.count(ty)
}

/// "Spells you cast cost [mana] less to cast."
fn reducer(mana: &str) -> CardDef {
    CB::new("Reducer")
        .enchantment()
        .ability(stat(StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::Spells(Filter::Any),
            who: PlayerRel::You,
            change: CostChange::ReduceMana {
                mana: ManaCost::parse(mana).unwrap(),
                colored_only: false,
            },
        })))
        .build()
}

fn gain_spell(name: &str, cost: &str) -> CardDef {
    CB::new(name)
        .instant()
        .cost(cost)
        .spell(Body::effect(gain(1)))
        .build()
}

fn alt_cost(cost: Cost) -> Ability {
    stat(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ThisSpell,
        who: PlayerRel::You,
        change: CostChange::AlternativeCost(cost),
    }))
}

fn additional_cost(cost: Cost) -> Ability {
    let mut s = StaticAbility::new(StaticEffect::CostModifier(CostModifier {
        applies_to: CostTarget::ThisSpell,
        who: PlayerRel::You,
        change: CostChange::AdditionalCost(cost),
    }));
    s.zone = FunctionZone::Anywhere;
    AbilityDef::new(AbilityKind::Static(s), "additional cost")
}

fn sac_creature() -> CostPart {
    CostPart::Sacrifice {
        filter: Filter::creature(),
        count: Value::c(1),
    }
}

fn option_answer(t: &mut TestGame, p: PlayerId, i: usize) {
    t.answer(p, DecisionKind::Option, Answer::Index(i));
}

#[test]
fn paying_a_cost_carries_out_its_instructions_activating_mana_abilities() {
    cr!("118.1", "118.2");
    let mut t = TestGame::new(2);
    let altar = CB::new("Altar")
        .artifact()
        .ability(act(
            Cost::mana(mana("{1}")).with(sac_creature()),
            Body::effect(gain(2)),
        ))
        .build();
    let a = t.custom(P0, altar, Zone::Battlefield);
    let forest = t.lands(P0, "Forest", 1)[0];
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, a, 0, &[]).unwrap();
    // The mana ability of the land was activated during payment, the creature sacrificed.
    assert!(t.obj(forest).tapped);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn a_cost_cant_be_paid_without_the_resources() {
    cr!("118.3");
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    t.g.tap(pyro);
    assert!(t.activate(P0, pyro, 0, &[Entity::Player(P1)]).is_err());
    let drain = CB::new("Blood Tap")
        .artifact()
        .ability(act(
            Cost::free().with(CostPart::PayLife(Value::c(2))),
            Body::effect(draw(1)),
        ))
        .build();
    let d = t.custom(P0, drain, Zone::Battlefield);
    t.g.players[0].life = 1;
    assert!(t.activate(P0, d, 0, &[]).is_err());
    assert_eq!(t.life(P0), 1);
}

#[test]
fn paying_mana_removes_it_from_the_pool() {
    cr!("118.3a");
    let mut t = TestGame::new(2);
    pool(&mut t, P0, &[ManaType::R, ManaType::R, ManaType::G]);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P1).go();
    // The excess mana remains in the pool.
    assert_eq!(pool_count(&t, P0, ManaType::R), 1);
    assert_eq!(pool_count(&t, P0, ManaType::G), 1);
    t.resolve();
    // Anyone can pay 0 mana.
    t.g.players[0].mana_pool.empty();
    let thopter = t.hand(P0, "Ornithopter");
    t.cast(P0, thopter).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
}

#[test]
fn paying_life_subtracts_it_and_zero_life_can_always_be_paid() {
    cr!("118.3b");
    let mut t = TestGame::new(2);
    let tap = |n: i32| {
        CB::new("Blood Tap")
            .artifact()
            .ability(act(
                Cost::free().with(CostPart::PayLife(Value::c(n))),
                Body::effect(draw(1)),
            ))
            .build()
    };
    let three = t.custom(P0, tap(3), Zone::Battlefield);
    t.activate(P0, three, 0, &[]).unwrap();
    assert_eq!(t.life(P0), 17);
    t.resolve();
    let zero = t.custom(P0, tap(0), Zone::Battlefield);
    t.g.players[0].life = 0;
    t.activate(P0, zero, 0, &[]).unwrap();
    assert_eq!(t.life(P0), 0);
}

#[test]
fn activating_mana_abilities_is_not_mandatory() {
    cr!("118.3c");
    let mut t = TestGame::new(2);
    let forest = t.lands(P0, "Forest", 1)[0];
    let offer = CB::new("Toll")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::PayOptional {
            who: PlayerRef::You,
            cost: Cost::mana(mana("{1}")),
            then: Box::new(gain(3)),
            otherwise: Box::new(Effect::Noop),
        }))
        .build();
    let s = t.custom(P0, offer, Zone::Hand(P0));
    t.answer_yes(P0, false);
    t.cast(P0, s).go();
    t.resolve();
    // The player didn't activate the land's mana ability, so the cost wasn't paid.
    assert!(!t.obj(forest).tapped);
    assert_eq!(t.life(P0), 20);
    // Mana already in the pool pays a cost without mana abilities being activated.
    pool(&mut t, P0, &[ManaType::G, ManaType::G]);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    assert!(!t.obj(forest).tapped);
}

#[test]
fn costs_with_x() {
    cr!("118.4");
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Plains", 4);
    let ballista = t.hand(P0, "Walking Ballista");
    let s = t.cast(P0, ballista).x(2).go();
    // X = 2 was paid: {X}{X} is four mana.
    assert!(lands.iter().all(|l| t.obj(*l).tapped));
    assert!(t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseX { .. })));
    assert_eq!(t.obj(s).stack.as_ref().unwrap().x, Some(2));
    t.resolve();
    let b = t.named_on_battlefield("Walking Ballista")[0];
    assert_eq!(t.counters(b, counters::PLUS1), 2);
}

#[test]
fn zero_costs_must_still_be_acknowledged() {
    cr!("118.5", "118.5a");
    let mut t = TestGame::new(2);
    let thopter = t.hand(P0, "Ornithopter");
    let free = CB::new("Idle Engine")
        .artifact()
        .ability(act(Cost::mana(mana("{0}")), Body::effect(gain(1))))
        .build();
    let engine = t.custom(P0, free, Zone::Battlefield);
    // Nothing is cast or activated automatically: the players just pass.
    t.g.take_action(P0, Action::Pass);
    t.g.take_action(P1, Action::Pass);
    assert!(t.in_hand(P0, "Ornithopter"));
    assert_eq!(t.life(P0), 20);
    // A {0} spell is cast like any other spell, and a {0} ability is activated.
    t.set_step(P0, Step::PrecombatMain);
    let s = t.cast(P0, thopter).go();
    assert_eq!(t.obj(s).zone, Zone::Stack);
    assert!(!events_matching(&t, |e| matches!(e, mtg_engine::events::Event::SpellCast { .. }))
        .is_empty());
    t.resolve();
    t.activate(P0, engine, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn an_object_with_no_mana_cost_has_an_unpayable_cost() {
    cr!("118.6");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let vision = t.hand(P0, "Ancestral Vision");
    // Attempting to cast it is legal; paying the unpayable cost isn't, so the attempt is
    // reversed.
    assert!(t.cast(P0, vision).try_go().is_err());
    assert!(t.in_hand(P0, "Ancestral Vision"));
    assert!(t.permanents().all(|o| !o.tapped));
    assert!(!t
        .g
        .legal_actions(P0)
        .iter()
        .any(|a| matches!(a, Action::Cast { card, .. } if *card == vision)));
}

#[test]
fn an_unpayable_cost_stays_unpayable_but_alternative_costs_can_be_paid() {
    cr!("118.6a");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    t.lands(P0, "Island", 5);
    let vision = t.hand(P0, "Ancestral Vision");
    // Increased by {1}: still unpayable.
    assert!(t.cast(P0, vision).try_go().is_err());
    // "Cast it without paying its mana cost" (an alternative cost) can be paid; the
    // cost increase applies to it (CR 118.9d).
    t.g.play_grants.push(mtg_engine::casting::PlayGrant {
        player: P0,
        object: vision,
        duration: Duration::EndOfTurn,
        free: true,
        source: None,
        turn: 1,
    });
    t.cast(P0, vision).method(CastMethod::Free).go();
    assert_eq!(t.permanents().filter(|o| o.tapped).count(), 1);
    t.resolve();
    assert_eq!(t.hand_size(P0), 3);
}

#[test]
fn a_cost_reduced_to_nothing_is_zero_and_counts_as_paying_the_original() {
    cr!("118.7");
    let mut t = TestGame::new(2);
    let artifact_discount = CB::new("Foundry")
        .enchantment()
        .ability(stat(StaticEffect::CostModifier(CostModifier {
            applies_to: CostTarget::Spells(Filter::Type(CardType::Artifact)),
            who: PlayerRel::You,
            change: CostChange::ReduceGeneric(Value::c(2)),
        })))
        .build();
    t.custom(P0, artifact_discount, Zone::Battlefield);
    let sable = t.hand(P0, "Bronze Sable");
    // {2} reduced to {0}: cast with no mana; it was cast for its (reduced) mana cost, not
    // for an alternative cost.
    let s = t.cast(P0, sable).go();
    assert_eq!(
        t.obj(s).stack.as_ref().unwrap().cast.method,
        CastMethod::Normal
    );
    t.resolve();
    assert_eq!(t.named_on_battlefield("Bronze Sable").len(), 1);
}

#[test]
fn generic_reductions_reduce_only_the_generic_component() {
    cr!("118.7a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Goblin Electromancer");
    let bolt = t.hand(P0, "Lightning Bolt");
    // Lightning Bolt still costs {R}.
    assert!(t.cast(P0, bolt).target(P1).try_go().is_err());
    pool(&mut t, P0, &[ManaType::R]);
    t.cast(P0, bolt).target(P1).go();
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    // A sorcery with generic mana in its cost costs {1} less.
    t.resolve();
    pool(&mut t, P0, &[ManaType::U, ManaType::U]);
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    assert_eq!(t.player(P0).mana_pool.total(), 0);
}

#[test]
fn a_reduction_by_a_color_the_cost_lacks_reduces_generic_mana() {
    cr!("118.7b");
    let mut t = TestGame::new(2);
    t.custom(P0, reducer("{W}"), Zone::Battlefield);
    pool(&mut t, P0, &[ManaType::U, ManaType::U]);
    let div = t.hand(P0, "Divination");
    // {2}{U} reduced by {W}: {1}{U}.
    t.cast(P0, div).go();
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    t.resolve();
    // A colorless reduction on a cost with no {C}.
    let mut t = TestGame::new(2);
    t.custom(P0, reducer("{C}"), Zone::Battlefield);
    pool(&mut t, P0, &[ManaType::U, ManaType::U]);
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    assert_eq!(t.player(P0).mana_pool.total(), 0);
}

#[test]
fn excess_colored_reduction_reduces_the_generic_component() {
    cr!("118.7c");
    let mut t = TestGame::new(2);
    t.custom(P0, reducer("{G}{G}"), Zone::Battlefield);
    // {1}{G} reduced by {G}{G}: {0}.
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn excess_colorless_reduction_reduces_the_generic_component() {
    cr!("118.7d");
    let mut t = TestGame::new(2);
    t.custom(P0, reducer("{C}{C}"), Zone::Battlefield);
    // {1}{C} reduced by {C}{C}: the {C} is gone and the generic {1} too.
    let s = t.custom(P0, gain_spell("Void Sip", "{1}{C}"), Zone::Hand(P0));
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn the_player_chooses_a_half_of_a_hybrid_reduction() {
    cr!("118.7e");
    // Divination {2}{U} reduced by {W/U}: the blue half makes it {2}, the white half
    // {1}{U}.
    for (half, castable_with_ww) in [(1, true), (0, false)] {
        let mut t = TestGame::new(2);
        t.custom(P0, reducer("{W/U}"), Zone::Battlefield);
        pool(&mut t, P0, &[ManaType::W, ManaType::W]);
        let div = t.hand(P0, "Divination");
        option_answer(&mut t, P0, half);
        assert_eq!(t.cast(P0, div).try_go().is_ok(), castable_with_ww);
    }
    // A {2/W} reduction's generic half reduces by two generic.
    let mut t = TestGame::new(2);
    t.custom(P0, reducer("{2/W}"), Zone::Battlefield);
    pool(&mut t, P0, &[ManaType::U]);
    let div = t.hand(P0, "Divination");
    option_answer(&mut t, P0, 1);
    t.cast(P0, div).go();
}

#[test]
fn a_phyrexian_reduction_reduces_one_mana_of_its_color() {
    cr!("118.7f");
    let mut t = TestGame::new(2);
    t.custom(P0, reducer("{B/P}"), Zone::Battlefield);
    pool(&mut t, P0, &[ManaType::G]);
    let s = t.custom(P0, gain_spell("Dark Sip", "{1}{B}"), Zone::Hand(P0));
    // {1}{B} reduced by {B/P}: {1}.
    t.cast(P0, s).go();
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn a_snow_reduction_reduces_generic_mana() {
    cr!("118.7g");
    let mut t = TestGame::new(2);
    t.custom(P0, reducer("{S}"), Zone::Battlefield);
    pool(&mut t, P0, &[ManaType::G]);
    let bears = t.hand(P0, "Grizzly Bears");
    // {1}{G} reduced by {S}: {G}.
    t.cast(P0, bears).go();
    assert_eq!(t.player(P0).mana_pool.total(), 0);
}

#[test]
fn additional_costs_are_paid_along_with_the_mana_cost() {
    cr!("118.8");
    let mut t = TestGame::new(2);
    let victim = t.battlefield(P0, "Grizzly Bears");
    let target = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 1);
    let splinters = t.hand(P0, "Bone Splinters");
    t.answer_choose(P0, &[Entity::Object(victim)]);
    t.cast(P0, splinters).target(target).go();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve();
    assert!(!t.on_battlefield(target));
    // Without a creature to sacrifice, it can't be cast.
    let splinters2 = t.hand(P0, "Bone Splinters");
    t.lands(P0, "Swamp", 1);
    let giant2 = t.battlefield(P1, "Hill Giant");
    assert!(t.cast(P0, splinters2).target(giant2).try_go().is_err());
}

#[test]
fn several_additional_costs_can_apply_and_some_are_optional() {
    cr!("118.8a", "118.8b");
    let mut t = TestGame::new(2);
    let mut kicked = CB::new("Costly Gift")
        .sorcery()
        .cost("{1}")
        .keyword(mtg_engine::keywords::KeywordKind::Kicker)
        .ability(additional_cost(
            Cost::free().with(CostPart::PayLife(Value::c(2))),
        ))
        .spell(Body::effect(draw(1)))
        .build();
    // Kicker {1}.
    if let AbilityKind::Keyword(k) = &mut std::sync::Arc::make_mut(
        kicked.faces[0]
            .chars
            .abilities
            .iter_mut()
            .find(|a| a.keyword().is_some())
            .unwrap(),
    )
    .kind
    {
        k.cost = Some(Cost::mana(mana("{1}")));
    }
    let s = t.custom(P0, kicked.clone(), Zone::Hand(P0));
    pool(&mut t, P0, &[ManaType::C, ManaType::C]);
    // Both the mandatory additional cost and the optional kicker cost are paid.
    t.cast(P0, s).kicked(true).go();
    assert_eq!(t.life(P0), 18);
    assert_eq!(t.player(P0).mana_pool.total(), 0);
    t.resolve();
    // The optional one may be declined.
    let s2 = t.custom(P0, kicked, Zone::Hand(P0));
    pool(&mut t, P0, &[ManaType::C, ManaType::C]);
    t.cast(P0, s2).kicked(false).go();
    assert_eq!(t.life(P0), 16);
    assert_eq!(t.player(P0).mana_pool.total(), 1);
}

#[test]
fn casting_if_able_isnt_required_with_a_hidden_additional_cost_with_a_quality() {
    cr!("118.8c");
    let offering = CB::new("Forced Offering")
        .instant()
        .cost("{0}")
        .ability(additional_cost(Cost::free().with(CostPart::Discard {
            filter: Filter::creature(),
            count: Value::c(1),
            random: false,
        })))
        .spell(Body::effect(gain(3)))
        .build();
    let compel = |name: &str| {
        CB::new("Compulsion")
            .sorcery()
            .cost("{0}")
            .spell(Body::effect(Effect::CastCard {
                who: PlayerRef::You,
                what: Sel::All(Filter::and(vec![
                    Filter::Named(name.into()),
                    Filter::InZone(ZoneKind::Hand),
                ])),
                free: true,
                optional: false,
            }))
            .build()
    };
    let mut t = TestGame::new(2);
    t.custom(P0, offering, Zone::Hand(P0));
    t.hand(P0, "Grizzly Bears");
    let c = t.custom(P0, compel("Forced Offering"), Zone::Hand(P0));
    t.answer_yes(P0, false);
    t.cast(P0, c).go();
    t.resolve_all();
    // P0 chose not to cast it, even though a creature card was in hand.
    assert!(t.in_hand(P0, "Forced Offering"));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    // A spell without such a cost must be cast.
    let mut t = TestGame::new(2);
    t.custom(P0, gain_spell("Plain Gift", "{0}"), Zone::Hand(P0));
    let c = t.custom(P0, compel("Plain Gift"), Zone::Hand(P0));
    t.answer_yes(P0, false);
    t.cast(P0, c).go();
    t.resolve_all();
    assert!(!t.in_hand(P0, "Plain Gift"));
    assert_eq!(t.life(P0), 21);
}

#[test]
fn additional_costs_dont_change_the_mana_cost() {
    cr!("118.8d");
    let mut t = TestGame::new(2);
    let victim = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 1);
    let splinters = t.hand(P0, "Bone Splinters");
    t.answer_choose(P0, &[Entity::Object(victim)]);
    let s = t.cast(P0, splinters).target(giant).go();
    assert_eq!(t.obj(s).chars.mana_value(), 1);
    assert_eq!(
        t.obj(s).chars.mana_cost.as_ref().unwrap().to_string(),
        "{B}"
    );
}

fn two_alternatives() -> CardDef {
    CB::new("Twin Path")
        .sorcery()
        .cost("{4}{U}{U}")
        .ability(alt_cost(
            Cost::free().with(CostPart::PayLife(Value::c(3))),
        ))
        .ability(alt_cost(Cost::free().with(sac_creature())))
        .spell(Body::effect(draw(1)))
        .build()
}

#[test]
fn alternative_costs_replace_the_mana_cost_and_only_one_applies() {
    cr!("118.9", "118.9a");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let c = t.custom(P0, two_alternatives(), Zone::Hand(P0));
    let alts: Vec<CastMethod> = t
        .cast_options(P0, c)
        .into_iter()
        .map(|o| o.method)
        .filter(|m| matches!(m, CastMethod::Alternative(_)))
        .collect();
    // Two separate ways to cast it; each applies a single alternative cost.
    assert_eq!(alts.len(), 2);
    t.cast(P0, c).method(alts[0].clone()).go();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn alternative_costs_are_optional_and_effects_may_require_one() {
    cr!("118.9b");
    let mut t = TestGame::new(2);
    let c = t.custom(P0, two_alternatives(), Zone::Hand(P0));
    // The mana cost can still be paid.
    t.lands(P0, "Island", 6);
    t.cast(P0, c).go();
    assert_eq!(t.life(P0), 20);
    t.resolve();
    // An effect lets P0 cast a card only by paying a certain alternative cost: "without
    // paying its mana cost".
    let c2 = t.custom(P0, two_alternatives(), Zone::Hand(P0));
    let compel = CB::new("Free Cast")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::CastCard {
            who: PlayerRef::You,
            what: Sel::All(Filter::and(vec![
                Filter::Named("Twin Path".into()),
                Filter::InZone(ZoneKind::Hand),
            ])),
            free: true,
            optional: false,
        }))
        .build();
    let s = t.custom(P0, compel, Zone::Hand(P0));
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.g.current(c2), *t.stack.last().unwrap());
    assert_eq!(t.life(P0), 20);
}

#[test]
fn alternative_costs_dont_change_the_mana_cost() {
    cr!("118.9c");
    let mut t = TestGame::new(2);
    let c = t.custom(P0, two_alternatives(), Zone::Hand(P0));
    let alts: Vec<CastMethod> = t
        .cast_options(P0, c)
        .into_iter()
        .map(|o| o.method)
        .filter(|m| matches!(m, CastMethod::Alternative(_)))
        .collect();
    let s = t.cast(P0, c).method(alts[0].clone()).go();
    assert_eq!(t.obj(s).chars.mana_value(), 6);
}

#[test]
fn cost_changes_apply_to_an_alternative_cost() {
    cr!("118.9d");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thalia, Guardian of Thraben");
    let c = t.custom(P0, two_alternatives(), Zone::Hand(P0));
    let alts: Vec<CastMethod> = t
        .cast_options(P0, c)
        .into_iter()
        .map(|o| o.method)
        .filter(|m| matches!(m, CastMethod::Alternative(_)))
        .collect();
    // Thalia's {1} increase applies to the "pay 3 life" alternative cost.
    assert!(t.cast(P0, c).method(alts[0].clone()).try_go().is_err());
    pool(&mut t, P0, &[ManaType::C]);
    t.cast(P0, c).method(alts[0].clone()).go();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.player(P0).mana_pool.total(), 0);
}

#[test]
fn each_payment_applies_to_only_one_cost() {
    cr!("118.10");
    let mut t = TestGame::new(2);
    let altar = || {
        CB::new("Altar")
            .artifact()
            .ability(act(Cost::free().with(sac_creature()), Body::effect(gain(1))))
            .build()
    };
    let a = t.custom(P0, altar(), Zone::Battlefield);
    let b = t.custom(P0, altar(), Zone::Battlefield);
    t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, a, 0, &[]).unwrap();
    // The one creature was sacrificed for the first ability; it can't pay the second.
    assert!(t.activate(P0, b, 0, &[]).is_err());
    // Nor does resolving an ability that sacrifices a creature pay a cost.
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

#[test]
fn a_cost_is_paid_even_if_its_actions_are_modified() {
    cr!("118.11");
    let mut t = TestGame::new(2);
    // "If a creature would die, exile it instead."
    let rip = CB::new("Resting Place")
        .enchantment()
        .ability(stat(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::Dies(Filter::creature()),
            action: ReplacementAction::MoveInstead(Destination::zone(ZoneKind::Exile)),
            self_replacement: false,
            optional: false,
        })))
        .build();
    t.custom(P0, rip, Zone::Battlefield);
    let victim = t.battlefield(P0, "Grizzly Bears");
    let target = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 1);
    let splinters = t.hand(P0, "Bone Splinters");
    t.answer_choose(P0, &[Entity::Object(victim)]);
    t.cast(P0, splinters).target(target).go();
    // The sacrificed creature was exiled, but the cost was paid.
    assert!(t.in_exile("Grizzly Bears"));
    t.resolve();
    assert!(!t.on_battlefield(target));
}

fn standstill() -> CardDef {
    // "When a player casts a spell, sacrifice this enchantment. If you do, you draw three
    // cards."
    CB::new("Stillness Pact")
        .enchantment()
        .ability(trig(
            TriggerCond::CastSpell {
                who: PlayerRel::Any,
                filter: Filter::Any,
            },
            Body::effect(Effect::seq(vec![
                Effect::SacrificeObjects { what: Sel::This },
                Effect::If {
                    cond: Condition::PrevHappened,
                    then: Box::new(draw(3)),
                    otherwise: Box::new(Effect::Noop),
                },
            ])),
        ))
        .build()
}

#[test]
fn if_you_do_checks_whether_the_cost_was_paid_not_what_happened() {
    cr!("118.12");
    let mut t = TestGame::new(2);
    t.custom(P0, standstill(), Zone::Battlefield);
    let s = t.custom(P1, free_instant("Trigger"), Zone::Hand(P1));
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, s).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 3);
    // The enchantment is exiled with the trigger on the stack: it can't be sacrificed,
    // so nobody draws.
    let mut t = TestGame::new(2);
    let pact = t.custom(P0, standstill(), Zone::Battlefield);
    let s = t.custom(P1, free_instant("Trigger"), Zone::Hand(P1));
    t.set_step(P1, Step::PrecombatMain);
    t.cast(P1, s).go();
    t.settle();
    t.g.exile_object(pact, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    // Choosing to pay counts even if the action is modified: the creature P0 puts onto
    // the battlefield enters under P1's control, and P0 still "did".
    let mut t = TestGame::new(2);
    let gather = CB::new("Gathered")
        .enchantment()
        .ability(stat(StaticEffect::Replacement(ReplacementDef {
            event: ReplacementEvent::EntersBattlefield(Filter::creature()),
            action: ReplacementAction::EnterUnderControl(PlayerRef::Player(P1)),
            self_replacement: false,
            optional: false,
        })))
        .build();
    t.custom(P1, gather, Zone::Battlefield);
    t.hand(P0, "Grizzly Bears");
    let spell = CB::new("Unleash")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::seq(vec![
            Effect::May {
                who: PlayerRef::You,
                effect: Box::new(Effect::Move {
                    what: Sel::Choose {
                        chooser: PlayerRef::You,
                        filter: Filter::and(vec![Filter::creature(), Filter::InZone(ZoneKind::Hand)]),
                        count: Value::c(1),
                        up_to: false,
                        store: None,
                    },
                    to: Destination::battlefield(),
                }),
            },
            Effect::If {
                cond: Condition::PrevHappened,
                then: Box::new(draw(1)),
                otherwise: Box::new(Effect::Noop),
            },
        ])))
        .build();
    let s = t.custom(P0, spell, Zone::Hand(P0));
    t.answer_yes(P0, true);
    t.cast(P0, s).go();
    t.resolve();
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.obj(bears).controller, P1);
    assert_eq!(t.hand_size(P0), 1);
}

#[test]
fn unless_means_a_player_may_do_something_and_if_they_dont() {
    cr!("118.12a");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.hand(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 2);
    let islands = t.lands(P1, "Island", 3);
    let spell = t.cast(P1, bears).go();
    t.lands(P0, "Island", 2);
    let leak = t.hand(P0, "Mana Leak");
    t.cast(P0, leak).target(spell).go();
    // The spell's controller may pay {3}; they do.
    t.answer_yes(P1, true);
    t.resolve();
    assert!(islands.iter().all(|i| t.obj(*i).tapped));
    assert!(t.stack.contains(&spell));
    t.resolve();
    // Another: they don't pay, and the spell is countered.
    let bears2 = t.hand(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 2);
    t.lands(P1, "Island", 3);
    let spell2 = t.cast(P1, bears2).go();
    t.lands(P0, "Island", 2);
    let leak2 = t.hand(P0, "Mana Leak");
    t.cast(P0, leak2).target(spell2).go();
    t.answer_yes(P1, false);
    t.resolve();
    assert!(!t.stack.contains(&spell2));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn if_you_do_after_a_search_checks_whether_the_player_searched() {
    cr!("118.12b");
    let mut t = TestGame::new(2);
    // "You may search your library for a basic land card, put it onto the battlefield,
    // then shuffle. If you do, you gain 2 life." No basic land is in the library.
    let spell = CB::new("Scout Ahead")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::seq(vec![
            Effect::May {
                who: PlayerRef::You,
                effect: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    whose: PlayerRef::You,
                    filter: Filter::and(vec![
                        Filter::Type(CardType::Land),
                        Filter::Supertype(Supertype::Basic),
                    ]),
                    count: Value::c(1),
                    to: Destination::battlefield(),
                    reveal: false,
                    shuffle: true,
                }),
            },
            Effect::If {
                cond: Condition::PrevHappened,
                then: Box::new(gain(2)),
                otherwise: Box::new(Effect::Noop),
            },
        ])))
        .build();
    let s = t.custom(P0, spell, Zone::Hand(P0));
    t.answer_yes(P0, true);
    t.cast(P0, s).go();
    t.resolve();
    assert!(t.permanents().all(|o| !o.is(CardType::Land)));
    assert_eq!(t.life(P0), 22);
}

#[test]
fn the_player_chooses_how_to_pay_hybrid_and_phyrexian_symbols_as_they_cast() {
    cr!("118.13", "118.13a");
    let mut t = TestGame::new(2);
    let forest = t.lands(P0, "Forest", 1)[0];
    let bears = t.battlefield(P0, "Grizzly Bears");
    let growth = t.hand(P0, "Mutagenic Growth");
    // {G/P}: options are "either", {G}, "2 life". P0 pays life even though a Forest is
    // available.
    option_answer(&mut t, P0, 2);
    t.cast(P0, growth).target(bears).go();
    assert_eq!(t.life(P0), 18);
    assert!(!t.obj(forest).tapped);
    t.resolve();
    // A hybrid symbol: P0 announces the green half.
    pool(&mut t, P0, &[ManaType::R, ManaType::G]);
    let s = t.custom(P0, gain_spell("Wild Sip", "{R/G}"), Zone::Hand(P0));
    option_answer(&mut t, P0, 2);
    t.cast(P0, s).go();
    assert_eq!(pool_count(&t, P0, ManaType::R), 1);
    assert_eq!(pool_count(&t, P0, ManaType::G), 0);
}

#[test]
fn the_choice_for_a_cost_paid_during_resolution_is_made_before_paying() {
    cr!("118.13b");
    let mut t = TestGame::new(2);
    let forest = t.lands(P0, "Forest", 1)[0];
    let spell = CB::new("Offer of Blood")
        .sorcery()
        .cost("{0}")
        .spell(Body::effect(Effect::PayOptional {
            who: PlayerRef::You,
            cost: Cost::mana(mana("{G/P}")),
            then: Box::new(gain(5)),
            otherwise: Box::new(Effect::Noop),
        }))
        .build();
    let s = t.custom(P0, spell, Zone::Hand(P0));
    t.answer_yes(P0, true);
    option_answer(&mut t, P0, 2);
    t.cast(P0, s).go();
    t.resolve();
    assert_eq!(t.life(P0), 23);
    assert!(!t.obj(forest).tapped);
}
