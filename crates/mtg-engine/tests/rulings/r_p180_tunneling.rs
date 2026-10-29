//! Rulings batch P180 — "target creature with power N or less can't be blocked this
//! turn" (Goblin Tunneler and friends): the power is checked as the ability is activated
//! and as it resolves, but not afterwards; a creature that's already blocked stays
//! blocked.

use crate::r_p180_common::*;
use crate::r_s06_common::activate_containing;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// The tunneling cards: (name, the power limit). Their ability's text contains "can't be
/// blocked"; mana for it comes from Wastes and a Mountain.
const TUNNELERS: [(&str, i32); 11] = [
    ("Pathmaker Initiate", 2),
    ("Break Through the Line", 2),
    ("Goblin Smuggler", 2),
    ("Access Tunnel", 3),
    ("Deserter's Disciple", 2),
    ("Sneaking Guide", 2),
    ("Underfoot Underdogs", 2),
    ("Crafty Pathmage", 2),
    ("Dwarven Warriors", 2),
    ("Ragged Playmate", 2),
    ("Goblin Tunneler", 2),
];

/// A game in P0's beginning of combat with the tunneling permanent `name`, mana for its
/// ability, P0's Grizzly Bears and P1's Hill Giant. Returns (source, bears, giant).
fn setup(name: &str) -> (TestGame, ObjectId, ObjectId, ObjectId) {
    supported(name);
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let src = t.battlefield(P0, name);
    t.lands(P0, "Wastes", 3);
    t.lands(P0, "Mountain", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    (t, src, bears, giant)
}

fn tunnel(t: &mut TestGame, src: ObjectId, bears: ObjectId) {
    t.answer_targets(P0, &[obj(bears)]);
    activate_containing(t, P0, src, "can't be blocked").expect("activated");
}

#[test]
fn tunneling_checks_power_on_resolution_but_not_afterwards() {
    cr!("608.2b", "611.2c");
    ruling!(
        "Pathmaker Initiate",
        "If the creature's power is greater than 2 as the activated ability tries to resolve, the ability won't resolve"
    );
    ruling!(
        "Break Through the Line",
        "If the creature’s power is greater than 2 as the activated ability tries to resolve, the ability won’t resolve"
    );
    ruling!(
        "Goblin Smuggler",
        "If the target creature's power is greater than 2 as the activated ability tries to resolve, the ability doesn't resolve."
    );
    ruling!(
        "Access Tunnel",
        "If the target creature's power is greater than 3 as Access Tunnel's second ability tries to resolve"
    );
    ruling!(
        "Deserter's Disciple",
        "If the target creature's power is increased to 3 or greater after Deserter's Disciple's ability is activated but before it resolves"
    );
    ruling!(
        "Sneaking Guide",
        "If the target creature’s power is greater than 2 as Sneaking Guide’s ability tries to resolve"
    );
    ruling!(
        "Underfoot Underdogs",
        "If the target creature’s power is increased above 2 before the ability resolves, it won’t be a legal target"
    );
    ruling!(
        "Crafty Pathmage",
        "If you increase the power of the targeted creature after the ability resolves, it still can't be blocked that turn."
    );
    ruling!(
        "Dwarven Warriors",
        "If you increase the power of the targeted creature after the ability resolves, it still can’t be blocked that turn."
    );
    ruling!(
        "Ragged Playmate",
        "Once Ragged Playmate's ability has resolved, raising the affected creature's power above 2 won't cause it to be able to be blocked this turn."
    );
    ruling!(
        "Goblin Tunneler",
        "The power of the targeted creature is checked both as you target it and as the ability resolves."
    );
    for (name, limit) in TUNNELERS {
        // A creature with too much power can't be targeted.
        let (mut t, src, _, giant) = setup(name);
        let big = t.battlefield(P0, "Hill Giant");
        if limit == 2 {
            let i = ability_index(&t, src);
            assert!(
                !crate::r_s04_common::ability_targets(&mut t, src, i).contains(&obj(big)),
                "{name}"
            );
        }
        let _ = giant;
        // Pumped in response: the ability doesn't resolve.
        let (mut t, src, bears, giant) = setup(name);
        tunnel(&mut t, src, bears);
        cast_targeting(&mut t, P0, "Giant Growth", &[obj(bears)]);
        t.resolve_all();
        assert_eq!(t.pt(bears), (5, 5));
        crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        assert!(can_block(&mut t, giant, bears), "{name}: the ability did nothing");
        if name == "Break Through the Line" {
            assert!(!t.obj_now(bears).has_keyword(KeywordKind::Haste), "{name}");
        }
        // Pumped after it resolved: still can't be blocked (and keeps haste).
        let (mut t, src, bears, giant) = setup(name);
        tunnel(&mut t, src, bears);
        t.resolve_all();
        cast_targeting(&mut t, P0, "Giant Growth", &[obj(bears)]);
        t.resolve_all();
        assert_eq!(t.pt(bears), (5, 5));
        crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        assert!(!can_block(&mut t, giant, bears), "{name}: still can't be blocked");
        if name == "Break Through the Line" {
            assert!(t.obj_now(bears).has_keyword(KeywordKind::Haste), "{name}");
        }
    }
}

/// The index of `src`'s activated ability containing "can't be blocked" among its
/// activated abilities.
fn ability_index(t: &TestGame, src: ObjectId) -> usize {
    use mtg_engine::ability::AbilityKind;
    t.obj_now(src)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .position(|a| a.text.contains("can't be blocked"))
        .expect("no tunneling ability")
}

#[test]
fn tunneling_doesnt_unblock_a_blocked_creature() {
    cr!("509.1h");
    ruling!(
        "Deserter's Disciple",
        "Once a creature with power 2 or less has been blocked, Deserter's Disciple's ability won't cause it to become unblocked."
    );
    ruling!(
        "Sneaking Guide",
        "Once a creature with power 2 or less has been blocked, Sneaking Guide’s ability won’t cause it to become unblocked."
    );
    ruling!(
        "Ragged Playmate",
        "Once a creature with power 2 or less has been blocked, activating Ragged Playmate's ability targeting that creature won't cause it to stop being blocked"
    );
    ruling!(
        "Access Tunnel",
        "Once a creature with power 3 or less has been blocked, Access Tunnel's second ability won't cause it to become unblocked."
    );
    ruling!(
        "Dwarven Warriors",
        "The ability can be activated after a creature is blocked, but it has no effect. Once a creature is blocked, it can’t be unblocked."
    );
    for name in [
        "Deserter's Disciple",
        "Sneaking Guide",
        "Ragged Playmate",
        "Access Tunnel",
        "Dwarven Warriors",
    ] {
        let (mut t, src, bears, _) = setup(name);
        let wall = t.battlefield(P1, "Wall of Wood");
        crate::r_s03_common::to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(wall, bears)]);
        tunnel(&mut t, src, bears);
        t.resolve_all();
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20, "{name}: still blocked");
        assert_eq!(damage_on(&t, wall), 2, "{name}");
    }
}

#[test]
fn goblin_tunnelers_effect_isnt_an_ability_the_creature_can_lose() {
    cr!("611.2c", "613.1f");
    ruling!(
        "Goblin Tunneler",
        "The ability doesn’t grant an ability to the targeted creature."
    );
    let (mut t, src, bears, giant) = setup("Goblin Tunneler");
    tunnel(&mut t, src, bears);
    t.resolve_all();
    // Turn to Frog: it loses all abilities and becomes a 1/1 Frog.
    cast_targeting(&mut t, P0, "Turn to Frog", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (1, 1));
    crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!can_block(&mut t, giant, bears));
}
