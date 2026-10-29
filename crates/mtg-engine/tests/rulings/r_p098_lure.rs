//! Rulings batch P098 — requirements on blocks (CR 509.1c): "all creatures able to block
//! this creature do so" (Treeshaker Chimera, Breaker of Armies, Ochran Assassin, Prized
//! Unicorn, Taunting Arbormage, Lure) and "must be blocked if able" (Gaea's Protector,
//! Irresistible Prey, Joraga Invocation, Crashing Boars), against creatures that can't
//! block and costs to block (CR 509.1d).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s06_common::attach_new;
use crate::r_s20_common::to_beginning_of_combat;
use crate::r_s21_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn p1() -> Entity {
    Entity::Player(P1)
}

fn at_p1(attackers: &[ObjectId]) -> Vec<(ObjectId, Entity)> {
    attackers.iter().map(|a| (*a, p1())).collect()
}

/// P1 declares `blocks`; the game stops after the declare blockers step begins (an
/// illegal declaration is replaced by a legal one, CR 509.1).
fn declare_blocks(t: &mut TestGame, blocks: &[(ObjectId, ObjectId)]) {
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(blocks.to_vec()),
    );
    go_to(t, Step::DeclareBlockers);
}

fn sorted(mut v: Vec<(ObjectId, ObjectId)>) -> Vec<(ObjectId, ObjectId)> {
    v.sort();
    v
}

/// The "all creatures able to block this creature do so" attackers of the batch, each
/// alone: a tapped creature and a creature that can't block (Pacifism) don't block it; a
/// creature that could block it only by paying a cost (Archangel of Tithes attacking too)
/// needn't; an untapped creature with no cost must.
#[test]
fn all_able_to_block_doesnt_force_tapped_restricted_or_taxed_creatures() {
    cr!("509.1a", "509.1c", "509.1d");
    ruling!(
        "Treeshaker Chimera",
        "If a creature can't block Treeshaker Chimera for any reason (such as being tapped), then it doesn't block."
    );
    ruling!(
        "Ochran Assassin",
        "If a creature the defending player controls can’t block Ochran Assassin for any reason (such as being tapped), then it doesn’t block Ochran Assassin. If there’s a cost associated with having a creature block Ochran Assassin, the defending player isn’t forced to pay that cost"
    );
    ruling!(
        "Prized Unicorn",
        "If a creature the defending player controls can’t block Prized Unicorn for any reason (such as being tapped), then it doesn’t block Prized Unicorn. If there’s a cost associated with having a creature block Prized Unicorn, the defending player isn’t forced to pay that cost, so it doesn’t have to block."
    );
    ruling!(
        "Breaker of Armies",
        "If, during its controller’s declare blockers step, a creature the defending player controls is tapped or is affected by a spell or ability that says it can’t block, then it doesn’t block. If there’s a cost associated with having it block, its controller isn’t forced to pay that cost."
    );
    for name in [
        "Treeshaker Chimera",
        "Ochran Assassin",
        "Prized Unicorn",
        "Breaker of Armies",
    ] {
        supported(name);
        // Tapped and Pacified creatures don't block; the free one must.
        let mut t = TestGame::new(2);
        let lurer = t.battlefield(P0, name);
        let tapped = t.battlefield(P1, "Llanowar Elves");
        t.g.tap(tapped);
        let pacified = t.battlefield(P1, "Gray Ogre");
        attach_new(&mut t, P0, "Pacifism", pacified);
        let free = t.battlefield(P1, "Hill Giant");
        to_beginning_of_combat(&mut t, P0);
        attack_with(&mut t, &at_p1(&[lurer]));
        assert!(!legal_blocks(&mut t, P1, &[]), "{name}");
        assert!(legal_blocks(&mut t, P1, &[(free, lurer)]), "{name}");
        assert!(
            !legal_blocks(&mut t, P1, &[(free, lurer), (tapped, lurer)]),
            "{name}"
        );
        declare_blocks(&mut t, &[]);
        assert_eq!(blocks_now(&t), vec![(free, lurer)], "{name}");

        // A cost to block (Archangel of Tithes attacking): nothing has to block.
        let mut t = TestGame::new(2);
        let lurer = t.battlefield(P0, name);
        let angel = t.battlefield(P0, "Archangel of Tithes");
        let giant = t.battlefield(P1, "Hill Giant");
        t.lands(P1, "Wastes", 1);
        to_beginning_of_combat(&mut t, P0);
        attack_with(&mut t, &at_p1(&[lurer, angel]));
        assert!(legal_blocks(&mut t, P1, &[]), "{name}");
        assert!(legal_blocks(&mut t, P1, &[(giant, lurer)]), "{name}");
        declare_blocks(&mut t, &[]);
        assert!(blocks_now(&t).is_empty(), "{name}");
        assert_eq!(tapped_lands(&t, P1), 0, "{name}");
    }
}

#[test]
fn a_cost_to_block_only_the_lure_creature_leaves_the_blocker_free() {
    cr!("509.1c", "509.1d");
    ruling!(
        "Treeshaker Chimera",
        "If there's a cost associated with having it block Treeshaker Chimera, but not to block other attacking creatures, that creature is free to block one of those other creatures or to not block at all."
    );
    ruling!(
        "Breaker of Armies",
        "Likewise, if a creature the defending player controls can’t block Breaker of Armies unless its controller pays a cost, its controller may decline to pay that cost and block a different attacking creature."
    );
    supported("Awesome Presence");
    // Awesome Presence: "Enchanted creature can't be blocked unless defending player pays
    // {3} for each creature they control that's blocking it."
    for name in ["Treeshaker Chimera", "Breaker of Armies"] {
        let mut t = TestGame::new(2);
        let lurer = t.battlefield(P0, name);
        attach_new(&mut t, P0, "Awesome Presence", lurer);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        t.lands(P1, "Wastes", 3);
        to_beginning_of_combat(&mut t, P0);
        attack_with(&mut t, &at_p1(&[lurer, bears]));
        assert!(legal_blocks(&mut t, P1, &[]), "{name}");
        assert!(legal_blocks(&mut t, P1, &[(giant, bears)]), "{name}");
        declare_blocks(&mut t, &[(giant, bears)]);
        assert_eq!(blocks_now(&t), vec![(giant, bears)], "{name}");
        assert_eq!(tapped_lands(&t, P1), 0, "{name}");
    }
}

#[test]
fn a_creature_that_cant_block_breaker_of_armies_may_block_another_attacker() {
    cr!("509.1b", "509.1c");
    ruling!(
        "Breaker of Armies",
        "If a creature can’t legally block Breaker of Armies but could block another attacking creature, it may do so."
    );
    supported("Cloud Elemental");
    // Cloud Elemental: "can block only creatures with flying."
    let mut t = TestGame::new(2);
    let breaker = t.battlefield(P0, "Breaker of Armies");
    let angel = t.battlefield(P0, "Serra Angel");
    let cloud = t.battlefield(P1, "Cloud Elemental");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[breaker, angel]));
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(cloud, angel)]));
    assert!(!legal_blocks(&mut t, P1, &[(cloud, breaker)]));
    declare_blocks(&mut t, &[(cloud, angel)]);
    assert_eq!(blocks_now(&t), vec![(cloud, angel)]);
}

#[test]
fn two_lure_attackers_each_blocker_picks_one_and_multi_blockers_block_both() {
    cr!("509.1c");
    ruling!(
        "Prized Unicorn",
        "If more than one Prized Unicorn is attacking, creatures that could block must block one Prized Unicorn, but they don’t all have to block the same Prized Unicorn."
    );
    ruling!(
        "Breaker of Armies",
        "If more than one Breaker of Armies is attacking, the controller of each creature that could block them chooses which one that creature blocks. In this case, creatures that can block multiple creatures must block as many attacking Breaker of Armies as possible."
    );
    supported("Palace Guard");
    for name in ["Prized Unicorn", "Breaker of Armies"] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P0, name);
        let b = t.battlefield(P0, name);
        let x = t.battlefield(P1, "Grizzly Bears");
        let y = t.battlefield(P1, "Hill Giant");
        to_beginning_of_combat(&mut t, P0);
        attack_with(&mut t, &at_p1(&[a, b]));
        assert!(legal_blocks(&mut t, P1, &[(x, a), (y, a)]), "{name}");
        assert!(legal_blocks(&mut t, P1, &[(x, a), (y, b)]), "{name}");
        assert!(!legal_blocks(&mut t, P1, &[(x, a)]), "{name}");
        declare_blocks(&mut t, &[(x, b), (y, a)]);
        assert_eq!(
            sorted(blocks_now(&t)),
            sorted(vec![(x, b), (y, a)]),
            "{name}"
        );
    }
    // Palace Guard ("can block any number of creatures each combat") must block both.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Breaker of Armies");
    let b = t.battlefield(P0, "Breaker of Armies");
    let guard = t.battlefield(P1, "Palace Guard");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[a, b]));
    assert!(!legal_blocks(&mut t, P1, &[(guard, a)]));
    assert!(legal_blocks(&mut t, P1, &[(guard, a), (guard, b)]));
    declare_blocks(&mut t, &[(guard, a)]);
    assert_eq!(sorted(blocks_now(&t)), sorted(vec![(guard, a), (guard, b)]));
}

#[test]
fn taunting_arbormage_kicked_tapped_and_taxed_creatures_needn_t_block() {
    cr!("509.1c", "509.1d");
    ruling!(
        "Taunting Arbormage",
        "If a creature the defending player controls can't block the target creature for any reason (such as being tapped), then it doesn't block that creature. If there's a cost associated with having a creature block the attacking creature, the defending player isn't forced to pay that cost"
    );
    supported("Taunting Arbormage");
    // "When this creature enters, if it was kicked, all creatures able to block target
    // creature this turn do so."
    let arbormage = |t: &mut TestGame, target: ObjectId| {
        let card = t.hand(P0, "Taunting Arbormage");
        t.lands(P0, "Forest", 6);
        t.answer_targets(P0, &[Entity::Object(target)]);
        t.cast(P0, card).kicked(true).go();
        t.resolve_all();
    };
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    arbormage(&mut t, bears);
    let tapped = t.battlefield(P1, "Llanowar Elves");
    t.g.tap(tapped);
    let free = t.battlefield(P1, "Hill Giant");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears]));
    assert!(!legal_blocks(&mut t, P1, &[]));
    declare_blocks(&mut t, &[]);
    assert_eq!(blocks_now(&t), vec![(free, bears)]);

    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    arbormage(&mut t, bears);
    let angel = t.battlefield(P0, "Archangel of Tithes");
    t.battlefield(P1, "Hill Giant");
    t.lands(P1, "Wastes", 1);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[bears, angel]));
    assert!(legal_blocks(&mut t, P1, &[]));
    declare_blocks(&mut t, &[]);
    assert!(blocks_now(&t).is_empty());
}

#[test]
fn lure_doesnt_let_creatures_block_what_they_couldnt() {
    cr!("509.1b", "509.1c");
    ruling!(
        "Lure",
        "Lure doesn’t give any creatures the ability to block the target creature. It just forces those creatures that are already able to block the creature to do so."
    );
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    attach_new(&mut t, P0, "Lure", angel);
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[angel]));
    assert!(legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(bears, angel)]));
    declare_blocks(&mut t, &[]);
    assert!(blocks_now(&t).is_empty());
}

#[test]
fn gaeas_protector_one_blocker_of_the_defenders_choice_unless_none_can_for_free() {
    cr!("509.1c", "509.1d");
    ruling!(
        "Gaea's Protector",
        "If each creature the defending player controls can’t block for any reason (such as being tapped), then Gaea’s Protector isn’t blocked. If there’s a cost associated with blocking Gaea’s Protector, the defending player isn’t forced to pay that cost"
    );
    ruling!(
        "Gaea's Protector",
        "Only one creature is required to block Gaea’s Protector. Other creatures may also block it, and are free to block other creatures or not block at all."
    );
    ruling!(
        "Gaea's Protector",
        "The defending player, not you, chooses which creature blocks Gaea’s Protector."
    );
    supported("Gaea's Protector");
    // All tapped: unblocked.
    let mut t = TestGame::new(2);
    let gp = t.battlefield(P0, "Gaea's Protector");
    let x = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(x);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[gp]));
    assert!(legal_blocks(&mut t, P1, &[]));
    declare_blocks(&mut t, &[]);
    assert!(t.g.combat.as_ref().unwrap().is_unblocked(gp));
    // A cost to block (Archangel of Tithes): not forced.
    let mut t = TestGame::new(2);
    let gp = t.battlefield(P0, "Gaea's Protector");
    let angel = t.battlefield(P0, "Archangel of Tithes");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Wastes", 1);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[gp, angel]));
    assert!(legal_blocks(&mut t, P1, &[]));
    // One blocker of P1's choice; the others are free.
    let mut t = TestGame::new(2);
    let gp = t.battlefield(P0, "Gaea's Protector");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let x = t.battlefield(P1, "Grizzly Bears");
    let y = t.battlefield(P1, "Hill Giant");
    let z = t.battlefield(P1, "Wall of Wood");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[gp, bears]));
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(x, bears)]));
    assert!(legal_blocks(&mut t, P1, &[(x, gp)]));
    assert!(legal_blocks(&mut t, P1, &[(y, gp), (x, bears)]));
    assert!(legal_blocks(&mut t, P1, &[(x, gp), (y, gp), (z, gp)]));
    declare_blocks(&mut t, &[(z, gp), (y, bears)]);
    assert_eq!(sorted(blocks_now(&t)), sorted(vec![(z, gp), (y, bears)]));
}

#[test]
fn irresistible_prey_must_be_blocked_targets_and_attacking() {
    cr!("509.1c", "608.2b");
    ruling!(
        "Irresistible Prey",
        "If the affected creature attacks, the defending player must assign at least one blocker to it during the declare blockers step, unless that player controls no creatures that could block it."
    );
    ruling!(
        "Irresistible Prey",
        "The creature affected by Irresistible Prey doesn’t have to attack that turn."
    );
    ruling!(
        "Irresistible Prey",
        "You can target a creature you don’t control or that otherwise couldn’t attack that turn."
    );
    ruling!(
        "Irresistible Prey",
        "If the targeted creature is an illegal target by the time Irresistible Prey resolves, the spell doesn’t resolve. You won’t draw a card."
    );
    supported("Irresistible Prey");
    let prey = |t: &mut TestGame, target: ObjectId| {
        let card = t.hand(P0, "Irresistible Prey");
        t.lands(P0, "Forest", 1);
        t.cast(P0, card).target(target).go();
    };
    // Targeting an opponent's creature, and a summoning-sick one of P0's.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    prey(&mut t, theirs);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    let mut t = TestGame::new(2);
    let sick = t.battlefield_sick(P0, "Grizzly Bears");
    prey(&mut t, sick);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    // It needn't attack (no attack requirement): declaring no attackers is legal.
    let bears = t.battlefield(P0, "Hill Giant");
    let card = t.hand(P0, "Irresistible Prey");
    t.lands(P0, "Forest", 1);
    t.cast(P0, card).target(bears).go();
    t.resolve_all();
    to_beginning_of_combat(&mut t, P0);
    assert!(crate::r_s09_common::legal_attack(&mut t, &[]));
    // When it attacks, a creature that could block it must.
    let flyer = t.battlefield(P1, "Serra Angel");
    let wall = t.battlefield(P1, "Wall of Wood");
    attack_with(&mut t, &at_p1(&[bears]));
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(legal_blocks(&mut t, P1, &[(wall, bears)]));
    assert!(legal_blocks(&mut t, P1, &[(flyer, bears)]));
    // The target leaves before it resolves: no card is drawn.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    prey(&mut t, bears);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert!(t.in_graveyard(P0, "Irresistible Prey"));
}

#[test]
fn joraga_invocation_the_defender_chooses_how_to_block() {
    cr!("509.1c");
    ruling!(
        "Joraga Invocation",
        "Joraga Invocation doesn't force any specific creature to block any specific attacking creature. The defending player still chooses how creatures they control block."
    );
    supported("Joraga Invocation");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let inv = t.hand(P0, "Joraga Invocation");
    t.lands(P0, "Forest", 6);
    t.cast(P0, inv).go();
    t.resolve_all();
    assert_eq!(t.pt(a), (5, 5));
    let x = t.battlefield(P1, "Wall of Wood");
    let y = t.battlefield(P1, "Gray Ogre");
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[a, b]));
    // Each attacker must get a blocker, but which blocks which is P1's choice.
    assert!(!legal_blocks(&mut t, P1, &[(x, a), (y, a)]));
    assert!(legal_blocks(&mut t, P1, &[(x, a), (y, b)]));
    assert!(legal_blocks(&mut t, P1, &[(x, b), (y, a)]));
    declare_blocks(&mut t, &[(x, b), (y, a)]);
    assert_eq!(sorted(blocks_now(&t)), sorted(vec![(x, b), (y, a)]));
}

#[test]
fn crashing_boars_is_mandatory_and_the_choice_must_still_be_untapped() {
    cr!("509.1c", "603.3d");
    ruling!(
        "Crashing Boars",
        "If the chosen creature is not still untapped at the time blockers are declared, it does not have to block."
    );
    ruling!(
        "Crashing Boars",
        "The ability is not optional. If there are no untapped creatures to choose, then the ability does nothing."
    );
    // The chosen creature becomes tapped before blockers: no requirement left.
    let mut t = TestGame::new(2);
    let boars = t.battlefield(P0, "Crashing Boars");
    let chosen = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P1, &[Entity::Object(chosen)]);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[boars]));
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(other, boars)]));
    assert!(legal_blocks(&mut t, P1, &[(chosen, boars)]));
    // The chosen creature is tapped before blockers are declared: the untapped Hill
    // Giant isn't required to block in its place.
    t.g.tap(chosen);
    assert!(legal_blocks(&mut t, P1, &[]));
    declare_blocks(&mut t, &[]);
    assert!(blocks_now(&t).is_empty());
    // No untapped creatures: nothing is chosen and nothing must block.
    let mut t = TestGame::new(2);
    let boars = t.battlefield(P0, "Crashing Boars");
    let x = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(x);
    to_beginning_of_combat(&mut t, P0);
    attack_with(&mut t, &at_p1(&[boars]));
    t.resolve_all();
    t.g.untap(x);
    assert!(legal_blocks(&mut t, P1, &[]));
}
