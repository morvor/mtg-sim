//! Rulings batch P213 — escalate (CR 702.120), escape (CR 702.138), eternalize
//! (CR 702.129), evoke (CR 702.74) and evolve (CR 702.100).

use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, can_cast, destroy};
use crate::r_s08_common::{legal_cast_methods, mana_value};
use crate::r_s24_common::enter_together;
use mtg_engine::ability::Duration;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const ESCAPE: CastMethod = CastMethod::Keyword(KeywordKind::Escape);
const EVOKE: CastMethod = CastMethod::Keyword(KeywordKind::Evoke);

fn graveyard_n(t: &mut TestGame, p: PlayerId, name: &str, n: usize) -> Vec<ObjectId> {
    (0..n).map(|_| t.graveyard(p, name)).collect()
}

// ---------------------------------------------------------------------------------------
// Escalate
// ---------------------------------------------------------------------------------------

#[test]
fn collective_resistance_escalate_is_paid_when_cast_for_free() {
    cr!("702.120a", "118.9");
    ruling!(
        "Collective Resistance",
        "If an effect allows you to cast Collective Resistance without paying its mana cost, you'll still have to pay the escalate costs if you choose more than one mode."
    );
    supported("Collective Resistance");
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P1, "Mind Stone");
    let anthem = t.battlefield(P1, "Glorious Anthem");
    t.lands(P0, "Forest", 1);
    let c = t.hand(P0, "Collective Resistance");
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![c],
        Duration::EndOfTurn,
        true,
        None,
    );
    t.cast(P0, c)
        .method(CastMethod::Free)
        .modes(&[0, 1])
        .target(stone)
        .target(anthem)
        .go();
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 0);
    t.resolve_all();
    assert!(!t.on_battlefield(stone));
    assert!(!t.on_battlefield(anthem));
}

// ---------------------------------------------------------------------------------------
// Escape
// ---------------------------------------------------------------------------------------

#[test]
fn escaped_spells_return_to_the_graveyard_and_can_escape_again() {
    cr!("702.138a", "608.2n", "400.7");
    ruling!(
        "From the Catacombs",
        "After From the Catacombs resolves, it returns to your graveyard. From there, you can use the escape ability to cast it again. Of course, you'll need five more cards to exile."
    );
    ruling!(
        "Cling to Dust",
        "After an escaped spell resolves, it returns to its owner’s graveyard if it’s not a permanent spell. If it is a permanent spell, it enters the battlefield and will return to its owner’s graveyard if it dies later."
    );
    supported("From the Catacombs");
    supported("Cling to Dust");
    // From the Catacombs: "Put target creature card from a graveyard onto the battlefield
    // under your control with a corpse counter on it. ..."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 10);
    let catacombs = t.graveyard(P0, "From the Catacombs");
    let bears = t.graveyard(P1, "Grizzly Bears");
    graveyard_n(&mut t, P0, "Forest", 5);
    t.cast(P0, catacombs).method(ESCAPE).target(bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "From the Catacombs"));
    let catacombs = t.g.current(catacombs);
    let giant = t.graveyard(P1, "Hill Giant");
    assert!(!can_cast(&mut t, P0, catacombs, ESCAPE));
    graveyard_n(&mut t, P0, "Forest", 5);
    assert!(can_cast(&mut t, P0, catacombs, ESCAPE));
    t.cast(P0, catacombs).method(ESCAPE).target(giant).go();
    t.resolve_all();
    assert!(t.on_battlefield(giant));
    // Cling to Dust: "Exile target card from a graveyard. If it was a creature card, you
    // gain 3 life. Otherwise, you draw a card."
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let cling = t.graveyard(P0, "Cling to Dust");
    let bears = t.graveyard(P1, "Grizzly Bears");
    graveyard_n(&mut t, P0, "Forest", 5);
    t.cast(P0, cling).method(ESCAPE).target(bears).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 23);
    assert!(t.in_graveyard(P0, "Cling to Dust"));
}

#[test]
fn escaped_permanents_die_to_the_graveyard_and_can_escape_again() {
    cr!("702.138a", "400.7");
    ruling!(
        "Skyway Robber",
        "After an escaped spell resolves, it returns to its owner's graveyard if it's not a permanent spell. If it is a permanent spell, it enters the battlefield and will return to its owner's graveyard if it dies later."
    );
    ruling!(
        "Tizerus Charger",
        "When Tizerus Charger escapes, it enters the battlefield and will return to its owner's graveyard if it dies later. From there, it can escape again."
    );
    for (name, cost) in [("Skyway Robber", 4), ("Tizerus Charger", 5)] {
        supported(name);
        let mut t = TestGame::new(2);
        t.lands(P0, "Island", 2 * cost);
        t.lands(P0, "Swamp", 2 * cost);
        let c = t.graveyard(P0, name);
        graveyard_n(&mut t, P0, "Forest", 5);
        t.cast(P0, c).method(ESCAPE).go();
        t.resolve_all();
        assert!(t.on_battlefield(c), "{name}");
        destroy(&mut t, c);
        assert!(t.in_graveyard(P0, name), "{name}");
        graveyard_n(&mut t, P0, "Forest", 5);
        let c = t.g.current(c);
        assert!(can_cast(&mut t, P0, c, ESCAPE), "{name}");
    }
}

#[test]
fn cling_to_dust_can_target_a_card_it_exiles_to_pay_for_escape() {
    cr!("601.2c", "601.2h", "608.2b");
    ruling!(
        "Cling to Dust",
        "Cling to Dust can target one of the five cards you’ll exile to pay its escape cost. In this case, the spell won’t resolve."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let cling = t.graveyard(P0, "Cling to Dust");
    let bears = graveyard_n(&mut t, P0, "Grizzly Bears", 5);
    let hand = t.hand_size(P0);
    t.cast(P0, cling).method(ESCAPE).target(bears[0]).go();
    assert_eq!(t.zone(bears[0]), Zone::Exile);
    t.resolve_all();
    // Its only target is illegal: it doesn't resolve (no life, no card).
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.hand_size(P0), hand);
    assert!(t.in_graveyard(P0, "Cling to Dust"));
}

#[test]
fn cling_to_dust_escape_keeps_its_instant_timing() {
    cr!("702.138a", "117.1a");
    ruling!(
        "Cling to Dust",
        "Escape’s permission doesn’t change when you may cast the spell from your graveyard."
    );
    // An instant: escaped during an opponent's turn.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let cling = t.graveyard(P0, "Cling to Dust");
    graveyard_n(&mut t, P0, "Forest", 5);
    t.set_step(P1, Step::Upkeep);
    assert!(can_cast(&mut t, P0, cling, ESCAPE));
    // Not from the hand.
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let cling = t.hand(P0, "Cling to Dust");
    graveyard_n(&mut t, P0, "Forest", 5);
    assert!(!can_cast(&mut t, P0, cling, ESCAPE));
}

#[test]
fn cling_to_dust_put_into_the_graveyard_can_be_escaped_right_away() {
    cr!("117.3b", "702.138a");
    ruling!(
        "Cling to Dust",
        "If a card with escape is put into your graveyard during your turn, you’ll be able to cast it right away if it’s legal to do so, before an opponent can take any actions."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    graveyard_n(&mut t, P0, "Forest", 5);
    let bears = t.graveyard(P1, "Grizzly Bears");
    let cling = t.hand(P0, "Cling to Dust");
    t.cast(P0, cling).target(bears).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Cling to Dust"));
    // The active player receives priority after the spell resolves.
    assert_eq!(t.g.turn.priority, Some(P0));
    let cling = t.g.current(cling);
    assert!(can_cast(&mut t, P0, cling, ESCAPE));
}

#[test]
fn escape_is_the_only_alternative_cost_and_total_cost_includes_increases() {
    cr!("118.9a", "601.2f", "202.3", "702.138a");
    ruling!(
        "Cling to Dust",
        "If you cast a spell with its escape permission, you can’t choose to apply any other alternative costs or to cast it without paying its mana cost. If it has any additional costs, you must pay those."
    );
    ruling!(
        "Cling to Dust",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you’re paying (such as an escape cost), add any cost increases, then apply any cost reductions. The mana value of the spell remains unchanged, no matter what the total cost to cast it was and no matter whether an alternative cost was paid."
    );
    supported("Sphere of Resistance");
    // From the graveyard, escape is the only way to cast it (not for free, not normally).
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 5);
    let cling = t.graveyard(P0, "Cling to Dust");
    graveyard_n(&mut t, P0, "Forest", 5);
    assert_eq!(legal_cast_methods(&mut t, P0, cling), vec![ESCAPE]);
    // Sphere of Resistance ("Spells cost {1} more to cast."): {3}{B} + {1}.
    t.battlefield(P1, "Sphere of Resistance");
    let bears = t.graveyard(P1, "Grizzly Bears");
    let spell = t.cast(P0, cling).method(ESCAPE).target(bears).go();
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 0);
    assert_eq!(mana_value(&t, spell), 1);
    // With one land fewer, it can't be escaped.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    t.lands(P0, "Swamp", 4);
    let cling = t.graveyard(P0, "Cling to Dust");
    t.graveyard(P1, "Grizzly Bears");
    graveyard_n(&mut t, P0, "Forest", 5);
    assert!(!can_cast(&mut t, P0, cling, ESCAPE));
}

#[test]
fn cling_to_dust_is_on_the_stack_while_it_is_being_cast() {
    cr!("601.2a", "601.2c");
    ruling!(
        "Cling to Dust",
        "Once you begin casting a spell with escape, it immediately moves to the stack. Players can’t take any other actions until you’re done casting the spell."
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Swamp", 4);
    let cling = t.graveyard(P0, "Cling to Dust");
    graveyard_n(&mut t, P0, "Forest", 5);
    let bears = t.graveyard(P1, "Grizzly Bears");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, mtg_engine::decision::Decision::ChooseTargets { .. }),
        |g| {
            g.stack
                .iter()
                .map(|id| g.obj(*id).chars.name.to_string())
                .collect::<Vec<_>>()
        },
    );
    let from = t.asked().len();
    t.cast(P0, cling).method(ESCAPE).target(bears).go();
    assert_eq!(
        seen.lock().unwrap().first().cloned(),
        Some(vec!["Cling to Dust".to_string()])
    );
    // No one else was asked anything during the casting.
    assert!(t.asked()[from..].iter().all(|(p, _)| *p == P0));
}

#[test]
fn titans_are_sacrificed_unless_they_escaped_but_their_enters_ability_still_triggers() {
    cr!("702.138b", "603.6a", "603.3");
    ruling!(
        "Uro, Titan of Nature's Wrath",
        "Uro's first ability causes you to sacrifice it if you didn't cast it, or if it was cast using any permission other than an escape ability."
    );
    ruling!(
        "Uro, Titan of Nature's Wrath",
        "Uro's second ability triggers when it enters the battlefield, even if it didn't escape."
    );
    ruling!(
        "Phlage, Titan of Fire's Fury",
        "Phlage's first ability causes you to sacrifice it if you didn't cast it, or if it was cast using any permission other than an escape ability."
    );
    ruling!(
        "Phlage, Titan of Fire's Fury",
        "Phlage's second ability triggers when it enters the battlefield, even if it didn't escape."
    );
    supported("Uro, Titan of Nature's Wrath");
    supported("Phlage, Titan of Fire's Fury");
    // Uro cast from the hand: sacrificed; still gain 3 life and draw a card.
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Island", 2);
    let uro = t.hand(P0, "Uro, Titan of Nature's Wrath");
    let hand = t.hand_size(P0);
    t.cast(P0, uro).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Uro, Titan of Nature's Wrath"));
    assert_eq!(t.life(P0), 23);
    assert_eq!(t.hand_size(P0), hand);
    // Phlage put onto the battlefield without being cast: sacrificed; it still deals 3
    // damage and you gain 3 life.
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Phlage, Titan of Fire's Fury");
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Phlage, Titan of Fire's Fury"));
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.life(P0), 23);
    // Phlage escaped: it stays.
    let mut t = TestGame::new(2);
    t.lands(P0, "Mountain", 2);
    t.lands(P0, "Plains", 2);
    let phlage = t.graveyard(P0, "Phlage, Titan of Fire's Fury");
    graveyard_n(&mut t, P0, "Forest", 5);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.cast(P0, phlage).method(ESCAPE).go();
    t.resolve_all();
    assert!(t.on_battlefield(phlage));
    assert_eq!(t.life(P1), 17);
}

#[test]
fn elspeth_escaped_the_same_turn_is_a_new_permanent() {
    cr!("400.7", "606.3", "702.138a");
    ruling!(
        "Elspeth, Sun's Nemesis",
        "If you activate a loyalty ability of Elspeth, she's put into your graveyard, and she escapes all in the same turn, she'll be a new permanent. You can activate one of her loyalty abilities even though she's represented by the same card."
    );
    supported("Elspeth, Sun's Nemesis");
    let mut t = TestGame::new(2);
    let elspeth = t.battlefield(P0, "Elspeth, Sun's Nemesis");
    let loyalty = t.counters(elspeth, counters::LOYALTY);
    t.g.remove_counters(Entity::Object(elspeth), counters::LOYALTY, loyalty - 3);
    // −3: You gain 5 life. (Loyalty 3 → 0: she's put into the graveyard.)
    t.activate(P0, elspeth, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 25);
    assert!(t.in_graveyard(P0, "Elspeth, Sun's Nemesis"));
    t.lands(P0, "Plains", 6);
    graveyard_n(&mut t, P0, "Forest", 4);
    let card = t.g.current(elspeth);
    t.cast(P0, card).method(ESCAPE).go();
    t.resolve_all();
    let again = t.g.current(elspeth);
    assert!(t.on_battlefield(again));
    assert_ne!(again, elspeth);
    t.activate(P0, again, 2, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 30);
}

#[test]
fn skyway_robber_casts_only_cards_exiled_by_its_last_escape() {
    cr!("702.138d", "607.2a", "400.7");
    ruling!(
        "Skyway Robber",
        "The ability Skyway Robber escapes with only allows you to cast cards exiled with its escape ability the last time it escaped. If it dies and escapes again later, or if it moves to another zone and returns to the battlefield, you won't be able to cast any of the cards exiled with it before."
    );
    supported("Skyway Robber");
    // "This creature escapes with 'Whenever this creature deals combat damage to a
    // player, you may cast an artifact, instant, or sorcery spell from among cards exiled
    // with this creature without paying its mana cost.'"
    fn escape(t: &mut TestGame, robber: ObjectId, others: &[&str]) -> ObjectId {
        for n in others {
            t.graveyard(P0, n);
        }
        let robber = t.g.current(robber);
        t.cast(P0, robber).method(ESCAPE).go();
        t.resolve_all();
        let robber = t.g.current(robber);
        assert!(t.on_battlefield(robber));
        t.g.objects[robber.0 as usize].summoning_sick = false;
        robber
    }
    // Deals combat damage to P1, choosing to cast any exiled Shock (at P1).
    fn hit(t: &mut TestGame, robber: ObjectId) {
        let shocks: Vec<Entity> = t
            .g
            .find_in_zone(Zone::Exile, "Shock")
            .into_iter()
            .map(Entity::Object)
            .collect();
        t.answer_choose(P0, &shocks);
        t.answer_targets(P0, &[Entity::Player(P1)]);
        attack_with(t, &[(robber, Entity::Player(P1))]);
        block_and_finish(t, P1, &[]);
        t.resolve_all();
    }
    let others = ["Shock", "Forest", "Forest", "Forest", "Forest"];
    // Escaped exiling Shock: Shock can be cast when it deals combat damage.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 4);
    let robber = t.graveyard(P0, "Skyway Robber");
    let robber = escape(&mut t, robber, &others);
    hit(&mut t, robber);
    assert_eq!(t.life(P1), 20 - 3 - 2);
    // It dies and escapes again: Shock, exiled by the earlier escape, can't be cast.
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 8);
    let robber = t.graveyard(P0, "Skyway Robber");
    let robber = escape(&mut t, robber, &others);
    destroy(&mut t, robber);
    let robber = escape(&mut t, robber, &["Forest"; 5]);
    hit(&mut t, robber);
    assert_eq!(t.life(P1), 17);
    assert!(t.in_exile("Shock"));
}

#[test]
fn timeless_dragon_can_be_eternalized_right_after_it_dies() {
    cr!("117.3b", "702.129a", "307.1");
    ruling!(
        "Timeless Dragon",
        "If a creature card with eternalize is put into your graveyard during your main phase, you'll have priority immediately afterward."
    );
    supported("Timeless Dragon");
    let mut t = TestGame::new(2);
    let dragon = t.battlefield(P0, "Timeless Dragon");
    t.lands(P0, "Swamp", 3);
    let murder = t.hand(P0, "Murder");
    t.cast(P0, murder).target(dragon).go();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Timeless Dragon"));
    assert_eq!(t.g.turn.priority, Some(P0));
    t.lands(P0, "Plains", 4);
    assert!(can_activate(&mut t, P0, dragon));
}

// ---------------------------------------------------------------------------------------
// Evoke
// ---------------------------------------------------------------------------------------

#[test]
fn night_incarnate_evoke_is_an_alternative_cost() {
    cr!("702.74a", "118.9a", "601.2f", "202.3");
    ruling!(
        "Night Incarnate",
        "Evoke is an alternative cost to cast the spell with evoke. It can't be combined with other alternative costs, such as casting a spell \"without paying its mana cost.\""
    );
    ruling!(
        "Night Incarnate",
        "To determine the total cost of a spell, start with the mana cost or alternative cost (such as an evoke cost) you're paying, add any cost increases, then apply any cost reductions. The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    supported("Night Incarnate");
    // Evoke {3}{B} plus Sphere of Resistance's {1}: five lands; mana value 5.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Sphere of Resistance");
    t.lands(P0, "Swamp", 5);
    let night = t.hand(P0, "Night Incarnate");
    let spell = t.cast(P0, night).method(EVOKE).go();
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 0);
    assert_eq!(mana_value(&t, spell), 5);
    // Cast evoked, it's sacrificed; cast "for free", it isn't evoked (and isn't).
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Night Incarnate"));
    let mut t = TestGame::new(2);
    let night = t.hand(P0, "Night Incarnate");
    mtg_engine::casting::grant_play_permission(
        &mut t.g,
        P0,
        vec![night],
        Duration::EndOfTurn,
        true,
        None,
    );
    let methods = legal_cast_methods(&mut t, P0, night);
    assert!(methods.contains(&CastMethod::Free));
    assert!(!methods.contains(&EVOKE));
    t.cast(P0, night).method(CastMethod::Free).go();
    t.resolve_all();
    assert!(t.on_battlefield(night));
}

#[test]
fn night_incarnate_put_onto_the_battlefield_isnt_sacrificed() {
    cr!("702.74a");
    ruling!(
        "Night Incarnate",
        "If a creature with evoke enters the battlefield without being cast, you won't have to sacrifice it."
    );
    let mut t = TestGame::new(2);
    let night = t.enter(P0, "Night Incarnate");
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(night));
}

// ---------------------------------------------------------------------------------------
// Evolve
// ---------------------------------------------------------------------------------------

#[test]
fn fathom_mage_draws_for_any_plus_one_counter() {
    cr!("122.6", "702.100a");
    ruling!(
        "Fathom Mage",
        "Fathom Mage's last ability will trigger whenever any +1/+1 counter is placed on it, not just ones due to the evolve ability."
    );
    supported("Fathom Mage");
    let mut t = TestGame::new(2);
    let mage = t.battlefield(P0, "Fathom Mage");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.g.add_counters(Entity::Object(mage), counters::PLUS1, 1, None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn evolve_rechecks_each_trigger_as_it_resolves() {
    cr!("702.100a", "603.4", "603.2c");
    ruling!(
        "Gluttonous Slug",
        "If multiple creatures enter the battlefield at the same time, evolve may trigger multiple times, although the comparison will take place each time one of those abilities tries to resolve. For example, if you control a 0/3 creature with evolve and two 1/1 creatures enter the battlefield"
    );
    ruling!(
        "Scurry Oak",
        "If multiple creatures enter the battlefield at the same time, evolve may trigger multiple times, although the comparison will take place each time one of those abilities tries to resolve. For example, if you control a 1/2 creature with evolve and two 2/2 creatures enter the battlefield"
    );
    supported("Gluttonous Slug");
    supported("Scurry Oak");
    // Gluttonous Slug (0/3) and two 1/1s.
    let mut t = TestGame::new(2);
    let slug = t.battlefield(P0, "Gluttonous Slug");
    enter_together(&mut t, &[(P0, "Llanowar Elves"), (P0, "Llanowar Elves")]);
    let evolve = |t: &TestGame| triggers_on_stack(t, "Evolve");
    assert_eq!(evolve(&t), 2);
    t.resolve_all();
    assert_eq!(t.counters(slug, counters::PLUS1), 1);
    assert_eq!(t.pt(slug), (1, 4));
    // Scurry Oak (1/2) and two 2/2s.
    let mut t = TestGame::new(2);
    let oak = t.battlefield(P0, "Scurry Oak");
    enter_together(&mut t, &[(P0, "Grizzly Bears"), (P0, "Silvercoat Lion")]);
    assert_eq!(evolve(&t), 2);
    t.resolve_all();
    assert_eq!(t.counters(oak, counters::PLUS1), 1);
    assert_eq!(t.pt(oak), (2, 3));
}

#[test]
fn evolves_once_however_many_counters_and_only_through_evolve() {
    cr!("702.100b", "702.100a", "614.1a");
    ruling!(
        "Renegade Krasis",
        "Renegade Krasis “evolves” when its evolve ability resolves and a +1/+1 counter is put on it. If a replacement effect such as Doubling Season’s causes the evolve ability to put more than one +1/+1 counter on Renegade Krasis, its last ability triggers only once. If no +1/+1 counter is put on it (perhaps because it left the battlefield in response to its evolve ability triggering), then its last ability doesn’t trigger."
    );
    ruling!(
        "Renegade Krasis",
        "Renegade Krasis’s last ability won’t trigger if a +1/+1 counter is put on it for any reason other than its evolve ability resolving."
    );
    ruling!(
        "Watchful Radstag",
        "Watchful Radstag “evolves” when its evolve ability resolves and a +1/+1 counter is put on it. If a replacement effect causes the evolve ability to put more than one +1/+1 counter on Watchful Radstag, its last ability triggers only once. If no +1/+1 counter is put on it (perhaps because it left the battlefield while its evolve ability was still on the stack), then its last ability doesn't trigger."
    );
    ruling!(
        "Watchful Radstag",
        "Watchful Radstag’s last ability won’t trigger if a +1/+1 counter is put on it for any reason other than its evolve ability resolving."
    );
    supported("Renegade Krasis");
    supported("Watchful Radstag");
    // Renegade Krasis (3/2) with Doubling Season: evolves once (two counters), so the
    // Bears (with a +1/+1 counter) gets one more counter (doubled: two).
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Doubling Season");
    let krasis = t.battlefield(P0, "Renegade Krasis");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.add_counters(Entity::Object(bears), counters::PLUS1, 1, None);
    t.settle();
    let before = t.counters(bears, counters::PLUS1);
    t.enter(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.counters(krasis, counters::PLUS1), 2);
    assert_eq!(t.counters(bears, counters::PLUS1), before + 2);
    // A +1/+1 counter from elsewhere: it doesn't evolve.
    t.g.add_counters(Entity::Object(krasis), counters::PLUS1, 1, None);
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Watchful Radstag (2/2): "Whenever this creature evolves, create a token that's a
    // copy of it." It leaves with evolve on the stack: no token.
    let mut t = TestGame::new(2);
    let stag = t.battlefield(P0, "Watchful Radstag");
    t.enter(P0, "Hill Giant");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "Evolve"), 1);
    destroy(&mut t, stag);
    t.resolve_all();
    assert!(tokens(&t, P0).is_empty());
    // With a replacement doubling counters, it evolves once: one token.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Hardened Scales");
    let stag = t.battlefield(P0, "Watchful Radstag");
    t.enter(P0, "Hill Giant");
    t.settle();
    t.resolve();
    assert_eq!(t.counters(stag, counters::PLUS1), 2);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "evolves"), 1);
    t.resolve();
    assert_eq!(tokens(&t, P0).len(), 1);
    // A counter from elsewhere: no token.
    t.g.add_counters(Entity::Object(stag), counters::PLUS1, 1, None);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "evolves"), 0);
}
