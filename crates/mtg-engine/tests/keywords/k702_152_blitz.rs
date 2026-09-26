//! CR 702.152 Blitz.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_052_066::destroy;
use crate::common_k702_140_152::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const BLITZ: CastMethod = CastMethod::Keyword(KeywordKind::Blitz);

#[test]
fn a_blitzed_creature_has_haste_and_is_sacrificed_at_the_next_end_step_drawing_a_card() {
    cr!("702.152", "702.152a");
    assert_supported("Riveteers Requisitioner");
    let mut t = TestGame::new(2);
    // Riveteers Requisitioner: {1}{R} 3/1, "When this creature dies, create a Treasure
    // token." Blitz {2}{R}.
    t.lands(P0, "Mountain", 3);
    let c = t.hand(P0, "Riveteers Requisitioner");
    let spell = t.cast(P0, c).method(BLITZ).go();
    // The blitz cost was paid rather than the mana cost; its mana value is still 2.
    assert_eq!(t.g.mana_value_of(spell), 2);
    t.resolve_all();
    let r = t.g.current(c);
    assert!(t.on_battlefield(c));
    assert!(t.obj(r).summoning_sick);
    assert!(t.obj(r).chars.has_keyword(KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack_now(&mut t, r));
    t.attack(&[(r, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 17);
    // At the beginning of the end step it's sacrificed: it dies, so both its own ability
    // (a Treasure) and the blitz ability (draw a card) trigger.
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Riveteers Requisitioner"));
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.named_on_battlefield("Treasure Token").len(), 1);
}

#[test]
fn cast_normally_it_has_no_haste_and_isnt_sacrificed() {
    cr!("702.152a");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    let c = t.hand(P0, "Riveteers Requisitioner");
    t.cast(P0, c).go();
    t.resolve_all();
    let r = t.g.current(c);
    assert!(!t.obj(r).chars.has_keyword(KeywordKind::Haste));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack_now(&mut t, r));
    let hand = t.hand_size(P0);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(c));
    // Dying later doesn't draw a card.
    destroy(&mut t, c);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn blitzing_is_casting_the_spell_with_its_normal_timing() {
    cr!("702.152a");
    ruling!(
        "Mezzio Mugger",
        "If you choose to pay the blitz cost rather than the mana cost, you're still casting the spell. It goes on the stack and can be responded to and countered."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.lands(P1, "Island", 2);
    let c = t.hand(P0, "Plasma Jockey");
    // Not at instant speed.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!castable(&mut t, P0, c, BLITZ));
    t.set_step(P0, Step::PrecombatMain);
    assert!(castable(&mut t, P0, c, BLITZ));
    let spell = t.cast(P0, c).method(BLITZ).go();
    let counter = t.hand(P1, "Counterspell");
    t.cast(P1, counter).target(spell).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Plasma Jockey"));
    // Countered: nothing is sacrificed and no card is drawn.
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn the_draw_ability_triggers_whenever_it_dies() {
    cr!("702.152a");
    ruling!(
        "Mezzio Mugger",
        "The triggered ability that lets its controller draw a card triggers when it dies for any reason, not just when you sacrifice it during the end step."
    );
    ruling!(
        "Mezzio Mugger",
        "If you pay the blitz cost to cast a creature spell, that permanent will be sacrificed only if it's still on the battlefield when that triggered ability resolves."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    let c = t.hand(P0, "Mezzio Mugger");
    t.cast(P0, c).method(BLITZ).go();
    t.resolve_all();
    let hand = t.hand_size(P0);
    destroy(&mut t, c);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mezzio Mugger"));
    assert_eq!(t.hand_size(P0), hand + 1);
    // The delayed triggered ability does nothing to the card in the graveyard.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Mezzio Mugger"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn a_copy_of_a_blitzed_creature_isnt_blitzed() {
    cr!("702.152a");
    ruling!(
        "Star Athlete",
        "If a creature enters as a copy of or becomes a copy of a creature whose blitz cost was paid, the copy won't have haste, won't be sacrificed, and its controller won't draw a card when it dies."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 4);
    let c = t.hand(P0, "Star Athlete");
    t.cast(P0, c).method(BLITZ).go();
    t.resolve_all();
    let athlete = t.g.current(c);
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.cast(P0, clone).go();
    t.answer_choose(P0, &[Entity::Object(athlete)]);
    t.resolve_all();
    let copy = t.g.current(clone);
    assert_eq!(t.obj(copy).chars.name, "Star Athlete");
    assert!(!t.obj(copy).chars.has_keyword(KeywordKind::Haste));
    assert!(t.obj(athlete).chars.has_keyword(KeywordKind::Haste));
    // Only the original is sacrificed, and only it draws a card.
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Star Athlete"));
    assert!(t.on_battlefield(clone));
    assert_eq!(t.hand_size(P0), hand + 1);
    destroy(&mut t, clone);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn a_blitz_cost_can_include_other_costs() {
    cr!("702.152a");
    // Tenacious Underdog: Blitz—{2}{B}{B}, Pay 2 life.
    let mut t = TestGame::new(2);
    let c = t.hand(P0, "Tenacious Underdog");
    add_mana(&mut t, P0, ManaType::B, 2);
    add_mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, c).method(BLITZ).go();
    assert_eq!(t.life(P0), 18);
    assert_eq!(pool(&t, P0), 0);
    t.resolve_all();
    assert!(has_kw(&t, c, KeywordKind::Haste));
}

#[test]
fn only_one_of_several_blitz_instances_is_used_and_only_it_applies() {
    cr!("702.152b");
    let def = custom_card(
        "Twice-Blitzed Brawler",
        "Creature — Human Warrior",
        Some((2, 2)),
        "Blitz {3}{R}\nBlitz {R}",
    );
    let mut t = TestGame::new(2);
    let c = t.custom(P0, def, Zone::Hand(P0));
    // Two ways to cast it for a blitz cost; the player chooses the second one ({R}).
    add_mana(&mut t, P0, ManaType::R, 1);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.cast(P0, c).method(BLITZ).go();
    assert_eq!(pool(&t, P0), 0);
    t.resolve_all();
    let b = t.g.current(c);
    // Only the instance that was paid for applies: one haste, one draw ability, and one
    // sacrifice trigger.
    assert_eq!(t.obj(b).chars.keyword_count(KeywordKind::Haste), 1);
    let draws = t
        .obj(b)
        .chars
        .abilities
        .iter()
        .filter(|a| a.text.contains("draw a card"))
        .count();
    assert_eq!(draws, 1);
    let hand = t.hand_size(P0);
    t.advance_to(P0, Step::End);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.zone(c), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), hand + 1);
}
