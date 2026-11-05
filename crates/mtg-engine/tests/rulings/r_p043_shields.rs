//! Rulings batch P043 — prevention and redirection shields created by resolving spells
//! and abilities: "the next N damage that a source of your choice would deal" (Refraction
//! Trap, Divine Deflection, Harm's Way; CR 609.7a, 615.5, 615.7), redirection to a
//! target (Captain's Maneuver, Martyrdom, Kor Dirge, Nova Pentacle; CR 614.9), and
//! "prevent all damage this turn" (Avacyn, Guardian Angel, Safe Passage, Saving Grace).

use crate::r_p007_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s05_common::move_to;
use crate::r_s06_common::{attach_new, damage};
use crate::r_s25_common::{cast_new, targets_of};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::events::Event;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// The damage events dealt this turn by `source` (followed to its current object).
fn damage_by(t: &TestGame, source: ObjectId) -> Vec<(Entity, u32, bool)> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Damage {
                source: s,
                target,
                amount,
                combat,
            } if *s == source || t.g.current(*s) == t.g.current(source) => {
                Some((*target, *amount, *combat))
            }
            _ => None,
        })
        .collect()
}

/// P0 casts Refraction Trap (for its mana cost) at `target`, choosing `src` as it
/// resolves.
fn trap(t: &mut TestGame, src: ObjectId, target: Entity) -> ObjectId {
    let s = cast_new(t, P0, "Refraction Trap", &[target]);
    t.answer_choose(P0, &[obj(src)]);
    t.resolve();
    s
}

/// P0 casts Divine Deflection with X = `x` at `target` (it stays on the stack).
fn deflection(t: &mut TestGame, x: i64, target: Entity) -> ObjectId {
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", x as usize);
    let card = t.hand(P0, "Divine Deflection");
    t.cast(P0, card).x(x).target(target).go()
}

/// `src` deals `n` noncombat damage to `to` (as a resolving effect would), then SBAs.
fn hit(t: &mut TestGame, src: ObjectId, to: Entity, n: u32) {
    t.g.deal_damage(src, to, n, false);
    t.g.flush_events();
    t.settle();
}

#[test]
fn refraction_trap_deals_its_damage_within_the_prevention() {
    cr!("615.5", "615.7", "608.2", "704.3");
    ruling!(
        "Refraction Trap",
        "If the chosen source would deal damage, Refraction Trap prevents 3 of that source’s damage and the source deals its excess damage (if any) at the same time. Immediately afterward, as part of that same prevention effect, Refraction Trap deals its damage. This happens before state-based actions are checked, and before the spell or ability that caused damage to be dealt resumes its resolution."
    );
    supported("Refraction Trap");
    // 5 damage from Hill Giant: 3 prevented, 2 dealt, and the Trap deals 3 to P1 at once.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    trap(&mut t, giant, pl(P1));
    t.g.deal_damage(giant, pl(P0), 5, false);
    assert_eq!((t.life(P0), t.life(P1)), (18, 17));
    assert_eq!(t.stack_len(), 0, "no trigger");
    // P1, at 3 life, casts Lightning Helix at P0. The Trap (source: the Helix) deals 3 to
    // P1 during the Helix's resolution; the Helix then gains P1 3 life before SBAs.
    let mut t = TestGame::new(2);
    t.g.players[P1.idx()].life = 3;
    let helix = cast_new(&mut t, P1, "Lightning Helix", &[pl(P0)]);
    trap(&mut t, helix, pl(P1));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 3);
    assert!(
        !t.has_lost(P1),
        "P1 was at 0 only during the Helix's resolution"
    );
}

#[test]
fn refraction_trap_and_divine_deflection_fizzle_with_an_illegal_target() {
    cr!("608.2b", "615.7");
    ruling!(
        "Refraction Trap",
        "If the targeted permanent or player is an illegal target by the time Refraction Trap resolves, the entire spell doesn’t resolve. No damage will be prevented."
    );
    ruling!(
        "Divine Deflection",
        "If the targeted permanent or player is an illegal target by the time Divine Deflection resolves, the entire spell doesn’t resolve. No damage will be prevented."
    );
    supported("Divine Deflection");
    for name in ["Refraction Trap", "Divine Deflection"] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        if name == "Refraction Trap" {
            cast_new(&mut t, P0, name, &[obj(bears)]);
        } else {
            deflection(&mut t, 3, obj(bears));
        }
        destroy(&mut t, bears);
        t.answer_choose(P0, &[obj(giant)]);
        t.resolve_all();
        hit(&mut t, giant, pl(P0), 3);
        assert_eq!(t.life(P0), 17, "{name}: nothing prevented");
    }
}

#[test]
fn refraction_trap_and_divine_deflection_dont_recheck_their_target() {
    cr!("608.2b", "614.9", "615.5");
    ruling!(
        "Refraction Trap",
        "Whether the targeted permanent or player is still a legal target is no longer checked after Refraction Trap resolves. For example, if a creature targeted by Refraction Trap gains shroud after Refraction Trap resolves but before it prevents damage, Refraction Trap will still prevent damage and still deal damage to that creature. If Refraction Trap can’t deal damage to the targeted permanent or player (because the creature is no longer on the battlefield, or is no longer a creature, or the player is no longer in the game, for example), it will still prevent damage. It just won’t deal any damage itself."
    );
    ruling!(
        "Divine Deflection",
        "Whether the targeted permanent or player is still a legal target is not checked after Divine Deflection resolves. For example, if a creature targeted by Divine Deflection gains shroud after Divine Deflection resolves, Divine Deflection can still deal damage to that creature."
    );
    for name in ["Refraction Trap", "Divine Deflection"] {
        // The target gains shroud (Lightning Greaves): still dealt the damage.
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let wurm = t.battlefield(P1, "Craw Wurm");
        if name == "Refraction Trap" {
            trap(&mut t, giant, obj(wurm));
        } else {
            deflection(&mut t, 3, obj(wurm));
            t.resolve_all();
        }
        attach_new(&mut t, P1, "Lightning Greaves", wurm);
        assert!(t
            .obj_now(wurm)
            .has_keyword(mtg_engine::keywords::KeywordKind::Shroud));
        hit(&mut t, giant, pl(P0), 3);
        assert_eq!(t.life(P0), 20, "{name}");
        assert_eq!(dmg(&t, wurm), 3, "{name}: dealt to the shrouded creature");
        // The target left the battlefield: the damage is still prevented.
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        if name == "Refraction Trap" {
            trap(&mut t, giant, obj(bears));
        } else {
            deflection(&mut t, 3, obj(bears));
            t.resolve_all();
        }
        destroy(&mut t, bears);
        hit(&mut t, giant, pl(P0), 3);
        assert_eq!(t.life(P0), 20, "{name}: still prevented");
        assert_eq!(t.life(P1), 20);
    }
}

#[test]
fn refraction_trap_shields_you_your_creatures_and_your_planeswalkers() {
    cr!("615.7", "120.3c");
    ruling!(
        "Refraction Trap",
        "Refraction Trap can prevent damage that would be dealt to you, one or more creatures you control, and/or one or more planeswalkers you control."
    );
    for what in ["you", "creature", "planeswalker"] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let to = match what {
            "you" => pl(P0),
            "creature" => obj(t.battlefield(P0, "Craw Wurm")),
            _ => obj(pw(&mut t, P0, "Jace Beleren", 4)),
        };
        trap(&mut t, giant, pl(P1));
        hit(&mut t, giant, to, 3);
        assert_eq!(t.life(P1), 17, "{what}: the Trap dealt 3");
        match to {
            Entity::Player(p) => assert_eq!(t.life(p), 20),
            Entity::Object(o) => {
                assert_eq!(dmg(&t, o), 0, "{what}");
                assert_eq!(
                    t.counters(o, "loyalty"),
                    if what == "planeswalker" { 4 } else { 0 }
                );
            }
        }
    }
    // Several of them: one shield of 3 for all of them.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P0, "Craw Wurm");
    trap(&mut t, giant, pl(P1));
    hit(&mut t, giant, obj(wurm), 2);
    hit(&mut t, giant, pl(P0), 2);
    assert_eq!((dmg(&t, wurm), t.life(P0), t.life(P1)), (0, 19, 17));
}

#[test]
fn refraction_traps_source_is_chosen_on_resolution_and_its_target_on_cast() {
    cr!("609.7a", "601.2c", "118.9");
    ruling!(
        "Refraction Trap",
        "Refraction Trap doesn’t target the source of the damage it prevents. You choose that source as Refraction Trap resolves. If you cast Refraction Trap for {W}, the source you choose doesn’t have to be the red instant or sorcery spell an opponent cast."
    );
    ruling!(
        "Refraction Trap",
        "Refraction Trap’s only target is the permanent or player it may deal damage to. You choose that target as you cast Refraction Trap, not at the time it prevents damage."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P1 casts Shock (a red instant) at P0's Bears.
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    cast_new(&mut t, P1, "Shock", &[obj(bears)]);
    t.resolve_all();
    // P0 casts Refraction Trap for {W}, targeting P1.
    let card = t.hand(P0, "Refraction Trap");
    t.lands(P0, "Plains", 1);
    let alt = crate::r_s07_common::cast_methods(&mut t, P0, card)
        .into_iter()
        .find(|m| matches!(m, CastMethod::Alternative(_)))
        .expect("the {W} alternative cost");
    let from = t.asked().len();
    let spell = t.cast(P0, card).method(alt).target(P1).go();
    // The target is chosen; no source was chosen (or targeted) yet.
    assert_eq!(targets_of(&t, spell), vec![pl(P1)]);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
    // As it resolves, P0 chooses Hill Giant (not the Shock).
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve();
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && matches!(d, Decision::ChooseEntities { candidates, .. } if candidates.contains(&obj(giant)))));
    hit(&mut t, giant, pl(P0), 3);
    assert_eq!((t.life(P0), t.life(P1)), (20, 17));
}

#[test]
fn refraction_trap_is_the_source_of_its_noncombat_damage() {
    cr!("615.5", "702.2b", "702.15b", "120.2");
    ruling!(
        "Refraction Trap",
        "Refraction Trap’s effect is not a redirection effect. If it prevents damage, Refraction Trap (not the chosen source) deals damage to the targeted permanent or player as part of that prevention effect. Refraction Trap is the source of the new damage, so the characteristics of the original source (such as its color, or whether it had lifelink or deathtouch) don’t affect this damage. The new damage is not combat damage, even if the prevented damage was."
    );
    // Vampire Nighthawk (deathtouch, lifelink) deals 2 combat damage to P0; the Trap
    // deals 2 to P1's Craw Wurm instead: no deathtouch, no life gained.
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let spell = trap(&mut t, hawk, obj(wurm));
    t.g.deal_damage(hawk, pl(P0), 2, true);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20, "no lifelink");
    assert!(t.on_battlefield(wurm), "no deathtouch");
    assert_eq!(dmg(&t, wurm), 2);
    assert_eq!(damage_by(&t, spell), vec![(obj(wurm), 2, false)]);
    assert!(damage_by(&t, hawk).is_empty());
}

#[test]
fn harms_way_redirects_the_next_two_damage_over_the_turn() {
    cr!("614.9", "615.7");
    ruling!(
        "Harm's Way",
        "If the chosen source would deal just 1 damage to you or a permanent you control, Harm’s Way’s effect will redirect that damage and still have a “shield” left for another 1 damage from that source later in the turn."
    );
    supported("Harm's Way");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_new(&mut t, P0, "Harm's Way", &[pl(P1)]);
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve_all();
    hit(&mut t, giant, pl(P0), 1);
    assert_eq!((t.life(P0), t.life(P1)), (20, 19));
    hit(&mut t, giant, obj(bears), 1);
    assert_eq!((dmg(&t, bears), t.life(P1)), (0, 18));
    // Used up.
    hit(&mut t, giant, pl(P0), 1);
    assert_eq!((t.life(P0), t.life(P1)), (19, 18));
}

#[test]
fn captains_maneuver_needs_its_first_target_to_be_dealt_the_damage() {
    cr!("614.9", "400.7");
    ruling!(
        "Captain's Maneuver",
        "If the first target is no longer on the battlefield, then the damage will not be dealt at all, so it can’t be redirected by this ability."
    );
    supported("Captain's Maneuver");
    // The first target, P0's Bears, leaves and returns: a new object, whose damage isn't
    // redirected (and nothing is dealt to P1).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Mountain", 2);
    let m = t.hand(P0, "Captain's Maneuver");
    t.cast(P0, m).x(2).target(bears).target(P1).go();
    t.resolve();
    move_to(&mut t, bears, Zone::Hand(P0));
    let back = t.battlefield(P0, "Grizzly Bears");
    hit(&mut t, giant, obj(back), 1);
    assert_eq!(dmg(&t, back), 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn captains_maneuver_does_nothing_if_its_second_target_cant_be_dealt_the_damage() {
    cr!("614.9", "205.1a");
    ruling!(
        "Captain's Maneuver",
        "If the second target is no longer on the battlefield, or if the second target is no longer a creature, player, or planeswalker when the damage would be redirected, the damage is dealt to the first target as if this spell was not cast."
    );
    for gone in ["left", "not a creature"] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let wurm = t.battlefield(P0, "Craw Wurm");
        let second = t.battlefield(P1, "Calming Licid");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.lands(P0, "Plains", 2);
        t.lands(P0, "Mountain", 2);
        let m = t.hand(P0, "Captain's Maneuver");
        t.cast(P0, m).x(2).target(wurm).target(second).go();
        t.resolve();
        if gone == "left" {
            destroy(&mut t, second);
        } else {
            // The Licid becomes an Aura attached to the Bears.
            t.lands(P1, "Plains", 1);
            t.activate(P1, second, 0, &[obj(bears)]).unwrap();
            t.resolve();
            assert!(!t.obj_now(second).chars.is_creature());
        }
        hit(&mut t, giant, obj(wurm), 2);
        assert_eq!(dmg(&t, wurm), 2, "{gone}");
        if gone != "left" {
            assert!(t.on_battlefield(second));
            assert_eq!(dmg(&t, second), 0);
        }
    }
}

#[test]
fn martyrdom_redirects_only_to_its_creature_while_it_is_one() {
    cr!("614.9", "609.7b");
    ruling!(
        "Martyrdom",
        "If the target creature is not still on the battlefield when the damage is to be redirected, the damage stays where it is."
    );
    ruling!(
        "Martyrdom",
        "If the target stops being a creature, you can’t redirect damage to it until/unless it becomes a creature again."
    );
    supported("Martyrdom");
    // The creature that gained the ability (Craw Wurm) leaves: the damage to P0 stays.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P0, "Craw Wurm");
    cast_new(&mut t, P0, "Martyrdom", &[obj(wurm)]);
    t.resolve_all();
    t.activate(P0, wurm, 0, &[pl(P0)]).unwrap();
    t.resolve_all();
    destroy(&mut t, wurm);
    hit(&mut t, giant, pl(P0), 1);
    assert_eq!(t.life(P0), 19);
    // Martyrdom on Calming Licid, which then becomes an Aura: the damage stays with P0;
    // once it's a creature again, the shield (not used up) redirects the next 1 damage.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let licid = t.battlefield(P0, "Calming Licid");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Martyrdom", &[obj(licid)]);
    t.resolve_all();
    // The granted ability is the Licid's second activated ability.
    t.activate(P0, licid, 1, &[pl(P0)]).unwrap();
    t.resolve_all();
    t.lands(P0, "Plains", 1);
    t.activate(P0, licid, 0, &[obj(bears)]).unwrap();
    t.resolve();
    assert!(!t.obj_now(licid).chars.is_creature());
    hit(&mut t, giant, pl(P0), 1);
    assert_eq!(t.life(P0), 19, "not redirected to a noncreature");
    end_licid_effect(&mut t, P0);
    assert!(t.obj_now(licid).chars.is_creature());
    hit(&mut t, giant, pl(P0), 1);
    assert_eq!(t.life(P0), 19, "redirected to the creature again");
    assert_eq!(dmg(&t, licid), 1);
}

/// `p` pays to end a Licid's effect (a special action, CR 116.2c), with {W} in its pool.
pub fn end_licid_effect(t: &mut TestGame, p: PlayerId) {
    use mtg_engine::decision::{Action, SpecialAction};
    t.g.players[p.idx()]
        .mana_pool
        .add_type(mtg_engine::mana::ManaType::W, 1);
    t.g.turn.priority = Some(p);
    let sa =
        t.g.legal_actions(p)
            .into_iter()
            .find_map(|a| match a {
                Action::Special(s @ SpecialAction::Offer { .. }) => Some(s),
                _ => None,
            })
            .expect("the action to end the Licid's effect");
    t.g.take_action(p, Action::Special(sa));
    t.g.recompute();
    t.settle();
}

#[test]
fn kor_dirge_redirected_damage_is_still_dealt_by_the_original_source() {
    cr!("614.9", "609.7a", "702.2b", "601.2c");
    ruling!(
        "Kor Dirge",
        "The damage dealt to the second target creature is dealt by the original source of damage, not by Kor Dirge."
    );
    ruling!(
        "Kor Dirge",
        "You choose the two target creatures as you cast Kor Dirge, but you choose the source as it resolves."
    );
    supported("Kor Dirge");
    let mut t = TestGame::new(2);
    let rats = t.battlefield(P1, "Typhoid Rats");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let from = t.asked().len();
    let spell = cast_new(&mut t, P0, "Kor Dirge", &[obj(bears), obj(wurm)]);
    assert_eq!(targets_of(&t, spell), vec![obj(bears), obj(wurm)]);
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. })));
    t.answer_choose(P0, &[obj(rats)]);
    t.resolve_all();
    hit(&mut t, rats, obj(bears), 1);
    // Dealt by the Rats (deathtouch): the Wurm dies.
    assert!(t.on_battlefield(bears));
    assert!(!t.on_battlefield(wurm));
    assert_eq!(damage_by(&t, rats), vec![(obj(wurm), 1, false)]);
    assert!(damage_by(&t, spell).is_empty());
}

#[test]
fn avacyns_abilities_dont_undo_damage_and_dont_recheck_targets() {
    cr!("615.1", "608.2b", "105.1");
    ruling!(
        "Avacyn, Guardian Angel",
        "Neither activated ability has any effect on damage that’s already been dealt."
    );
    ruling!(
        "Avacyn, Guardian Angel",
        "Whether the target permanent or player is still a legal target is no longer checked after the appropriate ability resolves. For example, if a creature targeted by the first ability gains protection from white after the ability resolves but before it prevents damage, the effect will still prevent damage."
    );
    ruling!(
        "Avacyn, Guardian Angel",
        "The possible color choices are white, blue, black, red, and green. You can’t choose artifact, colorless, or any other word."
    );
    let mut t = TestGame::new(2);
    let avacyn = t.battlefield(P0, "Avacyn, Guardian Angel");
    let wurm = t.battlefield(P0, "Craw Wurm");
    let goblin = t.battlefield(P1, "Raging Goblin");
    let golem = t.battlefield(P1, "Stone Golem");
    damage(&mut t, goblin, 1, wurm);
    // Red.
    t.lands(P0, "Plains", 2);
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.activate(P0, avacyn, 0, &[obj(wurm)]).unwrap();
    t.resolve_all();
    let options: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(
        options,
        vec![vec![
            "white".to_string(),
            "blue".into(),
            "black".into(),
            "red".into(),
            "green".into()
        ]]
    );
    assert_eq!(dmg(&t, wurm), 1, "already dealt");
    // The Wurm gains protection from white (Gods Willing): still shielded.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    cast_new(&mut t, P0, "Gods Willing", &[obj(wurm)]);
    t.resolve_all();
    assert!(t
        .obj_now(wurm)
        .chars
        .has_keyword(mtg_engine::keywords::KeywordKind::Protection));
    damage(&mut t, goblin, 2, wurm);
    assert_eq!(dmg(&t, wurm), 1, "red damage prevented");
    damage(&mut t, golem, 2, wurm);
    assert_eq!(dmg(&t, wurm), 3, "colorless damage isn't");
    // The second ability (red, at P0, already dealt 3): P0's life stays at 17, and the
    // next red damage is prevented.
    let mut t = TestGame::new(2);
    let avacyn = t.battlefield(P0, "Avacyn, Guardian Angel");
    let goblin = t.battlefield(P1, "Raging Goblin");
    t.g.deal_damage(goblin, pl(P0), 3, false);
    t.settle();
    t.lands(P0, "Plains", 2);
    t.lands(P0, "Wastes", 5);
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.activate(P0, avacyn, 1, &[pl(P0)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 17, "already dealt");
    t.g.deal_damage(goblin, pl(P0), 3, false);
    t.settle();
    assert_eq!(t.life(P0), 17);
}

#[test]
fn safe_passage_covers_creatures_entering_later_but_not_damage_already_dealt() {
    cr!("615.1", "611.2c");
    ruling!(
        "Safe Passage",
        "Safe Passage has no effect on damage that’s already been dealt."
    );
    ruling!(
        "Safe Passage",
        "Safe Passage will prevent damage dealt to creatures that weren’t on the battlefield at the time it resolved."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P0, "Craw Wurm");
    hit(&mut t, giant, obj(wurm), 2);
    hit(&mut t, giant, pl(P0), 2);
    cast_new(&mut t, P0, "Safe Passage", &[]);
    t.resolve_all();
    assert_eq!((dmg(&t, wurm), t.life(P0)), (2, 18), "already dealt");
    let bears = t.battlefield(P0, "Grizzly Bears");
    hit(&mut t, giant, obj(bears), 3);
    assert!(t.on_battlefield(bears));
    assert_eq!(dmg(&t, bears), 0);
}

#[test]
fn saving_grace_keeps_the_source_and_combat_damage_and_not_earlier_damage() {
    cr!("614.9", "702.2b", "510.2");
    ruling!(
        "Saving Grace",
        "Saving Grace's ability has no effect on damage already dealt earlier in the turn."
    );
    ruling!(
        "Saving Grace",
        "Saving Grace's redirection effect doesn't change the source of the damage or whether the damage is combat damage."
    );
    supported("Saving Grace");
    let mut t = TestGame::new(2);
    let hawk = t.battlefield(P1, "Vampire Nighthawk");
    let giant = t.battlefield(P1, "Hill Giant");
    let wurm = t.battlefield(P0, "Craw Wurm");
    hit(&mut t, giant, pl(P0), 3);
    cast_new(&mut t, P0, "Saving Grace", &[obj(wurm)]);
    t.resolve_all();
    assert_eq!(t.obj_now(wurm).attached_to, None);
    assert!(t
        .g
        .permanents()
        .any(|o| o.chars.name == "Saving Grace" && o.attached_to == Some(obj(wurm))));
    assert_eq!(t.life(P0), 17, "already dealt");
    // Nighthawk's combat damage to P0 is dealt to the Wurm, still by the Nighthawk
    // (deathtouch, lifelink) and still combat damage.
    t.g.deal_damage(hawk, pl(P0), 2, true);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 22, "lifelink");
    assert!(!t.on_battlefield(wurm), "deathtouch");
    assert_eq!(damage_by(&t, hawk), vec![(obj(wurm), 2, true)]);
}

/// Wraps `p`'s agent so that each division of a shield among simultaneous damage
/// (`Decision::Divide`) is answered by recipient, from `amounts` (0 for the others).
fn divide_by_recipient(t: &mut TestGame, p: PlayerId, amounts: Vec<(Entity, i64)>) {
    use mtg_engine::decision::Agent;
    struct ByRecipient {
        inner: Box<dyn Agent>,
        amounts: Vec<(Entity, i64)>,
    }
    impl Agent for ByRecipient {
        fn decide(&mut self, g: &mtg_engine::game::Game, p: PlayerId, d: &Decision) -> Answer {
            match d {
                Decision::Divide { recipients, .. } => {
                    // Keep the decision in the scripted agent's log.
                    let _ = self.inner.decide(g, p, d);
                    Answer::Numbers(
                        recipients
                            .iter()
                            .map(|r| {
                                self.amounts
                                    .iter()
                                    .find(|(e, _)| e == r)
                                    .map_or(0, |(_, n)| *n)
                            })
                            .collect(),
                    )
                }
                _ => self.inner.decide(g, p, d),
            }
        }
    }
    let mut agents = t.g.agents.0.lock().unwrap();
    let inner = std::mem::replace(
        &mut agents[p.idx()],
        Box::new(mtg_engine::decision::PassiveAgent),
    );
    agents[p.idx()] = Box::new(ByRecipient { inner, amounts });
}

/// The divisions asked of `p` since decision `from`: (total, recipients).
fn divisions_since(t: &TestGame, p: PlayerId, from: usize) -> Vec<(u32, Vec<Entity>)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(q, d)| match d {
            Decision::Divide {
                total, recipients, ..
            } if *q == p => Some((*total, recipients.clone())),
            _ => None,
        })
        .collect()
}

#[test]
fn harms_way_redirects_just_2_of_simultaneous_damage_as_you_choose() {
    cr!("614.9", "615.7", "609.7a");
    interp!("shield-divided-among-recipients");
    ruling!(
        "Harm's Way",
        "If the chosen source would simultaneously deal damage to multiple permanents you control (like Pyroclasm could) or to you and at least one permanent you control (like Earthquake could), Harm’s Way will redirect just 2 of that damage. It won’t redirect the next 2 damage that would be dealt to each recipient. You choose which 2 damage is redirected. If you like, you can choose to redirect 1 damage that would be dealt by the chosen source to each of two different recipients."
    );
    // P1's Pyroclasm (2 damage to each creature), the source chosen for P0's Harm's Way
    // (redirecting to P1): P0's Grizzly Bears and Hill Giant.
    for (bears_share, giant_share) in [(2, 0), (0, 2), (1, 1)] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let giant = t.battlefield(P0, "Hill Giant");
        t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
        let clasm = cast_new(&mut t, P1, "Pyroclasm", &[]);
        cast_new(&mut t, P0, "Harm's Way", &[pl(P1)]);
        t.answer_choose(P0, &[obj(clasm)]);
        t.resolve();
        divide_by_recipient(
            &mut t,
            P0,
            vec![(obj(bears), bears_share), (obj(giant), giant_share)],
        );
        let from = t.asked().len();
        t.resolve_all();
        let divs = divisions_since(&t, P0, from);
        assert_eq!(divs.len(), 1, "{divs:?}");
        assert_eq!(divs[0].0, 2);
        assert_eq!(t.life(P1), 18, "just 2 redirected in all");
        assert_eq!(t.on_battlefield(bears), bears_share > 0, "{bears_share}");
        assert_eq!(dmg(&t, giant), 2 - giant_share as u32);
        if bears_share == 1 {
            assert_eq!(dmg(&t, bears), 1);
        }
    }
    // Earthquake (X = 2): P0 and P0's Hill Giant; 2 of the damage to P0 redirected.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Wastes", 2);
    let quake = t.hand(P1, "Earthquake");
    let quake = t.cast(P1, quake).x(2).go();
    cast_new(&mut t, P0, "Harm's Way", &[pl(P1)]);
    t.answer_choose(P0, &[obj(quake)]);
    t.resolve();
    divide_by_recipient(&mut t, P0, vec![(pl(P0), 2)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(dmg(&t, giant), 2);
    assert_eq!(t.life(P1), 16, "2 from Earthquake, 2 redirected");
}

#[test]
fn refraction_trap_you_choose_which_simultaneous_damage_it_prevents_as_it_is_dealt() {
    cr!("615.7", "615.5", "609.7a");
    interp!("shield-divided-among-recipients");
    ruling!(
        "Refraction Trap",
        "If the chosen source would simultaneously deal damage to multiple permanents you control, or would simultaneously deal damage to you and at least one permanent you control, you choose which of that damage to prevent. For example, if the chosen source is Earthquake, you might choose to prevent the next 2 damage it would deal to you and the next 1 damage it would deal to a creature you control. You don’t decide until the point at which the source would deal its damage."
    );
    // P1's Earthquake (X = 2) at P0 and P0's Grizzly Bears: the Trap prevents 2 to P0 and
    // 1 to the Bears, and deals 3 to P1.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, mtg_engine::turn::Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    t.lands(P1, "Wastes", 2);
    let quake = t.hand(P1, "Earthquake");
    let quake = t.cast(P1, quake).x(2).go();
    divide_by_recipient(&mut t, P0, vec![(pl(P0), 2), (obj(bears), 1)]);
    let from = t.asked().len();
    trap(&mut t, quake, pl(P1));
    assert!(divisions_since(&t, P0, from).is_empty(), "not decided yet");
    t.resolve_all();
    let divs = divisions_since(&t, P0, from);
    assert_eq!(divs.len(), 1, "decided as the Earthquake deals its damage");
    assert_eq!(divs[0].0, 3);
    assert_eq!(t.life(P0), 20);
    assert!(t.on_battlefield(bears));
    assert_eq!(dmg(&t, bears), 1);
    assert_eq!(t.life(P1), 15, "2 from Earthquake, 3 from the Trap");
}

/// P0 activates Nova Pentacle: P0 picks `chooser`, who picks `target`. Returns the
/// decisions asked meanwhile.
fn pentacle(
    t: &mut TestGame,
    pent: ObjectId,
    chooser: PlayerId,
    target: ObjectId,
) -> Vec<(PlayerId, Decision)> {
    t.g.objects[pent.0 as usize].tapped = false;
    t.lands(P0, "Wastes", 3);
    let from = t.asked().len();
    // (With a single opponent, there's no choice of opponent to make.)
    if t.g.players.len() > 2 {
        t.answer_choose(P0, &[pl(chooser)]);
    }
    t.answer_targets(chooser, &[obj(target)]);
    t.activate(P0, pent, 0, &[]).expect("Nova Pentacle");
    t.asked()[from..].to_vec()
}

#[test]
fn nova_pentacle_an_opponent_picks_the_target_and_you_the_source() {
    cr!("115.1", "601.2c", "609.7a", "614.9");
    ruling!(
        "Nova Pentacle",
        "As you activate the ability, you choose an opponent, then that opponent chooses a creature for the ability to target. It doesn’t have to be a creature that opponent controls. If there are no creatures that can be targeted with the ability, it can’t be activated."
    );
    ruling!(
        "Nova Pentacle",
        "In multi-player games you can choose a different opposing player each time it is used."
    );
    ruling!(
        "Nova Pentacle",
        "As the ability resolves, you choose a damage source."
    );
    supported("Nova Pentacle");
    // No creature: it can't be activated.
    let mut t = TestGame::new(2);
    let pent = t.battlefield(P0, "Nova Pentacle");
    t.lands(P0, "Wastes", 3);
    assert!(!crate::r_s02_common::can_activate(&mut t, P0, pent));
    // Three players: P0 picks P2, who picks P0's own Bears.
    let mut t = TestGame::new(3);
    let pent = t.battlefield(P0, "Nova Pentacle");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P2, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    let asked = pentacle(&mut t, pent, P2, bears);
    assert!(asked.iter().any(|(p, d)| *p == P0
        && matches!(d, Decision::ChooseEntities { candidates, .. }
            if candidates.contains(&pl(P1)) && candidates.contains(&pl(P2)))));
    assert!(asked
        .iter()
        .any(|(p, d)| *p == P2 && matches!(d, Decision::ChooseTargets { .. })));
    let ability = *t.g.stack.last().unwrap();
    assert_eq!(targets_of(&t, ability), vec![obj(bears)]);
    // No source chosen yet: P0 chooses Hill Giant as it resolves.
    assert_eq!(asked.len(), 2, "{asked:?}");
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve_all();
    hit(&mut t, giant, pl(P0), 2);
    assert_eq!(t.life(P0), 20);
    assert!(!t.on_battlefield(bears));
    // The next activation: P0 picks P1, who picks P2's Craw Wurm.
    let asked = pentacle(&mut t, pent, P1, wurm);
    assert!(asked
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, Decision::ChooseTargets { .. })));
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve_all();
    hit(&mut t, giant, pl(P0), 3);
    assert_eq!((t.life(P0), dmg(&t, wurm)), (20, 3));
}

#[test]
fn nova_pentacle_doesnt_recheck_its_target_but_needs_a_creature_there() {
    cr!("614.9", "608.2b");
    ruling!(
        "Nova Pentacle",
        "The next time damage from the chosen source would be dealt to you this turn, that damage is dealt to the targeted creature instead. Whether it’s still a legal target is not checked at this time. However, if it’s not able to be dealt damage because it’s neither a creature nor a planeswalker, or it’s no longer on the battlefield, then the damage is not redirected and is dealt to you as normal."
    );
    // The target gains shroud: still redirected.
    let mut t = TestGame::new(2);
    let pent = t.battlefield(P0, "Nova Pentacle");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    pentacle(&mut t, pent, P1, wurm);
    t.answer_choose(P0, &[obj(giant)]);
    t.resolve_all();
    attach_new(&mut t, P1, "Lightning Greaves", wurm);
    hit(&mut t, giant, pl(P0), 3);
    assert_eq!((t.life(P0), dmg(&t, wurm)), (20, 3));
    // The target left the battlefield, or stopped being a creature (Calming Licid became
    // an Aura): dealt to P0.
    for gone in ["left", "not a creature"] {
        let mut t = TestGame::new(2);
        let pent = t.battlefield(P0, "Nova Pentacle");
        let licid = t.battlefield(P1, "Calming Licid");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let giant = t.battlefield(P1, "Hill Giant");
        pentacle(&mut t, pent, P1, licid);
        t.answer_choose(P0, &[obj(giant)]);
        t.resolve_all();
        if gone == "left" {
            destroy(&mut t, licid);
        } else {
            t.lands(P1, "Plains", 1);
            t.activate(P1, licid, 0, &[obj(bears)]).unwrap();
            t.resolve();
            assert!(!t.obj_now(licid).chars.is_creature());
        }
        hit(&mut t, giant, pl(P0), 3);
        assert_eq!(t.life(P0), 17, "{gone}");
    }
}
