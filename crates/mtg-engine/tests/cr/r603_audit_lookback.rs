//! CR 603.10 audit: abilities that "look back in time". Leaves-the-battlefield abilities
//! (and the other 603.10a–f exceptions) trigger based on the abilities that existed and on
//! what the objects looked like immediately before the event: a permanent leaving at the
//! same time as the others still sees them leave, an ability a permanent had only thanks
//! to something that left at the same time still triggers, and the control and type of
//! the leaving objects are those they had just before.

use crate::r603_audit_common::*;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Wrath of God (four Plains for it).
fn wrath(t: &mut TestGame) {
    t.lands(P0, "Plains", 4);
    let w = t.hand(P0, "Wrath of God");
    t.cast(P0, w).go();
    t.resolve();
}

/// P0 casts Akroma's Vengeance (destroy all artifacts, creatures, and enchantments).
fn vengeance(t: &mut TestGame) {
    t.lands(P0, "Plains", 6);
    let v = t.hand(P0, "Akroma's Vengeance");
    t.cast(P0, v).go();
    t.resolve();
}

#[test]
fn a_dies_trigger_leaving_with_the_others_triggers_for_each_of_them() {
    cr!("603.10", "603.10a", "603.6c");
    ruling!(
        "Blood Artist",
        "If Blood Artist and one or more other creatures die at the same time, its ability will trigger for each of those creatures."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Artist");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    targets_n(&mut t, P0, Entity::Player(P1), 3);
    wrath(&mut t);
    assert_eq!(triggers_of(&t, "Blood Artist"), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
}

#[test]
fn a_creature_dies_under_the_control_it_had_just_before_the_event() {
    cr!("603.10", "603.10a");
    ruling!(
        "Zulaport Cutthroat",
        "If another creature you control dies at the same time as Zulaport Cutthroat, its ability triggers for that creature."
    );
    supported("Control Magic");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Zulaport Cutthroat");
    t.battlefield(P1, "Zulaport Cutthroat");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 4);
    let cm = t.hand(P0, "Control Magic");
    t.cast(P0, cm).target(bears).go();
    t.resolve();
    assert_eq!(t.obj_now(bears).controller, P0);
    // The Aura stealing the Bears is destroyed at the same time as the Bears: they still
    // died under P0's control.
    vengeance(&mut t);
    assert_eq!(
        trigger_controllers(&t, "Zulaport Cutthroat"),
        vec![P0, P0, P1],
        "P0's Cutthroat triggers for itself and the stolen Bears, P1's only for itself"
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 20 + 2 - 1);
    assert_eq!(t.life(P1), 20 - 2 + 1);
}

#[test]
fn artifacts_animated_by_a_permanent_destroyed_with_them_die_as_creatures() {
    cr!("603.10", "603.10a");
    supported("March of the Machines");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Blood Artist");
    t.battlefield(P0, "March of the Machines");
    let ring = t.battlefield(P0, "Sol Ring");
    t.battlefield(P0, "Mind Stone");
    t.g.recompute();
    assert!(t.obj_now(ring).is(CardType::Creature));
    targets_n(&mut t, P0, Entity::Player(P1), 3);
    // March of the Machines leaves at the same time as the artifacts: immediately before
    // the event they were creatures, so each of them died.
    vengeance(&mut t);
    assert_eq!(triggers_of(&t, "Blood Artist"), 3);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_dies_ability_removed_by_an_effect_leaving_at_the_same_time_doesnt_trigger() {
    cr!("603.10", "603.10a");
    supported("Humility");
    supported("Doomed Traveler");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Humility");
    t.battlefield(P0, "Doomed Traveler");
    // Humility is destroyed with the Traveler: immediately before, the Traveler had no
    // abilities, so its dies ability doesn't exist to trigger.
    vengeance(&mut t);
    assert_eq!(t.stack_len(), 0);
    assert_eq!(tokens_named(&t, P0, "Spirit"), 0);
    // Without Humility, the same wipe gives the Spirit.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doomed Traveler");
    vengeance(&mut t);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Spirit"), 1);
}

#[test]
fn an_ability_lost_because_of_the_event_still_triggers_on_it() {
    cr!("603.10", "603.10a");
    supported("Relic Vial");
    let mut t = TestGame::new(2);
    let vial = t.battlefield(P0, "Relic Vial");
    // Soul Warden is P0's only Cleric: Relic Vial has its ability until the Wrath kills it.
    t.battlefield(P0, "Soul Warden");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    wrath(&mut t);
    assert!(t.on_battlefield(vial), "Wrath of God doesn't destroy artifacts");
    // It no longer has the ability, but it had it immediately before the creatures died.
    assert_eq!(
        triggers_of(&t, "Relic Vial"),
        2,
        "once for each creature P0 controlled (the Warden and P0's Bears)"
    );
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn an_ability_granted_while_a_cleric_lives_triggers_when_both_leave_together() {
    cr!("603.10a");
    ruling!(
        "Relic Vial",
        "If Relic Vial leaves the battlefield at the same time as each Cleric you control, the triggered ability triggers for any creatures you control that die at the same time (possibly including those Clerics)."
    );
    ruling!(
        "Relic Vial",
        "Once Relic Vial’s ability has triggered, that ability resolves even if you no longer control a Cleric."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Relic Vial");
    t.battlefield(P0, "Soul Warden");
    t.battlefield(P0, "Grizzly Bears");
    vengeance(&mut t);
    assert_eq!(triggers_of(&t, "Relic Vial"), 2);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn sacrificing_the_only_cleric_as_a_cost_triggers_the_ability_it_granted() {
    cr!("603.10a");
    ruling!(
        "Relic Vial",
        "If you sacrifice the only Cleric you control to activate Relic Vial’s first ability, the triggered ability it gives itself triggers."
    );
    let mut t = TestGame::new(2);
    let vial = t.battlefield(P0, "Relic Vial");
    let warden = t.battlefield(P0, "Soul Warden");
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[Entity::Object(warden)]);
    t.activate(P0, vial, 0, &[]).expect("activate Relic Vial");
    t.settle();
    assert!(t.in_graveyard(P0, "Soul Warden"));
    assert_eq!(triggers_of(&t, "Relic Vial"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 21);
}

#[test]
fn two_wizards_dying_together_each_had_the_granted_dies_ability() {
    cr!("603.10a");
    ruling!(
        "Expedition Diviner",
        "If Expedition Diviner dies at the same time that each other Wizard you control leaves the battlefield, its ability triggers."
    );
    supported("Expedition Diviner");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Expedition Diviner");
    t.battlefield(P0, "Expedition Diviner");
    let hand = t.hand_size(P0);
    wrath(&mut t);
    assert_eq!(triggers_of(&t, "Expedition Diviner"), 2);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn an_equipment_destroyed_with_the_equipped_creature_sees_it_die() {
    cr!("603.10a", "603.6c");
    supported("Skullclamp");
    let mut t = TestGame::new(2);
    let clamp = t.battlefield(P0, "Skullclamp");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(clamp, Entity::Object(bears));
    t.g.recompute();
    let hand = t.hand_size(P0);
    vengeance(&mut t);
    assert!(t.in_graveyard(P0, "Skullclamp"));
    assert_eq!(triggers_of(&t, "Skullclamp"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

/// P0's Grafted Exoskeleton attached to P0's Grizzly Bears: (Exoskeleton, Bears).
fn exoskeleton_on_bears(t: &mut TestGame) -> (ObjectId, ObjectId) {
    let exo = t.battlefield(P0, "Grafted Exoskeleton");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.attach(exo, Entity::Object(bears));
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 4));
    (exo, bears)
}

#[test]
fn an_equipment_leaving_the_battlefield_becomes_unattached_and_looks_back() {
    cr!("603.10c", "701.3d", "704.5n");
    ruling!(
        "Grafted Exoskeleton",
        "Grafted Exoskeleton becomes unattached from the creature it's equipping if a spell or ability (such as Fulgent Distraction) causes it to become unattached, if Grafted Exoskeleton leaves the battlefield"
    );
    supported("Fulgent Distraction");
    supported("Imprisoned in the Moon");
    // It leaves the battlefield: it was attached immediately before, and its ability
    // sacrifices the Bears.
    let mut t = TestGame::new(2);
    let (exo, _) = exoskeleton_on_bears(&mut t);
    t.lands(P1, "Mountain", 2);
    let shatter = t.hand(P1, "Shatter");
    t.cast(P1, shatter).target(exo).go();
    t.resolve();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grafted Exoskeleton"));
    assert!(
        t.in_graveyard(P0, "Grizzly Bears"),
        "the Exoskeleton's ability triggered as it left and sacrificed the Bears"
    );
    // A spell unattaches it.
    let mut t = TestGame::new(2);
    let (exo, bears) = exoskeleton_on_bears(&mut t);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Plains", 3);
    let fulgent = t.hand(P1, "Fulgent Distraction");
    t.cast(P1, fulgent)
        .targets(&[Entity::Object(bears), Entity::Object(giant)])
        .go();
    t.resolve();
    assert_eq!(t.obj_now(exo).attached_to, None);
    assert_eq!(triggers_of(&t, "Grafted Exoskeleton"), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(exo));
    // The equipped creature stops being a creature: the Equipment becomes unattached (a
    // state-based action) and the land the Bears became is sacrificed.
    let mut t = TestGame::new(2);
    let (exo, bears) = exoskeleton_on_bears(&mut t);
    t.lands(P0, "Island", 3);
    let moon = t.hand(P0, "Imprisoned in the Moon");
    t.cast(P0, moon).target(bears).go();
    t.resolve();
    assert_eq!(t.obj_now(exo).attached_to, None);
    assert_eq!(triggers_of(&t, "Grafted Exoskeleton"), 1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // The equipped creature leaves the battlefield: the ability triggers and does
    // nothing (nothing else is sacrificed).
    let mut t = TestGame::new(2);
    let (exo, bears) = exoskeleton_on_bears(&mut t);
    let other = t.battlefield(P0, "Hill Giant");
    t.g.destroy(bears, None);
    t.settle();
    assert_eq!(t.obj_now(exo).attached_to, None);
    assert_eq!(triggers_of(&t, "Grafted Exoskeleton"), 1);
    t.resolve_all();
    assert!(t.on_battlefield(exo) && t.on_battlefield(other));
    // (The ruling's last case, an Exoskeleton that stops being an Equipment, needs an
    // effect that removes the subtype but not the ability; no supported card has one.)
}

#[test]
fn an_unattach_trigger_cant_sacrifice_a_permanent_its_controller_no_longer_controls() {
    cr!("603.10c", "701.21a");
    ruling!(
        "Grafted Exoskeleton",
        "but you don't control the permanent it became unattached from at the time that ability resolves"
    );
    let mut t = TestGame::new(2);
    let exo = t.battlefield(P0, "Grafted Exoskeleton");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let mine = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Island", 4);
    let cm = t.hand(P0, "Control Magic");
    t.cast(P0, cm).target(theirs).go();
    t.resolve();
    t.g.attach(exo, Entity::Object(theirs));
    t.g.recompute();
    // The Control Magic goes; the Exoskeleton stays on the Bears, now P1's again.
    t.g.destroy(t.g.current(cm), None);
    t.settle();
    assert_eq!(t.obj_now(theirs).controller, P1);
    assert_eq!(t.obj_now(exo).attached_to, Some(Entity::Object(theirs)));
    // Moving it to P0's own creature unattaches it from the Bears: P0 can't sacrifice
    // P1's Bears.
    t.lands(P0, "Wastes", 2);
    t.activate(P0, exo, 0, &[Entity::Object(mine)])
        .expect("equip");
    t.resolve_all();
    assert_eq!(t.obj_now(exo).attached_to, Some(Entity::Object(mine)));
    assert!(t.on_battlefield(theirs), "P0 couldn't sacrifice P1's Bears");
}

#[test]
fn a_permanent_sacrificed_sees_its_own_sacrifice_but_the_legend_rule_isnt_one() {
    cr!("603.10a", "704.5j");
    ruling!("Mayhem Devil", "If you sacrifice Mayhem Devil, its ability triggers.");
    ruling!(
        "Mayhem Devil",
        "A legendary permanent that is put into a graveyard because of the \"legend rule\" isn't sacrificed."
    );
    let mut t = TestGame::new(2);
    let devil = t.battlefield(P0, "Mayhem Devil");
    let seer = t.battlefield(P0, "Viscera Seer");
    t.answer_choose(P0, &[Entity::Object(devil)]);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.activate(P0, seer, 0, &[]).expect("activate Viscera Seer");
    t.settle();
    assert!(t.in_graveyard(P0, "Mayhem Devil"));
    assert_eq!(triggers_of(&t, "Mayhem Devil"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // The legend rule puts a legendary permanent into the graveyard without sacrificing
    // it: the Devil doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mayhem Devil");
    t.battlefield(P0, "Isamaru, Hound of Konda");
    t.enter(P0, "Isamaru, Hound of Konda");
    t.settle();
    assert_eq!(controlled_named(&t, P0, "Isamaru, Hound of Konda"), 1);
    assert_eq!(triggers_of(&t, "Mayhem Devil"), 0);
}

#[test]
fn cards_leaving_a_graveyard_together_trigger_a_one_or_more_ability_once() {
    cr!("603.10a", "603.2c");
    ruling!(
        "Desecrated Tomb",
        "You create one Bat token each time Desecrated Tomb’s ability triggers, no matter how many cards left your graveyard."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Desecrated Tomb");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Lightning Bolt");
    let relic = t.battlefield(P0, "Relic of Progenitus");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, relic, 1, &[]).expect("activate Relic of Progenitus");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0, "the Relic was exiled as a cost");
    assert_eq!(tokens_named(&t, P0, "Bat"), 1);
    // Only a noncreature card leaving: no Bat. Creature cards one at a time: one each.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Desecrated Tomb");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Hill Giant");
    t.g.exile_object(bolt, None);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Bat"), 0);
    t.g.exile_object(a, None);
    t.resolve_all();
    t.g.exile_object(b, None);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Bat"), 2);
}

#[test]
fn losing_control_triggers_for_the_player_who_lost_it_each_way() {
    cr!("603.10d", "603.3a");
    supported("Khârn the Betrayer");
    let mut t = TestGame::new(2);
    let kharn = t.battlefield(P1, "Khârn the Betrayer");
    t.lands(P0, "Mountain", 3);
    let treason = t.hand(P0, "Act of Treason");
    let (h0, h1) = (t.hand_size(P0) - 1, t.hand_size(P1));
    t.cast(P0, treason).target(kharn).go();
    t.resolve();
    assert_eq!(t.obj_now(kharn).controller, P0);
    assert_eq!(trigger_controllers(&t, "Khârn the Betrayer"), vec![P1]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), h1 + 2, "P1 lost control of Khârn and draws");
    assert_eq!(t.hand_size(P0), h0);
    // At the end of the turn the control effect ends: now P0 loses control of it.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(kharn).controller, P1);
    assert_eq!(t.hand_size(P0), h0 + 2, "P0 lost control of Khârn and drew");
}

#[test]
fn leaving_the_battlefield_is_losing_control_of_it() {
    cr!("603.10d", "603.10a");
    ruling!(
        "Khârn the Betrayer",
        "You lose control of Khârn the Betrayer when it leaves the battlefield or another player gains control of it."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Khârn the Betrayer");
    let h1 = t.hand_size(P1);
    wrath(&mut t);
    assert_eq!(trigger_controllers(&t, "Khârn the Betrayer"), vec![P1]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), h1 + 2);
}

#[test]
fn a_creature_dying_as_the_watcher_is_destroyed_still_triggers_it() {
    cr!("603.10a");
    ruling!(
        "Revel in Riches",
        "If an opponent's creature dies at the same time that Revel in Riches is destroyed, you'll get a Treasure."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Revel in Riches");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    vengeance(&mut t);
    assert_eq!(triggers_of(&t, "Revel in Riches"), 1, "only the opponent's creature");
    t.resolve_all();
    assert_eq!(tokens_named(&t, P0, "Treasure"), 1);
}

#[test]
fn phasing_out_isnt_leaving_the_battlefield() {
    cr!("603.6c", "702.26d");
    supported("Slip Out the Back");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let hunter = t.hand(P0, "Fiend Hunter");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, hunter).go();
    t.resolve();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    let hunter = t.g.current(hunter);
    t.lands(P0, "Island", 1);
    let slip = t.hand(P0, "Slip Out the Back");
    t.cast(P0, slip).target(hunter).go();
    t.resolve_all();
    assert!(t.obj_now(hunter).phased_out);
    assert_eq!(t.stack_len(), 0, "nothing triggered");
    assert_eq!(t.zone(bears), Zone::Exile, "the Bears stay exiled");
    // It phases in during P0's next untap step: that isn't entering either.
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert!(!t.obj_now(hunter).phased_out);
    assert_eq!(triggers_of(&t, "Fiend Hunter"), 0);
    assert_eq!(t.zone(bears), Zone::Exile);
}

#[test]
fn a_leaves_ability_resolving_before_the_enters_ability_returns_nothing() {
    cr!("603.6c", "603.10a");
    ruling!(
        "Fiend Hunter",
        "If Fiend Hunter leaves the battlefield before its first ability has resolved, its second ability will trigger and do nothing."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Plains", 3);
    let hunter = t.hand(P0, "Fiend Hunter");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, hunter).go();
    t.resolve();
    t.settle();
    assert_eq!(triggers_of(&t, "Fiend Hunter"), 1);
    // In response, P1 kills the Hunter: its leaves ability triggers and resolves first.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let hunter = t.g.current(hunter);
    t.cast(P1, bolt).target(hunter).go();
    t.resolve();
    assert_eq!(triggers_of(&t, "Fiend Hunter"), 2);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile, "exiled indefinitely");
}

#[test]
fn a_dies_ability_uses_the_power_the_creature_last_had() {
    cr!("603.10a", "608.2h");
    ruling!(
        "Bottle Golems",
        "Use Bottle Golems' power when it was on the battlefield to determine the amount of life you gain."
    );
    let mut t = TestGame::new(2);
    let golems = t.battlefield(P0, "Bottle Golems");
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(golems).go();
    t.resolve();
    let power = t.pt(golems).0;
    assert_eq!(power, 3 + 3);
    t.g.destroy(golems, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 20 + power);
}

#[test]
fn an_opponent_gaining_control_triggers_for_the_player_it_was_taken_from() {
    cr!("603.10d", "603.3a");
    ruling!(
        "Zidane, Tantalus Thief",
        "If an opponent gains control of Zidane from you, Zidane's last ability will trigger and you'll create a Treasure token."
    );
    supported("Zidane, Tantalus Thief");
    let mut t = TestGame::new(2);
    let zidane = t.battlefield(P1, "Zidane, Tantalus Thief");
    t.lands(P0, "Mountain", 3);
    let treason = t.hand(P0, "Act of Treason");
    t.cast(P0, treason).target(zidane).go();
    t.resolve();
    assert_eq!(t.obj_now(zidane).controller, P0);
    assert_eq!(trigger_controllers(&t, "Zidane, Tantalus Thief"), vec![P1]);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P1, "Treasure"), 1);
    assert_eq!(tokens_named(&t, P0, "Treasure"), 0);
}

#[test]
fn several_permanents_taken_at_once_trigger_it_for_each() {
    cr!("603.10d", "603.2c");
    ruling!(
        "Zidane, Tantalus Thief",
        "If an opponent gains control of Zidane and one or more other permanents from you simultaneously, Zidane's last ability will trigger for each of those permanents, including itself."
    );
    supported("Insurrection");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Zidane, Tantalus Thief");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 8);
    let ins = t.hand(P0, "Insurrection");
    t.cast(P0, ins).go();
    t.resolve();
    assert_eq!(trigger_controllers(&t, "Zidane, Tantalus Thief"), vec![P1, P1]);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P1, "Treasure"), 2);
}

#[test]
fn a_permanent_phasing_out_sees_itself_phase_out() {
    cr!("603.10b", "702.26a");
    supported("Ertai's Familiar");
    let mut t = TestGame::new(2);
    let familiar = t.battlefield(P0, "Ertai's Familiar");
    t.advance_to(P1, Step::PrecombatMain);
    assert_eq!(t.graveyard_size(P0), 0);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(familiar).phased_out);
    t.settle();
    assert_eq!(triggers_of(&t, "Ertai's Familiar"), 1);
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 3);
}

#[test]
fn creature_cards_leaving_a_graveyard_together_trigger_a_per_card_ability_for_each() {
    cr!("603.10a", "603.2c");
    ruling!(
        "Syr Konrad, the Grim",
        "If one or more creatures die at the same time as Syr Konrad, its first ability triggers for each of those creatures."
    );
    supported("Syr Konrad, the Grim");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Syr Konrad, the Grim");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Lightning Bolt");
    t.graveyard(P1, "Craw Wurm");
    let relic = t.battlefield(P0, "Relic of Progenitus");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, relic, 1, &[]).expect("activate Relic of Progenitus");
    t.resolve();
    // Two creature cards left P0's graveyard (P1's isn't "your graveyard").
    assert_eq!(triggers_of(&t, "Syr Konrad, the Grim"), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Dying with two other creatures: it triggers for each of them.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Syr Konrad, the Grim");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    wrath(&mut t);
    assert_eq!(triggers_of(&t, "Syr Konrad, the Grim"), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn an_exchange_of_control_is_looked_back_at_for_both_permanents() {
    cr!("603.10d", "701.12a");
    supported("Switcheroo");
    let mut t = TestGame::new(2);
    let zidane = t.battlefield(P1, "Zidane, Tantalus Thief");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 5);
    let s = t.hand(P0, "Switcheroo");
    t.cast(P0, s)
        .targets(&[Entity::Object(bears), Entity::Object(zidane)])
        .go();
    t.resolve();
    assert_eq!(t.obj_now(zidane).controller, P0);
    assert_eq!(t.obj_now(bears).controller, P1);
    // Immediately before the exchange P1 controlled Zidane: P1's opponent gained control
    // of Zidane from P1 (a trigger for P1); P1 gaining the Bears from P0 isn't an
    // opponent of Zidane's controller gaining a permanent from them, though P0 controls
    // Zidane afterwards.
    assert_eq!(trigger_controllers(&t, "Zidane, Tantalus Thief"), vec![P1]);
    t.resolve_all();
    assert_eq!(tokens_named(&t, P1, "Treasure"), 1);
    assert_eq!(tokens_named(&t, P0, "Treasure"), 0);
}
