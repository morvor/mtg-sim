//! CR 614.1a, 614.6, 701.6a: "Counter target spell. If that spell is countered this way,
//! put it [somewhere] instead of into its owner's graveyard." The modified event moves the
//! card to the whole destination the replacement names: with time counters (Delay), to
//! the position its controller chooses (Hinder), onto the battlefield under the counter
//! spell's controller's control (Desertion).

use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

fn assert_supported(name: &str) {
    let c = mtg_engine::card::card(name);
    assert!(
        c.unsupported_text().is_empty(),
        "{name}: {:?}",
        c.unsupported_text()
    );
}

/// P1 casts `name` (with enough `land`s); returns the spell.
fn opponent_casts(t: &mut TestGame, name: &str, land: &str, n: usize) -> ObjectId {
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, land, n);
    let c = t.hand(P1, name);
    t.g.turn.priority = Some(P1);
    t.cast(P1, c).go()
}

/// P0 counters `spell` with `counter` (paying with Islands).
fn counter_with(t: &mut TestGame, counter: &str, islands: usize, spell: ObjectId) {
    t.lands(P0, "Island", islands);
    let c = t.hand(P0, counter);
    t.g.turn.priority = Some(P0);
    t.cast(P0, c).target(spell).go();
    t.resolve();
}

fn has_suspend(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).chars.has_keyword(KeywordKind::Suspend)
}

#[test]
fn delay_exiles_the_spell_with_time_counters_and_it_gains_suspend() {
    cr!("614.1a", "614.6", "701.6a", "702.62a", "702.62b");
    ruling!(
        "Delay",
        "When the last time counter is removed from the exiled card, it's cast as a completely new spell."
    );
    assert_supported("Delay");
    let mut t = TestGame::new(2);
    let bears = opponent_casts(&mut t, "Grizzly Bears", "Forest", 2);
    counter_with(&mut t, "Delay", 2, bears);
    let exiled = t.g.current(bears);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert!(!t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.counters(exiled, counters::TIME), 3);
    // It gained suspend: it's suspended, and counts down at its owner's upkeeps.
    assert!(has_suspend(&t, exiled));
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(exiled, counters::TIME), 2);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(exiled, counters::TIME), 2);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(exiled, counters::TIME), 1);
    // The last counter: its owner may cast it without paying its mana cost, and the
    // creature has haste.
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    t.answer_yes(P1, true);
    t.resolve_all();
    let now = t.g.current(bears);
    assert!(t.on_battlefield(now), "{}", t.dump_log());
    assert_eq!(t.obj_now(now).controller, P1);
    assert!(t.obj_now(now).chars.has_keyword(KeywordKind::Haste));
    // The permanent is a new object: it doesn't keep the suspend Delay gave the card.
    assert!(!has_suspend(&t, now));
}

#[test]
fn a_card_that_already_has_suspend_doesnt_gain_another() {
    cr!("614.6", "702.62a");
    let mut t = TestGame::new(2);
    // Rift Bolt: sorcery, {2}{R}, suspend 1—{R}.
    let bolt = opponent_casts(&mut t, "Rift Bolt", "Mountain", 3);
    counter_with(&mut t, "Delay", 2, bolt);
    let exiled = t.g.current(bolt);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(t.counters(exiled, counters::TIME), 3);
    let suspends = t
        .obj_now(exiled)
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Suspend)
        .count();
    assert_eq!(suspends, 1);
    // One counter removed per upkeep (two instances would remove two).
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.counters(exiled, counters::TIME), 2);
}

#[test]
fn delay_exiles_a_flashback_spell_with_counters_not_the_flashback_effect() {
    cr!("614.6", "616.1a", "702.34a");
    ruling!(
        "Delay",
        "If the target spell was cast with flashback, Delay's effect will exile it, not the flashback effect."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let think = t.graveyard(P1, "Think Twice");
    t.lands(P1, "Island", 3);
    t.g.turn.priority = Some(P1);
    let spell = t
        .cast(P1, think)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    counter_with(&mut t, "Delay", 2, spell);
    let exiled = t.g.current(think);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(t.counters(exiled, counters::TIME), 3);
    assert!(has_suspend(&t, exiled));
}

#[test]
fn delay_exiles_a_face_down_spell_face_up() {
    cr!("614.6", "708.9");
    ruling!(
        "Delay",
        "If the target spell is face down, it'll be exiled face up."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.lands(P1, "Plains", 3);
    let angel = t.hand(P1, "Exalted Angel");
    t.g.turn.priority = Some(P1);
    let spell = t
        .cast(P1, angel)
        .method(CastMethod::FaceDown(KeywordKind::Morph))
        .go();
    assert!(t.obj_now(spell).face_down);
    counter_with(&mut t, "Delay", 2, spell);
    let exiled = t.g.current(angel);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert!(!t.obj_now(exiled).face_down);
    assert_eq!(t.counters(exiled, counters::TIME), 3);
    assert!(has_suspend(&t, exiled));
    // "It can't be cast face down when casting it without paying its mana cost": with the
    // last counter it's cast face up, and resolves as a face-up Exalted Angel.
    for _ in 0..3 {
        t.advance_to(P1, Step::Upkeep);
        t.resolve_all();
        t.advance_to(P0, Step::Upkeep);
    }
    t.advance_to(P1, Step::Upkeep);
    t.answer_yes(P1, true);
    t.resolve_all();
    let now = t.g.current(angel);
    assert!(t.on_battlefield(now), "{}", t.dump_log());
    assert!(!t.obj_now(now).face_down);
    assert_eq!(t.obj_now(now).chars.name.as_str(), "Exalted Angel");
    assert_eq!(t.pt(now), (4, 5));
}

#[test]
fn hinders_controller_chooses_top_or_bottom() {
    cr!("614.6", "701.6a");
    ruling!(
        "Hinder",
        "Hinder's controller, not necessarily the controller of the countered spell, chooses where the countered spell goes."
    );
    assert_supported("Hinder");
    for (choice, bottom) in [(0, false), (1, true)] {
        let mut t = TestGame::new(2);
        for _ in 0..3 {
            t.library_top(P1, "Forest");
        }
        let bears = opponent_casts(&mut t, "Grizzly Bears", "Forest", 2);
        t.answer(P0, DecisionKind::Option, Answer::Index(choice));
        counter_with(&mut t, "Hinder", 3, bears);
        let now = t.g.current(bears);
        assert_eq!(t.zone(now), Zone::Library(P1));
        let lib = &t.g.player(P1).library;
        // The top of a library is its last card.
        let at = if bottom { lib[0] } else { *lib.last().unwrap() };
        assert_eq!(at, now);
        // P0 chose; P1 wasn't asked.
        let asked: Vec<PlayerId> = t
            .asked()
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(asked, vec![P0]);
    }
}

#[test]
fn desertion_puts_a_creature_spell_onto_the_battlefield_under_your_control() {
    cr!("614.6", "701.6a", "110.2a");
    ruling!(
        "Desertion",
        "This spell includes a replacement effect. If the target is an artifact or creature spell, it never goes to the graveyard."
    );
    ruling!(
        "Desertion",
        "any effects that check if the original card was \"cast from your hand\" will not trigger"
    );
    assert_supported("Desertion");
    assert_supported("Coal Stoker");
    let mut t = TestGame::new(2);
    // Coal Stoker: "When ~ enters, if you cast it from your hand, add {R}{R}{R}."
    let stoker = opponent_casts(&mut t, "Coal Stoker", "Mountain", 4);
    counter_with(&mut t, "Desertion", 5, stoker);
    let now = t.g.current(stoker);
    assert!(t.on_battlefield(now));
    assert_eq!(t.obj_now(now).controller, P0);
    assert_eq!(t.obj_now(now).owner, P1);
    assert_eq!(t.graveyard_size(P1), 0);
    t.resolve_all();
    // Its enters ability didn't add mana: it wasn't cast from P0's hand.
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    assert_eq!(t.g.player(P1).mana_pool.total(), 0);
}

#[test]
fn desertion_puts_a_noncreature_nonartifact_spell_into_the_graveyard() {
    cr!("614.6", "701.6a");
    let mut t = TestGame::new(2);
    let bolt = opponent_casts(&mut t, "Divination", "Island", 3);
    counter_with(&mut t, "Desertion", 5, bolt);
    assert!(t.in_graveyard(P1, "Divination"));
}

#[test]
fn desertion_does_nothing_to_a_spell_that_cant_be_countered() {
    cr!("701.6a", "614.6");
    ruling!(
        "Desertion",
        "If the spell is not countered (because the spell it targets can't be countered), then this card's ability does not put the card onto the battlefield."
    );
    let mut t = TestGame::new(2);
    let tyrant = opponent_casts(&mut t, "Carnage Tyrant", "Forest", 6);
    counter_with(&mut t, "Desertion", 5, tyrant);
    assert_eq!(t.zone(tyrant), Zone::Stack);
    t.resolve();
    let now = t.g.current(tyrant);
    assert!(t.on_battlefield(now));
    assert_eq!(t.obj_now(now).controller, P1);
}

#[test]
fn a_flashback_spell_hindered_is_still_exiled() {
    cr!("614.6", "616.1a", "616.1e", "702.34a");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let think = t.graveyard(P1, "Think Twice");
    t.lands(P1, "Island", 3);
    t.g.turn.priority = Some(P1);
    let spell = t
        .cast(P1, think)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    // Hinder's self-replacement effect puts it into the library instead of the graveyard;
    // flashback's "exile it instead of putting it anywhere else" then applies.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    counter_with(&mut t, "Hinder", 3, spell);
    let now = t.g.current(think);
    assert_eq!(t.zone(now), Zone::Exile);
}
