//! CR 613 audit: timestamps (CR 613.7) with real cards. What gives a new timestamp
//! (entering a zone, becoming attached, a Room door being unlocked) and what doesn't
//! (a change of control, equipping the creature an Equipment is already attached to),
//! control-changing effects in timestamp order, and keyword counters' timestamps.

use crate::r613_audit_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn gaining_control_of_humility_doesnt_give_its_effect_a_new_timestamp() {
    // CR 613.7a: a static ability's effect has the timestamp of the object it's on; a
    // change of control isn't among the events of CR 613.7d–k that give one. Jump's
    // flying (later than Humility) survives Humility changing hands.
    cr!("613.7", "613.7a", "613.7d", "613.1b", "613.1f");
    ruling!("Steal Enchantment", "They just become its controller");
    let mut t = TestGame::new(2);
    let humility = t.battlefield(P1, "Humility");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Jump", &[Entity::Object(bears)]);
    t.g.recompute();
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    assert_eq!(t.pt(bears), (1, 1));
    cast_resolve(&mut t, P0, "Steal Enchantment", &[Entity::Object(humility)]);
    t.g.recompute();
    assert_eq!(controller(&t, humility), P0);
    assert!(has_kw(&t, bears, KeywordKind::Flying), "Humility keeps its timestamp");
    assert_eq!(t.pt(bears), (1, 1));
}

#[test]
fn a_permanent_returning_to_the_battlefield_has_a_new_timestamp() {
    // CR 613.7d: Humility flickered is a new object with a new timestamp, now later than
    // Jump's effect, so the creature loses the flying Jump gave it.
    cr!("613.7d", "613.7", "400.7", "613.1f");
    let mut t = TestGame::new(2);
    let humility = t.battlefield(P1, "Humility");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Jump", &[Entity::Object(bears)]);
    t.g.recompute();
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    cast_resolve(&mut t, P0, "Flicker", &[Entity::Object(humility)]);
    t.g.recompute();
    assert!(t.on_battlefield(humility));
    assert_eq!(controller(&t, humility), P1);
    assert_ne!(t.g.current(humility), humility, "a new object");
    assert!(!has_kw(&t, bears, KeywordKind::Flying));
    assert_eq!(t.pt(bears), (1, 1));
}

#[test]
fn an_equipment_attached_again_after_turn_to_frog_grants_its_ability_again() {
    // CR 613.7e: an Equipment gets a new timestamp each time it becomes attached, so
    // re-equipping after Turn to Frog gives its flying again; equipping the creature it's
    // already attached to does nothing (CR 701.3b), so that gives no new timestamp.
    cr!("613.7e", "613.7a", "613.7b", "613.1f", "701.3b");
    ruling!(
        "Turn to Frog",
        "If the affected creature gains an ability after Turn to Frog resolves, it will keep that ability"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let sail = t.battlefield(P0, "Neurok Hoversail");
    t.lands(P0, "Wastes", 6);
    assert!(t.g.attach(sail, Entity::Object(bears)));
    t.g.recompute();
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    cast_resolve(&mut t, P1, "Turn to Frog", &[Entity::Object(bears)]);
    t.g.recompute();
    assert!(!has_kw(&t, bears, KeywordKind::Flying));
    assert_eq!(t.obj_now(sail).attached_to, Some(Entity::Object(bears)));
    // Equip the creature it's already attached to: nothing happens.
    activate_containing(&mut t, P0, sail, "Equip", &[Entity::Object(bears)]);
    t.g.recompute();
    assert_eq!(t.obj_now(sail).attached_to, Some(Entity::Object(bears)));
    assert!(!has_kw(&t, bears, KeywordKind::Flying), "no new timestamp");
    // Equip the Elves, then the Frog again: the Hoversail is newer than Turn to Frog.
    activate_containing(&mut t, P0, sail, "Equip", &[Entity::Object(elves)]);
    t.g.recompute();
    assert!(has_kw(&t, elves, KeywordKind::Flying));
    activate_containing(&mut t, P0, sail, "Equip", &[Entity::Object(bears)]);
    t.g.recompute();
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    assert!(!has_kw(&t, elves, KeywordKind::Flying));
    assert_eq!(t.pt(bears), (1, 1));
    assert!(has_subtype(&t, bears, "Frog"));
    // An Aura cast on the Frog after Turn to Frog: its trample and +2/+0 apply.
    cast_resolve(&mut t, P0, "Rancor", &[Entity::Object(bears)]);
    t.g.recompute();
    assert!(has_kw(&t, bears, KeywordKind::Trample));
    assert_eq!(t.pt(bears), (3, 1));
}

#[test]
fn rune_of_flight_on_colossus_hammer_with_real_cards() {
    // CR 613.7a: the ability Rune of Flight grants the Hammer has the Rune's later
    // timestamp, so "equipped creature has flying" applies after the Hammer's own "loses
    // flying"; when the Hammer is attached to another creature both abilities get new
    // timestamps in the same relative order (CR 613.7e).
    cr!("613.7a", "613.7e", "613.1f", "613.9");
    ruling!(
        "Colossus Hammer",
        "If the equipped creature gains flying after Colossus Hammer became attached to it, it will keep flying"
    );
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P0, "Wind Drake");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hammer = t.battlefield(P0, "Colossus Hammer");
    assert!(t.g.attach(hammer, Entity::Object(drake)));
    t.g.recompute();
    assert_eq!(t.pt(drake), (12, 12));
    assert!(!has_kw(&t, drake, KeywordKind::Flying));
    let rune = t.battlefield(P0, "Rune of Flight");
    assert!(t.g.attach(rune, Entity::Object(hammer)));
    t.g.recompute();
    assert!(has_ability_text(&t, hammer, "Equipped creature has flying"));
    assert!(has_kw(&t, drake, KeywordKind::Flying), "the granted ability is later");
    assert!(t.g.attach(hammer, Entity::Object(bears)));
    t.g.recompute();
    assert!(t.obj_now(hammer).timestamp > t.obj_now(rune).timestamp);
    assert_eq!(t.pt(bears), (12, 12));
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    assert_eq!(t.pt(drake), (2, 2));
    assert!(has_kw(&t, drake, KeywordKind::Flying));
}

#[test]
fn control_changing_effects_apply_in_timestamp_order_and_the_earlier_one_resumes() {
    // Layer 2 (CR 613.1b): the later control-changing effect wins (CR 613.7); when it
    // ends (CR 611.2b), the earlier one applies again.
    cr!("613.1b", "613.7", "613.7b", "613.7e", "611.2b");
    ruling!(
        "Rubinia Soulsinger",
        "If Rubinia Soulsinger leaves the battlefield, you no longer control it, so the duration of its control-change effect ends"
    );
    // Control Magic (P0) first, then Rubinia (P1): P1 controls the Bears until Rubinia
    // untaps.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Control Magic", &[Entity::Object(bears)]);
    t.g.recompute();
    assert_eq!(controller(&t, bears), P0);
    let rubinia = t.battlefield(P1, "Rubinia Soulsinger");
    activate_containing(&mut t, P1, rubinia, "Gain control", &[Entity::Object(bears)]);
    t.g.recompute();
    assert_eq!(controller(&t, bears), P1);
    assert!(t.obj_now(rubinia).tapped);
    assert!(t.g.untap(rubinia));
    t.g.recompute();
    assert_eq!(controller(&t, bears), P0, "Rubinia's effect ended; Control Magic applies");
    // Rubinia (P0) first, then P1's Control Magic: P1 controls, and keeps control when
    // Rubinia leaves the battlefield.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let rubinia = t.battlefield(P0, "Rubinia Soulsinger");
    activate_containing(&mut t, P0, rubinia, "Gain control", &[Entity::Object(bears)]);
    t.g.recompute();
    assert_eq!(controller(&t, bears), P0);
    t.set_step(P1, Step::PrecombatMain);
    cast_resolve(&mut t, P1, "Control Magic", &[Entity::Object(bears)]);
    t.g.recompute();
    assert_eq!(controller(&t, bears), P1);
    t.g.destroy(rubinia, None);
    t.settle();
    t.g.recompute();
    assert_eq!(controller(&t, bears), P1);
    // Rubinia leaving ended its effect: with the Aura gone too, the owner controls it.
    let magic = t.named_on_battlefield("Control Magic")[0];
    t.g.destroy(magic, None);
    t.settle();
    t.g.recompute();
    assert_eq!(controller(&t, bears), P1);
}

#[test]
fn keyword_counters_apply_in_layer_6_at_their_timestamp_against_humility() {
    // CR 613.1f: keyword counters are layer 6 effects with the counters' timestamps
    // (CR 613.7c): one put on before Humility entered is overridden, one put on after
    // gives the ability, and a new counter of a kind re-timestamps the ones already there.
    cr!("613.1f", "613.7c", "122.1b", "613.7");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    put_counters(&mut t, bears, "flying", 1);
    put_counters(&mut t, elves, "trample", 1);
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    assert!(has_kw(&t, elves, KeywordKind::Trample));
    t.battlefield(P1, "Humility");
    t.g.recompute();
    assert!(!has_kw(&t, bears, KeywordKind::Flying), "the counter is older than Humility");
    assert!(!has_kw(&t, elves, KeywordKind::Trample));
    put_counters(&mut t, elves, "flying", 1);
    assert!(has_kw(&t, elves, KeywordKind::Flying), "put on after Humility");
    assert!(!has_kw(&t, elves, KeywordKind::Trample), "the trample counter keeps its timestamp");
    // A second flying counter on the Bears: both flying counters get the new timestamp.
    put_counters(&mut t, bears, "flying", 1);
    assert!(has_kw(&t, bears, KeywordKind::Flying));
    assert_eq!(t.pt(bears), (1, 1));
    // A newer Humility overrides them all again.
    t.battlefield(P1, "Humility");
    t.g.recompute();
    assert!(!has_kw(&t, bears, KeywordKind::Flying));
    assert!(!has_kw(&t, elves, KeywordKind::Flying));
}

#[test]
fn dress_down_and_an_animated_land_in_both_orders() {
    cr!("613.1f", "613.7", "613.7a", "613.7b");
    ruling!(
        "Dress Down",
        "it will still gain any abilities given to it by the effect that animated it"
    );
    // Dress Down first: the later animation's flying and infect stay.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Dress Down");
    let nexus = t.battlefield(P0, "Inkmoth Nexus");
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, nexus, "becomes", &[]);
    t.g.recompute();
    assert!(has_kw(&t, nexus, KeywordKind::Flying));
    assert!(has_kw(&t, nexus, KeywordKind::Infect));
    assert!(!has_ability_text(&t, nexus, "Add {C}"), "its printed abilities are lost");
    assert_eq!(t.pt(nexus), (1, 1));
    // Animated first, then Dress Down (flash) enters: the creature loses them too.
    let mut t = TestGame::new(2);
    let nexus = t.battlefield(P0, "Inkmoth Nexus");
    t.lands(P0, "Wastes", 1);
    activate_containing(&mut t, P0, nexus, "becomes", &[]);
    cast_resolve(&mut t, P1, "Dress Down", &[]);
    t.g.recompute();
    assert!(t.obj_now(nexus).is_creature());
    assert!(!has_kw(&t, nexus, KeywordKind::Flying));
    assert!(!has_kw(&t, nexus, KeywordKind::Infect));
    assert_eq!(ability_count(&t, nexus), 0);
    assert_eq!(t.pt(nexus), (1, 1));
}

#[test]
fn an_aura_moved_to_another_creature_gets_a_new_timestamp() {
    // CR 613.7e: an Aura receives a new timestamp each time it becomes attached. Flight
    // attached before Humility is overridden; moved by Aura Graft after Humility, it
    // gives its new host flying.
    cr!("613.7e", "613.7a", "613.1f");
    ruling!(
        "Aura Graft",
        "You can target an Aura you already control just to move that Aura to a new permanent"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let flight = t.battlefield(P0, "Flight");
    assert!(t.g.attach(flight, Entity::Object(bears)));
    t.battlefield(P1, "Humility");
    t.g.recompute();
    assert!(!has_kw(&t, bears, KeywordKind::Flying), "Flight is older than Humility");
    t.answer_choose(P0, &[Entity::Object(elves)]);
    cast_resolve(&mut t, P0, "Aura Graft", &[Entity::Object(flight)]);
    t.g.recompute();
    assert_eq!(t.obj_now(flight).attached_to, Some(Entity::Object(elves)));
    assert!(has_kw(&t, elves, KeywordKind::Flying));
    assert!(!has_kw(&t, bears, KeywordKind::Flying));
    assert_eq!(t.pt(elves), (1, 1));
}

#[test]
fn a_room_door_unlocked_after_an_aura_attached_applies_after_the_auras_effect() {
    // The decided register entry: a door unlocked after the permanent entered has its
    // static abilities timestamped from the unlocking. Secret Arcade's "nonland
    // permanents you control are enchantments in addition" (layer 4) then applies after
    // Kenrith's Transformation's "is a green Elk creature" set the types; re-attaching
    // the Aura (CR 613.7e) puts it after the door again.
    cr!("613.7", "613.7a", "613.7e", "613.1d", "709.5e");
    interp!("room-door-unlock-timestamp");
    let mut t = TestGame::new(2);
    let room = t.battlefield(P0, "Secret Arcade // Dusty Parlor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let aura = t.battlefield(P0, "Kenrith's Transformation");
    assert!(t.g.attach(aura, Entity::Object(bears)));
    t.g.recompute();
    assert!(has_subtype(&t, bears, "Elk"));
    assert!(!has_subtype(&t, bears, "Bear"));
    assert!(!is_type(&t, bears, CardType::Enchantment));
    assert!(!is_type(&t, giant, CardType::Enchantment), "the door is still locked");
    assert!(mtg_engine::rooms::unlock(&mut t.g, room, 0, P0));
    t.g.flush_events();
    t.resolve_all();
    t.g.recompute();
    assert!(is_type(&t, giant, CardType::Enchantment));
    assert!(
        is_type(&t, bears, CardType::Enchantment),
        "the door's later timestamp adds the type after the Aura set the types"
    );
    assert!(has_subtype(&t, bears, "Elk"));
    assert_eq!(t.pt(bears), (3, 3));
    // The Aura attached to the Giant: it's newer than the door now.
    assert!(t.g.attach(aura, Entity::Object(giant)));
    t.g.recompute();
    assert!(has_subtype(&t, giant, "Elk"));
    assert!(!is_type(&t, giant, CardType::Enchantment));
    assert!(is_type(&t, bears, CardType::Enchantment));
    assert!(has_subtype(&t, bears, "Bear"));
    assert_eq!(t.pt(bears), (2, 2));
}
