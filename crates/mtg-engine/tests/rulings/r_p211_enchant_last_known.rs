//! Rulings batch P211 — enchant (CR 303.4, 608.2h, 113.7a): Aura abilities that refer to
//! "enchanted creature" (or "its power") use the object as it is when the ability resolves,
//! or as it last existed on the battlefield (last known information) if it's gone; an
//! ability that already triggered keeps affecting the creature it triggered for.

use crate::r_p208_common::put_in_graveyard;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::*;
use crate::r_s05_common::move_to;
use crate::r_s06_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `p` casts Giant Growth on `id` and it resolves (+3/+3 until end of turn).
fn giant_growth(t: &mut TestGame, p: PlayerId, id: ObjectId) {
    let gg = in_hand_with_mana(t, p, "Giant Growth");
    t.cast(p, gg).target(id).go();
    t.resolve();
}

// ---------------------------------------------------------------------------------------
// "Its power" as it last existed on the battlefield
// ---------------------------------------------------------------------------------------

#[test]
fn murder_investigation_uses_the_power_the_creature_last_had() {
    cr!("608.2h", "113.7a", "603.10a");
    ruling!(
        "Murder Investigation",
        "To determine how many Soldier tokens are created, use the power of the enchanted creature as it last existed on the battlefield."
    );
    supported("Murder Investigation");
    // "When enchanted creature dies, create X 1/1 white Soldier creature tokens, where X is
    // its power." Grizzly Bears pumped to 5/5 by Giant Growth.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Murder Investigation", bears);
    giant_growth(&mut t, P0, bears);
    assert_eq!(t.pt(bears), (5, 5));
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Soldier").len(), 5);
}

#[test]
fn necrosynthesis_looks_at_cards_equal_to_the_power_the_creature_last_had() {
    cr!("608.2h", "113.7a", "603.10a");
    ruling!(
        "Necrosynthesis",
        "Use the enchanted creature's power as it last existed on the battlefield to determine the value of X."
    );
    supported("Necrosynthesis");
    // "When enchanted creature dies, look at the top X cards of your library, where X is
    // its power. Put one of those cards into your hand and the rest on the bottom." The
    // enchanted Bears has a +1/+1 counter (from its granted ability) and Giant Growth: 6.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Necrosynthesis", bears);
    let elves = t.battlefield(P1, "Llanowar Elves");
    destroy(&mut t, elves);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 1);
    giant_growth(&mut t, P0, bears);
    assert_eq!(t.pt(bears).0, 6);
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    destroy(&mut t, bears);
    t.resolve_all();
    let looked: Vec<usize> = t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.len()),
            _ => None,
        })
        .collect();
    assert_eq!(looked, vec![6]);
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn burning_anger_uses_the_power_on_resolution_or_as_the_creature_last_existed() {
    cr!("608.2h", "113.7a", "602.2");
    ruling!(
        "Burning Anger",
        "Use the power of the enchanted creature as the activated ability resolves to determine how much damage is dealt. If the creature isn't on the battlefield at that time, use its power as it last existed on the battlefield."
    );
    supported("Burning Anger");
    // Enchanted creature has "{T}: This creature deals damage equal to its power to any
    // target." Giant Growth in response: 5 damage.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Burning Anger", bears);
    t.activate(P0, bears, 0, &[Entity::Player(P1)]).unwrap();
    giant_growth(&mut t, P0, bears);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    // Pumped, then destroyed in response: its last known power (5).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Burning Anger", bears);
    t.activate(P0, bears, 0, &[Entity::Player(P1)]).unwrap();
    giant_growth(&mut t, P0, bears);
    destroy(&mut t, bears);
    assert!(!t.on_battlefield(bears));
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
}

// ---------------------------------------------------------------------------------------
// "Enchanted creature" as the activated ability resolves
// ---------------------------------------------------------------------------------------

#[test]
fn crab_umbra_untaps_the_creature_it_enchants_on_resolution() {
    cr!("608.2h", "113.7a", "602.2", "701.26b");
    ruling!(
        "Crab Umbra",
        "When Crab Umbra's activated ability resolves, it will untap the creature Crab Umbra is enchanting at that time (regardless of what creature Crab Umbra was enchanting when the ability was activated). If Crab Umbra has left the battlefield by then, the ability will untap the creature it was enchanting at the time it left the battlefield."
    );
    supported("Crab Umbra");
    // Moved to another creature in response.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.g.tap(bears);
    t.g.tap(giant);
    let umbra = attach_new(&mut t, P0, "Crab Umbra", bears);
    t.lands(P0, "Island", 3);
    t.activate(P0, umbra, 0, &[]).unwrap();
    assert!(t.g.attach(umbra, Entity::Object(giant)));
    t.g.recompute();
    t.resolve_all();
    assert!(!t.obj_now(giant).tapped);
    assert!(t.obj_now(bears).tapped);
    // Crab Umbra leaves the battlefield in response.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    let umbra = attach_new(&mut t, P0, "Crab Umbra", bears);
    t.lands(P0, "Island", 3);
    t.activate(P0, umbra, 0, &[]).unwrap();
    put_in_graveyard(&mut t, umbra);
    t.settle();
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn shivs_embrace_pumps_the_creature_it_enchants_on_resolution() {
    cr!("608.2h", "113.7a", "602.2", "611.2c");
    ruling!(
        "Shiv's Embrace",
        "When Shiv’s Embrace’s activated ability resolves, the creature Shiv’s Embrace is enchanting at that time will get +1/+0 (regardless of what creature Shiv’s Embrace was enchanting when the ability was activated). If Shiv’s Embrace has left the battlefield by then, the creature it was enchanting at the time it left the battlefield will get +1/+0."
    );
    supported("Shiv's Embrace");
    // "Enchanted creature gets +2/+2 and has flying. {R}: Enchanted creature gets +1/+0
    // until end of turn." Moved to the Hill Giant in response.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let shiv = attach_new(&mut t, P0, "Shiv's Embrace", bears);
    t.lands(P0, "Mountain", 1);
    t.activate(P0, shiv, 0, &[]).unwrap();
    assert!(t.g.attach(shiv, Entity::Object(giant)));
    t.g.recompute();
    t.resolve_all();
    assert_eq!(t.pt(giant), (6, 5));
    assert_eq!(t.pt(bears), (2, 2));
    // Shiv's Embrace leaves the battlefield in response: the Bears gets +1/+0 only.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let shiv = attach_new(&mut t, P0, "Shiv's Embrace", bears);
    t.lands(P0, "Mountain", 1);
    t.activate(P0, shiv, 0, &[]).unwrap();
    put_in_graveyard(&mut t, shiv);
    t.settle();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 2));
}

// ---------------------------------------------------------------------------------------
// Triggered abilities keep affecting the permanent they triggered for
// ---------------------------------------------------------------------------------------

#[test]
fn relic_putrescence_poisons_the_artifacts_controller_on_resolution() {
    cr!("608.2h", "113.7a", "603.2", "122.1f");
    ruling!(
        "Relic Putrescence",
        "When the enchanted artifact becomes tapped, Relic Putrescence’s ability triggers. The player who gets the poison counter is the player who, at the time the ability resolves, controls the artifact that became tapped. If that artifact is no longer on the battlefield, its last existence on the battlefield is checked to determine its controller. It doesn’t matter whether Relic Putrescence is still on the battlefield as the ability resolves, what artifact it’s enchanting at that time, who controlled the artifact at the time it became tapped, or who tapped it."
    );
    supported("Relic Putrescence");
    let poison = |t: &TestGame, p: PlayerId| t.g.player(p).counter(counters::POISON);
    // P1's Mind Stone, enchanted by P0's Relic Putrescence, is tapped by P1; P0 gains
    // control of it before the ability resolves, and Relic Putrescence leaves: P0 gets the
    // poison counter.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Mind Stone");
    let relic = attach_new(&mut t, P0, "Relic Putrescence", stone);
    tap(&mut t, stone);
    assert_eq!(triggers_on_stack(&t, "poison"), 1);
    give_control(&mut t, stone, P0);
    put_in_graveyard(&mut t, relic);
    t.settle();
    t.resolve_all();
    assert_eq!(poison(&t, P0), 1);
    assert_eq!(poison(&t, P1), 0);
    // Tapped by an effect P0 controls; moved to another artifact; the tapped artifact
    // leaves the battlefield: its last controller (P1) gets the counter.
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Mind Stone");
    let other = t.battlefield(P0, "Millstone");
    let relic = attach_new(&mut t, P0, "Relic Putrescence", stone);
    tap(&mut t, stone);
    assert!(t.g.attach(relic, Entity::Object(other)));
    t.g.recompute();
    move_to(&mut t, stone, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(poison(&t, P1), 1);
    assert_eq!(poison(&t, P0), 0);
}

/// Taps the permanent as an effect would, then settles.
fn tap(t: &mut TestGame, id: ObjectId) {
    let id = t.g.current(id);
    t.g.tap(id);
    t.g.flush_events();
    t.settle();
}

#[test]
fn cryoshatter_and_brink_of_disaster_destroy_the_permanent_even_if_the_aura_left() {
    cr!("608.2h", "603.2", "603.4");
    ruling!(
        "Cryoshatter",
        "When the enchanted creature becomes tapped or is dealt damage, Cryoshatter’s last ability triggers. That creature will be destroyed when the ability resolves, even if Cryoshatter has left the battlefield or is somehow enchanting a different creature by then."
    );
    ruling!(
        "Brink of Disaster",
        "When the enchanted permanent becomes tapped, Brink of Disaster’s ability triggers. That permanent will be destroyed when the ability resolves, even if Brink of Disaster has left the battlefield or is somehow enchanting a different permanent by then."
    );
    supported("Cryoshatter");
    supported("Brink of Disaster");
    for aura in ["Cryoshatter", "Brink of Disaster"] {
        // Tapped; the Aura moves to another creature.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        let a = attach_new(&mut t, P0, aura, bears);
        tap(&mut t, bears);
        assert_eq!(triggers_on_stack(&t, "destroy"), 1, "{aura}");
        assert!(t.g.attach(a, Entity::Object(giant)));
        t.g.recompute();
        t.resolve_all();
        assert!(!t.on_battlefield(bears), "{aura}");
        assert!(t.on_battlefield(giant), "{aura}");
        // Tapped; the Aura leaves the battlefield.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let a = attach_new(&mut t, P0, aura, bears);
        tap(&mut t, bears);
        put_in_graveyard(&mut t, a);
        t.settle();
        t.resolve_all();
        assert!(!t.on_battlefield(bears), "{aura}");
    }
    // Cryoshatter: dealt damage.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let a = attach_new(&mut t, P0, "Cryoshatter", giant);
    damage(&mut t, elves, 1, giant);
    // (Regression: the batched "is dealt damage" alternative of a trigger with several
    // conditions triggered.)
    assert_eq!(triggers_on_stack(&t, "destroy"), 1);
    put_in_graveyard(&mut t, a);
    t.settle();
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
}

#[test]
fn crystallization_exiles_the_targeted_creature_even_if_it_left() {
    cr!("608.2h", "603.2", "115.1", "303.4c", "704.5m");
    ruling!(
        "Crystallization",
        "When the enchanted creature becomes the target of a spell or ability, Crystallization’s last ability will trigger. When that ability resolves, that creature will be exiled, even if Crystallization has left the battlefield or is enchanting a different creature by that time. If Crystallization is still enchanting the same creature, Crystallization will then be put into its owner’s graveyard as a state-based action."
    );
    supported("Crystallization");
    // P1 targets the enchanted Bears with Giant Growth: the trigger resolves first; the
    // Bears is exiled and Crystallization goes to the graveyard.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Crystallization", bears);
    let gg = in_hand_with_mana(&mut t, P1, "Giant Growth");
    t.cast(P1, gg).target(bears).go();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "exile"), 1);
    t.resolve();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.in_graveyard(P0, "Crystallization"));
    t.resolve_all();
    // Crystallization moved to another creature in response: the targeted one is exiled.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let c = attach_new(&mut t, P0, "Crystallization", bears);
    let gg = in_hand_with_mana(&mut t, P1, "Giant Growth");
    t.cast(P1, gg).target(bears).go();
    t.settle();
    assert!(t.g.attach(c, Entity::Object(giant)));
    t.g.recompute();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(t.on_battlefield(giant));
    assert!(t.on_battlefield(c));
    // Crystallization left in response: still exiled.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let c = attach_new(&mut t, P0, "Crystallization", bears);
    let gg = in_hand_with_mana(&mut t, P1, "Giant Growth");
    t.cast(P1, gg).target(bears).go();
    t.settle();
    put_in_graveyard(&mut t, c);
    t.settle();
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
}
