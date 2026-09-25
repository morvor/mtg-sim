//! CR 702.62 Suspend.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_027_037::untapped_lands;
use crate::common_k702_038_051::with_cost;
use crate::common_k702_052_066::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

const TIME: &str = "time";
const SUSPEND: &str = "Suspend";

/// Whether `p` may suspend `card` now.
fn can_suspend(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    let card = t.g.current(card);
    t.g.legal_actions(p)
        .contains(&Action::Special(SpecialAction::Suspend { card }))
}

/// Suspends `card` for `p` (paying its suspend cost); returns the card in exile.
fn suspend(t: &mut TestGame, p: PlayerId, card: ObjectId) -> ObjectId {
    t.g.turn.priority = Some(p);
    let card = t.g.current(card);
    t.g.perform_action(p, Action::Special(SpecialAction::Suspend { card }))
        .expect("couldn't suspend");
    let exiled = t.g.current(card);
    assert_eq!(t.zone(exiled), Zone::Exile);
    exiled
}

#[test]
fn a_suspended_creature_counts_down_is_cast_and_has_haste() {
    cr!("702.62", "702.62a");
    ruling!(
        "Greater Gargadon",
        "It will have haste until another player gains control of it."
    );
    ruling!(
        "Lotus Bloom",
        "Exiling a card with suspend isn't casting that card. This action doesn't use the stack and can't be responded to."
    );
    ruling!("Lotus Bloom", "Cards exiled with suspend are exiled face up.");
    assert_supported("Keldon Halberdier");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let halberdier = t.hand(P0, "Keldon Halberdier");
    // "Suspend 4—{R}": pay {R} and exile it face up with four time counters, without
    // using the stack or casting it.
    let exiled = suspend(&mut t, P0, halberdier);
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.stack_len(), 0);
    assert!(t.g.history.spells_cast.is_empty());
    assert!(!t.obj_now(exiled).face_down);
    assert_eq!(t.counters(exiled, TIME), 4);
    // Each of its owner's upkeeps removes one.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.counters(exiled, TIME), 4);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(stack_triggers(&t, SUSPEND).len(), 1);
    t.resolve();
    assert_eq!(t.counters(exiled, TIME), 3);
    remove_counters(&mut t, exiled, TIME, 2);
    // The last one: it may be cast without paying its mana cost.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve();
    assert_eq!(t.counters(exiled, TIME), 0);
    assert_eq!(stack_triggers(&t, SUSPEND).len(), 1);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(t.obj_now(halberdier).is_spell());
    t.resolve();
    assert!(t.on_battlefield(halberdier));
    // It has haste: it can attack this turn.
    assert!(t.obj_now(halberdier).chars.has_keyword(KeywordKind::Haste));
    t.advance_to(P0, Step::BeginningOfCombat);
    let h = t.g.current(halberdier);
    t.attack(&[(h, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 16);
    // Until another player gains control of it.
    run_effect(
        &mut t,
        None,
        P1,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[Entity::Object(h)],
    );
    assert_eq!(t.obj_now(h).controller, P1);
    assert!(!t.obj_now(h).chars.has_keyword(KeywordKind::Haste));
}

#[test]
fn the_card_may_stay_exiled_instead_of_being_cast() {
    cr!("702.62a");
    ruling!(
        "Lotus Bloom",
        "If you don't cast the card, it remains exiled with no time counters on it, and it's no longer suspended."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Rift Bolt");
    let exiled = suspend(&mut t, P0, bolt);
    t.answer_yes(P0, false);
    next_upkeep_resolved(&mut t);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(t.counters(exiled, TIME), 0);
    // No longer suspended: nothing more happens.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert!(stack_triggers(&t, SUSPEND).is_empty());
}

/// Advances to P0's next upkeep and resolves everything put on the stack there.
fn next_upkeep_resolved(t: &mut TestGame) {
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
}

#[test]
fn a_suspended_card_is_one_in_exile_with_suspend_and_a_time_counter() {
    cr!("702.62b");
    ruling!(
        "Lotus Bloom",
        "It doesn't matter why the last time counter was removed or what effect removed it."
    );
    assert_supported("Lotus Bloom");
    let mut t = TestGame::new(2);
    // A card with suspend in exile without time counters isn't suspended.
    let unsuspended = t.exile(P0, "Rift Bolt");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert!(stack_triggers(&t, SUSPEND).is_empty());
    assert_eq!(t.zone(unsuspended), Zone::Exile);
    // Removing the last time counter by another effect lets it be cast.
    t.set_step(P0, Step::PrecombatMain);
    let bloom = t.hand(P0, "Lotus Bloom");
    let exiled = suspend(&mut t, P0, bloom);
    assert_eq!(t.counters(exiled, TIME), 3);
    remove_counters(&mut t, exiled, TIME, 3);
    t.settle();
    assert_eq!(stack_triggers(&t, SUSPEND).len(), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(bloom));
}

#[test]
fn suspend_is_possible_only_if_the_card_could_be_cast() {
    cr!("702.62c");
    ruling!(
        "Lotus Bloom",
        "any other effects that stop you from casting it (such as from Meddling Mage's ability)"
    );
    assert_supported("Meddling Mage");
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Rift Bolt");
    assert!(can_suspend(&mut t, P0, bolt));
    // Meddling Mage naming Rift Bolt: it can't be cast, so it can't be suspended.
    t.answer(P1, DecisionKind::Name, Answer::Text("Rift Bolt".into()));
    t.enter(P1, "Meddling Mage");
    t.resolve_all();
    assert!(!can_suspend(&mut t, P0, bolt));
    assert!(t
        .g
        .perform_action(P0, Action::Special(SpecialAction::Suspend { card: bolt }))
        .is_err());
    assert!(t.in_hand(P0, "Rift Bolt"));
}

#[test]
fn a_split_second_spell_prevents_suspending_an_instant() {
    cr!("702.62c");
    ruling!(
        "Lotus Bloom",
        "you can exile a card with suspend that has no mana cost or that requires a target even if no legal targets are available at that time"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 2);
    // An instant with suspend, with no creature for it to target: it could still begin
    // to be cast, during an opponent's turn.
    let sentence = t.hand(P0, "Suspended Sentence");
    t.set_step(P1, Step::PrecombatMain);
    assert!(can_suspend(&mut t, P0, sentence));
    // Not while a spell with split second is on the stack.
    t.lands(P1, "Mountain", 2);
    let shock = t.hand(P1, "Sudden Shock");
    t.cast(P1, shock).target(P1).go();
    assert!(!can_suspend(&mut t, P0, sentence));
    t.resolve_all();
    assert!(can_suspend(&mut t, P0, sentence));
    // A card with no mana cost can be suspended though it can't be cast from the hand.
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Island", 1);
    let vision = t.hand(P0, "Ancestral Vision");
    assert!(can_suspend(&mut t, P0, vision));
    assert!(!crate::common_k702_027_037::can_cast(
        &mut t,
        P0,
        vision,
        mtg_engine::object::CastMethod::Normal
    ));
}

#[test]
fn a_card_without_a_mana_cost_is_cast_by_suspending_it() {
    cr!("702.62a", "702.62d");
    ruling!(
        "Lotus Bloom",
        "A card with no mana cost can't be cast normally; you'll need a way to cast it for an alternative cost or without paying its mana cost, such as by suspending it."
    );
    let mut t = TestGame::new(2);
    let bloom = t.hand(P0, "Lotus Bloom");
    assert!(t.cast(P0, bloom).try_go().is_err());
    let exiled = suspend(&mut t, P0, bloom);
    remove_counters(&mut t, exiled, TIME, 3);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(bloom));
}

#[test]
fn targets_are_chosen_when_the_suspended_card_is_cast() {
    cr!("702.62a");
    ruling!(
        "Lotus Bloom",
        "If the spell requires any targets, those targets are chosen when the spell is finally cast, not when it's exiled."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Rift Bolt");
    let asked = t.asked().len();
    let exiled = suspend(&mut t, P0, bolt);
    assert!(!t.asked()[asked..]
        .iter()
        .any(|(_, d)| matches!(d, decision::Decision::ChooseTargets { .. })));
    // A creature that wasn't there when it was suspended can be targeted.
    let bears = t.battlefield(P1, "Grizzly Bears");
    remove_counters(&mut t, exiled, TIME, 1);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn countering_the_upkeep_trigger_removes_no_counter() {
    cr!("702.62a");
    ruling!(
        "Lotus Bloom",
        "If the first triggered ability of suspend (the one that removes time counters) is countered, no time counter is removed."
    );
    let mut t = TestGame::new(2);
    let bloom = t.hand(P0, "Lotus Bloom");
    let exiled = suspend(&mut t, P0, bloom);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    let (trigger, _) = stack_triggers(&t, SUSPEND)[0];
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    t.cast(P1, stifle).target(trigger).go();
    t.resolve_all();
    assert_eq!(t.counters(exiled, TIME), 3);
    // It triggers again at the next upkeep.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(stack_triggers(&t, SUSPEND).len(), 1);
    t.resolve();
    assert_eq!(t.counters(exiled, TIME), 2);
}

#[test]
fn casting_with_suspend_is_casting_without_paying_the_mana_cost() {
    cr!("702.62d");
    ruling!(
        "Lotus Bloom",
        "You can, however, pay additional costs."
    );
    ruling!(
        "Lotus Bloom",
        "The mana value of a spell cast without paying its mana cost is determined by its mana cost"
    );
    let mut t = TestGame::new(2);
    // An optional additional cost (kicker) may be paid.
    let def = with_cost(
        custom_card(
            "Kicked Rift",
            "Sorcery",
            None,
            "Kicker {2}\nSuspend 1—{R}\n~ deals 1 damage to any target. If this spell was kicked, it deals 3 damage instead.",
        ),
        "{2}{R}",
    );
    let card = t.custom(P0, def, Zone::Hand(P0));
    t.lands(P0, "Mountain", 1);
    suspend(&mut t, P0, card);
    t.lands(P0, "Mountain", 2);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve();
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    let spell = t.g.current(card);
    assert!(t.obj_now(spell).is_spell());
    // Its mana value is its mana cost's, though that cost wasn't paid; the kicker ({2}
    // of the three untapped Mountains) was.
    assert_eq!(t.obj_now(spell).chars.mana_value(), 3);
    assert_eq!(untapped_lands(&t, P0), 1);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_suspended_spell_with_x_is_cast_with_x_equal_to_zero() {
    cr!("702.62d");
    ruling!(
        "Lotus Bloom",
        "If the card has {X} in its mana cost, you must choose 0 as the value of X when casting it without paying its mana cost."
    );
    let mut t = TestGame::new(2);
    let def = with_cost(
        custom_card(
            "Rift Blaze",
            "Sorcery",
            None,
            "Suspend 1—{R}\n~ deals X damage to any target.",
        ),
        "{X}{R}",
    );
    let card = t.custom(P0, def, Zone::Hand(P0));
    t.lands(P0, "Mountain", 1);
    suspend(&mut t, P0, card);
    t.lands(P0, "Mountain", 5);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve();
    t.answer_yes(P0, true);
    t.answer(P0, DecisionKind::X, Answer::Number(5));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    // X was 0 and nothing was paid (all six Mountains untapped in the upkeep's untap).
    assert_eq!(t.life(P1), 20);
    assert_eq!(untapped_lands(&t, P0), 6);
}

#[test]
fn suspend_x_exiles_the_card_with_x_time_counters() {
    cr!("702.62a");
    ruling!(
        "Benalish Commander",
        "both its triggered ability and the suspend \"play this card\" triggered ability will trigger"
    );
    assert_supported("Benalish Commander");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 4);
    let commander = t.hand(P0, "Benalish Commander");
    // "Suspend X—{X}{W}{W}. X can't be 0.": X = 2.
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    let exiled = suspend(&mut t, P0, commander);
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.counters(exiled, TIME), 2);
    // "Whenever a time counter is removed from this card while it's exiled, create a 1/1
    // white Soldier creature token."
    next_upkeep_resolved(&mut t);
    assert_eq!(t.counters(exiled, TIME), 1);
    assert_eq!(soldier_tokens(&t), 1);
    // The last counter: both abilities trigger.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve();
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(commander));
    // Two Soldier tokens and itself.
    assert_eq!(t.pt(commander), (3, 3));
}

/// Number of Soldier creature tokens on the battlefield.
fn soldier_tokens(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Soldier"))
        .count()
}

#[test]
fn suspend_x_needs_x_of_at_least_one() {
    cr!("702.62a");
    let mut t = TestGame::new(2);
    // {X}{W}{W} with X = 0 isn't allowed: two Plains aren't enough.
    t.lands(P0, "Plains", 2);
    let commander = t.hand(P0, "Benalish Commander");
    assert!(!can_suspend(&mut t, P0, commander));
    t.lands(P0, "Plains", 1);
    assert!(can_suspend(&mut t, P0, commander));
    t.answer(P0, DecisionKind::X, Answer::Number(0));
    let exiled = suspend(&mut t, P0, commander);
    assert_eq!(t.counters(exiled, TIME), 1);
    // Removing several time counters at once triggers once for each.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 5);
    let commander = t.hand(P0, "Benalish Commander");
    t.answer(P0, DecisionKind::X, Answer::Number(3));
    let exiled = suspend(&mut t, P0, commander);
    remove_counters(&mut t, exiled, TIME, 2);
    t.resolve_all();
    assert_eq!(soldier_tokens(&t), 2);
}

#[test]
fn abilities_that_work_only_while_the_card_is_suspended() {
    cr!("702.62b");
    ruling!(
        "Lotus Bloom",
        "If an effect refers to a \"suspended card,\" that means a card that (1) has suspend, (2) is in exile, and (3) has one or more time counters on it."
    );
    assert_supported("Greater Gargadon");
    let mut t = TestGame::new(2);
    let lands = t.lands(P0, "Mountain", 3);
    let gargadon = t.hand(P0, "Greater Gargadon");
    // "Sacrifice an artifact, creature, or land: Remove a time counter from this card.
    // Activate only if this card is suspended." Not from the hand.
    let sac = |t: &mut TestGame| {
        let src = t.g.current(gargadon);
        crate::common_k702_027_037::activate_named(
            t,
            P0,
            src,
            "Sacrifice an artifact, creature, or land: Remove a time counter from ~. Activate only if ~ is suspended.",
            0,
        )
    };
    assert!(sac(&mut t).is_err());
    let exiled = suspend(&mut t, P0, gargadon);
    assert_eq!(t.counters(exiled, TIME), 10);
    t.answer_choose(P0, &[Entity::Object(lands[1])]);
    sac(&mut t).unwrap();
    t.resolve();
    assert_eq!(t.counters(exiled, TIME), 9);
    assert!(!t.on_battlefield(lands[1]));
    // Once it has no time counter, it isn't suspended: the ability can't be activated.
    remove_counters(&mut t, exiled, TIME, 9);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert!(sac(&mut t).is_err());
}

#[test]
fn a_spell_that_exiles_itself_with_time_counters_is_suspended() {
    cr!("702.62b");
    assert_supported("Suspended Sentence");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 4);
    let sentence = t.hand(P0, "Suspended Sentence");
    t.cast(P0, sentence).target(bears).go();
    t.resolve();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P1), 17);
    // "Exile Suspended Sentence with three time counters on it": suspended.
    assert_eq!(t.zone(sentence), Zone::Exile);
    assert_eq!(t.counters(sentence, TIME), 3);
    assert!(!t.in_graveyard(P0, "Suspended Sentence"));
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(stack_triggers(&t, SUSPEND).len(), 1);
    t.resolve();
    assert_eq!(t.counters(sentence, TIME), 2);
}

#[test]
fn an_ability_can_trigger_when_the_last_time_counter_is_removed_in_exile() {
    cr!("702.62a");
    assert_supported("Riftmarked Knight");
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    let knight = t.hand(P0, "Riftmarked Knight");
    let exiled = suspend(&mut t, P0, knight);
    remove_counters(&mut t, exiled, TIME, 3);
    t.settle();
    // Suspend's cast trigger and the Knight's own trigger.
    assert_eq!(t.stack_len(), 2);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(knight));
    let tokens: Vec<_> = t
        .g
        .permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Knight"))
        .map(|o| o.chars.has_keyword(KeywordKind::Haste))
        .collect();
    assert_eq!(tokens, vec![true]);
}

#[test]
fn countering_the_cast_trigger_leaves_the_card_exiled() {
    cr!("702.62a", "702.62b");
    ruling!(
        "Lotus Bloom",
        "If the second triggered ability is countered, the card can't be cast."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Rift Bolt");
    let exiled = suspend(&mut t, P0, bolt);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve();
    let (trigger, _) = stack_triggers(&t, SUSPEND)[0];
    t.lands(P1, "Island", 1);
    let stifle = t.hand(P1, "Stifle");
    t.cast(P1, stifle).target(trigger).go();
    t.resolve_all();
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert_eq!(t.counters(exiled, TIME), 0);
    assert_eq!(t.life(P1), 20);
}
