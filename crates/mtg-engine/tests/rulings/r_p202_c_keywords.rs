//! Rulings batch P202 — celebration (an ability word, CR 207.2c), changeling (CR 702.73)
//! and channel (an ability word).

use crate::r_s01_common::*;
use crate::r_s02_common::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------
// Celebration
// ---------------------------------------------------------------------------------

#[test]
fn belligerent_of_the_ball_gives_the_same_bonus_for_more_than_two_permanents() {
    cr!("207.2c", "603.4");
    ruling!(
        "Belligerent of the Ball",
        "Celebration abilities only care if two or more nonland permanents entered the battlefield under your control in a turn. They won’t get more powerful if more than two permanents entered the battlefield under your control in a turn."
    );
    supported("Belligerent of the Ball");
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Belligerent of the Ball");
    t.enter(P0, "Ornithopter");
    t.enter(P0, "Memnite");
    t.enter(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(ogre)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "target creature you control gets"), 1);
    t.resolve_all();
    assert_eq!(t.pt(ogre), (4, 3));
    assert!(t.obj_now(ogre).chars.has_keyword(KeywordKind::Menace));
}

#[test]
fn belligerent_of_the_ball_counts_permanents_that_already_left() {
    cr!("207.2c", "603.4");
    ruling!(
        "Belligerent of the Ball",
        "The permanents that entered the battlefield don’t need to remain on the battlefield or under your control. Celebration abilities are checking for past events, not the current game state."
    );
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Belligerent of the Ball");
    let a = t.enter(P0, "Ornithopter");
    let b = t.enter(P0, "Memnite");
    destroy(&mut t, a);
    // The other one is now controlled by the opponent.
    t.g.objects[b.0 as usize].controller = P1;
    t.g.objects[b.0 as usize].base_controller = P1;
    t.g.recompute();
    t.answer_targets(P0, &[Entity::Object(ogre)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "target creature you control gets"), 1);
    t.resolve_all();
    assert_eq!(t.pt(ogre), (4, 3));
}

// ---------------------------------------------------------------------------------
// Changeling
// ---------------------------------------------------------------------------------

#[test]
fn shapesharer_copying_another_creature_loses_changeling_and_its_ability() {
    cr!("702.73a", "707.2", "613.1a");
    ruling!(
        "Shapesharer",
        "If Shapesharer itself becomes a copy of another creature, it loses both changeling and its activated ability (unless it’s copying another creature with changeling and/or another Shapesharer, of course)."
    );
    supported("Shapesharer");
    let mut t = TestGame::new(2);
    let sharer = t.battlefield(P0, "Shapesharer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(t.obj(sharer).chars.has_subtype("Elf"));
    t.lands(P0, "Island", 3);
    t.activate(
        P0,
        sharer,
        0,
        &[Entity::Object(sharer), Entity::Object(bears)],
    )
    .unwrap();
    t.resolve_all();
    let o = t.obj_now(sharer);
    assert_eq!(o.chars.name.as_str(), "Grizzly Bears");
    assert!(!o.chars.has_keyword(KeywordKind::Changeling));
    assert!(!o.chars.has_subtype("Shapeshifter") && !o.chars.has_subtype("Elf"));
    assert!(o.chars.has_subtype("Bear"));
    assert!(!can_activate(&mut t, P0, sharer));

    // Copying another changeling, it keeps changeling.
    let mut t = TestGame::new(2);
    let sharer = t.battlefield(P0, "Shapesharer");
    let moth = t.battlefield(P1, "Mothdust Changeling");
    t.lands(P0, "Island", 3);
    t.activate(P0, sharer, 0, &[Entity::Object(sharer), Entity::Object(moth)])
        .unwrap();
    t.resolve_all();
    let o = t.obj_now(sharer);
    assert_eq!(o.chars.name.as_str(), "Mothdust Changeling");
    assert!(o.chars.has_keyword(KeywordKind::Changeling));
    assert!(o.chars.has_subtype("Elf"));
}

#[test]
fn webweaver_changeling_checks_the_graveyard_on_trigger_and_resolution() {
    cr!("603.4", "702.73a");
    ruling!(
        "Webweaver Changeling",
        "If there aren't three or more creature cards in your graveyard as Webweaver Changeling enters the battlefield, its ability doesn't trigger at all. If there aren't three or more creature cards in your graveyard as the ability resolves, you don't gain 5 life. These don't have the be same creature cards at both times."
    );
    supported("Webweaver Changeling");
    // Two creature cards: no trigger.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Grizzly Bears");
    t.enter(P0, "Webweaver Changeling");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "you gain 5 life"), 0);
    // Three: it triggers, but one leaves before it resolves: no life.
    let mut t = TestGame::new(2);
    let g1 = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Grizzly Bears");
    t.enter(P0, "Webweaver Changeling");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "you gain 5 life"), 1);
    t.g.move_object(g1, Zone::Exile, events::MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // One is replaced by another creature card before it resolves: 5 life.
    let mut t = TestGame::new(2);
    let g1 = t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Grizzly Bears");
    t.enter(P0, "Webweaver Changeling");
    t.settle();
    t.g.move_object(g1, Zone::Exile, events::MoveCause::Effect, None);
    t.graveyard(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
}

#[test]
fn mothdust_changeling_can_tap_a_summoning_sick_creature_for_its_cost() {
    cr!("302.6", "602.5a");
    ruling!(
        "Mothdust Changeling",
        "Since the activated ability doesn't have a tap symbol in its cost, you can tap a creature (including Mothdust Changeling itself) that hasn't been under your control since your most recent turn began to pay the cost."
    );
    supported("Mothdust Changeling");
    // Tapping itself, summoning sick.
    let mut t = TestGame::new(2);
    let moth = t.battlefield_sick(P0, "Mothdust Changeling");
    assert!(can_activate(&mut t, P0, moth));
    t.answer_choose(P0, &[Entity::Object(moth)]);
    t.activate(P0, moth, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(moth).tapped);
    assert!(t.obj_now(moth).chars.has_keyword(KeywordKind::Flying));
    // Tapping another summoning-sick creature.
    let mut t = TestGame::new(2);
    let moth = t.battlefield_sick(P0, "Mothdust Changeling");
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.activate(P0, moth, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert!(!t.obj_now(moth).tapped);
    assert!(t.obj_now(moth).chars.has_keyword(KeywordKind::Flying));
}

// ---------------------------------------------------------------------------------
// Channel
// ---------------------------------------------------------------------------------

#[test]
fn touch_the_spirit_realm_returns_a_double_faced_card_front_face_up() {
    cr!("712.8a", "400.7", "610.3");
    ruling!(
        "Touch the Spirit Realm",
        "If a double-faced card is exiled and returned to the battlefield with Touch the Spirit Realm or its channel ability, that card will return to the battlefield front-face up."
    );
    supported("Touch the Spirit Realm");
    supported("Grub, Storied Matriarch");
    // The channel ability.
    let mut t = TestGame::new(2);
    let grub = t.battlefield(P1, "Grub, Storied Matriarch");
    mtg_engine::dfc::transform(&mut t.g, grub);
    t.g.recompute();
    assert_eq!(t.obj(grub).chars.name.as_str(), "Grub, Notorious Auntie");
    t.lands(P0, "Plains", 2);
    let touch = t.hand(P0, "Touch the Spirit Realm");
    t.activate(P0, touch, 0, &[Entity::Object(grub)]).unwrap();
    assert!(t.in_graveyard(P0, "Touch the Spirit Realm"));
    t.resolve_all();
    assert_eq!(t.zone(grub), Zone::Exile);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = t.named_on_battlefield("Grub, Storied Matriarch");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj(back[0]).face, FaceState::Front);
    assert_eq!(t.obj(back[0]).controller, P1);

    // The enters ability: exiled until Touch the Spirit Realm leaves.
    let mut t = TestGame::new(2);
    let grub = t.battlefield(P1, "Grub, Storied Matriarch");
    mtg_engine::dfc::transform(&mut t.g, grub);
    t.g.recompute();
    t.answer_targets(P0, &[Entity::Object(grub)]);
    let touch = t.enter(P0, "Touch the Spirit Realm");
    t.resolve_all();
    assert_eq!(t.zone(grub), Zone::Exile);
    destroy(&mut t, touch);
    t.resolve_all();
    let back = t.named_on_battlefield("Grub, Storied Matriarch");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj(back[0]).face, FaceState::Front);
}

// Omni-Changeling: "You may have this creature enter as a copy of any creature on the
// battlefield, except it has changeling."

/// P0's Omni-Changeling enters as a copy of `of` (or of nothing).
fn omni_enters(t: &mut TestGame, of: Option<ObjectId>) -> ObjectId {
    supported("Omni-Changeling");
    t.answer_yes(P0, of.is_some());
    if let Some(o) = of {
        t.answer_choose(P0, &[Entity::Object(o)]);
    }
    t.enter(P0, "Omni-Changeling")
}

#[test]
fn omni_changeling_copies_printed_values_and_has_changeling() {
    cr!("707.2", "707.9b", "702.73a");
    ruling!(
        "Omni-Changeling",
        "Omni-Changeling copies exactly what was printed on the original creature and nothing else, with the listed exception (unless that creature is copying something else or is a token; see below). It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.objects[giant.0 as usize].tapped = true;
    t.g.objects[giant.0 as usize]
        .counters
        .insert(counters::PLUS1.into(), 2);
    t.g.recompute();
    assert_eq!(t.pt(giant), (5, 5));
    let omni = omni_enters(&mut t, Some(giant));
    let o = t.obj_now(omni);
    assert_eq!(o.chars.name.as_str(), "Hill Giant");
    assert!(!o.tapped);
    assert_eq!(t.counters(omni, counters::PLUS1), 0);
    assert_eq!(t.pt(omni), (3, 3));
    assert!(o.chars.has_keyword(KeywordKind::Changeling));
    assert!(o.chars.has_subtype("Giant") && o.chars.has_subtype("Elf"));
}

#[test]
fn omni_changeling_gets_the_copied_creatures_enters_abilities() {
    cr!("707.9", "614.1c", "603.6a");
    ruling!(
        "Omni-Changeling",
        "Any \"enters\" abilities of the copied creature will trigger when Omni-Changeling enters. Any \"as this creature enters\" or \"this creature enters with\" abilities of the copied creature will also work."
    );
    supported("Elvish Visionary");
    supported("Spike Feeder");
    let mut t = TestGame::new(2);
    let visionary = t.battlefield(P1, "Elvish Visionary");
    let hand = t.hand_size(P0);
    omni_enters(&mut t, Some(visionary));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Spike Feeder: "This creature enters with two +1/+1 counters on it."
    let feeder = t.battlefield(P1, "Spike Feeder");
    let omni = omni_enters(&mut t, Some(feeder));
    assert_eq!(t.counters(omni, counters::PLUS1), 2);
}

#[test]
fn omni_changeling_copying_a_token_uses_the_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Omni-Changeling",
        "If the copied creature is a token, Omni-Changeling copies the original characteristics of that token as stated by the effect that created that token, with the listed exception."
    );
    let mut t = TestGame::new(2);
    let soldier = create_token(&mut t, P1, "Soldier");
    t.g.objects[soldier.0 as usize]
        .counters
        .insert(counters::PLUS1.into(), 1);
    t.g.recompute();
    let omni = omni_enters(&mut t, Some(soldier));
    let o = t.obj_now(omni);
    assert!(!o.is_token());
    assert_eq!(o.chars.name.as_str(), "Soldier");
    assert_eq!(t.pt(omni), (1, 1));
    assert!(o.chars.has_keyword(KeywordKind::Changeling));
}

#[test]
fn omni_changeling_copying_a_copy_gets_what_it_copied() {
    cr!("707.3", "707.9b");
    ruling!(
        "Omni-Changeling",
        "If the copied creature is copying something else, then Omni-Changeling enters as whatever that creature copied, with the listed exception."
    );
    supported("Clone");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(giant)]);
    let clone = t.enter(P1, "Clone");
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Hill Giant");
    let omni = omni_enters(&mut t, Some(clone));
    let o = t.obj_now(omni);
    assert_eq!(o.chars.name.as_str(), "Hill Giant");
    assert_eq!(t.pt(omni), (3, 3));
    assert!(o.chars.has_keyword(KeywordKind::Changeling));
}

#[test]
fn omni_changeling_copying_nothing_is_a_0_0_that_dies() {
    cr!("707.9", "704.5f");
    ruling!(
        "Omni-Changeling",
        "You can choose not to have Omni-Changeling enter as a copy of another creature. If you do, it will just be a 0/0 Shapeshifter with changeling, and unless another effect is increasing its toughness, it will be put into its owner's graveyard."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Hill Giant");
    let omni = omni_enters(&mut t, None);
    let o = t.obj_now(omni);
    assert_eq!(o.chars.name.as_str(), "Omni-Changeling");
    assert!(o.chars.has_keyword(KeywordKind::Changeling));
    t.settle();
    assert!(t.in_graveyard(P0, "Omni-Changeling"));
}
