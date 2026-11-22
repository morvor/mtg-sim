//! Rulings batch P209 — enchant (CR 303.4, 702.5): Auras about tapping — "becomes
//! tapped" triggers on permanents tapped for mana or as a cost, "doesn't untap" Auras on
//! untapped permanents, upkeep tap triggers, and Auras whose enchant restriction or
//! intervening "if" clause stops being satisfied.

use crate::r_p209_common::*;
use crate::r_s01_common::*;
use crate::r_s04_common::*;
use crate::r_s06_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// "Becomes tapped" triggers and costs
// ---------------------------------------------------------------------------------------

/// `aura` (controlled by P1) enchants P0's `creature`; P0 activates its mana ability: the
/// mana is added at once, then the Aura's trigger goes on the stack and destroys it.
fn tapped_for_mana_then_destroyed(aura: &str, permanent: &str) {
    supported(aura);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let p = t.battlefield(P0, permanent);
    attach_new(&mut t, P1, aura, p);
    t.activate(P0, p, 0, &[]).expect("mana ability");
    // The mana ability resolved immediately.
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    assert!(t.on_battlefield(p));
    t.settle();
    assert_eq!(t.stack_len(), 1, "{aura}'s trigger");
    t.resolve();
    assert!(!t.on_battlefield(p));
    assert!(t.in_graveyard(P0, permanent));
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn cryoshatter_and_brink_of_disaster_trigger_after_a_mana_ability_resolves() {
    cr!("605.3a", "605.3b", "603.3", "701.26a");
    ruling!(
        "Cryoshatter",
        "If the enchanted creature is tapped as a cost to activate a mana ability, the mana ability resolves immediately, then Cryoshatter’s last ability goes on the stack."
    );
    ruling!(
        "Brink of Disaster",
        "If the enchanted permanent is tapped as a cost to activate a mana ability, the mana ability resolves immediately, then Brink of Disaster’s ability goes on the stack."
    );
    supported("Llanowar Elves");
    tapped_for_mana_then_destroyed("Cryoshatter", "Llanowar Elves");
    tapped_for_mana_then_destroyed("Brink of Disaster", "Llanowar Elves");
    tapped_for_mana_then_destroyed("Brink of Disaster", "Forest");
}

/// `aura` (controlled by P1) enchants P0's Prodigal Pyromancer; P0 activates its {T}
/// ability targeting P1: the Aura's trigger goes on top, resolves first (destroying the
/// Pyromancer), then the ability still deals its damage.
fn tapped_for_ability_then_destroyed(aura: &str) {
    supported(aura);
    supported("Prodigal Pyromancer");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    attach_new(&mut t, P1, aura, pyro);
    t.activate(P0, pyro, 0, &[Entity::Player(P1)]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let items = stack_items(&t);
    assert!(items[0].contains("damage"), "{items:?}");
    assert!(items[1].contains("destroy"), "{items:?}");
    t.resolve();
    assert!(t.in_graveyard(P0, "Prodigal Pyromancer"));
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn cryoshatter_and_brink_of_disaster_resolve_before_the_ability_that_tapped_it() {
    cr!("603.3", "602.2", "608.2b", "113.7a");
    ruling!(
        "Cryoshatter",
        "If the enchanted creature is tapped as a cost to cast a spell or to activate an ability that’s not a mana ability, Cryoshatter’s last ability will go on the stack on top of that spell or activated ability. Cryoshatter’s last ability resolves first (destroying that creature), then the spell or activated ability resolves."
    );
    ruling!(
        "Brink of Disaster",
        "If the enchanted permanent is tapped as a cost to activate an ability that’s not a mana ability, Brink of Disaster’s ability will go on the stack on top of that activated ability. Brink of Disaster’s ability resolves first (destroying that permanent), then the permanent’s activated ability resolves."
    );
    tapped_for_ability_then_destroyed("Cryoshatter");
    tapped_for_ability_then_destroyed("Brink of Disaster");
}

/// P0's Forest enchanted by P1's Contaminated Ground (so it's a Swamp), and Grizzly Bears
/// for P1.
fn contaminated_forest() -> (TestGame, ObjectId, ObjectId) {
    supported("Contaminated Ground");
    supported("Disfigure");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let land = t.battlefield(P0, "Forest");
    attach_new(&mut t, P1, "Contaminated Ground", land);
    assert!(t.obj_now(land).chars.has_subtype("Swamp"));
    let bears = t.battlefield(P1, "Grizzly Bears");
    (t, land, bears)
}

#[test]
fn contaminated_ground_triggers_on_top_of_the_spell_its_mana_paid_for() {
    cr!("605.3b", "601.2g", "603.3");
    ruling!(
        "Contaminated Ground",
        "If, while casting a spell or activating an ability, the enchanted land's controller taps the land for mana to pay for it, Contaminated Ground's ability triggers and goes on the stack on top of that spell or ability. Contaminated Ground's ability will resolve first."
    );
    let (mut t, land, bears) = contaminated_forest();
    let d = t.hand(P0, "Disfigure");
    t.cast_with(P0, d, &[Entity::Object(bears)]).unwrap();
    assert!(t.obj_now(land).tapped);
    t.settle();
    assert_eq!(stack_items(&t)[0], "Disfigure");
    assert!(stack_items(&t)[1].contains("loses 2 life"));
    t.resolve();
    assert_eq!(t.life(P0), 18);
    assert!(t.on_battlefield(bears));
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn contaminated_grounds_trigger_can_be_responded_to_with_that_mana() {
    cr!("605.3b", "106.4", "117.3c");
    ruling!(
        "Contaminated Ground",
        "On the other hand, the enchanted land's controller may tap the land for mana, let Contaminated Ground's ability trigger and go on the stack, then spend that mana to cast an instant or activate an ability in response. In that case, that instant or ability will resolve first."
    );
    let (mut t, land, bears) = contaminated_forest();
    t.activate(P0, land, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    let d = t.hand(P0, "Disfigure");
    t.cast_with(P0, d, &[Entity::Object(bears)]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P0), 20);
    t.resolve();
    assert_eq!(t.life(P0), 18);
}

#[test]
fn lust_for_war_triggers_whenever_the_creature_becomes_tapped() {
    cr!("701.26a", "603.2", "508.1f");
    ruling!(
        "Lust for War",
        "Lust for War's triggered ability triggers when the enchanted creature becomes tapped for any reason, not just when it attacks."
    );
    supported("Lust for War");
    // Tapped by an effect.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Lust for War", bears);
    tap(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    // Tapped by attacking.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P1, "Lust for War", bears);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn volition_reins_on_an_untapped_permanent_doesnt_trigger() {
    cr!("603.4", "303.4a");
    ruling!(
        "Volition Reins",
        "If the enchanted permanent is untapped as Volition Reins enters, the triggered ability won't trigger at all."
    );
    supported("Volition Reins");
    for tapped in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bears = t.battlefield(P1, "Grizzly Bears");
        if tapped {
            t.g.tap(bears);
        }
        give_mana_for(&mut t, P0, "Volition Reins");
        let r = t.hand(P0, "Volition Reins");
        t.cast(P0, r).target(bears).go();
        t.resolve();
        assert_eq!(t.obj_now(bears).controller, P0);
        assert_eq!(t.stack_len(), usize::from(tapped));
        t.resolve_all();
        assert!(!t.obj_now(bears).tapped);
    }
}

// ---------------------------------------------------------------------------------------
// Untapped permanents and "doesn't untap"
// ---------------------------------------------------------------------------------------

/// P0 casts the Aura `name` on P1's untapped `target_card`: it stays untapped; once
/// tapped, it doesn't untap during P1's untap step.
fn doesnt_untap_on_an_untapped(name: &str, target_card: &str) {
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let target = t.battlefield(P1, target_card);
    let aura = cast_aura(&mut t, P0, name, target).expect("the Aura couldn't be cast");
    assert_eq!(attached_to(&t, aura), Some(Entity::Object(target)));
    assert!(!t.obj_now(target).tapped, "{name} tapped it");
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(target).tapped);
    t.resolve_all();
    tap(&mut t, target);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(target).tapped, "{name}: it untapped");
}

#[test]
fn numbing_dose_and_paralyzing_grasp_may_enchant_untapped_permanents() {
    cr!("303.4a", "302.6", "502.3");
    ruling!(
        "Numbing Dose",
        "Numbing Dose may target and may enchant an untapped artifact or creature."
    );
    ruling!(
        "Paralyzing Grasp",
        "Paralyzing Grasp may target, and may enchant, an untapped creature. If it does, it will have no effect unless that creature becomes tapped somehow."
    );
    doesnt_untap_on_an_untapped("Numbing Dose", "Grizzly Bears");
    doesnt_untap_on_an_untapped("Numbing Dose", "Mind Stone");
    doesnt_untap_on_an_untapped("Paralyzing Grasp", "Grizzly Bears");
}

#[test]
fn relic_putrescence_on_a_tapped_artifact_waits_for_it_to_become_tapped() {
    cr!("303.4a", "701.26a", "603.2");
    ruling!(
        "Relic Putrescence",
        "Relic Putrescence may target and may enchant an artifact that’s already tapped. It won’t do anything until the enchanted artifact changes from being untapped to being tapped."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let stone = t.battlefield(P1, "Mind Stone");
    t.g.tap(stone);
    let aura = cast_aura(&mut t, P0, "Relic Putrescence", stone).unwrap();
    assert_eq!(attached_to(&t, aura), Some(Entity::Object(stone)));
    assert_eq!(t.g.player(P1).poison(), 0);
    untap(&mut t, stone);
    t.resolve_all();
    assert_eq!(t.g.player(P1).poison(), 0);
    tap(&mut t, stone);
    t.resolve_all();
    assert_eq!(t.g.player(P1).poison(), 1);
}

#[test]
fn narcolepsy_taps_during_upkeep_and_can_be_responded_to() {
    cr!("502.3", "603.4", "503.1a");
    ruling!(
        "Narcolepsy",
        "Narcolepsy doesn’t keep the enchanted creature continually tapped. The creature untaps during its controller’s untap step as normal, then is tapped when Narcolepsy’s ability resolves during that upkeep. The creature may be tapped in response (for example, if it has an activated ability with {T} in its cost)."
    );
    supported("Narcolepsy");
    supported("Prodigal Pyromancer");
    let setup = || {
        let mut t = TestGame::new(2);
        let pyro = t.battlefield(P1, "Prodigal Pyromancer");
        attach_new(&mut t, P0, "Narcolepsy", pyro);
        t.g.tap(pyro);
        t.advance_to(P1, Step::Upkeep);
        t.settle();
        // It untapped as normal; the trigger is waiting.
        assert!(!t.obj_now(pyro).tapped);
        assert_eq!(t.stack_len(), 1);
        (t, pyro)
    };
    let (mut t, pyro) = setup();
    t.resolve();
    assert!(t.obj_now(pyro).tapped);
    // In response, P1 taps it for its ability.
    let (mut t, pyro) = setup();
    t.activate(P1, pyro, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert!(t.obj_now(pyro).tapped);
}

#[test]
fn winters_rest_applies_again_with_a_new_snow_permanent() {
    cr!("502.3", "611.3a", "205.4g");
    ruling!(
        "Winter's Rest",
        "If you lose control of all of your other snow permanents, then control a new one, the effect of Winter's Rest will apply to the enchanted creature again."
    );
    supported("Winter's Rest");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let snow = t.battlefield(P0, "Snow-Covered Forest");
    attach_new(&mut t, P0, "Winter's Rest", bears);
    t.g.tap(bears);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
    // No other snow permanent: it untaps.
    crate::r_s02_common::destroy(&mut t, snow);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped);
    // A new snow permanent: it applies again.
    t.battlefield(P0, "Snow-Covered Forest");
    tap(&mut t, bears);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
}

#[test]
fn observed_stasis_removes_from_combat_without_untapping() {
    cr!("506.4", "506.4a");
    ruling!(
        "Observed Stasis",
        "Removing the enchanted creature from combat doesn't cause it to untap."
    );
    supported("Observed Stasis");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    assert!(t.obj_now(bears).tapped);
    assert!(t.g.is_attacking(bears));
    let aura = cast_aura(&mut t, P0, "Observed Stasis", bears).unwrap();
    assert_eq!(attached_to(&t, aura), Some(Entity::Object(bears)));
    assert!(!t.g.is_attacking(bears));
    assert!(t.obj_now(bears).tapped);
}

// ---------------------------------------------------------------------------------------
// Conditions on the enchanted permanent
// ---------------------------------------------------------------------------------------

#[test]
fn runners_bane_falls_off_a_creature_whose_power_becomes_four() {
    cr!("303.4d", "704.5m");
    ruling!(
        "Runner's Bane",
        "If the enchanted creature's power increases to 4 or greater, Runner's Bane will be put into its owner's graveyard as a state-based action."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let aura = cast_aura(&mut t, P0, "Runner's Bane", bears).unwrap();
    assert_eq!(attached_to(&t, aura), Some(Entity::Object(bears)));
    cast_spell(&mut t, P1, "Giant Growth", &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert!(t.in_graveyard(P0, "Runner's Bane"));
}

#[test]
fn arachnus_web_doesnt_trigger_if_the_power_is_too_low() {
    cr!("603.4", "513.1a");
    ruling!(
        "Arachnus Web",
        "If the enchanted creature's power isn't 4 or greater when the end step begins, the last ability won't trigger at all."
    );
    supported("Arachnus Web");
    for (creature, triggers) in [("Grizzly Bears", false), ("Colossal Dreadmaw", true)] {
        let mut t = TestGame::new(2);
        let c = t.battlefield(P1, creature);
        let web = attach_new(&mut t, P0, "Arachnus Web", c);
        t.set_step(P0, Step::PrecombatMain);
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(on_stack(&t, "destroy"), usize::from(triggers), "{creature}");
        t.resolve_all();
        assert_eq!(t.on_battlefield(web), !triggers);
    }
}

#[test]
fn encase_in_ice_falls_off_a_creature_that_stops_being_red_or_green_or_a_creature() {
    cr!("303.4d", "704.5m", "105.3");
    ruling!(
        "Encase in Ice",
        "If the enchanted creature stops being red or green, or if it stops being a creature, Encase in Ice will be put into its owner's graveyard."
    );
    supported("Encase in Ice");
    // Stops being red (Cerulean Wisps: it becomes blue).
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let goblin = t.battlefield(P1, "Raging Goblin");
    let aura = cast_aura(&mut t, P0, "Encase in Ice", goblin).unwrap();
    assert_eq!(attached_to(&t, aura), Some(Entity::Object(goblin)));
    cast_spell(&mut t, P0, "Cerulean Wisps", &[Entity::Object(goblin)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Encase in Ice"));
    // Stops being a creature (Imprisoned in the Moon).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Encase in Ice", bears);
    t.settle();
    assert!(t.named_on_battlefield("Encase in Ice").len() == 1);
    attach_new(&mut t, P0, "Imprisoned in the Moon", bears);
    t.settle();
    assert!(t.in_graveyard(P0, "Encase in Ice"));
}

#[test]
fn chained_to_the_rocks_falls_off_a_land_that_stops_being_your_mountain() {
    cr!("303.4d", "704.5m", "610.3");
    ruling!(
        "Chained to the Rocks",
        "If the land Chained to the Rocks is enchanting stops being a Mountain or another player gains control of it, Chained to the Rocks will be put into its owner's graveyard when state-based actions are performed."
    );
    supported("Chained to the Rocks");
    let setup = || {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let mountain = t.battlefield(P0, "Mountain");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(bears)]);
        cast_aura(&mut t, P0, "Chained to the Rocks", mountain).unwrap();
        assert!(t.in_exile("Grizzly Bears"));
        (t, mountain)
    };
    // It stops being a Mountain (Contaminated Ground: it's a Swamp).
    let (mut t, mountain) = setup();
    attach_new(&mut t, P1, "Contaminated Ground", mountain);
    t.settle();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Chained to the Rocks"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    // Another player gains control of it.
    let (mut t, mountain) = setup();
    give_control(&mut t, mountain, P1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Chained to the Rocks"));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}
