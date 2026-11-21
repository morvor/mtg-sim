//! CR 710: flip cards.

use super::r709_common::*;
use mtg_engine::ability::*;
use mtg_engine::card::{card, Layout};
use mtg_engine::events::MoveCause;
use mtg_engine::flip;
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const LAVARUNNER: &str = "Akki Lavarunner // Tok-Tok, Volcano Born";

/// Akki Lavarunner ({3}{R}, 1/1 haste): "Whenever this creature deals damage to an
/// opponent, flip it." Attacks P1 and flips.
fn attack_and_flip(t: &mut TestGame, akki: ObjectId) {
    t.set_step(P0, Step::PrecombatMain);
    t.attack(&[(akki, Entity::Player(P1))], &[]);
    t.resolve_all();
}

#[test]
fn a_flip_card_has_normal_and_alternative_characteristics() {
    cr!("710.1", "710.1a", "710.1b", "710.2");
    supported(LAVARUNNER);
    let def = card(LAVARUNNER);
    assert_eq!(def.layout, Layout::Flip);
    // It's one card with a normal Magic card back, not a double-faced card.
    assert!(!def.layout.is_double_faced());
    let mut t = TestGame::new(2);
    // In the hand, only the normal characteristics.
    let in_hand = t.hand(P0, LAVARUNNER);
    t.g.recompute();
    assert_eq!(t.obj(in_hand).chars.name, "Akki Lavarunner");
    assert!(!t.obj(in_hand).chars.is_legendary());
    // On the battlefield before it flips, too.
    let akki = t.battlefield(P0, LAVARUNNER);
    let c = &t.obj(akki).chars;
    assert_eq!(c.name, "Akki Lavarunner");
    assert_eq!(t.pt(akki), (1, 1));
    assert!(c.has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    assert!(c.has_subtype("Goblin") && c.has_subtype("Warrior"));
    // Its ability flips it: now it has the alternative name, text, type line, power and
    // toughness of Tok-Tok, Volcano Born (a legendary 2/2 Goblin Shaman with protection
    // from red), and none of the normal ones.
    attack_and_flip(&mut t, akki);
    assert_eq!(t.life(P1), 19);
    assert!(t.g.is_live(akki), "flipping doesn't make a new object");
    let o = t.obj(akki);
    assert!(o.flipped);
    assert_eq!(o.chars.name, "Tok-Tok, Volcano Born");
    assert!(o.chars.is_legendary());
    assert!(o.chars.has_subtype("Shaman") && !o.chars.has_subtype("Warrior"));
    assert!(!o
        .chars
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    assert!(o
        .chars
        .has_keyword(mtg_engine::keywords::KeywordKind::Protection));
    assert_eq!(t.pt(akki), (2, 2));
}

#[test]
fn flipping_keeps_color_mana_cost_and_effects() {
    cr!("710.1c");
    let mut t = TestGame::new(2);
    let akki = t.battlefield(P0, LAVARUNNER);
    // A +1/+1 counter and "gets +2/+0 until end of turn" from before it flips.
    t.g.add_counters(Entity::Object(akki), counters::PLUS1, 1, None);
    run_effect(
        &mut t,
        P0,
        None,
        &[Entity::Object(akki)],
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::ModifyPT(Value::c(2), Value::c(0))],
            duration: Duration::EndOfTurn,
        },
    );
    assert_eq!(t.pt(akki), (4, 2));
    attack_and_flip(&mut t, akki);
    let o = t.obj(akki);
    assert!(o.flipped);
    // Still red with mana cost {3}{R} (the bottom half has no mana cost of its own).
    assert_eq!(o.chars.colors, ColorSet::single(Color::Red));
    assert_eq!(mv(&mut t, akki), 4);
    // Tok-Tok is 2/2; the counter and the pump still apply.
    assert_eq!(t.pt(akki), (5, 3));
}

#[test]
fn flipping_is_one_way_until_it_leaves_the_battlefield() {
    cr!("710.4");
    let mut t = TestGame::new(2);
    let akki = t.battlefield(P0, LAVARUNNER);
    attack_and_flip(&mut t, akki);
    assert!(t.obj(akki).flipped);
    // It can't flip again or become unflipped.
    assert!(!flip::flip(&mut t.g, akki));
    t.g.recompute();
    assert_eq!(t.obj(akki).chars.name, "Tok-Tok, Volcano Born");
    // Once it leaves the battlefield it retains no memory of its status.
    let hand =
        t.g.move_object(akki, Zone::Hand(P0), MoveCause::Effect, None)
            .unwrap();
    t.g.recompute();
    assert!(!t.obj(hand).flipped);
    assert_eq!(t.obj(hand).chars.name, "Akki Lavarunner");
    let back =
        t.g.move_object(hand, Zone::Battlefield, MoveCause::Effect, None)
            .unwrap();
    t.g.recompute();
    assert!(!t.obj(back).flipped);
    assert_eq!(t.obj(back).chars.name, "Akki Lavarunner");
    assert_eq!(t.pt(back), (1, 1));
}

#[test]
fn an_optional_flip() {
    cr!("710.1a");
    // Budoka Pupil: "At the beginning of the end step, if there are two or more ki
    // counters on this creature, you may flip it."
    supported("Budoka Pupil // Ichiga, Who Topples Oaks");
    let mut t = TestGame::new(2);
    let pupil = t.battlefield(P0, "Budoka Pupil // Ichiga, Who Topples Oaks");
    t.g.add_counters(Entity::Object(pupil), "ki", 2, None);
    t.set_step(P0, Step::PostcombatMain);
    t.answer_yes(P0, true);
    t.advance_to_step(Step::End);
    t.resolve_all();
    assert!(t.obj(pupil).flipped);
    assert_eq!(t.obj(pupil).chars.name, "Ichiga, Who Topples Oaks");
    assert_eq!(t.counters(pupil, "ki"), 2);
}

#[test]
fn a_flip_cards_alternative_name_may_be_chosen() {
    cr!("710.5");
    // Jushi Apprentice flips into Tomoya the Revealer ("{3}{U}{U}, {T}: Target player
    // draws X cards, where X is the number of cards in your hand."). Pithing Needle may
    // name Tomoya: its activated abilities can't be activated.
    let mut t = TestGame::new(2);
    let jushi = t.battlefield(P0, "Jushi Apprentice // Tomoya the Revealer");
    flip::flip(&mut t.g, jushi);
    t.g.recompute();
    assert_eq!(t.obj(jushi).chars.name, "Tomoya the Revealer");
    name_card(&mut t, P1, "Tomoya the Revealer");
    let needle = t.enter(P1, "Pithing Needle");
    assert_eq!(
        t.obj_now(needle).choices.card_name.as_deref(),
        Some("Tomoya the Revealer")
    );
    t.set_step(P0, Step::PrecombatMain);
    add_mana(&mut t, P0, ManaType::U, 5);
    assert!(t.activate(P0, jushi, 0, &[Entity::Player(P0)]).is_err());
    // An unflipped Jushi Apprentice doesn't have that name.
    let other = t.battlefield(P0, "Jushi Apprentice // Tomoya the Revealer");
    assert!(t.activate(P0, other, 0, &[]).is_ok());
}
