//! Combat damage assignment as rules (CR 510.1a, 510.1c): Siege Behemoth, Ruxa, Zilortha,
//! Indomitable Might, Gurzigost, Master of Cruelties; extra blocks counted (Kemba's
//! Legion, CR 509.1a).

use crate::combat2_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn damage(t: &TestGame, id: ObjectId) -> u32 {
    t.g.obj(id).damage
}

/// P0 attacks P1 with `attackers`; P1 blocks with `blocks`; combat damage is dealt.
fn fight(t: &mut TestGame, attackers: &[ObjectId], blocks: &[(ObjectId, ObjectId)]) {
    let decl: Vec<(ObjectId, Entity)> =
        attackers.iter().map(|a| (*a, Entity::Player(P1))).collect();
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&decl, blocks);
}

#[test]
fn siege_behemoth_lets_each_creature_choose_while_it_attacks() {
    cr!("510.1c", "510.1b");
    ruling!(
        "Siege Behemoth",
        "You decide whether to assign a creature’s combat damage as though it weren’t blocked just before it assigns that damage. You may make a different choice for each creature you control"
    );
    supported("Siege Behemoth");
    let mut t = TestGame::new(2);
    let behemoth = t.battlefield(P0, "Siege Behemoth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let wall1 = t.battlefield(P1, "Wall of Wood");
    let wall2 = t.battlefield(P1, "Wall of Wood");
    // Yes for the first creature asked, no for the second.
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    fight(
        &mut t,
        &[behemoth, bears, giant],
        &[(wall1, bears), (wall2, giant)],
    );
    let asked: Vec<ObjectId> = t
        .asked()
        .iter()
        .filter_map(|(p, d)| match d {
            mtg_engine::decision::Decision::YesNo { source, .. } if *p == P0 => *source,
            _ => None,
        })
        .collect();
    assert_eq!(asked.len(), 2, "one choice for each blocked creature");
    let (yes, no) = (asked[0], asked[1]);
    let (yes_wall, no_wall) = if yes == bears {
        (wall1, wall2)
    } else {
        (wall2, wall1)
    };
    let yes_power = t.pt(yes).0;
    assert_eq!(t.life(P1), 20 - 7 - yes_power);
    assert_eq!(damage(&t, yes_wall), 0);
    assert_eq!(damage(&t, no_wall), t.pt(no).0 as u32);
}

#[test]
fn siege_behemoth_not_attacking_gives_no_choice() {
    cr!("510.1c");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Siege Behemoth");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    // A yes queued: a wrong choice offer would send the damage to P1.
    t.answer_yes(P0, true);
    fight(&mut t, &[bears], &[(wall, bears)]);
    assert_eq!(t.life(P1), 20);
    assert_eq!(damage(&t, wall), 2);
}

#[test]
fn ruxa_lets_creatures_with_no_abilities_assign_as_though_unblocked() {
    cr!("510.1c", "113.1");
    ruling!(
        "Ruxa, Patient Professor",
        "You make this decision individually for each qualifying creature just before it deals combat damage."
    );
    supported("Ruxa, Patient Professor");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ruxa, Patient Professor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    t.answer_yes(P0, true);
    fight(&mut t, &[bears], &[(wall, bears)]);
    // 3/3 with Ruxa's bonus.
    assert_eq!(t.life(P1), 17);
    assert_eq!(damage(&t, wall), 0);
}

#[test]
fn ruxa_doesnt_apply_to_a_creature_an_aura_gives_an_ability() {
    cr!("510.1c");
    ruling!(
        "Ruxa, Patient Professor",
        "A creature with no abilities affected by such an Aura or Equipment would no longer get the +1/+1 bonus or be able to assign combat damage as though it weren't blocked."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ruxa, Patient Professor");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Arcane Flight");
    assert!(t.g.attach(aura, Entity::Object(bears)));
    let spider = t.battlefield(P1, "Giant Spider");
    t.answer_yes(P0, true);
    fight(&mut t, &[bears], &[(spider, bears)]);
    assert_eq!(t.life(P1), 20);
    assert_eq!(damage(&t, spider), 3);
}

#[test]
fn zilortha_lets_non_humans_assign_as_though_unblocked() {
    cr!("510.1c");
    let mut t = TestGame::new(2);
    let zilortha = t.battlefield(P0, "Invasion of Ikoria // Zilortha, Apex of Ikoria");
    // The back face (Zilortha) is the permanent.
    mtg_engine::dfc::transform(&mut t.g, zilortha);
    t.settle();
    let bears = t.battlefield(P0, "Grizzly Bears");
    let human = t.battlefield(P0, "Elite Vanguard");
    let wall1 = t.battlefield(P1, "Wall of Wood");
    let wall2 = t.battlefield(P1, "Wall of Wood");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    fight(&mut t, &[bears, human], &[(wall1, bears), (wall2, human)]);
    // The Bear assigned to P1; the Human (no choice) to its blocker.
    assert_eq!(t.life(P1), 18);
    assert_eq!(damage(&t, wall1), 0);
    assert_eq!(damage(&t, wall2), 2);
}

#[test]
fn indomitable_might_gives_the_choice_to_the_enchanted_creatures_controller() {
    cr!("510.1c", "303.4");
    ruling!(
        "Indomitable Might",
        "Even if it assigns damage as though it were not blocked, it will still receive damage from creatures that are blocking it."
    );
    supported("Indomitable Might");
    let mut t = TestGame::new(2);
    // P1 enchants P0's Bears.
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P1, "Indomitable Might");
    assert!(t.g.attach(aura, Entity::Object(bears)));
    let blocker = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    fight(&mut t, &[bears], &[(blocker, bears)]);
    assert_eq!(t.life(P1), 15);
    assert_eq!(damage(&t, blocker), 0);
    // The Giant's damage was still dealt to the 5/5 Bears.
    assert_eq!(damage(&t, bears), 3);
}

#[test]
fn indomitable_might_declined_assigns_to_blockers() {
    cr!("510.1c");
    ruling!(
        "Indomitable Might",
        "Assigning the creature's damage as though it weren't blocked is all or nothing."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let aura = t.battlefield(P0, "Indomitable Might");
    assert!(t.g.attach(aura, Entity::Object(bears)));
    let blocker = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, false);
    fight(&mut t, &[bears], &[(blocker, bears)]);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(blocker));
}

fn gurzigost_blocked(choose: bool) -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Gurzigost");
    t.lands(P0, "Forest", 2);
    t.hand(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Wood");
    t.set_step(P0, Step::BeginningOfCombat);
    declare(&mut t, P0, &[(g, Entity::Player(P1))]);
    declare_blocks(&mut t, P1, &[(wall, g)]);
    let card = t.g.player(P0).hand[0];
    t.answer_choose(P0, &[Entity::Object(card)]);
    t.answer_yes(P0, choose);
    t.activate(P0, g, 0, &[]).unwrap();
    t.resolve();
    // A yes left in the queue: asking again at damage would assign to P1.
    t.answer_yes(P0, true);
    to_end_of_combat(&mut t);
    (t, wall)
}

#[test]
fn gurzigost_chooses_as_its_ability_resolves() {
    cr!("510.1c", "608.2d");
    ruling!(
        "Gurzigost",
        "You decide how it will assign its damage at the time the ability resolves."
    );
    supported("Gurzigost");
    let (t, wall) = gurzigost_blocked(true);
    assert_eq!(t.life(P1), 14);
    assert_eq!(damage(&t, wall), 0);
    let (t, wall) = gurzigost_blocked(false);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(wall));
}

#[test]
fn master_of_cruelties_sets_life_to_one_and_assigns_no_damage() {
    cr!("510.1a", "119.5");
    ruling!(
        "Master of Cruelties",
        "Assigning no combat damage isn't the same as preventing that damage."
    );
    supported("Master of Cruelties");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Master of Cruelties");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(master, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 1);
}

#[test]
fn master_of_cruelties_blocked_deals_damage_normally() {
    cr!("510.1c");
    let mut t = TestGame::new(2);
    let master = t.battlefield(P0, "Master of Cruelties");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(master, Entity::Player(P1))], &[(bears, master)]);
    assert_eq!(t.life(P1), 20);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn kembas_legion_blocks_an_additional_creature_for_each_equipment() {
    cr!("509.1a", "509.1b");
    ruling!(
        "Kemba's Legion",
        "The number of creatures Kemba's Legion can block is calculated when blocking creatures are declared."
    );
    supported("Kemba's Legion");
    let mut t = TestGame::new(2);
    let legion = t.battlefield(P1, "Kemba's Legion");
    let a: Vec<ObjectId> = (0..3).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    t.set_step(P0, Step::BeginningOfCombat);
    declare(
        &mut t,
        P0,
        &a.iter()
            .map(|x| (*x, Entity::Player(P1)))
            .collect::<Vec<_>>(),
    );
    assert!(legal_blocks(&mut t, &[(legion, a[0])]));
    assert!(!legal_blocks(&mut t, &[(legion, a[0]), (legion, a[1])]));
    for _ in 0..2 {
        let e = t.battlefield(P1, "Bonesplitter");
        assert!(t.g.attach(e, Entity::Object(legion)));
    }
    assert!(legal_blocks(
        &mut t,
        &[(legion, a[0]), (legion, a[1]), (legion, a[2])]
    ));
    declare_blocks(
        &mut t,
        P1,
        &[(legion, a[0]), (legion, a[1]), (legion, a[2])],
    );
    for x in &a {
        assert!(t.g.combat.as_ref().unwrap().is_blocked(*x));
    }
}
