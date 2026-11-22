//! Rulings batch S25 — how a copy of a spell resolves: its controller is the player who
//! copied it and "you" refers to them; it resolves before the original; choices made on
//! resolution are made separately for it (CR 707.10, 608.2); a copy that leaves the stack
//! ceases to exist (CR 707.10a, 704.5e); a copy isn't cast, so it doesn't trigger "whenever
//! you cast a spell that targets" abilities.

use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s11_common::triggered_from;
use crate::r_s18_common::ADVENTURE;
use crate::r_s24_common::choose_creature_type;
use crate::r_s25_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn choices_made_on_resolution_are_made_separately_for_the_copy() {
    cr!("707.10", "608.2");
    ruling!(
        "Mica, Reader of Ruins",
        "Any choices made when the spell resolves won't have been made yet when it's copied. Any such choices will be made separately when the copy resolves."
    );
    supported("Mica, Reader of Ruins");
    supported("Distant Melody");
    // Mica: "Whenever you cast an instant or sorcery spell, you may sacrifice an artifact.
    // If you do, copy that spell and you may choose new targets for the copy." Distant
    // Melody: "Choose a creature type. Draw a card for each permanent you control of that
    // type."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mica, Reader of Ruins");
    let thopter = t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Goblin Piker");
    t.battlefield(P0, "Goblin Piker");
    t.battlefield(P0, "Llanowar Elves");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(thopter)]);
    cast_new(&mut t, P0, "Distant Melody", &[]);
    let hand = t.hand_size(P0);
    // The copy resolves first: Elf (one card); then the original: Goblin (two cards).
    choose_creature_type(&mut t, P0, "Elf");
    choose_creature_type(&mut t, P0, "Goblin");
    t.resolve();
    assert_eq!(spell_copies(&t).len(), 1);
    t.resolve();
    assert_eq!(t.hand_size(P0), hand + 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn a_guildmage_copy_makes_its_own_choices_on_resolution() {
    cr!("707.10", "608.2", "701.22a");
    ruling!(
        "Izzet Guildmage",
        "Any choices made as a spell resolves won't have been made yet once it's copied. Any such choices will be made separately as the copy resolves."
    );
    supported("Izzet Guildmage");
    // "{2}{U}: Copy target instant spell you control with mana value 2 or less." Opt:
    // "Scry 1. Draw a card."
    let mut t = TestGame::new(2);
    let guildmage = t.battlefield(P0, "Izzet Guildmage");
    let opt = cast_new(&mut t, P0, "Opt", &[]);
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    t.answer_targets(P0, &[Entity::Object(opt)]);
    activate_containing(&mut t, P0, guildmage, "instant spell").unwrap();
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.resolve_all();
    let scries = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, Decision::Scry { .. }))
        .count();
    assert_eq!(scries, 2);
    assert_eq!(t.hand_size(P0), hand + 2);
}

/// P1 (the active player) casts `spell`; P0 copies it by casting `copier` (targets kept).
/// Returns P1's spell.
fn p0_copies_p1_s(t: &mut TestGame, spell: &str, copier: &str) -> ObjectId {
    t.set_step(P1, Step::PrecombatMain);
    let s = cast_new(t, P1, spell, &[]);
    cast_new(t, P0, copier, &[Entity::Object(s)]);
    keep_copy_targets(t, P0);
    t.resolve();
    s
}

#[test]
fn you_control_the_copy_and_it_resolves_first() {
    cr!("707.10", "405.2", "405.5");
    ruling!(
        "Flare of Duplication",
        "If you copy a spell, you control the copy. It will resolve before the original spell does."
    );
    supported("Flare of Duplication");
    // P1 casts Divination ("Draw two cards."); P0 copies it.
    let mut t = TestGame::new(2);
    p0_copies_p1_s(&mut t, "Divination", "Flare of Duplication");
    let copy = spell_copies(&t)[0];
    assert_eq!(t.obj(copy).controller, P0);
    assert_eq!(*t.g.stack.last().unwrap(), copy);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    t.resolve();
    assert_eq!(t.hand_size(P0), h0 + 2);
    assert_eq!(t.hand_size(P1), h1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), h1 + 2);
}

#[test]
fn you_and_opponent_refer_to_the_copy_s_controller() {
    cr!("707.10", "109.5");
    ruling!(
        "Reverberate",
        "If the copy says that it affects \"you,\" it affects the controller of the copy, not the controller of the original spell. Similarly, if the copy says that it affects an \"opponent,\" it affects an opponent of the copy's controller, not an opponent of the original spell's controller."
    );
    supported("Reverberate");
    // P1 casts Blood Tithe ("Each opponent loses 3 life. You gain life equal to the life
    // lost this way."); P0's copy drains P1.
    let mut t = TestGame::new(2);
    p0_copies_p1_s(&mut t, "Blood Tithe", "Reverberate");
    t.resolve();
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.life(P1), 17);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn a_copy_returned_to_hand_ceases_to_exist() {
    cr!("707.10a", "704.5e");
    ruling!(
        "Venser, Shaper Savant",
        "If a copy of a spell is returned to its owner's hand, it's moved there, then it will cease to exist as a state-based action. It can't be recast."
    );
    supported("Venser, Shaper Savant");
    // "When Venser enters, return target spell or permanent to its owner's hand."
    let mut t = TestGame::new(2);
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    cast_new(&mut t, P0, "Twincast", &[Entity::Object(bolt)]);
    keep_copy_targets(&mut t, P0);
    t.resolve();
    let copy = spell_copies(&t)[0];
    let hand = t.hand_size(P0);
    cast_new(&mut t, P1, "Venser, Shaper Savant", &[]);
    t.answer_targets(P1, &[Entity::Object(copy)]);
    t.resolve();
    t.resolve();
    // The copy left the stack and ceased to exist; it isn't in P0's hand.
    assert!(!t.g.stack.contains(&copy));
    assert_eq!(t.hand_size(P0), hand);
    assert!(!t.in_hand(P0, "Lightning Bolt"));
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_copy_of_an_adventure_isnt_exiled_to_be_cast_later() {
    cr!("707.10a", "715.3d");
    ruling!(
        "Lucky Clover",
        "If an effect copies an Adventure spell, that copy is exiled as it resolves. It ceases to exist as a state-based action; it's not possible to cast the copy as a creature."
    );
    supported("Lucky Clover");
    supported("Bonecrusher Giant // Stomp");
    // "Whenever you cast an Adventure instant or sorcery spell, copy it. You may choose new
    // targets for the copy." Stomp: "Damage can't be prevented this turn. Stomp deals 2
    // damage to any target."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lucky Clover");
    t.lands(P0, "Mountain", 2);
    let giant = t.hand(P0, "Bonecrusher Giant // Stomp");
    t.cast(P0, giant).method(ADVENTURE).target(Entity::Player(P1)).go();
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    // Only the card is in exile (and may be cast as the creature later).
    assert_eq!(t.g.exile.len(), 1);
    assert_eq!(t.zone(giant), Zone::Exile);
}

#[test]
fn copying_a_spell_that_targets_a_hero_doesnt_trigger_it() {
    cr!("707.10", "603.2", "115.7d");
    ruling!(
        "Hero of the Nyxborn",
        "This ability doesn’t trigger if you copy a spell that targets it, or if a spell’s targets are changed to target it."
    );
    supported("Hero of the Nyxborn");
    // "Whenever you cast a spell that targets this creature, creatures you control get
    // +1/+0 until end of turn."
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Hero of the Nyxborn");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Giant Growth on the Hero: one trigger; Twincast copies it: no trigger.
    let growth = cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(hero)]);
    t.settle();
    assert_eq!(triggered_from(&t, hero), 1);
    cast_new(&mut t, P0, "Twincast", &[Entity::Object(growth)]);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(triggered_from(&t, hero), 1);
    // Giant Growth on the Bears, its copy changed to target the Hero: no trigger.
    let growth = cast_new(&mut t, P0, "Giant Growth", &[Entity::Object(bears)]);
    cast_new(&mut t, P0, "Twincast", &[Entity::Object(growth)]);
    change_copy_targets(&mut t, P0, &[Some(Entity::Object(hero))]);
    t.resolve_all();
    assert_eq!(triggered_from(&t, hero), 1);
    // The Hero got +3/+3 three times and +1/+0 once.
    assert_eq!(t.pt(hero), (2 + 9 + 1, 2 + 9));
}
