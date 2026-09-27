//! CR 702.171 Saddle: Mounts whose abilities care whether they're saddled (CR 702.171b)
//! and which creatures saddled them (CR 702.171c).

use crate::common_k702_011_017::assert_supported;
use crate::common_k702_140_152::*;
use crate::common_k702_168_177::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kw::saddle::is_saddled;
use mtg_engine::mana::ManaType;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// Activates the saddle ability of `mount`, tapping `saddlers`, and resolves it.
fn saddle(t: &mut TestGame, mount: ObjectId, saddlers: &[ObjectId]) {
    let uid = ability_uid(t, mount, "Saddle");
    let es: Vec<Entity> = saddlers.iter().map(|c| Entity::Object(*c)).collect();
    t.answer_choose(P0, &es);
    activate_uid(t, P0, mount, uid).expect("saddle");
    t.resolve_all();
    assert!(is_saddled(&t.g, mount));
}

#[test]
fn rambling_possum_returns_creatures_that_saddled_it() {
    cr!("702.171b", "702.171c");
    assert_supported("Rambling Possum");
    // Rambling Possum: 3/3, saddle 1, "Whenever this creature attacks while saddled, it
    // gets +1/+2 until end of turn. Then you may return any number of creatures that
    // saddled it this turn to their owner's hand."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let possum = t.battlefield(P0, "Rambling Possum");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    let other = t.battlefield(P0, "Hill Giant");
    saddle(&mut t, possum, &[bears, elves]);
    declare_attackers(&mut t, &[(possum, Entity::Player(P1))]);
    // Only the Bears go back; the Hill Giant didn't saddle it.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(possum), (4, 5));
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert!(t.on_battlefield(elves) && t.on_battlefield(other));
}

#[test]
fn bounding_felidar_gains_life_for_each_of_those_creatures() {
    cr!("702.171b");
    assert_supported("Bounding Felidar");
    // Bounding Felidar: saddle 2, "Whenever this creature attacks while saddled, put a
    // +1/+1 counter on each other creature you control. You gain 1 life for each of those
    // creatures."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let felidar = t.battlefield(P0, "Bounding Felidar");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P1, "Hill Giant");
    saddle(&mut t, felidar, &[bears]);
    declare_attackers(&mut t, &[(felidar, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.counters(elves, "+1/+1"), 1);
    assert_eq!(t.counters(felidar, "+1/+1"), 0);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn caustic_bronco_makes_opponents_lose_the_life_if_saddled() {
    cr!("702.171b");
    assert_supported("Caustic Bronco");
    // Caustic Bronco: saddle 3, "Whenever this creature attacks, reveal the top card of
    // your library and put it into your hand. You lose life equal to that card's mana
    // value if this creature isn't saddled. Otherwise, each opponent loses that much
    // life."
    for saddled in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let bronco = t.battlefield(P0, "Caustic Bronco");
        if saddled {
            let giant = t.battlefield(P0, "Hill Giant");
            saddle(&mut t, bronco, &[giant]);
        }
        // Hill Giant: mana value 4.
        t.library_top(P0, "Hill Giant");
        let hand = t.hand_size(P0);
        declare_attackers(&mut t, &[(bronco, Entity::Player(P1))]);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1);
        assert!(t.in_hand(P0, "Hill Giant"));
        if saddled {
            assert_eq!((t.life(P0), t.life(P1)), (20, 16));
        } else {
            assert_eq!((t.life(P0), t.life(P1)), (16, 20));
        }
    }
}

#[test]
fn archmages_newt_grants_flashback_0_if_saddled() {
    cr!("702.171b");
    assert_supported("Archmage's Newt");
    // Archmage's Newt: saddle 3, "Whenever this creature deals combat damage to a player,
    // target instant or sorcery card in your graveyard gains flashback until end of turn.
    // The flashback cost is equal to its mana cost. That card gains flashback {0} until end
    // of turn instead if this creature is saddled."
    const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);
    for saddled in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::PrecombatMain);
        let newt = t.battlefield(P0, "Archmage's Newt");
        if saddled {
            let giant = t.battlefield(P0, "Hill Giant");
            saddle(&mut t, newt, &[giant]);
        }
        // Divination: {2}{U} sorcery, "Draw two cards."
        let div = t.graveyard(P0, "Divination");
        t.answer_targets(P0, &[Entity::Object(div)]);
        t.set_step(P0, Step::BeginningOfCombat);
        t.attack(&[(newt, Entity::Player(P1))], &[]);
        t.advance_to(P0, Step::PostcombatMain);
        let div = t.g.current(div);
        assert!(t.obj_now(div).has_keyword(KeywordKind::Flashback));
        // With no mana, only the saddled Newt's flashback {0} can be paid.
        assert_eq!(castable(&mut t, P0, div, FLASHBACK), saddled);
        add_mana(&mut t, P0, ManaType::U, 3);
        assert!(castable(&mut t, P0, div, FLASHBACK));
        let hand = t.hand_size(P0);
        t.cast(P0, div).method(FLASHBACK).go();
        assert_eq!(pool(&t, P0), if saddled { 3 } else { 0 });
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 2);
        assert!(t.in_exile("Divination"));
    }
}

#[test]
fn the_gitrog_sacrifices_a_creature_that_saddled_it() {
    cr!("702.171c");
    assert_supported("The Gitrog, Ravenous Ride");
    // The Gitrog, Ravenous Ride: 6/5 trample, haste, saddle 1, "Whenever The Gitrog deals
    // combat damage to a player, you may sacrifice a creature that saddled it this turn.
    // If you do, draw X cards, then put up to X land cards from your hand onto the
    // battlefield tapped, where X is the sacrificed creature's power."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let gitrog = t.battlefield(P0, "The Gitrog, Ravenous Ride");
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    saddle(&mut t, gitrog, &[giant]);
    let forests: Vec<ObjectId> = (0..4).map(|_| t.hand(P0, "Forest")).collect();
    let hand = t.hand_size(P0);
    // It may sacrifice only the Hill Giant (power 3), not the Bears.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.answer_choose(
        P0,
        &forests[..3]
            .iter()
            .map(|f| Entity::Object(*f))
            .collect::<Vec<_>>(),
    );
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(gitrog, Entity::Player(P1))], &[]);
    assert_eq!(t.life(P1), 14);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    assert!(t.on_battlefield(bears));
    // Drew 3, put 3 Forests onto the battlefield tapped.
    assert_eq!(t.hand_size(P0), hand + 3 - 3);
    let lands = t.named_on_battlefield("Forest");
    assert_eq!(lands.len(), 3);
    assert!(lands.iter().all(|l| t.obj(*l).tapped));
}

#[test]
fn fortune_flickers_itself_and_a_creature_that_saddled_it() {
    cr!("702.171c", "400.7");
    assert_supported("Fortune, Loyal Steed");
    // Fortune, Loyal Steed: 2/4, saddle 1, "Whenever Fortune attacks while saddled, at end
    // of combat, exile it and up to one creature that saddled it this turn, then return
    // those cards to the battlefield under their owner's control."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let fortune = t.battlefield(P0, "Fortune, Loyal Steed");
    let bears = t.battlefield(P0, "Grizzly Bears");
    saddle(&mut t, fortune, &[bears]);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(fortune, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    // Both came back as new, untapped objects; Fortune is no longer saddled.
    let f = t.named_on_battlefield("Fortune, Loyal Steed");
    let b = t.named_on_battlefield("Grizzly Bears");
    assert_eq!((f.len(), b.len()), (1, 1));
    assert!(f[0] != fortune && b[0] != bears);
    assert!(!t.obj(f[0]).tapped && !t.obj(b[0]).tapped);
    assert!(!is_saddled(&t.g, f[0]));
}

#[test]
fn calamity_copies_creatures_that_saddled_it_twice() {
    cr!("702.171c");
    assert_supported("Calamity, Galloping Inferno");
    ruling!(
        "Calamity, Galloping Inferno",
        "You don't have to choose the same creature both times for Calamity's triggered ability."
    );
    // Calamity, Galloping Inferno: 4/6 haste, saddle 1, "Whenever Calamity attacks while
    // saddled, choose a nonlegendary creature that saddled it this turn and create a
    // tapped and attacking token that's a copy of it. Sacrifice that token at the
    // beginning of the next end step. Repeat this process once."
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let calamity = t.battlefield(P0, "Calamity, Galloping Inferno");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    saddle(&mut t, calamity, &[bears, giant]);
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(giant)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(calamity, Entity::Player(P1))], &[]);
    // Calamity 4 + a Bears token 2 + a Hill Giant token 3.
    assert_eq!(t.life(P1), 11);
    let tokens: Vec<ObjectId> = t
        .g
        .battlefield
        .iter()
        .copied()
        .filter(|o| t.g.obj(*o).is_token())
        .collect();
    assert_eq!(tokens.len(), 2);
    // Both are sacrificed at the beginning of the end step.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.g.battlefield.iter().all(|o| !t.g.obj(*o).is_token()));
}
