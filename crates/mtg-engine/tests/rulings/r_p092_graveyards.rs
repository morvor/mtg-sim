//! Rulings batch P092 — graveyard-watching abilities: "one or more cards leave your
//! graveyard" triggers, Magus of the Bridge, and characteristic values counted from
//! graveyards (Altar of the Goyf, Invade the City, Embodiment of Agonies, Bonehoard,
//! Exoskeletal Armor, Undergrowth Scavenger, Serpentine Curve, Vile Manifestation).

use crate::r_p017_common::animate;
use crate::r_p120_common::plus1;
use crate::r_s01_common::*;
use crate::r_s02_common::{create_token, destroy};
use crate::r_s05_common::move_to;
use crate::r_s06_common::{give_control, has_kw};
use crate::r_s09_common::declare;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// P0's whole graveyard is exiled at once (Bojuka Bog's "enters" ability).
fn bog_own_graveyard(t: &mut TestGame) {
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.enter(P0, "Bojuka Bog");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn one_or_more_cards_leaving_your_graveyard_at_once_trigger_once() {
    cr!("603.2c", "603.10a");
    ruling!(
        "Garrison Excavator",
        "If multiple cards leave your graveyard at the same time, Garrison Excavator's last ability will trigger only once."
    );
    ruling!(
        "Fuming Effigy",
        "If multiple cards leave your graveyard at the same time, this ability triggers only once, and will deal only 1 damage."
    );
    ruling!(
        "Cyan, Vengeful Samurai",
        "If multiple creature cards leave your graveyard at the same time, Cyan's last ability will trigger only once."
    );
    ruling!(
        "Desecrated Tomb",
        "You create one Bat token each time Desecrated Tomb’s ability triggers, no matter how many cards left your graveyard."
    );
    for name in [
        "Garrison Excavator",
        "Fuming Effigy",
        "Cyan, Vengeful Samurai",
        "Desecrated Tomb",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let perm = t.battlefield(P0, name);
        t.graveyard(P0, "Grizzly Bears");
        t.graveyard(P0, "Hill Giant");
        t.graveyard(P0, "Llanowar Elves");
        bog_own_graveyard(&mut t);
        match name {
            "Garrison Excavator" => assert_eq!(with_subtype(&t, P0, "Spirit").len(), 1),
            "Fuming Effigy" => assert_eq!(t.life(P1), 19),
            "Cyan, Vengeful Samurai" => assert_eq!(plus1(&t, perm), 1),
            _ => assert_eq!(with_subtype(&t, P0, "Bat").len(), 1),
        }
    }
}

#[test]
fn magus_of_the_bridge_looks_at_whose_graveyard_not_who_controlled_it() {
    cr!("108.3", "111.2", "404.2", "603.6a");
    ruling!(
        "Magus of the Bridge",
        "Neither ability cares who controlled the creature that died, only which graveyard it was put into. This means that if an opponent controls a creature you own, its death causes Magus of the Bridge's first ability to trigger, not its second."
    );
    ruling!(
        "Magus of the Bridge",
        "A token's owner is the player who created it, which may be different from the player who controls it"
    );
    supported("Magus of the Bridge");
    // P0's Grizzly Bears, controlled by P1, dies: into P0's graveyard.
    let mut t = TestGame::new(2);
    let magus = t.battlefield(P0, "Magus of the Bridge");
    let bears = t.battlefield(P0, "Grizzly Bears");
    give_control(&mut t, bears, P1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(with_subtype(&t, P0, "Zombie").len(), 1);
    assert!(t.on_battlefield(magus));
    // A token P1 created, controlled by P0, dies: into P1's graveyard (briefly).
    let token = create_token(&mut t, P1, "Goblin");
    give_control(&mut t, token, P0);
    destroy(&mut t, token);
    t.resolve_all();
    assert!(!t.on_battlefield(magus));
    assert_eq!(with_subtype(&t, P0, "Zombie").len(), 1);
}

#[test]
fn magus_of_the_bridge_does_nothing_from_the_graveyard() {
    cr!("113.6");
    ruling!(
        "Magus of the Bridge",
        "Unlike its namesake, Bridge from Below, the Magus's abilities work only while it's on the battlefield. They do not work while it is in your graveyard."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Magus of the Bridge");
    let bears = t.battlefield(P0, "Grizzly Bears");
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(with_subtype(&t, P0, "Zombie").is_empty());
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn altar_of_the_goyf_triggers_only_for_a_lone_attacker() {
    cr!("506.5", "508.1", "603.2");
    ruling!(
        "Altar of the Goyf",
        "A creature attacks alone if it's the only creature declared as an attacker during the declare attackers step (including creatures controlled by your teammates, if applicable). For example, Altar of the Goyf's first ability won't trigger if you attack with multiple creatures and all but one of them are removed from combat."
    );
    supported("Altar of the Goyf");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Altar of the Goyf");
    t.graveyard(P1, "Grizzly Bears");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let p1 = Entity::Player(P1);
    declare(&mut t, P0, &[(bears, p1), (giant, p1)]);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    mtg_engine::combat::remove_from_combat(&mut t.g, giant);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn altar_of_the_goyf_counts_card_types_only() {
    cr!("205.2a", "205.3", "205.4", "308.1");
    ruling!(
        "Altar of the Goyf",
        "The card types that can appear in a graveyard are artifact, battle, creature, enchantment, instant, kindred, land, planeswalker, and sorcery. Legendary, basic, and snow are supertypes, not card types; Aura and Lhurgoyf are subtypes, not card types."
    );
    supported("Crib Swap");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Altar of the Goyf");
    // artifact + creature, land (basic), enchantment (Aura), kindred + instant: 6 types.
    t.graveyard(P1, "Ornithopter");
    t.graveyard(P1, "Forest");
    t.graveyard(P0, "Holy Strength");
    t.graveyard(P0, "Crib Swap");
    t.graveyard(P0, "Snow-Covered Forest");
    let bears = t.battlefield(P0, "Grizzly Bears");
    declare(&mut t, P0, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (8, 8));
}

#[test]
fn an_animated_altar_of_the_goyf_has_trample() {
    cr!("205.3m", "613.1d", "613.1f");
    ruling!(
        "Altar of the Goyf",
        "If Altar of the Goyf somehow becomes a creature and the effect allows to keep its other types, it will give itself trample."
    );
    let mut t = TestGame::new(2);
    let altar = t.battlefield(P0, "Altar of the Goyf");
    assert!(!has_kw(&t, altar, KeywordKind::Trample));
    animate(&mut t, altar, 5);
    assert!(has_kw(&t, altar, KeywordKind::Trample));
}

#[test]
fn attacks_alone_is_decided_by_the_declaration() {
    cr!("506.5", "508.3a", "508.4");
    ruling!(
        "Bilbo's Ring",
        "A creature attacks alone if it's the only creature declared as an attacker during the declare attackers step. For example, Bilbo's Ring's second ability won't trigger if you attack with multiple creatures and all but one of them are removed from combat."
    );
    ruling!(
        "Grasping Shadows // Shadows' Lair",
        "A creature attacks alone if it's the only creature declared as an attacker during the declare attackers step. For example, the triggered ability of Grasping Shadows won't trigger if you attack with multiple creatures and all but one of them are removed from combat."
    );
    ruling!(
        "Derelict Attic // Widow's Walk",
        "Similarly, creatures that enter attacking later in combat won't be considered when determining whether or not a creature attacked alone."
    );
    supported("Bilbo's Ring");
    supported("Grasping Shadows // Shadows' Lair");
    supported("Derelict Attic // Widow's Walk");
    let p1 = Entity::Player(P1);
    for alone in [false, true] {
        let mut t = TestGame::new(2);
        let ring = t.battlefield(P0, "Bilbo's Ring");
        let shadows = t.battlefield(P0, "Grasping Shadows // Shadows' Lair");
        let room = t.battlefield(P0, "Derelict Attic // Widow's Walk");
        assert!(mtg_engine::rooms::unlock(&mut t.g, room, 1, P0));
        t.resolve_all();
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        assert!(t.g.attach(ring, obj(bears)));
        let life = t.life(P0);
        let hand = t.hand_size(P0);
        let decl: &[(ObjectId, Entity)] = if alone {
            &[(bears, p1)]
        } else {
            &[(bears, p1), (giant, p1)]
        };
        declare(&mut t, P0, decl);
        t.settle();
        if !alone {
            mtg_engine::combat::remove_from_combat(&mut t.g, giant);
        } else {
            // Another creature enters attacking: the Grizzly Bears still attacked alone.
            let late = t.battlefield(P0, "Llanowar Elves");
            mtg_engine::combat::put_onto_battlefield_attacking(&mut t.g, late, p1);
        }
        t.settle();
        t.resolve_all();
        let n = alone as u32;
        assert_eq!(t.hand_size(P0), hand + n as usize, "Bilbo's Ring draws: alone={alone}");
        assert_eq!(t.life(P0), life - n as i32, "alone={alone}");
        assert_eq!(t.counters(shadows, "dread"), n, "alone={alone}");
        // Widow's Walk: +1/+0 and deathtouch; Grasping Shadows: deathtouch and lifelink.
        assert_eq!(t.pt(bears).0, 2 + n as i32, "alone={alone}");
        assert_eq!(has_kw(&t, bears, KeywordKind::Deathtouch), alone);
    }
}

#[test]
fn invade_the_city_counts_its_graveyard_as_it_resolves_without_itself() {
    cr!("608.2h", "701.47a", "709.3");
    ruling!(
        "Invade the City",
        "Invade the City is still on the stack while you count your instant and sorcery cards in your graveyard. It doesn't count itself."
    );
    ruling!(
        "Invade the City",
        "The number of instant and sorcery cards in your graveyard is counted only as Invade the City resolves."
    );
    ruling!(
        "Invade the City",
        "A split card that's both an instant and a sorcery is counted only once for Invade the City."
    );
    supported("Invade the City");
    supported("Destined // Lead");
    let mut t = TestGame::new(2);
    // Destined is an instant, Lead a sorcery: one card.
    t.graveyard(P0, "Destined // Lead");
    give_mana_for(&mut t, P0, "Invade the City");
    let c = t.hand(P0, "Invade the City");
    t.cast(P0, c).go();
    // In response, a second instant reaches the graveyard.
    t.graveyard(P0, "Lightning Bolt");
    t.resolve_all();
    let army = with_subtype(&t, P0, "Army");
    assert_eq!(army.len(), 1);
    assert_eq!(plus1(&t, army[0]), 2);
    assert!(t.in_graveyard(P0, "Invade the City"));
}

/// Embodiment of Agonies enters with `P0`'s graveyard holding `cards`; its +1/+1 counters.
fn embodiment_counters(cards: &[&str]) -> u32 {
    let mut t = TestGame::new(2);
    for c in cards {
        supported(c);
        t.graveyard(P0, c);
    }
    let e = t.enter(P0, "Embodiment of Agonies");
    t.resolve_all();
    plus1(&t, e)
}

#[test]
fn embodiment_of_agonies_compares_printed_mana_costs() {
    cr!("202.1", "202.1a", "107.4e", "709.4d");
    ruling!(
        "Embodiment of Agonies",
        "Alternative costs, additional costs you could pay, cost increases, and cost reductions are all ignored by Embodiment of Agonies."
    );
    ruling!(
        "Embodiment of Agonies",
        "Embodiment of Agonies checks the mana symbols in a card's mana cost. For example, a mana cost of {X} is different than a mana cost of {0}."
    );
    ruling!(
        "Embodiment of Agonies",
        "Every Magic card has either no mana cost or exactly one mana cost. A card with no mana cost is never counted by Embodiment of Agonies. A card with a mana cost of {0} is counted."
    );
    ruling!(
        "Embodiment of Agonies",
        "Hybrid mana symbols are treated as the actual symbols, not as either of their two components individually. For example, {B/R}, {B}, and {R} are three different mana costs."
    );
    ruling!(
        "Embodiment of Agonies",
        "The mana cost of a split card is determined by its two halves combined. For example, the mana cost of Fire//Ice in your graveyard is {2}{U}{R}, the same as the mana cost of Ral's Outburst."
    );
    supported("Embodiment of Agonies");
    // Force of Will (alternative cost) and Air Elemental share {3}{U}{U}; Burst Lightning
    // (kicker) and Lightning Bolt share {R}.
    assert_eq!(
        embodiment_counters(&["Force of Will", "Air Elemental", "Burst Lightning", "Lightning Bolt"]),
        2
    );
    // {X} and {0} differ.
    assert_eq!(embodiment_counters(&["Endless One", "Ornithopter"]), 2);
    // No mana cost isn't counted; {0} is.
    assert_eq!(embodiment_counters(&["Ancestral Vision"]), 0);
    assert_eq!(embodiment_counters(&["Ancestral Vision", "Ornithopter"]), 1);
    // {B/R}, {B}, {R}.
    assert_eq!(
        embodiment_counters(&["Rakdos Cackler", "Dark Ritual", "Lightning Bolt"]),
        3
    );
    // Fire // Ice is {2}{U}{R}, like Ral's Outburst.
    assert_eq!(embodiment_counters(&["Fire // Ice", "Ral's Outburst"]), 1);
    assert_eq!(embodiment_counters(&["Fire // Ice", "Shock"]), 2);
}

#[test]
fn embodiment_of_agonies_returned_from_the_graveyard_counts_itself() {
    cr!("614.1c", "614.12");
    ruling!(
        "Embodiment of Agonies",
        "If you return Embodiment of Agonies from your graveyard to the battlefield, its last ability counts itself."
    );
    let mut t = TestGame::new(2);
    let e = t.graveyard(P0, "Embodiment of Agonies");
    t.graveyard(P0, "Lightning Bolt");
    move_to(&mut t, e, Zone::Battlefield);
    t.resolve_all();
    assert_eq!(plus1(&t, e), 2);
}

#[test]
fn undergrowth_scavenger_from_the_graveyard_counts_itself() {
    cr!("614.1c", "614.12");
    ruling!(
        "Undergrowth Scavenger",
        "If Undergrowth Scavenger enters the battlefield from a graveyard, it will count itself when determining how many +1/+1 counters it enters with."
    );
    supported("Undergrowth Scavenger");
    let mut t = TestGame::new(2);
    let u = t.graveyard(P0, "Undergrowth Scavenger");
    t.graveyard(P1, "Grizzly Bears");
    move_to(&mut t, u, Zone::Battlefield);
    t.resolve_all();
    assert_eq!(plus1(&t, u), 2);
    assert_eq!(t.pt(u), (2, 2));
}

#[test]
fn bonehoard_and_exoskeletal_armor_count_creature_cards_continuously() {
    cr!("611.3a", "111.7", "704.5d");
    ruling!(
        "Bonehoard",
        "The value of X is calculated continuously as the number of creature cards in graveyards changes."
    );
    ruling!(
        "Bonehoard",
        "Although creature tokens go to the graveyard before ceasing to exist, they never count as creature cards and won't increase the bonus granted by Bonehoard, however briefly."
    );
    ruling!(
        "Exoskeletal Armor",
        "The value of X changes as the number of creatures cards in graveyards changes."
    );
    supported("Bonehoard");
    supported("Exoskeletal Armor");
    let mut t = TestGame::new(2);
    t.graveyard(P1, "Grizzly Bears");
    let bonehoard = t.enter(P0, "Bonehoard");
    t.resolve_all();
    let germ = with_subtype(&t, P0, "Germ")[0];
    let wall = t.battlefield(P0, "Wall of Stone");
    let armor = t.battlefield(P0, "Exoskeletal Armor");
    assert!(t.g.attach(armor, obj(wall)));
    t.g.recompute();
    assert_eq!(t.pt(germ), (1, 1));
    assert_eq!(t.pt(wall), (1, 9));
    // A creature token dies: it never counts. Seen while it's in the graveyard, too.
    let token = create_token(&mut t, P1, "Goblin");
    let token_now = t.g.current(token);
    t.g.destroy(token_now, None);
    t.g.recompute();
    assert_eq!(t.zone(token), Zone::Graveyard(P1), "before state-based actions");
    assert_eq!(t.pt(germ), (1, 1));
    t.settle();
    assert_eq!(t.pt(germ), (1, 1));
    // More creature cards: more bonus.
    t.graveyard(P0, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.pt(germ), (2, 2));
    assert_eq!(t.pt(wall), (2, 10));
    let _ = bonehoard;
}

#[test]
fn serpentine_curve_doesnt_count_itself_or_face_down_exiled_cards() {
    cr!("608.2h", "406.3", "708.2");
    ruling!(
        "Serpentine Curve",
        "Serpentine Curve itself is not yet in your graveyard when you determine the value of X."
    );
    ruling!(
        "Serpentine Curve",
        "If you own face-down cards in exile, they won’t count toward the value of X, even if you’re allowed to look at them and you know they are instant or sorcery cards."
    );
    supported("Serpentine Curve");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Lightning Bolt");
    t.exile(P0, "Shock");
    let hidden = t.exile(P0, "Divination");
    t.g.objects[hidden.0 as usize].face_down = true;
    t.g.recompute();
    give_mana_for(&mut t, P0, "Serpentine Curve");
    let c = t.hand(P0, "Serpentine Curve");
    t.cast(P0, c).go();
    t.resolve_all();
    let fractal = with_subtype(&t, P0, "Fractal");
    assert_eq!(fractal.len(), 1);
    // One plus Lightning Bolt and Shock.
    assert_eq!(plus1(&t, fractal[0]), 3);
}

#[test]
fn vile_manifestation_has_power_0_off_the_battlefield() {
    cr!("611.3a", "113.6");
    ruling!(
        "Vile Manifestation",
        "Vile Manifestation’s first ability applies only while it’s on the battlefield. In all other zones, its power is 0."
    );
    supported("Vile Manifestation");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Vile Manifestation");
    t.graveyard(P0, "Krosan Tusker");
    let in_hand = t.hand(P0, "Vile Manifestation");
    let on_bf = t.battlefield(P0, "Vile Manifestation");
    t.g.recompute();
    assert_eq!(t.pt(in_hand), (0, 4));
    assert_eq!(t.pt(on_bf), (2, 4));
}
