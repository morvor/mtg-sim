//! Rulings batch P008 — burn that can hit planeswalkers, and costs: additional costs of
//! discarding X cards (CR 118.9, 107.3), cost reductions that reduce only generic mana
//! (CR 601.2f), costs locked in as the spell is cast (CR 601.2), mana value on the stack
//! (CR 202.3), the mana values of discarded cards (CR 202.3e, 702.35), copies (CR 707.10),
//! and "deals N damage to each of X targets" (not divided).

use crate::r_p007_common::*;
use crate::r_p008_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::can_cast;
use crate::r_s25_common::{cast_new, keep_copy_targets, lands_for_cost};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

const NORMAL: CastMethod = CastMethod::Normal;

/// Discards `cards` for Nahiri's Wrath (X = their number) and casts it with `method` at
/// `targets`. Returns the spell.
fn nahiris_wrath(
    t: &mut TestGame,
    cards: &[ObjectId],
    method: CastMethod,
    targets: &[Entity],
) -> ObjectId {
    supported("Nahiri's Wrath");
    if method == NORMAL {
        lands_for_cost(t, P0, "Nahiri's Wrath");
    }
    let card = t.hand(P0, "Nahiri's Wrath");
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(cards.iter().map(|c| obj(*c)).collect()),
    );
    t.answer_targets(P0, targets);
    if method == CastMethod::Free {
        t.answer(P0, DecisionKind::X, Answer::Number(cards.len() as i64));
        return free_cast(t, card);
    }
    t.cast(P0, card)
        .method(method)
        .x(cards.len() as i64)
        .try_go()
        .expect("Nahiri's Wrath")
}

/// P0 casts `card` without paying its mana cost, as a resolving effect would let them
/// (choices are answered from the queue).
fn free_cast(t: &mut TestGame, card: ObjectId) -> ObjectId {
    let s = mtg_engine::casting::cast_during_resolution(&mut t.g, P0, card, CastMethod::Free)
        .expect("cast without paying its mana cost");
    t.g.flush_events();
    s
}

#[test]
fn discard_x_spells_cast_without_paying_their_mana_cost_still_choose_x() {
    cr!("118.9", "107.3", "601.2b");
    ruling!(
        "Channeled Force",
        "If you cast Channeled Force without paying its mana cost, you still choose a value for X and discard that many cards. This is because the spell doesn’t have {X} in its mana cost."
    );
    ruling!(
        "Nahiri's Wrath",
        "If you cast Nahiri's Wrath without paying its mana cost, you'll still choose a value for X and discard X cards."
    );
    // Channeled Force, free, X = 2: two cards discarded, P1 draws two and the creature is
    // dealt 2.
    supported("Channeled Force");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    t.answer(
        P0,
        DecisionKind::Entities,
        Answer::Entities(vec![obj(a), obj(b)]),
    );
    let card = t.hand(P0, "Channeled Force");
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_targets(P0, &[obj(wurm)]);
    free_cast(&mut t, card);
    let hand = t.hand_size(P1);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears") && t.in_graveyard(P0, "Hill Giant"));
    assert_eq!((t.hand_size(P1), dmg(&t, wurm)), (hand + 2, 2));
    // Nahiri's Wrath, free: Grizzly Bears (2) and Hill Giant (4) discarded, 6 to each.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Indomitable Ancients");
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Hill Giant");
    nahiris_wrath(&mut t, &[a, b], CastMethod::Free, &[obj(target)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears") && t.in_graveyard(P0, "Hill Giant"));
    assert_eq!(dmg(&t, target), 6);
}

#[test]
fn nahiris_wrath_mana_values_of_the_discarded_cards() {
    cr!("202.3e", "107.3", "702.35a");
    ruling!(
        "Nahiri's Wrath",
        "If you discard a card with {X} in its mana cost, X is considered to be 0."
    );
    ruling!(
        "Nahiri's Wrath",
        "Nahiri's Wrath adds the mana values of the discarded cards in whatever zone they were put into as a result of being discarded, even if they weren't put into a graveyard (due to madness, for example) or are no longer in that zone."
    );
    ruling!(
        "Nahiri's Wrath",
        "The damage is not divided. Nahiri's Wrath will deal that much damage to each target."
    );
    // Fireball ({X}{R}): mana value 1.
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Indomitable Ancients");
    let fireball = t.hand(P0, "Fireball");
    nahiris_wrath(&mut t, &[fireball], NORMAL, &[obj(target)]);
    t.resolve_all();
    assert_eq!(dmg(&t, target), 1);
    // Fiery Temper (madness): exiled instead of put into the graveyard, then cast from
    // exile (so it's no longer there): its mana value 3 still counts. Two targets, each
    // dealt the full 3 + 2.
    for cast_it in [false, true] {
        let mut t = TestGame::new(2);
        let a = t.battlefield(P1, "Indomitable Ancients");
        let b = t.battlefield(P1, "Indomitable Ancients");
        let temper = t.hand(P0, "Fiery Temper");
        let bears = t.hand(P0, "Grizzly Bears");
        t.lands(P0, "Mountain", 1);
        nahiris_wrath(&mut t, &[temper, bears], NORMAL, &[obj(a), obj(b)]);
        // Fiery Temper's madness trigger: cast it (at P1) or not.
        t.answer_yes(P0, cast_it);
        t.answer_targets(P0, &[pl(P1)]);
        t.resolve_all();
        assert_eq!(
            (dmg(&t, a), dmg(&t, b)),
            (5, 5),
            "cast_it {cast_it}: 3 + 2 to each target"
        );
        if cast_it {
            assert_eq!(t.life(P1), 17, "Fiery Temper's madness cast");
        } else {
            assert!(t.in_exile("Fiery Temper") || t.in_graveyard(P0, "Fiery Temper"));
        }
    }
}

#[test]
fn nahiris_wrath_copy_has_the_same_targets_and_damage() {
    cr!("707.10", "707.2");
    ruling!(
        "Nahiri's Wrath",
        "If you copy Nahiri's Wrath, it has the same number of targets and deals the same amount of damage as the original spell."
    );
    supported("Twincast");
    let mut t = TestGame::new(2);
    let target = t.battlefield(P1, "Indomitable Ancients");
    let bears = t.hand(P0, "Grizzly Bears");
    let spell = nahiris_wrath(&mut t, &[bears], NORMAL, &[obj(target)]);
    cast_new(&mut t, P0, "Twincast", &[obj(spell)]);
    keep_copy_targets(&mut t, P0);
    t.resolve_all();
    assert_eq!(dmg(&t, target), 4, "2 from the copy and 2 from the original");
}

#[test]
fn cost_reductions_reduce_only_generic_mana() {
    cr!("601.2f", "118.7d");
    ruling!(
        "Chandra's Incinerator",
        "The cost reduction applies only to generic mana in the cost of Chandra's Incinerator. It can't reduce the {R} requirement."
    );
    ruling!(
        "Volcanic Salvo",
        "The cost reduction applies only to generic mana in the cost of Volcanic Salvo. It can't reduce the {R}{R} requirement."
    );
    ruling!(
        "Squash",
        "If you control more than one Giant, Squash’s cost is reduced by only {3}."
    );
    // Chandra's Incinerator after 10 noncombat damage to P1: {R} still due.
    supported("Chandra's Incinerator");
    let mut t = TestGame::new(2);
    let src = t.battlefield(P0, "Raging Goblin");
    hit_player(&mut t, src, P1, 10);
    let inc = t.hand(P0, "Chandra's Incinerator");
    t.lands(P0, "Wastes", 3);
    assert!(!can_cast(&mut t, P0, inc, NORMAL), "{{R}} can't be reduced");
    t.lands(P0, "Mountain", 1);
    assert!(can_cast(&mut t, P0, inc, NORMAL));
    // Volcanic Salvo with total power 20: {R}{R} still due.
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        t.battlefield(P0, "Krosan Colossus");
    }
    t.battlefield(P0, "Hill Giant");
    let salvo = t.hand(P0, "Volcanic Salvo");
    t.lands(P0, "Wastes", 4);
    t.lands(P0, "Mountain", 1);
    assert!(!can_cast(&mut t, P0, salvo, NORMAL));
    t.lands(P0, "Mountain", 1);
    assert!(can_cast(&mut t, P0, salvo, NORMAL));
    // Squash with two Giants: {1}{R}, not less.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Hill Giant");
    let squash = t.hand(P0, "Squash");
    t.lands(P0, "Mountain", 1);
    assert!(!can_cast(&mut t, P0, squash, NORMAL), "only {{3}} less");
    t.lands(P0, "Wastes", 1);
    assert!(can_cast(&mut t, P0, squash, NORMAL));
}

/// Whether a player other than `caster` was asked anything since `from`.
fn others_asked(t: &TestGame, from: usize, caster: PlayerId) -> bool {
    t.asked()[from..].iter().any(|(p, _)| *p != caster)
}

#[test]
fn costs_are_locked_in_while_the_spell_is_cast() {
    cr!("601.2", "601.2f", "601.2h");
    ruling!(
        "Squash",
        "Once you announce that you’re casting Squash, no player may take other actions until the spell’s been paid for. Notably, players can’t try to change the cost by removing your Giants."
    );
    ruling!(
        "Volcanic Salvo",
        "Once you begin to cast Volcanic Salvo, players can't take other actions until you've finished casting it. Notably, they can't try to change the total power of your creatures."
    );
    ruling!(
        "Corpse Explosion",
        "Once you have begun to cast Corpse Explosion, there is no point at which players can respond by removing cards from your graveyard to prevent you from being able to pay for it with a specific creature card."
    );
    ruling!(
        "Devour in Flames",
        "Once you announce that you’re casting Devour in Flames, it’s too late for anyone to interrupt you by trying to remove the land you’re returning."
    );
    // Squash with a Giant, paid with {1}{R}; nobody else is asked anything meanwhile.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Wastes", 1);
    t.lands(P0, "Mountain", 1);
    let squash = t.hand(P0, "Squash");
    let from = t.asked().len();
    t.cast(P0, squash).target(wurm).go();
    assert!(!others_asked(&t, from, P0));
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    // Volcanic Salvo with total power 10: {R}{R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Krosan Colossus");
    t.battlefield(P0, "Llanowar Elves");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Mountain", 2);
    let salvo = t.hand(P0, "Volcanic Salvo");
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(wurm)]);
    t.cast(P0, salvo).go();
    assert!(!others_asked(&t, from, P0));
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    // Corpse Explosion exiling Craw Wurm (power 6) from the graveyard.
    supported("Corpse Explosion");
    let mut t = TestGame::new(2);
    let corpse = t.graveyard(P0, "Craw Wurm");
    let giant = t.battlefield(P1, "Hill Giant");
    lands_for_cost(&mut t, P0, "Corpse Explosion");
    let ce = t.hand(P0, "Corpse Explosion");
    t.answer_choose(P0, &[obj(corpse)]);
    let from = t.asked().len();
    t.cast(P0, ce).go();
    assert!(!others_asked(&t, from, P0));
    assert_eq!(t.zone(corpse), Zone::Exile);
    t.resolve_all();
    assert!(!t.on_battlefield(giant));
    // Devour in Flames returning a land.
    supported("Devour in Flames");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let returned = t.battlefield(P0, "Forest");
    lands_for_cost(&mut t, P0, "Devour in Flames");
    let dif = t.hand(P0, "Devour in Flames");
    t.answer_choose(P0, &[obj(returned)]);
    let from = t.asked().len();
    t.cast(P0, dif).target(wurm).go();
    assert!(!others_asked(&t, from, P0));
    assert!(t.in_hand(P0, "Forest"));
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
}

#[test]
fn animists_might_mana_value_is_always_three() {
    cr!("202.3", "601.2f");
    ruling!(
        "Animist's Might",
        "The cost-reduction ability of Animist's Might doesn't change its mana cost or mana value, only the total cost you pay. Specifically, the mana value of Animist's Might is always 3."
    );
    supported("Animist's Might");
    let mut t = TestGame::new(2);
    let legend = t.battlefield(P0, "Isamaru, Hound of Konda");
    let wurm = t.battlefield(P1, "Indomitable Ancients");
    t.lands(P0, "Forest", 1);
    let card = t.hand(P0, "Animist's Might");
    assert_eq!(t.g.mana_value_of(card), 3);
    t.answer_targets(P0, &[obj(legend)]);
    t.answer_targets(P0, &[obj(wurm)]);
    let s = t.cast(P0, card).try_go().expect("cast for {G}");
    assert_eq!(t.g.mana_value_of(s), 3);
    t.resolve_all();
    assert_eq!(dmg(&t, wurm), 4);
}

#[test]
fn tundra_fumarole_without_snow_mana_adds_nothing() {
    cr!("106.3", "107.4h");
    ruling!(
        "Tundra Fumarole",
        "If you cast Tundra Fumarole without spending snow mana, you won’t add any mana. Damage will still be dealt."
    );
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_paying(
        &mut t,
        P0,
        "Tundra Fumarole",
        &["Wastes", "Mountain", "Mountain"],
        NORMAL,
        &[obj(wurm)],
    );
    t.resolve_all();
    assert!(!t.on_battlefield(wurm));
    assert_eq!(t.g.players[0].mana_pool.total(), 0);
    // With two snow mana: {C}{C}.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    cast_paying(
        &mut t,
        P0,
        "Tundra Fumarole",
        &["Wastes", "Snow-Covered Mountain", "Snow-Covered Mountain"],
        NORMAL,
        &[obj(wurm)],
    );
    t.resolve_all();
    assert_eq!(t.g.players[0].mana_pool.total(), 2);
}

#[test]
fn discard_then_draw_with_an_empty_hand() {
    cr!("701.9a", "603.12", "121.1");
    ruling!(
        "Fiery Encore",
        "If you have no cards in hand as Fiery Encore resolves, you won’t be able to discard a card, but you will draw a card. No damage will be dealt."
    );
    ruling!(
        "Torrent Sculptor // Flamethrower Sonata",
        "If you have no cards in hand as Flamethrower Sonata resolves, you won’t be able to discard a card, but you will draw a card. No damage will be dealt."
    );
    ruling!(
        "Cragganwick Cremator",
        "If you have no cards in hand when Cragganwick Cremator's ability resolves, you don't discard anything."
    );
    for (name, method) in [
        ("Fiery Encore", NORMAL),
        ("Torrent Sculptor // Flamethrower Sonata", CastMethod::Half(1)),
    ] {
        let mut t = TestGame::new(2);
        let wurm = t.battlefield(P1, "Craw Wurm");
        let lands: &[&str] = if name == "Fiery Encore" {
            &["Wastes", "Wastes", "Wastes", "Wastes", "Mountain"]
        } else {
            &["Wastes", "Mountain"]
        };
        let from = t.g.turn_events.len();
        t.answer_targets(P0, &[obj(wurm)]);
        cast_paying(&mut t, P0, name, lands, method, &[]);
        assert_eq!(t.hand_size(P0), 0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), 1, "{name}: drew a card");
        assert!(damage_events(&t, from).is_empty(), "{name}");
    }
    // Cragganwick Cremator with an empty hand: nothing discarded, no damage.
    let mut t = TestGame::new(2);
    let gy = t.graveyard_size(P0);
    t.answer_targets(P0, &[pl(P1)]);
    t.enter(P0, "Cragganwick Cremator");
    t.settle();
    t.resolve_all();
    assert_eq!((t.graveyard_size(P0), t.life(P1)), (gy, 20));
}

#[test]
fn into_the_fire_can_put_no_cards_on_the_bottom() {
    cr!("700.2", "608.2c");
    ruling!(
        "Into the Fire",
        "If you choose the second mode, you choose how many cards to put on the bottom of your library as Into the Fire resolves. You may choose to put no cards on the bottom of your library and just draw a card."
    );
    supported("Into the Fire");
    let mut t = TestGame::new(2);
    let keep = t.hand(P0, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Into the Fire");
    let card = t.hand(P0, "Into the Fire");
    let s = t.cast(P0, card).modes(&[1]).go();
    let _ = s;
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Entities, Answer::Entities(vec![]));
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(t.zone(keep), Zone::Hand(P0));
    assert_eq!(t.hand_size(P0), 2, "kept one, drew one");
    // The choice was made as it resolved.
    assert!(t.asked()[from..]
        .iter()
        .any(|(p, d)| *p == P0 && !matches!(d, Decision::Priority { .. })));
}
