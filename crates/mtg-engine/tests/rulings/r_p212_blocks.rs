//! Rulings batch P212 — Equipment (CR 301.5, 702.6) that changes how the equipped creature
//! can be blocked: block requirements (CR 509.1c), restrictions (CR 509.1b) checked only
//! as blockers are declared (CR 509.1h, 506.4), and combining restrictions.

use crate::r_p212_common::*;
use crate::r_s01_common::*;
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::*;
use crate::r_s10_common::blocking;
use crate::r_s21_common::legal_blocks;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

fn creature(t: &mut TestGame, p: PlayerId, name: &str, type_line: &str, pt: (i32, i32)) -> ObjectId {
    let def = custom_card(name, type_line, "{1}", Some(pt), "");
    t.custom(p, def, Zone::Battlefield)
}

/// "Must be blocked by an Eldrazi / a Dalek if able" with one or two copies of the
/// Equipment: one creature of that type blocking satisfies it; the others are free.
#[test]
fn must_be_blocked_by_a_creature_type_needs_only_one_blocker() {
    cr!("509.1c", "301.5");
    ruling!(
        "Slayer's Cleaver",
        "Only one Eldrazi is required to block the equipped creature. Other Eldrazi and non-Eldrazi creatures may also block that creature, and are free to block other creatures or not block at all."
    );
    ruling!(
        "Slayer's Cleaver",
        "One creature equipped with multiple Slayer’s Cleavers only requires a single Eldrazi to block it."
    );
    ruling!(
        "Ace's Baseball Bat",
        "Only one Dalek is required to block the equipped creature. Other Dalek and non-Dalek creatures may also block that creature and are free to block other creatures or not block at all."
    );
    ruling!(
        "Ace's Baseball Bat",
        "If one creature is somehow equipped with multiple Ace's Baseball Bats, that creature still requires only a single Dalek to block it."
    );
    supported("Slayer's Cleaver");
    supported("Ace's Baseball Bat");
    for (eq, ty) in [("Slayer's Cleaver", "Eldrazi"), ("Ace's Baseball Bat", "Dalek")] {
        for copies in [1, 2] {
            let mut t = TestGame::new(2);
            let bears = t.battlefield(P0, "Grizzly Bears");
            for _ in 0..copies {
                attach_new(&mut t, P0, eq, bears);
            }
            let giant = t.battlefield(P0, "Hill Giant");
            let line = format!("Creature — {ty}");
            let a = creature(&mut t, P1, "A", &line, (1, 1));
            let b = creature(&mut t, P1, "B", &line, (1, 1));
            let c = creature(&mut t, P1, "C", "Creature — Human", (1, 1));
            let both = Entity::Player(P1);
            attack_with(&mut t, &[(bears, both), (giant, both)]);
            let ctx = format!("{eq} x{copies}");
            assert!(!legal_blocks(&mut t, P1, &[]), "{ctx}");
            assert!(!legal_blocks(&mut t, P1, &[(c, bears)]), "{ctx}");
            assert!(!legal_blocks(&mut t, P1, &[(a, giant)]), "{ctx}");
            assert!(legal_blocks(&mut t, P1, &[(a, bears)]), "{ctx}");
            assert!(legal_blocks(&mut t, P1, &[(b, bears)]), "{ctx}");
            assert!(legal_blocks(&mut t, P1, &[(a, bears), (b, bears), (c, bears)]), "{ctx}");
            assert!(legal_blocks(&mut t, P1, &[(a, bears), (b, giant), (c, giant)]), "{ctx}");
        }
    }
}

/// The requirement can't be obeyed by a tapped creature or one that can't block, and a
/// player isn't forced to pay a cost to block.
#[test]
fn must_be_blocked_by_a_creature_type_doesnt_force_costs_or_impossible_blocks() {
    cr!("509.1c", "509.1d");
    ruling!(
        "Slayer's Cleaver",
        "If each Eldrazi that could block the equipped creature is tapped or is affected by a spell or ability that says it can’t block, then none of them block. If there’s a cost associated with having each of those creatures block, its controller isn’t forced to pay the cost, so they don’t have to block in that case either."
    );
    ruling!(
        "Ace's Baseball Bat",
        "If each Dalek that could block the equipped creature is tapped or is affected by a spell or ability that says it can't block, then none of them block. If there's a cost associated with having each of those creatures block, its controller isn't forced to pay the cost, so they don't have to block in that case either."
    );
    for (eq, ty) in [("Slayer's Cleaver", "Eldrazi"), ("Ace's Baseball Bat", "Dalek")] {
        let line = format!("Creature — {ty}");
        // Tapped, and under Pacifism ("can't block").
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, eq, bears);
        let a = creature(&mut t, P1, "A", &line, (1, 1));
        let b = creature(&mut t, P1, "B", &line, (1, 1));
        attach_new(&mut t, P1, "Pacifism", b);
        t.g.objects[a.0 as usize].tapped = true;
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        assert!(legal_blocks(&mut t, P1, &[]), "{eq}");
        // Control: an untapped one that can block must.
        let c = creature(&mut t, P1, "C", &line, (1, 1));
        let _ = c;
        assert!(!legal_blocks(&mut t, P1, &[]), "{eq}");
        // A cost to block (Archangel of Tithes attacking): not forced to pay it.
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, eq, bears);
        let angel = t.battlefield(P0, "Archangel of Tithes");
        let a = creature(&mut t, P1, "A", &line, (1, 1));
        t.lands(P1, "Wastes", 2);
        attack_with(
            &mut t,
            &[(bears, Entity::Player(P1)), (angel, Entity::Player(P1))],
        );
        assert!(legal_blocks(&mut t, P1, &[]), "{eq}");
        assert!(legal_blocks(&mut t, P1, &[(a, bears)]), "{eq}");
    }
}

#[test]
fn cant_be_blocked_by_more_than_one_plus_menace_means_unblockable() {
    cr!("702.111b", "509.1b");
    ruling!("Rope", "If the equipped creature also has menace, it can't be blocked at all.");
    ruling!(
        "Wolfrider's Saddle",
        "If the equipped creature has menace, it can't be blocked at all."
    );
    supported("Rope");
    supported("Wolfrider's Saddle");
    for eq in ["Rope", "Wolfrider's Saddle"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, eq, bears);
        let a = t.battlefield(P1, "Llanowar Elves");
        let b = t.battlefield(P1, "Llanowar Elves");
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        // Without menace one blocker is fine, two aren't.
        assert!(legal_blocks(&mut t, P1, &[(a, bears)]), "{eq}");
        assert!(!legal_blocks(&mut t, P1, &[(a, bears), (b, bears)]), "{eq}");
        grant(&mut t, bears, KeywordKind::Menace);
        assert!(!legal_blocks(&mut t, P1, &[(a, bears)]), "{eq}");
        assert!(!legal_blocks(&mut t, P1, &[(a, bears), (b, bears)]), "{eq}");
    }
}

/// Blocks a creature equipped with `eq` with `blocker`, then applies `change` to the
/// blocker: the attacker stays blocked and deals its damage to the blocker.
fn stays_blocked(eq: &str, blocker: (i32, i32), change: fn(&mut TestGame, ObjectId)) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, eq, bears);
    let x = creature(&mut t, P1, "Blocker", "Creature — Human", blocker);
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(x, bears)]);
    assert!(blocking(&t, x), "{eq}");
    change(&mut t, x);
    assert!(blocking(&t, x), "{eq}");
    t.advance_to(P0, mtg_engine::turn::Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "{eq}");
    let hit = !t.on_battlefield(x) || t.obj_now(x).damage > 0;
    assert!(hit, "{eq}: the blocker was dealt the damage");
}

#[test]
fn changing_the_blocker_after_blocks_doesnt_unblock_the_equipped_creature() {
    cr!("509.1h", "506.4");
    ruling!(
        "Hedgewitch's Mask",
        "The blocking restriction is applied only when blockers are declared. Once a creature equipped with Hedgewitch's Mask becomes blocked by a creature with power less than 4, increasing that creature's power won't cause the equipped creature to become unblocked."
    );
    ruling!(
        "Saddle of the Cavalier",
        "Reducing a creature's power after it has blocked a creature equipped with Saddle of the Cavalier will not remove that blocking creature from combat or make the equipped creature unblocked."
    );
    ruling!(
        "Wrecking Ball Arm",
        "Once a creature equipped with Wrecking Ball Arm has been blocked, changing the blocking creature's power to 2 or less won't cause the creature to become unblocked."
    );
    ruling!(
        "Skyblinder Staff",
        "If the equipped creature is blocked by a creature and then that creature gains flying, the equipped creature will remain blocked."
    );
    for c in [
        "Hedgewitch's Mask",
        "Saddle of the Cavalier",
        "Wrecking Ball Arm",
        "Skyblinder Staff",
    ] {
        supported(c);
    }
    stays_blocked("Hedgewitch's Mask", (3, 3), |t, x| boost(t, x, 2, 0));
    stays_blocked("Saddle of the Cavalier", (4, 4), |t, x| boost(t, x, -2, 0));
    stays_blocked("Wrecking Ball Arm", (3, 3), |t, x| boost(t, x, -2, 0));
    stays_blocked("Skyblinder Staff", (1, 1), |t, x| {
        grant(t, x, KeywordKind::Flying)
    });
    // The restrictions themselves, as blockers are declared.
    for (eq, pt, legal) in [
        ("Hedgewitch's Mask", (5, 5), false),
        ("Saddle of the Cavalier", (2, 2), false),
        ("Wrecking Ball Arm", (1, 1), false),
        ("Hedgewitch's Mask", (3, 3), true),
    ] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        attach_new(&mut t, P0, eq, bears);
        let x = creature(&mut t, P1, "Blocker", "Creature — Human", pt);
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        assert_eq!(legal_blocks(&mut t, P1, &[(x, bears)]), legal, "{eq}");
    }
}
