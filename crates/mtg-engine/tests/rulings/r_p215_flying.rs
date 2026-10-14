//! Rulings batch P215 — flying (CR 702.9): abilities that care about creatures with
//! flying, gaining and losing flying, and when that is checked.

use crate::r_p215_fight::grant;
use crate::r_p215_flash::lose;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s05_common::*;
use crate::r_s06_common::{activate_containing, damage};
use crate::r_s07_common::*;
use crate::r_s21_common::legal_blocks;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{counters, CardType, Color};
use mtg_engine::*;

const FLYING: KeywordKind = KeywordKind::Flying;

fn flies(t: &mut TestGame, id: ObjectId) -> bool {
    t.g.recompute();
    t.obj_now(id).has_keyword(FLYING)
}

fn give_flying(t: &mut TestGame, id: ObjectId) {
    let c = t.obj_now(id).controller;
    grant(t, c, id, Keyword::new(FLYING));
}

/// P0 attacks P1 with `attackers` and resolves the attack triggers (declare attackers
/// step).
fn attack_resolve(t: &mut TestGame, attackers: &[ObjectId]) {
    let a: Vec<_> = attackers
        .iter()
        .map(|id| (*id, Entity::Player(P1)))
        .collect();
    attack_with(t, &a);
    t.resolve_all();
}

#[test]
fn lords_leaving_can_make_marked_damage_lethal() {
    cr!("120.6", "704.5g", "611.3a");
    ruling!(
        "Empyrean Eagle",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to other creatures you control with flying may become lethal if Empyrean Eagle leaves the battlefield during that turn."
    );
    ruling!(
        "Spirit of the Spires",
        "Because damage remains marked on a creature until it’s removed as the turn ends, nonlethal damage dealt to other creatures you control with flying may become lethal if Spirit of the Spires leaves the battlefield during that turn."
    );
    ruling!(
        "Sephara, Sky's Blade",
        "Because damage remains marked on a creature until it's removed as the turn ends, lethal damage dealt to other creatures you control with flying may cause them to be destroyed if Sephara leaves the battlefield during that turn."
    );
    // (lord, damage dealt to a 1/2 Storm Crow)
    for (lord, n) in [
        ("Empyrean Eagle", 2),
        ("Spirit of the Spires", 2),
        ("Sephara, Sky's Blade", 3),
    ] {
        supported(lord);
        let mut t = TestGame::new(2);
        let l = t.battlefield(P0, lord);
        let crow = t.battlefield(P0, "Storm Crow");
        let src = t.battlefield(P1, "Grizzly Bears");
        damage(&mut t, src, n, crow);
        assert!(t.on_battlefield(crow), "{lord}");
        destroy(&mut t, l);
        assert!(!t.on_battlefield(crow), "{lord}");
    }
    ruling!(
        "Skycat Sovereign",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Skycat Sovereign may become lethal if other creatures with flying you control leave the battlefield during that turn."
    );
    supported("Skycat Sovereign");
    let mut t = TestGame::new(2);
    let cat = t.battlefield(P0, "Skycat Sovereign");
    let crow = t.battlefield(P0, "Storm Crow");
    assert_eq!(t.pt(cat), (2, 2));
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 1, cat);
    assert!(t.on_battlefield(cat));
    destroy(&mut t, crow);
    assert!(!t.on_battlefield(cat));
}

#[test]
fn fear_of_falling_target_can_gain_flying_again() {
    cr!("613.7", "613.1f", "611.2a");
    ruling!(
        "Fear of Falling",
        "A creature that loses flying due to the effect of Fear of Falling's last ability can still gain flying from the effect of another spell or ability, such as Jump Scare."
    );
    supported("Fear of Falling");
    let mut t = TestGame::new(2);
    let fear = t.battlefield(P0, "Fear of Falling");
    let crow = t.battlefield(P1, "Storm Crow");
    t.answer_targets(P0, &[Entity::Object(crow)]);
    attack_resolve(&mut t, &[fear]);
    assert!(!flies(&mut t, crow));
    assert_eq!(t.pt(crow), (-1, 2));
    give_flying(&mut t, crow);
    assert!(flies(&mut t, crow));
}

#[test]
fn final_iteration_pumps_every_wizard_you_control() {
    cr!("613.1f", "613.4c");
    ruling!(
        "Docent of Perfection // Final Iteration",
        "All Wizards you control get +2/+1 and have flying, not just those created by Docent of Perfection or Final Iteration."
    );
    supported("Docent of Perfection // Final Iteration");
    let mut t = TestGame::new(2);
    crate::r_s17_common::enter_transformed(&mut t, P0, "Docent of Perfection // Final Iteration");
    let wiz = t.battlefield(P0, "Prodigal Sorcerer");
    assert_eq!(t.pt(wiz), (3, 2));
    assert!(flies(&mut t, wiz));
    let theirs = t.battlefield(P1, "Prodigal Sorcerer");
    assert_eq!(t.pt(theirs), (1, 1));
}

#[test]
fn harbinger_and_scourge_that_lost_flying_damage_themselves() {
    cr!("611.3a", "120.3");
    ruling!(
        "Harbinger of the Hunt",
        "If Harbinger of the Hunt loses flying, its first activated ability will cause it to deal damage to itself."
    );
    ruling!(
        "Scourge of Kher Ridges",
        "If Scourge of Kher Ridges has lost flying, the first activated ability will cause it to deal damage to itself."
    );
    for (card, land, dmg) in [
        ("Harbinger of the Hunt", "Mountain", 1),
        ("Scourge of Kher Ridges", "Mountain", 2),
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        let d = t.battlefield(P0, card);
        let bears = t.battlefield(P1, "Giant Spider");
        t.lands(P0, land, 3);
        // With flying: only the Bears.
        activate_containing(&mut t, P0, d, "without flying").expect("activate");
        t.resolve_all();
        assert_eq!(damage_on(&t, d), 0, "{card}");
        assert_eq!(damage_on(&t, bears), dmg, "{card}");
        lose(&mut t, d, FLYING);
        t.lands(P0, land, 3);
        activate_containing(&mut t, P0, d, "without flying").expect("activate");
        t.resolve_all();
        assert_eq!(damage_on(&t, d), dmg, "{card}");
    }
}

#[test]
fn jackdaw_savior_triggers_for_each_flyer_dying_with_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Jackdaw Savior",
        "If Jackdaw Savior and another creature you control with flying die at the same time, Jackdaw Savior’s last ability triggers for each of them."
    );
    supported("Jackdaw Savior");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jackdaw Savior");
    t.battlefield(P0, "Storm Crow");
    t.graveyard(P0, "Llanowar Elves");
    t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve();
    // One trigger for Jackdaw Savior, one for the Storm Crow.
    assert_eq!(t.stack_len(), 2);
}

fn moth_game() -> (TestGame, ObjectId) {
    let mut t = TestGame::new(2);
    let moth = t.battlefield(P0, "Luminous Broodmoth");
    (t, moth)
}

#[test]
fn luminous_broodmoth_dying_triggers() {
    cr!("603.10a", "603.6c", "111.7");
    ruling!(
        "Luminous Broodmoth",
        "If Luminous Broodmoth dies after losing flying but not losing its triggered ability, its ability will trigger for itself."
    );
    ruling!(
        "Luminous Broodmoth",
        "If Luminous Broodmoth dies at the same time as a creature without flying, its ability triggers for that creature."
    );
    ruling!(
        "Luminous Broodmoth",
        "Luminous Broodmoth's ability triggers if a token creature without flying dies, but the token won't be returned to the battlefield. If that token was a merged creature, any cards that were part of it will be returned to the battlefield."
    );
    supported("Luminous Broodmoth");
    // Lost flying: it returns itself.
    let (mut t, moth) = moth_game();
    lose(&mut t, moth, FLYING);
    destroy(&mut t, moth);
    t.resolve_all();
    let back = t.named_on_battlefield("Luminous Broodmoth");
    assert_eq!(back.len(), 1);
    assert_eq!(t.counters(back[0], "flying"), 1);
    // Dying with a creature without flying: that creature returns.
    let (mut t, _) = moth_game();
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(bears.len(), 1);
    assert!(t.named_on_battlefield("Luminous Broodmoth").is_empty());
    // A token: it triggers, but nothing returns.
    let (mut t, _) = moth_game();
    let token = create_token(&mut t, P0, "Soldier");
    destroy(&mut t, token);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(tokens_with_subtype(&t, P0, "Soldier").is_empty());
}

#[test]
fn starry_eyed_skyrider_leaving_lets_tokens_be_blocked() {
    cr!("611.3a", "702.9b", "509.1b");
    ruling!(
        "Starry-Eyed Skyrider",
        "If Starry-Eyed Skyrider leaves the battlefield, any attacking tokens will no longer be granted flying by the last ability. If that happens before the declare blockers step, it may allow those tokens to be blocked by creatures without flying."
    );
    supported("Starry-Eyed Skyrider");
    let mut t = TestGame::new(2);
    let sky = t.battlefield(P0, "Starry-Eyed Skyrider");
    let token = create_token(&mut t, P0, "Soldier");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    attack_with(&mut t, &[(token, Entity::Player(P1))]);
    t.resolve_all();
    assert!(flies(&mut t, token));
    assert!(!legal_blocks(&mut t, P1, &[(bears, token)]));
    let _ = sky;
    destroy(&mut t, sky);
    assert!(!flies(&mut t, token));
    assert!(legal_blocks(&mut t, P1, &[(bears, token)]));
}

#[test]
fn soulcatcher_dealt_lethal_damage_with_another_flyer_dies_anyway() {
    cr!("704.3", "704.5g", "603.10a");
    ruling!(
        "Soulcatcher",
        "If another creature with flying is dealt lethal damage at the same time as Soulcatcher, they’ll be destroyed at the same time. Soulcatcher’s ability won’t save it."
    );
    supported("Soulcatcher");
    let mut t = TestGame::new(2);
    let sc = t.battlefield(P0, "Soulcatcher");
    let crow = t.battlefield(P0, "Storm Crow");
    t.lands(P0, "Mountain", 2);
    // Pyroclasm: 2 damage to each creature.
    let p = t.hand(P0, "Pyroclasm");
    t.cast(P0, p).go();
    t.resolve_all();
    assert!(!t.on_battlefield(sc));
    assert!(!t.on_battlefield(crow));
}

#[test]
fn shadow_puppeteers_creature_that_lost_flying_still_becomes_a_dragon() {
    cr!("608.2h", "613.1d", "613.4b");
    ruling!(
        "Shadow Puppeteers",
        "If the creature loses flying before Shadow Puppeteers's last ability resolves, the creature will still become a red Dragon with base power and toughness 4/4 in addition to its other colors and types until end of turn. It won't gain flying."
    );
    supported("Shadow Puppeteers");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shadow Puppeteers");
    let crow = t.battlefield(P0, "Storm Crow");
    attack_with(&mut t, &[(crow, Entity::Player(P1))]);
    lose(&mut t, crow, FLYING);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.pt(crow), (4, 4));
    assert!(t.obj_now(crow).chars.has_subtype("Dragon"));
    assert!(t.obj_now(crow).chars.colors.contains(Color::Red));
    assert!(t.obj_now(crow).chars.colors.contains(Color::Blue));
    assert!(!flies(&mut t, crow));
}

#[test]
fn aven_mimeomancer_feather_counters() {
    cr!("613.4b", "613.1f", "122.1");
    ruling!(
        "Aven Mimeomancer",
        "If the second ability affects a creature, it doesn't cause that creature to lose any of its abilities. It just gives that creature flying as well."
    );
    ruling!(
        "Aven Mimeomancer",
        "You may put a feather counter on a creature that already has a feather counter on it. Doing so has no visible effect unless some other effect has changed the creature's power or toughness, or caused it to lose flying, since the last feather counter was put on it."
    );
    supported("Aven Mimeomancer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Aven Mimeomancer");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let upkeep = |t: &mut TestGame| {
        t.answer_yes(P0, true);
        t.answer_targets(P0, &[Entity::Object(elves)]);
        crate::r_s04_common::next_upkeep(t, P0);
        t.resolve_all();
    };
    upkeep(&mut t);
    assert_eq!(t.counters(elves, "feather"), 1);
    assert_eq!(t.pt(elves), (3, 1));
    assert!(flies(&mut t, elves));
    // It keeps its mana ability.
    assert!(t
        .obj_now(elves)
        .chars
        .abilities
        .iter()
        .any(|a| matches!(a.kind, AbilityKind::Activated(_))));
    // It loses flying (indefinitely); a second feather counter gives it back.
    run_from(
        &mut t,
        P1,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveKeyword(FLYING)],
            duration: Duration::Permanent,
        },
        &[Entity::Object(elves)],
    );
    assert!(!flies(&mut t, elves));
    upkeep(&mut t);
    assert_eq!(t.counters(elves, "feather"), 2);
    assert!(flies(&mut t, elves));
    assert_eq!(t.pt(elves), (3, 1));
}
