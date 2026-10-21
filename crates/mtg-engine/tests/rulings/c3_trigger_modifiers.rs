//! Trigger modifiers (cards3-triggers-durations): trigger suppression scoped to dying
//! creatures or to opponents' permanents (CR 603.2, 603.10a), abilities triggering an
//! additional time for a scoped set of sources or events (CR 603.2d), and conditions on
//! the costs paid to activate the ability whose activation triggered (CR 602.2b).

use crate::r_s01_common::{supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn hushbringer_dying_creatures_trigger_nothing() {
    cr!("603.2", "700.4");
    ruling!(
        "Hushbringer",
        "Hushbringer affects a creature's own triggered abilities as well as other triggered abilities that trigger when that creature enters the battlefield or dies."
    );
    supported("Hushbringer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hushbringer");
    // "Whenever this creature or another creature dies, target player loses 1 life and
    // you gain 1 life."
    t.battlefield(P0, "Blood Artist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    assert_eq!(triggers_on_stack(&t, "dies"), 0);
    // Entering: Soul Warden ("Whenever another creature enters, you gain 1 life.").
    t.battlefield(P0, "Soul Warden");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P0), 20);
}

#[test]
fn hushbringer_dying_at_the_same_time_triggers_nothing() {
    cr!("603.10a");
    ruling!(
        "Hushbringer",
        "A creature entering the battlefield or dying at the same time as Hushbringer (including Hushbringer itself entering or dying) won't cause abilities to trigger."
    );
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    t.battlefield(P0, "Hushbringer");
    t.battlefield(P0, "Blood Artist");
    t.battlefield(P0, "Grizzly Bears");
    crate::r_s25_common::cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "dies"), 0);
    // Without Hushbringer, Blood Artist's own death and the Bears' trigger it.
    t.battlefield(P0, "Blood Artist");
    t.battlefield(P0, "Grizzly Bears");
    crate::r_s25_common::cast_new(&mut t, P0, "Wrath of God", &[]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, "dies"), 2);
}

#[test]
fn hushbringer_sacrifice_triggers_still_trigger() {
    cr!("603.2");
    ruling!(
        "Hushbringer",
        "An ability that triggers on an event that causes a creature to die will still trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hushbringer");
    // "Whenever a player sacrifices a permanent, this creature deals 1 damage to any
    // target."
    t.battlefield(P0, "Mayhem Devil");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.g.sacrifice(bears, P0);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "sacrifices"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn elesh_norn_stops_only_opponents_abilities() {
    cr!("603.2", "603.2d", "603.6a");
    ruling!(
        "Elesh Norn, Mother of Machines",
        "Elesh Norn's second and third ability each affects a permanent's own enters-the-battlefield triggered abilities as well as other triggered abilities that trigger when that permanent enters the battlefield."
    );
    supported("Elesh Norn, Mother of Machines");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Elesh Norn, Mother of Machines");
    // "Whenever another creature enters, you gain 1 life."
    t.battlefield(P0, "Soul Warden");
    t.battlefield(P1, "Soul Warden");
    // An opponent's creature entering: the opponent's own Soul Warden doesn't trigger,
    // mine triggers twice.
    t.enter(P1, "Grizzly Bears");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn elesh_norn_entering_at_the_same_time() {
    cr!("603.6a");
    ruling!(
        "Elesh Norn, Mother of Machines",
        "Similarly, a permanent entering the battlefield at the same time as Elesh Norn (still including Elesh Norn) won't cause abilities of permanents an opponent controls to trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Soul Warden");
    t.enter(P0, "Elesh Norn, Mother of Machines");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn panoptic_projektor_face_up_triggers_twice() {
    cr!("603.2d");
    ruling!(
        "Panoptic Projektor",
        "Panoptic Projektor’s last ability affects a permanent’s own abilities that trigger when it turns face up as well as other triggered abilities that trigger when that permanent turns face up."
    );
    supported("Panoptic Projektor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Panoptic Projektor");
    // "Whenever a permanent you control is turned face up, draw a card."
    t.battlefield(P0, "Secret Plans");
    // "When this creature is turned face up, you gain 2 life."
    let w = t.battlefield(P0, "Watcher of the Roost");
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, w));
    let hand = t.hand_size(P0);
    for _ in 0..4 {
        t.library_top(P0, "Island");
    }
    assert!(mtg_engine::facedown::turn_face_up_by(&mut t.g, w, P0));
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.hand_size(P0), hand + 2);
    // Another event (a creature entering) doesn't trigger twice.
    t.battlefield(P0, "Soul Warden");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
}

#[test]
fn cloud_equipment_triggers_twice() {
    cr!("603.2d", "603.10a");
    ruling!(
        "Cloud, Midgar Mercenary",
        "Cloud's last ability doesn't copy the triggered ability; it just causes the ability to trigger an additional time."
    );
    supported("Cloud, Midgar Mercenary");
    let mut t = TestGame::new(2);
    let cloud = t.battlefield(P0, "Cloud, Midgar Mercenary");
    // "Whenever equipped creature dies, draw two cards."
    let clamp = t.battlefield(P0, "Skullclamp");
    assert!(t.g.attach(clamp, Entity::Object(cloud)));
    t.g.recompute();
    // Blood Artist isn't attached to Cloud: its ability triggers once.
    t.battlefield(P0, "Blood Artist");
    for _ in 0..6 {
        t.library_top(P0, "Island");
    }
    let hand = t.hand_size(P0);
    destroy(&mut t, cloud);
    assert_eq!(triggers_on_stack(&t, "draw two cards"), 2);
    assert_eq!(triggers_on_stack(&t, "dies, target player"), 1);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 4);
}

#[test]
fn masamune_doubles_dies_triggers_of_equipped_creature_and_emblems() {
    cr!("603.2d", "114.4", "700.4");
    ruling!(
        "The Masamune",
        "The granted ability affects the equipped creature's own \"when this creature dies\" triggered abilities as well as other triggered abilities of that creature or emblems you own that trigger when a creature dies."
    );
    supported("The Masamune");
    let mut t = TestGame::new(2);
    let artist = t.battlefield(P0, "Blood Artist");
    let sword = t.battlefield(P0, "The Masamune");
    assert!(t.g.attach(sword, Entity::Object(artist)));
    // Another Blood Artist, not equipped: once.
    t.battlefield(P0, "Blood Artist");
    // An emblem P0 owns with Blood Artist's ability.
    let donor = t.exile(P1, "Blood Artist");
    let abilities = t.obj_now(donor).chars.abilities.clone();
    mtg_engine::tokens::create_emblem(&mut t.g, P0, abilities, None);
    t.g.recompute();
    let bears = t.battlefield(P1, "Grizzly Bears");
    destroy(&mut t, bears);
    // Equipped Blood Artist: 2; unequipped: 1; emblem: 2.
    assert_eq!(triggers_on_stack(&t, "dies"), 5);
}

#[test]
fn ashnod_copies_abilities_paid_for_by_sacrificing() {
    cr!("602.2b", "707.10");
    ruling!(
        "Ashnod the Uncaring",
        "“Sacrificed to activate it” means that a permanent was sacrificed to pay the ability’s costs."
    );
    supported("Ashnod the Uncaring");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ashnod the Uncaring");
    for _ in 0..3 {
        t.library_top(P0, "Forest");
    }
    let elder = t.battlefield(P0, "Sakura-Tribe Elder");
    t.answer_yes(P0, true);
    t.activate(P0, elder, 0, &[]).expect("activate");
    t.settle();
    t.resolve_all();
    // The ability and its copy each put a Forest onto the battlefield.
    assert_eq!(t.named_on_battlefield("Forest").len(), 2);
    // An ability with no sacrifice in its cost isn't copied.
    let sorcerer = t.battlefield(P0, "Prodigal Sorcerer");
    t.activate(P0, sorcerer, 0, &[Entity::Player(P1)]).expect("activate");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn verrak_pays_the_life_again_and_copies() {
    cr!("602.2b", "119.4", "707.10");
    ruling!(
        "Verrak, Warped Sengir",
        "Verrak’s triggered ability creates another instance of that activated ability on the stack."
    );
    supported("Verrak, Warped Sengir");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Verrak, Warped Sengir");
    // "{B}, Pay 2 life: Draw a card."
    let greed = t.battlefield(P0, "Greed");
    t.lands(P0, "Swamp", 2);
    for _ in 0..4 {
        t.library_top(P0, "Island");
    }
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.activate(P0, greed, 0, &[]).expect("activate");
    t.settle();
    t.resolve_all();
    // 2 life to activate, 2 more for the copy; two cards drawn.
    assert_eq!(t.life(P0), 16);
    assert_eq!(t.hand_size(P0), hand + 2);
    // Declining to pay: no copy.
    t.answer_yes(P0, false);
    t.activate(P0, greed, 0, &[]).expect("activate");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 14);
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn way_of_the_mind_sculptor_counts_loyalty_removed() {
    cr!("606.4", "602.2b");
    supported("Way of the Mind Sculptor");
    for (index, draws) in [(1usize, 2usize), (0, 0)] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        t.enter(P0, "Way of the Mind Sculptor");
        t.settle();
        t.resolve_all();
        let jace = crate::r_s01_common::with_subtype(&t, P0, "Jace")[0];
        assert_eq!(t.counters(jace, "loyalty"), 5);
        for _ in 0..4 {
            t.library_top(P0, "Island");
        }
        let hand = t.hand_size(P0);
        // [−3]: Draw a card (and Way's trigger draws one); [−1]: Surveil 1 (no trigger).
        t.activate(P0, jace, index, &[]).expect("activate");
        t.settle();
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + draws, "ability {index}");
    }
}
