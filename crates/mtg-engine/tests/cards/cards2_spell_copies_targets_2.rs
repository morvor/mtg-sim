//! Spell copies and target changes II (CR 115.7, 707.10): a restriction on the new target
//! ("The new target must be a player."), "change that spell's target to another creature"
//! for a spell with one target, "change the target to enchanted creature if able" for a
//! spell with a single target, each other player copying a spell they don't control, new
//! targets for the spell that triggered an ability, and "a new target for the copy".

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        c.unsupported_text()
    );
}

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn targets_of(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    t.g.obj(id)
        .stack
        .as_ref()
        .map(|s| {
            s.chosen
                .iter()
                .flat_map(|m| m.targets.iter().flatten().copied())
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn rebound_changes_a_player_target_to_another_player() {
    cr!("115.7", "115.7a");
    supported("Rebound");
    let mut t = TestGame::new(3);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    // P0 responds: the new target must be a player (not P1's creature).
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Island", 2);
    let r = t.hand(P0, "Rebound");
    t.g.turn.priority = Some(P0);
    t.cast(P0, r).target(spell).go();
    t.answer_targets(P0, &[Entity::Player(P2)]);
    t.resolve();
    assert_eq!(targets_of(&t, spell), vec![Entity::Player(P2)]);
    t.resolve_all();
    assert_eq!(t.life(P2), 17);
    assert_eq!(t.life(P0), 20);
    assert!(t.on_battlefield(bears));
}

#[test]
fn meddle_moves_a_single_creature_target_to_another_creature() {
    cr!("115.7", "115.9a");
    supported("Meddle");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(mine).go();
    t.lands(P0, "Island", 2);
    let m = t.hand(P0, "Meddle");
    t.g.turn.priority = Some(P0);
    t.cast(P0, m).target(spell).go();
    t.answer_targets(P0, &[obj(theirs)]);
    t.resolve();
    assert_eq!(targets_of(&t, spell), vec![obj(theirs)]);
    t.resolve_all();
    assert!(!t.on_battlefield(theirs));
    assert!(t.on_battlefield(mine));
    // A spell targeting a player isn't changed.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.lands(P0, "Island", 2);
    let m = t.hand(P0, "Meddle");
    t.g.turn.priority = Some(P0);
    t.cast(P0, m).target(spell).go();
    t.resolve();
    assert_eq!(targets_of(&t, spell), vec![Entity::Player(P0)]);
}

#[test]
fn quicksilver_dragon_redirects_a_spell_targeting_only_it() {
    cr!("115.7", "115.9c");
    supported("Quicksilver Dragon");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Quicksilver Dragon");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Swamp", 3);
    let kill = t.hand(P1, "Murder");
    let spell = t.cast(P1, kill).target(dragon).go();
    t.lands(P0, "Island", 1);
    t.g.turn.priority = Some(P0);
    t.activate(P0, dragon, 0, &[obj(spell)]).unwrap();
    t.answer_targets(P0, &[obj(theirs)]);
    t.resolve();
    assert_eq!(targets_of(&t, spell), vec![obj(theirs)]);
    t.resolve_all();
    assert!(t.on_battlefield(dragon));
    assert!(!t.on_battlefield(theirs));
}

#[test]
fn captured_by_the_consulate_redirects_single_target_spells_to_the_enchanted_creature() {
    cr!("115.7", "115.9a", "603.4");
    supported("Captured by the Consulate");
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    let other = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 4);
    let c = t.hand(P0, "Captured by the Consulate");
    t.cast(P0, c).target(theirs).go();
    t.resolve_all();
    let mine = t.battlefield(P0, "Craw Wurm");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(mine).go();
    t.settle();
    t.resolve();
    assert_eq!(targets_of(&t, spell), vec![obj(theirs)]);
    t.resolve_all();
    assert!(!t.on_battlefield(theirs));
    assert!(t.on_battlefield(other) && t.on_battlefield(mine));
}

#[test]
fn curse_of_echoes_lets_each_other_player_copy_the_spell() {
    cr!("707.10", "707.10c");
    supported("Curse of Echoes");
    let mut t = TestGame::new(3);
    t.lands(P0, "Island", 5);
    let curse = t.hand(P0, "Curse of Echoes");
    t.cast(P0, curse).target(Entity::Player(P1)).go();
    t.resolve_all();
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Island", 1);
    let opt = t.hand(P1, "Opt");
    // P0 copies; P2 declines. P1 (the caster) isn't asked.
    t.answer_yes(P0, true);
    t.answer_yes(P2, false);
    let before = (t.hand_size(P0), t.hand_size(P2));
    t.cast(P1, opt).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), before.0 + 1);
    assert_eq!(t.hand_size(P2), before.1);
}

#[test]
fn speedball_lets_you_choose_new_targets_for_the_spell_targeting_him() {
    cr!("115.7d", "603.2");
    supported("Speedball, New Warrior");
    let mut t = TestGame::new(2);
    let sb = t.battlefield(P0, "Speedball, New Warrior");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(theirs)]);
    t.cast(P1, bolt).target(sb).go();
    t.resolve_all();
    assert!(!t.on_battlefield(theirs));
    assert!(t.on_battlefield(sb));
    assert_eq!(t.obj_now(sb).damage, 0);
}

#[test]
fn ancestral_communion_copies_itself_while_you_control_your_commander() {
    cr!("707.10", "903.3");
    supported("Ancestral Communion");
    let mut t = TestGame::new(2);
    let cmd = t.battlefield(P0, "Grizzly Bears");
    t.g.objects[cmd.0 as usize].is_commander = true;
    let a = t.graveyard(P0, "Hill Giant");
    let b = t.graveyard(P0, "Craw Wurm");
    t.lands(P0, "Forest", 2);
    let s = t.hand(P0, "Ancestral Communion");
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(b)]);
    t.cast(P0, s).target(a).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Hill Giant"));
    assert!(t.in_hand(P0, "Craw Wurm"));
}

#[test]
fn radiate_copies_a_spell_for_each_other_permanent_or_player_it_could_target() {
    cr!("707.10d", "707.10", "115.9c");
    supported("Radiate");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(bears).go();
    t.lands(P0, "Mountain", 5);
    let r = t.hand(P0, "Radiate");
    t.g.turn.priority = Some(P0);
    t.cast(P0, r).target(spell).go();
    t.resolve();
    // Four copies (Hill Giant, Craw Wurm, each player), controlled by Radiate's caster.
    assert_eq!(t.stack_len(), 5);
    for c in t.g.stack.clone() {
        if c != spell {
            assert_eq!(t.g.obj(c).controller, P0);
        }
    }
    t.resolve_all();
    assert!(!t.on_battlefield(bears) && !t.on_battlefield(giant));
    assert_eq!(t.obj_now(wurm).damage, 3);
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn radiant_performer_copies_a_single_target_ability_for_each_other_target() {
    cr!("707.10d", "707.10", "603.4");
    supported("Radiant Performer");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Swamp", 3);
    let kill = t.hand(P1, "Murder");
    let spell = t.cast(P1, kill).target(a).go();
    // Flash: P0 casts Radiant Performer from hand in response, targeting Murder.
    t.lands(P0, "Mountain", 5);
    let rp = t.hand(P0, "Radiant Performer");
    t.g.turn.priority = Some(P0);
    t.answer_targets(P0, &[Entity::Object(spell)]);
    t.cast(P0, rp).go();
    t.resolve_all();
    // Murder's copies destroyed every other creature (Hill Giant, and Radiant Performer).
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert!(t.named_on_battlefield("Radiant Performer").is_empty());
}
