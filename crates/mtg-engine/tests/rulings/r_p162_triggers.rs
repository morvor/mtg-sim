//! Rulings batch P162 — leaves-the-battlefield triggers that include their own source
//! (CR 603.10a), "whenever another creature enters" with creatures entering together
//! (CR 603.6a), last known power of a creature that died, characteristics of an entering
//! creature, attack triggers, and damage triggers.

use crate::r_p162_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, supported, triggers_on_stack};
use crate::r_s02_common::destroy;
use crate::r_s05_common::enter;
use crate::r_s06_common::damage;
use crate::r_s17_common::enter_transformed;
use mtg_engine::decision::Action;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn wrath(t: &mut TestGame) {
    cast_resolve(t, P0, "Wrath of God", &[]);
}

#[test]
fn judith_triggers_for_itself_and_creatures_dying_with_it() {
    cr!("603.10a", "603.6c");
    ruling!(
        "Judith, the Scourge Diva",
        "If Judith dies at the same time as one or more other nontoken creatures you control, Judith's ability triggers for each of them."
    );
    ruling!(
        "Judith, the Scourge Diva",
        "Judith's last ability triggers when Judith dies if it's not a token."
    );
    supported("Judith, the Scourge Diva");
    // Judith alone.
    let mut t = TestGame::new(2);
    let judith = t.battlefield(P0, "Judith, the Scourge Diva");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, judith);
    assert_eq!(triggers_on_stack(&t, "1 damage"), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    // Judith and two nontoken creatures (and a token, which doesn't count) at once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Judith, the Scourge Diva");
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Hill Giant");
    crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    for _ in 0..3 {
        t.answer_targets(P0, &[Entity::Player(P1)]);
    }
    wrath(&mut t);
    // Wrath of God resolved; the three triggers went on the stack and resolved.
    assert_eq!(t.life(P1), 20 - 3);
}

#[test]
fn judith_controller_loses_before_the_trigger_is_put_on_the_stack() {
    cr!("704.3", "704.5a", "704.5g", "117.5");
    ruling!(
        "Judith, the Scourge Diva",
        "If your life total is brought to 0 or less at the same time that nontoken creatures you control are dealt lethal damage, you lose the game before Judith's triggered ability goes on the stack."
    );
    supported("Earthquake");
    // Three players so the game goes on after P0 loses.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Judith, the Scourge Diva");
    t.battlefield(P0, "Grizzly Bears");
    t.g.players[0].life = 2;
    // P1's Earthquake for 3 deals 3 damage to each creature without flying and each
    // player.
    t.set_step(P1, Step::PrecombatMain);
    crate::r_s01_common::give_mana_for(&mut t, P1, "Earthquake");
    t.lands(P1, "Wastes", 3);
    let quake = t.hand(P1, "Earthquake");
    t.cast(P1, quake).x(3).go();
    t.resolve();
    assert!(t.has_lost(P0));
    assert_eq!(triggers_on_stack(&t, "1 damage"), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P2), 17);
}

#[test]
fn polukranos_triggers_for_itself_and_hydras_dying_with_it() {
    cr!("603.10a");
    ruling!(
        "Polukranos Reborn // Polukranos, Engine of Ruin",
        "If Polukranos, Engine of Ruin and other nontoken Hydras you control die at the same time, Polukranos’s triggered ability will trigger once for itself and once for each of those other Hydras."
    );
    supported("Polukranos Reborn // Polukranos, Engine of Ruin");
    let mut t = TestGame::new(2);
    enter_transformed(&mut t, P0, "Polukranos Reborn // Polukranos, Engine of Ruin");
    t.battlefield(P0, "Hydra Broodmaster");
    t.battlefield(P0, "Grizzly Bears");
    wrath(&mut t);
    // Two triggers, two tokens each.
    let hydras = crate::r_s01_common::tokens(&t, P0);
    assert_eq!(hydras.len(), 4);
}

#[test]
fn enters_with_other_creatures_triggers_for_each() {
    cr!("603.6a", "603.2");
    ruling!(
        "Dazzling Angel",
        "If Dazzling Angel enters at the same time as one or more other creatures you control, its last ability triggers for each of those other creatures."
    );
    ruling!(
        "Healer of the Pride",
        "If Healer of the Pride enters under your control at the same time as other creatures, its ability will trigger once for each of those creatures."
    );
    ruling!(
        "Hinterland Sanctifier",
        "If Hinterland Sanctifier enters at the same time as one or more other creatures you control, its ability will trigger for each of those other creatures."
    );
    supported("Living Death");
    for (card, per) in [
        ("Dazzling Angel", 1),
        ("Healer of the Pride", 2),
        ("Hinterland Sanctifier", 1),
    ] {
        supported(card);
        let mut t = TestGame::new(2);
        t.graveyard(P0, card);
        t.graveyard(P0, "Grizzly Bears");
        t.graveyard(P0, "Hill Giant");
        cast_resolve(&mut t, P0, "Living Death", &[]);
        assert_eq!(t.named_on_battlefield(card).len(), 1, "{card}");
        assert_eq!(t.life(P0), 20 + 2 * per, "{card}");
    }
}

#[test]
fn infested_thrinax_uses_last_known_power() {
    cr!("608.2h", "603.10a");
    ruling!(
        "Infested Thrinax",
        "Use the power of the creature that died as it last existed on the battlefield to determine how many Saproling tokens to create."
    );
    supported("Infested Thrinax");
    let mut t = TestGame::new(2);
    enter(&mut t, P0, "Infested Thrinax");
    t.resolve_all();
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resolve(&mut t, P0, "Giant Growth", &[Entity::Object(bears)]);
    assert_eq!(t.pt(bears), (5, 5));
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(
        crate::r_s05_common::tokens_with_subtype(&t, P0, "Saproling").len(),
        5
    );
}

#[test]
fn symmetry_matrix_sees_static_abilities_applying_to_the_entering_creature() {
    cr!("603.6a", "603.6d");
    ruling!(
        "Symmetry Matrix",
        "If one or more static abilities that apply to a creature entering the battlefield change its power or toughness, those abilities are considered when determining whether or not Symmetry Matrix's ability triggers."
    );
    supported("Symmetry Matrix");
    // Judith: "Other creatures you control get +1/+0." Vampire Bats (0/1) enters as a 1/1;
    // Grizzly Bears (2/2) enters as a 3/2.
    for (name, triggers) in [("Vampire Bats", 1), ("Grizzly Bears", 0)] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Symmetry Matrix");
        t.battlefield(P0, "Judith, the Scourge Diva");
        enter(&mut t, P0, name);
        assert_eq!(t.stack_len(), triggers, "{name}");
    }
    // Without Judith, Vampire Bats enters as a 0/1: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Symmetry Matrix");
    enter(&mut t, P0, "Vampire Bats");
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn master_symmetrist_checks_only_when_it_attacks() {
    cr!("603.4", "611.2c");
    ruling!(
        "Master Symmetrist",
        "Master Symmetrist’s triggered ability only looks at the power and toughness of a creature when it attacks. Once the ability has triggered, changing the creature’s power or toughness will not stop it from gaining trample or remove trample from it."
    );
    supported("Master Symmetrist");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Master Symmetrist");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(triggers_on_stack(&t, "trample"), 1);
    // Before it resolves the bears become 3/2.
    t.battlefield(P0, "Judith, the Scourge Diva");
    assert_eq!(t.pt(bears), (3, 2));
    t.resolve_all();
    assert!(crate::r_s06_common::has_kw(&t, bears, KeywordKind::Trample));
}

#[test]
fn liliana_s_reaver_creates_the_zombie_with_an_empty_hand() {
    cr!("603.2", "608.2b");
    ruling!(
        "Liliana's Reaver",
        "If the player’s hand is empty when the triggered ability resolves, you’ll still put a Zombie token onto the battlefield."
    );
    supported("Liliana's Reaver");
    let mut t = TestGame::new(2);
    let reaver = t.battlefield(P0, "Liliana's Reaver");
    assert_eq!(t.hand_size(P1), 0);
    attack_with(&mut t, &[(reaver, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    let zombies = crate::r_s05_common::tokens_with_subtype(&t, P0, "Zombie");
    assert_eq!(zombies.len(), 1);
    assert!(t.obj_now(zombies[0]).tapped);
}

#[test]
fn larceny_triggers_once_per_creature() {
    cr!("510.2", "603.2");
    ruling!(
        "Larceny",
        "Triggers once per creature, not once per point of damage."
    );
    supported("Larceny");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Larceny");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..6 {
        t.hand(P1, "Forest");
    }
    attack_with(
        &mut t,
        &[(giant, Entity::Player(P1)), (bears, Entity::Player(P1))],
    );
    block_and_finish(&mut t, P1, &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 4);
}

#[test]
fn pain_magnification_needs_three_damage_at_once() {
    cr!("603.2", "120.1");
    ruling!(
        "Pain Magnification",
        "This triggers only if all 3 or more damage is dealt at the same time. It doesn’t trigger if damage that adds up to 3 or more is dealt at different times by the same source."
    );
    supported("Pain Magnification");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pain Magnification");
    for _ in 0..3 {
        t.hand(P1, "Forest");
    }
    let src = t.battlefield(P0, "Grizzly Bears");
    // 2, then 1, from the same source: no trigger.
    t.g.deal_damage(src, Entity::Player(P1), 2, false);
    t.g.flush_events();
    t.settle();
    t.g.deal_damage(src, Entity::Player(P1), 1, false);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 3);
    // 3 at once: triggers.
    t.g.deal_damage(src, Entity::Player(P1), 3, false);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 2);
}

#[test]
fn damage_reflection_triggers_even_if_the_creature_dies() {
    cr!("603.10a", "603.2");
    ruling!(
        "Spitemare",
        "Even if the amount of damage dealt to Spitemare is enough to destroy it, Spitemare’s ability will trigger and it will deal the full amount of damage to the target."
    );
    ruling!(
        "Tephraderm",
        "The second ability triggers even if this card is dealt lethal damage."
    );
    supported("Spitemare");
    supported("Tephraderm");
    // Spitemare (3/3) is dealt 5 damage by a creature: it dies and deals 5.
    let mut t = TestGame::new(2);
    let spite = t.battlefield(P0, "Spitemare");
    let src = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    damage(&mut t, src, 5, spite);
    assert_eq!(t.zone(spite), Zone::Graveyard(P0));
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    // Tephraderm (4/5) is dealt 6 damage by P1's Blaze: it dies and deals 6 damage to
    // that spell's controller.
    let mut t = TestGame::new(2);
    let teph = t.battlefield(P0, "Tephraderm");
    t.set_step(P1, Step::PrecombatMain);
    crate::r_s01_common::give_mana_for(&mut t, P1, "Blaze");
    t.lands(P1, "Wastes", 6);
    let dis = t.hand(P1, "Blaze");
    t.cast(P1, dis).x(6).target(teph).go();
    t.resolve_all();
    assert_eq!(t.zone(teph), Zone::Graveyard(P0));
    assert_eq!(t.life(P1), 14);
}

#[test]
fn blazing_sunsteel_ignores_opponents_who_left_the_game() {
    cr!("800.4a", "613.4c");
    ruling!(
        "Blazing Sunsteel",
        "Opponents who have left the game aren't counted by Blazing Sunsteel's first ability."
    );
    supported("Blazing Sunsteel");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sun = t.battlefield(P0, "Blazing Sunsteel");
    t.g.attach(sun, Entity::Object(bears));
    t.g.recompute();
    assert_eq!(t.pt(bears), (4, 2));
    t.g.take_action(P2, Action::Concede);
    t.g.flush_events();
    t.settle();
    t.g.recompute();
    assert_eq!(t.pt(bears), (3, 2));
}
