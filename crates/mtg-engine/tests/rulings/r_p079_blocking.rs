//! Rulings batch P079 — blocks made by effects and restrictions on blocking: "becomes
//! blocked" without blockers (CR 509.1h, 509.3c–d), tokens put onto the battlefield
//! blocking (CR 509.1, 509.3), "creatures can't block this turn" (CR 611.2c), timing
//! restrictions in combat (CR 506–511), and damage assignment by the defending player.

use crate::r_p079_common::*;
use crate::r_s01_common::{attack_with, supported, tokens};
use crate::r_s03_common::to_blockers;
use crate::r_s10_common::blocking;
use crate::r_s21_common::go_to;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn choking_vines_blocks_creatures_that_cant_be_blocked() {
    cr!("509.1h", "509.1a", "702.14c", "702.19d", "509.3c", "509.3d");
    ruling!(
        "Choking Vines",
        "Choking Vines will make any creatures it targets become blocked, even if the creature had blocking restrictions on it."
    );
    ruling!(
        "Choking Vines",
        "Since the creature with trample has no blocker to assign damage to, all the damage will be assigned to the defending player."
    );
    ruling!(
        "Choking Vines",
        "any “When this creature becomes blocked” abilities that creature has will trigger. Any “When this creature becomes blocked by a creature” abilities that creature has won’t trigger."
    );
    supported("Choking Vines");
    supported("Alien Salamander");
    supported("Deepwood Tantiv");
    supported("Sylvan Basilisk");
    // P1 controls an Island: the islandwalker, an unblockable Phantom Warrior, Deepwood
    // Tantiv ("whenever this creature becomes blocked, you gain 2 life"), Sylvan Basilisk
    // ("whenever this creature becomes blocked by a creature, destroy that creature") and a
    // trampling Colossal Dreadmaw all become blocked.
    let mut t = TestGame::new(2);
    t.lands(P1, "Island", 1);
    let names = [
        "Alien Salamander",
        "Phantom Warrior",
        "Deepwood Tantiv",
        "Sylvan Basilisk",
        "Colossal Dreadmaw",
    ];
    let ids: Vec<ObjectId> = names.iter().map(|n| t.battlefield(P0, n)).collect();
    let decl: Vec<(ObjectId, Entity)> = ids.iter().map(|i| (*i, Entity::Player(P1))).collect();
    to_blockers(&mut t, &decl, &[]);
    t.lands(P1, "Forest", 6);
    let vines = t.hand(P1, "Choking Vines");
    let targets: Vec<Entity> = ids.iter().map(|i| obj(*i)).collect();
    t.cast(P1, vines).x(5).targets(&targets).go();
    t.resolve();
    t.settle();
    let c = t.g.combat.as_ref().unwrap();
    for (i, n) in ids.iter().zip(names) {
        assert!(c.is_blocked(*i), "{n} is blocked");
    }
    assert_eq!(t.stack_len(), 1, "only the Tantiv's trigger");
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 14, "only the Dreadmaw's 6 trample damage");
}

#[test]
fn choking_vines_only_in_the_declare_blockers_step() {
    cr!("601.3", "509.1");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.lands(P1, "Forest", 2);
    let vines = t.hand(P1, "Choking Vines");
    assert!(!can_cast_now(&mut t, P1, vines), "declare attackers step");
    go_to(&mut t, Step::DeclareBlockers);
    assert!(can_cast_now(&mut t, P1, vines));
}

#[test]
fn chaos_doesnt_undo_blocks_and_stops_later_creatures_from_blocking() {
    cr!("611.2c", "509.1a", "506.4");
    ruling!(
        "Order // Chaos",
        "Casting Chaos after blockers have been declared won't cause any of those blocks to be undone."
    );
    ruling!(
        "Order // Chaos",
        "Chaos will stop any creature from being declared as a blocker that turn, including ones that weren't on the battlefield as Chaos resolved."
    );
    supported("Order // Chaos");
    // After blocks: the block stands.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Wall of Stone");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(wall, bears)]);
    let chaos = t.hand(P0, "Order // Chaos");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    t.cast(P0, chaos)
        .method(mtg_engine::object::CastMethod::Half(1))
        .go();
    t.resolve_all();
    assert!(blocking(&t, wall));
    assert!(t.g.combat.as_ref().unwrap().is_blocked(bears));
    // Before combat: a creature that enters afterwards can't block either.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let chaos = t.hand(P0, "Order // Chaos");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    t.cast(P0, chaos)
        .method(mtg_engine::object::CastMethod::Half(1))
        .go();
    t.resolve_all();
    let late = t.battlefield(P1, "Wall of Stone");
    t.attack(&[(bears, Entity::Player(P1))], &[(late, bears)]);
    assert!(!blocking(&t, late));
    assert_eq!(t.life(P1), 18);
}

#[test]
fn flash_foliage_timing_and_blocking_after_damage() {
    cr!("509.1", "510.1", "506.4", "500.8");
    ruling!(
        "Flash Foliage",
        "If it's cast after combat damage is dealt, the Saproling will enter blocking the attacking creature, but will neither deal combat damage to it nor be dealt combat damage by it."
    );
    ruling!(
        "Flash Foliage",
        "If a turn has multiple combat phases, this spell can be cast during any of them as long as it's after the beginning of that phase's Declare Blockers Step."
    );
    supported("Flash Foliage");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.lands(P1, "Forest", 3);
    let foliage = t.hand(P1, "Flash Foliage");
    assert!(
        !can_cast_now(&mut t, P1, foliage),
        "before blockers are declared"
    );
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    assert!(can_cast_now(&mut t, P1, foliage));
    t.answer_targets(P1, &[obj(bears)]);
    t.cast(P1, foliage).go();
    t.resolve_all();
    let saproling = tokens(&t, P1)[0];
    assert!(blocking(&t, saproling));
    assert_eq!(crate::r_s07_common::damage_on(&t, saproling), 0);
    assert_eq!(crate::r_s07_common::damage_on(&t, bears), 0);
    // A second combat phase this turn: castable again once its blockers are declared.
    t.g.objects[bears.0 as usize].tapped = false;
    crate::r_s09_common::to_combat(&mut t, P0);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    let foliage = t.hand(P1, "Flash Foliage");
    t.lands(P1, "Forest", 3);
    assert!(
        !can_cast_now(&mut t, P1, foliage),
        "second combat, declare attackers step"
    );
    go_to(&mut t, Step::DeclareBlockers);
    assert!(can_cast_now(&mut t, P1, foliage));
}

#[test]
fn mirror_mockery_tokens_are_all_exiled_and_copy_a_tokens_original_values() {
    cr!("707.2", "111.3", "603.7", "614.1a");
    ruling!(
        "Mirror Mockery",
        "If Mirror Mockery’s ability creates multiple tokens (due to an effect such as the one Doubling Season creates), all such tokens will be exiled by the delayed triggered ability."
    );
    ruling!(
        "Mirror Mockery",
        "If the copied creature is a token, the token created by Mirror Mockery copies the original characteristics of that token as stated by the effect that put the token onto the battlefield."
    );
    supported("Mirror Mockery");
    // P1's Mirror Mockery on P0's attacking Soldier token (with a +1/+1 counter); P1
    // controls Doubling Season: two 1/1 Soldier copies, both exiled at end of combat.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Doubling Season");
    let soldier = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    let soldier = t.g.current(soldier);
    t.g.objects[soldier.0 as usize].summoning_sick = false;
    crate::r_s13_common::add(&mut t, soldier, "+1/+1", 1);
    crate::r_s06_common::attach_new(&mut t, P1, "Mirror Mockery", soldier);
    t.answer_yes(P1, true);
    attack_with(&mut t, &[(soldier, Entity::Player(P1))]);
    t.resolve_all();
    let copies = tokens(&t, P1);
    assert_eq!(copies.len(), 2);
    for c in &copies {
        assert_eq!(t.pt(*c), (1, 1));
        assert_eq!(t.counters(*c, "+1/+1"), 0);
        assert!(t.obj_now(*c).chars.has_subtype("Soldier"));
    }
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    t.advance_to(P0, Step::PostcombatMain);
    assert!(tokens(&t, P1).is_empty());
}

#[test]
fn defensive_formation_the_defender_makes_trample_assignments() {
    cr!("510.1c", "510.1d", "702.19b");
    ruling!(
        "Defensive Formation",
        "Damage assignment choices from Trample and other things which allow alternative damage assignments for attacking creatures are all made by you."
    );
    supported("Defensive Formation");
    // P0's Colossal Dreadmaw (6/6 trample) is blocked by P1's Grizzly Bears; P1 assigns
    // its damage: all 6 to the Bears, or 2 to them and 4 to P1 (beyond lethal). P0 isn't
    // asked.
    for (split, life) in [([6i64, 0], 20), ([2, 4], 16), ([1, 5], 20)] {
        let mut t = TestGame::new(2);
        t.battlefield(P1, "Defensive Formation");
        let wurm = t.battlefield(P0, "Colossal Dreadmaw");
        let bears = t.battlefield(P1, "Grizzly Bears");
        to_blockers(&mut t, &[(wurm, Entity::Player(P1))], &[(bears, wurm)]);
        t.answer(
            P1,
            DecisionKind::Damage,
            mtg_engine::decision::Answer::Numbers(split.to_vec()),
        );
        let from = t.asked().len();
        go_to(&mut t, Step::EndOfCombat);
        let asked = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::AssignCombatDamage { .. }))
            .map(|(p, _)| *p)
            .collect::<Vec<_>>();
        assert_eq!(asked, vec![P1]);
        // [1, 5] isn't legal (the Bears need lethal damage first): the default is used
        // instead, which keeps it all on the Bears.
        assert_eq!(t.life(P1), life, "{split:?}");
        assert!(!t.on_battlefield(bears));
    }
}

/// The Nemesis Trap target: P0's attacker, from `make`; P1 casts Nemesis Trap at it.
/// Returns the game and the token copy P1 creates.
fn nemesis_trap(make: fn(&mut TestGame) -> ObjectId) -> (TestGame, Option<ObjectId>) {
    let mut t = TestGame::new(2);
    let a = make(&mut t);
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    t.resolve_all();
    cast_card(&mut t, P1, "Nemesis Trap", &[obj(a)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    let tok = tokens(&t, P1).first().copied();
    (t, tok)
}

#[test]
fn nemesis_trap_token_copies_the_exiled_creatures_copiable_values() {
    cr!("707.2", "707.3", "111.3", "107.3m", "603.6a", "614.1c");
    ruling!(
        "Nemesis Trap",
        "Any \"enters\" abilities of the exiled creature will trigger when the token is put onto the battlefield."
    );
    ruling!(
        "Nemesis Trap",
        "If the exiled creature has {X} in its mana cost (such as Protean Hydra), X is considered to be zero."
    );
    ruling!(
        "Nemesis Trap",
        "If the exiled creature was a token, your token copies the original characteristics of that token as stated by the effect that put it onto the battlefield."
    );
    ruling!(
        "Nemesis Trap",
        "If the exiled creature was copying something else (for example, if it was a Jwari Shapeshifter), then your token enters as a copy of whatever the exiled creature was copying."
    );
    supported("Nemesis Trap");
    // An "enters" trigger: Elvish Visionary's draw is P1's.
    let hand = {
        let (t, tok) = nemesis_trap(|t| t.battlefield(P0, "Elvish Visionary"));
        assert!(tok.is_some());
        t.hand_size(P1)
    };
    assert_eq!(hand, 1, "P1 drew for the copy's enters trigger");
    // Protean Hydra ("enters with X +1/+1 counters"): X is 0, so the 0/0 dies.
    let (t, tok) = nemesis_trap(|t| {
        let h = t.battlefield(P0, "Protean Hydra");
        crate::r_s13_common::add(t, h, "+1/+1", 3);
        h
    });
    let tok = tok.map(|x| t.on_battlefield(x));
    assert!(tok.is_none() || tok == Some(false), "the 0/0 copy died");
    // A Soldier token with a +1/+1 counter: a 1/1 Soldier.
    let (t, tok) = nemesis_trap(|t| {
        let s = crate::r_s02_common::create_token(t, P0, "Soldier");
        let s = t.g.current(s);
        t.g.objects[s.0 as usize].summoning_sick = false;
        crate::r_s13_common::add(t, s, "+1/+1", 1);
        s
    });
    let tok = tok.unwrap();
    assert_eq!(t.pt(tok), (1, 1));
    assert!(t.obj_now(tok).chars.has_subtype("Soldier"));
    // A Clone copying a Hill Giant: a Hill Giant.
    let (t, tok) = nemesis_trap(|t| {
        t.battlefield(P1, "Hill Giant");
        let giant = t.named_on_battlefield("Hill Giant")[0];
        t.answer_choose(P0, &[obj(giant)]);
        t.answer_yes(P0, true);
        let clone = t.enter(P0, "Clone");
        t.resolve_all();
        let clone = t.g.current(clone);
        t.g.objects[clone.0 as usize].summoning_sick = false;
        assert_eq!(t.obj_now(clone).chars.name.as_str(), "Hill Giant");
        clone
    });
    let tok = tok.unwrap();
    assert_eq!(t.obj_now(tok).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(tok), (3, 3));
}

#[test]
fn banding_defender_may_assign_a_tramplers_excess_damage_to_themselves() {
    cr!("702.22j", "702.19b");
    // The same choice when the defending player assigns because of banding: Benalish
    // Hero (banding, 1/1) blocks Colossal Dreadmaw; P1 may put the excess on P1.
    let mut t = TestGame::new(2);
    let maw = t.battlefield(P0, "Colossal Dreadmaw");
    let hero = t.battlefield(P1, "Benalish Hero");
    to_blockers(&mut t, &[(maw, Entity::Player(P1))], &[(hero, maw)]);
    t.answer(
        P1,
        DecisionKind::Damage,
        mtg_engine::decision::Answer::Numbers(vec![1, 5]),
    );
    go_to(&mut t, Step::EndOfCombat);
    assert!(!t.on_battlefield(hero));
    assert_eq!(t.life(P1), 15);
}
