//! Rulings batch P171 — costs and checks of assorted cards: Martyr of Ashes (reveal X red
//! cards), Lyzolda, the Blood Witch and Heartfire (sacrifice costs), Settle the Score,
//! Sarkhan's Dragonfire, Hotheaded Giant and Archon of Absolution.

use crate::r_p171_common::*;
use crate::r_s09_common::{declare, legal_attack, to_combat};
use crate::r_s21_common::castable;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::reveal::is_revealed;
use mtg_engine::testing::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------
// Martyr of Ashes: "{2}, Reveal X red cards from your hand, Sacrifice this creature: This
// creature deals X damage to each creature without flying."
// ---------------------------------------------------------------------------------

/// Activates Martyr of Ashes revealing `cards` (X = their number).
fn martyr(t: &mut TestGame, m: ObjectId, cards: &[ObjectId]) {
    t.lands(P0, "Wastes", 2);
    t.answer(P0, DecisionKind::X, Answer::Number(cards.len() as i64));
    let es: Vec<Entity> = cards.iter().map(|c| obj(*c)).collect();
    t.answer_choose(P0, &es);
    t.activate(P0, m, 0, &[]).expect("activate Martyr of Ashes");
}

#[test]
fn martyr_of_ashes_revealed_cards_stay_revealed_and_x_is_locked_in() {
    cr!("701.20a", "701.20c", "601.2h");
    ruling!(
        "Martyr of Ashes",
        "A card that's already revealed for another cost or because of an effect can be revealed to pay this cost."
    );
    ruling!(
        "Martyr of Ashes",
        "Each card that's revealed to pay the cost remains revealed until the ability leaves the stack."
    );
    ruling!(
        "Martyr of Ashes",
        "If one of the cards that's revealed to pay the cost leaves its owner's hand before the ability resolves, it stops being revealed, but the value of X is not affected."
    );
    supported("Martyr of Ashes");
    let mut t = TestGame::new(2);
    let m1 = t.battlefield(P0, "Martyr of Ashes");
    let m2 = t.battlefield(P0, "Martyr of Ashes");
    let bolt = t.hand(P0, "Lightning Bolt");
    let shock = t.hand(P0, "Shock");
    let wall = t.battlefield(P1, "Indomitable Ancients");
    // The first Martyr reveals both red cards (X = 2).
    martyr(&mut t, m1, &[bolt, shock]);
    assert!(is_revealed(&t.g, bolt) && is_revealed(&t.g, shock));
    // The second Martyr reveals the Bolt again while it's still revealed (X = 1).
    martyr(&mut t, m2, &[bolt]);
    assert_eq!(t.stack_len(), 2);
    // The second ability resolves: 1 damage. The cards stay revealed while the first
    // ability is on the stack.
    t.resolve();
    assert_eq!(damage_on(&t, wall), 1);
    assert!(is_revealed(&t.g, bolt) && is_revealed(&t.g, shock));
    // The Shock leaves P0's hand (cast at P1): it's no longer revealed, but X stays 2.
    t.lands(P0, "Mountain", 1);
    t.cast(P0, shock).target(P1).go();
    t.resolve();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.stack_len(), 1);
    t.resolve();
    assert_eq!(damage_on(&t, wall), 3);
    // The ability left the stack: the Bolt is no longer revealed.
    assert!(t.in_hand(P0, "Lightning Bolt"));
    assert!(!is_revealed(&t.g, bolt));
}

// ---------------------------------------------------------------------------------
// Lyzolda, the Blood Witch: "{2}, Sacrifice a creature: Lyzolda deals 2 damage to any
// target if the sacrificed creature was red. Draw a card if the sacrificed creature was
// black."
// ---------------------------------------------------------------------------------

/// Activates Lyzolda sacrificing `victim`, targeting `target`.
fn lyzolda(t: &mut TestGame, l: ObjectId, victim: ObjectId, target: Entity) {
    t.lands(P0, "Wastes", 2);
    t.answer_choose(P0, &[obj(victim)]);
    t.activate(P0, l, 0, &[target]).expect("activate Lyzolda");
}

#[test]
fn lyzolda_any_creature_can_be_sacrificed() {
    cr!("602.2b", "601.2h");
    ruling!(
        "Lyzolda, the Blood Witch",
        "You can sacrifice any creature you control to pay for this ability. If the creature was neither red nor black, the ability has no effect."
    );
    supported("Lyzolda, the Blood Witch");
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Lyzolda, the Blood Witch");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hand = t.hand_size(P0);
    lyzolda(&mut t, l, bears, Entity::Player(P1));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P0), hand);
    // A red creature: 2 damage.
    let goblin = t.battlefield(P0, "Raging Goblin");
    lyzolda(&mut t, l, goblin, Entity::Player(P1));
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.hand_size(P0), hand);
}

#[test]
fn lyzolda_needs_a_target_and_does_nothing_if_it_becomes_illegal() {
    cr!("602.2b", "608.2b");
    ruling!(
        "Lyzolda, the Blood Witch",
        "You must choose a target for the ability, even if the creature you sacrifice isn't red. If the target becomes illegal before the ability resolves, the ability won't resolve; you won't draw a card, even if the creature you sacrificed was black."
    );
    for target_dies in [false, true] {
        let mut t = TestGame::new(2);
        let l = t.battlefield(P0, "Lyzolda, the Blood Witch");
        let rats = t.battlefield(P0, "Typhoid Rats");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let hand = t.hand_size(P0);
        let from = t.asked().len();
        lyzolda(&mut t, l, rats, obj(bears));
        // A target was chosen even though the sacrificed creature (black) isn't red.
        assert!(t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
        if target_dies {
            cast_targeting(&mut t, P0, "Lightning Bolt", &[obj(bears)]);
            t.resolve();
        }
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + usize::from(!target_dies));
    }
}

// ---------------------------------------------------------------------------------
// Heartfire: "As an additional cost to cast this spell, sacrifice a creature or
// planeswalker. Heartfire deals 4 damage to any target."
// ---------------------------------------------------------------------------------

#[test]
fn heartfire_sacrifices_exactly_one_creature_or_planeswalker() {
    cr!("601.2b", "601.2h", "118.8");
    ruling!(
        "Heartfire",
        "You must sacrifice exactly one creature or planeswalker to cast this spell; you can’t cast it without sacrificing one, and you can’t sacrifice additional permanents."
    );
    supported("Heartfire");
    // Nothing to sacrifice: it can't be cast.
    let mut t = TestGame::new(2);
    give_mana_for(&mut t, P0, "Heartfire");
    let card = t.hand(P0, "Heartfire");
    assert!(!castable(&mut t, P0, card));
    // Two creatures and a planeswalker: exactly one is sacrificed (asking for both
    // sacrifices only one).
    t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let walker = t.battlefield(P0, "Ajani Goldmane");
    assert!(castable(&mut t, P0, card));
    t.answer_choose(P0, &[obj(walker), obj(giant)]);
    t.cast_with(P0, card, &[Entity::Player(P1)])
        .expect("cast Heartfire");
    let gone = [
        t.in_graveyard(P0, "Grizzly Bears"),
        t.in_graveyard(P0, "Hill Giant"),
        t.in_graveyard(P0, "Ajani Goldmane"),
    ];
    assert_eq!(gone.iter().filter(|g| **g).count(), 1, "{gone:?}");
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

// ---------------------------------------------------------------------------------
// Settle the Score: "Exile target creature. Put two loyalty counters on a planeswalker you
// control."
// ---------------------------------------------------------------------------------

#[test]
fn settle_the_score_chooses_the_planeswalker_as_it_resolves() {
    cr!("608.2c", "601.2c");
    ruling!(
        "Settle the Score",
        "You don’t choose which planeswalker receives loyalty counters until Settle the Score resolves. If you don’t control a planeswalker, you’ll simply exile the target creature and not put loyalty counters on anything."
    );
    supported("Settle the Score");
    // No planeswalker: the creature is simply exiled.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_targeting(&mut t, P0, "Settle the Score", &[obj(bears)]);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    // Two planeswalkers: no choice as it's cast; the second one is chosen as it
    // resolves.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ajani Goldmane");
    let b = t.battlefield(P0, "Garruk Wildspeaker");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    cast_targeting(&mut t, P0, "Settle the Score", &[obj(bears)]);
    let chose = |t: &TestGame, from: usize| {
        t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseEntities { .. }))
            .count()
    };
    assert_eq!(chose(&t, from), 0);
    let mid = t.asked().len();
    t.answer_choose(P0, &[obj(b)]);
    t.resolve_all();
    assert_eq!(chose(&t, mid), 1);
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(t.counters(a, "loyalty"), 4);
    assert_eq!(t.counters(b, "loyalty"), 5);
}

// ---------------------------------------------------------------------------------
// Sarkhan's Dragonfire, Hotheaded Giant, Archon of Absolution.
// ---------------------------------------------------------------------------------

#[test]
fn sarkhans_dragonfire_with_an_illegal_target_doesnt_look_at_the_library() {
    cr!("608.2b");
    ruling!(
        "Sarkhan's Dragonfire",
        "If the chosen target is an illegal target when Sarkhan’s Dragonfire tries to resolve, the spell doesn’t resolve. You won’t look at the top five cards of your library."
    );
    supported("Sarkhan's Dragonfire");
    for fizzle in [false, true] {
        let mut t = TestGame::new(2);
        let top = t.library_top(P0, "Lightning Bolt");
        let bears = t.battlefield(P1, "Grizzly Bears");
        cast_targeting(&mut t, P0, "Sarkhan's Dragonfire", &[obj(bears)]);
        if fizzle {
            cast_targeting(&mut t, P0, "Shock", &[obj(bears)]);
            t.resolve();
        }
        t.answer_choose(P0, &[obj(top)]);
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Grizzly Bears"));
        // Resolving: the Bolt is put into P0's hand; fizzling: it stays on top.
        assert_eq!(t.in_hand(P0, "Lightning Bolt"), !fizzle);
        assert_eq!(t.g.player(P0).library.last() == Some(&top), fizzle);
    }
}

#[test]
fn hotheaded_giant_checks_as_it_enters_even_if_it_wasnt_cast() {
    cr!("614.1c", "614.12");
    ruling!(
        "Hotheaded Giant",
        "Although Hotheaded Giant says it looks for \"another red spell,\" there's no requirement that Hotheaded Giant actually be cast as a spell (or be red) for this part of its ability to work."
    );
    ruling!(
        "Hotheaded Giant",
        "Hotheaded Giant doesn't check whether you've cast another red spell until Hotheaded Giant enters."
    );
    supported("Hotheaded Giant");
    // Put onto the battlefield without being cast: no red spell this turn → counters.
    let mut t = TestGame::new(2);
    let g = t.enter(P0, "Hotheaded Giant");
    assert_eq!(t.counters(g, "-1/-1"), 2);
    // After a red spell this turn, it enters without them.
    let mut t = TestGame::new(2);
    cast_targeting(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    let g = t.enter(P0, "Hotheaded Giant");
    assert_eq!(t.counters(g, "-1/-1"), 0);
    // Cast as the first red spell of the turn, with Shock cast while it's on the stack:
    // it checks only as it enters, so it enters without counters.
    let mut t = TestGame::new(2);
    let giant = cast_card(&mut t, P0, "Hotheaded Giant");
    cast_targeting(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.resolve_all();
    let g = t.g.current(giant);
    assert!(t.on_battlefield(g));
    assert_eq!(t.counters(g, "-1/-1"), 0);
}

#[test]
fn archon_of_absolution_attacks_if_able_need_not_pay() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Archon of Absolution",
        "Your opponents can choose not to pay to attack with a creature that attacks “if able.” If there's no other player or planeswalker to attack, that creature simply doesn't attack."
    );
    supported("Archon of Absolution");
    supported("Juggernaut");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Archon of Absolution");
    let jugg = t.battlefield(P1, "Juggernaut");
    t.lands(P1, "Wastes", 2);
    to_combat(&mut t, P1);
    assert!(legal_attack(&mut t, &[]));
    assert!(declare(&mut t, P1, &[]).is_empty());
    assert!(!t.obj_now(jugg).tapped);
    // With another opponent to attack for free, it must attack that player.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Archon of Absolution");
    let jugg = t.battlefield(P1, "Juggernaut");
    to_combat(&mut t, P1);
    assert!(!legal_attack(&mut t, &[]));
    assert_eq!(declare(&mut t, P1, &[]), vec![(jugg, Entity::Player(P2))]);
}
