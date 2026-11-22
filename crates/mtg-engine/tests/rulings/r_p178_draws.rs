//! Rulings batch P178 — draw and search triggers and replacements: Alms Collector (with
//! dredge, Thought Reflection, two Collectors), Consecrated Sphinx (one trigger per card,
//! two Sphinxes), Krang's extra triggers, Archivist of Oghma, Cephalid Sage.

use crate::r_p178_common::*;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` draws `n` cards (one draw instruction); triggers are put on the stack.
fn draw(t: &mut TestGame, p: PlayerId, n: u32) {
    t.g.draw_cards(p, n);
    t.g.flush_events();
    t.settle();
}

#[test]
fn alms_collector_applies_before_dredge_replaces_single_draws() {
    cr!("616.1", "702.52a", "121.6");
    ruling!(
        "Alms Collector",
        "Alms Collector’s replacement effect applies to an instruction to draw more than one card before any replacement effects apply to individual cards drawn."
    );
    supported("Alms Collector");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Alms Collector");
    let imp = t.graveyard(P1, "Stinkweed Imp");
    t.set_step(P1, Step::PrecombatMain);
    for _ in 0..4 {
        t.answer_yes(P1, true);
    }
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    cast_targeting(&mut t, P1, "Divination", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 1, "Alms Collector applied");
    // P1's one remaining draw was replaced by dredge.
    assert_eq!(t.zone(imp), Zone::Hand(P1));
    assert_eq!(t.hand_size(P1), h1 + 1);
}

#[test]
fn alms_collector_ignores_cards_put_into_hand_without_drawing() {
    cr!("121.1");
    ruling!(
        "Alms Collector",
        "If an effect puts cards into a player’s hand without using the word “draw” at all, Alms Collector’s replacement effect doesn’t apply."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Alms Collector");
    let a = t.graveyard(P1, "Grizzly Bears");
    let b = t.graveyard(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    let h0 = t.hand_size(P0);
    t.answer_targets(P1, &[obj(a), obj(b)]);
    cast_targeting(&mut t, P1, "Morbid Plunder", &[]);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Hand(P1));
    assert_eq!(t.zone(b), Zone::Hand(P1));
    assert_eq!(t.hand_size(P0), h0, "P0 drew nothing");
}

#[test]
fn alms_collector_two_collectors_the_drawing_player_chooses() {
    cr!("616.1", "614.5");
    ruling!(
        "Alms Collector",
        "If two players each control an Alms Collector and a third player would draw two or more cards, the third player chooses which Alms Collector’s replacement effect will apply"
    );
    for pick in [0usize, 1] {
        let mut t = ffa(3);
        t.battlefield(P0, "Alms Collector");
        t.battlefield(P1, "Alms Collector");
        t.answer(
            P2,
            DecisionKind::Option,
            mtg_engine::decision::Answer::Index(pick),
        );
        let from = t.asked().len();
        let (h0, h1, h2) = (t.hand_size(P0), t.hand_size(P1), t.hand_size(P2));
        draw(&mut t, P2, 2);
        assert!(
            t.asked()[from..].iter().any(|(p, d)| *p == P2
                && matches!(d, mtg_engine::decision::Decision::ChooseOption { prompt, .. }
                    if prompt.contains("replacement"))),
            "P2 chooses"
        );
        assert_eq!(t.hand_size(P2), h2 + 1);
        // The first option is P0's Collector (created first), the second P1's.
        let (d0, d1) = if pick == 0 { (1, 0) } else { (0, 1) };
        assert_eq!(t.hand_size(P0), h0 + d0, "pick {pick}");
        assert_eq!(t.hand_size(P1), h1 + d1, "pick {pick}");
    }
}

#[test]
fn alms_collector_two_collectors_each_player_draws_two() {
    cr!("614.5", "616.1");
    ruling!(
        "Alms Collector",
        "If two players each control an Alms Collector and an effect instructs them to each draw two or more cards, the replacement effect of each Alms Collector is applied and both players end up drawing two cards."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Alms Collector");
    t.battlefield(P1, "Alms Collector");
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    cast_targeting(&mut t, P0, "Vision Skeins", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 2);
    assert_eq!(t.hand_size(P1), h1 + 2);
}

#[test]
fn alms_collector_isnt_applied_again_to_thought_reflections_draw() {
    cr!("614.5", "616.1");
    ruling!(
        "Alms Collector",
        "Once a replacement effect has been applied to an event, it can’t be applied again to the resulting events. For example, once Alms Collector’s replacement effect has modified the effect of a player’s Divination, Thought Reflection can double that player’s resulting card draw without Alms Collector’s replacement effect applying again."
    );
    supported("Thought Reflection");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Alms Collector");
    t.battlefield(P1, "Thought Reflection");
    t.set_step(P1, Step::PrecombatMain);
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    cast_targeting(&mut t, P1, "Divination", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 1);
    assert_eq!(t.hand_size(P1), h1 + 2, "one card, doubled");
}

#[test]
fn consecrated_sphinx_triggers_once_per_card_with_separate_choices() {
    cr!("603.2c", "603.5");
    ruling!(
        "Consecrated Sphinx",
        "The ability triggers once for each card an opponent draws. You choose whether to draw two cards as each of those abilities resolves."
    );
    supported("Consecrated Sphinx");
    let mut t = TestGame::new(2);
    let sphinx = t.battlefield(P0, "Consecrated Sphinx");
    t.set_step(P1, Step::PrecombatMain);
    draw(&mut t, P1, 2);
    assert_eq!(triggers_from(&t, sphinx), 2);
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    let h0 = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 2);
}

#[test]
fn two_consecrated_sphinxes_trigger_each_other_until_one_declines() {
    cr!("603.2", "603.5");
    ruling!(
        "Consecrated Sphinx",
        "If each player controls a Consecrated Sphinx, their abilities will cause each other to trigger until one player chooses not to draw cards."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Consecrated Sphinx");
    t.battlefield(P1, "Consecrated Sphinx");
    t.set_step(P1, Step::PrecombatMain);
    // P1 draws one: P0 draws two; P1's Sphinx triggers twice: P1 draws two, then
    // declines; P0's two new triggers: P0 declines both.
    for a in [true, false, false] {
        t.answer_yes(P0, a);
    }
    for a in [true, false] {
        t.answer_yes(P1, a);
    }
    let (h0, h1) = (t.hand_size(P0), t.hand_size(P1));
    draw(&mut t, P1, 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 2);
    assert_eq!(t.hand_size(P1), h1 + 3);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn krang_extra_trigger_choices_are_made_separately() {
    cr!("603.2d", "603.5");
    ruling!(
        "Krang, the All-Powerful",
        "Krang's first ability doesn't copy the triggered ability; it just causes the ability to trigger an additional time."
    );
    supported("Krang, the All-Powerful");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Krang, the All-Powerful");
    let sphinx = t.battlefield(P0, "Consecrated Sphinx");
    t.set_step(P1, Step::PrecombatMain);
    draw(&mut t, P1, 1);
    assert_eq!(triggers_from(&t, sphinx), 2, "an additional time");
    // Separate choices on resolution: draw for one, not for the other.
    t.answer_yes(P0, true);
    t.answer_yes(P0, false);
    let h0 = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 2, "drew for one instance only");
}

#[test]
fn archivist_of_oghma_triggers_after_the_search_with_other_triggers_in_apnap_order() {
    cr!("603.3b", "101.4", "701.23a");
    ruling!(
        "Archivist of Oghma",
        "Archivist of Oghma's triggered ability won't be put onto the stack until after the spell or ability causing the opponent to search their library finishes resolving."
    );
    supported("Archivist of Oghma");
    supported("Steppe Lynx");
    let mut t = TestGame::new(2);
    let archivist = t.battlefield(P0, "Archivist of Oghma");
    let wurm = t.battlefield(P0, "Sarinth Greatwurm");
    let lynx = t.battlefield(P1, "Steppe Lynx");
    t.library_top(P1, "Forest");
    let wilds = t.battlefield(P1, "Evolving Wilds");
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P1, wilds, 0, &[]).unwrap();
    t.resolve();
    // All three triggered on the search and the land entering; P1's (the active
    // player's) went on the stack first, then P0's.
    assert_eq!(triggers_from(&t, archivist), 1);
    assert_eq!(triggers_from(&t, wurm), 1);
    assert_eq!(triggers_from(&t, lynx), 1);
    let bottom = t.g.stack[0];
    assert!(matches!(
        t.g.obj(bottom).stack.as_deref().map(|s| &s.kind),
        Some(mtg_engine::object::StackKind::Triggered { source, .. }) if *source == lynx
    ));
    let (h0, l0) = (t.hand_size(P0), t.life(P0));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), h0 + 1);
    assert_eq!(t.life(P0), l0 + 1);
}

#[test]
fn archivist_of_oghma_ignores_searches_of_another_players_library() {
    cr!("701.23a");
    ruling!(
        "Archivist of Oghma",
        "Archivist of Oghma's triggered ability won't trigger if you search an opponent's library or an opponent searches another player's library."
    );
    supported("Bribery");
    // An opponent searches P0's library.
    let mut t = TestGame::new(2);
    let archivist = t.battlefield(P0, "Archivist of Oghma");
    t.library_top(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_targeting(&mut t, P1, "Bribery", &[Entity::Player(P0)]);
    t.resolve();
    assert_eq!(triggers_from(&t, archivist), 0);
    // P0 searches its opponent's library.
    let mut t = TestGame::new(2);
    let archivist = t.battlefield(P0, "Archivist of Oghma");
    t.library_top(P1, "Grizzly Bears");
    cast_targeting(&mut t, P0, "Bribery", &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(triggers_from(&t, archivist), 0);
    // (An opponent searching their own library does trigger it.)
    let mut t = TestGame::new(2);
    let archivist = t.battlefield(P0, "Archivist of Oghma");
    t.library_top(P1, "Forest");
    let wilds = t.battlefield(P1, "Evolving Wilds");
    t.set_step(P1, Step::PrecombatMain);
    t.activate(P1, wilds, 0, &[]).unwrap();
    t.resolve();
    assert_eq!(triggers_from(&t, archivist), 1);
}

#[test]
fn cephalid_sage_draws_and_discards_in_one_resolution() {
    cr!("608.2c");
    ruling!(
        "Cephalid Sage",
        "You draw three cards and discard two cards all while Cephalid Sage’s ability is resolving."
    );
    supported("Cephalid Sage");
    let mut t = TestGame::new(2);
    fill_graveyard(&mut t, P0, 7);
    let (h, g) = (t.hand_size(P0), t.graveyard_size(P0));
    t.enter(P0, "Cephalid Sage");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.hand_size(P0), h + 1);
    assert_eq!(t.graveyard_size(P0), g + 2);
}
