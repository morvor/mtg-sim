//! Rulings batch P017 — permanents that may enter as a copy of a permanent on the
//! battlefield (Clever Impersonator, Copy Enchantment, Estrid's Invocation, Mirrormade,
//! Sculpting Steel, Copy Artifact, Masterwork of Ingenuity): what they can choose, what
//! they copy (CR 707.2, 707.3, 707.9), and copying Auras and Equipment (CR 303.4f–g,
//! 301.5e).

use crate::r_p017_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::create_token;
use crate::r_s24_common::enter_together;
use crate::r_s26_common::{dress_up, fresh, modify_until_eot, mv};
use mtg_engine::ability::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn has_triggered_ability(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id)
        .chars
        .abilities
        .iter()
        .any(|a| matches!(a.kind, AbilityKind::Triggered(_)))
}

// ---------------------------------------------------------------------------
// What can be chosen
// ---------------------------------------------------------------------------

/// `name` enters at the same time as `other`: it can't copy `other`, only `existing`,
/// which was already on the battlefield.
fn cant_copy_what_enters_with_it(name: &str, other: &str, existing: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let old = t.battlefield(P0, existing);
    let from = t.asked().len();
    let entered = enter_together(&mut t, &[(P0, name), (P0, other)]);
    let offered = choice_candidates(&t, from);
    assert_eq!(offered.len(), 1, "{name}: asked {offered:?}");
    assert!(offered[0].contains(&Entity::Object(old)), "{name}");
    assert!(
        !offered[0].contains(&Entity::Object(entered[1])),
        "{name} could copy {other} entering at the same time"
    );
}

#[test]
fn clever_impersonator_cant_copy_a_permanent_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Clever Impersonator",
        "If Clever Impersonator enters the battlefield at the same time as another permanent, it can't become a copy of that permanent."
    );
    cant_copy_what_enters_with_it("Clever Impersonator", "Grizzly Bears", "Hill Giant");
}

#[test]
fn copy_enchantment_cant_copy_a_permanent_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Copy Enchantment",
        "If Copy Enchantment somehow enters the battlefield at the same time as another permanent, Copy Enchantment can't become a copy of that permanent."
    );
    cant_copy_what_enters_with_it("Copy Enchantment", "Glorious Anthem", "Bad Moon");
}

#[test]
fn estrids_invocation_cant_copy_an_enchantment_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Estrid's Invocation",
        "If Estrid's Invocation somehow enters the battlefield at the same time as another enchantment, it can't become a copy of that enchantment."
    );
    cant_copy_what_enters_with_it("Estrid's Invocation", "Glorious Anthem", "Bad Moon");
}

#[test]
fn mirrormade_cant_copy_a_permanent_entering_at_the_same_time() {
    cr!("707.2", "614.12");
    ruling!(
        "Mirrormade",
        "If Mirrormade somehow enters the battlefield at the same time as another artifact or enchantment, it can't become a copy of that permanent."
    );
    cant_copy_what_enters_with_it("Mirrormade", "Sol Ring", "Glorious Anthem");
}

#[test]
fn clever_impersonator_doesnt_target_what_it_copies() {
    cr!("707.2", "115.1");
    ruling!(
        "Clever Impersonator",
        "You choose which nonland permanent Clever Impersonator will copy, if any, as it enters the battlefield. This doesn't target that nonland permanent."
    );
    supported("Gladecover Scout");
    // An opponent's creature with hexproof can be chosen; a land can't.
    let mut t = TestGame::new(2);
    let scout = t.battlefield(P1, "Gladecover Scout");
    let land = t.battlefield(P1, "Forest");
    let from = t.asked().len();
    let c = enter_copying(&mut t, P0, "Clever Impersonator", Some(scout));
    assert!(!choice_candidates(&t, from)[0].contains(&Entity::Object(land)));
    assert_eq!(t.obj_now(c).chars.name, "Gladecover Scout");
    assert_eq!(t.obj_now(c).controller, P0);
}

#[test]
fn estrids_invocation_cant_copy_itself_and_has_no_upkeep_ability_without_a_copy() {
    cr!("707.9a", "707.2");
    ruling!(
        "Estrid's Invocation",
        "If Estrid's Invocation doesn't copy an enchantment as it enters the battlefield, it won't have the ability to exile it at the beginning of your upkeep. You can't have it copy itself to get this ability."
    );
    supported("Estrid's Invocation");
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    let inv = enter_copying(&mut t, P0, "Estrid's Invocation", None);
    // With no other enchantment around, there's nothing to choose: not even itself.
    assert!(choice_candidates(&t, from)
        .iter()
        .all(|c| !c.contains(&Entity::Object(inv))));
    assert_eq!(t.obj_now(inv).chars.name, "Estrid's Invocation");
    assert!(!has_triggered_ability(&t, inv));
    // Its upkeep comes and goes: nothing triggers, and it stays where it is.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Draw);
    assert!(t.on_battlefield(inv));
    assert_eq!(t.g.current(inv), inv);
}

// ---------------------------------------------------------------------------
// Choosing not to copy
// ---------------------------------------------------------------------------

#[test]
fn clever_impersonator_copying_nothing_is_a_0_0_and_dies() {
    cr!("707.2", "704.5f");
    ruling!(
        "Clever Impersonator",
        "You can choose not to copy anything. In that case, Clever Impersonator enters the battlefield as a 0/0 creature, and it's put into the graveyard immediately"
    );
    supported("Clever Impersonator");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Grizzly Bears");
    let c = enter_copying(&mut t, P0, "Clever Impersonator", None);
    assert!(!t.on_battlefield(c));
    assert!(t.in_graveyard(P0, "Clever Impersonator"));
    // Unless something raises its toughness above 0.
    t.battlefield(P0, "Glorious Anthem");
    let c = enter_copying(&mut t, P0, "Clever Impersonator", None);
    assert!(t.on_battlefield(c));
    assert_eq!(t.pt(c), (1, 1));
}

#[test]
fn copy_enchantment_copying_nothing_stays_as_an_enchantment() {
    cr!("707.2");
    ruling!(
        "Copy Enchantment",
        "You can choose not to copy anything. In that case, Copy Enchantment simply enters the battlefield as an enchantment with an irrelevant ability."
    );
    supported("Copy Enchantment");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Glorious Anthem");
    let c = enter_copying(&mut t, P0, "Copy Enchantment", None);
    assert!(t.on_battlefield(c));
    assert_eq!(t.obj_now(c).chars.name, "Copy Enchantment");
    assert!(t.obj_now(c).is(CardType::Enchantment));
}

#[test]
fn sculpting_steel_copying_nothing_stays_as_an_artifact() {
    cr!("707.2");
    ruling!(
        "Sculpting Steel",
        "You can choose not to copy anything. In that case, Sculpting Steel stays on the battlefield as an artifact that doesn't do much of anything."
    );
    supported("Sculpting Steel");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sol Ring");
    let c = enter_copying(&mut t, P0, "Sculpting Steel", None);
    assert!(t.on_battlefield(c));
    assert_eq!(t.obj_now(c).chars.name, "Sculpting Steel");
    assert!(t.obj_now(c).is(CardType::Artifact));
}

#[test]
fn copy_artifact_with_no_artifact_is_an_enchantment_with_no_effect() {
    cr!("707.2", "707.9b");
    ruling!(
        "Copy Artifact",
        "The artifact to copy is chosen at the time this card enters. If there is no valid artifact to choose, then this card enters as an enchantment that has no effect."
    );
    supported("Copy Artifact");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Glorious Anthem");
    let c = enter_copying(&mut t, P0, "Copy Artifact", None);
    assert!(t.on_battlefield(c));
    let o = t.obj_now(c);
    assert_eq!(o.chars.name, "Copy Artifact");
    assert!(o.is(CardType::Enchantment) && !o.is(CardType::Artifact));
    // An artifact entering later isn't copied: the choice was made as it entered.
    let ring = t.battlefield(P1, "Sol Ring");
    t.settle();
    assert_eq!(t.obj_now(c).chars.name, "Copy Artifact");
    let _ = ring;
}

// ---------------------------------------------------------------------------
// What's copied
// ---------------------------------------------------------------------------

#[test]
fn clever_impersonator_copying_a_planeswalker_gets_its_printed_loyalty() {
    cr!("707.2", "306.5b");
    ruling!(
        "Clever Impersonator",
        "If Clever Impersonator enters the battlefield as a copy of a planeswalker, it will enter the battlefield with a number of loyalty counters on it equal to the loyalty printed in the lower right corner of the planeswalker card."
    );
    supported("Liliana of the Veil");
    let mut t = TestGame::new(2);
    let lili = t.battlefield(P1, "Liliana of the Veil");
    t.g.add_counters(Entity::Object(lili), counters::LOYALTY, 4, None);
    t.settle();
    let printed = t.counters(lili, counters::LOYALTY) - 4;
    let c = enter_copying(&mut t, P0, "Clever Impersonator", Some(lili));
    assert_eq!(t.obj_now(c).chars.name, "Liliana of the Veil");
    assert!(t.obj_now(c).is(CardType::Planeswalker));
    assert_eq!(t.counters(c, counters::LOYALTY), printed);
    assert_eq!(printed, 3);
}

#[test]
fn clever_impersonator_copying_your_legend_triggers_the_legend_rule() {
    cr!("704.5j", "707.2");
    ruling!(
        "Clever Impersonator",
        "Remember that if you control more than one legendary permanent with the same name, you'll choose one to remain on the battlefield and put the rest into their owner's graveyard."
    );
    supported("Isamaru, Hound of Konda");
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    // P0 keeps the copy.
    let c = {
        t.answer_choose(P0, &[Entity::Object(isamaru)]);
        let c = t.enter(P0, "Clever Impersonator");
        t.answer_choose(P0, &[Entity::Object(c)]);
        t.settle();
        t.g.current(c)
    };
    assert!(t.on_battlefield(c));
    assert!(!t.on_battlefield(isamaru));
    assert!(t.in_graveyard(P0, "Isamaru, Hound of Konda"));
    // An opponent's legend with the same name isn't affected.
    let theirs = t.battlefield(P1, "Isamaru, Hound of Konda");
    t.settle();
    assert!(t.on_battlefield(theirs) && t.on_battlefield(c));
}

/// `name` enters as a copy of the token `tok`, which was created by an effect and then
/// pumped by a non-copy effect.
fn copies_the_tokens_original_characteristics(
    name: &str,
    tok: &str,
    check: impl Fn(&TestGame, ObjectId),
) {
    supported(name);
    let mut t = TestGame::new(2);
    let token = create_token(&mut t, P1, tok);
    let c = enter_copying(&mut t, P0, name, Some(token));
    let o = t.obj_now(c);
    assert!(!o.is_token(), "{name} became a token");
    assert_eq!(o.chars.name, t.obj_now(token).chars.name);
    check(&t, c);
}

#[test]
fn clever_impersonator_copying_a_token_isnt_a_token() {
    cr!("707.2", "111.4");
    ruling!(
        "Clever Impersonator",
        "If the chosen permanent is a token, Clever Impersonator copies the original characteristics of that token as defined by the effect that put that token onto the battlefield. Clever Impersonator is not a token."
    );
    supported("Clever Impersonator");
    let mut t = TestGame::new(2);
    let token = create_token(&mut t, P1, "Soldier");
    modify_until_eot(
        &mut t,
        token,
        vec![Modification::ModifyPT(Value::c(3), Value::c(3))],
    );
    assert_eq!(t.pt(token), (4, 4));
    let c = enter_copying(&mut t, P0, "Clever Impersonator", Some(token));
    assert!(!t.obj_now(c).is_token());
    assert_eq!(t.obj_now(c).chars.name, t.obj_now(token).chars.name);
    assert_eq!(t.pt(c), (1, 1));
}

#[test]
fn sculpting_steel_copying_a_token_isnt_a_token() {
    cr!("707.2", "111.4");
    ruling!(
        "Sculpting Steel",
        "If the chosen artifact is a token, your Sculpting Steel copies the original characteristics of that token as stated by the effect that put it onto the battlefield. Your Sculpting Steel is not considered to be a token."
    );
    copies_the_tokens_original_characteristics("Sculpting Steel", "Treasure", |t, c| {
        assert!(t.obj_now(c).chars.has_subtype("Treasure"));
    });
}

#[test]
fn copy_enchantment_copying_a_token_isnt_a_token() {
    cr!("707.2", "111.4");
    ruling!(
        "Copy Enchantment",
        "If the chosen permanent is a token, Copy Enchantment copies the original characteristics of that token as stated by the effect that created the token. Copy Enchantment does not become a token."
    );
    copies_the_tokens_original_characteristics("Copy Enchantment", "Shard", |t, c| {
        assert!(t.obj_now(c).is(CardType::Enchantment));
    });
}

#[test]
fn mirrormade_copying_a_token_isnt_a_token() {
    cr!("707.2", "111.4");
    ruling!(
        "Mirrormade",
        "If the chosen permanent is a token, Mirrormade copies the original characteristics of that token as stated by the effect that created the token. Mirrormade doesn't become a token in this case."
    );
    copies_the_tokens_original_characteristics("Mirrormade", "Food", |t, c| {
        assert!(t.obj_now(c).chars.has_subtype("Food"));
    });
}

#[test]
fn estrids_invocation_copying_a_token_isnt_a_token() {
    cr!("707.2", "111.4", "707.9a");
    ruling!(
        "Estrid's Invocation",
        "If the chosen enchantment is a token, Estrid's Invocation copies the original characteristics of that token as stated by the effect that created the token. Estrid's Invocation doesn't become a token in this case."
    );
    supported("Estrid's Invocation");
    let mut t = TestGame::new(2);
    let token = create_token(&mut t, P0, "Shard");
    let c = enter_copying(&mut t, P0, "Estrid's Invocation", Some(token));
    assert!(!t.obj_now(c).is_token());
    assert_eq!(t.obj_now(c).chars.name, t.obj_now(token).chars.name);
    assert!(has_triggered_ability(&t, c));
}

/// `name` copies a permanent that's itself a copy (Sculpting Steel copying `original`):
/// it enters as `original`.
fn copies_what_the_copy_copied(name: &str, first: &str, original: &str) {
    supported(name);
    let mut t = TestGame::new(2);
    let orig = t.battlefield(P0, original);
    let copy = enter_copying(&mut t, P0, first, Some(orig));
    assert_eq!(t.obj_now(copy).chars.name, original);
    let c = enter_copying(&mut t, P0, name, Some(copy));
    assert_eq!(t.obj_now(c).chars.name, original, "{name}");
}

#[test]
fn sculpting_steel_copying_a_copy_copies_what_it_copied() {
    cr!("707.3");
    ruling!(
        "Sculpting Steel",
        "If the chosen artifact is copying something else (for example, if the chosen artifact is another Sculpting Steel), then your Sculpting Steel enters as whatever the chosen artifact copied."
    );
    copies_what_the_copy_copied("Sculpting Steel", "Sculpting Steel", "Sol Ring");
}

#[test]
fn clever_impersonator_copying_a_copy_copies_what_it_copied() {
    cr!("707.3");
    ruling!(
        "Clever Impersonator",
        "If the chosen permanent is a copy of something else (for example, if the chosen permanent is another Clever Impersonator), then your Clever Impersonator enters the battlefield as whatever the chosen permanent copied."
    );
    copies_what_the_copy_copied("Clever Impersonator", "Clever Impersonator", "Serra Angel");
}

#[test]
fn copy_enchantment_copying_a_copy_copies_what_it_copied() {
    cr!("707.3");
    ruling!(
        "Copy Enchantment",
        "If the chosen permanent is copying something else (for example, if the chosen permanent is another Copy Enchantment), then your Copy Enchantment enters the battlefield as whatever the chosen permanent copied."
    );
    copies_what_the_copy_copied("Copy Enchantment", "Copy Enchantment", "Glorious Anthem");
}

#[test]
fn mirrormade_copying_a_copy_copies_what_it_copied() {
    cr!("707.3");
    ruling!(
        "Mirrormade",
        "If the chosen permanent is copying something else (for example, if the chosen permanent is another Mirrormade), then Mirrormade enters the battlefield as whatever the chosen permanent copied."
    );
    copies_what_the_copy_copied("Mirrormade", "Mirrormade", "Sol Ring");
}

#[test]
fn estrids_invocation_copying_a_copy_copies_what_it_copied() {
    cr!("707.3", "707.9b");
    ruling!(
        "Estrid's Invocation",
        "If the chosen enchantment is copying something else (for example, if the chosen enchantment is another Estrid's Invocation), then Estrid's Invocation enters the battlefield as whatever the chosen enchantment copied."
    );
    copies_what_the_copy_copied(
        "Estrid's Invocation",
        "Estrid's Invocation",
        "Glorious Anthem",
    );
    // The first copy's exception is part of what it copied: the copy of the copy has
    // the upkeep ability too.
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let first = enter_copying(&mut t, P0, "Estrid's Invocation", Some(anthem));
    let second = enter_copying(&mut t, P0, "Estrid's Invocation", Some(first));
    assert!(has_triggered_ability(&t, second));
}

#[test]
fn clever_impersonator_and_mirrormade_copy_x_as_zero() {
    cr!("707.2", "202.3e", "107.3i");
    ruling!(
        "Clever Impersonator",
        "If the chosen permanent has {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Mirrormade",
        "If the chosen permanent has {X} in its mana cost, X is considered to be 0."
    );
    supported("Chalice of the Void");
    for name in ["Clever Impersonator", "Mirrormade"] {
        let mut t = TestGame::new(2);
        let chalice = t.battlefield(P1, "Chalice of the Void");
        t.g.add_counters(Entity::Object(chalice), counters::CHARGE, 2, None);
        let c = enter_copying(&mut t, P0, name, Some(chalice));
        assert_eq!(t.obj_now(c).chars.name, "Chalice of the Void", "{name}");
        assert_eq!(mv(&mut t, c), 0, "{name}");
        assert_eq!(t.counters(c, counters::CHARGE), 0, "{name}");
    }
}

#[test]
fn estrids_invocation_copies_x_as_zero() {
    cr!("707.2", "202.3e", "107.3i");
    ruling!(
        "Estrid's Invocation",
        "If the copied enchantment has {X} in its mana cost, X is considered to be 0."
    );
    supported("Mana Bloom");
    let mut t = TestGame::new(2);
    let bloom = t.battlefield(P0, "Mana Bloom");
    t.g.add_counters(Entity::Object(bloom), counters::CHARGE, 3, None);
    let c = enter_copying(&mut t, P0, "Estrid's Invocation", Some(bloom));
    assert_eq!(t.obj_now(c).chars.name, "Mana Bloom");
    assert_eq!(mv(&mut t, c), 1);
    assert_eq!(t.counters(c, counters::CHARGE), 0);
}

/// `name` copies `what` after `what` was tapped, given counters, and changed by non-copy
/// effects: it gets exactly what's printed.
fn copies_only_the_printed_values(name: &str, what: &str) -> (TestGame, ObjectId, ObjectId) {
    supported(name);
    let mut t = TestGame::new(2);
    let orig = t.battlefield(P1, what);
    dress_up(&mut t, orig);
    let c = enter_copying(&mut t, P0, name, Some(orig));
    assert_eq!(t.obj_now(c).chars.name, what);
    assert!(fresh(&t, c), "{name}");
    let colors = t.obj_now(c).chars.colors;
    assert!(
        !colors.contains(Color::Green) || card_is_green(what),
        "{name}"
    );
    (t, orig, c)
}

fn card_is_green(name: &str) -> bool {
    card(name).front().chars.colors.contains(Color::Green)
}

#[test]
fn mirrormade_copies_only_whats_printed() {
    cr!("707.2", "613.2");
    ruling!(
        "Mirrormade",
        "Mirrormade copies exactly what was printed on the original permanent (unless that permanent is copying something else or is a token; see below). It doesn't copy whether that permanent is tapped or untapped, whether it has any counters on it or any Auras attached to it, or any non-copy effects that have changed its types, color, or so on."
    );
    copies_only_the_printed_values("Mirrormade", "Sol Ring");
    // An artifact made a creature by Bring to Life: the copy isn't a creature.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    animate(&mut t, ring, 4);
    assert!(t.obj_now(ring).is(CardType::Creature));
    let c = enter_copying(&mut t, P0, "Mirrormade", Some(ring));
    assert!(!t.obj_now(c).is(CardType::Creature));
    assert_eq!(t.counters(c, counters::PLUS1), 0);
}

#[test]
fn sculpting_steel_copies_only_whats_printed() {
    cr!("707.2", "613.2");
    ruling!(
        "Sculpting Steel",
        "Sculpting Steel doesn't copy whether the original artifact is tapped or untapped. It also doesn't copy any counters on that artifact, any Auras or Equipment attached to that artifact, or any effects that are currently affecting that artifact"
    );
    copies_only_the_printed_values("Sculpting Steel", "Sol Ring");
    // An animated artifact: the copy is a normal, nonanimated one.
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P0, "Sol Ring");
    animate(&mut t, ring, 4);
    let c = enter_copying(&mut t, P0, "Sculpting Steel", Some(ring));
    assert!(!t.obj_now(c).is(CardType::Creature));
}

#[test]
fn copy_artifact_is_an_artifact_enchantment_of_the_copied_color() {
    cr!("707.9b", "707.2");
    ruling!(
        "Copy Artifact",
        "The copy is both an artifact and an enchantment, so it is an artifact-enchantment"
    );
    ruling!(
        "Copy Artifact",
        "The copy of the artifact is not still blue. It copies the color of the thing it is copying."
    );
    supported("Copy Artifact");
    let mut t = TestGame::new(2);
    let ring = t.battlefield(P1, "Sol Ring");
    let c = enter_copying(&mut t, P0, "Copy Artifact", Some(ring));
    let o = t.obj_now(c);
    assert_eq!(o.chars.name, "Sol Ring");
    assert!(o.is(CardType::Artifact) && o.is(CardType::Enchantment));
    assert!(o.chars.colors.is_colorless());
    // It's affected by what affects either type: an enchantment-destroying spell
    // destroys it.
    t.lands(P1, "Plains", 1);
    let disenchant = t.hand(P1, "Erase");
    t.cast(P1, disenchant).target(c).go();
    t.resolve_all();
    assert!(!t.on_battlefield(c));
}

#[test]
fn masterwork_of_ingenuity_enters_unattached() {
    cr!("707.2", "301.5e");
    ruling!(
        "Masterwork of Ingenuity",
        "Masterwork of Ingenuity enters the battlefield unattached. It doesn't enter attached to the same creature as the Equipment it copies."
    );
    supported("Masterwork of Ingenuity");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = t.battlefield(P0, "Bonesplitter");
    assert!(t.g.attach(splitter, Entity::Object(bears)));
    t.g.recompute();
    let c = enter_copying(&mut t, P0, "Masterwork of Ingenuity", Some(splitter));
    assert_eq!(t.obj_now(c).chars.name, "Bonesplitter");
    assert_eq!(t.obj_now(c).attached_to, None);
    assert_eq!(t.pt(bears), (4, 2));
}

// ---------------------------------------------------------------------------
// Copying an Aura
// ---------------------------------------------------------------------------

/// `name` (cast as a spell by P0) copies an Aura: P0 chooses what it enchants as it
/// enters, without targeting (an opponent's hexproof creature can be chosen) but only
/// among what it could legally enchant (not a creature with protection from white).
fn aura_copy_chooses_a_legal_recipient(name: &str) {
    supported(name);
    supported("Pacifism");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pacifism = t.battlefield(P0, "Pacifism");
    assert!(t.g.attach(pacifism, Entity::Object(bears)));
    let scout = t.battlefield(P1, "Gladecover Scout");
    let knight = t.battlefield(P1, "Black Knight");
    t.g.recompute();
    let from = t.asked().len();
    t.answer_choose(P0, &[Entity::Object(pacifism)]);
    t.answer_choose(P0, &[Entity::Object(scout)]);
    let c = t.enter(P0, name);
    t.settle();
    let c = t.g.current(c);
    let offered = prompted_candidates(&t, from, "enchant");
    assert_eq!(offered.len(), 1, "{name}: {offered:?}");
    assert!(offered[0].contains(&Entity::Object(scout)), "{name}");
    assert!(offered[0].contains(&Entity::Object(bears)), "{name}");
    assert!(!offered[0].contains(&Entity::Object(knight)), "{name}");
    assert_eq!(t.obj_now(c).chars.name, "Pacifism");
    assert_eq!(t.obj_now(c).attached_to, Some(Entity::Object(scout)));
    assert_eq!(t.obj_now(c).controller, P0);
}

/// `name`, cast as a spell, copies an Aura that has nothing it could legally enchant:
/// it's put into its owner's graveyard from the stack; put onto the battlefield from
/// another zone, it stays there. P0's Pacifism, made colorless by a non-copy effect, is
/// attached to Black Knight (protection from white), the only creature: the copy is
/// white and can't enchant it.
fn aura_copy_with_nothing_to_enchant(name: &str) {
    supported("Black Knight");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P1, "Black Knight");
    let pacifism = t.battlefield(P0, "Pacifism");
    modify_no_settle(
        &mut t,
        pacifism,
        vec![Modification::SetColors(ColorSet::NONE)],
    );
    assert!(t.g.attach(pacifism, Entity::Object(knight)));
    t.g.recompute();
    t.settle();
    assert!(t.on_battlefield(pacifism));
    assert_eq!(
        t.obj_now(pacifism).attached_to,
        Some(Entity::Object(knight))
    );
    let spell = t.hand(P0, name);
    t.lands(P0, "Island", 3);
    t.answer_choose(P0, &[Entity::Object(pacifism)]);
    t.cast(P0, spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, name), "{name}");
    assert_eq!(t.g.current(pacifism), pacifism);
    // From the hand, put onto the battlefield by an effect: it stays in the hand.
    let in_hand = t.hand(P0, name);
    t.answer_choose(P0, &[Entity::Object(pacifism)]);
    let r = t.g.move_object_ev(mtg_engine::replacement::MoveEv {
        obj: in_hand,
        to: Zone::Battlefield,
        pos: LibraryPosition::Top,
        cause: mtg_engine::events::MoveCause::Effect,
        by: Some(P0),
        etb: mtg_engine::replacement::EtbInfo {
            controller: Some(P0),
            ..Default::default()
        },
        source: None,
    });
    t.settle();
    assert!(r.is_none(), "{name}");
    assert_eq!(t.zone(in_hand), Zone::Hand(P0), "{name}");
}

#[test]
fn copy_enchantment_copying_an_aura_chooses_what_it_enchants() {
    cr!("303.4f", "303.4g", "707.2", "702.16b");
    ruling!(
        "Copy Enchantment",
        "If Copy Enchantment copies an Aura this way, you choose what the Aura will enchant just before it enters the battlefield."
    );
    aura_copy_chooses_a_legal_recipient("Copy Enchantment");
    aura_copy_with_nothing_to_enchant("Copy Enchantment");
}

#[test]
fn mirrormade_copying_an_aura_chooses_what_it_enchants() {
    cr!("303.4f", "303.4g", "707.2", "702.16b");
    ruling!(
        "Mirrormade",
        "If Mirrormade copies an Aura this way, you choose what the Aura will enchant just before it enters the battlefield."
    );
    aura_copy_chooses_a_legal_recipient("Mirrormade");
    aura_copy_with_nothing_to_enchant("Mirrormade");
}

#[test]
fn estrids_invocation_copying_an_aura_chooses_what_it_enchants() {
    cr!("303.4f", "303.4g", "707.2", "702.16b");
    ruling!(
        "Estrid's Invocation",
        "If the chosen enchantment is an Aura, you choose what it enchants just before Estrid's Invocation enters the battlefield."
    );
    ruling!(
        "Estrid's Invocation",
        "If the chosen enchantment is an Aura but Estrid's Invocation won't be able to legally enchant anything, Estrid's Invocation remains in its current zone and doesn't enter the battlefield."
    );
    aura_copy_chooses_a_legal_recipient("Estrid's Invocation");
    aura_copy_with_nothing_to_enchant("Estrid's Invocation");
}

// ---------------------------------------------------------------------------
// Estrid's Invocation's upkeep flicker
// ---------------------------------------------------------------------------

#[test]
fn estrids_invocation_returns_as_a_new_object_and_copies_anew() {
    cr!("400.7", "707.9a", "603.2");
    ruling!(
        "Estrid's Invocation",
        "Once Estrid's Invocation returns, it's considered a new object with no relation to the object that it was. You must choose an enchantment that's currently on the battlefield to copy (or to not copy anything). Auras that were attached to it will be put into their owners' graveyards. Any counters that were on it cease to exist."
    );
    supported("Confiscate");
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let moon = t.battlefield(P0, "Bad Moon");
    let inv = enter_copying(&mut t, P0, "Estrid's Invocation", Some(anthem));
    assert_eq!(t.obj_now(inv).chars.name, "Glorious Anthem");
    t.g.add_counters(Entity::Object(inv), counters::CHARGE, 2, None);
    let confiscate = t.battlefield(P0, "Confiscate");
    assert!(t.g.attach(confiscate, Entity::Object(inv)));
    t.g.recompute();
    t.settle();
    // P0's next upkeep: exile it and return it, copying Bad Moon this time.
    t.advance_to(P1, Step::PrecombatMain);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(moon)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    let now = t.g.current(inv);
    assert_ne!(now, inv);
    assert!(t.on_battlefield(now), "{}", t.dump_log());
    assert_eq!(t.obj_now(now).chars.name, "Bad Moon");
    assert_eq!(t.counters(now, counters::CHARGE), 0);
    assert!(t.in_graveyard(P0, "Confiscate"));
    assert!(has_triggered_ability(&t, now));
}
