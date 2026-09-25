//! CR 702.33 Kicker, multikicker, and sticker kicker.

use crate::common_k702_011_017::*;
use crate::common_k702_018_026::*;
use crate::common_k702_027_037::*;
use mtg_engine::ability::*;
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::stickers::{self, SheetFormat, StickerDef, StickerKind, StickerSheet};
use mtg_engine::testing::*;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Answers the optional cost decisions of `p`, in order.
fn kick(t: &mut TestGame, p: PlayerId, answers: &[bool]) {
    for a in answers {
        t.answer(p, DecisionKind::OptionalCost, Answer::Bool(*a));
    }
}

#[test]
fn kicker_is_an_optional_additional_cost_paid_once() {
    cr!("702.33", "702.33a");
    ruling!(
        "Burst Lightning",
        "The kicker ability doesn't let you pay a kicker cost more than once."
    );
    ruling!(
        "Burst Lightning",
        "The spell's mana value remains unchanged, no matter what the total cost to cast it was."
    );
    assert_supported("Burst Lightning");
    // Not kicked: {R}, 2 damage.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let bolt = t.hand(P0, "Burst Lightning");
    t.cast(P0, bolt).target(P1).kicked(false).go();
    assert_eq!(untapped_lands(&t, P0), 4);
    t.resolve();
    assert_eq!(t.life(P1), 18);
    // Kicked: {R} + {4}, 4 damage; still mana value 1.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 5);
    let bolt = t.hand(P0, "Burst Lightning");
    let spell = t.cast(P0, bolt).target(P1).kicked(true).go();
    assert_eq!(optional_costs_offered(&t, P0), vec!["kicker".to_string()]);
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.obj_now(spell).chars.mana_value(), 1);
    t.resolve();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn kicker_and_or_is_two_kicker_abilities() {
    cr!("702.33b", "702.33f");
    assert_supported("Thunderscape Battlemage");
    let bm = "Thunderscape Battlemage";
    let kws: Vec<_> = card(bm)
        .front()
        .chars
        .keywords()
        .filter(|k| k.kind == KeywordKind::Kicker)
        .cloned()
        .collect();
    assert_eq!(kws.len(), 1);
    assert_eq!(kws[0].costs.len(), 1);
    for (pay_b, pay_g) in [(false, false), (true, false), (false, true), (true, true)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 2);
        t.lands(P0, "Swamp", 2);
        t.lands(P0, "Forest", 2);
        let ench = t.battlefield(P1, "Cover of Darkness");
        t.hand(P1, "Grizzly Bears");
        t.hand(P1, "Llanowar Elves");
        let mage = t.hand(P0, bm);
        kick(&mut t, P0, &[pay_b, pay_g]);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer_targets(P0, &[Entity::Object(ench)]);
        t.cast(P0, mage).go();
        assert_eq!(
            optional_costs_offered(&t, P0),
            vec!["kicker".to_string(), "kicker".to_string()]
        );
        t.resolve_all();
        // {1}{B} kicker: target player discards two cards; {G} kicker: destroy target
        // enchantment. Each ability is linked to its own kicker cost.
        assert_eq!(t.hand_size(P1) == 0, pay_b, "{pay_b} {pay_g}");
        assert_eq!(!t.on_battlefield(ench), pay_g, "{pay_b} {pay_g}");
        let spent = 3 + 2 * pay_b as usize + pay_g as usize;
        assert_eq!(untapped_lands(&t, P0), 6 - spent);
    }
}

#[test]
fn multikicker_may_be_paid_any_number_of_times() {
    cr!("702.33c");
    assert_supported("Gnarlid Pack");
    for times in [0usize, 1, 3] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Forest", 8);
        let pack = t.hand(P0, "Gnarlid Pack");
        t.answer(
            P0,
            DecisionKind::OptionalCost,
            Answer::Number(times as i64),
        );
        t.cast(P0, pack).go();
        // {1}{G} plus {1}{G} for each time.
        assert_eq!(untapped_lands(&t, P0), 8 - 2 - 2 * times);
        t.resolve();
        assert_eq!(t.counters(pack, counters::PLUS1), times as u32);
    }
}

#[test]
fn a_spell_is_kicked_once_for_each_kicker_cost_paid() {
    cr!("702.33d");
    ruling!(
        "Archangel of Wrath",
        "You can kick it twice by paying {B}{R}, but you can't kick it twice by paying {B}{B} or {R}{R}."
    );
    ruling!(
        "Archangel of Wrath",
        "If it was kicked twice, each of Archangel of Wrath's last two abilities will trigger once."
    );
    assert_supported("Archangel of Wrath");
    for (kicks, damage) in [(0usize, 0), (1, 2), (2, 4)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Plains", 4);
        t.lands(P0, "Swamp", 1);
        t.lands(P0, "Mountain", 1);
        let angel = t.hand(P0, "Archangel of Wrath");
        kick(&mut t, P0, &[kicks >= 1, kicks >= 2]);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.cast(P0, angel).go();
        // Only one {B} kicker and one {R} kicker are offered.
        assert_eq!(optional_costs_offered(&t, P0).len(), 2);
        t.resolve_all();
        assert!(t.on_battlefield(angel));
        assert_eq!(t.life(P1), 20 - damage, "kicked {kicks} times");
    }
}

#[test]
fn a_permanent_put_onto_the_battlefield_without_being_cast_isnt_kicked() {
    cr!("702.33d", "702.33e");
    ruling!(
        "Burst Lightning",
        "If you put a permanent with a kicker ability onto the battlefield without casting it, you can't kick it."
    );
    ruling!(
        "Burst Lightning",
        "If a card or token enters as a copy of a permanent, the new permanent isn't kicked, even if the original was."
    );
    assert_supported("Kavu Titan");
    assert_supported("Clone");
    let mut t = TestGame::new(2);
    let titan = t.enter(P0, "Kavu Titan");
    assert_eq!(t.counters(titan, counters::PLUS1), 0);
    assert!(!t.obj_now(titan).has_keyword(KeywordKind::Trample));
    // A kicked Kavu Titan, then a Clone copying it: the Clone isn't kicked.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let titan = t.hand(P0, "Kavu Titan");
    t.cast(P0, titan).kicked(true).go();
    t.resolve();
    assert_eq!(t.counters(titan, counters::PLUS1), 3);
    assert!(t.obj_now(titan).has_keyword(KeywordKind::Trample));
    t.lands(P0, "Island", 4);
    let clone = t.hand(P0, "Clone");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(t.g.current(titan))]);
    t.cast(P0, clone).kicked(true).go();
    t.resolve();
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Kavu Titan");
    assert_eq!(t.counters(clone, counters::PLUS1), 0);
    assert!(!t.obj_now(clone).has_keyword(KeywordKind::Trample));
}

/// A sticker sheet with one power/toughness sticker costing a ticket.
fn give_sheet(t: &mut TestGame, p: PlayerId) {
    let sheet = StickerSheet {
        name: "Sheet".into(),
        stickers: vec![StickerDef {
            kind: StickerKind::PowerToughness(5, 5),
            ticket_cost: 1,
        }],
    };
    stickers::choose_sheets(&mut t.g, p, vec![sheet], SheetFormat::Limited).unwrap();
}

#[test]
fn abilities_that_check_kicked_refer_only_to_the_printed_kicker() {
    cr!("702.33e", "702.33h");
    ruling!(
        "Wicker Picker",
        "that ability is linked to its own printed kicker or multikicker ability and will apply only if you paid its own kicker or multikicker cost. Sticker kicker won't turn on those effects."
    );
    assert_supported("Wicker Picker");
    assert_supported("Bog Badger");
    // Only the sticker kicker is paid: Bog Badger's ability doesn't trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wicker Picker");
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Swamp", 1);
    let badger = t.hand(P0, "Bog Badger");
    // Bog Badger's own kicker first, then the granted sticker kicker.
    kick(&mut t, P0, &[false, true]);
    t.cast(P0, badger).go();
    assert_eq!(
        optional_costs_offered(&t, P0),
        vec!["kicker".to_string(), "sticker kicker".to_string()]
    );
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P0).counter(counters::TICKET), 1);
    // Its own kicker paid: it triggers.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wicker Picker");
    t.lands(P0, "Forest", 3);
    t.lands(P0, "Swamp", 1);
    let badger = t.hand(P0, "Bog Badger");
    kick(&mut t, P0, &[true, false]);
    t.cast(P0, badger).go();
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert!(t.obj_now(badger).has_keyword(KeywordKind::Menace));
    assert_eq!(t.g.player(P0).counter(counters::TICKET), 0);
}

#[test]
fn sticker_kicker_gives_a_ticket_and_may_put_a_sticker_on_the_spell() {
    cr!("702.33h");
    ruling!(
        "Wicker Picker",
        "If you sticker kick a creature spell, you get {TK} and may choose a sticker to put on the creature spell."
    );
    ruling!(
        "Wicker Picker",
        "If you control multiple Wicker Pickers, then you can pay {1} for each of them, gaining {TK} and a sticker each time."
    );
    let mut t = TestGame::new(2);
    give_sheet(&mut t, P0);
    t.battlefield(P0, "Wicker Picker");
    t.lands(P0, "Forest", 3);
    let bears = t.hand(P0, "Grizzly Bears");
    kick(&mut t, P0, &[true]);
    t.answer_yes(P0, true);
    let spell = t.cast(P0, bears).go();
    // {1}{G} + {1}; the ticket it gave paid for the sticker.
    assert_eq!(untapped_lands(&t, P0), 0);
    assert!(stickers::is_stickered(&t.g, spell));
    assert_eq!(t.g.player(P0).counter(counters::TICKET), 0);
    t.resolve();
    assert_eq!(t.pt(bears), (5, 5));
    // Two Wicker Pickers: two sticker kicker costs; paying both gives two tickets.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wicker Picker");
    t.battlefield(P0, "Wicker Picker");
    t.lands(P0, "Forest", 4);
    let bears = t.hand(P0, "Grizzly Bears");
    kick(&mut t, P0, &[true, true]);
    t.cast(P0, bears).go();
    assert_eq!(
        optional_costs_offered(&t, P0),
        vec!["sticker kicker".to_string(), "sticker kicker".to_string()]
    );
    assert_eq!(untapped_lands(&t, P0), 0);
    assert_eq!(t.g.player(P0).counter(counters::TICKET), 2);
    // Noncreature spells don't have it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Wicker Picker");
    t.lands(P0, "Mountain", 3);
    let bolt = t.hand(P0, "Lightning Bolt");
    t.cast(P0, bolt).target(P1).go();
    assert!(optional_costs_offered(&t, P0).is_empty());
}

#[test]
fn targets_for_a_kicked_only_part_are_chosen_only_if_kicked() {
    cr!("702.33g");
    ruling!(
        "Orim's Thunder",
        "If Orim's Thunder is kicked, it's the source of damage dealt to the target creature, not the artifact or enchantment."
    );
    assert_supported("Jilt");
    assert_supported("Orim's Thunder");
    // Not kicked: Jilt needs only one target creature.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Island", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let jilt = t.hand(P0, "Jilt");
    let spell = t.cast(P0, jilt).target(bears).kicked(false).go();
    let chosen = &t.g.obj(spell).stack.as_ref().unwrap().chosen;
    assert_eq!(chosen[0].targets.iter().flatten().count(), 1);
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    // Kicked: a second target creature is chosen and dealt 2 damage by Jilt.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 3);
    t.lands(P0, "Island", 1);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let elves = t.battlefield(P1, "Llanowar Elves");
    let jilt = t.hand(P0, "Jilt");
    t.cast(P0, jilt)
        .target(bears)
        .target(elves)
        .kicked(true)
        .go();
    t.resolve();
    assert!(t.in_hand(P1, "Grizzly Bears"));
    assert!(t.in_graveyard(P1, "Llanowar Elves"));
    // Orim's Thunder: the spell deals the damage.
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 3);
    t.lands(P0, "Mountain", 1);
    let cover = t.battlefield(P1, "Cover of Darkness");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let thunder = t.hand(P0, "Orim's Thunder");
    let spell = t
        .cast(P0, thunder)
        .target(cover)
        .target(bears)
        .kicked(true)
        .go();
    t.resolve();
    assert!(!t.on_battlefield(cover));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    let damage_source = t.g.turn_events.iter().find_map(|e| match e {
        events::Event::Damage { source, .. } => Some(*source),
        _ => None,
    });
    assert_eq!(damage_source, Some(spell));
}

#[test]
fn abilities_can_check_that_a_permanent_wasnt_kicked() {
    cr!("702.33d");
    assert_supported("Skizzik");
    for kicked in [false, true] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Mountain", 5);
        let skizzik = t.hand(P0, "Skizzik");
        t.cast(P0, skizzik).kicked(kicked).go();
        t.resolve();
        assert!(t.on_battlefield(skizzik));
        // "At the beginning of the end step, if ~ wasn't kicked, sacrifice it."
        t.advance_to(P0, mtg_engine::turn::Step::End);
        t.resolve_all();
        assert_eq!(t.on_battlefield(skizzik), kicked, "kicked: {kicked}");
    }
}

#[test]
fn a_multikicker_ability_is_a_kicker_ability() {
    cr!("702.33c");
    assert_supported("Murasa Sproutling");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 5);
    let burst = t.graveyard(P0, "Burst Lightning");
    let pack = t.graveyard(P0, "Gnarlid Pack");
    let bolt = t.graveyard(P0, "Lightning Bolt");
    let sprout = t.hand(P0, "Murasa Sproutling");
    t.answer_targets(P0, &[Entity::Object(pack)]);
    t.cast(P0, sprout).kicked(true).go();
    t.resolve();
    // "Return target card with a kicker ability from your graveyard to your hand": cards
    // with kicker or multikicker, not others.
    let candidates: Vec<Entity> = t
        .asked()
        .into_iter()
        .rev()
        .find_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseTargets { candidates, .. } => Some(candidates),
            _ => None,
        })
        .unwrap();
    assert!(candidates.contains(&Entity::Object(burst)));
    assert!(candidates.contains(&Entity::Object(pack)));
    assert!(!candidates.contains(&Entity::Object(bolt)));
    t.resolve();
    assert!(t.in_hand(P0, "Gnarlid Pack"));
}
