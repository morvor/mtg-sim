//! CR 208: power and toughness — printed values, `*` and characteristic-defining
//! abilities, noncreature permanents, and base power and toughness.

use crate::r703_common::{run_effect, supported};
use mtg_engine::ability::*;
use mtg_engine::card::card;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn pt_of(t: &TestGame, id: ObjectId) -> (Option<i32>, Option<i32>) {
    let c = &t.obj_now(id).chars;
    (c.power, c.toughness)
}

/// `what` gets the modifications until end of turn (a resolving effect).
fn modify(t: &mut TestGame, what: ObjectId, mods: Vec<Modification>) {
    run_effect(
        t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods,
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(what)],
    );
}

#[test]
fn power_is_combat_damage_dealt_and_toughness_the_damage_that_destroys() {
    cr!("208.1");
    let mut t = TestGame::new(2);
    // Hill Giant (3/3) blocked by Grizzly Bears (2/2).
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(giant), (3, 3));
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(giant, Entity::Player(P1))], &[(bears, giant)]);
    assert!(
        t.in_graveyard(P1, "Grizzly Bears"),
        "3 damage destroys a 2-toughness creature"
    );
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj_now(giant).damage, 2);
    // Effects can modify and set power and toughness.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(giant).go();
    t.resolve();
    assert_eq!(t.pt(giant), (6, 6));
    modify(
        &mut t,
        giant,
        vec![Modification::SetPT(Some(Value::c(0)), Some(Value::c(1)))],
    );
    assert_eq!(t.pt(giant), (3, 4));
}

#[test]
fn star_power_and_toughness_are_defined_by_a_characteristic_defining_ability() {
    cr!("208.2", "208.2a");
    ruling!(
        "Awakened Amalgam",
        "The ability that defines Awakened Amalgam’s power and toughness works in all zones"
    );
    // Tarmogoyf: */1+*, "power equal to the number of card types among cards in all
    // graveyards and toughness equal to that number plus 1".
    let goyf = card("Tarmogoyf");
    assert!(goyf.front().star_power && goyf.front().star_toughness);
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P0, "Tarmogoyf");
    let in_gy = t.graveyard(P1, "Tarmogoyf");
    let outside = t.custom(P0, (*card("Tarmogoyf")).clone(), Zone::Outside(P0));
    t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P0, "Forest");
    t.g.recompute();
    // Creature (the graveyard Tarmogoyf), instant, land: 3.
    for id in [in_hand, in_gy, outside] {
        assert_eq!(pt_of(&t, id), (Some(3), Some(4)), "it works in every zone");
    }
    // Awakened Amalgam ("power and toughness are each equal to the number of differently
    // named lands you control") in its owner's hand and graveyard.
    t.lands(P0, "Plains", 2);
    t.battlefield(P0, "Island");
    let amalgam_hand = t.hand(P0, "Awakened Amalgam");
    let amalgam_gy = t.graveyard(P0, "Awakened Amalgam");
    t.g.recompute();
    assert_eq!(pt_of(&t, amalgam_hand), (Some(2), Some(2)));
    assert_eq!(pt_of(&t, amalgam_gy), (Some(2), Some(2)));
    // A number that can't be determined is 0: Lost Order of Jarkeld (1+*/1+*) with no
    // chosen player is 1/1.
    let order = t.hand(P0, "Lost Order of Jarkeld");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.g.recompute();
    assert_eq!(pt_of(&t, order), (Some(1), Some(1)));
    let order = t.enter(P0, "Lost Order of Jarkeld");
    assert_eq!(t.obj_now(order).choices.player, Some(P1));
    assert_eq!(t.pt(order), (3, 3));
}

#[test]
fn power_and_toughness_chosen_as_it_enters_are_copiable() {
    cr!("208.2b");
    supported("Primal Clay");
    let mut t = TestGame::new(2);
    // Not on the battlefield, its power and toughness are each 0.
    let hand = t.hand(P0, "Primal Clay");
    assert_eq!(pt_of(&t, hand), (Some(0), Some(0)));
    // It enters as the 1/6 Wall artifact creature with defender.
    t.answer(P0, DecisionKind::Option, Answer::Index(2));
    let clay = t.enter(P0, "Primal Clay");
    assert_eq!(t.pt(clay), (1, 6));
    assert!(t.obj_now(clay).chars.has_subtype("Wall"));
    assert!(t.obj_now(clay).has_keyword(KeywordKind::Defender));
    // The choice is part of its copiable values: a copy is a 1/6 Wall too.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(bears)], vec![Entity::Object(clay)]];
    t.g.exec(
        &Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::Target(1),
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    assert_eq!(t.pt(bears), (1, 6));
    assert!(t.obj_now(bears).chars.has_subtype("Wall"));
    // In the graveyard, it's 0/0 again.
    t.g.destroy(clay, None);
    t.settle();
    let gy = t.g.find_in_zone(Zone::Graveyard(P0), "Primal Clay")[0];
    assert_eq!(pt_of(&t, gy), (Some(0), Some(0)));
}

#[test]
fn noncreature_permanents_have_no_power_or_toughness() {
    cr!("208.3");
    let mut t = TestGame::new(2);
    // Smuggler's Copter: a Vehicle with a printed 3/3.
    let copter = t.battlefield(P0, "Smuggler's Copter");
    assert_eq!(pt_of(&t, copter), (None, None));
    let pt_filter = Filter::Power(Cmp::Ge, Box::new(Value::c(0)));
    assert!(!crate::r105_util::matches(&t, copter, &pt_filter, P0));
    // Not on the battlefield, a noncreature card has power and toughness only if they're
    // printed on it.
    let in_hand = t.hand(P0, "Smuggler's Copter");
    assert_eq!(pt_of(&t, in_hand), (Some(3), Some(3)));
    assert!(crate::r105_util::matches(&t, in_hand, &pt_filter, P0));
    let bolt = t.hand(P0, "Lightning Bolt");
    assert_eq!(pt_of(&t, bolt), (None, None));
}

#[test]
fn pt_effects_on_a_noncreature_permanent_apply_once_it_becomes_a_creature() {
    cr!("208.3a");
    let mut t = TestGame::new(2);
    // War Balloon: a Vehicle (4/3) that's an artifact creature with three fire counters.
    let balloon = t.battlefield(P0, "War Balloon");
    assert!(!t.obj_now(balloon).is_creature());
    modify(
        &mut t,
        balloon,
        vec![Modification::ModifyPT(Value::c(1), Value::c(1))],
    );
    assert_eq!(
        pt_of(&t, balloon),
        (None, None),
        "the effect exists but does nothing"
    );
    t.g.add_counters(Entity::Object(balloon), "fire", 3, None);
    t.g.recompute();
    assert!(t.obj_now(balloon).is_creature());
    assert_eq!(t.pt(balloon), (5, 4));
    // An effect setting the base power and toughness of a noncreature permanent is
    // created too.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    modify(
        &mut t,
        ring,
        vec![Modification::SetPT(Some(Value::c(5)), Some(Value::c(5)))],
    );
    assert_eq!(pt_of(&t, ring), (None, None));
    modify(
        &mut t,
        ring,
        vec![Modification::AddTypes(vec![CardType::Creature])],
    );
    assert_eq!(t.pt(ring), (5, 5));
}

#[test]
fn effects_may_set_base_power_and_toughness() {
    cr!("208.4", "208.4a");
    // Frogify: "Enchanted creature loses all abilities and is a blue Frog creature with
    // base power and toughness 1/1." Other effects and counters still modify it.
    supported("Frogify");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(Entity::Object(giant), "+1/+1", 2, None);
    t.g.recompute();
    assert_eq!(t.pt(giant), (5, 5));
    t.lands(P0, "Island", 2);
    let frog = t.hand(P0, "Frogify");
    t.cast(P0, frog).target(giant).go();
    t.resolve_all();
    assert!(t.obj_now(giant).chars.has_subtype("Frog"));
    assert_eq!(t.obj_now(giant).base_pt, (Some(1), Some(1)));
    assert_eq!(t.pt(giant), (3, 3));
    t.battlefield(P1, "Glorious Anthem");
    assert_eq!(t.pt(giant), (4, 4));
}

#[test]
fn base_power_ignores_modifications_and_counters() {
    cr!("208.4b");
    // Baird, Argivian Recruiter: "At the beginning of your end step, if you control a
    // creature with power greater than its base power, create a 1/1 white Soldier
    // creature token."
    supported("Baird, Argivian Recruiter");
    let soldiers = |t: &TestGame| t.named_on_battlefield("Soldier Token").len();
    let to_end_step = |t: &mut TestGame| {
        t.advance_to(P0, Step::End);
        t.settle();
        t.resolve_all();
    };
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Baird, Argivian Recruiter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // An effect that sets power doesn't raise it above its base power.
    modify(
        &mut t,
        bears,
        vec![Modification::SetPT(Some(Value::c(5)), Some(Value::c(5)))],
    );
    assert_eq!(t.obj_now(bears).base_pt, (Some(5), Some(5)));
    assert_eq!(t.pt(bears), (5, 5));
    to_end_step(&mut t);
    assert_eq!(soldiers(&t), 0);
    // A +1/+1 counter does.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Baird, Argivian Recruiter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "+1/+1", 1, None);
    t.g.recompute();
    assert_eq!(t.obj_now(bears).base_pt, (Some(2), Some(2)));
    to_end_step(&mut t);
    assert_eq!(soldiers(&t), 1);
    // So does a modifying effect (Glorious Anthem) — even for Baird itself.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Baird, Argivian Recruiter");
    t.battlefield(P0, "Glorious Anthem");
    to_end_step(&mut t);
    assert_eq!(soldiers(&t), 1);
    // A characteristic-defining ability sets its base power: Tarmogoyf.
    let mut t = TestGame::new(2);
    let goyf = t.battlefield(P0, "Tarmogoyf");
    t.graveyard(P1, "Lightning Bolt");
    t.g.recompute();
    assert_eq!(t.obj_now(goyf).base_pt, (Some(1), Some(2)));
}

#[test]
fn base_power_includes_characteristic_defining_abilities_but_not_bonuses() {
    cr!("208.4b");
    ruling!(
        "Baird, Argivian Recruiter",
        "that ability is taken into account when determining its base power and toughness"
    );
    ruling!(
        "Baird, Argivian Recruiter",
        "Those are not characteristic-defining abilities, and that ability doesn’t change its base power and toughness."
    );
    let soldiers = |t: &TestGame| t.named_on_battlefield("Soldier Token").len();
    let to_end_step = |t: &mut TestGame| {
        t.advance_to(P0, Step::End);
        t.settle();
        t.resolve_all();
    };
    // Tarmogoyf's power comes from its characteristic-defining ability: it isn't greater
    // than its base power.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Baird, Argivian Recruiter");
    let goyf = t.battlefield(P0, "Tarmogoyf");
    t.graveyard(P1, "Lightning Bolt");
    t.graveyard(P1, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(goyf), (2, 3));
    to_end_step(&mut t);
    assert_eq!(soldiers(&t), 0);
    // Kavu Scout (0/2) "gets +1/+0 for each basic land type among lands you control":
    // that bonus isn't part of its base power.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Baird, Argivian Recruiter");
    let scout = t.battlefield(P0, "Kavu Scout");
    t.battlefield(P0, "Mountain");
    t.g.recompute();
    assert_eq!(t.pt(scout), (1, 2));
    assert_eq!(t.obj_now(scout).base_pt, (Some(0), Some(2)));
    to_end_step(&mut t);
    assert_eq!(soldiers(&t), 1);
}

#[test]
fn a_trigger_for_creatures_with_power_greater_than_their_base_power() {
    cr!("208.4b");
    ruling!(
        "Kutzil, Malamet Exemplar",
        "If an effect modifies a creature's power and/or toughness without setting them, that is not included when determining its base power and toughness."
    );
    // Kutzil: "Whenever one or more creatures you control each with power greater than its
    // base power deals combat damage to a player, draw a card."
    supported("Kutzil, Malamet Exemplar");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kutzil, Malamet Exemplar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(
        t.hand_size(P0),
        hand,
        "a 2/2 Grizzly Bears isn't above its base power"
    );
    // With a +1/+1 counter, it is.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kutzil, Malamet Exemplar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), "+1/+1", 1, None);
    t.set_step(P0, Step::PrecombatMain);
    let hand = t.hand_size(P0);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.hand_size(P0), hand + 1);
}
