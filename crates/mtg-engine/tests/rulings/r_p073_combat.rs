//! Rulings batch P073 — combat: creatures put onto the battlefield attacking (Winota,
//! Basri Ket; CR 508.4), Akiri's attack trigger, block requirements (Roar of Challenge,
//! Destined // Lead; CR 509.1c), attack requirements (Daring Fiendbonder; CR 508.1d),
//! and Agent of the Shadow Thieves' life comparison.

use crate::r_p073_common::*;
use crate::r_s01_common::{attack_with, stack_library, supported};
use crate::r_s06_common::{activate_containing, attach_new};
use crate::r_s09_common::{legal_attack, to_combat};
use crate::r_s13_common::add;
use crate::r_s21_common::legal_blocks;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Whether the creature is an attacking creature now.
fn attacking(t: &TestGame, id: ObjectId) -> bool {
    let id = t.g.current(id);
    t.g.combat
        .as_ref()
        .is_some_and(|c| c.attackers.iter().any(|a| a.id == id))
}

/// What the creature is attacking now.
fn attacks(t: &TestGame, id: ObjectId) -> Option<Entity> {
    crate::r_s12_common::attack_target(t, t.g.current(id))
}

/// Answers each "Choose what the creature is attacking" prompt of `p` with `targets`, in
/// order.
fn choose_new_attack_targets(t: &mut TestGame, p: PlayerId, targets: &[Entity]) {
    for e in targets {
        t.answer_choose(p, &[*e]);
    }
}

// ---------------------------------------------------------------------------
// Winota, Joiner of Forces
// ---------------------------------------------------------------------------

/// P0's Winota and a non-Human (Grizzly Bears); the top of P0's library has `human` among
/// five fillers. Returns (Winota, Bears, the Human card).
fn winota(t: &mut TestGame, human: &str) -> (ObjectId, ObjectId, ObjectId) {
    let w = t.battlefield(P0, "Winota, Joiner of Forces");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lib = stack_library(t, P0, &[human]);
    (w, bears, lib[0])
}

#[test]
fn winotas_human_enters_attacking_without_attack_triggers() {
    cr!("508.4", "508.3a", "603.2");
    ruling!(
        "Winota, Joiner of Forces",
        "Although the Human is an attacking creature, it was never declared as an attacking creature. This means that abilities that trigger whenever a creature attacks won’t trigger when it enters the battlefield attacking."
    );
    supported("Winota, Joiner of Forces");
    supported("Hellrider");
    let mut t = TestGame::new(2);
    let (_, bears, human) = winota(&mut t, "Accorder Paladin");
    t.battlefield(P0, "Hellrider");
    t.answer_choose(P0, &[obj(human)]);
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.resolve_all();
    let paladin = t.g.current(human);
    assert_eq!(t.zone(paladin), Zone::Battlefield);
    assert!(attacking(&t, paladin) && t.obj_now(paladin).tapped);
    assert!(kw(&t, paladin, INDESTRUCTIBLE));
    // Hellrider triggered once (for the Bears), not for the Paladin; the Paladin's battle
    // cry didn't trigger (the Bears would be 3/2).
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.pt(bears), (2, 2));
}

#[test]
fn winotas_human_enters_attacking_despite_cant_attack_effects() {
    cr!("508.4", "508.4c");
    ruling!(
        "Winota, Joiner of Forces",
        "Any effects that say that the Human can’t attack (such as that of Propaganda, or if you get a Human with defender) affect only the declaration of attackers. They won’t stop the Human you find from entering the battlefield attacking."
    );
    supported("Propaganda");
    // A Human with defender on top of the library (the other cards Winota looks at aren't
    // Humans), and Propaganda: the two lands pay only for the attacking Bears.
    let mut t = TestGame::new(2);
    let (_, bears, _) = winota(&mut t, "Grizzly Bears");
    let def = crate::r_s01_common::custom_card(
        "Human Pikeman",
        "Creature — Human Soldier",
        "{1}{W}",
        Some((1, 4)),
        "Defender",
    );
    let pikeman = t.custom(P0, def, Zone::Library(P0));
    t.battlefield(P1, "Propaganda");
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[obj(pikeman)]);
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.resolve_all();
    let pikeman = t.g.current(pikeman);
    assert_eq!(t.zone(pikeman), Zone::Battlefield);
    assert!(attacking(&t, pikeman));
    assert_eq!(attacks(&t, pikeman), Some(pl(P1)));
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 2);
}

#[test]
fn winota_triggers_separately_for_each_attacker() {
    cr!("603.2", "603.2c", "508.3a");
    ruling!(
        "Winota, Joiner of Forces",
        "If more than one non-Human creature you control attacks, Winota’s ability triggers separately for each of them. For example, if three non-Human creatures attack, you won’t look at the top eighteen cards of your library at once"
    );
    let mut t = TestGame::new(2);
    let (_, bears, _) = winota(&mut t, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let giant = t.battlefield(P0, "Hill Giant");
    attack_with(&mut t, &[(bears, pl(P1)), (elves, pl(P1)), (giant, pl(P1))]);
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, "top six"), 3);
    // Each instance looks at six cards and puts them on the bottom before the next one
    // resolves: after the first, the old seventh card is on top.
    let top = crate::r_s15_common::library_top_n(&t, P0, 13);
    t.resolve();
    assert_eq!(crate::r_s01_common::triggers_on_stack(&t, "top six"), 2);
    assert_eq!(crate::r_s15_common::library_top_n(&t, P0, 1), vec![top[6]]);
    t.resolve();
    assert_eq!(crate::r_s15_common::library_top_n(&t, P0, 1), vec![top[12]]);
}

#[test]
fn winotas_human_may_attack_a_different_player() {
    cr!("508.4");
    ruling!(
        "Winota, Joiner of Forces",
        "You choose which player or planeswalker the Human is attacking. It doesn’t have to be attacking the same player or planeswalker that the non-Human creature is attacking."
    );
    let mut t = TestGame::new(3);
    let (_, bears, human) = winota(&mut t, "Accorder Paladin");
    t.answer_choose(P0, &[obj(human)]);
    choose_new_attack_targets(&mut t, P0, &[pl(P2)]);
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.resolve_all();
    let paladin = t.g.current(human);
    assert_eq!(attacks(&t, paladin), Some(pl(P2)));
    assert_eq!(attacks(&t, bears), Some(pl(P1)));
}

// ---------------------------------------------------------------------------
// Basri Ket
// ---------------------------------------------------------------------------

/// P0's Basri Ket (three loyalty), its −2 activated (and resolved) in P0's main phase.
fn basri_minus_two(t: &mut TestGame) -> ObjectId {
    let basri = t.battlefield(P0, "Basri Ket");
    activate_containing(t, P0, basri, "nontoken creatures attack").unwrap();
    t.resolve_all();
    basri
}

#[test]
fn basri_soldiers_enter_attacking_without_attack_triggers_and_despite_propaganda() {
    cr!("508.4", "508.3a", "508.4c", "603.7a");
    ruling!(
        "Basri Ket",
        "Although the Soldiers are attacking creatures, they were never declared as attacking creatures. This means that abilities that trigger whenever a creature attacks won't trigger when the Soldiers enter the battlefield attacking."
    );
    ruling!(
        "Basri Ket",
        "Any effects that say that the Soldiers can't attack (such as that of Propaganda) affect only the declaration of attackers. They won't stop the Soldier tokens from entering the battlefield attacking."
    );
    supported("Basri Ket");
    let mut t = TestGame::new(2);
    basri_minus_two(&mut t);
    t.battlefield(P0, "Hellrider");
    t.battlefield(P1, "Propaganda");
    t.lands(P0, "Wastes", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, pl(P1))]);
    t.resolve_all();
    let soldiers = crate::r_s05_common::tokens_with_subtype(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 1);
    assert!(attacking(&t, soldiers[0]) && t.obj_now(soldiers[0]).tapped);
    assert_eq!(t.life(P1), 19, "Hellrider triggered for the Bears only");
    t.advance_to(P0, Step::PostcombatMain);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn basri_delayed_trigger_counts_nontoken_attackers() {
    cr!("603.7a", "508.3a", "506.4");
    ruling!(
        "Basri Ket",
        "The delayed triggered ability of Basri's second ability triggers if any nontoken creatures attack, and it counts how many nontoken creatures attacked. Attacking nontoken creatures that leave combat before the triggered ability resolves are counted. Token creatures that are attacking don't stop the ability from triggering and aren't counted."
    );
    // Two nontoken attackers and a token; one nontoken one dies in response: two Soldiers.
    let mut t = TestGame::new(2);
    basri_minus_two(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let tok = crate::r_s02_common::create_token(&mut t, P0, "Warrior");
    t.g.objects[tok.0 as usize].summoning_sick = false;
    attack_with(&mut t, &[(bears, pl(P1)), (giant, pl(P1)), (tok, pl(P1))]);
    assert_eq!(t.stack_len(), 1);
    destroy_now(&mut t, giant);
    t.resolve_all();
    assert_eq!(
        crate::r_s05_common::tokens_with_subtype(&t, P0, "Soldier").len(),
        2
    );
    // Only tokens attack: it doesn't trigger.
    let mut t = TestGame::new(2);
    basri_minus_two(&mut t);
    let tok = crate::r_s02_common::create_token(&mut t, P0, "Warrior");
    t.g.objects[tok.0 as usize].summoning_sick = false;
    attack_with(&mut t, &[(tok, pl(P1))]);
    assert_eq!(t.stack_len(), 0);
    assert!(crate::r_s05_common::tokens_with_subtype(&t, P0, "Soldier").is_empty());
}

#[test]
fn basri_soldiers_choose_their_own_attack_targets() {
    cr!("508.4");
    ruling!(
        "Basri Ket",
        "You choose which player or planeswalker each Soldier token is attacking. They don't have to be attacking the same players or planeswalkers that the nontoken creatures are attacking, and they don't have to be attacking the same player or planeswalker as each other."
    );
    let mut t = TestGame::new(3);
    basri_minus_two(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    choose_new_attack_targets(&mut t, P0, &[pl(P2), pl(P1)]);
    attack_with(&mut t, &[(bears, pl(P1)), (giant, pl(P1))]);
    t.resolve_all();
    let soldiers = crate::r_s05_common::tokens_with_subtype(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 2);
    let mut targets: Vec<Entity> = soldiers.iter().filter_map(|s| attacks(&t, *s)).collect();
    targets.sort_by_key(|e| format!("{e:?}"));
    assert_eq!(targets, vec![pl(P1), pl(P2)]);
}

#[test]
fn basri_emblem_soldier_cant_attack_the_turn_its_created() {
    cr!("302.6", "114.4");
    ruling!(
        "Basri Ket",
        "The Soldier token created by Basri's last ability can't attack during the turn you create it unless it gains haste somehow."
    );
    let mut t = TestGame::new(2);
    let basri = t.battlefield(P0, "Basri Ket");
    add(&mut t, basri, "loyalty", 3);
    activate_containing(&mut t, P0, basri, "emblem").unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let soldiers = crate::r_s05_common::tokens_with_subtype(&t, P0, "Soldier");
    assert_eq!(soldiers.len(), 1);
    assert_eq!(t.pt(soldiers[0]), (2, 2));
    assert!(!legal_attack(&mut t, &[(soldiers[0], pl(P1))]));
}

// ---------------------------------------------------------------------------
// Akiri, Fearless Voyager
// ---------------------------------------------------------------------------

#[test]
fn akiri_draws_once_per_player_attacked_with_equipped_creatures() {
    cr!("508.3a", "603.2c");
    ruling!(
        "Akiri, Fearless Voyager",
        "Akiri's first ability has you draw just one card per player you attack with an equipped creature, no matter how many equipped creatures you attack them with beyond the first and no matter how many Equipment that creature's carrying beyond the first."
    );
    ruling!(
        "Akiri, Fearless Voyager",
        "Equipped creatures attacking a planeswalker or battle won't cause Akiri's first ability to trigger."
    );
    supported("Akiri, Fearless Voyager");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Akiri, Fearless Voyager");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let c = t.battlefield(P0, "Craw Wurm");
    let d = t.battlefield(P0, "Llanowar Elves");
    attach_new(&mut t, P0, "Bonesplitter", obj(a));
    attach_new(&mut t, P0, "Bonesplitter", obj(a));
    attach_new(&mut t, P0, "Bonesplitter", obj(b));
    attach_new(&mut t, P0, "Bonesplitter", obj(c));
    attach_new(&mut t, P0, "Bonesplitter", obj(d));
    let jace = t.battlefield(P2, "Jace Beleren");
    let hand = t.hand_size(P0);
    // Two equipped creatures at P1, one at P2, one at P2's planeswalker.
    attack_with(
        &mut t,
        &[(a, pl(P1)), (b, pl(P1)), (c, pl(P2)), (d, obj(jace))],
    );
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Only a planeswalker attacked: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Akiri, Fearless Voyager");
    let a = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Bonesplitter", obj(a));
    let jace = t.battlefield(P1, "Jace Beleren");
    attack_with(&mut t, &[(a, obj(jace))]);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn akiri_unattaches_the_chosen_equipment_on_resolution() {
    cr!("701.3d", "608.2c");
    ruling!(
        "Akiri, Fearless Voyager",
        "If the creature is already tapped, it just gains indestructible until end of turn."
    );
    ruling!(
        "Akiri, Fearless Voyager",
        "You choose which Equipment to unattach as Akiri's second ability resolves. Unattaching an Equipment from a creature won't affect other Equipment attached to that creature."
    );
    let mut t = TestGame::new(2);
    let akiri = t.battlefield(P0, "Akiri, Fearless Voyager");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let first = attach_new(&mut t, P0, "Bonesplitter", obj(bears));
    let second = attach_new(&mut t, P0, "Bonesplitter", obj(bears));
    t.g.tap(bears);
    t.lands(P0, "Plains", 1);
    let from = t.asked().len();
    activate_containing(&mut t, P0, akiri, "unattach").unwrap();
    // Chosen while it resolves, not as it's activated.
    let chosen = |t: &TestGame| {
        crate::r_s07_common::count_asked(t, from, |d| matches!(d, Decision::ChooseEntities { .. }))
    };
    assert_eq!(chosen(&t), 0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(second)]);
    t.resolve_all();
    assert_eq!(chosen(&t), 1);
    assert_eq!(t.obj_now(second).attached_to, None);
    assert_eq!(t.obj_now(first).attached_to, Some(obj(bears)));
    assert!(t.obj_now(bears).tapped);
    assert!(kw(&t, bears, INDESTRUCTIBLE));
    assert_eq!(t.pt(bears), (4, 2));
}

// ---------------------------------------------------------------------------
// Block requirements: Roar of Challenge, Destined // Lead
// ---------------------------------------------------------------------------

/// P0 attacks P1 with `attacker`, P0 having cast the "all creatures able to block it do
/// so" spell `lure` on it precombat; returns whether P1 declaring `blocks` is legal.
fn lured(t: &mut TestGame, lure: &str, attacker: ObjectId) {
    t.lands(P0, "Forest", 4);
    t.lands(P0, "Wastes", 2);
    if lure == "Destined // Lead" {
        // Lead is the aftermath half, cast from the graveyard.
        let c = t.graveyard(P0, lure);
        t.cast(P0, c)
            .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Aftermath))
            .target(obj(attacker))
            .go();
    } else {
        crate::r_p079_common::cast_card(t, P0, lure, &[obj(attacker)]).unwrap();
    }
    t.resolve_all();
    attack_with(t, &[(attacker, pl(P1))]);
}

#[test]
fn lure_spells_dont_force_tapped_or_cant_block_creatures() {
    cr!("509.1c", "509.1a");
    ruling!(
        "Roar of Challenge",
        "If, during the declare blockers step, a creature is tapped or is affected by a spell or ability that says it can’t block, then it doesn’t block."
    );
    ruling!(
        "Destined // Lead",
        "As blockers are declared, any creature that's tapped or affected by a spell or ability that says it can't block doesn't block."
    );
    supported("Roar of Challenge");
    supported("Destined // Lead");
    for lure in ["Roar of Challenge", "Destined // Lead"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let able = t.battlefield(P1, "Hill Giant");
        let tapped = t.battlefield(P1, "Grizzly Bears");
        t.g.tap(tapped);
        let raider = t.battlefield(P1, "Goblin Raider");
        lured(&mut t, lure, bears);
        assert!(!legal_blocks(&mut t, P1, &[]), "{lure}");
        assert!(legal_blocks(&mut t, P1, &[(able, bears)]), "{lure}");
        assert!(!legal_blocks(&mut t, P1, &[(raider, bears)]), "{lure}");
        assert!(!legal_blocks(&mut t, P1, &[(tapped, bears)]), "{lure}");
    }
}

#[test]
fn lure_spells_dont_grant_the_ability_to_block() {
    cr!("509.1c", "702.9b");
    ruling!(
        "Roar of Challenge",
        "Roar of Challenge doesn’t give any creatures the ability to block the target creature. It just forces those creatures that are already able to block the creature to do so."
    );
    ruling!(
        "Destined // Lead",
        "Lead doesn't give any creatures the ability to block the target creature. It just forces those creatures that are already able to block the creature to do so."
    );
    for lure in ["Roar of Challenge", "Destined // Lead"] {
        let mut t = TestGame::new(2);
        let angel = t.battlefield(P0, "Serra Angel");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let spider = t.battlefield(P1, "Giant Spider");
        lured(&mut t, lure, angel);
        assert!(legal_blocks(&mut t, P1, &[(spider, angel)]), "{lure}");
        assert!(!legal_blocks(&mut t, P1, &[(bears, angel)]), "{lure}");
        assert!(!legal_blocks(&mut t, P1, &[]), "{lure}: the Spider must block");
    }
}

// ---------------------------------------------------------------------------
// Daring Fiendbonder, Agent of the Shadow Thieves
// ---------------------------------------------------------------------------

#[test]
fn daring_fiendbonder_needn_t_attack_when_it_cant_or_it_costs() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Daring Fiendbonder",
        "If Daring Fiendbonder can't attack for any reason (such as being tapped), then it doesn't attack. If there's a cost associated with having it attack, the player isn't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    supported("Daring Fiendbonder");
    let mut t = TestGame::new(2);
    let fb = t.battlefield(P0, "Daring Fiendbonder");
    to_combat(&mut t, P0);
    assert!(!legal_attack(&mut t, &[]), "it must attack if able");
    t.g.tap(fb);
    assert!(legal_attack(&mut t, &[]));
    let mut t = TestGame::new(2);
    let fb = t.battlefield(P0, "Daring Fiendbonder");
    t.battlefield(P1, "Ghostly Prison");
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    to_combat(&mut t, P0);
    t.advance_to(P0, Step::DeclareAttackers);
    assert!(!attacking(&t, fb));
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 0);
}

#[test]
fn agent_of_the_shadow_thieves_compares_only_opponents_life() {
    cr!("903.3", "603.4");
    ruling!(
        "Agent of the Shadow Thieves",
        "The ability granted by Agent of the Shadow Thieves is looking to see if any of the attacking player's opponents have a higher life total than that defending player. The attacking player's life total is not included in this comparison."
    );
    supported("Agent of the Shadow Thieves");
    for (life, counters) in [([40, 30, 20], 1), ([10, 30, 20], 1), ([40, 20, 30], 0)] {
        let mut t = TestGame::new(3);
        t.battlefield(P0, "Agent of the Shadow Thieves");
        let cmd = bf_commander(&mut t, P0, "Grizzly Bears");
        for (i, l) in life.iter().enumerate() {
            t.g.players[i].life = *l;
        }
        attack_with(&mut t, &[(cmd, pl(P1))]);
        t.resolve_all();
        assert_eq!(t.counters(cmd, "+1/+1"), counters, "{life:?}");
        assert_eq!(kw(&t, cmd, INDESTRUCTIBLE), counters == 1);
    }
}
