//! Rulings batch S30 — lifelink life-gain events (CR 702.15b, 119.9): each creature with
//! lifelink dealing combat damage is a separate life-gaining event, so two of them make a
//! "whenever you gain life" ability trigger twice, while one creature dealing combat
//! damage to several creatures and players at once makes it trigger only once. And
//! combat versus noncombat damage (CR 120.2).

use crate::r_s01_common::{supported, with_subtype};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 attacks P1 with two Child of Night (2/1 lifelink), unblocked, and resolves the
/// triggers.
fn two_lifelinkers(t: &mut TestGame) {
    let a = t.battlefield(P0, "Child of Night");
    let b = t.battlefield(P0, "Child of Night");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    t.resolve_all();
}

/// P0 attacks P1 with Grizzly Bears wearing Shadowspear (3/3 trample, lifelink); P1's
/// Raging Goblin blocks it: 1 damage to the Goblin and 2 to P1 at the same time.
fn one_trampling_lifelinker(t: &mut TestGame) {
    let bears = t.battlefield(P0, "Grizzly Bears");
    let spear = t.battlefield(P0, "Shadowspear");
    t.g.attach(spear, Entity::Object(bears));
    t.g.recompute();
    let goblin = t.battlefield(P1, "Raging Goblin");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[(goblin, bears)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Raging Goblin"));
}

#[test]
fn bloodbond_vampire_triggers_once_per_lifelinking_creature() {
    cr!("702.15b", "119.9", "510.2");
    ruling!(
        "Bloodbond Vampire",
        "A creature with lifelink dealing combat damage is a single life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, the ability will trigger twice."
    );
    supported("Bloodbond Vampire");
    // "Whenever you gain life, put a +1/+1 counter on this creature."
    let mut t = TestGame::new(2);
    let vampire = t.battlefield(P0, "Bloodbond Vampire");
    two_lifelinkers(&mut t);
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.counters(vampire, "+1/+1"), 2);
    let mut t = TestGame::new(2);
    let vampire = t.battlefield(P0, "Bloodbond Vampire");
    one_trampling_lifelinker(&mut t);
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.counters(vampire, "+1/+1"), 1);
}

#[test]
fn two_lifelinkers_are_two_events_for_first_time_each_turn_abilities_too() {
    cr!("702.15b", "119.9", "603.2c");
    ruling!(
        "Attended Healer",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, a “whenever you gain life” ability will trigger twice."
    );
    supported("Attended Healer");
    // Ajani's Pridemate's "whenever you gain life" ability triggers twice; Attended
    // Healer's "Whenever you gain life for the first time each turn, create a 1/1 white Cat
    // creature token" triggers only for the first of the two events.
    let mut t = TestGame::new(2);
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    t.battlefield(P0, "Attended Healer");
    two_lifelinkers(&mut t);
    assert_eq!(t.counters(pridemate, "+1/+1"), 2);
    // (Ajani's Pridemate is a Cat too; one Cat token.)
    let cats = with_subtype(&t, P0, "Cat");
    assert_eq!(cats.iter().filter(|c| t.obj_now(**c).is_token()).count(), 1);
    // One creature dealing damage to a creature and a player: once.
    let mut t = TestGame::new(2);
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    one_trampling_lifelinker(&mut t);
    assert_eq!((t.life(P0), t.life(P1)), (23, 18));
    assert_eq!(t.counters(pridemate, "+1/+1"), 1);
}

#[test]
fn trudge_garden_triggers_twice_for_two_lifelinkers() {
    cr!("702.15b", "119.9");
    ruling!(
        "Trudge Garden",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, a \"whenever you gain life\" ability will trigger twice. However, if a single creature you control with lifelink deals combat damage to multiple creatures, players, and/or planeswalkers at the same time (perhaps because it has trample or was blocked by more than one creature), the ability will trigger only once."
    );
    supported("Trudge Garden");
    // "Whenever you gain life, you may pay {2}. If you do, create a 4/4 green Fungus Beast
    // creature token with trample."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trudge Garden");
    t.lands(P0, "Wastes", 4);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    two_lifelinkers(&mut t);
    assert_eq!(with_subtype(&t, P0, "Fungus").len(), 2);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Trudge Garden");
    t.lands(P0, "Wastes", 4);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    one_trampling_lifelinker(&mut t);
    assert_eq!(with_subtype(&t, P0, "Fungus").len(), 1);
}

#[test]
fn dina_makes_each_opponent_lose_1_life_per_lifelink_event() {
    cr!("702.15b", "119.9");
    ruling!(
        "Dina, Soul Steeper",
        "Each creature with lifelink dealing combat damage causes a separate life gain event. For example, if two creatures you control with lifelink deal combat damage at the same time, a \"whenever you gain life\" ability will trigger twice."
    );
    supported("Dina, Soul Steeper");
    // "Whenever you gain life, each opponent loses 1 life."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dina, Soul Steeper");
    two_lifelinkers(&mut t);
    assert_eq!(t.life(P1), 20 - 4 - 2);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dina, Soul Steeper");
    one_trampling_lifelinker(&mut t);
    assert_eq!(t.life(P1), 20 - 2 - 1);
}

#[test]
fn archangel_of_thune_triggers_for_each_lifelinking_creature() {
    cr!("702.15b", "119.9");
    ruling!(
        "Archangel of Thune",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, the ability will trigger twice."
    );
    supported("Archangel of Thune");
    // Archangel of Thune (3/4 flying, lifelink; "Whenever you gain life, put a +1/+1
    // counter on each creature you control") and Child of Night attack together.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Archangel of Thune");
    let child = t.battlefield(P0, "Child of Night");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[(angel, Entity::Player(P1)), (child, Entity::Player(P1))],
        &[],
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    assert_eq!(t.counters(angel, "+1/+1"), 2);
    assert_eq!(t.counters(child, "+1/+1"), 2);
    // One lifelinking creature dealing damage to a creature and a player: once.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Archangel of Thune");
    one_trampling_lifelinker(&mut t);
    assert_eq!(t.counters(angel, "+1/+1"), 1);
}

#[test]
fn noncombat_damage_dealt_during_combat_by_an_attacker_is_still_noncombat() {
    cr!("120.2a", "120.2b");
    ruling!(
        "Chandra's Spitfire",
        "Combat damage is the damage that's dealt automatically by attacking and blocking creatures. Any other damage is noncombat damage, even if it's dealt during a combat phase by an attacking or blocking creature."
    );
    supported("Chandra's Spitfire");
    supported("Falkenrath Perforator");
    // Chandra's Spitfire: "Whenever an opponent is dealt noncombat damage, this creature
    // gets +3/+0 until end of turn." Falkenrath Perforator: "Whenever this creature
    // attacks, it deals 1 damage to defending player."
    let mut t = TestGame::new(2);
    let spitfire = t.battlefield(P0, "Chandra's Spitfire");
    let perforator = t.battlefield(P0, "Falkenrath Perforator");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(
        &[
            (spitfire, Entity::Player(P1)),
            (perforator, Entity::Player(P1)),
        ],
        &[],
    );
    t.resolve_all();
    // The attacking Perforator's 1 damage during the declare attackers step was
    // noncombat damage: one trigger. The combat damage dealt afterward didn't trigger it.
    assert_eq!(t.pt(spitfire), (4, 3));
    let (sp, pp) = (1 + 3, t.pt(perforator).0);
    assert_eq!(t.life(P1), 20 - 1 - sp - pp);
}
