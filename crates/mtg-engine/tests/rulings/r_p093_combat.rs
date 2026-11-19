//! Rulings batch P093 — attack and combat-damage triggers that draw cards for life: Nazar's
//! life-gain events, Preacher of the Schism's "most life" checks, Mindblade Render, Syr
//! Gwyn's equip Knight, The Chief Warg, Ketramose, Summon: Primal Odin, Roshan, Grasping
//! Shadows, Hazoret, Shessra's block requirement and Tymna's extra main phases.

use crate::r_p093_common::*;
use mtg_engine::ability::{Destination, Effect, Modification, Sel, Value};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const FEEDING: &str = "feeding";
const NAZAR: &str = "Nazar, the Velvet Fang";

// --- Nazar, the Velvet Fang ------------------------------------------------------------------

#[test]
fn nazar_counts_life_gaining_events() {
    cr!("119.9", "702.15b", "510.2");
    ruling!(
        "Nazar, the Velvet Fang",
        "Each creature with lifelink dealing combat damage causes a separate life-gaining event. For example, if two creatures you control with lifelink deal combat damage at the same time, Nazar's second ability will trigger twice. However, if a single creature you control with lifelink deals combat damage to multiple creatures, players, planeswalkers, and/or battles at the same time (perhaps because it has trample or was blocked by more than one creature), the ability will trigger only once."
    );
    let mut t = TestGame::new(2);
    let nazar = t.battlefield(P0, NAZAR);
    crate::r_p096_common::two_lifelinkers(&mut t);
    assert_eq!(t.counters(nazar, FEEDING), 2);
    let mut t = TestGame::new(2);
    let nazar = t.battlefield(P0, NAZAR);
    crate::r_p096_common::one_trampling_lifelinker(&mut t);
    assert_eq!(t.counters(nazar, FEEDING), 1);

    ruling!(
        "Nazar, the Velvet Fang",
        "If you gain an amount of life \"for each\" of something or \"equal to the number\" of something, that life is gained as one event and Nazar's second ability triggers only once."
    );
    // Congregate: "Target player gains 2 life for each creature on the battlefield."
    let mut t = TestGame::new(2);
    let nazar = t.battlefield(P0, NAZAR);
    crate::r_p096_common::congregate(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P0), 28);
    assert_eq!(t.counters(nazar, FEEDING), 1);

    ruling!(
        "Nazar, the Velvet Fang",
        "Nazar's second ability triggers just once for each life-gaining event, whether it's 1 life from Dazzling Angel or 4 life from Apothecary Stomper."
    );
    supported("Dazzling Angel");
    supported("Apothecary Stomper");
    // Dazzling Angel: "Whenever another creature you control enters, you may gain 1 life."
    let mut t = TestGame::new(2);
    let nazar = t.battlefield(P0, NAZAR);
    t.battlefield(P0, "Dazzling Angel");
    t.answer_yes(P0, true);
    crate::r_s05_common::enter(&mut t, P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    assert_eq!(t.counters(nazar, FEEDING), 1);
    // Apothecary Stomper: "When this creature enters, choose one — • Put two +1/+1 counters
    // on target creature. • You gain 4 life."
    let mut t = TestGame::new(2);
    let nazar = t.battlefield(P0, NAZAR);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    crate::r_s05_common::enter(&mut t, P0, "Apothecary Stomper");
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.counters(nazar, FEEDING), 1);
}

#[test]
fn nazar_ignores_a_teammates_life_gain() {
    cr!("810.9", "119.9");
    ruling!(
        "Nazar, the Velvet Fang",
        "In a Two-Headed Giant game, life gained by your teammate won't cause Nazar's second ability to trigger, even though it caused your team's life total to increase."
    );
    let mut t = crate::r_s33_common::two_headed_giant();
    let nazar = t.battlefield(P0, NAZAR);
    crate::r_p096_common::gain_one(&mut t, P1);
    assert_eq!(t.counters(nazar, FEEDING), 0);
    crate::r_p096_common::gain_one(&mut t, P0);
    assert_eq!(t.counters(nazar, FEEDING), 1);
}

#[test]
fn nazar_must_be_on_the_battlefield_to_remove_its_counters() {
    cr!("118.12", "118.3", "400.7");
    ruling!(
        "Nazar, the Velvet Fang",
        "If Nazar isn't on the battlefield as its last ability resolves, you can't remove three feeding counters from it, even if it had three or more feeding counters on it before it left the battlefield. You won't draw cards or lose life."
    );
    let mut t = TestGame::new(2);
    let nazar = t.battlefield(P0, NAZAR);
    put_counters(&mut t, nazar, FEEDING, 3);
    attack_with(&mut t, &[(nazar, Entity::Player(P1))]);
    assert_eq!(from_source(&t, nazar), 1);
    crate::r_s05_common::move_to(&mut t, nazar, Zone::Hand(P0));
    t.answer_yes(P0, true);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand, 20));
    // On the battlefield, it works.
    let mut t = TestGame::new(2);
    let nazar = t.battlefield(P0, NAZAR);
    put_counters(&mut t, nazar, FEEDING, 4);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(nazar, Entity::Player(P1))]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 3, 17));
    assert_eq!(t.counters(nazar, FEEDING), 1);
}

// --- Preacher of the Schism ----------------------------------------------------------------

const PREACHER: &str = "Preacher of the Schism";

fn vampires(t: &TestGame) -> usize {
    crate::r_s05_common::tokens_with_subtype(t, P0, "Vampire").len()
}

#[test]
fn preacher_tied_for_most_life_triggers_both() {
    cr!("508.1m", "603.2");
    ruling!(
        "Preacher of the Schism",
        "If you are tied for most life and Preacher of the Schism attacks another player who is tied for most life, both its middle and its last ability will trigger."
    );
    let mut t = TestGame::new(2);
    let preacher = t.battlefield(P0, PREACHER);
    attack_with(&mut t, &[(preacher, Entity::Player(P1))]);
    assert_eq!(from_source(&t, preacher), 2);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(vampires(&t), 1);
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 1, 19));
}

#[test]
fn preacher_doesnt_recheck_life_totals_on_resolution() {
    cr!("603.2", "608.2c");
    ruling!(
        "Preacher of the Schism",
        "Once Preacher of the Schism's middle ability triggers, it doesn't matter what happens to players' life totals before the ability resolves. You'll create a Vampire token even if the player Preacher of the Schism attacked doesn't have the most life as the ability resolves."
    );
    let mut t = TestGame::new(2);
    let preacher = t.battlefield(P0, PREACHER);
    set_life(&mut t, P0, 10);
    attack_with(&mut t, &[(preacher, Entity::Player(P1))]);
    assert_eq!(from_source(&t, preacher), 1);
    t.g.lose_life(P1, 15);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(vampires(&t), 1);
    assert_eq!(t.life(P0), 10);

    ruling!(
        "Preacher of the Schism",
        "Similarly, once Preacher of the Schism's last ability triggers, it doesn't matter what happens to players' life totals before the ability resolves. You'll draw a card and lose 1 life even if you don't have the most life as the ability resolves."
    );
    let mut t = TestGame::new(2);
    let preacher = t.battlefield(P0, PREACHER);
    set_life(&mut t, P1, 10);
    attack_with(&mut t, &[(preacher, Entity::Player(P1))]);
    assert_eq!(from_source(&t, preacher), 1);
    t.g.lose_life(P0, 15);
    t.g.flush_events();
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(vampires(&t), 0);
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 1, 4));
}

#[test]
fn preacher_attacking_a_planeswalker_doesnt_trigger_its_middle_ability() {
    cr!("508.1m", "506.3");
    ruling!(
        "Preacher of the Schism",
        "Preacher of the Schism's middle ability triggers only if it attacks a player. It won't trigger if Preacher of the Schism attacks a planeswalker or battle, no matter what its controller's life total is."
    );
    // P1 has the most life; Preacher attacks P1's Chandra.
    let mut t = TestGame::new(2);
    let preacher = t.battlefield(P0, PREACHER);
    let chandra = crate::r_p197_common::walker(&mut t, P1, "Chandra, Torch of Defiance", 4);
    set_life(&mut t, P0, 10);
    attack_with(&mut t, &[(preacher, obj(chandra))]);
    assert_eq!(from_source(&t, preacher), 0);
    t.resolve_all();
    assert_eq!(vampires(&t), 0);
}

// --- Mindblade Render ------------------------------------------------------------------------

#[test]
fn mindblade_render_and_an_opponents_warrior() {
    cr!("603.2", "608.2c", "806.1");
    ruling!(
        "Mindblade Render",
        "Mindblade Render's ability triggers if a Warrior an opponent controls deals combat damage to another player who's also your opponent. You can't choose not to draw a card or lose 1 life if this happens."
    );
    supported("Mindblade Render");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Mindblade Render");
    let warrior = t.battlefield(P1, "Elvish Warrior");
    t.set_step(P1, Step::BeginningOfCombat);
    let from = t.asked().len();
    let hand = t.hand_size(P0);
    t.attack(&[(warrior, Entity::Player(P2))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P2), 18);
    assert_eq!(yes_nos(&t, P0, from), 0);
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 1, 19));
}

#[test]
fn mindblade_render_triggers_once_per_combat_damage_step() {
    cr!("603.2c", "510.2");
    ruling!(
        "Mindblade Render",
        "Mindblade Render's ability triggers only once, regardless of how many opponents were dealt damage, how much damage they were dealt, or how many Warriors dealt damage."
    );
    let mut t = TestGame::new(3);
    let render = t.battlefield(P0, "Mindblade Render");
    let a = t.battlefield(P0, "Elvish Warrior");
    let b = t.battlefield(P0, "Elvish Warrior");
    let c = t.battlefield(P0, "Elvish Warrior");
    attack_with(
        &mut t,
        &[
            (a, Entity::Player(P1)),
            (b, Entity::Player(P1)),
            (c, Entity::Player(P2)),
        ],
    );
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!((t.life(P1), t.life(P2)), (16, 18));
    assert_eq!(from_source(&t, render), 1);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 1, 19));
}

// --- Syr Gwyn, Hero of Ashvale -------------------------------------------------------------

const GWYN: &str = "Syr Gwyn, Hero of Ashvale";

/// The texts of the activated abilities of `id`.
fn activated_texts(t: &mut TestGame, id: ObjectId) -> Vec<String> {
    t.g.recompute();
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .map(|a| a.text.to_string())
        .collect()
}

const KNIGHT: bool = true;

/// Activates an equip ability of `equipment` targeting `target`: equip Knight {0} (granted
/// by Syr Gwyn) with `knight`, else the Equipment's own (non-zero cost) equip ability. It
/// stays on the stack.
fn equip(t: &mut TestGame, equipment: ObjectId, knight: bool, target: ObjectId) {
    use mtg_engine::ability::AbilityKind;
    t.g.recompute();
    let uid = t
        .obj_now(equipment)
        .chars
        .abilities
        .iter()
        .find(|a| match &a.kind {
            AbilityKind::Activated(x) => {
                (x.cost.mana.as_ref().map_or(0, |m| m.mana_value()) == 0) == knight
            }
            _ => false,
        })
        .map(|a| a.uid)
        .expect("no such equip ability");
    t.answer_targets(P0, &[obj(target)]);
    t.g.turn.priority = Some(P0);
    t.g.activate_ability(P0, t.g.current(equipment), uid)
        .expect("equip");
    t.g.flush_events();
    t.settle();
}

#[test]
fn syr_gwyn_equipment_keep_their_own_equip_abilities() {
    cr!("702.6a", "613.1f");
    ruling!(
        "Syr Gwyn, Hero of Ashvale",
        "Equipment you control still have their other equip abilities in addition to equip Knight {0}."
    );
    supported(GWYN);
    let mut t = TestGame::new(2);
    t.battlefield(P0, GWYN);
    let splitter = t.battlefield(P0, "Bonesplitter");
    let texts = activated_texts(&mut t, splitter);
    assert_eq!(texts.len(), 2, "{texts:?}");
    // Equip {1} onto a non-Knight.
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    equip(&mut t, splitter, !KNIGHT, bears);
    t.resolve_all();
    assert_eq!(crate::r_s06_common::attached_to(&t, splitter), Some(obj(bears)));
}

#[test]
fn equip_knight_resolves_after_syr_gwyn_leaves() {
    cr!("113.7a", "702.6a");
    ruling!(
        "Syr Gwyn, Hero of Ashvale",
        "If Syr Gwyn leaves the battlefield after an equip Knight ability has been activated, that ability still resolves even though the Equipment has lost that ability."
    );
    let mut t = TestGame::new(2);
    let gwyn = t.battlefield(P0, GWYN);
    let splitter = t.battlefield(P0, "Bonesplitter");
    let knight = t.battlefield(P0, "White Knight");
    equip(&mut t, splitter, KNIGHT, knight);
    kill(&mut t, gwyn);
    assert_eq!(activated_texts(&mut t, splitter).len(), 1);
    t.resolve_all();
    assert_eq!(crate::r_s06_common::attached_to(&t, splitter), Some(obj(knight)));
}

#[test]
fn syr_gwyn_triggers_once_per_equipped_attacker() {
    cr!("603.2", "508.1m");
    ruling!(
        "Syr Gwyn, Hero of Ashvale",
        "Syr Gwyn's triggered ability triggers once per attacking creature that's equipped, regardless of how many Equipment those creatures each carry."
    );
    let mut t = TestGame::new(2);
    let gwyn = t.battlefield(P0, GWYN);
    let knight = t.battlefield(P0, "White Knight");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lions = t.battlefield(P0, "Silvercoat Lion");
    for e in ["Bonesplitter", "Short Sword"] {
        crate::r_s06_common::attach_new(&mut t, P0, e, knight);
    }
    crate::r_s06_common::attach_new(&mut t, P0, "Short Sword", bears);
    attack_with(
        &mut t,
        &[
            (knight, Entity::Player(P1)),
            (bears, Entity::Player(P1)),
            (lions, Entity::Player(P1)),
        ],
    );
    assert_eq!(from_source(&t, gwyn), 2);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 2, 18));
}

#[test]
fn equip_knight_checks_knight_on_activation_and_resolution_only() {
    cr!("608.2b", "702.6a", "301.5c");
    ruling!(
        "Syr Gwyn, Hero of Ashvale",
        "Whether the target creature is a Knight is checked only as the equip Knight ability is activated and as that ability resolves. If the creature somehow becomes a non-Knight creature later, the Equipment remains attached to it."
    );
    let not_a_knight = || vec![Modification::RemoveSubtypes(vec!["Knight".into()])];
    // Later: it stays attached.
    let mut t = TestGame::new(2);
    t.battlefield(P0, GWYN);
    let splitter = t.battlefield(P0, "Bonesplitter");
    let knight = t.battlefield(P0, "White Knight");
    equip(&mut t, splitter, KNIGHT, knight);
    t.resolve_all();
    modify_until_eot(&mut t, knight, not_a_knight());
    assert!(!t.obj_now(knight).chars.has_subtype("Knight"));
    assert_eq!(crate::r_s06_common::attached_to(&t, splitter), Some(obj(knight)));
    assert_eq!(t.pt(knight), (4, 2));
    // Before it resolves: an illegal target, the ability doesn't resolve.
    let mut t = TestGame::new(2);
    t.battlefield(P0, GWYN);
    let splitter = t.battlefield(P0, "Bonesplitter");
    let knight = t.battlefield(P0, "White Knight");
    equip(&mut t, splitter, KNIGHT, knight);
    modify_until_eot(&mut t, knight, not_a_knight());
    t.resolve_all();
    assert_eq!(crate::r_s06_common::attached_to(&t, splitter), None);
}

// --- The Chief Warg ------------------------------------------------------------------------

#[test]
fn the_chief_warg_checks_power_only_when_you_attack() {
    cr!("603.2", "508.1m");
    ruling!(
        "The Chief Warg",
        "The Chief Warg's triggered ability will check when you attack to see if you control a creature with power 4 or greater. If you don't, the ability won't trigger at all. It will not check again when it resolves, so the trigger will resolve even if you no longer control a creature with power 4 or greater."
    );
    supported("The Chief Warg");
    // No creature with power 4 or greater: no trigger.
    let mut t = TestGame::new(2);
    let warg = t.battlefield(P0, "The Chief Warg");
    attack_with(&mut t, &[(warg, Entity::Player(P1))]);
    assert_eq!(from_source(&t, warg), 0);
    // Craw Wurm (6/4): it triggers, and resolves after the Wurm shrinks.
    let mut t = TestGame::new(2);
    let warg = t.battlefield(P0, "The Chief Warg");
    let wurm = t.battlefield(P0, "Craw Wurm");
    attack_with(&mut t, &[(warg, Entity::Player(P1))]);
    assert_eq!(from_source(&t, warg), 1);
    modify_until_eot(
        &mut t,
        wurm,
        vec![Modification::ModifyPT(Value::c(-5), Value::c(0))],
    );
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (hand + 1, 19));
}

// --- Ketramose, the New Dawn -----------------------------------------------------------------

const KETRAMOSE: &str = "Ketramose, the New Dawn";

#[test]
fn ketramose_counts_every_card_in_exile() {
    cr!("508.1c", "509.1b");
    ruling!(
        "Ketramose, the New Dawn",
        "The ability that prevents Ketramose from attacking or blocking counts the total number of cards in exile, regardless of who owns them or how they were exiled."
    );
    supported(KETRAMOSE);
    let mut t = TestGame::new(2);
    let ketramose = t.battlefield(P0, KETRAMOSE);
    t.set_step(P0, Step::BeginningOfCombat);
    for i in 0..6 {
        t.exile(if i % 2 == 0 { P0 } else { P1 }, "Grizzly Bears");
    }
    assert!(!crate::r_s02_common::can_attack(&mut t, ketramose));
    // A seventh card, P1's, exiled by an effect.
    let giant = t.graveyard(P1, "Hill Giant");
    crate::r_s05_common::move_to(&mut t, giant, Zone::Exile);
    assert!(crate::r_s02_common::can_attack(&mut t, ketramose));
    // Blocking: P1's Grizzly Bears attack; Ketramose can block them once a seventh card,
    // P0's this time, is exiled.
    let mut t = TestGame::new(2);
    let ketramose = t.battlefield(P0, KETRAMOSE);
    let bears = t.battlefield(P1, "Grizzly Bears");
    for i in 0..6 {
        t.exile(if i % 2 == 0 { P0 } else { P1 }, "Hill Giant");
    }
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    let block = [(ketramose, bears)];
    assert!(!crate::r_s21_common::legal_blocks(&mut t, P0, &block));
    let giant = t.graveyard(P0, "Hill Giant");
    crate::r_s05_common::move_to(&mut t, giant, Zone::Exile);
    assert!(crate::r_s21_common::legal_blocks(&mut t, P0, &block));
}

#[test]
fn ketramose_keeps_attacking_when_exile_shrinks() {
    cr!("506.4a", "508.1c", "509.1b");
    ruling!(
        "Ketramose, the New Dawn",
        "Once Ketramose has been declared as an attacker or blocker, it will continue to attack or block that combat even if the number of cards in exile falls below seven."
    );
    let mut t = TestGame::new(2);
    let ketramose = t.battlefield(P0, KETRAMOSE);
    let exiled: Vec<ObjectId> = (0..7).map(|_| t.exile(P1, "Grizzly Bears")).collect();
    attack_with(&mut t, &[(ketramose, Entity::Player(P1))]);
    crate::r_s05_common::move_to(&mut t, exiled[0], Zone::Graveyard(P1));
    assert!(crate::r_s10_common::attacking(&t, ketramose));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.life(P0), 24);
    // Blocking: it blocks P1's Hill Giant with seven cards in exile, keeps blocking with
    // six, and deals and takes its damage.
    let mut t = TestGame::new(2);
    let ketramose = t.battlefield(P0, KETRAMOSE);
    let giant = t.battlefield(P1, "Hill Giant");
    let exiled: Vec<ObjectId> = (0..7).map(|_| t.exile(P1, "Grizzly Bears")).collect();
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(giant, Entity::Player(P0))]);
    t.answer(
        P0,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(ketramose, giant)]),
    );
    t.advance_to(P1, Step::DeclareBlockers);
    assert!(crate::r_s10_common::blocking(&t, ketramose));
    crate::r_s05_common::move_to(&mut t, exiled[0], Zone::Graveyard(P1));
    assert!(!crate::r_s21_common::legal_blocks(&mut t, P0, &[(ketramose, giant)]));
    assert!(crate::r_s10_common::blocking(&t, ketramose));
    t.advance_to(P1, Step::EndOfCombat);
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert_eq!(t.life(P0), 24);
    assert_eq!(t.obj_now(ketramose).damage, 3);
}

// --- Summon: Primal Odin -----------------------------------------------------------------------

#[test]
fn primal_odins_granted_trigger_works_whatever_happens_to_odin() {
    cr!("113.7a", "104.3e");
    ruling!(
        "Summon: Primal Odin",
        "Once Summon: Primal Odin deals combat damage to a player and the ability granted by Summon: Primal Odin's second chapter ability triggers, it doesn't matter what happens to Summon: Primal Odin. When that ability resolves, the player who was dealt combat damage this way loses the game even if Summon: Primal Odin isn't on the battlefield anymore or is controlled by another player."
    );
    for leave in [true, false] {
        let mut t = TestGame::new(3);
        let odin = t.battlefield(P0, "Summon: Primal Odin");
        t.g.objects[odin.0 as usize].counters.clear();
        put_counters(&mut t, odin, counters::LORE, 1);
        t.resolve_all();
        crate::r_s19_common::add_lore(&mut t, odin, 1);
        t.resolve_all();
        attack_with(&mut t, &[(odin, Entity::Player(P1))]);
        t.advance_to(P0, Step::CombatDamage);
        t.settle();
        assert_eq!(t.life(P1), 15);
        assert_eq!(from_source(&t, odin), 1, "{leave}");
        if leave {
            crate::r_s05_common::move_to(&mut t, odin, Zone::Hand(P0));
        } else {
            crate::r_s06_common::give_control(&mut t, odin, P2);
        }
        t.resolve_all();
        assert!(t.has_lost(P1), "{leave}");
        assert!(!t.has_lost(P2));
    }
}

// --- Roshan, Hidden Magister ------------------------------------------------------------------

#[test]
fn turned_face_down_after_blocks_stays_blocked() {
    cr!("509.1h", "702.111b", "708.2");
    ruling!(
        "Roshan, Hidden Magister",
        "Once a creature you control has become blocked by a single creature, giving it menace by turning it face down somehow won’t cause it to become unblocked."
    );
    supported("Roshan, Hidden Magister");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Roshan, Hidden Magister");
    let giant = t.battlefield(P0, "Hill Giant");
    let elves = t.battlefield(P1, "Llanowar Elves");
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(elves, giant)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    assert!(mtg_engine::facedown::turn_face_down(&mut t.g, giant));
    t.g.recompute();
    assert!(crate::r_s06_common::has_kw(&t, giant, KeywordKind::Menace));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
}

// --- Grasping Shadows ---------------------------------------------------------------------------

#[test]
fn grasping_shadows_isnt_undone_by_creatures_entering_attacking() {
    cr!("506.5", "508.4");
    ruling!(
        "Grasping Shadows // Shadows' Lair",
        "Once the ability of Grasping Shadows has triggered, creatures entering the battlefield attacking won't affect that ability."
    );
    supported("Grasping Shadows // Shadows' Lair");
    let mut t = TestGame::new(2);
    let shadows = t.battlefield(P0, "Grasping Shadows // Shadows' Lair");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(from_source(&t, shadows), 1);
    // Another creature enters attacking.
    let giant = t.hand(P0, "Hill Giant");
    let mut to = Destination::battlefield();
    to.attacking = true;
    crate::r_s03_common::run_effect(
        &mut t,
        None,
        P0,
        Effect::Move {
            what: Sel::Target(0),
            to,
        },
        &[obj(giant)],
    );
    assert!(crate::r_s10_common::attacking(&t, giant));
    t.resolve_all();
    assert!(crate::r_s06_common::has_kw(&t, bears, KeywordKind::Deathtouch));
    assert!(crate::r_s06_common::has_kw(&t, bears, KeywordKind::Lifelink));
    assert_eq!(t.counters(shadows, "dread"), 1);
}

// --- Hazoret, Godseeker ------------------------------------------------------------------------

#[test]
fn hazoret_unblockable_stays_after_power_grows() {
    cr!("611.2c", "509.1b");
    ruling!(
        "Hazoret, Godseeker",
        "After the activated ability resolves, the creature can’t be blocked this turn even if its power later increases to 3 or greater."
    );
    let mut t = TestGame::new(2);
    let hazoret = t.battlefield(P0, "Hazoret, Godseeker");
    t.lands(P0, "Mountain", 1);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wall = t.battlefield(P1, "Craw Wurm");
    crate::r_p087_common::activate(&mut t, P0, hazoret, "can't be blocked", &[obj(bears)]);
    t.resolve_all();
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::ModifyPT(Value::c(3), Value::c(3))],
    );
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(!crate::r_s21_common::legal_blocks(&mut t, P1, &[(wall, bears)]));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 15);
}

// --- Shessra, Death's Whisper ------------------------------------------------------------------

#[test]
fn shessras_block_requirement_yields_to_tapped_cant_block_and_costs() {
    cr!("509.1c", "509.1d");
    ruling!(
        "Shessra, Death's Whisper",
        "If the creature affected by the Bewitching Whispers ability is tapped or is affected by a spell or ability that says it can't block, then it doesn't block. If there's a cost associated with having that creature block, its controller isn't forced to pay that cost, so it doesn't have to block in that case either."
    );
    supported("Shessra, Death's Whisper");
    supported("War Cadence");
    // `setup` prepares P1's creature; then Shessra enters targeting it and P0's Hill Giant
    // attacks. Returns whether P1 may decline to block.
    fn may_decline(blocker: &str, setup: impl FnOnce(&mut TestGame, ObjectId)) -> bool {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let b = t.battlefield(P1, blocker);
        setup(&mut t, b);
        t.answer_targets(P0, &[obj(b)]);
        t.enter(P0, "Shessra, Death's Whisper");
        t.resolve_all();
        attack_with(&mut t, &[(giant, Entity::Player(P1))]);
        t.resolve_all();
        crate::r_s21_common::legal_blocks(&mut t, P1, &[])
    }
    // Untapped and able: it must block.
    assert!(!may_decline("Grizzly Bears", |_, _| {}));
    // Tapped.
    assert!(may_decline("Grizzly Bears", |t, b| {
        t.g.objects[b.0 as usize].tapped = true;
    }));
    // Craven Giant: "This creature can't block."
    assert!(may_decline("Craven Giant", |_, _| {}));
    // War Cadence: "{X}{R}: This turn, creatures can't block unless their controller pays
    // {X} for each blocking creature they control." Activated for X = 1; P1 keeps two
    // untapped Mountains, enough to pay for the block, and still needn't block.
    assert!(may_decline("Grizzly Bears", |t, b| {
        let cadence = t.battlefield(P1, "War Cadence");
        t.lands(P1, "Mountain", 2);
        t.set_step(P1, Step::PrecombatMain);
        t.answer(P1, DecisionKind::X, Answer::Number(1));
        crate::r_s06_common::activate_containing(t, P1, cadence, "can't block").unwrap();
        t.resolve_all();
        t.lands(P1, "Mountain", 2);
        t.set_step(P0, Step::PrecombatMain);
        assert!(mtg_engine::combat::required_block_cost(&t.g, b).is_some());
    }));
}

// --- Tymna the Weaver ------------------------------------------------------------------------

#[test]
fn tymna_triggers_in_each_postcombat_main_phase() {
    cr!("505.1a", "500.8", "603.2");
    ruling!(
        "Tymna the Weaver",
        "If an effect creates an additional combat phase in a turn, it may also create an additional main phase after that combat phase. Tymna's ability triggers at the beginning of each of these postcombat main phases."
    );
    supported("Relentless Assault");
    // Relentless Assault: "Untap all creatures that attacked this turn. After this main
    // phase, there is an additional combat phase followed by an additional main phase."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tymna the Weaver");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(bears, Entity::Player(P1))], &[]);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!((t.hand_size(P0), t.life(P0)), (1, 19));
    cast_spell(&mut t, P0, "Relentless Assault", &[]);
    t.answer_yes(P0, true);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(bears, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::EndOfCombat);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!((t.hand_size(P0), t.life(P0)), (2, 18));
}

