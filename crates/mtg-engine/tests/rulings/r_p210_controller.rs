//! Rulings batch P210 — enchant (CR 303.4, 702.5): an Aura on another player's creature.
//! Who may activate the abilities it grants, who controls the triggered abilities it has
//! or grants, whose permanents its "you" counts, and who chooses what the creature attacks.

use crate::r_p210_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::*;
use crate::r_s05_common::tokens_with_subtype;
use crate::r_s06_common::*;
use crate::r_s09_common::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Activated abilities: granted ones belong to the creature's controller
// ---------------------------------------------------------------------------------------

#[test]
fn abilities_granted_by_an_aura_are_activated_by_the_creatures_controller() {
    cr!("113.6", "602.2");
    ruling!(
        "Sinking Feeling",
        "Only the creature’s controller can activate that ability, not Sinking Feeling’s controller."
    );
    ruling!(
        "Hold for Ransom",
        "The ability Hold for Ransom grants to the enchanted creature can be activated only by that creature's controller"
    );
    ruling!(
        "Singing Bell Strike",
        "The activated ability that untaps the creature is granted to the enchanted creature. Only the creature's controller can activate this ability."
    );
    ruling!(
        "Predatory Urge",
        "The controller of the enchanted creature may activate the ability, not the controller of Predatory Urge."
    );
    ruling!(
        "Savage Silhouette",
        "The controller of the enchanted creature may activate the regeneration ability, not the controller of Savage Silhouette."
    );
    ruling!(
        "Molting Snakeskin",
        "The regeneration ability is granted to the enchanted creature. Only the creature’s controller can activate the regeneration ability."
    );
    ruling!(
        "Scourge of the Nobilis",
        "The controller of the enchanted creature, not the controller of Scourge of the Nobilis, can activate the +1/+0 ability."
    );
    ruling!(
        "Splinter Twin",
        "That creature's controller (who is not necessarily the Aura's controller) can activate it."
    );
    for (aura, creature) in [
        ("Sinking Feeling", "Grizzly Bears"),
        ("Hold for Ransom", "Grizzly Bears"),
        ("Singing Bell Strike", "Grizzly Bears"),
        ("Predatory Urge", "Grizzly Bears"),
        ("Savage Silhouette", "Grizzly Bears"),
        ("Molting Snakeskin", "Grizzly Bears"),
        // Scourge grants its pump ability only to a red creature.
        ("Scourge of the Nobilis", "Raging Goblin"),
        ("Splinter Twin", "Grizzly Bears"),
    ] {
        assert_eq!(
            who_can_activate(aura, creature, false),
            (false, true),
            "{aura}"
        );
    }
}

#[test]
fn hold_for_ransoms_granted_ability_makes_the_auras_controller_sacrifice_and_draw() {
    cr!("113.6", "602.2", "109.5");
    ruling!(
        "Hold for Ransom",
        "who is usually not the same player who controls Hold for Ransom"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Hold for Ransom", bears);
    t.set_step(P1, Step::PrecombatMain);
    lots_of_mana(&mut t, P1);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    activate_containing(&mut t, P1, bears, "sacrifices it").unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Hold for Ransom"));
    assert_eq!(t.hand_size(P0), h0 + 1);
    assert_eq!(t.hand_size(P1), h1);
}

#[test]
fn splinter_twin_token_goes_to_the_enchanted_creatures_controller() {
    cr!("113.6", "111.2");
    ruling!(
        "Splinter Twin",
        "The creature's controller is the player who gets the token."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Splinter Twin", bears);
    activate_containing(&mut t, P1, bears, "Create a token").unwrap();
    t.resolve_all();
    assert_eq!(tokens(&t, P1).len(), 1);
    assert!(tokens(&t, P0).is_empty());
}

#[test]
fn firebreathings_own_ability_is_activated_by_the_auras_controller() {
    cr!("602.2", "303.4");
    ruling!(
        "Firebreathing",
        "This ability can be activated by Firebreathing’s controller, not the enchanted creature’s controller"
    );
    assert_eq!(
        who_can_activate("Firebreathing", "Grizzly Bears", true),
        (true, false)
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let fb = attach_new(&mut t, P0, "Firebreathing", bears);
    lots_of_mana(&mut t, P0);
    activate_containing(&mut t, P0, fb, "+1/+0").unwrap();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 2));
}

// ---------------------------------------------------------------------------------------
// Triggered abilities: the Aura's own vs. granted ones
// ---------------------------------------------------------------------------------------

#[test]
fn captivating_glance_triggers_in_its_controllers_end_step_and_you_is_that_player() {
    cr!("603.3a", "701.30a", "109.5");
    ruling!(
        "Captivating Glance",
        "The ability triggers at the end of Captivating Glance's controller's turn. \"You\" refers to the controller of Captivating Glance, not the controller of the enchanted creature."
    );
    supported("Captivating Glance");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Captivating Glance", bears);
    // Not in P1's end step.
    t.advance_to(P1, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "clash"), 0);
    // In P0's end step: P0 wins the clash (Hill Giant, mana value 4, against a Bears).
    t.advance_to(P0, Step::PrecombatMain);
    stack_library(&mut t, P0, &["Hill Giant"]);
    stack_library(&mut t, P1, &["Grizzly Bears"]);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "clash"), 1);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
}

#[test]
fn granted_triggered_abilities_belong_to_the_creatures_controller() {
    cr!("113.6", "603.3a", "109.5");
    ruling!(
        "Cathar's Call",
        "The controller of the enchanted creature creates the Human token, even if they aren't the controller of Cathar's Call."
    );
    ruling!(
        "Commander's Authority",
        "The controller of the enchanted creature gets the token."
    );
    ruling!(
        "Verdant Embrace",
        "The token is put onto the battlefield under the control of the player who controls the enchanted creature, not the player who controls Verdant Embrace."
    );
    ruling!(
        "Pillory of the Sleepless",
        "The controller of the enchanted creature will lose 1 life, not the controller of Pillory of the Sleepless."
    );
    // Cathar's Call: in P1's end step (not P0's), P1 creates a Human.
    supported("Cathar's Call");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Cathar's Call", bears);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty() && tokens(&t, P1).is_empty());
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P1, "Human").len(), 1);
    assert!(tokens(&t, P0).is_empty());

    // Commander's Authority: in P1's upkeep (not P0's), P1 creates a Human.
    supported("Commander's Authority");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Commander's Authority", bears);
    next_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P1, "Human").len(), 1);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    assert_eq!(tokens(&t, P1).len(), 1);

    // Verdant Embrace: each upkeep, P1 creates a Saproling.
    supported("Verdant Embrace");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Verdant Embrace", bears);
    next_upkeep(&mut t, P1);
    t.resolve_all();
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P1, "Saproling").len(), 2);
    assert!(tokens(&t, P0).is_empty());

    // Pillory of the Sleepless: P1 loses 1 life in P1's upkeep, P0 never.
    supported("Pillory of the Sleepless");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Pillory of the Sleepless", bears);
    next_upkeep(&mut t, P1);
    t.resolve_all();
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (20, 19));
}

#[test]
fn recumbent_bliss_triggers_in_the_auras_controllers_upkeep() {
    cr!("603.3a", "109.5");
    ruling!(
        "Recumbent Bliss",
        "The triggered ability is on the Aura, not the creature. It triggers at the beginning of the upkeep of Recumbent Bliss's controller, not the enchanted creature's controller, and Recumbent Bliss's controller will gain the life."
    );
    supported("Recumbent Bliss");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Recumbent Bliss", bears);
    next_upkeep(&mut t, P1);
    assert_eq!(triggers_on_stack(&t, "gain 1 life"), 0);
    t.answer_yes(P0, true);
    next_upkeep(&mut t, P0);
    assert_eq!(triggers_on_stack(&t, "gain 1 life"), 1);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (21, 20));
}

#[test]
fn artificers_hex_triggers_in_the_hex_controllers_upkeep() {
    cr!("603.3a", "603.4", "109.5");
    ruling!(
        "Artificer's Hex",
        "This ability triggers at the beginning of the upkeep of the controller of Artificer’s Hex, not the upkeep of the controller of the enchanted Equipment"
    );
    supported("Artificer's Hex");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let sword = attach_new(&mut t, P1, "Bonesplitter", bears);
    attach_new(&mut t, P0, "Artificer's Hex", sword);
    next_upkeep(&mut t, P1);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn parasitic_implant_the_auras_controller_gets_the_myr() {
    cr!("603.3a", "109.5", "111.2");
    ruling!(
        "Parasitic Implant",
        "The controller of Parasitic Implant will control the Myr token no matter who controlled and sacrificed the enchanted creature."
    );
    supported("Parasitic Implant");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Parasitic Implant", bears);
    next_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(tokens_with_subtype(&t, P0, "Myr").len(), 1);
    assert!(tokens(&t, P1).is_empty());
}

#[test]
fn an_auras_enchanted_creatures_controller_sacrifices_it_means_the_creature() {
    // Regression: "enchanted permanent's controller sacrifices it" sacrificed nothing
    // ("it" was read as the Aura, which that player doesn't control).
    cr!("701.21a", "603.10a");
    supported("Reality Acid");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let acid = attach_new(&mut t, P0, "Reality Acid", bears);
    destroy(&mut t, acid);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn luminous_wake_its_controller_gains_life_when_the_creature_attacks_or_blocks() {
    cr!("603.3a", "508.1m", "509.1i");
    ruling!(
        "Luminous Wake",
        "The controller of Luminous Wake (who is not necessarily the controller of the enchanted creature) controls the triggered ability and gains life when it resolves."
    );
    supported("Luminous Wake");
    // Attacks.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Luminous Wake", bears);
    to_combat(&mut t, P1);
    attack_with(&mut t, &[(bears, P0.into())]);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (24, 20));
    // Blocks.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Luminous Wake", bears);
    let giant = t.battlefield(P0, "Hill Giant");
    to_combat(&mut t, P0);
    attack_with(&mut t, &[(giant, P1.into())]);
    block_and_finish(&mut t, P1, &[(bears, giant)]);
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn curses_give_the_curses_controller_the_token_or_card() {
    cr!("303.4", "603.2", "111.2");
    ruling!(
        "Curse of Clinging Webs",
        "The controller of Curse of Clinging Webs gets the Spider token. This may not be the same player as the enchanted player."
    );
    ruling!(
        "Curse of the Restless Dead",
        "The controller of Curse of the Restless Dead gets the Zombie token. This may not be the same player as the enchanted player."
    );
    supported("Curse of Clinging Webs");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Curse of Clinging Webs", P1);
    destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Spider").len(), 1);
    assert!(tokens(&t, P1).is_empty());
    assert!(t.in_exile("Grizzly Bears"));

    supported("Curse of the Restless Dead");
    let mut t = TestGame::new(2);
    attach_new(&mut t, P0, "Curse of the Restless Dead", P1);
    t.enter(P1, "Forest");
    t.resolve_all();
    assert_eq!(tokens_with_subtype(&t, P0, "Zombie").len(), 1);
    assert!(tokens(&t, P1).is_empty());
}

#[test]
fn lifelink_aura_the_creatures_controller_gains_the_life() {
    cr!("702.15b", "702.15d");
    ruling!(
        "Lifelink",
        "The controller of the enchanted creature, not the controller of Lifelink, gains the life"
    );
    supported("Lifelink");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Lifelink", bears);
    deal(&mut t, bears, 2, P0);
    assert_eq!((t.life(P0), t.life(P1)), (18, 22));
}

#[test]
fn dawns_reflection_the_lands_controller_gets_the_mana() {
    cr!("605.1b", "106.12a");
    ruling!(
        "Dawn's Reflection",
        "The controller of the land gets the additional mana, not the controller of the enchantment."
    );
    supported("Dawn's Reflection");
    let mut t = TestGame::new(2);
    let forest = t.battlefield(P1, "Forest");
    attach_new(&mut t, P0, "Dawn's Reflection", forest);
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P1, forest, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.g.player(P1).mana_pool.total(), 3);
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

// ---------------------------------------------------------------------------------------
// "Opponent" in a granted ability vs. in the Aura's own ability
// ---------------------------------------------------------------------------------------

#[test]
fn granted_damage_to_an_opponent_triggers_mean_the_creatures_controllers_opponent() {
    cr!("113.6", "603.2", "120.3");
    ruling!(
        "Snake Umbra",
        "Snake Umbra grants the triggered ability to the creature. It triggers whenever the enchanted creature deals damage to an opponent of its controller (who is not necessarily an opponent of the Aura's controller). In other words, if your Snake Umbra winds up enchanting your opponent's creature, that opponent will draw a card whenever that creature damages you."
    );
    ruling!(
        "Helm of the Ghastlord",
        "The two abilities trigger when the enchanted creature deals damage to an opponent of the creature’s controller, not when it deals damage to an opponent of the Helm’s controller."
    );
    // Snake Umbra on P1's creature: it damages P0 → P1 (may) draw.
    supported("Snake Umbra");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Snake Umbra", bears);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.answer_yes(P1, true);
    deal(&mut t, bears, 1, P0);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (h0, h1 + 1));
    // Helm of the Ghastlord on P1's blue creature: it damages P0 → P1 draws.
    supported("Helm of the Ghastlord");
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P1, "Wind Drake");
    attach_new(&mut t, P0, "Helm of the Ghastlord", drake);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    deal(&mut t, drake, 1, P0);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.hand_size(P1)), (h0, h1 + 1));
}

#[test]
fn ophidian_eye_means_an_opponent_of_the_auras_controller() {
    cr!("603.2", "109.5");
    ruling!(
        "Ophidian Eye",
        "The third ability triggers when the enchanted creature deals damage to an opponent of the player who controls Ophidian Eye."
    );
    supported("Ophidian Eye");
    // On P1's creature, damage to P0 (not P0's opponent): no trigger.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    attach_new(&mut t, P0, "Ophidian Eye", bears);
    deal(&mut t, bears, 1, P0);
    assert_eq!(triggers_on_stack(&t, "draw a card"), 0);
    // On P0's own creature, damage to P1: P0 draws.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Ophidian Eye", bears);
    let h0 = t.hand_size(P0);
    t.answer_yes(P0, true);
    deal(&mut t, bears, 1, P1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 1);
}

// ---------------------------------------------------------------------------------------
// "You control" counts the Aura's controller's permanents
// ---------------------------------------------------------------------------------------

#[test]
fn auras_counting_what_you_control_count_the_auras_controllers_permanents() {
    cr!("109.5", "613.4c");
    ruling!(
        "Armored Ascension",
        "This ability counts the number of Plains controlled by Armored Ascension’s controller, not the enchanted creature’s controller"
    );
    ruling!(
        "Quag Sickness",
        "This ability counts the number of Swamps controlled by Quag Sickness’s controller, not the enchanted creature’s controller"
    );
    ruling!(
        "Sigil of the Nayan Gods",
        "Sigil of the Nayan Gods's ability counts the number of creatures you control, regardless of who controls the creature the Aura is enchanting. If you control the creature it's enchanting, the bonus will include that creature too."
    );
    supported("Armored Ascension");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 2);
    t.lands(P1, "Plains", 3);
    attach_new(&mut t, P0, "Armored Ascension", giant);
    assert_eq!(t.pt(giant), (5, 5));

    supported("Quag Sickness");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 1);
    t.lands(P1, "Swamp", 2);
    attach_new(&mut t, P0, "Quag Sickness", giant);
    assert_eq!(t.pt(giant), (2, 2));

    supported("Sigil of the Nayan Gods");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P0, "Grizzly Bears");
    let sigil = attach_new(&mut t, P0, "Sigil of the Nayan Gods", giant);
    assert_eq!(t.pt(giant), (4, 4));
    // P0 gains control of the enchanted creature: it counts itself too.
    give_control(&mut t, giant, P0);
    assert!(t.on_battlefield(sigil));
    assert_eq!(t.pt(giant), (5, 5));
}

// ---------------------------------------------------------------------------------------
// "Attacks each combat if able": the creature's controller still chooses what it attacks
// ---------------------------------------------------------------------------------------

#[test]
fn must_attack_auras_and_curses_leave_the_choice_of_defender_to_the_attacker() {
    cr!("508.1a", "508.1d");
    ruling!(
        "Guise of Fire",
        "The controller of the enchanted creature still decides which player or planeswalker the creature attacks."
    );
    ruling!(
        "Skin Invasion // Skin Shedder",
        "The enchanted creature's controller still chooses which player or planeswalker that creature attacks."
    );
    ruling!(
        "Curse of the Nightly Hunt",
        "The enchanted player still chooses which player or planeswalker each creature they control attacks."
    );
    let setup = |aura: &str, curse: bool| {
        supported(aura);
        let mut t = TestGame::new(3);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let jace = t.battlefield(P0, "Jace Beleren");
        if curse {
            attach_new(&mut t, P0, aura, P1);
        } else {
            attach_new(&mut t, P0, aura, bears);
        }
        to_combat(&mut t, P1);
        (t, bears, jace)
    };
    for (aura, curse) in [
        ("Guise of Fire", false),
        ("Skin Invasion // Skin Shedder", false),
        ("Curse of the Nightly Hunt", true),
    ] {
        let (mut t, bears, jace) = setup(aura, curse);
        assert!(!legal_attack(&mut t, &[]), "{aura}: must attack");
        for defender in [Entity::Player(P0), Entity::Player(P2), Entity::Object(jace)] {
            assert!(legal_attack(&mut t, &[(bears, defender)]), "{aura}");
        }
        let attacks = declare(&mut t, P1, &[(bears, Entity::Player(P2))]);
        assert_eq!(attacks, vec![(bears, Entity::Player(P2))], "{aura}");
    }
}
