//! CR 701.59: collect evidence; CR 701.61: forage.

use crate::a701_028_071_common::*;
use mtg_engine::kwa::evidence_forage::{ALT_COST_METHOD, COLLECTED_EVIDENCE, FORAGED};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

fn thopters(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Thopter"))
        .count()
}

fn food(t: &mut TestGame, p: PlayerId) -> ObjectId {
    run(
        t,
        p,
        None,
        mtg_engine::ability::Effect::CreateToken {
            spec: mtg_engine::tokens::predefined("Food").expect("Food"),
            count: mtg_engine::ability::Value::c(1),
            controller: mtg_engine::ability::PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &[],
    );
    t.g.permanents()
        .filter(|o| o.chars.has_subtype("Food"))
        .map(|o| o.id)
        .last()
        .expect("a Food")
}

#[test]
fn collecting_evidence_exiles_cards_with_total_mana_value_n_or_greater() {
    cr!("701.59a");
    supported("Forensic Researcher");
    // "{T}, Collect evidence 3: Tap target creature you don't control."
    let mut t = TestGame::new(2);
    let researcher = t.battlefield(P0, "Forensic Researcher");
    let target = t.battlefield(P1, "Grizzly Bears");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let elves = t.graveyard(P0, "Llanowar Elves");
    let giant = t.graveyard(P0, "Hill Giant");
    // The player chooses the cards: 2 + 1 = 3.
    choose(&mut t, P0, &[bears, elves]);
    t.activate(P0, researcher, 1, &[Entity::Object(target)])
        .expect("activate");
    t.resolve_all();
    assert!(t.obj_now(target).tapped);
    assert_eq!(t.zone(bears), Zone::Exile);
    assert_eq!(t.zone(elves), Zone::Exile);
    assert_eq!(t.zone(giant), Zone::Graveyard(P0));
    assert_eq!(custom_events(&t, COLLECTED_EVIDENCE), vec![(Some(P0), None, 3)]);
    // Any number of cards: more than needed may be exiled.
    let mut t = TestGame::new(2);
    let researcher = t.battlefield(P0, "Forensic Researcher");
    let target = t.battlefield(P1, "Grizzly Bears");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let elves = t.graveyard(P0, "Llanowar Elves");
    let giant = t.graveyard(P0, "Hill Giant");
    choose(&mut t, P0, &[bears, elves, giant]);
    t.activate(P0, researcher, 1, &[Entity::Object(target)])
        .expect("activate");
    assert_eq!(t.graveyard_size(P0), 0);
}

#[test]
fn whenever_you_collect_evidence_triggers_for_any_collection() {
    cr!("701.59a");
    ruling!(
        "Surveillance Monitor",
        "Surveillance Monitor's last ability triggers whenever you collect evidence for any reason, not just when you collect evidence with its first ability."
    );
    supported("Surveillance Monitor");
    // "Whenever you collect evidence, create a 1/1 colorless Thopter artifact creature
    // token with flying."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Surveillance Monitor");
    let researcher = t.battlefield(P0, "Forensic Researcher");
    let target = t.battlefield(P1, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    t.activate(P0, researcher, 1, &[Entity::Object(target)])
        .expect("activate");
    t.resolve_all();
    assert_eq!(thopters(&t), 1);
    // Its own "you may collect evidence 4" as it enters, too; an opponent collecting
    // evidence doesn't count.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Surveillance Monitor");
    t.graveyard(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.enter(P0, "Surveillance Monitor");
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(thopters(&t), 1);
    assert!(t.g.permanents().all(|o| !o.is_token() || o.controller == P0));
}

#[test]
fn a_player_who_cant_exile_enough_cant_choose_to_collect_evidence() {
    cr!("701.59b");
    ruling!(
        "Vitu-Ghazi Inspector",
        "If you can't exile enough cards to meet or exceed the required mana value, you can't choose to collect evidence at all."
    );
    // As a cost: total mana value 2 < 3.
    let mut t = TestGame::new(2);
    let researcher = t.battlefield(P0, "Forensic Researcher");
    let target = t.battlefield(P1, "Grizzly Bears");
    let bears = t.graveyard(P0, "Grizzly Bears");
    assert!(t
        .activate(P0, researcher, 1, &[Entity::Object(target)])
        .is_err());
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert!(!t.obj_now(researcher).tapped);
    // As an optional instruction: the player isn't given the choice.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let elves = t.graveyard(P0, "Llanowar Elves");
    t.answer_yes(P0, true);
    t.enter(P0, "Surveillance Monitor");
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Graveyard(P0));
    assert_eq!(t.zone(elves), Zone::Graveyard(P0));
    assert_eq!(thopters(&t), 0);
    assert!(custom_events(&t, COLLECTED_EVIDENCE).is_empty());
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
    // As an optional additional cost: not offered.
    supported("Vitu-Ghazi Inspector");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    t.lands(P0, "Forest", 2);
    let card = t.hand(P0, "Vitu-Ghazi Inspector");
    t.cast(P0, card).kicked(true).go();
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::OptionalCost { .. })));
    assert_eq!(t.graveyard_size(P0), 1);
}

/// Casts Crimestopper Sprite ("As an additional cost to cast this spell, you may collect
/// evidence 6. ... When this creature enters, tap target creature. If evidence was
/// collected, put a stun counter on it.") targeting `target`.
fn sprite(t: &mut TestGame, target: ObjectId, collect: bool) {
    supported("Crimestopper Sprite");
    t.lands(P0, "Island", 3);
    let card = t.hand(P0, "Crimestopper Sprite");
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.cast(P0, card).kicked(collect).go();
    t.resolve_all();
}

#[test]
fn abilities_can_refer_to_whether_evidence_was_collected_as_an_additional_cost() {
    cr!("701.59c");
    ruling!(
        "Crimestopper Sprite",
        "If the target creature is already tapped as the ability resolves and evidence was collected, you will still put a stun counter on it."
    );
    // Evidence collected.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Grizzly Bears");
    t.g.tap(target);
    let dragon = t.graveyard(P0, "Shivan Dragon");
    sprite(&mut t, target, true);
    assert_eq!(t.zone(dragon), Zone::Exile);
    assert!(t.obj_now(target).tapped);
    assert_eq!(t.counters(target, counters::STUN), 1);
    // Not collected (although the player could have).
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Grizzly Bears");
    let dragon = t.graveyard(P0, "Shivan Dragon");
    sprite(&mut t, target, false);
    assert_eq!(t.zone(dragon), Zone::Graveyard(P0));
    assert!(t.obj_now(target).tapped);
    assert_eq!(t.counters(target, counters::STUN), 0);
    // Collecting evidence some other way doesn't count: Vitu-Ghazi Inspector's linked
    // "if evidence was collected" looks only at its own additional cost.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P0, "Shivan Dragon");
    t.lands(P0, "Forest", 2);
    let card = t.hand(P0, "Vitu-Ghazi Inspector");
    t.cast(P0, card).kicked(false).go();
    run(
        &mut t,
        P0,
        None,
        ka(
            mtg_engine::ability::KeywordAction::CollectEvidence,
            mtg_engine::ability::Sel::None,
            6,
        ),
        &[],
    );
    assert_eq!(t.graveyard_size(P0), 0);
    t.answer_targets(P0, &[Entity::Object(target)]);
    let life = t.life(P0);
    t.resolve_all();
    assert_eq!(t.counters(target, counters::PLUS1), 0);
    assert_eq!(t.life(P0), life);
    // With it: +1/+1 counter and 2 life.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P0, "Grizzly Bears");
    t.graveyard(P0, "Shivan Dragon");
    t.lands(P0, "Forest", 2);
    let card = t.hand(P0, "Vitu-Ghazi Inspector");
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.cast(P0, card).kicked(true).go();
    let life = t.life(P0);
    t.resolve_all();
    assert_eq!(t.counters(target, counters::PLUS1), 1);
    assert_eq!(t.life(P0), life + 2);
}

/// Treetop Sentries: "When this creature enters, you may forage. If you do, draw a card."
fn sentries(t: &mut TestGame) {
    supported("Treetop Sentries");
    t.answer_yes(P0, true);
    t.enter(P0, "Treetop Sentries");
    t.resolve_all();
}

#[test]
fn to_forage_exile_three_cards_from_your_graveyard_or_sacrifice_a_food() {
    cr!("701.61a");
    // Exile three cards from your graveyard.
    let mut t = TestGame::new(2);
    let a = t.graveyard(P0, "Grizzly Bears");
    let b = t.graveyard(P0, "Llanowar Elves");
    let c = t.graveyard(P0, "Hill Giant");
    let d = t.graveyard(P0, "Shivan Dragon");
    choose(&mut t, P0, &[a, c, d]);
    let hand = t.hand_size(P0);
    sentries(&mut t);
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Graveyard(P0));
    assert_eq!(t.zone(c), Zone::Exile);
    assert_eq!(t.zone(d), Zone::Exile);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(custom_events(&t, FORAGED), vec![(Some(P0), None, 0)]);
    // Or sacrifice a Food; with both possible, the player chooses.
    let mut t = TestGame::new(2);
    let f = food(&mut t, P0);
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
    }
    option(&mut t, P0, 1);
    let hand = t.hand_size(P0);
    sentries(&mut t);
    assert!(!t.on_battlefield(f));
    assert_eq!(t.graveyard_size(P0), 3);
    assert_eq!(t.hand_size(P0), hand + 1);
    // Only a Food: it's sacrificed.
    let mut t = TestGame::new(2);
    let f = food(&mut t, P0);
    t.graveyard(P0, "Grizzly Bears");
    sentries(&mut t);
    assert!(!t.on_battlefield(f));
    assert_eq!(t.graveyard_size(P0), 1);
    // An opponent's Food can't be sacrificed to forage.
    let mut t = TestGame::new(2);
    let f = food(&mut t, P1);
    let hand = t.hand_size(P0);
    sentries(&mut t);
    assert!(t.on_battlefield(f));
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn a_player_who_cant_forage_cant_choose_to() {
    cr!("701.61a");
    ruling!(
        "Treetop Sentries",
        "If you don't have enough cards in your graveyard or a Food on the battlefield, you can't choose to forage."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    t.graveyard(P0, "Hill Giant");
    let hand = t.hand_size(P0);
    sentries(&mut t);
    assert_eq!(t.graveyard_size(P0), 2);
    assert_eq!(t.hand_size(P0), hand);
    assert!(custom_events(&t, FORAGED).is_empty());
}

#[test]
fn whenever_you_forage() {
    cr!("701.61a");
    supported("Corpseberry Cultivator");
    // "Whenever you forage, put a +1/+1 counter on this creature."
    let mut t = TestGame::new(2);
    let cultivator = t.battlefield(P0, "Corpseberry Cultivator");
    let f = food(&mut t, P0);
    sentries(&mut t);
    assert!(!t.on_battlefield(f));
    assert_eq!(t.counters(cultivator, counters::PLUS1), 1);
}

fn clues(t: &TestGame) -> usize {
    t.g.permanents()
        .filter(|o| o.is_token() && o.chars.has_subtype("Clue"))
        .count()
}

#[test]
fn collecting_evidence_for_an_alternative_cost_isnt_the_linked_additional_cost() {
    cr!("701.59c");
    ruling!(
        "Conspiracy Unraveler",
        "any additional effects that occur \"if evidence was collected\" will occur only if that additional cost was paid. Using the alternative cost from Conspiracy Unraveler will not cause those additional effects to occur."
    );
    ruling!(
        "Conspiracy Unraveler",
        "abilities of permanents that trigger \"whenever you collect evidence\" will trigger twice."
    );
    supported("Conspiracy Unraveler");
    supported("Evidence Examiner");
    // Conspiracy Unraveler: "You may collect evidence 10 rather than pay the mana cost for
    // spells you cast." Evidence Examiner: "Whenever you collect evidence, investigate."
    let setup = |t: &mut TestGame| -> ObjectId {
        t.battlefield(P0, "Conspiracy Unraveler");
        t.battlefield(P0, "Evidence Examiner");
        for _ in 0..3 {
            t.graveyard(P0, "Shivan Dragon");
        }
        let target = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[Entity::Object(target)]);
        target
    };
    // Crimestopper Sprite for the alternative cost only: evidence was collected, but not
    // as the linked additional cost.
    let mut t = TestGame::new(2);
    let target = setup(&mut t);
    let card = t.hand(P0, "Crimestopper Sprite");
    t.cast(P0, card)
        .method(mtg_engine::object::CastMethod::Alternative(ALT_COST_METHOD))
        .kicked(false)
        .go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Crimestopper Sprite").len(), 1);
    assert_eq!(t.graveyard_size(P0), 1);
    assert!(t.obj_now(target).tapped);
    assert_eq!(t.counters(target, counters::STUN), 0);
    assert_eq!(clues(&t), 1);
    // Both: two collections, two triggers, and the stun counter.
    let mut t = TestGame::new(2);
    let target = setup(&mut t);
    let card = t.hand(P0, "Crimestopper Sprite");
    t.cast(P0, card)
        .method(mtg_engine::object::CastMethod::Alternative(ALT_COST_METHOD))
        .kicked(true)
        .go();
    t.resolve_all();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(custom_events(&t, COLLECTED_EVIDENCE).len(), 2);
    assert_eq!(clues(&t), 2);
    assert_eq!(t.counters(target, counters::STUN), 1);
}

#[test]
fn a_card_exiled_by_the_same_instruction_isnt_evidence() {
    cr!("701.59a", "701.59b");
    ruling!(
        "Lamplight Phoenix",
        "You can't exile Lamplight Phoenix from your graveyard to pay the collect evidence cost of its triggered ability."
    );
    supported("Lamplight Phoenix");
    // {1}{R}{R}: "When this creature dies, you may exile it and collect evidence 4. If you
    // do, return this card to the battlefield tapped."
    // The Phoenix (mana value 3) and Llanowar Elves (1) total 4, but without the Phoenix
    // there's only 1: the player can't choose to.
    let mut t = TestGame::new(2);
    let phoenix = t.battlefield(P0, "Lamplight Phoenix");
    let elves = t.graveyard(P0, "Llanowar Elves");
    t.answer_yes(P0, true);
    t.g.destroy(phoenix, None);
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(phoenix)), Zone::Graveyard(P0));
    assert_eq!(t.zone(elves), Zone::Graveyard(P0));
    assert!(!t
        .asked()
        .iter()
        .any(|(_, d)| matches!(d, Decision::YesNo { .. })));
    // With Hill Giant (4) instead, it can: the Giant is exiled and the Phoenix returns.
    let mut t = TestGame::new(2);
    let phoenix = t.battlefield(P0, "Lamplight Phoenix");
    let giant = t.graveyard(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.g.destroy(phoenix, None);
    t.resolve_all();
    let back = t.g.current(phoenix);
    assert_eq!(t.zone(back), Zone::Battlefield);
    assert!(t.obj_now(back).tapped);
    assert_eq!(t.zone(giant), Zone::Exile);
    assert_eq!(custom_events(&t, COLLECTED_EVIDENCE), vec![(Some(P0), None, 4)]);
}
