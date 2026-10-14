//! Abilities granted with the granting object's name (`src/granted_by.rs`, gap R3):
//! quotes that name the granting card and also say "this creature" ("Whenever this
//! creature attacks, you may sacrifice Sunfire Torch. When you do, this creature deals 2
//! damage to any target."). The ability is the creature's (CR 113.7): "this creature" is
//! the creature, while the card's name still means that specific granting object
//! (CR 201.5a), also in costs ("Tap Fishing Pole", "Remove all aim counters from Hankyu")
//! and values ("the number of arrow counters on Archery Training").

use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

fn assert_supported(names: &[&str]) {
    for n in names {
        let u = card(n).unsupported_text().join(" | ");
        assert!(u.is_empty(), "{n} has unsupported text: {u}");
    }
}

fn attach_new(t: &mut TestGame, p: PlayerId, name: &str, to: ObjectId) -> ObjectId {
    let id = t.battlefield(p, name);
    assert!(t.g.attach(id, Entity::Object(to)));
    t.g.recompute();
    id
}

/// The indices of the activated abilities `id` has now.
fn activated(t: &mut TestGame, id: ObjectId) -> Vec<usize> {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .enumerate()
        .map(|(i, _)| i)
        .collect()
}

/// Declares attackers for the active player and stops in the declare attackers step
/// with the attack triggers on the stack.
fn attack_with(t: &mut TestGame, attackers: &[(ObjectId, Entity)]) {
    let ap = t.g.turn.active;
    if t.g.turn.step != Step::BeginningOfCombat {
        t.set_step(ap, Step::BeginningOfCombat);
    }
    t.answer(
        ap,
        DecisionKind::Attackers,
        Answer::Attackers(attackers.to_vec()),
    );
    let turn = t.g.turn.number;
    let ok = t.g.run_until(10_000, |g| {
        (g.turn.step == Step::DeclareAttackers
            && g.turn.stage == Stage::Priority
            && g.turn.priority == Some(ap))
            || g.turn.number != turn
    });
    assert!(ok && t.g.turn.number == turn, "attackers not declared");
}

#[test]
fn sunfire_torch_the_creature_deals_the_damage_after_the_torch_is_sacrificed() {
    cr!("113.7", "201.5a", "603.12", "701.21a");
    ruling!(
        "Sunfire Torch",
        "The source of the damage is the equipped creature, not Sunfire Torch."
    );
    ruling!(
        "Sunfire Torch",
        "You don't choose a target for the triggered ability granted by Sunfire Torch at the time it triggers."
    );
    assert_supported(&["Sunfire Torch", "Spare Dagger"]);
    // Green Grizzly Bears equipped with Sunfire Torch ({R}... red artifact) attack; the
    // reflexive trigger targets Tel-Jilad Chosen (protection from artifacts): the Bears
    // deal the damage, so it isn't prevented.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let torch = attach_new(&mut t, P0, "Sunfire Torch", bears);
    let other = t.battlefield(P0, "Sunfire Torch");
    assert_eq!(t.pt(bears), (3, 2));
    let chosen = t.battlefield(P1, "Tel-Jilad Chosen");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(chosen)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    // The attached Torch was sacrificed, not the other one.
    assert!(!t.on_battlefield(torch));
    assert!(t.on_battlefield(other));
    // (Tel-Jilad Chosen is a 2/1.)
    assert!(t.in_graveyard(P1, "Tel-Jilad Chosen"));
    // Spare Dagger: 1 damage; declining keeps the Dagger.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let dagger = attach_new(&mut t, P0, "Spare Dagger", bears);
    t.answer_yes(P0, false);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.on_battlefield(dagger));
    assert_eq!(t.life(P1), 20);
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let dagger = attach_new(&mut t, P0, "Spare Dagger", bears);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    assert!(!t.on_battlefield(dagger));
    assert_eq!(t.life(P1), 19);
}

#[test]
fn tricksters_talisman_copies_the_creature_but_only_its_controller_can_sacrifice_it() {
    cr!("113.7", "201.5a", "701.21a", "707.2");
    ruling!(
        "Trickster's Talisman",
        "that player will not be able to sacrifice Trickster's Talisman"
    );
    assert_supported(&["Trickster's Talisman"]);
    // P0's Talisman on P0's Bears: the combat damage trigger copies the Bears.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let talisman = attach_new(&mut t, P0, "Trickster's Talisman", bears);
    t.answer_yes(P0, true);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert!(!t.on_battlefield(talisman));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    // P1's Talisman on P0's Bears: P0 can't sacrifice it, so there's no copy.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let talisman = attach_new(&mut t, P1, "Trickster's Talisman", bears);
    t.answer_yes(P0, true);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(t.on_battlefield(talisman));
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn archery_training_counts_the_counters_on_the_aura_that_granted_the_ability() {
    cr!("113.7", "201.5a", "613.1f");
    ruling!(
        "Archery Training",
        "each granted ability counts the counters on the Archery Training that granted that ability"
    );
    ruling!(
        "Archery Training",
        "You do not remove counters when using the ability."
    );
    assert_supported(&["Archery Training"]);
    // Two Archery Trainings on P0's Bears, with one and two arrow counters.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let one = attach_new(&mut t, P0, "Archery Training", bears);
    let two = attach_new(&mut t, P0, "Archery Training", bears);
    t.g.add_counters(Entity::Object(one), "arrow", 1, None);
    t.g.add_counters(Entity::Object(two), "arrow", 2, None);
    let abilities = activated(&mut t, bears);
    assert_eq!(abilities.len(), 2);
    // P0's Craw Wurm (6/4) attacks; the Bears stay home and shoot it.
    let wurm = t.battlefield(P0, "Craw Wurm");
    attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
    t.resolve_all();
    let mut dealt = Vec::new();
    for i in abilities {
        let before = t.obj_now(wurm).damage;
        t.activate(P0, bears, i, &[Entity::Object(wurm)]).unwrap();
        t.resolve_all();
        dealt.push(t.obj_now(wurm).damage - before);
        t.g.untap(t.g.current(bears));
    }
    dealt.sort();
    assert_eq!(dealt, vec![1, 2]);
    assert_eq!(t.counters(one, "arrow"), 1);
    assert_eq!(t.counters(two, "arrow"), 2);
}

#[test]
fn hankyu_and_fishing_pole_counters_and_tapping_are_the_equipments() {
    cr!("113.7", "201.5a", "602.2b", "107.3");
    ruling!(
        "Fishing Pole",
        "refers only to the Fishing Pole granting that ability"
    );
    assert_supported(&["Hankyu", "Fishing Pole"]);
    // Hankyu: "{T}: Put an aim counter on Hankyu" and "{T}, Remove all aim counters from
    // Hankyu: This creature deals damage to any target equal to the number of aim
    // counters removed this way."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hankyu = attach_new(&mut t, P0, "Hankyu", bears);
    for _ in 0..2 {
        t.activate(P0, bears, 0, &[]).unwrap();
        t.resolve_all();
        t.g.untap(t.g.current(bears));
    }
    assert_eq!(t.counters(hankyu, "aim"), 2);
    assert_eq!(t.counters(bears, "aim"), 0);
    t.activate(P0, bears, 1, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(hankyu, "aim"), 0);
    assert_eq!(t.life(P1), 18);
    // Fishing Pole: "{1}, {T}, Tap Fishing Pole: Put a bait counter on Fishing Pole."
    // Two Poles on two creatures: each ability taps and counts on its own Pole.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pole = attach_new(&mut t, P0, "Fishing Pole", bears);
    let ogre = t.battlefield(P0, "Gray Ogre");
    let other = attach_new(&mut t, P0, "Fishing Pole", ogre);
    t.lands(P0, "Wastes", 2);
    let i = activated(&mut t, bears)[0];
    t.activate(P0, bears, i, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(pole, "bait"), 1);
    assert_eq!(t.counters(other, "bait"), 0);
    assert!(t.obj_now(pole).tapped);
    assert!(!t.obj_now(other).tapped);
    // A tapped Pole can't be tapped to pay the cost again.
    t.g.untap(t.g.current(bears));
    assert!(t.activate(P0, bears, i, &[]).is_err());
}

#[test]
fn animal_friend_counts_other_auras_and_equipment() {
    cr!("113.7", "201.5a", "122.6");
    assert_supported(&["Animal Friend"]);
    // Bears with Animal Friend and Bonesplitter: the Squirrel gets one counter (for the
    // Bonesplitter, not for Animal Friend).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Animal Friend", bears);
    attach_new(&mut t, P0, "Bonesplitter", bears);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    let squirrels = t.named_on_battlefield("Squirrel Token");
    assert_eq!(squirrels.len(), 1);
    assert_eq!(t.counters(squirrels[0], "+1/+1"), 1);
}

#[test]
fn grothama_fights_the_attacking_creature_if_its_controller_chooses() {
    cr!("113.7", "201.5a", "701.14a");
    ruling!(
        "Grothama, All-Devouring",
        "The controller of the attacking creature chooses whether that creature fights Grothama."
    );
    assert_supported(&["Grothama, All-Devouring"]);
    // P1's Grothama (10/8); P0's Craw Wurm (6/4) attacks and fights it.
    let mut t = TestGame::new(2);
    let grothama = t.battlefield(P1, "Grothama, All-Devouring");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Craw Wurm"));
    assert_eq!(t.obj_now(grothama).damage, 6);
    // Declined: no fight.
    let mut t = TestGame::new(2);
    let grothama = t.battlefield(P1, "Grothama, All-Devouring");
    let wurm = t.battlefield(P0, "Craw Wurm");
    t.answer_yes(P0, false);
    attack_with(&mut t, &[(wurm, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.obj_now(grothama).damage, 0);
}

#[test]
fn shifting_shadow_moves_to_the_new_creature_even_if_the_old_one_survives() {
    cr!("113.7", "201.5a", "701.8a");
    ruling!(
        "Shifting Shadow",
        "If the enchanted creature can't be destroyed, most likely because that creature has indestructible"
    );
    assert_supported(&["Shifting Shadow"]);
    // Darksteel Myr (indestructible) enchanted with Shifting Shadow; at P0's upkeep a
    // Hill Giant is revealed under a Forest.
    let mut t = TestGame::new(2);
    let myr = t.battlefield(P0, "Darksteel Myr");
    let shadow = attach_new(&mut t, P0, "Shifting Shadow", myr);
    t.library_top(P0, "Hill Giant");
    t.library_top(P0, "Forest");
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(myr));
    let giant = t.named_on_battlefield("Hill Giant");
    assert_eq!(giant.len(), 1);
    assert_eq!(
        t.obj_now(shadow).attached_to,
        Some(Entity::Object(giant[0]))
    );
    assert!(t.obj_now(giant[0]).has_keyword(mtg_engine::keywords::KeywordKind::Haste));
}
