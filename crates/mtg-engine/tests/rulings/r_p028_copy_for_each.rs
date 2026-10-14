//! Rulings batch P028 — "copy that spell for each other [creature] that spell could
//! target. Each copy targets a different one of those creatures." (Zada, Hedron Grinder;
//! Mirrorwing Dragon; Precursor Golem; Ink-Treader Nephilim): the abilities trigger on a
//! spell that targets only that one object, even more than once (CR 115.9c); each copy
//! gets one new target, used for every target of the copy (CR 707.10d); the copies keep
//! the original's X and modes (CR 707.10).

use crate::r_p030_common::resolve_until_copies;
use crate::r_s01_common::supported;
use crate::r_s07_common::chosen_modes;
use crate::r_s25_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// P0 casts Agony Warp ("Target creature gets -3/-0 until end of turn. Target creature
/// gets -0/-3 until end of turn.") choosing `target` for both of its targets.
fn agony_warp_twice_at(t: &mut TestGame, p: PlayerId, target: ObjectId) -> ObjectId {
    lands_for_cost(t, p, "Agony Warp");
    let card = t.hand(p, "Agony Warp");
    t.cast(p, card).target(target).target(target).go()
}

/// Whether any "choose new targets" question was asked since `from`.
fn new_targets_offered(t: &TestGame, from: usize) -> bool {
    t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { prompt, .. } if prompt.contains("target")))
}

#[test]
fn zada_s_copies_of_a_spell_targeting_only_zada_twice_each_target_one_other_creature() {
    cr!("115.9c", "707.10d", "707.10");
    ruling!(
        "Zada, Hedron Grinder",
        "If you cast an instant or sorcery spell that has multiple targets, but it's targeting only Zada with all of them, Zada's ability will trigger. The copies will similarly each be targeting only one of your other creatures. You can't change any of the copy's targets to other creatures."
    );
    supported("Zada, Hedron Grinder");
    supported("Agony Warp");
    let mut t = TestGame::new(2);
    let zada = t.battlefield(P0, "Zada, Hedron Grinder");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    agony_warp_twice_at(&mut t, P0, zada);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 2);
    let mut targeted: Vec<Vec<Entity>> = copies.iter().map(|c| targets_of(&t, *c)).collect();
    targeted.sort();
    let mut want = vec![vec![obj(a), obj(a)], vec![obj(b), obj(b)]];
    want.sort();
    assert_eq!(targeted, want);
    assert!(!new_targets_offered(&t, from));
    t.resolve_all();
    // Each got -3/-3; the opponent's Bears weren't a "creature you control".
    assert!(!t.on_battlefield(zada) && !t.on_battlefield(a) && !t.on_battlefield(b));
    assert!(t.on_battlefield(theirs));
}

#[test]
fn zada_s_copies_keep_x_and_modes() {
    cr!("707.10", "107.3", "700.2");
    ruling!(
        "Zada, Hedron Grinder",
        "If the spell that's copied has an X whose value was determined as it was cast (like Rolling Thunder does), the copies have the same value of X."
    );
    ruling!(
        "Zada, Hedron Grinder",
        "If the spell that's copied is modal (that is, it says \"Choose one —\" or the like), the copies will have the same mode. You can't choose a different one."
    );
    // Blaze, X = 2, at Zada: the copy deals 2 to the Bears.
    let mut t = TestGame::new(2);
    let zada = t.battlefield(P0, "Zada, Hedron Grinder");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Blaze");
    t.cast(P0, card).x(2).target(zada).go();
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(x_of(&t, copies[0]), Some(2));
    assert_eq!(targets_of(&t, copies[0]), vec![obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.obj_now(zada).damage, 2);
    // Izzet Charm's second mode ("2 damage to target creature") at Zada.
    let mut t = TestGame::new(2);
    let zada = t.battlefield(P0, "Zada, Hedron Grinder");
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Izzet Charm");
    let card = t.hand(P0, "Izzet Charm");
    t.cast(P0, card).modes(&[1]).target(zada).go();
    let copies = resolve_until_copies(&mut t);
    assert_eq!(chosen_modes(&t, copies[0]), vec![1]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn mirrorwing_dragon_s_copies_of_a_spell_targeting_it_twice_target_one_other_creature_each() {
    cr!("115.9c", "707.10d", "707.10");
    ruling!(
        "Mirrorwing Dragon",
        "If a player casts an instant or sorcery spell that has multiple targets and Mirrorwing Dragon is chosen as the target in each instance, Mirrorwing Dragon's ability will trigger. Each of the copies will similarly be targeting only one of the player's other creatures."
    );
    supported("Mirrorwing Dragon");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Mirrorwing Dragon");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    agony_warp_twice_at(&mut t, P0, dragon);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 2);
    let mut targeted: Vec<Vec<Entity>> = copies.iter().map(|c| targets_of(&t, *c)).collect();
    targeted.sort();
    let mut want = vec![vec![obj(a), obj(a)], vec![obj(b), obj(b)]];
    want.sort();
    assert_eq!(targeted, want);
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    // Mirrorwing Dragon is a 4/5: -3/-3 leaves it a 1/2.
    assert_eq!(t.pt(dragon), (1, 2));
}

#[test]
fn mirrorwing_dragon_s_copies_go_to_the_caster_for_the_caster_s_creatures() {
    cr!("707.10d", "707.10");
    // "Whenever a player casts an instant or sorcery spell that targets only this creature,
    // that player copies that spell for each other creature they control that the spell
    // could target."
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let dragon = t.battlefield(P0, "Mirrorwing Dragon");
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P1, "Izzet Charm");
    let card = t.hand(P1, "Izzet Charm");
    t.cast(P1, card).modes(&[1]).target(dragon).go();
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 1);
    assert_eq!(t.obj(copies[0]).controller, P1);
    assert_eq!(targets_of(&t, copies[0]), vec![obj(theirs)]);
    t.resolve_all();
    assert!(t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn mirrorwing_dragon_s_copies_keep_x_and_modes() {
    cr!("707.10", "107.3", "700.2");
    ruling!(
        "Mirrorwing Dragon",
        "If the spell that's copied has an X whose value was determined as it was cast (like Burn from Within does), the copies have the same value of X."
    );
    ruling!(
        "Mirrorwing Dragon",
        "If the spell that's copied is modal (that is, it says \"Choose one —\" or the like), the copies will have the same mode. A different mode cannot be chosen."
    );
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Mirrorwing Dragon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    let card = t.hand(P0, "Blaze");
    t.cast(P0, card).x(2).target(dragon).go();
    let copies = resolve_until_copies(&mut t);
    assert_eq!(x_of(&t, copies[0]), Some(2));
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Mirrorwing Dragon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Izzet Charm");
    let card = t.hand(P0, "Izzet Charm");
    t.cast(P0, card).modes(&[1]).target(dragon).go();
    let copies = resolve_until_copies(&mut t);
    assert_eq!(chosen_modes(&t, copies[0]), vec![1]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

/// Precursor Golem enters (cast), making two 3/3 Golem tokens: three Golems in all.
fn three_golems(t: &mut TestGame) -> Vec<ObjectId> {
    cast_new(t, P0, "Precursor Golem", &[]);
    t.resolve_all();
    let golems = crate::r_s01_common::with_subtype(t, P0, "Golem");
    assert_eq!(golems.len(), 3);
    golems
}

#[test]
fn precursor_golem_triggers_on_a_spell_targeting_the_same_golem_twice() {
    cr!("115.9c", "707.10d");
    ruling!(
        "Precursor Golem",
        "If an instant or sorcery spell has multiple targets, but it's targeting the same Golem with all of them (such as Agony Warp targeting the same Golem twice), Precursor Golem's last ability will trigger."
    );
    supported("Precursor Golem");
    let mut t = TestGame::new(2);
    let golems = three_golems(&mut t);
    // Non-Golem creatures get no copy (only "each other Golem that spell could target").
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    agony_warp_twice_at(&mut t, P0, golems[1]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 2);
    let mut firsts = vec![];
    for c in &copies {
        let ts = targets_of(&t, *c);
        assert_eq!(ts.len(), 2);
        assert_eq!(ts[0], ts[1]);
        assert_ne!(ts[0], obj(golems[1]));
        firsts.push(ts[0]);
    }
    firsts.sort();
    let mut want = vec![obj(golems[0]), obj(golems[2])];
    want.sort();
    assert_eq!(firsts, want);
    t.resolve_all();
    // Every Golem got -3/-3; the Bears and the Giant weren't touched.
    assert!(crate::r_s01_common::with_subtype(&t, P0, "Golem").is_empty());
    assert!(t.on_battlefield(bears) && t.on_battlefield(giant));
}

#[test]
fn precursor_golem_s_copies_keep_x_and_modes() {
    cr!("707.10", "107.3", "700.2");
    ruling!(
        "Precursor Golem",
        "If the spell that's copied has an X whose value was determined as it was cast (like Fireball does), the copies have the same value of X."
    );
    ruling!(
        "Precursor Golem",
        "If the spell that's copied is modal (that is, it says \"Choose one —\" or the like), the copies will have the same mode. Their controller can't choose a different one."
    );
    let mut t = TestGame::new(2);
    let golems = three_golems(&mut t);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 3);
    let card = t.hand(P0, "Blaze");
    t.cast(P0, card).x(3).target(golems[0]).go();
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 2);
    assert!(copies.iter().all(|c| x_of(&t, *c) == Some(3)));
    t.resolve_all();
    assert!(crate::r_s01_common::with_subtype(&t, P0, "Golem").is_empty());
    // Izzet Charm, 2 damage to target creature: each Golem is dealt 2.
    let mut t = TestGame::new(2);
    let golems = three_golems(&mut t);
    lands_for_cost(&mut t, P0, "Izzet Charm");
    let card = t.hand(P0, "Izzet Charm");
    t.cast(P0, card).modes(&[1]).target(golems[0]).go();
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 2);
    assert!(copies.iter().all(|c| chosen_modes(&t, *c) == vec![1]));
    t.resolve_all();
    assert!(golems.iter().all(|g| t.obj_now(*g).damage == 2));
}

#[test]
fn ink_treader_nephilim_s_copies_use_the_original_spell_s_perspective_and_then_fizzle() {
    cr!("707.10d", "608.2b", "115.9c", "603.4");
    ruling!(
        "Ink-Treader Nephilim",
        "Even though you control the copies, the ability cares about who cast the original spell when determining which other creatures that spell could target. If the original spell has a targeting restriction that refers to \"an opponent,\" the copies will see that from the perspective of the original spell's controller."
    );
    supported("Ink-Treader Nephilim");
    supported("Deadly Embrace");
    // P1 casts Deadly Embrace ("Destroy target creature an opponent controls. Then draw a
    // card for each creature that died this turn.") at P0's Ink-Treader Nephilim.
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let nephilim = t.battlefield(P0, "Ink-Treader Nephilim");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let embrace = cast_new(&mut t, P1, "Deadly Embrace", &[obj(nephilim)]);
    let copies = resolve_until_copies(&mut t);
    // A copy for each of P0's other creatures (an opponent's, as P1 sees it), controlled
    // by P0; none for P1's Hill Giant.
    assert_eq!(copies.len(), 2);
    let mut targeted: Vec<Entity> = copies.iter().flat_map(|c| targets_of(&t, *c)).collect();
    targeted.sort();
    let mut want = vec![obj(a), obj(b)];
    want.sort();
    assert_eq!(targeted, want);
    assert!(copies.iter().all(|c| t.obj(*c).controller == P0));
    let hand = t.hand_size(P0);
    t.resolve_all();
    // For P0, the Bears aren't creatures an opponent controls: both copies are countered
    // on resolution. The original destroys the Nephilim.
    assert!(t.on_battlefield(a) && t.on_battlefield(b) && t.on_battlefield(giant));
    assert!(!t.on_battlefield(nephilim));
    assert_eq!(t.hand_size(P0), hand);
    assert!(!t.g.is_live(embrace) || !t.g.stack.contains(&embrace));
}

#[test]
fn ink_treader_nephilim_doesnt_copy_a_spell_with_another_target() {
    cr!("603.4", "115.9c");
    // "if that spell targets only this creature": Agony Warp at the Nephilim and at the
    // Bears doesn't target only it.
    let mut t = TestGame::new(2);
    let nephilim = t.battlefield(P0, "Ink-Treader Nephilim");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    lands_for_cost(&mut t, P0, "Agony Warp");
    let card = t.hand(P0, "Agony Warp");
    t.cast(P0, card).target(nephilim).target(bears).go();
    assert!(resolve_until_copies(&mut t).is_empty());
}
