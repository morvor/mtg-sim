//! The worked examples of CR 707 (copying objects), played out as tests. Examples already
//! set up by other tests are cited there instead:
//! - the third and fourth 707.3 examples (a face-down Grinning Demon becomes a copy of
//!   Branchsnap Lorian / Wandering Ones): `r708_face_down.rs`;
//! - the second 707.5 example (Wall of Omens), 707.6 (Adaptive Automaton) and the second
//!   707.8a example (a token copy of a Clone): `r707_copying_objects.rs`.

use crate::r703_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::eval::Ctx;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;
use smol_str::SmolStr;

/// `p`'s Clone enters as a copy of `what`.
fn clone_of(t: &mut TestGame, p: PlayerId, what: ObjectId) -> ObjectId {
    t.answer_choose(p, &[Entity::Object(what)]);
    let c = t.enter(p, "Clone");
    t.g.current(c)
}

/// `p` enters `name` (a "you may have this enter as a copy" card) copying `what`.
fn enter_copying(t: &mut TestGame, p: PlayerId, name: &str, what: ObjectId) -> ObjectId {
    t.answer_choose(p, &[Entity::Object(what)]);
    let c = t.enter(p, name);
    t.g.current(c)
}

/// `obj` becomes a copy of `of` (a copy effect from a resolving ability).
fn become_copy(t: &mut TestGame, obj: ObjectId, of: ObjectId) {
    let mut ctx = Ctx::new(None, P0);
    ctx.targets = vec![vec![Entity::Object(obj)], vec![Entity::Object(of)]];
    t.g.exec(
        &Effect::BecomeCopy {
            what: Sel::Target(0),
            of: Sel::Target(1),
            duration: Duration::Permanent,
        },
        &mut ctx,
    );
    t.g.recompute();
    t.g.flush_events();
}

/// Puts `name` from `p`'s hand onto the battlefield face down.
fn put_face_down(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let card = t.hand(p, name);
    run_effect(
        t,
        p,
        None,
        Effect::Move {
            what: Sel::Target(0),
            to: Destination {
                face_down: true,
                ..Destination::battlefield()
            },
        },
        &[Entity::Object(card)],
    );
    let id = t.g.current(card);
    assert!(t.obj(id).face_down && t.on_battlefield(id));
    id
}

fn turn_up_action(t: &mut TestGame, p: PlayerId, obj: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.legal_actions(p)
        .contains(&Action::Special(SpecialAction::TurnFaceUp { obj }))
}

fn is_a(t: &TestGame, id: ObjectId, subtype: &str) -> bool {
    let id = t.g.current(id);
    t.g.matches(
        id,
        &Filter::Subtype(SmolStr::new(subtype)),
        &Ctx::new(None, P0),
    )
}

fn triggered_count(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Triggered(_)))
        .count()
}

fn activated_count(t: &TestGame, id: ObjectId) -> usize {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .count()
}

/// The newest object on the stack.
fn top_of_stack(t: &TestGame) -> ObjectId {
    *t.g.stack.last().expect("empty stack")
}

// --- 707.2 ----------------------------------------------------------------------------

#[test]
fn a_clone_of_an_animated_chimeric_staff_is_just_an_artifact() {
    cr!("707.2", "707.2a");
    cr_example!("707.2", "Chimeric Staff is an artifact");
    supported("Chimeric Staff");
    supported("Clone");
    let mut t = TestGame::new(2);
    let staff = t.battlefield(P0, "Chimeric Staff");
    t.lands(P0, "Wastes", 5);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.activate(P0, staff, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(staff).is_creature());
    assert_eq!(t.pt(staff), (5, 5));
    let c = clone_of(&mut t, P0, staff);
    let o = t.obj_now(c);
    assert_eq!(o.chars.name.as_str(), "Chimeric Staff");
    assert!(o.chars.is(CardType::Artifact));
    assert!(!o.is_creature());
    assert!(!o.chars.has_subtype("Construct"));
    // It has the Staff's ability, and becomes a creature if it's activated.
    assert_eq!(activated_count(&t, c), 1);
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.activate(P0, c, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(c).is_creature());
    assert!(t.obj_now(c).chars.has_subtype("Construct"));
    assert_eq!(t.pt(c), (2, 2));
}

#[test]
fn a_clone_of_a_face_down_grinning_demon_is_a_face_up_blank_2_2() {
    cr!("707.2", "708.2");
    cr_example!("707.2", "Clone enters the battlefield as a copy of a face-down Grinning Demon");
    supported("Grinning Demon");
    let mut t = TestGame::new(2);
    let demon = put_face_down(&mut t, P1, "Grinning Demon");
    let c = clone_of(&mut t, P0, demon);
    let o = t.obj_now(c);
    assert!(!o.face_down);
    assert!(o.chars.name.is_empty());
    assert_eq!(o.chars.colors, ColorSet::NONE);
    assert!(o.chars.subtypes.is_empty());
    assert_eq!(o.chars.supertypes, SupertypeSet::NONE);
    assert!(o.chars.abilities.is_empty());
    assert!(o.chars.mana_cost.is_none());
    assert!(o.is_creature());
    assert_eq!(t.pt(c), (2, 2));
    // Its controller can't pay {2}{B}{B} to turn it face up.
    t.lands(P0, "Swamp", 4);
    assert!(!turn_up_action(&mut t, P0, c));
}

// --- 707.3 ----------------------------------------------------------------------------

#[test]
fn a_clone_of_a_vesuvan_doppelganger_copying_runeclaw_bear_is_a_blue_bear_with_its_ability() {
    cr!("707.3", "707.9b");
    cr_example!("707.3", "Vesuvan Doppelganger reads");
    supported("Vesuvan Doppelganger");
    supported("Runeclaw Bear");
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P1, "Runeclaw Bear");
    t.lands(P0, "Island", 5);
    let dopp = t.hand(P0, "Vesuvan Doppelganger");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bear)]);
    t.cast(P0, dopp).go();
    t.resolve_all();
    let dopp = t.g.current(dopp);
    assert_eq!(t.obj(dopp).chars.name.as_str(), "Runeclaw Bear");
    assert_eq!(t.obj(dopp).chars.colors, ColorSet::single(Color::Blue));
    let c = clone_of(&mut t, P0, dopp);
    let o = t.obj_now(c);
    assert_eq!(o.chars.name.as_str(), "Runeclaw Bear");
    assert_eq!(o.chars.colors, ColorSet::single(Color::Blue));
    assert!(o.chars.has_subtype("Bear"));
    assert_eq!(t.pt(c), (2, 2));
    // It has the Doppelganger's upkeep-triggered ability.
    assert_eq!(triggered_count(&t, c), 1);
    assert!(o
        .chars
        .abilities
        .iter()
        .any(|a| a.text.to_lowercase().contains("upkeep")));
}

#[test]
fn tomoya_becoming_a_copy_of_nezumi_shortfang_is_stabwhisker() {
    cr!("707.3", "710.4");
    cr_example!("707.3", "Tomoya the Revealer (a flipped flip card)");
    let mut t = TestGame::new(2);
    let jushi = t.battlefield(P0, "Jushi Apprentice // Tomoya the Revealer");
    mtg_engine::flip::flip(&mut t.g, jushi);
    t.g.recompute();
    assert_eq!(t.obj(jushi).chars.name.as_str(), "Tomoya the Revealer");
    let nezumi = t.battlefield(P1, "Nezumi Shortfang // Stabwhisker the Odious");
    assert_eq!(t.obj(nezumi).chars.name.as_str(), "Nezumi Shortfang");
    become_copy(&mut t, jushi, nezumi);
    let o = t.obj_now(jushi);
    assert_eq!(o.chars.name.as_str(), "Stabwhisker the Odious");
    assert!(o.chars.has_supertype(Supertype::Legendary));
    assert_eq!(t.pt(jushi), (3, 3));
}

// --- 707.4 ----------------------------------------------------------------------------

#[test]
fn unstable_shapeshifter_keeps_giant_growth_as_it_becomes_a_copy() {
    cr!("707.4", "611.2c");
    cr_example!("707.4", "Unstable Shapeshifter reads");
    supported("Unstable Shapeshifter");
    let mut t = TestGame::new(2);
    let shifter = t.battlefield(P0, "Unstable Shapeshifter");
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(shifter).go();
    t.resolve_all();
    assert_eq!(t.pt(shifter), (3, 4));
    t.enter(P1, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.obj_now(shifter).chars.name.as_str(), "Hill Giant");
    assert!(t.on_battlefield(shifter));
    assert_eq!(t.pt(shifter), (6, 6));
}

// --- 707.5 ----------------------------------------------------------------------------

#[test]
fn a_clone_of_skyshroud_behemoth_enters_tapped_with_two_fade_counters() {
    cr!("707.5", "702.32a");
    cr_example!("707.5", "Skyshroud Behemoth reads");
    supported("Skyshroud Behemoth");
    let mut t = TestGame::new(2);
    let behemoth = t.battlefield(P1, "Skyshroud Behemoth");
    let c = clone_of(&mut t, P0, behemoth);
    assert_eq!(t.obj(c).chars.name.as_str(), "Skyshroud Behemoth");
    assert!(t.obj(c).tapped);
    assert_eq!(t.counters(c, "fade"), 2);
}

// --- 707.8a ---------------------------------------------------------------------------

#[test]
fn a_token_copy_of_a_transformed_afflicted_deserter_has_both_faces() {
    cr!("707.8a", "712.8");
    cr_example!("707.8a", "Afflicted Deserter is the front face");
    supported("Afflicted Deserter // Werewolf Ransacker");
    let mut t = TestGame::new(2);
    let deserter = t.battlefield(P1, "Afflicted Deserter // Werewolf Ransacker");
    assert!(mtg_engine::dfc::transform(&mut t.g, deserter));
    t.g.recompute();
    assert_eq!(t.obj(deserter).chars.name.as_str(), "Werewolf Ransacker");
    let before = t.g.objects.len();
    run_effect(
        &mut t,
        P0,
        None,
        Effect::CreateTokenCopy {
            of: Sel::Target(0),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
            mods: vec![],
        },
        &[Entity::Object(deserter)],
    );
    let tok = (before..t.g.objects.len())
        .map(|i| ObjectId(i as u32))
        .find(|id| t.g.is_live(*id) && t.obj(*id).zone == Zone::Battlefield)
        .expect("token created");
    // It enters with the same face up, and can transform.
    assert_eq!(t.obj(tok).chars.name.as_str(), "Werewolf Ransacker");
    assert_eq!(t.pt(tok), (5, 4));
    assert!(mtg_engine::dfc::transform(&mut t.g, tok));
    t.g.recompute();
    assert_eq!(t.obj(tok).chars.name.as_str(), "Afflicted Deserter");
    assert_eq!(t.pt(tok), (3, 2));
}

// --- 707.9 ----------------------------------------------------------------------------

#[test]
fn a_clone_of_a_shapeshifter_copying_quirion_elves_has_the_shapeshifters_ability() {
    cr!("707.9a", "707.3");
    cr_example!("707.9a", "Quirion Elves enters the battlefield and an Unstable Shapeshifter");
    supported("Quirion Elves");
    let mut t = TestGame::new(2);
    let shifter = t.battlefield(P0, "Unstable Shapeshifter");
    t.enter(P1, "Quirion Elves");
    t.resolve_all();
    assert_eq!(t.obj_now(shifter).chars.name.as_str(), "Quirion Elves");
    // The Elves' mana abilities and the Shapeshifter's own ability.
    assert_eq!(triggered_count(&t, shifter), 1);
    let c = clone_of(&mut t, P0, shifter);
    // (The Clone entering also triggers the Shapeshifter, which copies the Clone: still
    // the same values.)
    t.resolve_all();
    assert_eq!(t.obj_now(c).chars.name.as_str(), "Quirion Elves");
    assert_eq!(triggered_count(&t, c), 1);
    // The Clone's copy of that ability works: another creature entering makes it a copy
    // of that creature.
    t.enter(P1, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.obj_now(c).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.obj_now(shifter).chars.name.as_str(), "Hill Giant");
    assert_eq!(triggered_count(&t, c), 1);
}

#[test]
fn copy_artifact_copying_juggernaut_is_also_an_enchantment() {
    cr!("707.9b", "707.9a");
    cr_example!("707.9b", "Copy Artifact is an enchantment");
    supported("Copy Artifact");
    supported("Juggernaut");
    let mut t = TestGame::new(2);
    let jug = t.battlefield(P1, "Juggernaut");
    let ca = enter_copying(&mut t, P0, "Copy Artifact", jug);
    let o = t.obj_now(ca);
    assert_eq!(o.chars.name.as_str(), "Juggernaut");
    for ty in [CardType::Artifact, CardType::Creature, CardType::Enchantment] {
        assert!(o.chars.is(ty), "{ty:?}");
        assert!(o.copiable.card_types.contains(ty), "copiable {ty:?}");
    }
    assert_eq!(t.pt(ca), (5, 3));
    // The modification is part of its copiable values: a Clone of it is an enchantment.
    let c = clone_of(&mut t, P0, ca);
    assert!(t.obj_now(c).chars.is(CardType::Enchantment));
    assert_eq!(t.obj_now(c).chars.name.as_str(), "Juggernaut");
}

#[test]
fn quicksilver_gargantuan_copying_tarmogoyf_is_7_7() {
    cr!("707.9d", "604.3");
    cr_example!("707.9d", "Quicksilver Gargantuan is a creature that reads");
    supported("Quicksilver Gargantuan");
    supported("Tarmogoyf");
    let mut t = TestGame::new(2);
    let goyf = t.battlefield(P1, "Tarmogoyf");
    t.graveyard(P1, "Grizzly Bears");
    t.graveyard(P1, "Forest");
    t.g.recompute();
    assert_eq!(t.pt(goyf), (2, 3));
    let q = enter_copying(&mut t, P0, "Quicksilver Gargantuan", goyf);
    assert_eq!(t.obj_now(q).chars.name.as_str(), "Tarmogoyf");
    assert_eq!(t.pt(q), (7, 7));
    // It doesn't have the characteristic-defining ability.
    t.graveyard(P1, "Lightning Bolt");
    t.g.recompute();
    assert_eq!(t.pt(goyf), (3, 4));
    assert_eq!(t.pt(q), (7, 7));
}

#[test]
fn glasspool_mimic_copying_a_changeling_has_changeling_and_every_creature_type() {
    cr!("707.9d", "702.73a");
    cr_example!("707.9d", "Glasspool Mimic is a creature that reads");
    supported("Glasspool Mimic // Glasspool Shore");
    supported("Woodland Changeling");
    let mut t = TestGame::new(2);
    let wc = t.battlefield(P0, "Woodland Changeling");
    let m = enter_copying(&mut t, P0, "Glasspool Mimic // Glasspool Shore", wc);
    let o = t.obj_now(m);
    assert_eq!(o.chars.name.as_str(), "Woodland Changeling");
    assert!(o.has_keyword(KeywordKind::Changeling));
    for ty in ["Shapeshifter", "Rogue", "Goblin", "Elf", "Dragon"] {
        assert!(is_a(&t, m, ty), "{ty}");
    }
}

#[test]
fn altered_ego_copying_a_clone_that_copies_gets_no_counters() {
    cr!("707.9e", "707.9b");
    cr_example!("707.9e", "Altered Ego reads");
    supported("Altered Ego");
    let mut t = TestGame::new(2);
    // A Clone for which no creature was chosen (kept alive by Glorious Anthem).
    t.battlefield(P0, "Glorious Anthem");
    let clone = t.battlefield(P0, "Clone");
    assert_eq!(t.pt(clone), (1, 1));
    let giant = t.battlefield(P1, "Hill Giant");
    // Altered Ego (X = 2) copies the Clone; the replacement effect it gains then copies
    // the Hill Giant: no +1/+1 counters.
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Island", 3);
    let ego = t.hand(P0, "Altered Ego");
    t.answer_choose(P0, &[Entity::Object(clone)]);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, ego).x(2).go();
    t.resolve_all();
    let ego = t.g.current(ego);
    assert_eq!(t.obj(ego).chars.name.as_str(), "Hill Giant", "{}", t.dump_log());
    assert_eq!(t.counters(ego, "+1/+1"), 0);
    assert_eq!(t.pt(ego), (4, 4));
    // Copying the Hill Giant directly, it would have two.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Island", 3);
    let ego = t.hand(P0, "Altered Ego");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.cast(P0, ego).x(2).go();
    t.resolve_all();
    let ego = t.g.current(ego);
    assert_eq!(t.obj(ego).chars.name.as_str(), "Hill Giant");
    assert_eq!(t.counters(ego, "+1/+1"), 2);
}

#[test]
fn moritte_copying_an_animated_land_gets_no_counters_or_changeling() {
    cr!("707.9f");
    cr_example!("707.9f", "Moritte of the Frost says, in part");
    supported("Moritte of the Frost");
    supported("Animate Land");
    let mut t = TestGame::new(2);
    let land = t.battlefield(P0, "Forest");
    t.lands(P0, "Forest", 2);
    let animate = t.hand(P0, "Animate Land");
    t.cast(P0, animate).target(land).go();
    t.resolve_all();
    assert!(t.obj_now(land).is_creature());
    let m = enter_copying(&mut t, P0, "Moritte of the Frost", land);
    let o = t.obj_now(m);
    assert_eq!(o.chars.name.as_str(), "Forest");
    assert!(!o.is_creature());
    assert!(o.chars.has_supertype(Supertype::Legendary));
    assert!(o.chars.has_supertype(Supertype::Snow));
    assert_eq!(t.counters(m, "+1/+1"), 0);
    assert!(!o.has_keyword(KeywordKind::Changeling));
    // It doesn't have changeling even if it becomes a creature later in the turn.
    let animate = t.hand(P0, "Animate Land");
    t.cast(P0, animate).target(m).go();
    t.resolve_all();
    assert!(t.obj_now(m).is_creature());
    assert!(!t.obj_now(m).has_keyword(KeywordKind::Changeling));
    assert_eq!(t.counters(m, "+1/+1"), 0);
}

// --- 707.10 ---------------------------------------------------------------------------

#[test]
fn fork_copies_emerald_charm_red_with_the_same_mode() {
    cr!("707.10", "707.10c", "700.2g");
    cr_example!("707.10", "A player casts Fork, targeting an Emerald Charm");
    supported("Fork");
    supported("Emerald Charm");
    let mut t = TestGame::new(2);
    let drake = t.battlefield(P1, "Wind Drake");
    let angel = t.battlefield(P1, "Serra Angel");
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Mountain", 2);
    let charm = t.hand(P0, "Emerald Charm");
    // "Target creature loses flying until end of turn."
    let charm = t.cast(P0, charm).modes(&[2]).target(drake).go();
    let fork = t.hand(P0, "Fork");
    t.cast(P0, fork).target(charm).go();
    // The copy may get a new target.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(angel)]);
    t.resolve();
    let copy = top_of_stack(&t);
    assert_ne!(copy, charm);
    let o = t.obj(copy);
    assert_eq!(o.chars.name.as_str(), "Emerald Charm");
    assert_eq!(o.chars.colors, ColorSet::single(Color::Red));
    let modes = |id: ObjectId| -> Vec<Option<usize>> {
        let si = t.obj(id).stack.as_ref().unwrap();
        si.chosen.iter().map(|c| c.mode).collect()
    };
    assert_eq!(modes(copy), vec![Some(2)]);
    assert_eq!(modes(copy), modes(charm));
    t.resolve_all();
    assert!(!t.obj_now(drake).has_keyword(KeywordKind::Flying));
    assert!(!t.obj_now(angel).has_keyword(KeywordKind::Flying));
}

#[test]
fn a_copy_of_fling_uses_the_power_of_the_creature_sacrificed_for_the_original() {
    cr!("707.10", "608.2h");
    cr_example!("707.10", "Fling is an instant that reads");
    supported("Fling");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 4);
    let fling = t.hand(P0, "Fling");
    t.answer_choose(P0, &[Entity::Object(giant)]);
    let fling = t.cast(P0, fling).target(Entity::Player(P1)).go();
    assert!(t.in_graveyard(P0, "Hill Giant"));
    let fork = t.hand(P0, "Fork");
    t.cast(P0, fork).target(fling).go();
    t.answer_yes(P0, false);
    t.resolve();
    // The copy resolves first: 3 damage, the Hill Giant's power.
    t.resolve();
    assert_eq!(t.life(P1), 17);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
}

#[test]
fn a_copy_of_dawnglow_infusion_gains_no_life() {
    cr!("707.10", "106.1");
    cr_example!("707.10", "Dawnglow Infusion is a sorcery that reads");
    supported("Dawnglow Infusion");
    let mut t = TestGame::new(2);
    // X = 2, paid with {G} and {W} spent: 4 life.
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Plains", 2);
    let dawn = t.hand(P0, "Dawnglow Infusion");
    let dawn = t.cast(P0, dawn).x(2).go();
    t.lands(P0, "Mountain", 2);
    let fork = t.hand(P0, "Fork");
    t.cast(P0, fork).target(dawn).go();
    t.resolve();
    // The copy: no mana was spent to cast it.
    t.resolve();
    assert_eq!(t.life(P0), 20);
    t.resolve_all();
    assert_eq!(t.life(P0), 24);
}

#[test]
fn frontline_heroism_with_anointed_procession_copy_targets_one_soldier() {
    cr!("707.10e", "707.10c");
    cr_example!("707.10e", "Frontline Heroism is an enchantment");
    supported("Frontline Heroism");
    supported("Anointed Procession");
    supported("Moment of Triumph");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Frontline Heroism");
    t.battlefield(P0, "Anointed Procession");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Plains", 1);
    let mot = t.hand(P0, "Moment of Triumph");
    t.cast(P0, mot).target(bears).go();
    t.resolve_all();
    let soldiers: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|id| t.obj(*id).is_token() && t.obj(*id).chars.has_subtype("Soldier"))
        .collect();
    assert_eq!(soldiers.len(), 2, "{}", t.dump_log());
    let mut pts: Vec<(i32, i32)> = soldiers.iter().map(|s| t.pt(*s)).collect();
    pts.sort();
    // The copy targeted only one of them.
    assert_eq!(pts, vec![(1, 1), (3, 3)]);
    assert_eq!(t.pt(bears), (4, 4));
    assert_eq!(t.life(P0), 24);
}

// --- 707.11 ---------------------------------------------------------------------------

#[test]
fn shapeshifters_copied_olivia_ability_still_puts_the_counter_on_it() {
    cr!("707.11", "113.7a");
    cr_example!("707.11", "An Unstable Shapeshifter copies an Olivia Voldaren");
    supported("Olivia Voldaren");
    let mut t = TestGame::new(2);
    let shifter = t.battlefield(P0, "Unstable Shapeshifter");
    t.enter(P1, "Olivia Voldaren");
    t.resolve_all();
    assert_eq!(t.obj_now(shifter).chars.name.as_str(), "Olivia Voldaren");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 2);
    t.activate(P0, shifter, 0, &[Entity::Object(bears)]).unwrap();
    // In response, another creature enters: the Shapeshifter becomes a Hill Giant.
    t.enter(P1, "Hill Giant");
    t.settle();
    t.resolve();
    assert_eq!(t.obj_now(shifter).chars.name.as_str(), "Hill Giant");
    t.resolve_all();
    assert_eq!(t.obj_now(bears).damage, 1);
    assert!(t.obj_now(bears).chars.has_subtype("Vampire"));
    assert_eq!(t.counters(shifter, "+1/+1"), 1);
    assert_eq!(t.pt(shifter), (4, 4));
}
