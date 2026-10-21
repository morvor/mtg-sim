//! Rulings batch P212 — Equipment (CR 301.5, 702.6): "whenever equipped creature dies"
//! and similar triggers when the Equipment and the creature leave together (CR 603.10a),
//! simultaneous lethal damage (CR 704.3), and last known information of the equipped
//! creature (CR 608.2h, 113.7a).

use crate::r_p209_common::cast_spell;
use crate::r_p210_common::deal;
use crate::r_p212_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s05_common::move_to;
use crate::r_s06_common::*;
use crate::r_s30_common::set_life;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn equipped_creature_dies_triggers_when_the_equipment_leaves_at_the_same_time() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Heirloom Blade",
        "If Heirloom Blade leaves the battlefield at the same time that the equipped creature dies, its triggered ability triggers."
    );
    ruling!(
        "Sylvok Lifestaff",
        "If Sylvok Lifestaff and the equipped creature are put into their owners’ graveyards at the same time (due to Planar Cleansing, perhaps), the triggered ability will still trigger. You’ll gain 3 life."
    );
    supported("Heirloom Blade");
    supported("Sylvok Lifestaff");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    attach_new(&mut t, P0, "Heirloom Blade", bears);
    attach_new(&mut t, P0, "Sylvok Lifestaff", lion);
    stack_library(&mut t, P0, &["Forest", "Runeclaw Bear"]);
    cast_spell(&mut t, P0, "Planar Cleansing", &[]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Heirloom Blade"));
    assert!(t.in_graveyard(P0, "Sylvok Lifestaff"));
    assert!(t.in_hand(P0, "Runeclaw Bear"));
    assert_eq!(t.life(P0), 23);
}

#[test]
fn heirloom_blade_finds_nothing_for_a_creature_with_no_creature_type() {
    cr!("205.3d", "603.10a");
    ruling!(
        "Heirloom Blade",
        "If the equipped creature has no creature type, no card can share a creature type with it."
    );
    // A creature with no creature types; the library holds a changeling (every creature
    // type) and a Bear.
    let typeless = custom_card("Typeless", "Creature", "{2}", Some((2, 2)), "");
    let mut t = TestGame::new(2);
    let c = t.custom(P0, typeless, Zone::Battlefield);
    attach_new(&mut t, P0, "Heirloom Blade", c);
    stack_library(&mut t, P0, &["Changeling Outcast", "Grizzly Bears"]);
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    destroy(&mut t, c);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert!(!t.in_hand(P0, "Changeling Outcast"));
}

#[test]
fn creatures_dealt_lethal_damage_together_die_together() {
    cr!("704.3", "704.5g", "603.10a");
    ruling!(
        "Bonehoard",
        "If lethal damage is dealt simultaneously to the equipped creature and another creature, both creatures are put into the graveyard at the same time. The equipped creature won't get an additional bonus from the other creature."
    );
    ruling!(
        "Blade of the Bloodchief",
        "If the equipped creature and another creature are dealt lethal damage at the same time, the equipped creature will be put into a graveyard before it would receive any counters."
    );
    supported("Bonehoard");
    supported("Blade of the Bloodchief");
    // Earthquake deals 3 damage to the Hill Giant equipped with Bonehoard (3/3, no
    // creature cards in graveyards) and to Llanowar Elves: if the Elves died first, the
    // Giant would be 4/4 and survive.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Bonehoard", giant);
    t.battlefield(P1, "Llanowar Elves");
    assert_eq!(t.pt(giant), (3, 3));
    cast_x(&mut t, P0, "Earthquake", 3);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    // The same with Blade of the Bloodchief: the Giant dies before any counter arrives.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let blade = attach_new(&mut t, P0, "Blade of the Bloodchief", giant);
    t.battlefield(P1, "Llanowar Elves");
    let wall = t.battlefield(P0, "Wall of Stone");
    cast_x(&mut t, P0, "Earthquake", 3);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    // The triggers resolved with no equipped creature: no counters anywhere.
    assert_eq!(attached_to(&t, blade), None);
    assert_eq!(t.counters(wall, counters::PLUS1), 0);
}

#[test]
fn blade_of_the_bloodchief_counters_go_to_the_creature_equipped_on_resolution() {
    cr!("603.10a", "608.2h", "122.1");
    ruling!(
        "Blade of the Bloodchief",
        "If the equipped creature itself is put into a graveyard, Blade of the Bloodchief's ability will trigger. However, it won't do anything when it resolves unless a spell or ability has caused it to become attached to another creature by that time."
    );
    ruling!(
        "Blade of the Bloodchief",
        "The +1/+1 counters are put on the equipped creature, not Blade of the Bloodchief. If Blade of the Bloodchief is moved to a different creature, the +1/+1 counters will stay where they are."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lion = t.battlefield(P0, "Silvercoat Lion");
    let blade = attach_new(&mut t, P0, "Blade of the Bloodchief", bears);
    destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 1);
    // Unattached as it resolves: nothing happens.
    t.resolve_all();
    assert_eq!(t.counters(lion, counters::PLUS1), 0);
    assert_eq!(t.counters(blade, counters::PLUS1), 0);
    // Another death; the Blade is attached to the Lion before the trigger resolves.
    let elves = t.battlefield(P1, "Llanowar Elves");
    destroy(&mut t, elves);
    assert_eq!(t.stack_len(), 1);
    move_eq(&mut t, blade, lion);
    t.resolve_all();
    assert_eq!(t.counters(lion, counters::PLUS1), 1);
    assert_eq!(t.counters(blade, counters::PLUS1), 0);
    // Moving the Blade away leaves the counter on the Lion.
    let giant = t.battlefield(P0, "Hill Giant");
    move_eq(&mut t, blade, giant);
    assert_eq!(t.counters(lion, counters::PLUS1), 1);
    assert_eq!(t.pt(lion), (3, 3));
}

#[test]
fn excalibur_ii_creature_dealt_lethal_damage_while_you_gain_life_dies_first() {
    cr!("510.2", "704.5g", "603.3");
    ruling!(
        "Excalibur II",
        "If the equipped creature is dealt lethal damage at the same time that you gain life, it will die before Excalibur II's first ability would resolve."
    );
    supported("Excalibur II");
    // Child of Night (2/1 lifelink) equipped with Excalibur II (no charge counters)
    // attacks and is blocked by Llanowar Elves: it gains life and is dealt 1 damage at
    // once. Had the counter arrived first, it would have been 3/2 and survived.
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    let hawk = t.battlefield(P0, "Child of Night");
    let ex = attach_new(&mut t, P0, "Excalibur II", hawk);
    let elves = t.battlefield(P1, "Llanowar Elves");
    attack_with(&mut t, &[(hawk, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(elves, hawk)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert!(t.in_graveyard(P0, "Child of Night"));
    assert_eq!(t.counters(ex, counters::CHARGE), 1);
}

#[test]
fn caduceus_indestructible_applies_before_state_based_actions() {
    cr!("510.2", "704.5g", "613.1f", "702.12b");
    ruling!(
        "Caduceus, Staff of Hermes",
        "If the equipped creature is dealt damage at the same time that you gain life (most likely because it dealt combat damage to a creature blocking or blocked by it) and your life total becomes 30 or greater as a result, the damage won’t be prevented. However, the equipped creature will have indestructible before state-based actions are checked, so it won’t be destroyed even if that damage would ordinarily be lethal."
    );
    supported("Caduceus, Staff of Hermes");
    let mut t = TestGame::new(2);
    t.set_step(P0, mtg_engine::turn::Step::BeginningOfCombat);
    set_life(&mut t, P0, 28);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Caduceus, Staff of Hermes", bears);
    let giant = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(giant, bears)]);
    assert_eq!(t.life(P0), 30);
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).damage, 3);
    assert_eq!(t.pt(bears), (7, 7));
}

#[test]
fn blazing_sunsteel_controller_loses_before_the_trigger_is_put_on_the_stack() {
    cr!("704.5a", "800.4a", "603.3");
    ruling!(
        "Blazing Sunsteel",
        "If your life total is brought to 0 or less at the same time that the equipped creature is dealt damage, you lose the game before Blazing Sunsteel's middle ability goes on the stack."
    );
    supported("Blazing Sunsteel");
    let mut t = TestGame::new(3);
    set_life(&mut t, P0, 2);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Blazing Sunsteel", giant);
    cast_x(&mut t, P0, "Earthquake", 2);
    t.resolve_all();
    assert!(t.has_lost(P0));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 18);
}

#[test]
fn damage_dealt_by_a_creature_that_left_uses_its_last_known_lifelink() {
    cr!("608.2h", "702.15b", "113.7a");
    ruling!(
        "Fiendlash",
        "If the equipped creature isn't on the battlefield as the triggered ability resolves but it had lifelink when it left the battlefield, you will gain life."
    );
    ruling!(
        "Dire Flail // Dire Blunderbuss",
        "If the equipped creature isn't on the battlefield as the reflexive triggered ability granted by Dire Blunderbuss resolves but it had lifelink when it left the battlefield, you will gain life."
    );
    supported("Fiendlash");
    supported("Dire Flail // Dire Blunderbuss");
    // Fiendlash: Vampire Nighthawk (4/3 with it) is dealt lethal damage; its trigger deals
    // 4 damage to P1 and P0 gains 4 life.
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    attach_new(&mut t, P0, "Fiendlash", hawk);
    let shocker = t.battlefield(P1, "Prodigal Pyromancer");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    deal(&mut t, shocker, 3, hawk);
    assert!(t.in_graveyard(P0, "Vampire Nighthawk"));
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 24);
    // Dire Blunderbuss: the Nighthawk (5/3) attacks, an artifact is sacrificed, and the
    // Nighthawk is destroyed before the reflexive trigger resolves.
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    let flail = attach_new(&mut t, P0, "Dire Flail // Dire Blunderbuss", hawk);
    assert!(mtg_engine::dfc::transform(&mut t.g, flail));
    t.g.recompute();
    assert_eq!(t.pt(hawk), (5, 3));
    t.battlefield(P0, "Ornithopter");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    attack_with(&mut t, &[(hawk, Entity::Player(P1))]);
    // Resolve the attack trigger: the reflexive trigger goes on the stack.
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, hawk);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    assert_eq!(t.life(P0), 25);
}

#[test]
fn mortarpod_damage_is_dealt_by_the_sacrificed_creature_as_it_last_existed() {
    cr!("608.2h", "702.2b", "702.15b", "113.7a");
    ruling!(
        "Mortarpod",
        "If you sacrifice the equipped creature to deal 1 damage to a target permanent or player, the damage is dealt by the creature as it last existed on the battlefield."
    );
    supported("Mortarpod");
    // Vampire Nighthawk (deathtouch, lifelink) is sacrificed: its 1 damage destroys the
    // Craw Wurm and P0 gains 1 life.
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P0, "Vampire Nighthawk");
    attach_new(&mut t, P0, "Mortarpod", hawk);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(wurm)]);
    activate_containing(&mut t, P0, hawk, "Sacrifice ~").unwrap();
    assert!(t.in_graveyard(P0, "Vampire Nighthawk"));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Craw Wurm"));
    assert_eq!(t.life(P0), 21);
}

#[test]
fn conjurers_mantle_uses_last_known_creature_types() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Conjurer's Mantle",
        "If the equipped creature is no longer on the battlefield as the triggered ability resolves, use its creature types from when it was last on the battlefield."
    );
    supported("Conjurer's Mantle");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Conjurer's Mantle", bears);
    let runeclaw = stack_library(&mut t, P0, &["Forest", "Runeclaw Bear", "Forest"])[1];
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, bears);
    t.answer_choose(P0, &[Entity::Object(runeclaw)]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_hand(P0, "Runeclaw Bear"));
}

#[test]
fn sword_of_the_realms_doesnt_return_a_card_that_left_the_graveyard() {
    cr!("400.7", "608.2b");
    ruling!(
        "Halvar, God of Battle // Sword of the Realms",
        "If the creature equipped by Sword of the Realms dies but leaves the graveyard before the triggered ability resolves, it won't be returned to its owner's hand. It will just stay wherever it went."
    );
    supported("Halvar, God of Battle // Sword of the Realms");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sword = back_face(&mut t, P0, "Halvar, God of Battle // Sword of the Realms");
    move_eq(&mut t, sword, bears);
    assert_eq!(t.pt(bears), (4, 2));
    destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, bears, Zone::Exile);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert!(!t.in_hand(P0, "Grizzly Bears"));
    // Control: without the exile, it returns to its owner's hand.
    let bears = t.battlefield(P0, "Grizzly Bears");
    move_eq(&mut t, sword, bears);
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn razor_boomerang_ability_resolution() {
    cr!("608.2b", "608.2c", "608.2h", "602.2b");
    ruling!(
        "Razor Boomerang",
        "If Razor Boomerang is no longer on the battlefield by time the equipped creature’s ability resolves, it’s not returned to its owner’s hand. The rest of the ability resolves as normal, so Razor Boomerang will still deal damage to the targeted creature."
    );
    ruling!(
        "Razor Boomerang",
        "If the targeted creature is an illegal target by the time the equipped creature’s ability resolves, the entire ability doesn’t resolve. Razor Boomerang remains on the battlefield unattached."
    );
    supported("Razor Boomerang");
    // The Boomerang is destroyed in response: the damage is still dealt.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let boomerang = attach_new(&mut t, P0, "Razor Boomerang", bears);
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(elves)]);
    activate_containing(&mut t, P0, bears, "Unattach").unwrap();
    assert_eq!(attached_to(&t, boomerang), None);
    destroy(&mut t, boomerang);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    assert!(t.in_graveyard(P0, "Razor Boomerang"));
    assert!(!t.in_hand(P0, "Razor Boomerang"));
    // The same wording on Toralf's Hammer ("It deals 3 damage to any target. Return
    // Toralf's Hammer to its owner's hand."): the Hammer, destroyed in response, deals the
    // damage as it last existed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hammer = back_face(&mut t, P0, "Toralf, God of Fury // Toralf's Hammer");
    move_eq(&mut t, hammer, bears);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    activate_containing(&mut t, P0, bears, "Unattach").unwrap();
    destroy(&mut t, hammer);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.zone(t.g.current(hammer)), Zone::Graveyard(P0));
    // The target becomes illegal: nothing happens; the Boomerang stays unattached.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let boomerang = attach_new(&mut t, P0, "Razor Boomerang", bears);
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(elves)]);
    activate_containing(&mut t, P0, bears, "Unattach").unwrap();
    move_to(&mut t, elves, Zone::Hand(P1));
    t.resolve_all();
    assert!(t.on_battlefield(boomerang));
    assert_eq!(attached_to(&t, boomerang), None);
    assert!(!t.in_hand(P0, "Razor Boomerang"));
}
