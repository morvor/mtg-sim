//! CR 702.122 Crew.

use crate::common_k702_111_124::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Activates the crew ability (the first activated ability) of `vehicle`, tapping
/// `crew` (if given; otherwise the engine's default choice).
fn crew(t: &mut TestGame, p: PlayerId, vehicle: ObjectId, crew: &[ObjectId]) -> bool {
    if !crew.is_empty() {
        let e: Vec<Entity> = crew.iter().map(|c| Entity::Object(*c)).collect();
        t.answer_choose(p, &e);
    }
    let r = t.activate(p, vehicle, 0, &[]);
    t.clear_answers();
    r.is_ok()
}

fn is_creature(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).is(CardType::Creature)
}

#[test]
fn crew_makes_the_vehicle_an_artifact_creature_until_end_of_turn() {
    cr!("702.122", "702.122a");
    ruling!(
        "Smuggler's Copter",
        "Any untapped creature you control can be tapped to pay a crew cost, even one that just came under your control."
    );
    ruling!(
        "Smuggler's Copter",
        "Each Vehicle is printed with a power and toughness, but it's not a creature. If it becomes a creature (most likely through its crew ability), it will have that power and toughness."
    );
    assert_supported_card("Smuggler's Copter");
    let mut t = TestGame::new(2);
    // Smuggler's Copter: 3/3 flying Vehicle, crew 1.
    let copter = t.battlefield(P0, "Smuggler's Copter");
    assert!(!is_creature(&t, copter));
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, copter, &[bears]));
    // The cost was paid: the summoning-sick creature is tapped.
    assert!(t.obj(bears).tapped);
    assert!(!is_creature(&t, copter), "not until the ability resolves");
    t.resolve();
    assert!(is_creature(&t, copter));
    assert!(t.obj_now(copter).is(CardType::Artifact));
    assert!(t.obj_now(copter).chars.has_subtype("Vehicle"));
    assert_eq!(t.pt(copter), (3, 3));
    // It can attack.
    declare_attack(&mut t, &[(copter, Entity::Player(P1))]);
    t.answer_yes(P0, false);
    finish_combat(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 17);
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(!is_creature(&t, copter));
}

#[test]
fn a_crewed_vehicle_is_an_ordinary_artifact_creature() {
    cr!("702.122a");
    ruling!(
        "Smuggler's Copter",
        "Vehicle is an artifact type, not a creature type. A Vehicle that's crewed won't normally have any creature type."
    );
    ruling!(
        "Smuggler's Copter",
        "When a Vehicle becomes a creature, that doesn't count as having a creature enter the battlefield."
    );
    ruling!(
        "Smuggler's Copter",
        "It can't attack unless you've controlled it continuously since your turn began"
    );
    ruling!(
        "Smuggler's Copter",
        "Creatures that crew a Vehicle aren't attached to it or related in any other way."
    );
    let mut t = TestGame::new(2);
    // Soul Warden: "Whenever another creature enters, you gain 1 life."
    t.battlefield(P0, "Soul Warden");
    let copter = t.battlefield_sick(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    let o = t.obj_now(copter);
    assert!(!o
        .chars
        .subtypes
        .iter()
        .any(|s| mtg_engine::types::is_creature_type(s)));
    // It came under its controller's control this turn: it can't attack.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!mtg_engine::combat::attack_options(&t.g)
        .iter()
        .any(|(c, _)| *c == copter));
    // Destroying the Vehicle doesn't affect the creature that crewed it.
    t.g.destroy(copter, None);
    t.settle();
    assert!(t.on_battlefield(bears));
}

#[test]
fn a_copy_of_a_crewed_vehicle_isnt_a_creature() {
    cr!("702.122a");
    ruling!(
        "Smuggler's Copter",
        "If a permanent becomes a copy of a Vehicle, the copy won't be a creature, even if the Vehicle it's copying has become an artifact creature."
    );
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, copter, &[bears]));
    t.resolve_all();
    // Clone may enter as a copy of any creature: the crewed Copter.
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.cast(P0, clone).go();
    t.answer_choose(P0, &[Entity::Object(copter)]);
    t.resolve_all();
    let copy = t.g.current(clone);
    assert_eq!(t.obj(copy).chars.name, "Smuggler's Copter");
    assert!(!is_creature(&t, copy));
    assert!(t.obj(copy).is(CardType::Artifact));
}

#[test]
fn crew_needs_total_power_n_or_greater_from_other_untapped_creatures() {
    cr!("702.122a");
    let mut t = TestGame::new(2);
    // Mobilizer Mech: crew 3.
    let mech = t.battlefield(P0, "Mobilizer Mech");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // 1 + 2 = 3, but not with a tapped creature, or an opponent's.
    t.g.tap(elves);
    t.battlefield(P1, "Hill Giant");
    assert!(!crew(&mut t, P0, mech, &[bears]));
    assert!(!t.obj(bears).tapped);
    t.g.untap(elves);
    // Choosing too little power isn't allowed: the engine taps enough instead.
    assert!(crew(&mut t, P0, mech, &[elves]));
    assert!(t.obj(elves).tapped && t.obj(bears).tapped);
    t.resolve_all();
    assert!(is_creature(&t, mech));
    // A crewed Vehicle can't crew itself, but it can crew another Vehicle.
    let copter = t.battlefield(P0, "Smuggler's Copter");
    assert!(crew(&mut t, P0, copter, &[mech]));
    assert!(t.obj(mech).tapped);
}

#[test]
fn you_may_tap_more_creatures_than_necessary() {
    cr!("702.122a");
    ruling!(
        "Smuggler's Copter",
        "You may tap more creatures than necessary to activate a crew ability."
    );
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    assert!(crew(&mut t, P0, copter, &[a, b]));
    assert!(t.obj(a).tapped && t.obj(b).tapped);
}

#[test]
fn crewing_an_artifact_creature_again_changes_nothing() {
    cr!("702.122a");
    ruling!(
        "Smuggler's Copter",
        "You may activate a crew ability of a Vehicle even if it's already an artifact creature."
    );
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, copter, &[a]));
    t.resolve();
    assert!(crew(&mut t, P0, copter, &[b]));
    t.resolve();
    assert_eq!(t.pt(copter), (3, 3));
}

#[test]
fn a_vehicle_can_be_crewed_at_instant_speed_to_block() {
    cr!("702.122a");
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P1, "Smuggler's Copter");
    let crewer = t.battlefield(P1, "Grizzly Bears");
    let attacker = t.battlefield(P0, "Hill Giant");
    declare_attack(&mut t, &[(attacker, Entity::Player(P1))]);
    t.g.turn.priority = Some(P1);
    assert!(crew(&mut t, P1, copter, &[crewer]));
    t.resolve();
    assert!(is_creature(&t, copter));
    t.answer_yes(P1, false);
    finish_combat(&mut t, P1, &[(copter, attacker)]);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P0, "Hill Giant"));
}

#[test]
fn a_creature_crews_a_vehicle_when_tapped_to_pay_its_crew_cost() {
    cr!("702.122b");
    ruling!(
        "Speedway Fanatic",
        "Speedway Fanatic's triggered ability triggers when it becomes tapped to activate a crew ability. Even if Speedway Fanatic leaves the battlefield before its triggered ability resolves, that Vehicle still gains haste until end of turn."
    );
    assert_supported_card("Speedway Fanatic");
    let mut t = TestGame::new(2);
    // Speedway Fanatic: "Whenever this creature crews a Vehicle, that Vehicle gains haste
    // until end of turn."
    let copter = t.battlefield_sick(P0, "Smuggler's Copter");
    let fanatic = t.battlefield(P0, "Speedway Fanatic");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, copter, &[fanatic]));
    t.settle();
    // The trigger is above the crew ability.
    assert_eq!(on_stack(&t, "crews a Vehicle"), 1);
    // The fanatic leaves the battlefield before its trigger resolves.
    t.g.destroy(fanatic, None);
    t.resolve_all();
    assert!(is_creature(&t, copter));
    assert!(has(&t, copter, KeywordKind::Haste));
    // Another creature crewing doesn't trigger a Speedway Fanatic that didn't crew.
    let idle = t.battlefield(P0, "Speedway Fanatic");
    let other = t.battlefield(P0, "Smuggler's Copter");
    assert!(crew(&mut t, P0, other, &[bears]));
    assert!(t.obj(bears).tapped && !t.obj(idle).tapped);
    t.settle();
    assert_eq!(on_stack(&t, "crews a Vehicle"), 0);
    t.resolve_all();
    assert!(is_creature(&t, other));
    assert!(!has(&t, other, KeywordKind::Haste));
}

#[test]
fn abilities_can_refer_to_the_creatures_that_crewed_a_vehicle() {
    cr!("702.122c");
    assert_supported_card("Luxurious Locomotive");
    let mut t = TestGame::new(2);
    // Luxurious Locomotive: 6/5; "Whenever this Vehicle attacks, create a Treasure token for
    // each creature that crewed it this turn." Crew 1. Activate only once each turn.
    let loco = t.battlefield(P0, "Luxurious Locomotive");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    // A creature crewing another Vehicle doesn't count.
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let c = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, copter, &[c]));
    t.resolve();
    assert!(crew(&mut t, P0, loco, &[a, b]));
    t.resolve();
    declare_attack(&mut t, &[(loco, Entity::Player(P1))]);
    t.resolve_all();
    let treasures = tokens_of(&t, P0)
        .into_iter()
        .filter(|x| t.g.obj(*x).chars.has_subtype("Treasure"))
        .count();
    assert_eq!(treasures, 2);
}

#[test]
fn creatures_that_crewed_it_and_left_the_battlefield_still_count() {
    cr!("702.122c");
    ruling!(
        "Luxurious Locomotive",
        "Luxurious Locomotive's triggered ability counts each creature that crewed it this turn, including creatures that are no longer on the battlefield as the ability resolves."
    );
    let mut t = TestGame::new(2);
    let loco = t.battlefield(P0, "Luxurious Locomotive");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, loco, &[a, b]));
    t.resolve();
    // One of them is destroyed before the Locomotive attacks.
    t.g.destroy(a, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    declare_attack(&mut t, &[(loco, Entity::Player(P1))]);
    t.resolve_all();
    let treasures = tokens_of(&t, P0)
        .into_iter()
        .filter(|x| t.g.obj(*x).chars.has_subtype("Treasure"))
        .count();
    assert_eq!(treasures, 2);
}

/// A Vehicle with "Whenever ~ attacks, if an Assassin crewed it this turn, draw a card."
/// (Adrestia's second ability, without its last sentence) and crew 1.
fn assassin_rig(t: &mut TestGame) -> ObjectId {
    let def = crate::k702_001_010_common::custom_card(
        "Assassin Rig",
        "Artifact — Vehicle",
        "{3}",
        Some((4, 4)),
        "Whenever ~ attacks, if an Assassin crewed it this turn, draw a card.\nCrew 1",
    );
    t.custom(P0, def, mtg_engine::object::Zone::Battlefield)
}

#[test]
fn an_assassin_that_crewed_it_counts_even_after_it_leaves() {
    cr!("702.122c");
    ruling!(
        "Adrestia",
        "Adrestia’s second ability will trigger as long as it was crewed by an Assassin this turn, even if that creature has left the battlefield or stopped being an Assassin since."
    );
    let mut t = TestGame::new(2);
    let rig = assassin_rig(&mut t);
    // Royal Assassin: a 1/1 Human Assassin.
    let assassin = t.battlefield(P0, "Royal Assassin");
    assert!(crew(&mut t, P0, rig, &[assassin]));
    t.resolve();
    t.g.destroy(assassin, None);
    t.settle();
    assert!(t.in_graveyard(P0, "Royal Assassin"));
    let hand = t.hand_size(P0);
    declare_attack(&mut t, &[(rig, Entity::Player(P1))]);
    assert_eq!(on_stack(&t, "an Assassin crewed it"), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn a_creature_that_becomes_an_assassin_after_crewing_doesnt_count() {
    cr!("702.122c");
    ruling!(
        "Adrestia",
        "Adrestia’s second ability will trigger only if it was crewed this turn by a creature that was an Assassin when it was tapped to pay the cost of Adrestia’s crew ability."
    );
    let mut t = TestGame::new(2);
    let rig = assassin_rig(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, rig, &[bears]));
    t.resolve();
    // The Bears become an Assassin after crewing it.
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(bears)]];
    t.g.exec(
        &mtg_engine::ability::Effect::Modify {
            what: mtg_engine::ability::Sel::Target(0),
            mods: vec![mtg_engine::ability::Modification::AddSubtypes(vec![
                Subtype::new("Assassin"),
            ])],
            duration: mtg_engine::ability::Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    assert!(t.obj_now(bears).chars.has_subtype("Assassin"));
    let hand = t.hand_size(P0);
    declare_attack(&mut t, &[(rig, Entity::Player(P1))]);
    // The intervening "if" is false: it doesn't trigger.
    assert_eq!(on_stack(&t, "an Assassin crewed it"), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn crew_that_can_be_activated_only_once_each_turn() {
    cr!("702.122a");
    let mut t = TestGame::new(2);
    let loco = t.battlefield(P0, "Luxurious Locomotive");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, loco, &[a]));
    t.resolve();
    assert!(!crew(&mut t, P0, loco, &[b]));
    assert!(!t.obj(b).tapped);
    // Next turn it can be crewed again.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(crew(&mut t, P0, loco, &[b]));
}

#[test]
fn getaway_car_returns_a_creature_that_crewed_it() {
    cr!("702.122c");
    assert_supported_card("Getaway Car");
    let mut t = TestGame::new(2);
    // Getaway Car: 4/3 haste, "Whenever this Vehicle attacks or blocks, return up to one
    // target creature that crewed it this turn to its owner's hand." Crew 1.
    let car = t.battlefield(P0, "Getaway Car");
    let crewer = t.battlefield(P0, "Grizzly Bears");
    let bystander = t.battlefield(P0, "Llanowar Elves");
    assert!(crew(&mut t, P0, car, &[crewer]));
    t.resolve();
    // Only the creature that crewed it is a legal target.
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(crewer)]);
    declare_attack(&mut t, &[(car, Entity::Player(P1))]);
    t.resolve_all();
    let offered: Vec<Entity> = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            decision::Decision::ChooseTargets { candidates, .. } => Some(candidates.clone()),
            _ => None,
        })
        .unwrap_or_default();
    assert!(offered.contains(&Entity::Object(crewer)));
    assert!(!offered.contains(&Entity::Object(bystander)));
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn a_creature_that_cant_crew_vehicles_cant_pay_crew_costs() {
    cr!("702.122d");
    assert_supported_card("Revoke Privileges");
    let mut t = TestGame::new(2);
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Revoke Privileges: "Enchanted creature can't attack, block, or crew Vehicles."
    t.lands(P1, "Plains", 3);
    let aura = t.hand(P1, "Revoke Privileges");
    t.g.turn.active = P1;
    t.cast(P1, aura).target(bears).go();
    t.resolve_all();
    t.g.turn.active = P0;
    assert!(!crew(&mut t, P0, copter, &[bears]));
    assert!(!t.obj(bears).tapped);
    // Another creature can.
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert!(crew(&mut t, P0, copter, &[bears, elves]));
    assert!(!t.obj(bears).tapped && t.obj(elves).tapped);
}

#[test]
fn an_arrested_vehicle_cant_activate_crew_or_crew_other_vehicles() {
    cr!("702.122d");
    assert_supported_card("Intercessor's Arrest");
    let mut t = TestGame::new(2);
    // Intercessor's Arrest: "Enchanted permanent can't attack, block, or crew Vehicles.
    // Its activated abilities can't be activated unless they're mana abilities."
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let mech = t.battlefield(P0, "Mobilizer Mech");
    let giant = t.battlefield(P0, "Hill Giant");
    let aura = t.hand(P1, "Intercessor's Arrest");
    t.lands(P1, "Plains", 3);
    t.g.turn.active = P1;
    t.cast(P1, aura).target(mech).go();
    t.resolve_all();
    t.g.turn.active = P0;
    // Its crew ability can't be activated.
    assert!(!crew(&mut t, P0, mech, &[giant]));
    // Made a creature by another effect, it still can't crew the copter.
    let mut ctx = mtg_engine::eval::Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(mech)]];
    t.g.exec(
        &mtg_engine::ability::Effect::Modify {
            what: mtg_engine::ability::Sel::Target(0),
            mods: vec![mtg_engine::ability::Modification::AddTypes(vec![
                CardType::Creature,
            ])],
            duration: mtg_engine::ability::Duration::EndOfTurn,
        },
        &mut ctx,
    );
    t.g.recompute();
    assert!(is_creature(&t, mech));
    t.g.tap(giant);
    assert!(!crew(&mut t, P0, copter, &[mech]));
    assert!(!t.obj(mech).tapped);
}

#[test]
fn some_creatures_crew_vehicles_with_more_power() {
    cr!("702.122a");
    assert_supported_card("Giant Ox");
    let mut t = TestGame::new(2);
    let mech = t.battlefield(P0, "Mobilizer Mech");
    // Giant Ox: 0/6, "This creature crews Vehicles using its toughness rather than its
    // power."
    let ox = t.battlefield(P0, "Giant Ox");
    assert!(crew(&mut t, P0, mech, &[ox]));
    t.resolve_all();
    assert!(is_creature(&t, mech));
    // "Crews Vehicles as though its power were 2 greater": a 1/1 crews 3.
    let mut t = TestGame::new(2);
    let mech = t.battlefield(P0, "Mobilizer Mech");
    let pilot = t.battlefield(P0, "Experimental Pilot");
    assert!(crew(&mut t, P0, mech, &[pilot]));
    t.resolve_all();
    assert!(is_creature(&t, mech));
}

#[test]
fn becomes_crewed_triggers_when_the_crew_ability_resolves() {
    cr!("702.122e");
    ruling!(
        "Mobilizer Mech",
        "Mobilizer Mech's second ability triggers whenever its crew ability resolves, even if it is already a creature at that time."
    );
    ruling!(
        "Mobilizer Mech",
        "That ability doesn't count as \"crewing\" a Vehicle for any ability that would trigger off of a Vehicle becoming crewed."
    );
    assert_supported_card("Mobilizer Mech");
    let mut t = TestGame::new(2);
    // Mobilizer Mech: "Whenever this Vehicle becomes crewed, up to one other target
    // Vehicle you control becomes an artifact creature until end of turn." Crew 3.
    let mech = t.battlefield(P0, "Mobilizer Mech");
    let copter = t.battlefield(P0, "Smuggler's Copter");
    let giant = t.battlefield(P0, "Hill Giant");
    let other_mech = t.battlefield(P0, "Mobilizer Mech");
    assert!(crew(&mut t, P0, mech, &[giant]));
    t.settle();
    assert_eq!(on_stack(&t, "becomes crewed"), 0, "not on activation");
    t.answer_targets(P0, &[Entity::Object(copter)]);
    t.resolve();
    assert!(is_creature(&t, mech));
    assert_eq!(on_stack(&t, "becomes crewed"), 1);
    t.resolve_all();
    assert!(is_creature(&t, copter));
    // The copter becoming a creature this way isn't it being crewed. Crewing an
    // already-crewed Vehicle triggers again.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    assert!(crew(&mut t, P0, mech, &[bears, elves]));
    t.answer_targets(P0, &[Entity::Object(other_mech)]);
    t.resolve();
    assert_eq!(on_stack(&t, "becomes crewed"), 1);
    t.resolve_all();
    assert!(is_creature(&t, other_mech));
}

#[test]
fn a_countered_crew_ability_doesnt_make_it_crewed() {
    cr!("702.122e");
    let mut t = TestGame::new(2);
    let mech = t.battlefield(P0, "Mobilizer Mech");
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(crew(&mut t, P0, mech, &[giant]));
    let ability = *t.g.stack.last().unwrap();
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    t.cast(P1, stifle).target(ability).go();
    t.resolve_all();
    assert!(!is_creature(&t, mech));
    assert_eq!(on_stack(&t, "becomes crewed"), 0);
    // The giant still crewed it (CR 702.122b–c).
    assert!(t.obj(giant).tapped);
    assert!(mtg_engine::kw::crew::crewed_this_turn(&t.g, mech, giant));
}

#[test]
fn an_intervening_if_counts_only_the_creatures_that_crewed_it_that_time() {
    cr!("702.122e");
    assert_supported_card("Mighty Servant of Leuk-o");
    let mut t = TestGame::new(2);
    // Mighty Servant of Leuk-o: "Whenever this Vehicle becomes crewed for the first time
    // each turn, if it was crewed by exactly two creatures, it gains 'Whenever this
    // creature deals combat damage to a player, draw two cards' until end of turn."
    // Crew 4.
    let servant = t.battlefield(P0, "Mighty Servant of Leuk-o");
    let a = t.battlefield(P0, "Hill Giant");
    let b = t.battlefield(P0, "Llanowar Elves");
    let c = t.battlefield(P0, "Grizzly Bears");
    let abilities = t.obj(servant).chars.abilities.len();
    assert!(crew(&mut t, P0, servant, &[a, b, c]));
    t.resolve_all();
    assert_eq!(t.obj_now(servant).chars.abilities.len(), abilities);
    // Next turn, exactly two.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(crew(&mut t, P0, servant, &[a, c]));
    t.resolve_all();
    assert_eq!(t.obj_now(servant).chars.abilities.len(), abilities + 1);
}

#[test]
fn only_the_creatures_tapped_for_the_crew_ability_that_resolved_count() {
    cr!("702.122e");
    ruling!(
        "Mighty Servant of Leuk-o",
        "it will trigger only if exactly two creatures were tapped to pay the crew cost of the ability that caused it to become crewed for the first time that turn"
    );
    // Crewed by one creature, then (in response) by two: the second ability resolves
    // first, and exactly two creatures were tapped for it, though three crewed it.
    let mut t = TestGame::new(2);
    let servant = t.battlefield(P0, "Mighty Servant of Leuk-o");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let abilities = t.obj(servant).chars.abilities.len();
    assert!(crew(&mut t, P0, servant, &[wurm]));
    assert!(crew(&mut t, P0, servant, &[giant, bears]));
    t.resolve();
    t.settle();
    assert_eq!(on_stack(&t, "becomes crewed"), 1);
    t.resolve_all();
    assert_eq!(t.obj_now(servant).chars.abilities.len(), abilities + 1);
    // The other way around: the ability paid for by one creature makes it crewed first,
    // and the later one doesn't trigger it.
    let mut t = TestGame::new(2);
    let servant = t.battlefield(P0, "Mighty Servant of Leuk-o");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    assert!(crew(&mut t, P0, servant, &[giant, bears]));
    assert!(crew(&mut t, P0, servant, &[wurm]));
    t.resolve_all();
    assert!(is_creature(&t, servant));
    assert_eq!(t.obj_now(servant).chars.abilities.len(), abilities);
}
