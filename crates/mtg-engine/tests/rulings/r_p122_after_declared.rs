//! Rulings batch P122 — "can't attack" and "can't block" effects that begin after
//! attackers or blockers have been declared don't remove those creatures from combat or
//! undo the blocks: restrictions are checked only as attackers and blockers are declared
//! (CR 506.4, 508.1c, 509.1b), and a blocked creature stays blocked (CR 509.1h).

use crate::r_p122_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s09_common::{legal_attack, to_combat};
use crate::r_s21_common::legal_blocks;
use crate::r_s28_common::cast_card;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::*;

/// P0 attacks with a Hill Giant that P1 blocks with Grizzly Bears; then `source_name`'s
/// `idx`th activated ability (controlled by `who`, with `pool` mana) targets the blocker.
fn activate_on_blocker(source_name: &str, who: PlayerId, idx: usize, pool: &[(ManaType, u32)]) {
    supported(source_name);
    let mut t = TestGame::new(2);
    let source = t.battlefield(who, source_name);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(who, "Island");
    still_blocks_after(&mut t, giant, bears, |t| {
        for (ty, n) in pool {
            mana(t, who, *ty, *n);
        }
        t.activate(who, source, idx, &[obj(bears)])
            .unwrap_or_else(|e| panic!("{source_name}: {e:?}"));
    });
}

#[test]
fn chainwhip_cyclops_after_blocks() {
    cr!("506.4", "509.1h");
    ruling!(
        "Chainwhip Cyclops",
        "Activating Chainwhip Cyclops’s ability after a creature has blocked won’t remove the blocking creature from combat"
    );
    activate_on_blocker("Chainwhip Cyclops", P0, 0, &[(ManaType::R, 4)]);
}

#[test]
fn clan_guildmage_after_blocks() {
    cr!("506.4", "509.1h");
    ruling!(
        "Clan Guildmage",
        "Activating Clan Guildmage’s first ability after a creature has blocked won’t remove the blocking creature from combat"
    );
    activate_on_blocker("Clan Guildmage", P0, 0, &[(ManaType::R, 2)]);
}

#[test]
fn siegebreaker_giant_after_blocks() {
    cr!("506.4", "509.1h");
    ruling!(
        "Siegebreaker Giant",
        "Activating Siegebreaker Giant’s last ability after a creature has blocked won’t remove the blocking creature from combat"
    );
    activate_on_blocker("Siegebreaker Giant", P0, 0, &[(ManaType::R, 4)]);
}

#[test]
fn zirda_after_blocks() {
    cr!("506.4", "509.1h");
    ruling!(
        "Zirda, the Dawnwaker",
        "Activating Zirda's last ability after a creature has blocked won't remove the blocking creature from combat"
    );
    activate_on_blocker("Zirda, the Dawnwaker", P0, 0, &[(ManaType::R, 1)]);
}

#[test]
fn lambholt_harrier_after_blocks() {
    cr!("506.4", "509.1h");
    ruling!(
        "Lambholt Harrier",
        "Activating it after a creature has been declared as a blocker will not remove that creature from combat."
    );
    activate_on_blocker("Lambholt Harrier", P0, 0, &[(ManaType::R, 4)]);
}

#[test]
fn lambholt_harrier_before_blocks() {
    cr!("509.1b");
    ruling!(
        "Lambholt Harrier",
        "Lambholt Harrier's ability must be activated before blockers are declared to have any effect."
    );
    let mut t = TestGame::new(2);
    let harrier = t.battlefield(P0, "Lambholt Harrier");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    crate::r_s01_common::attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[(bears, giant)]));
    mana(&mut t, P0, ManaType::R, 4);
    t.activate(P0, harrier, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P1, &[(bears, giant)]));
}

#[test]
fn sower_of_chaos_after_blocks() {
    cr!("506.4", "509.1h");
    ruling!(
        "Sower of Chaos",
        "Once a creature has been declared as a blocker, activating Sower of Chaos's last ability targeting that creature won't cause that creature to stop blocking."
    );
    activate_on_blocker("Sower of Chaos", P0, 0, &[(ManaType::R, 3)]);
}

#[test]
fn merciless_javelineer_after_blocks() {
    cr!("506.4", "509.1h");
    ruling!(
        "Merciless Javelineer",
        "Once a creature has blocked, Merciless Javelineer’s ability can’t undo that block."
    );
    activate_on_blocker("Merciless Javelineer", P0, 0, &[(ManaType::R, 2)]);
}

#[test]
fn endbringer_after_blocks_and_attacks() {
    cr!("506.4", "509.1h");
    ruling!(
        "Endbringer",
        "Activating the second activated ability after a creature has legally been declared as an attacker or blocker won't change or undo that attack or block."
    );
    // After blocks: P0's Endbringer (not attacking) targets P1's blocker.
    activate_on_blocker("Endbringer", P0, 1, &[(ManaType::C, 1)]);
    // After attacks: P1's Endbringer targets P0's attacker.
    let mut t = TestGame::new(2);
    let endbringer = t.battlefield(P1, "Endbringer");
    let giant = t.battlefield(P0, "Hill Giant");
    still_attacks_after(&mut t, giant, |t| {
        mana(t, P1, ManaType::C, 1);
        t.activate(P1, endbringer, 1, &[obj(giant)]).unwrap();
    });
}

#[test]
fn bribers_purse_after_attacks_and_blocks() {
    cr!("506.4", "509.1h");
    ruling!(
        "Briber's Purse",
        "Activating the ability targeting a creature that’s already attacking or blocking won’t remove it from combat or affect that attack or block."
    );
    supported("Briber's Purse");
    let mut t = TestGame::new(2);
    let purse = t.battlefield(P0, "Briber's Purse");
    t.g.add_counters(Entity::Object(purse), "gem", 2, None);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    still_blocks_after(&mut t, giant, bears, |t| {
        mana(t, P0, ManaType::C, 1);
        t.activate(P0, purse, 0, &[obj(bears)]).unwrap();
    });
    let mut t = TestGame::new(2);
    let purse = t.battlefield(P1, "Briber's Purse");
    t.g.add_counters(Entity::Object(purse), "gem", 2, None);
    let giant = t.battlefield(P0, "Hill Giant");
    still_attacks_after(&mut t, giant, |t| {
        mana(t, P1, ManaType::C, 1);
        t.activate(P1, purse, 0, &[obj(giant)]).unwrap();
    });
}

#[test]
fn fifty_feet_of_rope_climb_over_after_a_wall_blocked() {
    cr!("506.4", "509.1h");
    ruling!(
        "Fifty Feet of Rope",
        "Activating the Climb Over ability after a Wall has blocked won't change or undo that block."
    );
    supported("Fifty Feet of Rope");
    let mut t = TestGame::new(2);
    let rope = t.battlefield(P0, "Fifty Feet of Rope");
    let giant = t.battlefield(P0, "Hill Giant");
    // A 4/1 Wall: the Hill Giant dies only if the block stands.
    let wall = t.battlefield(P1, "Wall of Torches");
    still_blocks_after(&mut t, giant, wall, |t| {
        t.activate(P0, rope, 0, &[obj(wall)]).unwrap();
    });
    assert!(!t.on_battlefield(giant), "the Wall dealt its combat damage");
}

#[test]
fn kozileks_pathfinder_after_blocked() {
    cr!("506.4", "509.1h");
    ruling!(
        "Kozilek's Pathfinder",
        "Activating the ability once Kozilek’s Pathfinder has been legally blocked won’t change or undo that block."
    );
    supported("Kozilek's Pathfinder");
    let mut t = TestGame::new(2);
    let pathfinder = t.battlefield(P0, "Kozilek's Pathfinder");
    let bears = t.battlefield(P1, "Grizzly Bears");
    still_blocks_after(&mut t, pathfinder, bears, |t| {
        mana(t, P0, ManaType::C, 1);
        t.activate(P0, pathfinder, 0, &[obj(bears)]).unwrap();
    });
}

#[test]
fn sly_instigator_after_blocked() {
    cr!("506.4", "509.1h");
    ruling!(
        "Sly Instigator",
        "Once a creature an opponent controls has been legally blocked, activating Sly Instigator's ability won't change or undo that block."
    );
    supported("Sly Instigator");
    let mut t = TestGame::new(2);
    let instigator = t.battlefield(P1, "Sly Instigator");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    still_blocks_after(&mut t, giant, bears, |t| {
        mana(t, P1, ManaType::U, 1);
        t.activate(P1, instigator, 0, &[obj(giant)]).unwrap();
    });
}

#[test]
fn malicious_intent_after_blocks() {
    cr!("506.4", "509.1h");
    ruling!(
        "Malicious Intent",
        "Once a creature has been declared as a blocking creature, making it unable to block will have no effect."
    );
    supported("Malicious Intent");
    let mut t = TestGame::new(2);
    let elf = t.battlefield(P0, "Llanowar Elves");
    attach_new(&mut t, P0, "Malicious Intent", elf);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    still_blocks_after(&mut t, giant, bears, |t| {
        t.answer_targets(P0, &[obj(bears)]);
        activate_containing(t, P0, elf, "can't block").unwrap();
    });
}

#[test]
fn fearsome_temper_after_blocks() {
    cr!("506.4", "509.1h");
    ruling!(
        "Fearsome Temper",
        "The activated ability granted by Fearsome Temper can’t change or undo a block that’s already happened."
    );
    supported("Fearsome Temper");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Fearsome Temper", giant);
    let bears = t.battlefield(P1, "Grizzly Bears");
    still_blocks_after(&mut t, giant, bears, |t| {
        mana(t, P0, ManaType::R, 3);
        t.activate(P0, giant, 0, &[obj(bears)]).unwrap();
    });
}

#[test]
fn fearsome_temper_in_the_declare_attackers_step() {
    cr!("509.1b", "506.4");
    ruling!(
        "Fearsome Temper",
        "For it to have an effect, you must activate it no later than the declare attackers step."
    );
    ruling!(
        "Fearsome Temper",
        "The target creature can still block other attacking creatures."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Fearsome Temper", giant);
    let lions = t.battlefield(P0, "Savannah Lions");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    crate::r_s01_common::attack_with(
        &mut t,
        &[(giant, Entity::Player(P1)), (lions, Entity::Player(P1))],
    );
    mana(&mut t, P0, ManaType::R, 3);
    t.activate(P0, giant, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P1, &[(bears, giant)]));
    assert!(legal_blocks(&mut t, P1, &[(bears, lions)]));
}

/// P0 casts `name` (lands for it are added) after blocks, with the given modes and target
/// answers, targeting the blocking Grizzly Bears.
fn spell_on_blocker(name: &str, modes: Option<&[usize]>, blocker: &str) -> TestGame {
    supported(name);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let blocker = t.battlefield(P1, blocker);
    crate::r_s25_common::lands_for_cost(&mut t, P0, name);
    let card = t.hand(P0, name);
    still_blocks_after(&mut t, giant, blocker, |t| {
        let mut c = t.cast(P0, card);
        if let Some(m) = modes {
            c = c.modes(m);
        }
        c.target(obj(blocker)).go();
    });
    t
}

#[test]
fn blindblast_after_blocks() {
    cr!("506.4", "509.1h");
    ruling!(
        "Blindblast",
        "Casting Blindblast after a creature has blocked won’t remove the blocking creature from combat unless the damage Blindblast deals causes that creature to die."
    );
    // The blocker survives the damage and keeps blocking.
    spell_on_blocker("Blindblast", None, "Grizzly Bears");
    // The blocker dies: the attacker stays blocked.
    let t = spell_on_blocker("Blindblast", None, "Llanowar Elves");
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
}

#[test]
fn gnoll_camp_intimidate_them_after_blocks() {
    cr!("506.4", "509.1h");
    ruling!(
        "You Come to the Gnoll Camp",
        "Casting this spell and choosing the Intimidate Them mode after blockers have been declared won't change or undo any blocks."
    );
    spell_on_blocker("You Come to the Gnoll Camp", Some(&[0]), "Grizzly Bears");
}

#[test]
fn untimely_malfunction_last_mode_after_blocks() {
    cr!("506.4", "509.1h");
    ruling!(
        "Untimely Malfunction",
        "The effect of Untimely Malfunction's last mode won't cause creatures that are already blocking to stop blocking."
    );
    spell_on_blocker("Untimely Malfunction", Some(&[2]), "Grizzly Bears");
}

#[test]
fn off_balance_before_and_after_declarations() {
    cr!("506.4", "508.1c", "509.1b");
    ruling!(
        "Off Balance",
        "The ability only does something if used before attackers or blockers (as appropriate) are declared during a turn."
    );
    // After declarations: no effect.
    spell_on_blocker("Off Balance", None, "Grizzly Bears");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    still_attacks_after(&mut t, giant, |t| {
        t.answer_targets(P1, &[obj(giant)]);
        cast_card(t, P1, "Off Balance");
    });
    // Before declarations: the creature can't attack, or can't block.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    t.answer_targets(P1, &[obj(giant)]);
    cast_card(&mut t, P1, "Off Balance");
    t.resolve_all();
    assert!(!legal_attack(&mut t, &[(giant, Entity::Player(P1))]));
    let mut t2 = TestGame::new(2);
    let giant = t2.battlefield(P0, "Hill Giant");
    let bears2 = t2.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t2, P0);
    crate::r_s01_common::attack_with(&mut t2, &[(giant, Entity::Player(P1))]);
    t2.answer_targets(P0, &[obj(bears2)]);
    cast_card(&mut t2, P0, "Off Balance");
    t2.resolve_all();
    assert!(!legal_blocks(&mut t2, P1, &[(bears2, giant)]));
    let _ = bears;
}

#[test]
fn lawmages_binding_after_attacks_blocks_and_activations() {
    cr!("506.4", "509.1h", "113.7a");
    ruling!(
        "Lawmage's Binding",
        "Once a creature has attacked or blocked, casting Lawmage’s Binding won’t remove that creature from combat. Similarly, once a creature’s ability has been activated, casting Lawmage’s Binding won’t counter that ability."
    );
    spell_on_blocker("Lawmage's Binding", None, "Grizzly Bears");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    still_attacks_after(&mut t, giant, |t| {
        t.answer_targets(P1, &[obj(giant)]);
        cast_card(t, P1, "Lawmage's Binding");
    });
    // An activated ability on the stack still resolves.
    let mut t = TestGame::new(2);
    let sorcerer = t.battlefield(P1, "Prodigal Sorcerer");
    t.activate(P1, sorcerer, 0, &[Entity::Player(P0)]).unwrap();
    t.answer_targets(P0, &[obj(sorcerer)]);
    cast_card(&mut t, P0, "Lawmage's Binding");
    t.resolve_all();
    let binding = t.named_on_battlefield("Lawmage's Binding")[0];
    assert_eq!(t.g.obj(binding).attached_to, Some(obj(sorcerer)));
    assert_eq!(t.life(P0), 19);
}

#[test]
fn arena_athlete_heroic_after_blocks() {
    cr!("506.4", "509.1h");
    ruling!(
        "Arena Athlete",
        "Resolving Arena Athlete's ability after a creature has blocked won't remove the blocking creature from combat"
    );
    supported("Arena Athlete");
    let mut t = TestGame::new(2);
    let athlete = t.battlefield(P0, "Arena Athlete");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    still_blocks_after(&mut t, giant, bears, |t| {
        t.answer_targets(P0, &[obj(athlete)]);
        t.answer_targets(P0, &[obj(bears)]);
        cast_card(t, P0, "Giant Growth");
        t.settle();
        assert_eq!(t.stack_len(), 2, "heroic triggered");
    });
}

#[test]
fn smelt_ward_minotaur_after_blocks() {
    cr!("506.4", "509.1h");
    ruling!(
        "Smelt-Ward Minotaur",
        "Resolving Smelt-Ward Minotaur’s triggered ability after a creature has blocked won’t remove the blocking creature from combat"
    );
    supported("Smelt-Ward Minotaur");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Smelt-Ward Minotaur");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    still_blocks_after(&mut t, giant, bears, |t| {
        t.answer_targets(P0, &[obj(giant)]);
        t.answer_targets(P0, &[obj(bears)]);
        cast_card(t, P0, "Giant Growth");
        t.settle();
        assert_eq!(t.stack_len(), 2, "the Minotaur's ability triggered");
    });
}

#[test]
fn burden_of_proof_on_a_detectives_blocker() {
    cr!("506.4", "509.1h");
    ruling!(
        "Burden of Proof",
        "Once a Detective has been blocked, attaching Burden of Proof to a creature blocking it won't cause that Detective to become unblocked."
    );
    supported("Burden of Proof");
    let mut t = TestGame::new(2);
    let agent = t.battlefield(P0, "Unscrupulous Agent");
    let lions = t.battlefield(P1, "Savannah Lions");
    still_blocks_after(&mut t, agent, lions, |t| {
        t.answer_targets(P0, &[obj(lions)]);
        cast_card(t, P0, "Burden of Proof");
    });
    // The (now 1/1) Savannah Lions dealt its combat damage to the 1/1 Detective.
    assert!(!t.on_battlefield(agent));
}

#[test]
fn intimidation_bolt_after_attacks() {
    cr!("506.4", "508.1c");
    ruling!(
        "Intimidation Bolt",
        "For the second part of Intimidation Bolt's effect to do anything, it must be cast before attackers are declared."
    );
    supported("Intimidation Bolt");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    still_attacks_after(&mut t, giant, |t| {
        t.answer_targets(P1, &[obj(bears)]);
        cast_card(t, P1, "Intimidation Bolt");
    });
    // Before attackers are declared, the other creatures can't attack.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_combat(&mut t, P0);
    t.answer_targets(P1, &[obj(bears)]);
    cast_card(&mut t, P1, "Intimidation Bolt");
    t.resolve_all();
    assert!(!legal_attack(&mut t, &[(giant, Entity::Player(P1))]));
}

#[test]
fn change_of_heart_after_attacks() {
    cr!("506.4", "508.1c");
    ruling!(
        "Change of Heart",
        "Change of Heart will not remove an already attacking creature from combat. It must be cast before attackers are declared"
    );
    supported("Change of Heart");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    still_attacks_after(&mut t, giant, |t| {
        t.answer_targets(P1, &[obj(giant)]);
        cast_card(t, P1, "Change of Heart");
    });
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    to_combat(&mut t, P0);
    t.answer_targets(P1, &[obj(giant)]);
    cast_card(&mut t, P1, "Change of Heart");
    t.resolve_all();
    assert!(!legal_attack(&mut t, &[(giant, Entity::Player(P1))]));
}

#[test]
fn netter_en_dal_before_and_after_attacks() {
    cr!("506.4", "508.1c");
    ruling!(
        "Netter en-Dal",
        "The ability only does something if used before attackers are declared during a turn. If used after the creature is declared as an attacker, nothing happens."
    );
    supported("Netter en-Dal");
    let mut t = TestGame::new(2);
    let netter = t.battlefield(P1, "Netter en-Dal");
    t.hand(P1, "Island");
    let giant = t.battlefield(P0, "Hill Giant");
    still_attacks_after(&mut t, giant, |t| {
        mana(t, P1, ManaType::W, 1);
        t.activate(P1, netter, 0, &[obj(giant)]).unwrap();
    });
    let mut t = TestGame::new(2);
    let netter = t.battlefield(P1, "Netter en-Dal");
    t.hand(P1, "Island");
    let giant = t.battlefield(P0, "Hill Giant");
    to_combat(&mut t, P0);
    mana(&mut t, P1, ManaType::W, 1);
    t.activate(P1, netter, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    assert!(!legal_attack(&mut t, &[(giant, Entity::Player(P1))]));
}

#[test]
fn orims_chant_kicked_after_attacks() {
    cr!("506.4", "508.1c");
    ruling!(
        "Orim's Chant",
        "Orim's Chant also won't affect creatures that are already attacking. It does not remove them from combat."
    );
    supported("Orim's Chant");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    still_attacks_after(&mut t, giant, |t| {
        crate::r_s25_common::lands_for_cost(t, P1, "Orim's Chant");
        t.lands(P1, "Plains", 1);
        let chant = t.hand(P1, "Orim's Chant");
        t.cast(P1, chant).kicked(true).target(Entity::Player(P0)).go();
    });
}

#[test]
fn arachnus_web_attached_to_an_attacker() {
    cr!("506.4", "508.1c");
    ruling!(
        "Arachnus Web",
        "If Arachnus Web enters the battlefield attached to an attacking or blocking creature (due to Arachnus Spinner's ability, for example), that creature will continue to attack or block."
    );
    supported("Arachnus Web");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    still_attacks_after(&mut t, giant, |t| {
        attach_new(t, P1, "Arachnus Web", giant);
    });
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    still_blocks_after(&mut t, giant, bears, |t| {
        attach_new(t, P0, "Arachnus Web", bears);
    });
}

#[test]
fn bound_in_silence_attached_to_an_attacker() {
    cr!("506.4", "508.1c");
    ruling!(
        "Bound in Silence",
        "If Bound in Silence becomes attached to a creature that's already attacking or blocking, the creature continues to attack or block as normal."
    );
    supported("Bound in Silence");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    still_attacks_after(&mut t, giant, |t| {
        attach_new(t, P1, "Bound in Silence", giant);
    });
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    still_blocks_after(&mut t, giant, bears, |t| {
        attach_new(t, P0, "Bound in Silence", bears);
    });
}
