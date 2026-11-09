//! Rulings batch P081 — "blocks or becomes blocked" triggers of creatures that punish
//! attackers and blockers: how many times they trigger (CR 509.3c–509.3e), that they
//! resolve in the declare blockers step before combat damage and leave the attacker
//! blocked (CR 509.1h, 506.4), the toughness they read (last known information,
//! CR 608.2h), and what happens when their source leaves (CR 112.7a, 610.3c).

use crate::r_p081_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::Color;
use mtg_engine::*;

#[test]
fn acolyte_of_the_inferno_pings_each_blocker_before_damage_and_stays_blocked() {
    cr!("509.3d", "509.1h", "506.4", "510.1a");
    ruling!(
        "Acolyte of the Inferno",
        "Acolyte of the Inferno's last ability will trigger once for each creature that blocks it. Each of those creatures will be dealt 2 damage."
    );
    ruling!(
        "Acolyte of the Inferno",
        "Acolyte of the Inferno's last ability triggers and resolves before combat damage is dealt. If that causes each creature blocking Acolyte of the Inferno to be destroyed, Acolyte of the Inferno will remain blocked and neither deal nor be dealt combat damage."
    );
    let mut t = TestGame::new(2);
    let (a, bs) = block_with(
        &mut t,
        "Acolyte of the Inferno",
        &["Grizzly Bears", "Grizzly Bears"],
    );
    assert_eq!(triggers_from(&t, a), 2);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert!(bs.iter().all(|b| died(&t, *b)));
    assert!(is_blocked(&t, a));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 20);
    assert_eq!(marked(&t, a), 0);
    assert!(t.on_battlefield(a));
}

/// "Whenever this creature blocks or becomes blocked by a creature, this creature deals N
/// damage to that creature": once per creature; a blocker destroyed by it leaves the
/// attacker blocked.
fn pings_each_blocker(t: &mut TestGame, a: ObjectId, bs: &[ObjectId]) {
    assert_eq!(triggers_from(t, a), bs.len());
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert!(bs.iter().all(|b| died(t, *b)));
    assert!(is_blocked(t, a));
    finish_combat(t);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn ashmouth_hound_and_kolaghan_aspirant_trigger_once_per_blocker() {
    cr!("509.3d", "509.1h");
    ruling!(
        "Ashmouth Hound",
        "Ashmouth Hound’s ability triggers once for each creature it blocks or becomes blocked by."
    );
    ruling!(
        "Kolaghan Aspirant",
        "If Kolaghan Aspirant becomes blocked by multiple creatures, its ability will trigger once for each of them."
    );
    for name in ["Ashmouth Hound", "Kolaghan Aspirant"] {
        let mut t = TestGame::new(2);
        let (a, bs) = block_with(&mut t, name, &["Llanowar Elves", "Llanowar Elves"]);
        pings_each_blocker(&mut t, a, &bs);
    }
    // Ashmouth Hound blocking: one trigger for the creature it blocks.
    let mut t = TestGame::new(2);
    let hound = t.battlefield(P0, "Ashmouth Hound");
    let elf = t.battlefield(P1, "Llanowar Elves");
    blocked_by_p0(&mut t, &[elf], &[(hound, elf)]);
    assert_eq!(triggers_from(&t, hound), 1);
    t.resolve_all();
    assert!(died(&t, elf));
}

#[test]
fn flameheart_werewolf_pings_each_blocker_before_damage() {
    cr!("509.3d", "509.1h", "506.4");
    ruling!(
        "Kessig Forgemaster // Flameheart Werewolf",
        "Flameheart Werewolf's first ability triggers once for each creature blocking or blocked by it. The ability resolves and deals damage to that creature before combat damage is dealt. If that damage destroys a creature blocking Flameheart Werewolf, Flameheart Werewolf doesn't become unblocked."
    );
    supported("Kessig Forgemaster // Flameheart Werewolf");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Kessig Forgemaster // Flameheart Werewolf");
    transform(&mut t, w);
    assert_eq!(t.pt(w), (3, 2), "the back face");
    let bs = [
        t.battlefield(P1, "Grizzly Bears"),
        t.battlefield(P1, "Grizzly Bears"),
    ];
    blocked(&mut t, &[w], &[(bs[0], w), (bs[1], w)]);
    pings_each_blocker(&mut t, w, &bs);
    assert!(t.on_battlefield(w));
}

#[test]
fn deeproot_warrior_and_amphibious_kavu_trigger_once() {
    cr!("509.3c");
    ruling!(
        "Deeproot Warrior",
        "Deeproot Warrior’s ability triggers only once if multiple creatures block it."
    );
    ruling!("Amphibious Kavu", "The ability only triggers once per combat.");
    let mut t = TestGame::new(2);
    let (w, _) = block_with(&mut t, "Deeproot Warrior", &["Grizzly Bears", "Grizzly Bears"]);
    assert_eq!(triggers_from(&t, w), 1);
    t.resolve_all();
    assert_eq!(t.pt(w), (3, 3));
    // Amphibious Kavu blocked by two blue creatures: +3/+3 once.
    let mut t = TestGame::new(2);
    let (k, _) = block_with(
        &mut t,
        "Amphibious Kavu",
        &["Merfolk of the Pearl Trident", "Merfolk of the Pearl Trident"],
    );
    assert_eq!(triggers_from(&t, k), 1);
    t.resolve_all();
    assert_eq!(t.pt(k), (5, 5));
}

#[test]
fn baneblade_scoundrel_shrinks_blockers_before_combat_damage() {
    cr!("509.1h", "510.1", "603.3");
    ruling!(
        "Baneblade Scoundrel // Baneclaw Marauder",
        "Abilities that trigger when a creature \"becomes blocked\" resolve before combat damage is dealt."
    );
    let mut t = TestGame::new(2);
    let (s, bs) = block_with(
        &mut t,
        "Baneblade Scoundrel // Baneclaw Marauder",
        &["Hill Giant"],
    );
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert_eq!(t.pt(bs[0]), (2, 2), "-1/-1 before damage");
    finish_combat(&mut t);
    // The 4/3 Scoundrel kills the 2/2 Giant and takes 2, not 3.
    assert!(died(&t, bs[0]));
    assert!(t.on_battlefield(s));
    assert_eq!(marked(&t, s), 2);
}

#[test]
fn plague_wight_shrinks_its_blockers_after_leaving_the_battlefield() {
    cr!("113.7a", "603.3", "608.2h");
    ruling!(
        "Plague Wight",
        "If Plague Wight leaves the battlefield while its triggered ability is on the stack, the creatures that were blocking it before it left the battlefield get -1/-1."
    );
    let mut t = TestGame::new(2);
    let (w, bs) = block_with(&mut t, "Plague Wight", &["Grizzly Bears", "Grizzly Bears"]);
    assert_eq!(triggers_from(&t, w), 1);
    // In response, P1 bolts the Wight.
    cast_new(&mut t, P1, "Lightning Bolt", &[obj(w)]);
    t.resolve();
    assert!(died(&t, w));
    t.resolve_all();
    assert!(bs.iter().all(|b| t.pt(*b) == (1, 1)));
}

#[test]
fn engulfing_slagwurm_gains_the_toughness_as_the_ability_resolves_or_last_known() {
    cr!("608.2h", "603.3", "701.15a");
    ruling!(
        "Engulfing Slagwurm",
        "As each ability resolves, the amount of life you gain is equal to the appropriate creature’s current toughness (if it’s somehow still on the battlefield), or its toughness as it last existed on the battlefield (in all other cases)."
    );
    ruling!(
        "Engulfing Slagwurm",
        "If Engulfing Slagwurm’s ability resolves and the other creature is not destroyed (perhaps because it has already left the battlefield or it regenerates), you’ll still gain life equal to that creature’s toughness."
    );
    // Current toughness: the blocker grows in response.
    let mut t = TestGame::new(2);
    let (w, bs) = block_with(&mut t, "Engulfing Slagwurm", &["Hill Giant"]);
    cast_new(&mut t, P1, "Giant Growth", &[obj(bs[0])]);
    t.resolve();
    assert_eq!(t.pt(bs[0]), (6, 6));
    t.resolve_all();
    assert!(died(&t, bs[0]));
    assert_eq!(t.life(P0), 26);
    assert!(is_blocked(&t, w));
    // Last known toughness: the blocker is bolted in response.
    let mut t = TestGame::new(2);
    let (_, bs) = block_with(&mut t, "Engulfing Slagwurm", &["Grizzly Bears"]);
    cast_new(&mut t, P0, "Lightning Bolt", &[obj(bs[0])]);
    t.resolve();
    assert!(died(&t, bs[0]));
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // The blocker regenerates: not destroyed, life still gained.
    let mut t = TestGame::new(2);
    t.lands(P1, "Swamp", 1);
    let (_, bs) = block_with(&mut t, "Engulfing Slagwurm", &["Drudge Skeletons"]);
    t.activate(P1, bs[0], 0, &[]).unwrap();
    t.resolve();
    t.resolve_all();
    assert!(t.on_battlefield(bs[0]));
    assert!(tapped(&t, bs[0]), "regenerated");
    assert_eq!(t.life(P0), 21);
}

#[test]
fn engulfing_slagwurm_destroys_blockers_in_the_declare_blockers_step_and_needs_trample() {
    cr!("509.1h", "506.4", "702.19d", "510.1a");
    ruling!(
        "Engulfing Slagwurm",
        "Engulfing Slagwurm’s ability triggers and resolves during the declare blockers step. Creatures destroyed this way will not deal combat damage."
    );
    ruling!(
        "Engulfing Slagwurm",
        "Even if the ability of an attacking Engulfing Slagwurm destroys each creature blocking it, Engulfing Slagwurm won’t deal combat damage to the player or planeswalker it’s attacking unless it somehow gains trample."
    );
    let mut t = TestGame::new(2);
    let (w, bs) = block_with(&mut t, "Engulfing Slagwurm", &["Hill Giant"]);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert!(died(&t, bs[0]));
    assert!(is_blocked(&t, w));
    finish_combat(&mut t);
    assert_eq!(marked(&t, w), 0, "the destroyed blocker dealt no damage");
    assert_eq!(t.life(P1), 20, "blocked, no trample");
    // With trample (Rancor: +2/+0 and trample), all its damage goes to the player.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Engulfing Slagwurm");
    crate::r_s06_common::attach_new(&mut t, P0, "Rancor", w);
    let b = t.battlefield(P1, "Hill Giant");
    blocked(&mut t, &[w], &[(b, w)]);
    t.resolve_all();
    assert!(died(&t, b));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 11);
}

#[test]
fn grasping_giant_stays_blocked_and_doesnt_exile_if_it_left_first() {
    cr!("509.1h", "506.4", "610.3c", "702.19d");
    ruling!(
        "Grasping Giant",
        "Grasping Giant doesn’t become unblocked when the creatures blocking it are exiled. Unless it gains trample, it won’t deal combat damage."
    );
    ruling!(
        "Grasping Giant",
        "If Grasping Giant leaves the battlefield before its triggered ability resolves, the blocking creature won’t be exiled."
    );
    let mut t = TestGame::new(2);
    let (g, bs) = block_with(&mut t, "Grasping Giant", &["Grizzly Bears"]);
    t.resolve_all();
    assert_eq!(t.zone(bs[0]), Zone::Exile);
    assert!(is_blocked(&t, g));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 20);
    // With trample, the damage all goes to the player.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Grasping Giant");
    crate::r_s06_common::attach_new(&mut t, P0, "Rancor", g);
    let b = t.battlefield(P1, "Grizzly Bears");
    blocked(&mut t, &[g], &[(b, g)]);
    t.resolve_all();
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 13);
    // The Giant leaves before its trigger resolves: the blocker stays.
    let mut t = TestGame::new(2);
    let (g, bs) = block_with(&mut t, "Grasping Giant", &["Grizzly Bears"]);
    cast_new(&mut t, P1, "Murder", &[obj(g)]);
    t.resolve();
    assert!(died(&t, g));
    t.resolve_all();
    assert!(t.on_battlefield(bs[0]));
}

#[test]
fn grazilaxx_returns_a_blocked_creature_before_damage() {
    cr!("509.1h", "510.1");
    ruling!(
        "Grazilaxx, Illithid Scholar",
        "Grazilaxx's first ability triggers before combat damage is dealt. If you choose to return the creature to your hand, it won't deal or receive combat damage."
    );
    supported("Grazilaxx, Illithid Scholar");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grazilaxx, Illithid Scholar");
    let (a, bs) = block_with(&mut t, "Grizzly Bears", &["Hill Giant"]);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.g.turn.step, Step::DeclareBlockers);
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(!t.on_battlefield(a));
    finish_combat(&mut t);
    assert_eq!(marked(&t, bs[0]), 0);
}

#[test]
fn shape_stealer_dies_when_its_new_toughness_is_below_its_damage() {
    cr!("704.5g", "613.4b", "613.7");
    ruling!(
        "Shape Stealer",
        "If Shape Stealer's new toughness is less than any damage on Shape Stealer, Shape Stealer will be destroyed as a state-based action once the triggered ability resolves."
    );
    supported("Relentless Assault");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shape Stealer");
    let spider = t.battlefield(P1, "Giant Spider");
    let elf = t.battlefield(P1, "Llanowar Elves");
    // First combat: blocked by the 2/4 Spider, Shape Stealer becomes 2/4 and takes 2.
    blocked(&mut t, &[s], &[(spider, s)]);
    t.resolve_all();
    assert_eq!(t.pt(s), (2, 4));
    finish_combat(&mut t);
    assert_eq!(marked(&t, s), 2);
    assert!(t.on_battlefield(s));
    // Relentless Assault: untap, additional combat. Blocked by the 1/1 Elves, it becomes
    // 1/1 with 2 damage on it.
    t.advance_to(P0, Step::PostcombatMain);
    cast_new(&mut t, P0, "Relentless Assault", &[]);
    t.resolve_all();
    blocked(&mut t, &[s], &[(elf, s)]);
    assert_eq!(triggers_from(&t, s), 1);
    t.resolve_all();
    assert!(died(&t, s));
    assert!(t.on_battlefield(elf));
}

#[test]
fn gorgon_recluse_destroys_at_end_of_combat_even_if_gone_or_the_creature_turns_black() {
    cr!("603.7a", "603.7c", "608.2h");
    ruling!(
        "Gorgon Recluse",
        "Gorgon Recluse's ability triggers when it blocks or becomes blocked by a nonblack creature and creates a delayed triggered ability that will destroy that creature at the end of combat step. It destroys that creature then even if Gorgon Recluse doesn't survive that long or the creature somehow becomes black."
    );
    // The Recluse is bolted in response to its trigger.
    let mut t = TestGame::new(2);
    let (r, bs) = block_with(&mut t, "Gorgon Recluse", &["Hill Giant"]);
    assert_eq!(triggers_from(&t, r), 1);
    cast_new(&mut t, P1, "Lightning Bolt", &[obj(r)]);
    cast_new(&mut t, P1, "Lightning Bolt", &[obj(r)]);
    t.resolve();
    t.resolve();
    assert!(died(&t, r));
    t.resolve_all();
    assert!(t.on_battlefield(bs[0]));
    finish_combat(&mut t);
    t.settle();
    t.resolve_all();
    assert!(died(&t, bs[0]));
    // The blocker becomes black (Darkest Hour) before end of combat.
    let mut t = TestGame::new(2);
    let (_, bs) = block_with(&mut t, "Gorgon Recluse", &["Hill Giant"]);
    t.resolve_all();
    t.battlefield(P1, "Darkest Hour");
    t.g.recompute();
    assert!(colors(&t, bs[0]).contains(&Color::Black));
    finish_combat(&mut t);
    t.resolve_all();
    assert!(died(&t, bs[0]));
    // A black creature doesn't trigger it at all.
    let mut t = TestGame::new(2);
    let (r, _) = block_with(&mut t, "Gorgon Recluse", &["Black Knight"]);
    assert_eq!(triggers_from(&t, r), 0);
}

#[test]
fn giant_shark_gets_the_bonus_per_creature_damaged_before_blocks() {
    cr!("509.3d", "603.2", "603.2c");
    ruling!(
        "Giant Shark",
        "Giant Shark gets the bonus once for each creature it blocks or is blocked by that meets the criteria."
    );
    ruling!(
        "Giant Shark",
        "Giant Shark only gains its bonus if the blocking creature was damaged before blocking was declared. Damaging the creature later will not give the Shark the bonus."
    );
    supported("Giant Shark");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 1);
    t.lands(P1, "Island", 1);
    let shark = t.battlefield(P0, "Giant Shark");
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P1, "Hill Giant");
    // Two blockers damaged before combat, one not.
    for x in [a, b] {
        cast_new(&mut t, P0, "Shock", &[obj(x)]);
        t.resolve();
    }
    blocked(&mut t, &[shark], &[(a, shark), (b, shark), (c, shark)]);
    assert_eq!(triggers_from(&t, shark), 2);
    // Damaging the third blocker now is too late.
    cast_new(&mut t, P0, "Shock", &[obj(c)]);
    t.resolve();
    assert_eq!(marked(&t, c), 2);
    assert_eq!(triggers_from(&t, shark), 2);
    t.resolve_all();
    assert_eq!(t.pt(shark), (8, 4));
    assert!(has_keyword(&t, shark, KeywordKind::Trample));
}

#[test]
fn dream_fighter_phases_out_with_each_blocker_and_beats_flanking_to_it() {
    cr!("509.3d", "603.3b", "702.26b", "702.25a");
    ruling!(
        "Dream Fighter",
        "Dream Fighter affects all creatures that end up blocking it, including ones that are members of bands."
    );
    ruling!(
        "Dream Fighter",
        "If Dream Fighter is assigned to block a creature with Flanking, the Flanking ability and the Dream Fighter ability both trigger. The active player’s Flanking effect is put on the stack first and resolves after the Dream Fighter’s phasing out effect happen, so the Flanking does nothing."
    );
    let mut t = TestGame::new(2);
    let (f, bs) = block_with(&mut t, "Dream Fighter", &["Grizzly Bears", "Hill Giant"]);
    assert_eq!(triggers_from(&t, f), 2);
    t.resolve_all();
    assert!(phased_out(&t, f));
    assert!(bs.iter().all(|b| phased_out(&t, *b)));
    // Dream Fighter (P0's, the defending player) blocks P1's Knight of the Mists
    // (flanking). The flanking trigger goes on the stack first (APNAP), Dream Fighter's
    // on top of it; the 1/1 Dream Fighter is phased out when flanking's -1/-1 would apply.
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Dream Fighter");
    let knight = t.battlefield(P1, "Knight of the Mists");
    blocked_by_p0(&mut t, &[knight], &[(f, knight)]);
    assert_eq!(t.stack_len(), 2);
    assert_eq!(triggers_from(&t, f), 1);
    assert_eq!(triggers_from(&t, knight), 1);
    t.resolve();
    assert!(phased_out(&t, f));
    assert!(phased_out(&t, knight));
    t.resolve_all();
    assert!(phased_out(&t, f), "not destroyed");
    assert!(!died(&t, f));
    // It phases in at P0's untap step as the 1/1 it is.
    t.advance_to(P0, Step::Upkeep);
    assert!(!phased_out(&t, f));
    assert_eq!(t.pt(f), (1, 1));
}

#[test]
fn jabaris_banner_flanking_granted_after_blocks_is_too_late() {
    cr!("702.25a", "603.2", "603.4");
    ruling!(
        "Jabari's Banner",
        "Giving a creature flanking after blockers have been declared won’t have any effect. Specifically: — If you give flanking to an attacking creature after blockers have been declared, it’s too late for flanking to trigger. — If you give flanking to a creature that’s blocking an attacking creature with flanking, it’s too late to prevent the blocking creature from getting -1/-1, even if the attacker’s flanking ability hasn’t resolved yet."
    );
    supported("Jabari's Banner");
    // Flanking given to the blocked attacker: no trigger.
    let mut t = TestGame::new(2);
    let banner = t.battlefield(P0, "Jabari's Banner");
    t.lands(P0, "Wastes", 1);
    let (a, bs) = block_with(&mut t, "Grizzly Bears", &["Grizzly Bears"]);
    assert_eq!(t.stack_len(), 0);
    t.activate(P0, banner, 0, &[obj(a)]).unwrap();
    t.resolve();
    assert!(has_keyword(&t, a, KeywordKind::Flanking));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.pt(bs[0]), (2, 2));
    // Flanking given to a blocker of a flanker in response to the flanking trigger: the
    // blocker still gets -1/-1.
    let mut t = TestGame::new(2);
    let banner = t.battlefield(P1, "Jabari's Banner");
    t.lands(P1, "Wastes", 1);
    let (k, bs) = block_with(&mut t, "Knight of the Mists", &["Hill Giant"]);
    assert_eq!(triggers_from(&t, k), 1);
    t.activate(P1, banner, 0, &[obj(bs[0])]).unwrap();
    t.resolve();
    assert!(has_keyword(&t, bs[0], KeywordKind::Flanking));
    t.resolve_all();
    assert_eq!(t.pt(bs[0]), (2, 2));
}

#[test]
fn knight_of_the_holy_nimbus_cant_be_regenerated_after_its_last_ability() {
    cr!("701.19c", "701.19b", "614.1c");
    ruling!(
        "Knight of the Holy Nimbus",
        "After the last ability resolves, regeneration shields don't apply if Knight of the Holy Nimbus would be destroyed for the remainder of the turn. This includes its own effect as well as regeneration shields from other sources."
    );
    supported("Knight of the Holy Nimbus");
    // Without the last ability: its own replacement regenerates it.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Knight of the Holy Nimbus");
    cast_new(&mut t, P1, "Lightning Bolt", &[obj(k)]);
    t.resolve();
    assert!(t.on_battlefield(k));
    assert!(tapped(&t, k));
    // After P1 activates it: neither its own effect nor a Regenerate shield applies.
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Knight of the Holy Nimbus");
    t.lands(P1, "Wastes", 2);
    assert!(
        t.activate(P0, k, 0, &[]).is_err(),
        "only opponents may activate it"
    );
    t.activate(P1, k, 0, &[]).unwrap();
    t.resolve();
    cast_new(&mut t, P0, "Regenerate", &[obj(k)]);
    t.resolve();
    cast_new(&mut t, P1, "Lightning Bolt", &[obj(k)]);
    t.resolve();
    assert!(died(&t, k));
}

#[test]
fn carnage_gladiator_triggers_for_any_players_blocker() {
    cr!("603.2");
    ruling!(
        "Carnage Gladiator",
        "Carnage Gladiator’s ability triggers whenever any creature blocks, regardless of who controls that creature or which creature it blocked."
    );
    // P0's Gladiator: P1's blocker costs P1 a life; on P1's turn, P0's own blocker costs
    // P0 a life.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Carnage Gladiator");
    let (a, bs) = block_with(&mut t, "Grizzly Bears", &["Hill Giant"]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert!(is_blocked(&t, a));
    let _ = bs;
    let mut t = TestGame::new(2);
    let glad = t.battlefield(P0, "Carnage Gladiator");
    let a = t.battlefield(P1, "Grizzly Bears");
    blocked_by_p0(&mut t, &[a], &[(glad, a)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 19);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn benefactors_draught_gives_priority_after_its_trigger_before_damage() {
    cr!("509.4", "117.3b", "603.7");
    ruling!(
        "Benefactor's Draught",
        "Each player will have a chance to cast instants and activate abilities after Benefactor’s Draught’s delayed triggered abilities have resolved, before combat damage is dealt."
    );
    supported("Benefactor's Draught");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    cast_new(&mut t, P0, "Benefactor's Draught", &[]);
    t.resolve();
    let hand = t.hand_size(P0);
    let seen0 = priority_steps(&mut t, P0);
    let seen1 = priority_steps(&mut t, P1);
    blocked(&mut t, &[a], &[(b, a)]);
    assert_eq!(t.stack_len(), 1, "the delayed trigger");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(marked(&t, a), 0);
    let before = (seen0.lock().unwrap().len(), seen1.lock().unwrap().len());
    finish_combat(&mut t);
    let got0 = &seen0.lock().unwrap()[before.0..];
    let got1 = &seen1.lock().unwrap()[before.1..];
    assert!(got0.contains(&Step::DeclareBlockers), "{got0:?}");
    assert!(got1.contains(&Step::DeclareBlockers), "{got1:?}");
    assert!(died(&t, a));
}

#[test]
fn anzrag_adds_a_combat_phase_each_time_it_resolves() {
    cr!("500.8", "509.3c");
    ruling!(
        "Anzrag, the Quake-Mole",
        "Anzrag, the Quake-Mole's first ability may trigger multiple times in the same turn. Each time it resolves, you create an additional combat phase."
    );
    supported("Anzrag, the Quake-Mole");
    let mut t = TestGame::new(2);
    let anz = t.battlefield(P0, "Anzrag, the Quake-Mole");
    let w1 = t.battlefield(P1, "Wall of Stone");
    let w2 = t.battlefield(P1, "Wall of Stone");
    blocked(&mut t, &[anz], &[(w1, anz)]);
    t.resolve_all();
    assert!(!tapped(&t, anz));
    finish_combat(&mut t);
    // A second combat phase: Anzrag attacks and is blocked again.
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![(anz, pl(P1))]));
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(w2, anz)]));
    t.advance_to(P0, Step::DeclareBlockers);
    assert_eq!(t.g.turn.combat_phases, 2);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    // A third one.
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(t.g.turn.combat_phases, 3);
}

#[test]
fn wiitigo_removes_a_counter_from_any_source() {
    cr!("122.1a");
    ruling!("Wiitigo", "Any +1/+1 counter from any source can be used with his ability.");
    supported("Wiitigo");
    let mut t = TestGame::new(2);
    let w = t.enter(P0, "Wiitigo");
    assert_eq!(t.counters(w, "+1/+1"), 6);
    // A seventh counter from another source.
    t.g.add_counters(obj(w), "+1/+1", 1, None);
    t.g.recompute();
    assert_eq!(t.pt(w), (7, 7));
    // It hasn't blocked or been blocked: a counter is removed at the upkeep.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(w, "+1/+1"), 6);
}

#[test]
fn godsend_that_left_doesnt_lock_the_names_a_new_one_didnt_exile() {
    cr!("400.7", "608.2h");
    ruling!(
        "Godsend",
        "If Godsend leaves the battlefield, and later another Godsend enters the battlefield, it is a new object (even if the two were represented by the same card). The new Godsend won't stop opponents from casting cards with the same name as cards exiled with the first one."
    );
    supported("Godsend");
    let mut t = TestGame::new(2);
    let gs = t.battlefield(P0, "Godsend");
    let a = t.battlefield(P0, "Hill Giant");
    assert!(t.g.attach(gs, obj(a)));
    let b = t.battlefield(P1, "Grizzly Bears");
    blocked(&mut t, &[a], &[(b, a)]);
    t.answer_yes(P1, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(b)]);
    t.resolve_all();
    assert_eq!(t.zone(b), Zone::Exile);
    finish_combat(&mut t);
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!can_cast_now(&mut t, P1, "Grizzly Bears"));
    // The first Godsend leaves; another enters.
    t.g.destroy(gs, None);
    t.settle();
    assert!(died(&t, gs));
    t.battlefield(P0, "Godsend");
    t.g.recompute();
    assert!(can_cast_now(&mut t, P1, "Grizzly Bears"));
}

#[test]
fn aisling_leprechaun_turns_the_creature_green_by_a_trigger_after_blocks() {
    cr!("509.1a", "702.16b", "603.2", "613.1d");
    ruling!(
        "Aisling Leprechaun",
        "Creatures change color as a triggered ability upon the block being declared."
    );
    ruling!(
        "Aisling Leprechaun",
        "A Leprechaun with Protection from Green can be blocked by a creature (since it is not green at that time) but will not take damage from the creature (since it is green during the combat damage step)."
    );
    supported("Aisling Leprechaun");
    supported("Shield of Duty and Reason");
    let mut t = TestGame::new(2);
    let lep = t.battlefield(P0, "Aisling Leprechaun");
    // Shield of Duty and Reason: protection from green and from blue.
    crate::r_s06_common::attach_new(&mut t, P0, "Shield of Duty and Reason", lep);
    t.g.recompute();
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(&mut t, &[(lep, pl(P1))]);
    t.resolve_all();
    assert!(!t.g.can_block(bears, lep), "a green creature can't block it");
    assert!(t.g.can_block(giant, lep));
    block_and_finish_with_check(&mut t, lep, giant);
}

/// P1 blocks `lep` with `giant`: the color change is a trigger; once it resolves the
/// blocker is green, and its combat damage to the protected Leprechaun is prevented.
fn block_and_finish_with_check(t: &mut TestGame, lep: ObjectId, giant: ObjectId) {
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(giant, lep)]));
    t.advance_to(P0, Step::DeclareBlockers);
    t.settle();
    assert_eq!(triggers_from(t, lep), 1);
    assert!(!colors(t, giant).contains(&Color::Green), "not green yet");
    t.resolve_all();
    assert_eq!(colors(t, giant), vec![Color::Green], "green, and only green");
    finish_combat(t);
    assert!(t.on_battlefield(lep));
    assert_eq!(marked(t, lep), 0, "damage from a green source is prevented");
    assert_eq!(marked(t, giant), 1);
}

#[test]
fn bill_ferny_removed_from_combat_stays_tapped() {
    cr!("506.4", "508.1f");
    ruling!(
        "Bill Ferny, Bree Swindler",
        "If Bill Ferny, Bree Swindler is removed from combat, it remains tapped if it was tapped."
    );
    supported("Bill Ferny, Bree Swindler");
    supported("Armored Warhorse");
    let mut t = TestGame::new(2);
    let bill = t.battlefield(P0, "Bill Ferny, Bree Swindler");
    let horse = t.battlefield(P0, "Armored Warhorse");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(horse)]);
    blocked(&mut t, &[bill], &[(b, bill)]);
    assert!(tapped(&t, bill));
    t.resolve_all();
    assert_eq!(t.obj_now(horse).controller, P1);
    assert!(!t.g.is_attacking(bill), "removed from combat");
    assert!(tapped(&t, bill));
    assert_eq!(tokens(&t, P0).len(), 3);
    finish_combat(&mut t);
    assert!(t.on_battlefield(bill));
    assert!(tapped(&t, bill));
}
