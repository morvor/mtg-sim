//! Rulings batch P082 — combat interactions of abilities that care about blocks: blocks
//! that stay put (CR 509.1h, 506.4), triggers whose effects outlast their sources or the
//! block (CR 603.7, 611.2), APNAP order of block triggers (CR 101.4, 603.3b), banding
//! (CR 702.22h), and "blocked or was blocked this turn" histories.

use crate::r_p082_common::*;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::destroy;
use crate::r_s04_common::add_mana;
use crate::r_s21_common::{blocks_now, legal_blocks};
use crate::r_s25_common::cast_new;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// ---------------------------------------------------------------------------------
// Attacking a planeswalker: its controller is the defending player.
// ---------------------------------------------------------------------------------

#[test]
fn defending_player_is_the_attacked_planeswalkers_controller() {
    cr!("506.2", "508.5");
    ruling!(
        "Labyrinth Raptor",
        "If a creature is attacking a planeswalker, the planeswalker’s controller is the defending player."
    );
    ruling!(
        "Slivdrazi Monstrosity",
        "If a creature is attacking a planeswalker, the planeswalker’s controller is the defending player."
    );
    // Three players; P2 controls the planeswalker P0 attacks.
    // Labyrinth Raptor (menace): blocked by two of P2's creatures, P2 sacrifices one.
    supported("Labyrinth Raptor");
    let mut t = TestGame::new(3);
    let raptor = t.battlefield(P0, "Labyrinth Raptor");
    let pw = t.battlefield(P2, "Ajani Goldmane");
    let b1 = t.battlefield(P2, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    crate::r_s03_common::to_blockers(&mut t, &[(raptor, obj(pw))], &[(b1, raptor), (b2, raptor)]);
    t.resolve_all();
    assert_eq!(
        [b1, b2].iter().filter(|b| t.on_battlefield(**b)).count(),
        1,
        "P2 sacrificed a blocker"
    );
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    // Slivdrazi Monstrosity (annihilator 1): P2 sacrifices a permanent.
    supported("Slivdrazi Monstrosity");
    let mut t = TestGame::new(3);
    let sliv = t.battlefield(P0, "Slivdrazi Monstrosity");
    let pw = t.battlefield(P2, "Ajani Goldmane");
    let p2_land = t.battlefield(P2, "Plains");
    let p1_land = t.battlefield(P1, "Plains");
    t.answer_choose(P2, &[obj(p2_land)]);
    attack_with(&mut t, &[(sliv, obj(pw))]);
    t.resolve_all();
    assert!(!t.on_battlefield(p2_land));
    assert!(t.on_battlefield(p1_land) && t.on_battlefield(pw));
}

// ---------------------------------------------------------------------------------
// Blocks stay put.
// ---------------------------------------------------------------------------------

#[test]
fn shrinking_an_attacker_doesnt_undo_a_block_sidar_kondo_restricts() {
    cr!("509.1h", "509.1b");
    ruling!(
        "Sidar Kondo of Jamuraa",
        "Once an attacking creature has been blocked by a creature without flying or reach, reducing its power to 2 or less won't change or undo that block."
    );
    supported("Sidar Kondo of Jamuraa");
    supported("Disfigure");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sidar Kondo of Jamuraa");
    let giant = t.battlefield(P0, "Hill Giant");
    let ox = t.battlefield(P1, "Centaur Courser");
    blocked(&mut t, &[giant], &[(ox, giant)]);
    cast_new(&mut t, P0, "Disfigure", &[obj(giant)]);
    t.resolve_all();
    assert_eq!(t.pt(giant), (1, 1));
    assert!(is_blocked(&t, giant));
    assert!(blocks_now(&t).contains(&(ox, giant)));
}

#[test]
fn sidar_kondo_restriction_applies_when_another_player_attacks() {
    cr!("509.1b", "802.1");
    ruling!(
        "Sidar Kondo of Jamuraa",
        "Sidar Kondo's blocking restriction applies to creatures your opponents control even when an opponent is being attacked by a player other than you."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Sidar Kondo of Jamuraa");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ox = t.battlefield(P2, "Centaur Courser");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P2))]);
    assert!(!legal_blocks(&mut t, P2, &[(ox, bears)]));
    assert!(legal_blocks(&mut t, P2, &[]));
}

#[test]
fn tapping_a_blocker_doesnt_remove_it_from_combat() {
    cr!("506.4", "510.1");
    ruling!(
        "Master of Arms",
        "Tapping the blocker does not remove it from combat and does not stop it from dealing or receiving damage."
    );
    supported("Master of Arms");
    let mut t = TestGame::new(2);
    let (m, bs) = block_with(&mut t, "Master of Arms", &["Hill Giant"]);
    add_mana(&mut t, P0, ManaType::W, 2);
    t.activate(P0, m, 0, &[obj(bs[0])]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(bs[0]).tapped);
    finish_combat(&mut t);
    // First strike: the Master deals 2 to the Giant, then the tapped Giant deals 3 back.
    assert_eq!(t.obj_now(bs[0]).damage, 2);
    assert!(!t.on_battlefield(m));
}

#[test]
fn menace_gained_after_blocks_does_nothing() {
    cr!("509.1h", "702.111b");
    ruling!(
        "Searing Spear Askari",
        "The ability only does something if used before blockers are declared. You can use it afterwards but to no useful effect."
    );
    // After blocks: still blocked by a single creature.
    let mut t = TestGame::new(2);
    let (a, bs) = block_with(&mut t, "Searing Spear Askari", &["Hill Giant"]);
    add_mana(&mut t, P0, ManaType::R, 2);
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    assert!(is_blocked(&t, a));
    assert!(blocks_now(&t).contains(&(bs[0], a)));
    // Before blocks: one creature can't block it.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Searing Spear Askari");
    let g = t.battlefield(P1, "Hill Giant");
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    add_mana(&mut t, P0, ManaType::R, 2);
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!legal_blocks(&mut t, P1, &[(g, a)]));
}

// ---------------------------------------------------------------------------------
// Once triggered, the trigger doesn't care what happens next.
// ---------------------------------------------------------------------------------

#[test]
fn dead_iron_sledge_moved_after_the_trigger_still_destroys_both() {
    cr!("603.2", "608.2b");
    ruling!(
        "Dead-Iron Sledge",
        "Moving Dead-Iron Sledge after the ability triggers will not affect which creatures are destroyed."
    );
    supported("Dead-Iron Sledge");
    supported("Brass Squire");
    let mut t = TestGame::new(2);
    let squire = t.battlefield(P0, "Brass Squire");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let sledge = t.battlefield(P0, "Dead-Iron Sledge");
    t.g.attach(sledge, obj(bears));
    t.settle();
    let giant = t.battlefield(P1, "Hill Giant");
    blocked(&mut t, &[bears], &[(giant, bears)]);
    assert_eq!(triggers_from(&t, sledge), 1);
    // In response, Brass Squire moves the Sledge onto itself.
    t.answer_targets(P0, &[obj(sledge)]);
    t.answer_targets(P0, &[obj(squire)]);
    t.activate(P0, squire, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(t.obj_now(sledge).attached_to, Some(obj(squire)));
    t.resolve_all();
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(giant));
    assert!(t.on_battlefield(squire));
}

#[test]
fn seifer_deathtouch_stays_after_blockers_leave_and_works_for_any_attacker() {
    cr!("509.3e", "611.2a");
    ruling!(
        "Seifer, Balamb Rival",
        "Once Seifer's last ability has triggered, it doesn't matter what happens to the blocking creatures. The creature will keep deathtouch even if one or more of the creatures blocking it are removed from combat or leave the battlefield."
    );
    ruling!(
        "Seifer, Balamb Rival",
        "Seifer's last ability triggers regardless of which player controls the attacking creatures."
    );
    supported("Seifer, Balamb Rival");
    let mut t = TestGame::new(2);
    let seifer = t.battlefield(P0, "Seifer, Balamb Rival");
    let giant = t.battlefield(P0, "Hill Giant");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    blocked(&mut t, &[giant], &[(b1, giant), (b2, giant)]);
    assert_eq!(triggers_from(&t, seifer), 1);
    destroy(&mut t, b1);
    t.resolve_all();
    assert!(t.obj_now(giant).chars.has_keyword(KeywordKind::Deathtouch));
    destroy(&mut t, b2);
    assert!(t.obj_now(giant).chars.has_keyword(KeywordKind::Deathtouch));
    // Three players: P1 attacks P2 (one of P0's opponents); P2 double-blocks.
    let mut t = TestGame::new(3);
    let seifer = t.battlefield(P0, "Seifer, Balamb Rival");
    let giant = t.battlefield(P1, "Hill Giant");
    let b1 = t.battlefield(P2, "Grizzly Bears");
    let b2 = t.battlefield(P2, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    crate::r_s03_common::to_blockers(
        &mut t,
        &[(giant, Entity::Player(P2))],
        &[(b1, giant), (b2, giant)],
    );
    assert_eq!(triggers_from(&t, seifer), 1);
    t.resolve_all();
    assert!(t.obj_now(giant).chars.has_keyword(KeywordKind::Deathtouch));
}

#[test]
fn flanking_resolves_even_if_the_sliver_lost_flanking() {
    cr!("702.25a", "603.4");
    ruling!(
        "Sidewinder Sliver",
        "Once a Sliver has become blocked by a creature without flanking, the blocking creature gets -1/-1 even if the blocked Sliver loses flanking before the ability resolves."
    );
    supported("Sidewinder Sliver");
    supported("Metallic Sliver");
    let mut t = TestGame::new(2);
    let side = t.battlefield(P0, "Sidewinder Sliver");
    let sliver = t.battlefield(P0, "Metallic Sliver");
    let giant = t.battlefield(P1, "Hill Giant");
    blocked(&mut t, &[sliver], &[(giant, sliver)]);
    assert_eq!(triggers_from(&t, sliver), 1);
    destroy(&mut t, side);
    assert!(!t.obj_now(sliver).chars.has_keyword(KeywordKind::Flanking));
    t.resolve_all();
    assert_eq!(t.pt(giant), (2, 2));
}

#[test]
fn nahiris_machinations_target_can_be_destroyed_in_response() {
    cr!("603.3", "117.3c");
    ruling!(
        "Nahiri's Machinations",
        "Players can respond to the triggered ability of Nahiri's Machinations by destroying the creature you target before the ability that gives it indestructible has resolved."
    );
    supported("Nahiri's Machinations");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Nahiri's Machinations");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    cast_new(&mut t, P1, "Doom Blade", &[obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------------
// Effects that outlast their sources or the block.
// ---------------------------------------------------------------------------------

#[test]
fn aisling_leprechaun_color_change_outlasts_it() {
    cr!("611.2a", "611.2c");
    ruling!(
        "Aisling Leprechaun",
        "The change to green does not end even if the Leprechaun leaves the battlefield."
    );
    supported("Aisling Leprechaun");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let lep = t.battlefield(P1, "Aisling Leprechaun");
    blocked(&mut t, &[giant], &[(lep, giant)]);
    t.resolve_all();
    finish_combat(&mut t);
    assert!(!t.on_battlefield(lep));
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(giant).chars.colors, ColorSet::single(Color::Green));
}

#[test]
fn abomination_checks_color_only_as_the_block_is_declared() {
    cr!("509.3f", "603.7");
    ruling!(
        "Abomination",
        "The color of the blocking/blocked creature is checked only at declaration of the block. If the creature is green or white at that time, it will be destroyed at end of combat even if it changes color before combat ends."
    );
    supported("Abomination");
    supported("Cerulean Wisps");
    // Green when blocked, blue afterward: still destroyed.
    let mut t = TestGame::new(2);
    let courser = t.battlefield(P0, "Centaur Courser");
    let abom = t.battlefield(P1, "Abomination");
    blocked(&mut t, &[courser], &[(abom, courser)]);
    assert_eq!(triggers_from(&t, abom), 1);
    t.resolve_all();
    cast_new(&mut t, P0, "Cerulean Wisps", &[obj(courser)]);
    t.resolve_all();
    assert_eq!(t.obj_now(courser).chars.colors, ColorSet::single(Color::Blue));
    finish_combat(&mut t);
    t.resolve_all();
    assert!(!t.on_battlefield(courser));
    // Red when blocked: no trigger, even if it turns green later.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let abom = t.battlefield(P1, "Abomination");
    let lep = t.battlefield(P1, "Aisling Leprechaun");
    let bears = t.battlefield(P0, "Grizzly Bears");
    blocked(&mut t, &[giant, bears], &[(abom, giant), (lep, bears)]);
    assert_eq!(triggers_from(&t, abom), 0);
    t.resolve_all();
    finish_combat(&mut t);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
}

#[test]
fn tolarian_entrancer_control_change_is_permanent() {
    cr!("603.7c", "611.2a");
    ruling!(
        "Tolarian Entrancer",
        "The control change has no duration; you’ll retain control of that creature until the game ends or until some other effect causes it to change control. It doesn’t matter if Tolarian Entrancer leaves the battlefield, or if Tolarian Entrancer has already left the battlefield by the time the “at end of combat” ability triggers."
    );
    let mut t = TestGame::new(2);
    let (e, bs) = block_with(&mut t, "Tolarian Entrancer", &["Grizzly Bears"]);
    t.resolve_all();
    finish_combat(&mut t);
    t.resolve_all();
    assert!(!t.on_battlefield(e), "the Entrancer died in combat");
    assert_eq!(t.obj_now(bs[0]).controller, P0);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.obj_now(bs[0]).controller, P0);
}

#[test]
fn sawtooth_ogre_deals_its_damage_even_if_it_left() {
    cr!("603.7c", "609.7");
    ruling!(
        "Sawtooth Ogre",
        "The damage happens even if this card is not on the battlefield at the end of combat."
    );
    // Sawtooth Ogre (3/3) blocked by Craw Wurm (6/4): the Ogre dies, the Wurm has 3
    // damage; 1 more at end of combat kills it.
    let mut t = TestGame::new(2);
    let (o, bs) = block_with(&mut t, "Sawtooth Ogre", &["Craw Wurm"]);
    t.resolve_all();
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert!(!t.on_battlefield(o));
    assert!(t.on_battlefield(bs[0]));
    finish_combat(&mut t);
    t.resolve_all();
    assert!(!t.on_battlefield(bs[0]));
}

#[test]
fn giant_shark_keeps_its_bonus_and_counts_band_members() {
    cr!("509.3f", "611.2c", "702.22h");
    ruling!(
        "Giant Shark",
        "The bonus happens when a previously damaged creature becomes a blocker and the bonus is not removed if that creature is removed from the blocking situation by any means."
    );
    ruling!(
        "Giant Shark",
        "If a creature enters a blocking situation with the Giant Shark due to being part of a banded group of attackers, the Shark will get the bonus if that creature is damaged at that time."
    );
    supported("Giant Shark");
    // The Shark blocks a damaged Hill Giant; the Giant then leaves.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let island = t.battlefield(P1, "Island");
    crate::r_s06_common::damage(&mut t, island, 1, giant);
    let shark = t.battlefield(P1, "Giant Shark");
    blocked(&mut t, &[giant], &[(shark, giant)]);
    assert_eq!(triggers_from(&t, shark), 1);
    t.resolve_all();
    assert_eq!(t.pt(shark), (6, 4));
    destroy(&mut t, giant);
    assert_eq!(t.pt(shark), (6, 4));
    assert!(t.obj_now(shark).chars.has_keyword(KeywordKind::Trample));
    // A band of Benalish Hero (undamaged) and the damaged Hill Giant: the Shark blocks the
    // Hero, and the Giant becomes blocked by the Shark too.
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Benalish Hero");
    let giant = t.battlefield(P0, "Hill Giant");
    let island = t.battlefield(P1, "Island");
    crate::r_s06_common::damage(&mut t, island, 1, giant);
    let shark = t.battlefield(P1, "Giant Shark");
    t.answer_choose(P0, &[obj(giant)]);
    blocked(&mut t, &[hero, giant], &[(shark, hero)]);
    assert!(is_blocked(&t, giant));
    assert_eq!(triggers_from(&t, shark), 1);
    t.resolve_all();
    assert_eq!(t.pt(shark), (6, 4));
}

// ---------------------------------------------------------------------------------
// Laccolith Rig.
// ---------------------------------------------------------------------------------

#[test]
fn laccolith_rig_and_a_laccolith_both_trigger() {
    cr!("603.2", "510.1a");
    ruling!(
        "Laccolith Rig",
        "If placed on a Laccolith creature, both this card’s ability and the Laccolith’s ability will trigger and have their effect."
    );
    supported("Laccolith Rig");
    supported("Laccolith Grunt");
    let mut t = TestGame::new(2);
    let grunt = t.battlefield(P0, "Laccolith Grunt");
    let rig = t.battlefield(P0, "Laccolith Rig");
    t.g.attach(rig, obj(grunt));
    t.settle();
    let wurm = t.battlefield(P1, "Craw Wurm");
    let e1 = t.battlefield(P1, "Llanowar Elves");
    let e2 = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[obj(e1)]);
    t.answer_targets(P0, &[obj(e2)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    blocked(&mut t, &[grunt], &[(wurm, grunt)]);
    assert_eq!(triggers_from(&t, grunt) + triggers_from(&t, rig), 2);
    t.resolve_all();
    assert!(!t.on_battlefield(e1) && !t.on_battlefield(e2));
    finish_combat(&mut t);
    assert_eq!(t.obj_now(wurm).damage, 0, "the Grunt assigned no combat damage");
}

#[test]
fn laccolith_rig_on_an_opponents_creature_is_its_controllers_choice() {
    cr!("603.3a", "113.8");
    ruling!(
        "Laccolith Rig",
        "If this is on an opponent’s creature, the controller of this card (not the creature’s controller) decides whether to use this card’s ability or not, and decides what the target will be."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let rig = t.battlefield(P0, "Laccolith Rig");
    t.g.attach(rig, obj(bears));
    t.settle();
    let elves = t.battlefield(P1, "Llanowar Elves");
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, Step::BeginningOfCombat);
    t.answer_targets(P0, &[obj(elves)]);
    t.answer_yes(P0, true);
    let from = t.asked().len();
    crate::r_s03_common::to_blockers(&mut t, &[(bears, Entity::Player(P0))], &[(giant, bears)]);
    t.resolve_all();
    let asked = t.asked()[from..].to_vec();
    assert!(!asked
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, mtg_engine::decision::Decision::YesNo { .. })));
    assert!(!t.on_battlefield(elves), "the Bears dealt 2 damage to P1's own Elves");
    t.advance_to(P1, Step::EndOfCombat);
    assert_eq!(t.obj_now(giant).damage, 0);
}

// ---------------------------------------------------------------------------------
// Outrider en-Kor.
// ---------------------------------------------------------------------------------

#[test]
fn outrider_redirection_needs_its_target_and_keeps_the_source() {
    cr!("615.7", "609.7", "120.3");
    ruling!(
        "Outrider en-Kor",
        "If the targeted creature leaves the battlefield after the activated ability resolves but before Outrider en-Kor would be dealt damage, the damage isn't redirected away from Outrider en-Kor. The same is true if the target is no longer a creature (or a planeswalker) at that time."
    );
    ruling!(
        "Outrider en-Kor",
        "The damage dealt to the target is dealt by the original source of damage, not by Outrider en-Kor."
    );
    supported("Outrider en-Kor");
    // Redirected: Shock deals 1 to Outrider and 1 to the Giant — dealt by Shock.
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Outrider en-Kor");
    let giant = t.battlefield(P0, "Hill Giant");
    t.activate(P0, o, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    let shock = cast_new(&mut t, P1, "Shock", &[obj(o)]);
    t.resolve_all();
    assert!(t.on_battlefield(o));
    assert_eq!(t.obj_now(giant).damage, 1);
    let to_giant: Vec<ObjectId> = t
        .g
        .turn_events
        .iter()
        .filter_map(|e| match e {
            mtg_engine::events::Event::Damage { source, target, .. } if *target == obj(giant) => {
                Some(*source)
            }
            _ => None,
        })
        .collect();
    assert_eq!(to_giant.len(), 1);
    assert_ne!(t.g.current(to_giant[0]), t.g.current(o));
    assert_eq!(
        t.g.obj(to_giant[0]).chars.name,
        t.g.obj(shock).chars.name,
        "Shock dealt it"
    );
    // The target left: no redirection, Outrider takes all 2.
    let mut t = TestGame::new(2);
    let o = t.battlefield(P0, "Outrider en-Kor");
    let giant = t.battlefield(P0, "Hill Giant");
    t.activate(P0, o, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    destroy(&mut t, giant);
    cast_new(&mut t, P1, "Shock", &[obj(o)]);
    t.resolve_all();
    assert!(!t.on_battlefield(o));
}

// ---------------------------------------------------------------------------------
// Shape Stealer.
// ---------------------------------------------------------------------------------

#[test]
fn shape_stealer_copies_on_resolution_and_then_is_fixed() {
    cr!("608.2h", "613.4b");
    ruling!(
        "Shape Stealer",
        "The other creature's power and toughness are determined as the triggered ability resolves."
    );
    ruling!(
        "Shape Stealer",
        "Once the ability resolves, changing the other creature's power or toughness doesn't affect Shape Stealer."
    );
    supported("Shape Stealer");
    // Pumped in response: Shape Stealer becomes 5/5.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shape Stealer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocked(&mut t, &[s], &[(bears, s)]);
    cast_new(&mut t, P1, "Giant Growth", &[obj(bears)]);
    t.resolve();
    t.resolve_all();
    assert_eq!(t.pt(s), (5, 5));
    // Pumped afterwards: Shape Stealer stays 2/2.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shape Stealer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    blocked(&mut t, &[s], &[(bears, s)]);
    t.resolve_all();
    assert_eq!(t.pt(s), (2, 2));
    cast_new(&mut t, P1, "Giant Growth", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.pt(s), (2, 2));
}

#[test]
fn shape_stealer_blocked_by_several_ends_with_the_last_resolving() {
    cr!("509.3d", "603.3b", "613.4b");
    ruling!(
        "Shape Stealer",
        "This ability triggers once for each creature blocked by or blocking Shape Stealer. If multiple creatures block it, Shape Stealer's power and toughness will change for each one in succession. The first trigger put on the stack will be the last to resolve, so that will set Shape Stealer's final power and toughness."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shape Stealer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    blocked(&mut t, &[s], &[(bears, s), (giant, s)]);
    assert_eq!(triggers_from(&t, s), 2);
    t.resolve();
    let first = t.pt(s);
    t.resolve();
    let last = t.pt(s);
    assert!(
        (first, last) == ((2, 2), (3, 3)) || (first, last) == ((3, 3), (2, 2)),
        "{first:?} then {last:?}"
    );
    t.resolve_all();
    assert_eq!(t.pt(s), last);
}

#[test]
fn shape_stealer_and_bushido_depend_on_whose_turn_it_is() {
    cr!("101.4", "603.3b", "702.45a");
    ruling!(
        "Shape Stealer",
        "If the other creature has bushido, whether Shape Stealer gets the bushido bonus depends on whose turn it is. If Shape Stealer is blocking, it won't get the attacker's bushido bonus. If the Shape Stealer is being blocked, the bushido ability will resolve just before Shape Stealer determines what the creature's power and toughness are."
    );
    supported("Devoted Retainer");
    // Shape Stealer blocks Devoted Retainer (1/1, bushido 1): the defending player's
    // Shape Stealer trigger is on top and resolves first — it becomes 1/1.
    let mut t = TestGame::new(2);
    let retainer = t.battlefield(P0, "Devoted Retainer");
    let s = t.battlefield(P1, "Shape Stealer");
    blocked(&mut t, &[retainer], &[(s, retainer)]);
    t.resolve_all();
    assert_eq!(t.pt(retainer), (2, 2));
    assert_eq!(t.pt(s), (1, 1));
    // Shape Stealer attacks and is blocked by the Retainer: bushido resolves first, so
    // Shape Stealer becomes 2/2.
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Shape Stealer");
    let retainer = t.battlefield(P1, "Devoted Retainer");
    blocked(&mut t, &[s], &[(retainer, s)]);
    t.resolve_all();
    assert_eq!(t.pt(retainer), (2, 2));
    assert_eq!(t.pt(s), (2, 2));
}

// ---------------------------------------------------------------------------------
// Anzrag's block requirement.
// ---------------------------------------------------------------------------------

fn anzrag_must_be_blocked(t: &mut TestGame) -> ObjectId {
    supported("Anzrag, the Quake-Mole");
    let a = t.battlefield(P0, "Anzrag, the Quake-Mole");
    add_mana(t, P0, ManaType::R, 2);
    add_mana(t, P0, ManaType::G, 2);
    add_mana(t, P0, ManaType::C, 3);
    t.activate(P0, a, 0, &[]).unwrap();
    t.resolve_all();
    a
}

#[test]
fn anzrag_needs_only_one_blocker() {
    cr!("509.1c");
    ruling!(
        "Anzrag, the Quake-Mole",
        "If you activate Anzrag, the Quake-Mole's last ability, only one creature is required to block it each combat this turn. Other creatures may also block it and are free to block other creatures or not block at all."
    );
    let mut t = TestGame::new(2);
    let a = anzrag_must_be_blocked(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(a, Entity::Player(P1)), (bears, Entity::Player(P1))]);
    assert!(!legal_blocks(&mut t, P1, &[]));
    assert!(!legal_blocks(&mut t, P1, &[(b1, bears)]));
    assert!(legal_blocks(&mut t, P1, &[(b1, a)]));
    assert!(legal_blocks(&mut t, P1, &[(b1, a), (b2, a)]));
    assert!(legal_blocks(&mut t, P1, &[(b1, a), (b2, bears)]));
}

#[test]
fn anzrag_unblockable_requirement_is_ignored_when_no_creature_can_block() {
    cr!("509.1c", "509.1d");
    ruling!(
        "Anzrag, the Quake-Mole",
        "If you activate Anzrag, the Quake-Mole's last ability, but each creature the defending player controls can't block for any reason, Anzrag isn't blocked. If there's a cost associated with blocking Anzrag, the defending player isn't forced to pay that cost, so it doesn't have to be blocked in that case either."
    );
    // Tapped: can't block.
    let mut t = TestGame::new(2);
    let a = anzrag_must_be_blocked(&mut t);
    let b = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[b.0 as usize].tapped = true;
    attack_with(&mut t, &[(a, Entity::Player(P1))]);
    assert!(legal_blocks(&mut t, P1, &[]));
    // A cost to block (Archangel of Tithes attacking too): P1 doesn't block.
    supported("Archangel of Tithes");
    let mut t = TestGame::new(2);
    let a = anzrag_must_be_blocked(&mut t);
    let angel = t.battlefield(P0, "Archangel of Tithes");
    t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Wastes", 1);
    attack_with(
        &mut t,
        &[(a, Entity::Player(P1)), (angel, Entity::Player(P1))],
    );
    t.answer(
        P1,
        DecisionKind::Blockers,
        mtg_engine::decision::Answer::Blockers(vec![]),
    );
    crate::r_s21_common::go_to(&mut t, Step::DeclareBlockers);
    assert!(blocks_now(&t).is_empty());
    assert!(!is_blocked(&t, a));
}

// ---------------------------------------------------------------------------------
// Sisters of Stone Death.
// ---------------------------------------------------------------------------------

#[test]
fn sisters_return_any_creature_card_they_exiled_but_not_other_cards() {
    cr!("607.2a", "406.6");
    ruling!(
        "Sisters of Stone Death",
        "The third ability can get back any creature card exiled by the second ability, no matter when it was exiled. If a card that’s not a creature card (such as Svogthos, the Restless Tomb) was exiled, the third ability can’t return it."
    );
    supported("Sisters of Stone Death");
    supported("Mutavault");
    let mut t = TestGame::new(2);
    let sisters = t.battlefield(P0, "Sisters of Stone Death");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let vault = t.battlefield(P1, "Mutavault");
    add_mana(&mut t, P1, ManaType::C, 1);
    crate::r_s06_common::activate_containing(&mut t, P1, vault, "becomes").unwrap();
    t.resolve_all();
    blocked(&mut t, &[sisters], &[(bears, sisters), (vault, sisters)]);
    for b in [bears, vault] {
        add_mana(&mut t, P0, ManaType::B, 1);
        add_mana(&mut t, P0, ManaType::G, 1);
        t.activate(P0, sisters, 1, &[obj(b)]).unwrap();
        t.resolve_all();
        assert_eq!(t.zone(b), Zone::Exile);
    }
    // Two turns later, the Bears come back; Mutavault (a land card) can't.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    for _ in 0..2 {
        add_mana(&mut t, P0, ManaType::B, 1);
        add_mana(&mut t, P0, ManaType::C, 2);
        t.activate(P0, sisters, 2, &[]).unwrap();
        t.resolve_all();
    }
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.zone(vault), Zone::Exile);
}

// ---------------------------------------------------------------------------------
// End of combat and "blocked this turn".
// ---------------------------------------------------------------------------------

#[test]
fn heat_stroke_waits_for_end_of_combat() {
    cr!("511.2", "603.2");
    ruling!(
        "Heat Stroke",
        "This doesn’t trigger until the end of combat."
    );
    supported("Heat Stroke");
    let mut t = TestGame::new(2);
    let hs = t.battlefield(P1, "Heat Stroke");
    let (g, bs) = block_with(&mut t, "Hill Giant", &["Palace Guard"]);
    assert_eq!(triggers_from(&t, hs), 0);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    assert_eq!(triggers_from(&t, hs), 0);
    assert!(t.on_battlefield(g) && t.on_battlefield(bs[0]));
    finish_combat(&mut t);
    t.settle();
    assert_eq!(triggers_from(&t, hs), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(g) && !t.on_battlefield(bs[0]));
}

#[test]
fn venomous_breath_destroys_creatures_that_blocked_even_if_no_longer_blocking() {
    cr!("603.7", "506.4");
    ruling!(
        "Venomous Breath",
        "It affects creatures that had blocked the target when this spell resolves, even if they are no longer blocking the creature."
    );
    supported("Venomous Breath");
    supported("Reconnaissance");
    let mut t = TestGame::new(2);
    let recon = t.battlefield(P0, "Reconnaissance");
    let (g, bs) = block_with(&mut t, "Hill Giant", &["Centaur Courser"]);
    cast_new(&mut t, P0, "Venomous Breath", &[obj(g)]);
    t.resolve_all();
    // The Giant is removed from combat: the Courser isn't blocking it anymore.
    t.activate(P0, recon, 0, &[obj(g)]).unwrap();
    t.resolve_all();
    assert!(blocks_now(&t).is_empty());
    finish_combat(&mut t);
    t.resolve_all();
    assert!(!t.on_battlefield(bs[0]));
    assert!(t.on_battlefield(g));
}

#[test]
fn blocked_this_turn_targets_count_past_blocks() {
    cr!("115.1", "509.1");
    ruling!(
        "Sizzling Barrage",
        "Sizzling Barrage can target a creature that’s currently blocking or one that blocked earlier in the turn and is no longer blocking."
    );
    supported("Sizzling Barrage");
    let mut t = TestGame::new(2);
    let (_, bs) = block_with(&mut t, "Hill Giant", &["Palace Guard"]);
    let idle = t.battlefield(P1, "Centaur Courser");
    // Currently blocking (castable now), and after combat.
    let card = t.hand(P0, "Sizzling Barrage");
    t.lands(P0, "Mountain", 2);
    assert!(crate::r_s21_common::castable(&mut t, P0, card));
    t.advance_to(P0, Step::PostcombatMain);
    let from = t.asked().len();
    t.cast(P0, card).target(obj(bs[0])).go();
    let cands = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(cands.iter().any(|c| c.contains(&obj(bs[0]))));
    assert!(!cands.iter().any(|c| c.contains(&obj(idle))));
    t.resolve_all();
    assert!(!t.on_battlefield(bs[0]));
}

/// P0's `attacker` (a real card) is blocked by P1's Palace Guard (1/4); returns both.
fn attacked_and_blocked(t: &mut TestGame, attacker: &str) -> (ObjectId, ObjectId) {
    let (a, bs) = block_with(t, attacker, &["Palace Guard"]);
    (a, bs[0])
}

#[test]
fn time_to_reflect_targets_creatures_that_fought_a_zombie_this_turn() {
    cr!("115.1", "608.2b", "509.1");
    ruling!(
        "Time to Reflect",
        "Time to Reflect can target a creature that’s currently blocking or being blocked by a Zombie, or one that blocked or was blocked by a Zombie earlier in the turn."
    );
    ruling!(
        "Time to Reflect",
        "Time to Reflect only cares that the second creature was a Zombie at the moment it blocked or became blocked by the target creature. If that Zombie has become a non-Zombie creature or left the battlefield, Time to Reflect can still target the first creature."
    );
    supported("Time to Reflect");
    // During combat: the Bears blocking Walking Corpse (a Zombie).
    let mut t = TestGame::new(2);
    let (_, bears) = attacked_and_blocked(&mut t, "Walking Corpse");
    cast_new(&mut t, P0, "Time to Reflect", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    // After combat, with the Zombie gone.
    let mut t = TestGame::new(2);
    let (corpse, bears) = attacked_and_blocked(&mut t, "Walking Corpse");
    t.advance_to(P0, Step::PostcombatMain);
    destroy(&mut t, corpse);
    cast_new(&mut t, P0, "Time to Reflect", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    // Not a Zombie: no legal target.
    let mut t = TestGame::new(2);
    let (_, bears) = attacked_and_blocked(&mut t, "Hill Giant");
    t.lands(P0, "Plains", 1);
    let c = t.hand(P0, "Time to Reflect");
    assert!(t.cast_with(P0, c, &[obj(bears)]).is_err());
}

#[test]
fn you_cannot_pass_targets_creatures_that_fought_a_legend_this_turn() {
    cr!("115.1", "608.2b", "509.1");
    ruling!(
        "You Cannot Pass!",
        "You Cannot Pass! can target a creature that's currently blocking or being blocked by a legendary creature, or one that blocked or was blocked by a legendary creature earlier in the turn."
    );
    ruling!(
        "You Cannot Pass!",
        "You Cannot Pass! only cares that the second creature was a legendary creature at the moment it blocked or became blocked by the target creature. If that legendary creature has become a nonlegendary creature (perhaps because it's no longer your Ring-bearer) or left the battlefield, You Cannot Pass! can still target the first creature."
    );
    supported("You Cannot Pass!");
    supported("Isamaru, Hound of Konda");
    let mut t = TestGame::new(2);
    let (_, bears) = attacked_and_blocked(&mut t, "Isamaru, Hound of Konda");
    cast_new(&mut t, P0, "You Cannot Pass!", &[obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    // After combat, with the legend gone.
    let mut t = TestGame::new(2);
    let (isamaru, guard) = attacked_and_blocked(&mut t, "Isamaru, Hound of Konda");
    t.advance_to(P0, Step::PostcombatMain);
    destroy(&mut t, isamaru);
    cast_new(&mut t, P0, "You Cannot Pass!", &[obj(guard)]);
    t.resolve_all();
    assert!(!t.on_battlefield(guard));
}

// ---------------------------------------------------------------------------------
// Tide of War.
// ---------------------------------------------------------------------------------

#[test]
fn tide_of_war_triggers_once_only_with_blockers_and_spares_unblocked() {
    cr!("509.3a", "705.2");
    ruling!(
        "Tide of War",
        "Tide of War’s ability triggers only once each combat and only if at least one blocker is declared. The effect does nothing to unblocked attacking creatures."
    );
    supported("Tide of War");
    // No blockers: no trigger.
    let mut t = TestGame::new(2);
    let tide = t.battlefield(P0, "Tide of War");
    let g = t.battlefield(P0, "Hill Giant");
    blocked(&mut t, &[g], &[]);
    assert_eq!(triggers_from(&t, tide), 0);
    // Two blockers on one attacker, another attacker unblocked; P0 loses the flip: the
    // blocked attacker is sacrificed, the unblocked one isn't.
    let mut t = TestGame::new(2);
    let tide = t.battlefield(P0, "Tide of War");
    let g = t.battlefield(P0, "Hill Giant");
    let free = t.battlefield(P0, "Grizzly Bears");
    let b1 = t.battlefield(P1, "Llanowar Elves");
    let b2 = t.battlefield(P1, "Llanowar Elves");
    crate::r_p017_common::load_coins(&mut t, &[false]);
    blocked(&mut t, &[g, free], &[(b1, g), (b2, g)]);
    assert_eq!(triggers_from(&t, tide), 1);
    t.resolve_all();
    assert!(!t.on_battlefield(g));
    assert!(t.on_battlefield(free) && t.on_battlefield(b1) && t.on_battlefield(b2));
}

#[test]
fn tide_of_war_controller_flips_the_coin() {
    cr!("705.2", "603.3a");
    ruling!(
        "Tide of War",
        "Tide of War’s controller always flips the coin. This means that you want to win the flip when you’re attacking but lose the flip when you’re defending."
    );
    // P1 (the defending player) controls it and wins the flip: P1's blocker is sacrificed.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Tide of War");
    crate::r_p017_common::load_coins(&mut t, &[true]);
    let (g, bs) = block_with(&mut t, "Hill Giant", &["Centaur Courser"]);
    t.resolve_all();
    let flips: Vec<(PlayerId, bool)> = t
        .g
        .turn_events
        .iter()
        .filter_map(|e| match e {
            mtg_engine::events::Event::CoinFlipped { player, won, .. } => Some((*player, *won)),
            _ => None,
        })
        .collect();
    assert_eq!(flips, vec![(P1, true)]);
    assert!(!t.on_battlefield(bs[0]));
    assert!(t.on_battlefield(g));
}

// ---------------------------------------------------------------------------------
// Benefactor's Draught.
// ---------------------------------------------------------------------------------

#[test]
fn benefactors_draught_delayed_triggers_stack_and_need_casting_before_blocks() {
    cr!("603.7", "509.3a");
    ruling!(
        "Benefactor's Draught",
        "If multiple copies of Benefactor’s Draught resolve in one turn, each creates a delayed triggered ability and each will trigger when its condition is met."
    );
    ruling!(
        "Benefactor's Draught",
        "You must cast Benefactor’s Draught before blockers are declared in order to draw any cards from its delayed triggered ability."
    );
    supported("Benefactor's Draught");
    // Two resolved before combat: one block draws two cards.
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        cast_new(&mut t, P0, "Benefactor's Draught", &[]);
        t.resolve_all();
    }
    let hand = t.hand_size(P0);
    block_with(&mut t, "Hill Giant", &["Centaur Courser"]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // Cast after blockers: only its own "Draw a card".
    let mut t = TestGame::new(2);
    block_with(&mut t, "Hill Giant", &["Centaur Courser"]);
    t.resolve_all();
    cast_new(&mut t, P0, "Benefactor's Draught", &[]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    finish_combat(&mut t);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------------
// Roar of Jukai.
// ---------------------------------------------------------------------------------

#[test]
fn roar_of_jukai_can_be_cast_without_forest_or_blocked_creatures() {
    cr!("601.2", "608.2c");
    ruling!(
        "Roar of Jukai",
        "Roar of Jukai can be cast even if you don’t control a Forest or there are no blocked creatures. If you control no Forests or if there are no blocked creatures, the spell does nothing."
    );
    supported("Roar of Jukai");
    let roar = |t: &mut TestGame| {
        add_mana(t, P0, ManaType::G, 1);
        add_mana(t, P0, ManaType::C, 2);
        let c = t.hand(P0, "Roar of Jukai");
        t.cast(P0, c).go();
        t.resolve_all();
        assert!(t.in_graveyard(P0, "Roar of Jukai"));
    };
    // No Forest, a blocked creature: nothing.
    let mut t = TestGame::new(2);
    let (g, _) = block_with(&mut t, "Hill Giant", &["Grizzly Bears"]);
    roar(&mut t);
    assert_eq!(t.pt(g), (3, 3));
    // A Forest, but no blocked creatures (main phase): nothing.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    let g = t.battlefield(P0, "Hill Giant");
    roar(&mut t);
    assert_eq!(t.pt(g), (3, 3));
    // A Forest and a blocked creature: +2/+2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Forest");
    let (g, _) = block_with(&mut t, "Hill Giant", &["Grizzly Bears"]);
    roar(&mut t);
    assert_eq!(t.pt(g), (5, 5));
}

// ---------------------------------------------------------------------------------
// Knight of the Mists.
// ---------------------------------------------------------------------------------

#[test]
fn knight_of_the_mists_alone_destroys_itself_unless_paid() {
    cr!("603.3d", "115.1");
    ruling!(
        "Knight of the Mists",
        "If no other Knights are on the battlefield, pay the mana or destroy this card."
    );
    supported("Knight of the Mists");
    // Not paid: it's the only Knight, so it's destroyed.
    let mut t = TestGame::new(2);
    t.answer_yes(P0, false);
    let k = t.enter(P0, "Knight of the Mists");
    t.resolve_all();
    assert!(!t.on_battlefield(k));
    // Paid: it stays.
    let mut t = TestGame::new(2);
    add_mana(&mut t, P0, ManaType::U, 1);
    t.answer_yes(P0, true);
    let k = t.enter(P0, "Knight of the Mists");
    t.resolve_all();
    assert!(t.on_battlefield(k));
}

// ---------------------------------------------------------------------------------
// In the Darkness Bind Them.
// ---------------------------------------------------------------------------------

#[test]
fn in_the_darkness_bind_them_wraith_can_be_the_ring_bearer() {
    cr!("701.54a", "714.3b");
    ruling!(
        "In the Darkness Bind Them",
        "While resolving In the Darkness Bind Them's first three chapter abilities, you can choose the Wraith token you just created to be your Ring-bearer as the Ring tempts you."
    );
    supported("In the Darkness Bind Them");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    let from = t.asked().len();
    t.enter(P0, "In the Darkness Bind Them");
    t.resolve_all();
    let wraith = t
        .g
        .permanents()
        .find(|o| o.chars.has_subtype("Wraith"))
        .map(|o| o.id)
        .expect("a Wraith");
    let offered = t.asked()[from..].iter().any(|(p, d)| {
        *p == P0
            && matches!(d, mtg_engine::decision::Decision::ChooseEntities { candidates, .. }
                if candidates.contains(&obj(wraith)))
    });
    assert!(offered, "the Wraith was offered as Ring-bearer");
    assert!(t.g.player(P0).ring_bearer.is_some());
}
