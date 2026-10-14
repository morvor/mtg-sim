//! Rulings batch P030 review: behavior of every card the batch's compiler patterns made
//! fully supported, beyond the batch's own rulings (Essence of the Wild and tokens,
//! "teammates" across game variants, Sphinx of False Conclusions, Venser, Fervent Forger,
//! Flamehold Grappler, Kalamax, the Stormsire).

use crate::r_p030_common::*;
use crate::r_s01_common::supported;
use crate::r_s06_common::activate_containing;
use crate::r_s25_common::{cast_new, change_copy_targets, keep_copy_targets};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// --- Essence of the Wild ------------------------------------------------------------------

#[test]
fn essence_of_the_wild_affects_tokens_but_not_other_players_creatures_or_noncreatures() {
    cr!("614.1c", "707.2", "111.1");
    supported("Essence of the Wild");
    // "Creatures you control enter as a copy of this creature."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Essence of the Wild");
    // Creature tokens are creatures you control too.
    cast_new(&mut t, P0, "Raise the Alarm", &[]);
    t.resolve_all();
    let toks: Vec<ObjectId> = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.is_token())
        .map(|o| o.id)
        .collect();
    assert_eq!(toks.len(), 2);
    for tok in toks {
        assert_eq!(t.obj_now(tok).chars.name, "Essence of the Wild");
        assert_eq!(t.pt(tok), (6, 6));
    }
    // An opponent's creature and a noncreature permanent of yours are unaffected.
    let bears = t.enter(P1, "Grizzly Bears");
    t.settle();
    assert_eq!(t.obj_now(bears).chars.name, "Grizzly Bears");
    let ring = t.enter(P0, "Sol Ring");
    t.settle();
    assert_eq!(t.obj_now(ring).chars.name, "Sol Ring");
}

#[test]
fn creatures_your_opponents_control_enter_tapped_includes_tokens() {
    cr!("614.1c", "111.1");
    supported("Urabrask the Hidden");
    // "Creatures your opponents control enter tapped."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Urabrask the Hidden");
    cast_new(&mut t, P0, "Raise the Alarm", &[]);
    t.resolve_all();
    let toks: Vec<_> = t
        .g
        .permanents()
        .filter(|o| o.controller == P0 && o.is_token())
        .map(|o| o.tapped)
        .collect();
    assert_eq!(toks, vec![true, true]);
}

// --- Imperial Mask: "each of your teammates" -------------------------------------------------

/// P0's Imperial Mask ("When this enchantment enters, if it's not a token, each of your
/// teammates creates a token that's a copy of this enchantment.") enters and resolves.
fn mask(t: &mut TestGame, p: PlayerId) {
    supported("Imperial Mask");
    t.enter(p, "Imperial Mask");
    t.resolve_all();
}

fn mask_tokens(t: &TestGame) -> Vec<PlayerId> {
    let mut v: Vec<PlayerId> = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.chars.name == "Imperial Mask")
        .map(|o| o.controller)
        .collect();
    v.sort();
    v
}

#[test]
fn imperial_mask_makes_no_tokens_without_teammates() {
    cr!("102.3", "806.1");
    // Two-player and free-for-all games: every other player is an opponent.
    let mut t = TestGame::new(2);
    mask(&mut t, P0);
    assert!(mask_tokens(&t).is_empty());
    let mut t = TestGame::with_config(
        4,
        GameConfig {
            variant: Variant::FreeForAll,
            ..Default::default()
        },
    );
    mask(&mut t, P0);
    assert!(mask_tokens(&t).is_empty());
}

#[test]
fn imperial_mask_gives_each_teammate_a_token_in_team_vs_team() {
    cr!("102.3", "808.1", "111.2");
    // Two teams of three: P0, P1, P2 against P3, P4, P5.
    let mut t = TestGame::with_config(
        6,
        GameConfig {
            variant: Variant::TeamVsTeam,
            teams: Some(vec![0, 0, 0, 1, 1, 1]),
            ..Default::default()
        },
    );
    mask(&mut t, P0);
    // The tokens' own triggers do nothing: they're tokens.
    assert_eq!(mask_tokens(&t), vec![P1, P2]);
}

#[test]
fn imperial_mask_gives_tokens_only_to_teammates_within_range_of_influence() {
    cr!("102.3", "809.1", "801.10");
    ruling!(
        "Imperial Mask",
        "Only teammates within the range of influence of Imperial Mask's controller will get a token. Imperial Mask's controller doesn't get a token"
    );
    // Emperor: P1 is team 0's emperor (range 2), P0 and P2 its generals (range 1).
    let emperor = || {
        TestGame::with_config(
            6,
            GameConfig {
                variant: Variant::Emperor,
                teams: Some(vec![0, 0, 0, 1, 1, 1]),
                ..Default::default()
            },
        )
    };
    let mut t = emperor();
    assert_eq!(t.g.range_of_influence(P0), Some(1));
    mask(&mut t, P0);
    // P2 sits two seats from P0, outside P0's range.
    assert_eq!(mask_tokens(&t), vec![P1]);
    let mut t = emperor();
    mask(&mut t, P1);
    assert_eq!(mask_tokens(&t), vec![P0, P2]);
}

// --- "if it isn't a token" ------------------------------------------------------------------

#[test]
fn sphinx_of_false_conclusions_returns_once_as_a_token() {
    cr!("603.4", "603.10a", "707.2");
    supported("Sphinx of False Conclusions");
    // "When this creature dies, if it isn't a token, create a token that's a copy of it."
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sphinx of False Conclusions");
    kill(&mut t, s);
    t.resolve_all();
    let toks = tokens_named(&t, P0, "Sphinx of False Conclusions");
    assert_eq!(toks.len(), 1);
    assert_eq!(t.pt(toks[0]), (4, 2));
    kill(&mut t, toks[0]);
    assert_eq!(t.stack_len(), 0, "the token's dies trigger doesn't trigger");
    assert!(tokens_named(&t, P0, "Sphinx of False Conclusions").is_empty());
}

// --- Venser, Fervent Forger ------------------------------------------------------------------

#[test]
fn venser_copies_an_opponent_s_spell_twice() {
    cr!("707.10", "707.10c", "115.1");
    supported("Venser, Fervent Forger");
    // "When Venser enters, choose one — • Copy target instant or sorcery spell an opponent
    // controls twice. You may choose new targets for the copies. • ..."
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P0, &[obj(bolt)]);
    t.enter(P0, "Venser, Fervent Forger");
    change_copy_targets(&mut t, P0, &[Some(Entity::Player(P1))]);
    change_copy_targets(&mut t, P0, &[Some(Entity::Player(P1))]);
    let copies = resolve_until_copies(&mut t);
    assert_eq!(copies.len(), 2);
    assert!(copies.iter().all(|c| t.obj(*c).controller == P0));
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    assert_eq!(t.life(P0), 17);

    // P0's own spell isn't a legal target: the trigger is removed with no target.
    let mut t = TestGame::new(2);
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P0, &[obj(bolt)]);
    t.enter(P0, "Venser, Fervent Forger");
    keep_copy_targets(&mut t, P0);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

// --- Flamehold Grappler ---------------------------------------------------------------------

#[test]
fn flamehold_grappler_copies_a_permanent_spell_and_lets_you_retarget() {
    cr!("707.10", "707.10c", "111.1", "603.7b");
    supported("Flamehold Grappler");
    // "When this creature enters, copy the next spell you cast this turn when you cast it.
    // You may choose new targets for the copy. (A copy of a permanent spell becomes a
    // token.)"
    let mut t = TestGame::new(2);
    t.enter(P0, "Flamehold Grappler");
    t.resolve_all();
    cast_new(&mut t, P0, "Grizzly Bears", &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    assert_eq!(tokens_named(&t, P0, "Grizzly Bears").len(), 1);

    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.enter(P0, "Flamehold Grappler");
    t.resolve_all();
    cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    change_copy_targets(&mut t, P0, &[Some(obj(giant))]);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    assert_eq!(t.life(P1), 17);
}

// --- Kalamax, the Stormsire -------------------------------------------------------------------

#[test]
fn kalamax_counts_each_instant_copy_you_make() {
    cr!("707.10", "603.2");
    ruling!(
        "Kalamax, the Stormsire",
        "If an effect copies an instant spell multiple times, Kalamax's last ability triggers that many times."
    );
    supported("Kalamax, the Stormsire");
    supported("Echo Mage");
    // "Whenever you copy an instant spell, put a +1/+1 counter on Kalamax." (Kalamax is
    // untapped, so its first ability doesn't copy anything.)
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kalamax, the Stormsire");
    let em = t.battlefield(P0, "Echo Mage");
    t.g.objects[em.0 as usize]
        .counters
        .insert(counters::LEVEL.into(), 4);
    t.g.dirty = true;
    t.lands(P0, "Island", 2);
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    t.answer_targets(P0, &[obj(bolt)]);
    activate_containing(&mut t, P0, em, "twice").unwrap();
    keep_copy_targets(&mut t, P0);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(t.life(P1), 11);
    assert_eq!(t.counters(k, counters::PLUS1), 2);
}

#[test]
fn kalamax_ignores_sorcery_copies_and_opponents_copies() {
    cr!("707.10", "603.2");
    supported("Wild Ricochet");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kalamax, the Stormsire");
    // P0 copies a sorcery (Wild Ricochet on its own Divination).
    let div = cast_new(&mut t, P0, "Divination", &[]);
    cast_new(&mut t, P0, "Wild Ricochet", &[obj(div)]);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.counters(k, counters::PLUS1), 0);
    // P1 copies P0's instant.
    let bolt = cast_new(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    cast_new(&mut t, P1, "Wild Ricochet", &[obj(bolt)]);
    t.answer_yes(P1, false);
    change_copy_targets(&mut t, P1, &[Some(Entity::Player(P0))]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.counters(k, counters::PLUS1), 0);
}
