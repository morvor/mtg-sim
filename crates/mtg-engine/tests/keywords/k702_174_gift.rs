//! CR 702.174 Gift.

use crate::common_k702_011_017::{assert_supported, custom_card};
use crate::common_k702_140_152::*;
use crate::common_k702_168_177::*;
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

/// Casts `card` for P0 promising (or not) its gift; `targets` are answered in order.
fn cast_gift(t: &mut TestGame, card: ObjectId, promise: bool, targets: &[Entity]) -> ObjectId {
    pay_optional(t, P0, promise);
    for e in targets {
        t.answer_targets(P0, &[*e]);
    }
    t.cast(P0, card).go()
}

/// Treasure tokens `p` controls.
fn treasures(t: &TestGame, p: PlayerId) -> usize {
    tokens_of_subtype(t, p, "Treasure").len()
}

#[test]
fn a_promised_gift_is_given_before_the_spells_other_effects() {
    cr!("702.174", "702.174a", "702.174b", "702.174h", "702.174j");
    assert_supported("Blooming Blast");
    // Blooming Blast: {1}{R} instant, gift a Treasure, "Blooming Blast deals 2 damage to
    // target creature. If the gift was promised, Blooming Blast also deals 3 damage to
    // that creature's controller."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blast = t.hand(P0, "Blooming Blast");
    add_mana(&mut t, P0, ManaType::R, 2);
    cast_gift(&mut t, blast, true, &[Entity::Object(bears)]);
    // Choosing an opponent is an additional cost, announced as it's cast: nothing is
    // given yet.
    assert_eq!(optional_costs_asked(&t, P0), vec!["gift".to_string()]);
    assert_eq!(treasures(&t, P1), 0);
    t.resolve_all();
    // The chosen opponent created a Treasure, before the damage was dealt.
    assert_eq!(treasures(&t, P1), 1);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 17);
    let events = &t.g.turn_events;
    let token = events
        .iter()
        .position(|e| matches!(e, Event::TokenCreated { .. }))
        .unwrap();
    let damage = events
        .iter()
        .position(|e| matches!(e, Event::Damage { .. }))
        .unwrap();
    assert!(token < damage);
    // Not promised: no Treasure, and no damage to the controller.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blast = t.hand(P0, "Blooming Blast");
    add_mana(&mut t, P0, ManaType::R, 2);
    cast_gift(&mut t, blast, false, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(treasures(&t, P1), 0);
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn gift_a_card_and_alternative_targets_if_the_gift_was_promised() {
    cr!("702.174e", "702.174m");
    assert_supported("Wear Down");
    ruling!(
        "Wear Down",
        "Some instant or sorcery spells require alternative or additional targets if the gift was promised. You ignore these targeting requirements if the gifts aren’t promised for those spells."
    );
    // Wear Down: {1}{G} sorcery, gift a card, "Destroy target artifact or enchantment. If
    // the gift was promised, instead destroy two target artifacts and/or enchantments."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Ornithopter");
    let b = t.battlefield(P1, "Glorious Anthem");
    let wear = t.hand(P0, "Wear Down");
    add_mana(&mut t, P0, ManaType::G, 2);
    let hand = t.hand_size(P1);
    cast_gift(&mut t, wear, true, &[Entity::Object(a), Entity::Object(b)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert!(t.in_graveyard(P1, "Glorious Anthem"));
    // Not promised: one target, and no card.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Ornithopter");
    t.battlefield(P1, "Glorious Anthem");
    let wear = t.hand(P0, "Wear Down");
    add_mana(&mut t, P0, ManaType::G, 2);
    let hand = t.hand_size(P1);
    let spell = cast_gift(&mut t, wear, false, &[Entity::Object(a)]);
    let chosen = &t.obj(spell).stack.as_ref().unwrap().chosen;
    let n: usize = chosen
        .iter()
        .flat_map(|c| c.targets.iter())
        .map(Vec::len)
        .sum();
    assert_eq!(n, 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand);
    assert!(t.in_graveyard(P1, "Ornithopter"));
    assert_eq!(t.named_on_battlefield("Glorious Anthem").len(), 1);
}

#[test]
fn gift_a_tapped_fish_and_additional_targets_only_if_promised() {
    cr!("702.174f", "702.174m");
    assert_supported("Mind Spiral");
    // Mind Spiral: {4}{U} sorcery, gift a tapped Fish, "Target player draws three cards.
    // If the gift was promised, tap target creature an opponent controls and put a stun
    // counter on it."
    let mut t = TestGame::new(2);
    let spiral = t.hand(P0, "Mind Spiral");
    add_mana(&mut t, P0, ManaType::U, 5);
    // No creature to target: it can still be cast without promising the gift.
    let hand = t.hand_size(P0);
    cast_gift(&mut t, spiral, false, &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 3);
    assert!(tokens_named(&t, P1, "Fish Token").is_empty());
    // Promised: the opponent's creature is targeted, and they get a tapped 1/1 Fish.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let spiral = t.hand(P0, "Mind Spiral");
    add_mana(&mut t, P0, ManaType::U, 5);
    cast_gift(
        &mut t,
        spiral,
        true,
        &[Entity::Player(P0), Entity::Object(giant)],
    );
    t.resolve_all();
    assert!(t.obj(giant).tapped);
    assert_eq!(t.counters(giant, "stun"), 1);
    let fish: Vec<ObjectId> =
        t.g.battlefield
            .iter()
            .copied()
            .filter(|id| t.obj(*id).is_token() && t.obj(*id).chars.has_subtype("Fish"))
            .collect();
    assert_eq!(fish.len(), 1);
    let f = t.obj(fish[0]);
    assert_eq!(f.controller, P1);
    assert!(f.tapped);
    assert_eq!((f.power(), f.toughness()), (1, 1));
    assert!(f.chars.colors.contains(mtg_engine::types::Color::Blue));
}

#[test]
fn gift_a_food() {
    cr!("702.174d");
    assert_supported("Crumb and Get It");
    // Crumb and Get It: {W} instant, gift a Food, "Target creature you control gets +2/+2
    // until end of turn. If the gift was promised, that creature also gains indestructible
    // until end of turn."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let crumb = t.hand(P0, "Crumb and Get It");
    add_mana(&mut t, P0, ManaType::W, 1);
    cast_gift(&mut t, crumb, true, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(tokens_of_subtype(&t, P1, "Food").len(), 1);
    assert_eq!(t.pt(bears), (4, 4));
    assert!(t.obj(bears).chars.has_keyword(KeywordKind::Indestructible));
}

#[test]
fn gift_an_extra_turn() {
    cr!("702.174g");
    let mut def = custom_card(
        "Generous Offer",
        "Instant",
        None,
        "Gift an extra turn\nDraw a card.",
    );
    def.faces[0].chars.mana_cost = Some(mtg_engine::mana::ManaCost::generic(0));
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let card = t.custom(P0, def, Zone::Hand(P0));
    cast_gift(&mut t, card, true, &[]);
    t.resolve_all();
    // The opponent takes an extra turn after this one, then P0's turn would normally come
    // after theirs: P1 takes two turns in a row.
    assert_eq!(t.g.extra_turns, vec![P1]);
    let turn = t.g.turn.number;
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.g.turn.number, turn + 1);
    t.advance_to(P1, Step::End);
    t.advance_to_step(Step::Upkeep);
    assert_eq!(t.g.turn.active, P1);
    assert_eq!(t.g.turn.number, turn + 2);
}

#[test]
fn a_permanents_gift_is_given_when_it_enters() {
    cr!("702.174b", "702.174i");
    // Octomancer: gift an Octopus: "When it enters, they create an 8/8 blue Octopus
    // creature token."
    let mut t = TestGame::new(2);
    let octo = t.hand(P0, "Octomancer");
    add_mana(&mut t, P0, ManaType::G, 4);
    add_mana(&mut t, P0, ManaType::U, 1);
    cast_gift(&mut t, octo, true, &[]);
    t.resolve();
    // The permanent entered; its gift ability triggered.
    assert!(t.on_battlefield(octo));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    let octopus: Vec<ObjectId> =
        t.g.battlefield
            .iter()
            .copied()
            .filter(|id| t.obj(*id).is_token() && t.obj(*id).chars.has_subtype("Octopus"))
            .collect();
    assert_eq!(octopus.len(), 1);
    assert_eq!(t.obj(octopus[0]).controller, P1);
    assert_eq!(t.pt(octopus[0]), (8, 8));
    assert!(t.obj(octopus[0]).chars.is(CardType::Creature));
    // Not promised: nothing triggers.
    let mut t = TestGame::new(2);
    let octo = t.hand(P0, "Octomancer");
    add_mana(&mut t, P0, ManaType::G, 4);
    add_mana(&mut t, P0, ManaType::U, 1);
    cast_gift(&mut t, octo, false, &[]);
    t.resolve();
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn a_permanents_other_abilities_can_check_whether_the_gift_was_promised() {
    cr!("702.174b", "702.174k");
    assert_supported("Scrapshooter");
    ruling!(
        "Into the Flood Maw",
        "On the other hand, you can promise a gift for a permanent spell even if you won’t be able to choose targets for an enters ability of that permanent once the spell resolves."
    );
    // Scrapshooter: gift a card, "When this creature enters, if the gift was promised,
    // destroy target artifact or enchantment an opponent controls."
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    let scrap = t.hand(P0, "Scrapshooter");
    add_mana(&mut t, P0, ManaType::G, 3);
    let hand = t.hand_size(P1);
    cast_gift(&mut t, scrap, true, &[]);
    t.answer_targets(P0, &[Entity::Object(thopter)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
    assert!(t.in_graveyard(P1, "Ornithopter"));
    // Promised with nothing to target: the gift is still given.
    let mut t = TestGame::new(2);
    let scrap = t.hand(P0, "Scrapshooter");
    add_mana(&mut t, P0, ManaType::G, 3);
    let hand = t.hand_size(P1);
    cast_gift(&mut t, scrap, true, &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
    // Not promised: neither ability does anything.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ornithopter");
    let scrap = t.hand(P0, "Scrapshooter");
    add_mana(&mut t, P0, ManaType::G, 3);
    cast_gift(&mut t, scrap, false, &[]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
}

#[test]
fn if_the_gift_wasnt_promised() {
    cr!("702.174k");
    assert_supported("Nocturnal Hunger");
    // Nocturnal Hunger: gift a Food, "Destroy target creature. If the gift wasn't
    // promised, you lose 2 life."
    for promise in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let hunger = t.hand(P0, "Nocturnal Hunger");
        add_mana(&mut t, P0, ManaType::B, 3);
        cast_gift(&mut t, hunger, promise, &[Entity::Object(bears)]);
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Grizzly Bears"));
        assert_eq!(t.life(P0), if promise { 20 } else { 18 });
        assert_eq!(tokens_of_subtype(&t, P1, "Food").len(), promise as usize);
    }
}

#[test]
fn whenever_you_give_a_gift() {
    cr!("702.174c");
    assert_supported("Jolly Gerbils");
    // Jolly Gerbils: "Whenever you give a gift, draw a card."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Jolly Gerbils");
    let bears = t.battlefield(P1, "Grizzly Bears");
    // An instant whose gift cost was paid resolves.
    let blast = t.hand(P0, "Blooming Blast");
    add_mana(&mut t, P0, ManaType::R, 2);
    let hand = t.hand_size(P0);
    cast_gift(&mut t, blast, true, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 1);
    // The gift triggered ability of a permanent resolves.
    let scrap = t.hand(P0, "Scrapshooter");
    add_mana(&mut t, P0, ManaType::G, 3);
    let hand = t.hand_size(P0);
    cast_gift(&mut t, scrap, true, &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1 + 1);
    // Without a promise, no gift is given.
    let blast = t.hand(P0, "Blooming Blast");
    let bears = t.battlefield(P1, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::R, 2);
    let hand = t.hand_size(P0);
    cast_gift(&mut t, blast, false, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand - 1);
    // An opponent's gift isn't yours.
    let blast = t.hand(P1, "Blooming Blast");
    let bears = t.battlefield(P0, "Grizzly Bears");
    add_mana(&mut t, P1, ManaType::R, 2);
    let hand = t.hand_size(P0);
    pay_optional(&mut t, P1, true);
    t.cast(P1, blast).target(bears).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn a_countered_spell_gives_no_gift() {
    cr!("702.174j");
    ruling!(
        "Wear Down",
        "If a spell for which the gift was promised is countered, doesn’t resolve (perhaps because all of its targets are illegal), or is otherwise removed from the stack, the gift won’t be given."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blast = t.hand(P0, "Blooming Blast");
    add_mana(&mut t, P0, ManaType::R, 2);
    let spell = cast_gift(&mut t, blast, true, &[Entity::Object(bears)]);
    let counter = t.hand(P1, "Counterspell");
    add_mana(&mut t, P1, ManaType::U, 2);
    t.cast(P1, counter).target(spell).go();
    t.resolve_all();
    assert_eq!(treasures(&t, P1), 0);
    // All of its targets illegal: it doesn't resolve.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blast = t.hand(P0, "Blooming Blast");
    add_mana(&mut t, P0, ManaType::R, 2);
    cast_gift(&mut t, blast, true, &[Entity::Object(bears)]);
    crate::common_k702_052_066::destroy(&mut t, bears);
    t.resolve_all();
    assert_eq!(treasures(&t, P1), 0);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn the_gift_is_promised_to_one_chosen_opponent() {
    cr!("702.174a");
    ruling!(
        "Wear Down",
        "As an additional cost to cast a spell with gift, you can promise the listed gift to an opponent. That opponent is chosen as part of that additional cost."
    );
    ruling!("Wear Down", "You can’t pay a gift cost more than once.");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blast = t.hand(P0, "Blooming Blast");
    add_mana(&mut t, P0, ManaType::R, 2);
    // Promise it to P2, not the controller of the target.
    t.answer_choose(P0, &[Entity::Player(P2)]);
    cast_gift(&mut t, blast, true, &[Entity::Object(bears)]);
    assert_eq!(optional_costs_asked(&t, P0).len(), 1);
    t.resolve_all();
    assert_eq!(treasures(&t, P2), 1);
    assert_eq!(treasures(&t, P1), 0);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn a_copy_of_a_permanent_whose_gift_was_promised_has_no_promise() {
    cr!("702.174b");
    ruling!(
        "Wear Down",
        "If a card or token enters as a copy of a permanent that’s already on the battlefield, the gift isn’t promised for that new permanent, even if it was promised for the original."
    );
    let mut t = TestGame::new(2);
    let scrap = t.hand(P0, "Scrapshooter");
    add_mana(&mut t, P0, ManaType::G, 3);
    let hand = t.hand_size(P1);
    cast_gift(&mut t, scrap, true, &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), hand + 1);
    let original = t.g.current(scrap);
    let clone = t.hand(P0, "Clone");
    add_mana(&mut t, P0, ManaType::U, 4);
    t.answer_choose(P0, &[Entity::Object(original)]);
    t.cast(P0, clone).go();
    t.resolve_all();
    assert_eq!(t.obj(t.g.current(clone)).chars.name, "Scrapshooter");
    assert_eq!(t.hand_size(P1), hand + 1);
}
