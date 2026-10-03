//! CR 702.69 Gravestorm.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_038_051::{spell_copies_on_stack, triggers_named, with_cost};
use crate::common_k702_052_066::{destroy, run_effect};
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::*;

/// Kills `n` Grizzly Bears controlled by `p` (destroyed: put into the graveyard from the
/// battlefield).
fn kill_bears(t: &mut TestGame, p: PlayerId, n: usize) {
    for _ in 0..n {
        let b = t.battlefield(p, "Grizzly Bears");
        destroy(t, b);
    }
    t.resolve_all();
}

#[test]
fn gravestorm_copies_the_spell_for_each_permanent_put_into_a_graveyard_this_turn() {
    cr!("702.69", "702.69a");
    assert_supported("Ominous Harvest");
    assert_supported("Bitter Ordeal");
    let mut t = TestGame::new(2);
    kill_bears(&mut t, P0, 1);
    kill_bears(&mut t, P1, 1);
    // Leaving the battlefield another way, or going to the graveyard from elsewhere,
    // doesn't count.
    let exiled = t.battlefield(P1, "Grizzly Bears");
    run_effect(
        &mut t,
        None,
        P0,
        Effect::Exile {
            what: Sel::Target(0),
            face_down: false,
            link: false,
        },
        &[Entity::Object(exiled)],
    );
    let discarded = t.hand(P1, "Grizzly Bears");
    run_effect(
        &mut t,
        None,
        P1,
        Effect::Discard {
            who: PlayerRef::You,
            n: Value::c(1),
            random: false,
            filter: Filter::Any,
        },
        &[],
    );
    assert_eq!(t.zone(discarded), Zone::Graveyard(P1));
    t.lands(P0, "Swamp", 3);
    let harvest = t.hand(P0, "Ominous Harvest");
    let p1_hand = t.hand_size(P1);
    t.cast(P0, harvest).target(P1).go();
    t.settle();
    assert_eq!(triggers_named(&t, "Gravestorm").len(), 1);
    t.resolve();
    assert_eq!(spell_copies_on_stack(&t, "Ominous Harvest"), 2);
    t.resolve_all();
    // The spell and its two copies: three cards and three life.
    assert_eq!(t.hand_size(P1), p1_hand + 3);
    assert_eq!(t.life(P1), 17);
    // The copies weren't cast.
    assert_eq!(t.g.history.spells_cast.len(), 1);
}

#[test]
fn tokens_put_into_a_graveyard_count() {
    cr!("702.69a");
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, "Grizzly Bears");
    run_effect(
        &mut t,
        Some(src),
        P0,
        Effect::CreateTokenCopy {
            of: Sel::Target(0),
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
            mods: vec![],
        },
        &[Entity::Object(src)],
    );
    let token = t
        .g
        .permanents()
        .find(|o| o.is_token())
        .map(|o| o.id)
        .expect("token");
    destroy(&mut t, token);
    t.resolve_all();
    t.lands(P0, "Swamp", 3);
    let harvest = t.hand(P0, "Ominous Harvest");
    t.cast(P0, harvest).target(P1).go();
    t.resolve();
    assert_eq!(spell_copies_on_stack(&t, "Ominous Harvest"), 1);
}

#[test]
fn the_number_of_copies_is_counted_as_the_trigger_resolves() {
    cr!("702.69a");
    let mut t = TestGame::new(2);
    kill_bears(&mut t, P1, 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 3);
    let harvest = t.hand(P0, "Ominous Harvest");
    t.cast(P0, harvest).target(P1).go();
    t.settle();
    // In response to the gravestorm trigger, another permanent dies.
    destroy(&mut t, bears);
    t.settle();
    t.resolve();
    assert_eq!(spell_copies_on_stack(&t, "Ominous Harvest"), 2);
}

#[test]
fn gravestorm_copies_may_have_new_targets() {
    cr!("702.69a");
    let mut t = TestGame::new(2);
    kill_bears(&mut t, P1, 1);
    t.lands(P0, "Swamp", 3);
    let harvest = t.hand(P0, "Ominous Harvest");
    t.cast(P0, harvest).target(P1).go();
    // The copy targets P0 instead.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn with_nothing_put_into_a_graveyard_there_are_no_copies() {
    cr!("702.69a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 3);
    let harvest = t.hand(P0, "Ominous Harvest");
    t.cast(P0, harvest).target(P1).go();
    t.resolve();
    assert_eq!(spell_copies_on_stack(&t, "Ominous Harvest"), 0);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn each_instance_of_gravestorm_triggers_separately() {
    cr!("702.69b");
    let def = with_cost(
        custom_card(
            "Twice-Grieving Harvest",
            "Sorcery",
            None,
            "Gravestorm\nGravestorm\nTarget player loses 1 life.",
        ),
        "{B}",
    );
    let mut t = TestGame::new(2);
    kill_bears(&mut t, P1, 2);
    t.lands(P0, "Swamp", 1);
    let spell = t.custom(P0, def, Zone::Hand(P0));
    t.cast(P0, spell).target(P1).go();
    t.settle();
    assert_eq!(triggers_named(&t, "Gravestorm").len(), 2);
    t.resolve_all();
    // The spell plus two copies for each of the two triggers.
    assert_eq!(t.life(P1), 15);
}
