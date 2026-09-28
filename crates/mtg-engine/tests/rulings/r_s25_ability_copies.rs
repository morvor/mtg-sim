//! Rulings batch S25 — copying activated and triggered abilities (CR 707.10): "Copy target
//! activated or triggered ability you control" (Lithoform Engine, Adric, Peter Parker's
//! Camera, Strionic Resonator, Vantress Visions). The copy has the same source (CR
//! 707.10b), targets unless new ones are chosen (CR 707.10c), X and cost choices; choices
//! made on resolution are made separately; it resolves first. Copying a permanent spell
//! makes a token that isn't "created" (CR 707.10f).

use crate::r_s01_common::supported;
use crate::r_s02_common::{destroy, target_candidates};
use crate::r_s06_common::activate_containing;
use crate::r_s18_common::ADVENTURE;
use crate::r_s25_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

/// `copier`'s ability whose text contains `needle` targets the topmost ability on the
/// stack.
fn copy_top_ability(t: &mut TestGame, copier: ObjectId, needle: &str) {
    let top = *t.g.stack.last().expect("nothing to copy");
    t.answer_targets(P0, &[Entity::Object(top)]);
    activate_containing(t, P0, copier, needle).unwrap();
}

#[test]
fn a_copy_of_a_triggered_ability_has_the_same_source() {
    cr!("707.10", "707.10b");
    ruling!(
        "Strionic Resonator",
        "The source of the copy is the same as the source of the original ability."
    );
    supported("Strionic Resonator");
    // "{2}, {T}: Copy target triggered ability you control." Ajani's Pridemate: "Whenever
    // you gain life, put a +1/+1 counter on this creature."
    let mut t = TestGame::new(2);
    let resonator = t.battlefield(P0, "Strionic Resonator");
    let pridemate = t.battlefield(P0, "Ajani's Pridemate");
    cast_new(&mut t, P0, "Angel's Mercy", &[]);
    t.resolve();
    assert_eq!(abilities_from(&t, pridemate).len(), 1);
    t.lands(P0, "Wastes", 2);
    copy_top_ability(&mut t, resonator, "Copy target");
    t.resolve();
    let both = abilities_from(&t, pridemate);
    assert_eq!(both.len(), 2);
    t.resolve_all();
    assert_eq!(t.counters(pridemate, counters::PLUS1), 2);
    assert_eq!(t.counters(resonator, counters::PLUS1), 0);
}

#[test]
fn a_copy_of_an_activated_ability_may_get_new_targets() {
    cr!("707.10", "707.10c", "115.7d");
    ruling!(
        "Adric, Mathematical Genius",
        "The copy will have the same targets as the ability it's copying unless you choose new ones. You may change any number of the targets, including all of them or none of them. If, for one of the targets, you can't choose a new legal target, then it remains unchanged (even if the current target is illegal)."
    );
    supported("Adric, Mathematical Genius");
    // Adric: "{2}{U}, {T}: Copy target activated or triggered ability you control. You may
    // choose new targets for the copy." Prodigal Pyromancer: "{T}: This creature deals 1
    // damage to any target."
    let mut t = TestGame::new(2);
    let adric = t.battlefield(P0, "Adric, Mathematical Genius");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let a = t.battlefield(P1, "Llanowar Elves");
    let b = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(a)]);
    let ping = activate_containing(&mut t, P0, pyro, "damage").unwrap().unwrap();
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    copy_top_ability(&mut t, adric, "Copy target");
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(b))]);
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, ping);
    assert_eq!(targets_of(&t, copy), vec![Entity::Object(b)]);
    assert_eq!(targets_of(&t, ping), vec![Entity::Object(a)]);
    t.resolve_all();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
}

#[test]
fn a_copy_of_an_ability_resolves_before_the_original() {
    cr!("707.10", "405.2", "405.5");
    ruling!(
        "Lithoform Engine",
        "The copy will resolve before the original spell or ability does."
    );
    supported("Lithoform Engine");
    // "{2}, {T}: Copy target activated or triggered ability you control. You may choose
    // new targets for the copy."
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(elves)]);
    activate_containing(&mut t, P0, pyro, "damage").unwrap();
    t.lands(P0, "Wastes", 2);
    copy_top_ability(&mut t, engine, "activated or triggered");
    change_copy_targets(&mut t, P0, &[Some(Entity::Player(P1))]);
    t.resolve();
    // The copy (at P1) resolves first.
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert!(t.on_battlefield(elves));
    t.resolve();
    assert!(!t.on_battlefield(elves));
}

#[test]
fn a_copy_of_an_ability_with_x_has_the_same_x() {
    cr!("707.10", "107.3");
    ruling!(
        "Lithoform Engine",
        "If the spell or ability that's copied has an X whose value was determined as it was cast or activated, the copy will have the same value of X."
    );
    supported("Helix Pinnacle");
    // Helix Pinnacle: "{X}: Put X tower counters on this enchantment."
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    let pinnacle = t.battlefield(P0, "Helix Pinnacle");
    t.lands(P0, "Wastes", 3);
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    let ability = activate_containing(&mut t, P0, pinnacle, "tower").unwrap().unwrap();
    assert_eq!(x_of(&t, ability), Some(3));
    t.lands(P0, "Wastes", 2);
    copy_top_ability(&mut t, engine, "activated or triggered");
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_eq!(x_of(&t, copy), Some(3));
    t.resolve_all();
    assert_eq!(t.counters(pinnacle, "tower"), 6);
}

#[test]
fn a_camera_copy_of_an_ability_with_x_has_the_same_x() {
    cr!("707.10", "107.3");
    ruling!(
        "Peter Parker's Camera",
        "If the ability that's copied has an X whose value was determined as it was activated, the copy will have the same value of X."
    );
    supported("Peter Parker's Camera");
    // "This artifact enters with three film counters on it. {2}, {T}, Remove a film counter
    // from this artifact: Copy target activated or triggered ability you control."
    let mut t = TestGame::new(2);
    let camera = t.enter(P0, "Peter Parker's Camera");
    assert_eq!(t.counters(camera, "film"), 3);
    let pinnacle = t.battlefield(P0, "Helix Pinnacle");
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    activate_containing(&mut t, P0, pinnacle, "tower").unwrap();
    t.lands(P0, "Wastes", 2);
    copy_top_ability(&mut t, camera, "Copy target");
    assert_eq!(t.counters(camera, "film"), 2);
    t.resolve_all();
    assert_eq!(t.counters(pinnacle, "tower"), 4);
}

#[test]
fn choices_and_costs_on_resolution_are_made_separately_for_a_copied_ability() {
    cr!("707.10", "608.2", "603.5");
    ruling!(
        "Peter Parker's Camera",
        "Any choices made when the ability resolves won't have been made yet when it's copied. Any such choices will be made separately when the copy resolves. If a triggered ability asks you to pay a cost, you pay that cost for the copy separately."
    );
    supported("Keldon Raider");
    // Keldon Raider: "When this creature enters, you may discard a card. If you do, draw a
    // card."
    let mut t = TestGame::new(2);
    let camera = t.enter(P0, "Peter Parker's Camera");
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    t.enter(P0, "Keldon Raider");
    t.settle();
    t.lands(P0, "Wastes", 2);
    copy_top_ability(&mut t, camera, "Copy target");
    let from = t.asked().len();
    // The copy: discard (and draw). The original: don't.
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    t.resolve_all();
    let asked = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
        .count();
    assert_eq!(asked, 2);
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.hand_size(P0), 2);
}

#[test]
fn a_copied_permanent_spell_enters_as_a_token() {
    cr!("707.10f", "608.3f", "603.6a");
    ruling!(
        "Lithoform Engine",
        "If a permanent spell is copied, it's put onto the battlefield as a token as the spell resolves rather than putting the copy of the spell onto the battlefield. The rules that apply to a permanent spell becoming a permanent apply to a copy of a spell becoming a token."
    );
    // "{4}, {T}: Copy target permanent spell you control. (The copy becomes a token.)"
    // Elvish Visionary: "When this creature enters, draw a card."
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    let hand = t.hand_size(P0);
    let visionary = cast_new(&mut t, P0, "Elvish Visionary", &[]);
    t.lands(P0, "Wastes", 4);
    t.answer_targets(P0, &[Entity::Object(visionary)]);
    activate_containing(&mut t, P0, engine, "permanent spell").unwrap();
    t.resolve_all();
    let all = t.named_on_battlefield("Elvish Visionary");
    assert_eq!(all.len(), 2);
    assert_eq!(all.iter().filter(|id| t.obj(**id).is_token()).count(), 1);
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn the_token_a_copied_spell_becomes_isnt_created() {
    cr!("707.10f", "608.3f", "701.7a");
    ruling!(
        "Lithoform Engine",
        "The token that a resolving copy of a spell becomes isn't said to have been “created.”"
    );
    supported("Parallel Lives");
    let mut t = TestGame::new(2);
    let engine = t.battlefield(P0, "Lithoform Engine");
    t.battlefield(P0, "Parallel Lives");
    let bears = cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.lands(P0, "Wastes", 4);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    activate_containing(&mut t, P0, engine, "permanent spell").unwrap();
    t.resolve_all();
    // Parallel Lives doesn't double it: one token.
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
}

#[test]
fn a_copy_of_a_linked_ability_is_linked_too() {
    cr!("707.10", "607.2a", "707.7");
    ruling!(
        "Virtue of Knowledge // Vantress Visions",
        "If an ability is linked to a second ability, copies of that ability are also linked to that second ability. If the second ability refers to \"the exiled card,\" it refers to all cards exiled by the ability and the copy."
    );
    supported("Virtue of Knowledge // Vantress Visions");
    supported("Fiend Hunter");
    // Vantress Visions: "Copy target activated or triggered ability you control. You may
    // choose new targets for the copy." Fiend Hunter: "When this creature enters, you may
    // exile another target creature. When this creature leaves the battlefield, return the
    // exiled card to the battlefield under its owner's control."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(a)]);
    let hunter = t.enter(P0, "Fiend Hunter");
    t.settle();
    let trigger = *t.g.stack.last().unwrap();
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 1);
    let card = t.hand(P0, "Virtue of Knowledge // Vantress Visions");
    t.cast(P0, card)
        .method(ADVENTURE)
        .target(Entity::Object(trigger))
        .go();
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(b))]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    // Both exiled cards return when Fiend Hunter leaves.
    destroy(&mut t, hunter);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

/// The candidates P0 was offered for the first target chosen since decision `from`.
fn first_offered(t: &TestGame, from: usize) -> Vec<Entity> {
    target_candidates(t, P0, from)
        .first()
        .cloned()
        .unwrap_or_default()
}

#[test]
fn an_ability_copier_can_be_limited_to_abilities_from_certain_sources() {
    cr!("707.10", "601.2c", "602.2b");
    supported("Tawnos, Urza's Apprentice");
    supported("Abstruse Archaic");
    supported("The Peregrine Dynamo");
    // Tawnos (legendary): "{U}{R}, {T}: Copy target activated or triggered ability you
    // control from an artifact source." Abstruse Archaic: "... from a colorless source."
    // The Peregrine Dynamo: "... from another legendary source that's not a commander."
    let mut t = TestGame::new(2);
    let tawnos = t.battlefield(P0, "Tawnos, Urza's Apprentice");
    let archaic = t.battlefield(P0, "Abstruse Archaic");
    let dynamo = t.battlefield(P0, "The Peregrine Dynamo");
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    let bomb = t.battlefield(P0, "Ratchet Bomb");
    // A red creature's ability and a colorless artifact's ability.
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let ping = activate_containing(&mut t, P0, pyro, "damage")
        .unwrap()
        .unwrap();
    let charge = activate_containing(&mut t, P0, bomb, "Put a charge counter")
        .unwrap()
        .unwrap();
    t.lands(P0, "Island", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    // Tawnos: only the artifact's ability.
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(charge)]);
    let tawnos_ability = activate_containing(&mut t, P0, tawnos, "Copy target")
        .unwrap()
        .unwrap();
    let c = first_offered(&t, from);
    assert!(c.contains(&Entity::Object(charge)));
    assert!(!c.contains(&Entity::Object(ping)));
    // Abstruse Archaic: only the colorless source's ability (not the blue-red Tawnos's).
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(charge)]);
    activate_containing(&mut t, P0, archaic, "Copy target").unwrap();
    let c = first_offered(&t, from);
    assert!(c.contains(&Entity::Object(charge)));
    assert!(!c.contains(&Entity::Object(ping)));
    assert!(!c.contains(&Entity::Object(tawnos_ability)));
    // The Peregrine Dynamo: only the legendary Tawnos's ability.
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Object(tawnos_ability)]);
    activate_containing(&mut t, P0, dynamo, "Copy target").unwrap();
    let c = first_offered(&t, from);
    assert!(c.contains(&Entity::Object(tawnos_ability)));
    assert!(!c.contains(&Entity::Object(charge)));
    assert!(!c.contains(&Entity::Object(ping)));
}

#[test]
fn a_dynamo_copy_of_a_linked_trigger_is_linked_too() {
    cr!("707.10", "607.2a", "607.3");
    ruling!(
        "The Peregrine Dynamo",
        "If an ability is linked to a second ability, copies of that ability are also linked to that second ability. If the second ability refers to “the exiled card,” it refers to all cards exiled by the ability and the copy."
    );
    supported("Leyline of Singularity");
    // Leyline of Singularity: "All nonland permanents are legendary." — so Fiend Hunter
    // ("When this creature enters, you may exile another target creature. When this
    // creature leaves the battlefield, return the exiled card to the battlefield under its
    // owner's control.") is another legendary source for The Peregrine Dynamo.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Leyline of Singularity");
    let dynamo = t.battlefield(P0, "The Peregrine Dynamo");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(a)]);
    let hunter = t.enter(P0, "Fiend Hunter");
    t.settle();
    assert!(legendary(&t, hunter));
    t.lands(P0, "Wastes", 1);
    copy_top_ability(&mut t, dynamo, "Copy target");
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(b))]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    // Both exiled cards return when Fiend Hunter leaves.
    destroy(&mut t, hunter);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}
