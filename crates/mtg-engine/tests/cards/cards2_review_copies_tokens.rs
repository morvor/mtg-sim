//! Review additions for cards2-copies-tokens-counters: a restriction on a changed target
//! (Rebound), copies of a spell that another player controls and that aren't cast
//! (Curse of Echoes), abilities given to the tokens just created (Rapacious One), and the
//! "first time you would create one or more tokens" copy replacements (Esix, Moonlit
//! Meditation, Mirrormind Crown).

use mtg_engine::keywords::KeywordKind;
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

fn yes_no_asks(t: &TestGame, p: PlayerId) -> usize {
    t.asked()
        .iter()
        .filter(|(q, d)| *q == p && matches!(d, mtg_engine::decision::Decision::YesNo { .. }))
        .count()
}

fn raise_the_alarm(t: &mut TestGame, p: PlayerId) {
    t.lands(p, "Plains", 2);
    let c = t.hand(p, "Raise the Alarm");
    t.cast(p, c).go();
    t.resolve_all();
}

#[test]
fn rebound_new_target_must_be_a_player_not_a_creature() {
    cr!("115.7", "115.7a");
    supported("Rebound");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.lands(P0, "Island", 2);
    let r = t.hand(P0, "Rebound");
    t.g.turn.priority = Some(P0);
    t.cast(P0, r).target(spell).go();
    // P0 tries to move it to P1's creature: not allowed, the only legal new target is P1.
    t.answer_targets(P0, &[obj(bears)]);
    t.resolve();
    assert_eq!(targets_of(&t, spell), vec![Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 20);
    // A spell that targets a creature can't be targeted by Rebound.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let spell = t.cast(P1, bolt).target(bears).go();
    t.lands(P0, "Island", 2);
    let r = t.hand(P0, "Rebound");
    t.g.turn.priority = Some(P0);
    assert!(t.cast(P0, r).target(spell).try_go().is_err());
}

#[test]
fn curse_of_echoes_copy_is_controlled_by_the_other_player_and_not_cast() {
    cr!("707.10", "707.10c", "702.108a");
    supported("Curse of Echoes");
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    let curse = t.hand(P0, "Curse of Echoes");
    t.cast(P0, curse).target(Entity::Player(P1)).go();
    t.resolve_all();
    // Prowess triggers on casting a noncreature spell: P0's copy isn't cast.
    let swift = t.battlefield(P0, "Monastery Swiftspear");
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let spell = t.cast(P1, bolt).target(Entity::Player(P0)).go();
    t.settle();
    // Resolve the Curse's trigger: the copy goes on the stack above the Bolt.
    t.resolve();
    let copies: Vec<ObjectId> = t.g.stack.iter().copied().filter(|c| *c != spell).collect();
    assert_eq!(copies.len(), 1);
    assert_eq!(t.g.obj(copies[0]).controller, P0);
    assert_eq!(targets_of(&t, copies[0]), vec![Entity::Player(P1)]);
    assert_eq!(targets_of(&t, spell), vec![Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.pt(swift), (1, 2));
}

#[test]
fn rapacious_one_spawn_tokens_have_the_mana_ability() {
    cr!("111.1", "113.3b");
    supported("Rapacious One");
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Rapacious One");
    t.attack(&[(r, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    let spawn = t.named_on_battlefield("Eldrazi Spawn Token");
    assert_eq!(spawn.len(), 5);
    assert_eq!(t.pt(spawn[0]), (0, 1));
    assert_eq!(t.obj_now(spawn[0]).chars.colors, mtg_engine::types::ColorSet::default());
    t.activate(P0, spawn[0], 0, &[]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(spawn[0]));
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn esix_declined_is_not_offered_again_that_turn() {
    cr!("614.1a");
    ruling!(
        "Esix, Fractal Bloom",
        "If you choose not to apply the replacement effect, you will not get the choice to apply it again until your next turn."
    );
    supported("Esix, Fractal Bloom");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Esix, Fractal Bloom");
    t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, false);
    raise_the_alarm(&mut t, P0);
    assert_eq!(t.named_on_battlefield("Soldier Token").len(), 2);
    let asks = yes_no_asks(&t, P0);
    assert_eq!(asks, 1);
    t.answer_yes(P0, true);
    raise_the_alarm(&mut t, P0);
    assert_eq!(yes_no_asks(&t, P0), asks);
    assert_eq!(t.named_on_battlefield("Soldier Token").len(), 4);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    t.clear_answers();
}

#[test]
fn esix_can_replace_a_noncreature_token_with_a_copy() {
    cr!("614.1a", "707.2");
    ruling!(
        "Esix, Fractal Bloom",
        "This effect can apply to any token, not just creature tokens."
    );
    supported("Esix, Fractal Bloom");
    let mut t = TestGame::new(2);
    let esix = t.battlefield(P0, "Esix, Fractal Bloom");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_yes(P0, true);
    // Esix itself isn't offered ("a creature other than Esix"): choosing it falls back.
    t.answer_choose(P0, &[obj(esix)]);
    t.lands(P0, "Mountain", 1);
    let s = t.hand(P0, "Strike It Rich");
    t.cast(P0, s).go();
    t.resolve_all();
    assert!(t.named_on_battlefield("Treasure Token").is_empty());
    assert_eq!(t.named_on_battlefield("Esix, Fractal Bloom").len(), 1);
    let giants = t.named_on_battlefield("Hill Giant");
    assert_eq!(giants.len(), 2);
    assert!(giants.iter().any(|g| *g != giant && t.obj_now(*g).is_token()));
}

#[test]
fn esix_does_not_apply_after_tokens_were_created_earlier_that_turn() {
    cr!("614.1a");
    ruling!(
        "Esix, Fractal Bloom",
        "If you create one or more tokens during your turn, and then Esix comes under your control, the replacement effect won't apply"
    );
    supported("Esix, Fractal Bloom");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    raise_the_alarm(&mut t, P0);
    t.battlefield(P0, "Esix, Fractal Bloom");
    t.answer_yes(P0, true);
    raise_the_alarm(&mut t, P0);
    assert_eq!(t.named_on_battlefield("Soldier Token").len(), 4);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    t.clear_answers();
}

#[test]
fn moonlit_meditation_copies_of_a_legendary_permanent_are_legendary() {
    cr!("614.1a", "707.2", "704.5j");
    ruling!(
        "Moonlit Meditation",
        "If the enchanted permanent is legendary, the copies will also be legendary."
    );
    supported("Moonlit Meditation");
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.lands(P0, "Island", 3);
    t.answer_targets(P0, &[obj(isamaru)]);
    let m = t.hand(P0, "Moonlit Meditation");
    t.cast(P0, m).go();
    t.resolve_all();
    t.answer_yes(P0, true);
    raise_the_alarm(&mut t, P0);
    // Two legendary copies were created; the legend rule leaves one Isamaru.
    assert!(t.named_on_battlefield("Soldier Token").is_empty());
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda").len(), 1);
    assert!(t
        .g
        .events
        .iter()
        .chain(t.turn_events.iter())
        .filter(|e| matches!(e, mtg_engine::events::Event::TokenCreated { .. }))
        .count()
        >= 2);
}

#[test]
fn mirrormind_crown_copies_only_the_copiable_values() {
    cr!("614.1a", "707.2");
    ruling!(
        "Mirrormind Crown",
        "They don't copy whether that creature is tapped or untapped, whether it has any counters on it"
    );
    supported("Mirrormind Crown");
    let mut t = TestGame::new(2);
    let crown = t.battlefield(P0, "Mirrormind Crown");
    let giant = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, crown, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    t.g.add_counters(obj(giant), "+1/+1", 2, None);
    t.g.objects[giant.0 as usize].tapped = true;
    t.answer_yes(P0, true);
    raise_the_alarm(&mut t, P0);
    let copies: Vec<ObjectId> = t
        .named_on_battlefield("Hill Giant")
        .into_iter()
        .filter(|g| *g != giant)
        .collect();
    assert_eq!(copies.len(), 2);
    for c in copies {
        assert!(t.obj_now(c).is_token());
        assert!(!t.obj_now(c).tapped);
        assert_eq!(t.counters(c, "+1/+1"), 0);
        assert_eq!(t.pt(c), (3, 3));
        assert!(!t.obj_now(c).has_keyword(KeywordKind::Haste));
    }
}

