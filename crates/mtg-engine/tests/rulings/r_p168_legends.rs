//! Rulings batch P168, legendary synergies (part 1): The Peregrine Dynamo copying abilities
//! of legendary sources (CR 707.10), Acclaimed Contender's intervening "if" (CR 603.4),
//! permanents entering together (CR 603.6a) and the legend rule (CR 704.5j), Senu entering
//! attacking (CR 508.4), Mobilized District's P/T layers (CR 613.4), last known information
//! (Gríma Wormtongue, CR 603.10a), Gríma's life-gain lock (CR 119.7, 119.8), illegal
//! targets on resolution (Ancient Animus, Friendly Rivalry, Mjölnir: CR 608.2b), copies of
//! uncounterable spells (CR 707.10) and Goryo's Vengeance's delayed exile (CR 610.3).

use crate::r_s01_common::*;
use crate::r_s02_common::can_attack;
use crate::r_s03_common::to_blockers;
use crate::r_s04_common::add_mana;
use crate::r_s05_common::move_to;
use crate::r_s06_common::{attach_new, damage, give_control};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// Casts the real card `name` from `p`'s hand, giving `p` the lands for its mana cost.
fn cast_new(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    give_mana_for(t, p, name);
    let c = t.hand(p, name);
    for e in targets {
        t.answer_targets(p, &[*e]);
    }
    t.g.turn.priority = Some(p);
    t.cast(p, c).go()
}

/// The candidates of the target and entity choices asked since decision `from`.
fn choices_since(t: &TestGame, from: usize) -> Vec<(Vec<Entity>, u32)> {
    t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseTargets {
                candidates, max, ..
            }
            | Decision::ChooseEntities {
                candidates, max, ..
            } => Some((candidates.clone(), *max)),
            _ => None,
        })
        .collect()
}

fn food(t: &TestGame, p: PlayerId) -> usize {
    with_subtype(t, p, "Food").len()
}

// ---------------------------------------------------------------------------
// The Peregrine Dynamo
// ---------------------------------------------------------------------------

#[test]
fn peregrine_dynamo_copies_a_cycling_ability_of_a_legendary_card() {
    cr!("707.10", "113.7a");
    ruling!(
        "The Peregrine Dynamo",
        "A “legendary source” is a permanent, spell, or card in any zone with the supertype “legendary.” For example, The Peregrine Dynamo’s ability can target a cycling ability you’ve activated if the discarded card is a legendary card."
    );
    ruling!(
        "The Peregrine Dynamo",
        "The Peregrine Dynamo’s ability targets an ability that is on the stack and creates another instance of that ability on the stack. It doesn’t cause any object to gain an ability."
    );
    supported("The Peregrine Dynamo");
    supported("Ozolith, the Shattered Spire");
    let mut t = TestGame::new(2);
    let dynamo = t.battlefield(P0, "The Peregrine Dynamo");
    let ozolith = t.hand(P0, "Ozolith, the Shattered Spire");
    t.lands(P0, "Wastes", 3);
    crate::r_s04_common::cycle(&mut t, P0, ozolith, 0).unwrap();
    let cycling = *t.g.stack.last().unwrap();
    let abilities = t.obj_now(t.g.current(ozolith)).chars.abilities.len();
    t.activate(P0, dynamo, 0, &[obj(cycling)]).unwrap();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    // A copy of the cycling ability is on the stack; no object gained an ability.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(
        t.obj_now(t.g.current(ozolith)).chars.abilities.len(),
        abilities
    );
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
    // A nonlegendary card's cycling ability can't be targeted.
    let mut t = TestGame::new(2);
    let dynamo = t.battlefield(P0, "The Peregrine Dynamo");
    let barrens = t.hand(P0, "Ash Barrens");
    t.lands(P0, "Wastes", 2);
    crate::r_s04_common::cycle(&mut t, P0, barrens, 0).unwrap();
    let cycling = *t.g.stack.last().unwrap();
    assert!(t.activate(P0, dynamo, 0, &[obj(cycling)]).is_err());
}

#[test]
fn peregrine_dynamo_copy_makes_its_own_choices_and_payments() {
    cr!("707.10", "608.2");
    ruling!(
        "The Peregrine Dynamo",
        "Any choices made when the ability resolves won’t have been made yet when it’s copied. Any such choices will be made separately when the copy resolves. If a triggered ability asks you to pay a cost, you pay that cost for the copy separately."
    );
    // Garruk's −3 copied: each instance chooses its own green creature card.
    let mut t = TestGame::new(2);
    let dynamo = t.battlefield(P0, "The Peregrine Dynamo");
    let garruk = t.battlefield(P0, "Garruk, Caller of Beasts");
    let a = t.hand(P0, "Grizzly Bears");
    let b = t.hand(P0, "Kalonian Tusker");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, garruk, 1, &[]).unwrap();
    let minus = *t.g.stack.last().unwrap();
    t.activate(P0, dynamo, 0, &[obj(minus)]).unwrap();
    t.resolve();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a)]);
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(b)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Kalonian Tusker").len(), 1);
    // Oloro's "you may pay {1}" trigger copied: with mana for one payment, only one of
    // the two instances is paid for.
    supported("Oloro, Ageless Ascetic");
    for mana in [1usize, 2] {
        let mut t = TestGame::new(2);
        let dynamo = t.battlefield(P0, "The Peregrine Dynamo");
        t.battlefield(P0, "Oloro, Ageless Ascetic");
        cast_new(&mut t, P0, "Chaplain's Blessing", &[]);
        t.resolve();
        let trig = *t.g.stack.last().unwrap();
        t.lands(P0, "Wastes", 1);
        t.activate(P0, dynamo, 0, &[obj(trig)]).unwrap();
        t.resolve();
        assert_eq!(t.stack_len(), 2);
        t.lands(P0, "Wastes", mana);
        t.answer_yes(P0, true);
        t.answer_yes(P0, true);
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + mana, "{mana}");
        assert_eq!(t.life(P1), 20 - mana as i32, "{mana}");
    }
}

// ---------------------------------------------------------------------------
// Acclaimed Contender
// ---------------------------------------------------------------------------

#[test]
fn acclaimed_contender_gets_at_most_one_card() {
    cr!("608.2");
    ruling!(
        "Acclaimed Contender",
        "Acclaimed Contender's ability can get you at most one card from the top five cards, no matter how many other Knights you control."
    );
    supported("Acclaimed Contender");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Acclaimed Contender");
    t.battlefield(P0, "Acclaimed Contender");
    let lib = stack_library(
        &mut t,
        P0,
        &[
            "Acclaimed Contender",
            "Shadowspear",
            "Pacifism",
            "Acclaimed Contender",
            "Grizzly Bears",
        ],
    );
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(lib[0])]);
    t.enter(P0, "Acclaimed Contender");
    t.resolve_all();
    // The choice among the four eligible cards is of one card.
    let c = choices_since(&t, from);
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].0.len(), 4);
    assert_eq!(c[0].1, 1);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.zone(t.g.current(lib[0])), Zone::Hand(P0));
}

#[test]
fn acclaimed_contender_checks_for_a_knight_twice() {
    cr!("603.4");
    ruling!(
        "Acclaimed Contender",
        "Acclaimed Contender's ability won't trigger if you don't control another Knight immediately after it enters the battlefield. If you don't control another Knight as that ability resolves, the ability has no effect. This doesn't have the be the same Knight at both times, however."
    );
    fn game(knight_first: bool) -> (TestGame, Option<ObjectId>, ObjectId) {
        let mut t = TestGame::new(2);
        let k = knight_first.then(|| t.battlefield(P0, "Acclaimed Contender"));
        let shadow = t.library_top(P0, "Shadowspear");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(shadow)]);
        t.enter(P0, "Acclaimed Contender");
        t.settle();
        (t, k, shadow)
    }
    // No other Knight: no trigger.
    let (t, _, _) = game(false);
    assert_eq!(t.stack_len(), 0);
    // The Knight leaves before it resolves: no effect.
    let (mut t, k, shadow) = game(true);
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, k.unwrap(), Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(shadow)), Zone::Library(P0));
    // A different Knight is there as it resolves: it works.
    let (mut t, k, shadow) = game(true);
    move_to(&mut t, k.unwrap(), Zone::Hand(P0));
    t.battlefield(P0, "Thalia's Lancers");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(shadow)), Zone::Hand(P0));
}

// ---------------------------------------------------------------------------
// Key to the Side-Door
// ---------------------------------------------------------------------------

#[test]
fn key_to_the_side_door_doesnt_unblock_a_blocked_creature() {
    cr!("509.1h");
    ruling!(
        "Key to the Side-Door",
        "Activating the first ability of Key to the Side-Door after a creature has become blocked won't cause that creature to become unblocked."
    );
    supported("Key to the Side-Door");
    let mut t = TestGame::new(2);
    let key = t.battlefield(P0, "Key to the Side-Door");
    let giant = t.battlefield(P0, "Hill Giant");
    let wall = t.battlefield(P1, "Wall of Stone");
    t.lands(P0, "Wastes", 2);
    to_blockers(&mut t, &[(giant, pl(P1))], &[(wall, giant)]);
    t.activate(P0, key, 0, &[obj(giant)]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20);
    assert_eq!(crate::r_s07_common::damage_on(&t, wall), 3);
}

// ---------------------------------------------------------------------------
// Primevals' Glorious Rebirth, Kamahl's Druidic Vow
// ---------------------------------------------------------------------------

#[test]
fn legendary_permanents_put_onto_the_battlefield_together_see_each_other() {
    cr!("603.6a", "603.2");
    ruling!(
        "Primevals' Glorious Rebirth",
        "All of the permanents put onto the battlefield this way enter at the same time. If any have triggered abilities that trigger on something else entering the battlefield, they'll see each other."
    );
    ruling!(
        "Kamahl's Druidic Vow",
        "All of the permanents put onto the battlefield this way enter at the same time. If any have triggered abilities that trigger on something else entering the battlefield, they’ll see each other."
    );
    supported("Primevals' Glorious Rebirth");
    supported("Marwyn, the Nurturer");
    supported("Ezuri, Renegade Leader");
    // Marwyn: "Whenever another Elf you control enters, put a +1/+1 counter on Marwyn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Isamaru, Hound of Konda");
    t.graveyard(P0, "Marwyn, the Nurturer");
    t.graveyard(P0, "Ezuri, Renegade Leader");
    cast_new(&mut t, P0, "Primevals' Glorious Rebirth", &[]);
    t.resolve_all();
    let marwyn = t.named_on_battlefield("Marwyn, the Nurturer")[0];
    assert_eq!(t.named_on_battlefield("Ezuri, Renegade Leader").len(), 1);
    assert_eq!(t.counters(marwyn, "+1/+1"), 1);
    // Kamahl's Druidic Vow with X = 3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Isamaru, Hound of Konda");
    let lib = stack_library(
        &mut t,
        P0,
        &["Marwyn, the Nurturer", "Ezuri, Renegade Leader", "Forest"],
    );
    t.answer_choose(P0, &lib.iter().map(|c| obj(*c)).collect::<Vec<_>>());
    t.lands(P0, "Forest", 5);
    let vow = t.hand(P0, "Kamahl's Druidic Vow");
    t.cast(P0, vow).x(3).go();
    t.resolve_all();
    let marwyn = t.named_on_battlefield("Marwyn, the Nurturer");
    assert_eq!(marwyn.len(), 1);
    assert_eq!(t.named_on_battlefield("Ezuri, Renegade Leader").len(), 1);
    assert_eq!(t.counters(marwyn[0], "+1/+1"), 1);
}

#[test]
fn primevals_returns_every_legend_and_stacks_triggers_after_the_legend_rule() {
    cr!("704.5j", "117.5", "603.3");
    ruling!(
        "Primevals' Glorious Rebirth",
        "You must return all legendary permanent cards to the battlefield, even if the “legend rule” will put some right back into your graveyard. If any abilities triggered from the legendary permanents entering the battlefield, those abilities will be put onto the stack after applying the “legend rule.”"
    );
    supported("Bosco, Just a Bear");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Isamaru, Hound of Konda");
    t.graveyard(P0, "Bosco, Just a Bear");
    t.graveyard(P0, "Bosco, Just a Bear");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.to_lowercase().contains("legend")),
        |g| (g.stack.len(), g.permanents().filter(|o| o.chars.name == "Bosco, Just a Bear").count()),
    );
    cast_new(&mut t, P0, "Primevals' Glorious Rebirth", &[]);
    t.resolve_all();
    // Both returned; as the legend rule applied, no trigger was on the stack yet.
    assert_eq!(*seen.lock().unwrap(), vec![(0, 2)]);
    assert_eq!(t.named_on_battlefield("Bosco, Just a Bear").len(), 1);
    // Both Boscos' triggers resolved, each counting the two legendary creatures left.
    assert_eq!(food(&t, P0), 4);
}

// ---------------------------------------------------------------------------
// Senu, Keen-Eyed Protector; Master's Guidance
// ---------------------------------------------------------------------------

#[test]
fn senu_enters_attacking_without_attack_triggers() {
    cr!("508.4", "508.3a");
    ruling!(
        "Senu, Keen-Eyed Protector",
        "Although Senu enters the battlefield attacking as a result of its last ability, it was never declared as an attacking creature. Abilities that trigger whenever a creature attacks won't trigger when Senu enters the battlefield attacking."
    );
    ruling!(
        "Master's Guidance",
        "Creatures that enter the battlefield attacking will not trigger Master's Guidance's first triggered ability. You have to declare two or more legendary creatures as attackers during your declare attackers step to trigger the ability."
    );
    supported("Senu, Keen-Eyed Protector");
    supported("Master's Guidance");
    supported("Druids' Repository");
    let mut t = TestGame::new(2);
    // Druids' Repository: "Whenever a creature you control attacks, put a charge counter".
    let repo = t.battlefield(P0, "Druids' Repository");
    t.battlefield(P0, "Master's Guidance");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.exile(P0, "Senu, Keen-Eyed Protector");
    to_blockers(&mut t, &[(isamaru, pl(P1))], &[]);
    t.resolve_all();
    let senu = t.named_on_battlefield("Senu, Keen-Eyed Protector");
    assert_eq!(senu.len(), 1);
    assert!(t.g.combat.as_ref().unwrap().attackers.iter().any(|a| a.id == senu[0]));
    assert_eq!(t.counters(repo, "charge"), 1);
    // Master's Guidance didn't trigger: no counters.
    assert_eq!(t.counters(isamaru, "+1/+1"), 0);
    assert_eq!(t.counters(senu[0], "+1/+1"), 0);
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 16);
}

#[test]
fn senu_must_already_be_exiled_after_blockers_are_declared() {
    cr!("603.4", "509.1");
    ruling!(
        "Senu, Keen-Eyed Protector",
        "Senu's last ability will trigger only if Senu is already exiled at the point immediately after blockers are declared and at least one legendary creature you control that attacked isn't blocked. It's not possible to exile Senu during the declare blockers step in time to have the ability trigger."
    );
    let mut t = TestGame::new(2);
    let senu = t.battlefield(P0, "Senu, Keen-Eyed Protector");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    to_blockers(&mut t, &[(isamaru, pl(P1))], &[]);
    // In the declare blockers step, Senu exiles itself.
    t.answer(P0, DecisionKind::Scry, Answer::Default);
    t.activate(P0, senu, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    assert_eq!(t.zone(t.g.current(senu)), Zone::Exile);
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert_eq!(t.zone(t.g.current(senu)), Zone::Exile);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn senu_chooses_its_own_defender() {
    cr!("508.4", "506.2");
    ruling!(
        "Senu, Keen-Eyed Protector",
        "You choose which player, planeswalker, or battle Senu is attacking when you put it onto the battlefield attacking. It doesn't have to be the same one that the unblocked legendary creature you control is attacking."
    );
    let mut t = TestGame::new(3);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.exile(P0, "Senu, Keen-Eyed Protector");
    t.answer_choose(P0, &[pl(P2)]);
    to_blockers(&mut t, &[(isamaru, pl(P1))], &[]);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 18);
}

#[test]
fn masters_guidance_counters_go_on_any_attacking_creatures() {
    cr!("508.1m", "115.1");
    ruling!(
        "Master's Guidance",
        "Master's Guidance's first ability can put a +1/+1 counter on each of any two attacking creatures, not just on the legendary attacking creatures that caused it to trigger."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Master's Guidance");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let brimaz = t.battlefield(P0, "Brimaz, King of Oreskos");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears), obj(isamaru)]);
    crate::r_s01_common::attack_with(
        &mut t,
        &[(isamaru, pl(P1)), (brimaz, pl(P1)), (bears, pl(P1))],
    );
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert_eq!(t.counters(isamaru, "+1/+1"), 1);
    assert_eq!(t.counters(brimaz, "+1/+1"), 0);
}

// ---------------------------------------------------------------------------
// Mobilized District
// ---------------------------------------------------------------------------

#[test]
fn mobilized_district_overwrites_earlier_pt_but_keeps_counters() {
    cr!("613.4b", "613.4c", "613.7");
    ruling!(
        "Mobilized District",
        "An ability that turns a land into a creature also sets that creature's power and toughness. If the land was already a creature (for example, if it was the target of Awakening of Vitu-Ghazi), this will overwrite the previous effect that set its power and toughness. Effects that modify its power or toughness will continue to apply no matter when they started to take effect. The same is true for counters that change its power or toughness (such as +1/+1 counters) and effects that switch its power and toughness. For example, if Mobilized District has been made a 0/0 creature with nine +1/+1 counters on it, activating its last ability will turn it into a 12/12 Citizen creature that's still a land."
    );
    supported("Mobilized District");
    supported("Awakening of Vitu-Ghazi");
    let mut t = TestGame::new(2);
    let district = t.battlefield(P0, "Mobilized District");
    cast_new(&mut t, P0, "Awakening of Vitu-Ghazi", &[obj(district)]);
    t.resolve_all();
    assert_eq!(t.pt(district), (9, 9));
    add_mana(&mut t, P0, ManaType::C, 4);
    t.activate(P0, district, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(district), (12, 12));
    let o = t.obj_now(district);
    assert!(o.is(CardType::Land) && o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Citizen"));
}

#[test]
fn mobilized_district_that_entered_this_turn_cant_attack_or_tap() {
    cr!("302.6", "508.1a");
    ruling!(
        "Mobilized District",
        "If Mobilized District becomes a creature the same turn it enters the battlefield, you can't attack with it or activate its mana ability."
    );
    let mut t = TestGame::new(2);
    let district = t.enter(P0, "Mobilized District");
    t.settle();
    add_mana(&mut t, P0, ManaType::C, 4);
    t.activate(P0, district, 1, &[]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(district).is(CardType::Creature));
    t.g.turn.priority = Some(P0);
    assert!(!t
        .g
        .activatable_abilities(P0)
        .iter()
        .any(|(s, a)| *s == district && a.text.contains("Add {C}")));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_attack(&mut t, district));
    // One controlled since the turn began can do both.
    let mut t = TestGame::new(2);
    let district = t.battlefield(P0, "Mobilized District");
    add_mana(&mut t, P0, ManaType::C, 4);
    t.activate(P0, district, 1, &[]).unwrap();
    t.resolve_all();
    t.g.turn.priority = Some(P0);
    assert!(t
        .g
        .activatable_abilities(P0)
        .iter()
        .any(|(s, a)| *s == district && a.text.contains("Add {C}")));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(can_attack(&mut t, district));
}

#[test]
fn mobilized_district_counts_a_legendary_planeswalker_creature_once() {
    cr!("601.2f", "602.2b", "118.7");
    ruling!(
        "Mobilized District",
        "If you control four or more legendary permanents that are creatures and/or planeswalkers, the last ability of Mobilized District costs no mana to activate. A legendary planeswalker that's also a creature reduces this cost by only {1}."
    );
    supported("Gideon Blackblade");
    // Four legendary creatures: free.
    let mut t = TestGame::new(2);
    let district = t.battlefield(P0, "Mobilized District");
    for n in [
        "Isamaru, Hound of Konda",
        "Brimaz, King of Oreskos",
        "Bosco, Just a Bear",
        "Reki, the History of Kamigawa",
    ] {
        t.battlefield(P0, n);
    }
    // (Tapped, so it can't pay with its own mana ability.)
    t.g.tap(district);
    assert!(t.activate(P0, district, 1, &[]).is_ok());
    // Two legendary creatures and Gideon Blackblade (a creature on P0's turn): {1}.
    for mana in [0u32, 1] {
        let mut t = TestGame::new(2);
        let district = t.battlefield(P0, "Mobilized District");
        t.battlefield(P0, "Isamaru, Hound of Konda");
        t.battlefield(P0, "Brimaz, King of Oreskos");
        let gideon = t.battlefield(P0, "Gideon Blackblade");
        t.g.recompute();
        assert!(t.obj_now(gideon).is(CardType::Creature));
        assert!(t.obj_now(gideon).is(CardType::Planeswalker));
        add_mana(&mut t, P0, ManaType::C, mana);
        t.g.tap(district);
        assert_eq!(t.activate(P0, district, 1, &[]).is_ok(), mana == 1, "{mana}");
    }
}

// ---------------------------------------------------------------------------
// Arvad, Bosco, Gríma
// ---------------------------------------------------------------------------

#[test]
fn arvad_leaving_can_make_marked_damage_lethal() {
    cr!("704.5g", "611.3a");
    ruling!(
        "Arvad the Cursed",
        "Because damage remains marked on a creature until it's removed as the turn ends, nonlethal damage dealt to a legendary creature you control may become lethal if Arvad leaves the battlefield during that turn."
    );
    supported("Arvad the Cursed");
    let mut t = TestGame::new(2);
    let arvad = t.battlefield(P0, "Arvad the Cursed");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.g.recompute();
    assert_eq!(t.pt(isamaru), (4, 4));
    let src = t.battlefield(P1, "Raging Goblin");
    damage(&mut t, src, 3, isamaru);
    assert!(t.on_battlefield(isamaru));
    move_to(&mut t, arvad, Zone::Hand(P0));
    assert!(!t.on_battlefield(isamaru));
}

#[test]
fn bosco_counts_himself() {
    cr!("603.2", "111.10b");
    ruling!(
        "Bosco, Just a Bear",
        "Bosco will count himself as a legendary creature for his first ability. He's a bit of a legend around Ba Sing Se!"
    );
    let mut t = TestGame::new(2);
    t.enter(P0, "Bosco, Just a Bear");
    t.resolve_all();
    assert_eq!(food(&t, P0), 1);
}

#[test]
fn grima_checks_legendary_as_the_creature_last_existed() {
    cr!("603.10a", "608.2h", "701.47a");
    ruling!(
        "Gríma Wormtongue",
        "Gríma Wormtongue's activated ability checks whether the sacrificed creature was legendary as it last existed on the battlefield. It doesn't matter whether or not it's legendary in the graveyard."
    );
    supported("Gríma Wormtongue");
    supported("Leyline of Singularity");
    for (leyline, orcs) in [(true, 1), (false, 0)] {
        let mut t = TestGame::new(2);
        let grima = t.battlefield(P0, "Gríma Wormtongue");
        if leyline {
            // "All nonland permanents are legendary."
            t.battlefield(P0, "Leyline of Singularity");
        }
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.answer_choose(P0, &[obj(bears)]);
        t.activate(P0, grima, 0, &[pl(P1)]).unwrap();
        let card = t.g.current(bears);
        assert!(!t
            .obj_now(card)
            .chars
            .supertypes
            .contains(Supertype::Legendary));
        t.resolve_all();
        assert_eq!(t.life(P1), 19);
        assert_eq!(with_subtype(&t, P0, "Army").len(), orcs, "{leyline}");
    }
}

#[test]
fn grima_stops_opponents_life_from_rising() {
    cr!("119.5");
    ruling!(
        "Gríma Wormtongue",
        "If an effect says to set an opponent's life total to a number that's higher than the opponent's current life total while Gríma Wormtongue is on the battlefield, the player's life total doesn't change."
    );
    ruling!(
        "Gríma Wormtongue",
        "Spells and abilities that cause opponents to gain life still resolve while Gríma Wormtongue is on the battlefield. No opponent will gain life, but any other effects of that spell or ability will still happen."
    );
    supported("Sorin Markov");
    // Sorin Markov's −3: "Target opponent's life total becomes 10."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gríma Wormtongue");
    let sorin = t.battlefield(P0, "Sorin Markov");
    t.g.players[P1.idx()].life = 5;
    t.activate(P0, sorin, 1, &[pl(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 5);
    // Revitalize cast by P1: no life, but the card is drawn.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Gríma Wormtongue");
    t.set_step(P1, Step::PrecombatMain);
    give_mana_for(&mut t, P1, "Revitalize");
    let r = t.hand(P1, "Revitalize");
    let hand = t.hand_size(P1);
    t.cast(P1, r).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.hand_size(P1), hand);
    assert_eq!(t.graveyard_size(P1), 1);
}

// ---------------------------------------------------------------------------
// Heroes' Podium
// ---------------------------------------------------------------------------

#[test]
fn heroes_podium_bonus_applies_as_the_legend_rule_is_applied() {
    cr!("704.5j", "613.4c");
    ruling!(
        "Heroes' Podium",
        "Heroes' Podium will give a bonus to each legendary creature you control, even if gaining control of one causes the \"legend rule\" to apply. For example, if you control Brimaz, King of Oreskos (a 3/4 legendary creature), and gain control of another one, they'll each be 4/5 when you put one into its owner's graveyard. Then the remaining one will return to being 3/4."
    );
    supported("Heroes' Podium");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Heroes' Podium");
    let mine = t.battlefield(P0, "Brimaz, King of Oreskos");
    let theirs = t.battlefield(P1, "Brimaz, King of Oreskos");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseEntities { prompt, .. } if prompt.to_lowercase().contains("legend")),
        |g| {
            g.permanents()
                .filter(|o| o.chars.name == "Brimaz, King of Oreskos")
                .map(|o| (o.chars.power.unwrap_or(0), o.chars.toughness.unwrap_or(0)))
                .collect::<Vec<_>>()
        },
    );
    t.answer_choose(P0, &[obj(mine)]);
    give_control(&mut t, theirs, P0);
    assert_eq!(*seen.lock().unwrap(), vec![vec![(4, 5), (4, 5)]]);
    assert!(t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
    assert_eq!(t.pt(mine), (3, 4));
}

#[test]
fn heroes_podium_counts_itself_only_as_a_creature() {
    // The card's own behavior: "Each legendary creature you control gets +1/+1 for each
    // other legendary creature you control." (Its ruling's "in addition to being a land" is
    // left uncited; see the batch report.)
    cr!("613.4c", "205.4d");
    supported("March of the Machines");
    let mut t = TestGame::new(2);
    let podium = t.battlefield(P0, "Heroes' Podium");
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    t.battlefield(P0, "Grizzly Bears");
    t.g.recompute();
    assert_eq!(t.pt(isamaru), (2, 2));
    // March of the Machines: noncreature artifacts are creatures with P/T equal to their
    // mana value. The Podium is now a legendary creature: it counts, and gets the bonus.
    t.battlefield(P0, "March of the Machines");
    t.g.recompute();
    assert_eq!(t.pt(isamaru), (3, 3));
    assert_eq!(t.pt(podium), (6, 6));
}

// ---------------------------------------------------------------------------
// Ancient Animus, Friendly Rivalry
// ---------------------------------------------------------------------------

#[test]
fn ancient_animus_targets_and_illegal_targets() {
    cr!("608.2b", "701.14b", "601.2c");
    ruling!(
        "Ancient Animus",
        "You can't cast Ancient Animus unless you choose both a creature you control and a creature you don't control as targets."
    );
    ruling!(
        "Ancient Animus",
        "If either target is an illegal target as Ancient Animus tries to resolve, neither creature will deal or be dealt damage."
    );
    ruling!(
        "Ancient Animus",
        "If the creature you control is an illegal target as Ancient Animus tries to resolve, you won't put a +1/+1 counter on it. If that creature is a legal target but the creature you don't control isn't, you'll still put the counter on the creature you control if it's legendary."
    );
    supported("Ancient Animus");
    // One target only: can't be cast.
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    give_mana_for(&mut t, P0, "Ancient Animus");
    let animus = t.hand(P0, "Ancient Animus");
    assert!(t.cast(P0, animus).target(isamaru).try_go().is_err());
    // The opponent's creature leaves: the counter, but no fight.
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let ogre = t.battlefield(P1, "Gray Ogre");
    let wall = t.battlefield(P1, "Wall of Stone");
    cast_new(&mut t, P0, "Ancient Animus", &[obj(isamaru), obj(ogre)]);
    move_to(&mut t, ogre, Zone::Hand(P1));
    t.resolve_all();
    assert_eq!(t.counters(isamaru, "+1/+1"), 1);
    assert_eq!(crate::r_s07_common::damage_on(&t, isamaru), 0);
    let _ = wall;
    // P0's creature leaves: no counter, no damage to the other.
    let mut t = TestGame::new(2);
    let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
    let ogre = t.battlefield(P1, "Gray Ogre");
    cast_new(&mut t, P0, "Ancient Animus", &[obj(isamaru), obj(ogre)]);
    move_to(&mut t, isamaru, Zone::Hand(P0));
    t.resolve_all();
    assert_eq!(crate::r_s07_common::damage_on(&t, ogre), 0);
    assert_eq!(t.counters(t.g.current(isamaru), "+1/+1"), 0);
}

#[test]
fn friendly_rivalry_with_illegal_targets() {
    cr!("608.2b");
    ruling!(
        "Friendly Rivalry",
        "If one of the two target creatures you control is an illegal target as Friendly Rivalry resolves, the other will still deal damage equal to its power."
    );
    ruling!(
        "Friendly Rivalry",
        "If the last target is an illegal target as Friendly Rivalry resolves, or if both of the first targets are illegal targets, no creature deals or is dealt damage."
    );
    supported("Friendly Rivalry");
    // (which of P0's targets leaves, the damage dealt to Wall of Stone)
    for (gone, dealt) in [
        (vec![0usize], 2u32),
        (vec![1], 3),
        (vec![0, 1], 0),
        (vec![2], 0),
    ] {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
        let wall = t.battlefield(P1, "Wall of Stone");
        cast_new(
            &mut t,
            P0,
            "Friendly Rivalry",
            &[obj(giant), obj(isamaru), obj(wall)],
        );
        let ids = [giant, isamaru, wall];
        for i in &gone {
            let owner = if *i == 2 { P1 } else { P0 };
            move_to(&mut t, ids[*i], Zone::Hand(owner));
        }
        t.resolve_all();
        if t.on_battlefield(wall) {
            assert_eq!(crate::r_s07_common::damage_on(&t, wall), dealt, "{gone:?}");
        }
        for c in [giant, isamaru] {
            if t.on_battlefield(c) {
                assert_eq!(crate::r_s07_common::damage_on(&t, c), 0, "{gone:?}");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Delighted Halfling, Goryo's Vengeance, Mjölnir
// ---------------------------------------------------------------------------

#[test]
fn delighted_halfling_copy_of_the_spell_can_be_countered() {
    cr!("707.10", "106.6");
    ruling!(
        "Delighted Halfling",
        "If the legendary spell you cast this way is copied, the copy can be countered."
    );
    supported("Delighted Halfling");
    supported("Twincast");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Isamaru, Hound of Konda");
    let h = t.battlefield(P0, "Delighted Halfling");
    t.answer(P0, DecisionKind::Option, Answer::Index(3));
    t.activate(P0, h, 1, &[]).unwrap();
    assert_eq!(t.g.player(P0).mana_pool.count(ManaType::R), 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    let jaya = t.hand(P0, "Jaya's Immolating Inferno");
    t.answer_targets(P0, &[pl(P1)]);
    let spell = t.cast(P0, jaya).x(1).go();
    t.answer_yes(P0, false);
    cast_new(&mut t, P0, "Twincast", &[obj(spell)]);
    t.resolve();
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, spell);
    // P1 counters the copy: it's countered.
    cast_new(&mut t, P1, "Counterspell", &[obj(copy)]);
    t.resolve();
    assert!(!t.g.stack.contains(&copy));
    // P1 tries to counter the original: it resolves anyway.
    cast_new(&mut t, P1, "Counterspell", &[obj(spell)]);
    t.resolve();
    assert!(t.g.stack.contains(&spell));
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn goryos_vengeance_leaves_a_creature_that_moved_alone() {
    cr!("603.7c", "400.7");
    ruling!(
        "Goryo's Vengeance",
        "If the returned creature leaves the battlefield before the end step, it will remain in its current zone. It won't be exiled."
    );
    supported("Goryo's Vengeance");
    let mut t = TestGame::new(2);
    let isamaru = t.graveyard(P0, "Isamaru, Hound of Konda");
    cast_new(&mut t, P0, "Goryo's Vengeance", &[obj(isamaru)]);
    t.resolve_all();
    let back = t.g.current(isamaru);
    assert!(t.on_battlefield(back));
    let in_hand = move_to(&mut t, back, Zone::Hand(P0)).unwrap();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.zone(in_hand), Zone::Hand(P0));
}

#[test]
fn mjolnir_with_an_illegal_target_deals_no_damage() {
    cr!("608.2b");
    ruling!(
        "Mjölnir, Storm Hammer",
        "If the target creature is an illegal target as Mjölnir’s last ability triers to resolve, it won’t resolve and none of its effects will happen. No opponents will be dealt damage."
    );
    supported("Mjölnir, Storm Hammer");
    for leaves in [true, false] {
        let mut t = TestGame::new(2);
        let isamaru = t.battlefield(P0, "Isamaru, Hound of Konda");
        attach_new(&mut t, P0, "Mjölnir, Storm Hammer", isamaru);
        let tapped = t.battlefield(P1, "Grizzly Bears");
        t.g.tap(tapped);
        let target = t.battlefield(P1, "Gray Ogre");
        t.answer_targets(P0, &[obj(target)]);
        crate::r_s01_common::attack_with(&mut t, &[(isamaru, pl(P1))]);
        if leaves {
            move_to(&mut t, target, Zone::Hand(P1));
        }
        t.resolve_all();
        // Two tapped creatures (the Bears and the Ogre) or nothing.
        assert_eq!(t.life(P1), if leaves { 20 } else { 18 }, "{leaves}");
    }
}


