//! Rulings batch P218 — landfall ("Whenever a land you control enters, ..."; an ability
//! word, CR 207.2c) and landwalk (CR 702.14).

use crate::r_p214_common::pump as pump_until_eot;
use crate::r_s01_common::{attack_with, supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s05_common::enter;
use crate::r_s06_common::attach_new;
use crate::r_s21_common::{castable, legal_blocks};
use mtg_engine::ability::{Duration, Effect, Modification, PlayerRef, Sel, Value};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const LANDFALL: &str = "a land you control enters";

/// Runs a resolving effect of `p`'s on `target` with `mods` for `duration`.
fn modify(t: &mut TestGame, p: PlayerId, target: ObjectId, mods: Vec<Modification>) {
    crate::r_s05_common::run_from(
        t,
        p,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods,
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(target)],
    );
}

#[test]
fn skyclave_pick_axe_bonus_stays_with_the_creature_equipped_at_resolution() {
    cr!("611.2c", "608.2h", "301.5");
    ruling!(
        "Skyclave Pick-Axe",
        "The bonus from Skyclave Pick-Axe's landfall ability applies to the creature it's attached to at the time the ability resolves. That bonus doesn't apply to a new creature if Skyclave Pick-Axe becomes attached to a new creature."
    );
    supported("Skyclave Pick-Axe");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ogre = t.battlefield(P0, "Gray Ogre");
    let axe = attach_new(&mut t, P0, "Skyclave Pick-Axe", bears);
    enter(&mut t, P0, "Forest");
    assert_eq!(triggers_on_stack(&t, LANDFALL), 1);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4));
    // Moved to the Ogre: the Bears keep the bonus, the Ogre doesn't get it.
    assert!(t.g.attach(axe, Entity::Object(ogre)));
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.pt(ogre), (2, 2));
    // Moved before the ability resolves: the creature equipped at resolution gets it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ogre = t.battlefield(P0, "Gray Ogre");
    let axe = attach_new(&mut t, P0, "Skyclave Pick-Axe", bears);
    enter(&mut t, P0, "Forest");
    assert!(t.g.attach(axe, Entity::Object(ogre)));
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(ogre), (4, 4));
}

#[test]
fn genemorph_imago_overwrites_earlier_set_effects_but_not_modifiers() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Genemorph Imago",
        "The effect of Genemorph Imago’s landfall ability will overwrite any previous effects that set the creature’s power and toughness to specific values."
    );
    supported("Genemorph Imago");
    // Hill Giant (3/3) was made 0/1 earlier and got +1/+1 before that; it has a +1/+1
    // counter. Imago's landfall makes its base 3/3: 3/3 +1/+1 (pump) +1/+1 (counter).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Genemorph Imago");
    let giant = t.battlefield(P1, "Hill Giant");
    pump_until_eot(&mut t, giant, 1, 1);
    modify(
        &mut t,
        P1,
        giant,
        vec![Modification::SetPT(Some(Value::c(0)), Some(Value::c(1)))],
    );
    t.g.add_counters(Entity::Object(giant), "+1/+1", 1, None);
    t.g.recompute();
    assert_eq!(t.pt(giant), (2, 3));
    t.answer_targets(P0, &[Entity::Object(giant)]);
    enter(&mut t, P0, "Island");
    assert_eq!(triggers_on_stack(&t, LANDFALL), 1);
    t.resolve_all();
    assert_eq!(t.pt(giant), (5, 5));
    // A pump that takes effect afterwards still applies.
    pump_until_eot(&mut t, giant, 2, 0);
    assert_eq!(t.pt(giant), (7, 5));
}

#[test]
fn akoum_firebird_triggers_only_from_the_graveyard_as_the_land_enters() {
    cr!("603.6a", "603.10", "603.2");
    ruling!(
        "Akoum Firebird",
        "The landfall ability triggers only if Akoum Firebird is in your graveyard at the moment the land enters the battlefield."
    );
    supported("Akoum Firebird");
    // On the battlefield as the land enters, then dies in response: no trigger.
    let mut t = TestGame::new(2);
    let bird = t.battlefield(P0, "Akoum Firebird");
    enter(&mut t, P0, "Mountain");
    assert_eq!(triggers_on_stack(&t, LANDFALL), 0);
    destroy(&mut t, bird);
    assert!(t.in_graveyard(P0, "Akoum Firebird"));
    assert_eq!(triggers_on_stack(&t, LANDFALL), 0);
    // In the graveyard: it triggers, and P0 may pay {4}{R}{R} to return it.
    t.lands(P0, "Mountain", 6);
    enter(&mut t, P0, "Mountain");
    assert_eq!(triggers_on_stack(&t, LANDFALL), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Akoum Firebird").len(), 1);
}

#[test]
fn turntimber_basilisk_choice_is_made_as_the_ability_resolves() {
    cr!("603.5", "608.2d", "509.1c");
    ruling!(
        "Turntimber Basilisk",
        "You decide whether to have the targeted creature block Turntimber Basilisk this turn if able at the time the landfall ability resolves, not at the time Turntimber Basilisk attacks."
    );
    supported("Turntimber Basilisk");
    for yes in [true, false] {
        let mut t = TestGame::new(2);
        let basilisk = t.battlefield(P0, "Turntimber Basilisk");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        enter(&mut t, P0, "Forest");
        assert_eq!(triggers_on_stack(&t, LANDFALL), 1);
        // The "you may" question is asked as the ability resolves.
        let from = t.asked().len();
        t.answer_yes(P0, yes);
        t.resolve_all();
        assert_eq!(
            t.asked()[from..]
                .iter()
                .filter(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }))
                .count(),
            1
        );
        crate::r_s20_common::to_beginning_of_combat(&mut t, P0);
        let from = t.asked().len();
        attack_with(&mut t, &[(basilisk, Entity::Player(P1))]);
        // Nothing more is asked of P0 as the Basilisk attacks.
        assert!(!t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. })));
        assert_eq!(legal_blocks(&mut t, P1, &[]), !yes);
        assert!(legal_blocks(&mut t, P1, &[(bears, basilisk)]));
    }
}

#[test]
fn roil_elemental_targeting_itself_overwrites_a_temporary_control_change() {
    cr!("613.7", "613.2", "611.2b");
    ruling!(
        "Roil Elemental",
        "You may target a creature you already control with Roil Elemental's ability. This will usually have no visible effect, but it will overwrite any previous control-change effects."
    );
    supported("Roil Elemental");
    // P1's Roil Elemental: P0 gains control of it until end of turn (as Mark of Mutiny
    // would), then its landfall ability triggers for P0, targeting itself.
    let mut t = TestGame::new(2);
    let roil = t.battlefield(P1, "Roil Elemental");
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(roil)]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    assert_eq!(t.obj_now(roil).controller, P0);
    t.answer_targets(P0, &[Entity::Object(roil)]);
    t.answer_yes(P0, true);
    enter(&mut t, P0, "Island");
    assert_eq!(triggers_on_stack(&t, LANDFALL), 1);
    t.resolve_all();
    assert_eq!(t.obj_now(roil).controller, P0);
    // After the turn ends, P0 still controls it (for as long as P0 controls it).
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(roil).controller, P0);
    // Without the landfall ability, control returns to P1 at end of turn.
    let mut t = TestGame::new(2);
    let roil = t.battlefield(P1, "Roil Elemental");
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(roil)]];
    t.g.exec(
        &Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(roil).controller, P1);
}

#[test]
fn territorial_bruntar_cards_follow_timing_rules_and_costs() {
    cr!("601.3", "307.1", "608.2c");
    ruling!(
        "Territorial Bruntar",
        "You pay all costs and follow all timing rules for cards cast with the permission granted by Territorial Bruntar’s landfall ability."
    );
    supported("Territorial Bruntar");
    // A land enters during P0's beginning of combat step; the Bruntar exiles Lava Spike
    // (a sorcery) past a Forest.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Territorial Bruntar");
    let spike = t.library_top(P0, "Lava Spike");
    t.library_top(P0, "Forest");
    t.set_step(P0, Step::BeginningOfCombat);
    enter(&mut t, P0, "Mountain");
    assert_eq!(triggers_on_stack(&t, LANDFALL), 1);
    t.resolve_all();
    let spike = t.g.current(spike);
    assert_eq!(t.zone(spike), mtg_engine::object::Zone::Exile);
    // A sorcery can't be cast outside a main phase (P0 has the Mountain's mana).
    assert!(!castable(&mut t, P0, spike));
    // In the main phase with an empty stack, it can.
    t.set_step(P0, Step::PostcombatMain);
    assert!(castable(&mut t, P0, spike));
    // Its mana cost must still be paid: with the Mountain tapped, it can't be cast.
    let mountain =
        t.g.permanents()
            .find(|o| o.controller == P0 && o.chars.name == "Mountain")
            .map(|o| o.id)
            .unwrap();
    t.g.tap(mountain);
    assert!(!castable(&mut t, P0, spike));
}

#[test]
fn nonbasic_landwalk_checks_for_lands_without_the_basic_supertype() {
    cr!("702.14c", "205.4a");
    ruling!(
        "Dryad Sophisticate",
        "\"Nonbasic landwalk\" means \"This creature can't be blocked as long as defending player controls a nonbasic land.\""
    );
    supported("Dryad Sophisticate");
    // The defending player's Tundra (Plains Island, not basic) makes the Dryad unblockable;
    // basic lands don't.
    for (land, unblockable) in [
        ("Tundra", true),
        ("Island", false),
        ("Snow-Covered Island", false),
    ] {
        let mut t = TestGame::new(2);
        let dryad = t.battlefield(P0, "Dryad Sophisticate");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.battlefield(P1, land);
        crate::r_s20_common::to_beginning_of_combat(&mut t, P0);
        attack_with(&mut t, &[(dryad, Entity::Player(P1))]);
        assert_eq!(
            legal_blocks(&mut t, P1, &[(bears, dryad)]),
            !unblockable,
            "{land}"
        );
    }
}

use mtg_engine::decision::Decision;
