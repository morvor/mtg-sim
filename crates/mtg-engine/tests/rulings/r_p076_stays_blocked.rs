//! Rulings batch P076 — "can't be blocked" abilities activated (or gained) after the
//! creature has become blocked don't make it unblocked (CR 509.1h, 506.4).

use crate::r_p076_common::*;
use crate::r_s01_common::supported;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Activates `source`'s `idx`th activated ability targeting `target` after `attacker`
/// is blocked, with `pool` mana of each listed type, and checks it stays blocked.
fn activate_after_block(
    source_name: &str,
    attacker_name: Option<&str>,
    idx: usize,
    pool: &[(ManaType, u32)],
) {
    supported(source_name);
    let mut t = TestGame::new(2);
    let source = t.battlefield(P0, source_name);
    let attacker = match attacker_name {
        Some(n) => t.battlefield(P0, n),
        None => source,
    };
    let blocker = t.battlefield(P1, "Grizzly Bears");
    for (ty, n) in pool {
        mana(&mut t, P0, *ty, *n);
    }
    unblockable_after_blocked(&mut t, attacker, blocker, |t| {
        for (ty, n) in pool {
            mana(t, P0, *ty, *n);
        }
        t.activate(P0, source, idx, &[Entity::Object(attacker)])
            .unwrap_or_else(|e| panic!("{source_name}: {e:?}"));
    });
}

#[test]
fn coralhelm_guide_after_blocks() {
    cr!("509.1h", "506.4");
    ruling!(
        "Coralhelm Guide",
        "Activating Coralhelm Guide's ability after a creature has become blocked won't cause that creature to become unblocked."
    );
    activate_after_block("Coralhelm Guide", Some("Hill Giant"), 0, &[(ManaType::U, 5)]);
}

#[test]
fn corsairs_of_umbar_after_blocks() {
    cr!("509.1h", "506.4");
    ruling!(
        "Corsairs of Umbar",
        "Activating Corsairs of Umbar's first ability after a Goblin, Orc, or Pirate has become blocked won't cause that creature to become unblocked."
    );
    activate_after_block("Corsairs of Umbar", None, 0, &[(ManaType::U, 3)]);
}

#[test]
fn manifold_key_after_blocks() {
    cr!("509.1h", "506.4");
    ruling!(
        "Manifold Key",
        "Activating Manifold Key's last ability after a creature has become blocked won't cause that creature to become unblocked."
    );
    activate_after_block("Manifold Key", Some("Hill Giant"), 1, &[(ManaType::C, 3)]);
}

#[test]
fn passwall_adept_after_blocks() {
    cr!("509.1h", "506.4");
    ruling!(
        "Passwall Adept",
        "Activating Passwall Adept’s ability after a creature has become blocked won’t cause that creature to become unblocked."
    );
    activate_after_block("Passwall Adept", Some("Hill Giant"), 0, &[(ManaType::U, 3)]);
}

#[test]
fn suspicious_bookcase_after_blocks() {
    cr!("509.1h", "506.4");
    ruling!(
        "Suspicious Bookcase",
        "Activating Suspicious Bookcase's ability after a creature has become blocked won't cause that creature to become unblocked."
    );
    activate_after_block("Suspicious Bookcase", Some("Hill Giant"), 0, &[(ManaType::C, 3)]);
}

#[test]
fn amphin_pathmage_after_blocks() {
    cr!("509.1h", "506.4");
    ruling!(
        "Amphin Pathmage",
        "Activating the ability after blockers have been declared won’t have any effect. It won’t change or undo any blocks."
    );
    activate_after_block("Amphin Pathmage", Some("Hill Giant"), 0, &[(ManaType::U, 3)]);
}

#[test]
fn aquatic_incursion_after_blocks() {
    cr!("509.1h", "506.4");
    ruling!(
        "Aquatic Incursion",
        "Activating the last ability of Aquatic Incursion after a Merfolk has become blocked won’t cause it to become unblocked."
    );
    activate_after_block(
        "Aquatic Incursion",
        Some("Merfolk of the Pearl Trident"),
        0,
        &[(ManaType::U, 4)],
    );
}

#[test]
fn rogues_passage_after_blocks() {
    cr!("509.1h", "506.4");
    ruling!(
        "Rogue's Passage",
        "Activating the second ability of Rogue's Passage after a creature has become blocked won't cause that creature to become unblocked."
    );
    activate_after_block("Rogue's Passage", Some("Hill Giant"), 1, &[(ManaType::C, 4)]);
}

#[test]
fn elvenkings_harper_after_blocks() {
    cr!("509.1h", "506.4");
    ruling!(
        "Elvenking's Harper",
        "Once a creature has been blocked, activating the ability of Elvenking's Harper won't cause that creature to become unblocked."
    );
    activate_after_block("Elvenking's Harper", Some("Hill Giant"), 0, &[(ManaType::U, 5)]);
}

#[test]
fn whirler_rogue_after_blocks() {
    cr!("509.1h", "506.4");
    ruling!(
        "Whirler Rogue",
        "Activating the second ability of Whirler Rogue after a creature has become blocked won't cause it to become unblocked."
    );
    supported("Whirler Rogue");
    let mut t = TestGame::new(2);
    let rogue = t.battlefield(P0, "Whirler Rogue");
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, giant, bears, |t| {
        t.activate(P0, rogue, 0, &[Entity::Object(giant)]).unwrap();
    });
}

#[test]
fn key_to_the_city_after_blocks() {
    cr!("509.1h", "506.4");
    ruling!(
        "Key to the City",
        "Activating Key to the City's ability targeting a creature that has already been blocked won't cause it to become unblocked."
    );
    supported("Key to the City");
    let mut t = TestGame::new(2);
    let key = t.battlefield(P0, "Key to the City");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P0, "Island");
    unblockable_after_blocked(&mut t, giant, bears, |t| {
        t.activate(P0, key, 0, &[Entity::Object(giant)]).unwrap();
    });
    assert!(t.in_graveyard(P0, "Island"));
}

#[test]
fn merfolk_sovereign_before_and_after_blocks() {
    cr!("509.1h", "506.4", "509.1b");
    ruling!(
        "Merfolk Sovereign",
        "To have any effect, Merfolk Sovereign's activated ability must be activated before the declare blockers step begins. Once a Merfolk has become blocked, activating Merfolk Sovereign's ability won't change that."
    );
    supported("Merfolk Sovereign");
    // After blocks: no effect.
    let mut t = TestGame::new(2);
    let sov = t.battlefield(P0, "Merfolk Sovereign");
    let merfolk = t.battlefield(P0, "Coralhelm Guide");
    let bears = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, merfolk, bears, |t| {
        t.activate(P0, sov, 0, &[Entity::Object(merfolk)]).unwrap();
    });
    // Before blocks (in the declare attackers step): the Merfolk can't be blocked.
    let mut t = TestGame::new(2);
    let sov = t.battlefield(P0, "Merfolk Sovereign");
    let merfolk = t.battlefield(P0, "Coralhelm Guide");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s01_common::attack_with(&mut t, &[(merfolk, Entity::Player(P1))]);
    t.activate(P0, sov, 0, &[Entity::Object(merfolk)]).unwrap();
    t.resolve_all();
    assert!(!crate::r_s21_common::legal_blocks(
        &mut t,
        P1,
        &[(bears, merfolk)]
    ));
}

#[test]
fn silver_shroud_costume_attached_after_blocks() {
    cr!("509.1h", "506.4", "301.5");
    ruling!(
        "Silver Shroud Costume",
        "Attaching Silver Shroud Costume to a creature that has already been blocked won't cause it to become unblocked."
    );
    supported("Silver Shroud Costume");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, giant, bears, |t| {
        // Flash: cast during the declare blockers step; its enters trigger attaches it.
        crate::r_s25_common::cast_new(t, P0, "Silver Shroud Costume", &[]);
        t.resolve();
        t.answer_targets(P0, &[Entity::Object(giant)]);
        t.resolve_all();
        let costume = t.named_on_battlefield("Silver Shroud Costume")[0];
        assert_eq!(
            t.obj_now(costume).attached_to,
            Some(Entity::Object(t.g.current(giant)))
        );
    });
}

#[test]
fn detective_of_the_month_blessing_after_blocks() {
    cr!("509.1h", "506.4", "702.131c");
    ruling!(
        "Detective of the Month",
        "Gaining the city’s blessing after a Detective you control has become blocked won’t cause that Detective to become unblocked."
    );
    supported("Detective of the Month");
    let mut t = TestGame::new(2);
    let det = t.battlefield(P0, "Detective of the Month");
    let bears = t.battlefield(P1, "Grizzly Bears");
    unblockable_after_blocked(&mut t, det, bears, |t| {
        t.lands(P0, "Island", 9);
        t.g.recompute();
        t.settle();
        assert!(t.g.player(P0).has_citys_blessing);
    });
}
