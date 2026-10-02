//! Rulings batch P184 — Dragon typal cards: what a "Dragon card" is (CR 205.3m), cost
//! reductions for Dragon spells (CR 118.7, 601.2f), Dragons of Tarkir's "if you revealed
//! a Dragon card or controlled a Dragon as you cast this spell" (CR 601.2b, 601.2i) and
//! copies of those spells (CR 707.10), cast triggers that resolve before the spell
//! (CR 603.3), "doesn't untap during its controller's next untap step" (CR 502.3), and
//! permissions to cast Dragon spells (CR 601.3).

use crate::r_s01_common::*;
use crate::r_s02_common::destroy;
use crate::r_s03_common::run_effect;
use crate::r_s06_common::give_control;
use crate::r_s07_common::cast_methods;
use crate::r_s14_common::{cast_from_hand, triggers_on_stack_now};
use crate::r_s21_common::castable;
use crate::r_s25_common::cast_new;
use mtg_engine::ability::{Effect, Sel, Value};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::counters;
use mtg_engine::*;

/// Casts `card` for P0 with `method` (paying automatically with P0's lands); whether it
/// was cast.
fn try_cast(t: &mut TestGame, card: ObjectId, method: CastMethod) -> bool {
    let ok = t.cast(P0, card).method(method).try_go().is_ok();
    t.clear_answers();
    ok
}

#[test]
fn temple_of_the_dragon_queen_a_dragon_has_the_dragon_creature_type() {
    cr!("205.3m", "614.12", "614.1d");
    ruling!(
        "Temple of the Dragon Queen",
        "A Dragon card is a card with the creature type Dragon in its type line. Similarly, a creature on the battlefield is a Dragon if it has the Dragon creature type. A card that has \"Dragon\" in the name (such as Temple of the Dragon Queen) isn't a Dragon card unless it also has the Dragon creature type."
    );
    supported("Temple of the Dragon Queen");
    supported("Dragon Fodder");
    supported("Dragonlord's Servant");
    // Cards and a creature with "Dragon" in their names, none of them a Dragon: P0 tries
    // to reveal a Dragon card as the Temple enters.
    let play = |dragon_in_hand: bool, dragon_on_battlefield: bool| {
        let mut t = TestGame::new(2);
        t.hand(P0, "Dragon Fodder");
        t.hand(P0, "Dragonlord's Servant");
        t.hand(P0, "Temple of the Dragon Queen");
        t.battlefield(P0, "Dragonlord's Servant");
        if dragon_in_hand {
            t.hand(P0, "Shivan Dragon");
        }
        if dragon_on_battlefield {
            t.battlefield(P0, "Shivan Dragon");
        }
        t.answer_yes(P0, true);
        let temple = t.hand(P0, "Temple of the Dragon Queen");
        t.play_land(P0, temple).unwrap();
        t.settle();
        t.obj_now(temple).tapped
    };
    assert!(play(false, false), "no Dragon: enters tapped");
    assert!(!play(true, false), "a revealed Shivan Dragon");
    assert!(!play(false, true), "a controlled Shivan Dragon");
}

#[test]
fn dragonlords_servant_reduces_only_generic_mana() {
    cr!("118.7a", "601.2f", "202.3");
    ruling!(
        "Dragonlord's Servant",
        "Dragonlord's Servant can't reduce the colored mana requirement of a Dragon spell you cast."
    );
    ruling!(
        "Dragonlord's Servant",
        "Dragonlord's Servant's ability can't reduce the amount of colored mana you pay for a spell. It reduces only the generic mana component of that spell."
    );
    ruling!(
        "Dragonlord's Servant",
        "Dragonlord's Servant's ability doesn't change the mana cost or mana value of any spell. It changes only the total cost you pay to cast Dragon spells."
    );
    supported("Slumbering Dragon");
    // Slumbering Dragon ({R}) still costs {R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dragonlord's Servant");
    let slumbering = t.hand(P0, "Slumbering Dragon");
    t.lands(P0, "Wastes", 2);
    assert!(!castable(&mut t, P0, slumbering));
    t.lands(P0, "Mountain", 1);
    assert!(castable(&mut t, P0, slumbering));
    // Shivan Dragon ({4}{R}{R}) costs {3}{R}{R}; its mana value stays 6.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dragonlord's Servant");
    let shivan = t.hand(P0, "Shivan Dragon");
    t.lands(P0, "Wastes", 4);
    t.lands(P0, "Mountain", 1);
    assert!(!castable(&mut t, P0, shivan));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dragonlord's Servant");
    let shivan = t.hand(P0, "Shivan Dragon");
    t.lands(P0, "Wastes", 3);
    t.lands(P0, "Mountain", 2);
    let spell = t.cast(P0, shivan).go();
    assert_eq!(tapped_lands(&t, P0), 5);
    assert_eq!(t.g.mana_value_of(spell), 6);
    assert_eq!(
        t.obj(spell).chars.mana_cost.as_ref().map(|m| m.to_string()),
        Some("{4}{R}{R}".to_string())
    );
}

#[test]
fn dragonlords_servant_applies_after_additional_costs_and_to_alternative_costs() {
    cr!("601.2f", "903.8", "118.9", "702.109a");
    ruling!(
        "Dragonlord's Servant",
        "If there are any additional costs to cast a Dragon spell, apply those before applying Dragonlord's Servant and any other cost reductions."
    );
    ruling!(
        "Dragonlord's Servant",
        "The ability can apply to alternative costs to cast a Dragon spell."
    );
    supported("Kolaghan, the Storm's Fury");
    // A commander Slumbering Dragon ({R}) cast a second time from the command zone: the
    // additional {2} is added first, then reduced by {1}: {1}{R}.
    let mut t = crate::r_s13_common::commander_game();
    t.battlefield(P0, "Dragonlord's Servant");
    let cmd = crate::r_s13_common::commander(&mut t, P0, "Slumbering Dragon");
    let key = mtg_engine::kw::partner::commander_key(&t.g, cmd);
    t.g.players[0].commander_casts.insert(key, 1);
    t.lands(P0, "Mountain", 1);
    assert!(!castable(&mut t, P0, cmd));
    t.lands(P0, "Wastes", 1);
    assert!(castable(&mut t, P0, cmd));
    t.cast(P0, cmd).go();
    assert_eq!(tapped_lands(&t, P0), 2);
    // Kolaghan's dash cost {3}{B}{R} costs {2}{B}{R}.
    let dash = CastMethod::Keyword(KeywordKind::Dash);
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dragonlord's Servant");
    let kolaghan = t.hand(P0, "Kolaghan, the Storm's Fury");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 2);
    assert!(try_cast(&mut t, kolaghan, dash));
    assert_eq!(tapped_lands(&t, P0), 4);
    t.resolve_all();
    assert!(t.on_battlefield(kolaghan));
}

#[test]
fn dromoka_each_bolster_chooses_the_least_toughness_as_it_resolves() {
    cr!("701.39a", "508.1m", "603.3b");
    ruling!(
        "Dromoka, the Eternal",
        "Each Dragon you attack with will cause Dromoka’s triggered ability to trigger. For each instance of the ability, you’ll determine which creature to put +1/+1 counters on as it resolves. Specifically, the creature you control with the lowest toughness as the first such ability resolves may not have the lowest toughness as the second such ability resolves."
    );
    supported("Dromoka, the Eternal");
    let mut t = TestGame::new(2);
    let dromoka = t.battlefield(P0, "Dromoka, the Eternal");
    let shivan = t.battlefield(P0, "Shivan Dragon");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let elves = t.battlefield(P0, "Llanowar Elves");
    attack_with(
        &mut t,
        &[(dromoka, Entity::Player(P1)), (shivan, Entity::Player(P1))],
    );
    assert_eq!(triggers_on_stack_now(&t), 2);
    // The first bolster: Llanowar Elves (toughness 1) gets the counters.
    t.resolve();
    assert_eq!(t.counters(elves, counters::PLUS1), 2);
    // The second: now the Bears (toughness 2) have the least toughness.
    t.resolve();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    assert_eq!(t.counters(elves, counters::PLUS1), 2);
}

#[test]
fn cast_triggers_resolve_first_even_if_the_spell_is_countered() {
    cr!("603.3", "601.2i", "701.6a");
    ruling!(
        "Rohgahh, Kher Keep Overlord",
        "Each of Rohgahh, Kher Keep Overlord’s last two abilities goes on the stack above the spell that caused it to trigger and resolves first. It resolves even if that spell is countered or otherwise left the stack."
    );
    supported("Rohgahh, Kher Keep Overlord");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Rohgahh, Kher Keep Overlord");
    let spell = cast_new(&mut t, P0, "Shivan Dragon", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert_ne!(*t.g.stack.last().unwrap(), spell);
    cast_new(&mut t, P1, "Counterspell", &[Entity::Object(spell)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Shivan Dragon"));
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Kobolds of Kher Keep").len(), 1);
}

#[test]
fn thunderbreak_regent_resolves_before_the_targeting_spell() {
    cr!("603.3", "603.2");
    ruling!(
        "Thunderbreak Regent",
        "Thunderbreak Regent's ability will resolve before the spell or ability that caused it to trigger."
    );
    supported("Thunderbreak Regent");
    let mut t = TestGame::new(2);
    let regent = t.battlefield(P0, "Thunderbreak Regent");
    let shock = cast_new(&mut t, P1, "Shock", &[Entity::Object(regent)]);
    t.resolve();
    // The trigger resolved; Shock is still on the stack.
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.zone(shock), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.obj_now(regent).damage, 2);
}

#[test]
fn scalelord_reckoner_resolves_even_if_the_targeting_spell_is_countered() {
    cr!("603.3", "603.2");
    ruling!(
        "Scalelord Reckoner",
        "Scalelord Reckoner's triggered ability resolves before the spell or ability that caused it to trigger. The ability will resolve even if that spell or ability is countered."
    );
    supported("Scalelord Reckoner");
    let mut t = TestGame::new(2);
    let reckoner = t.battlefield(P0, "Scalelord Reckoner");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    let shock = cast_new(&mut t, P1, "Shock", &[Entity::Object(reckoner)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    // P0 counters Shock in response; the trigger above it resolves first anyway.
    cast_new(&mut t, P0, "Counterspell", &[Entity::Object(shock)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Shock"));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.obj_now(reckoner).damage, 0);
}

#[test]
fn scalelord_reckoners_targeting_each_other_loop_into_a_draw() {
    cr!("104.4b", "603.2", "115.2");
    ruling!(
        "Scalelord Reckoner",
        "If the only nonland permanents on the battlefield are Scalelord Reckoners and Dragons controlled by players who control Scalelord Reckoners, any spell or ability that targets one of those Dragons will cause an infinite loop of triggered abilities and the game will immediately end in a draw."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Scalelord Reckoner");
    t.battlefield(P1, "Scalelord Reckoner");
    // P1 targets P0's Reckoner: P0's trigger must target P1's Reckoner, whose trigger must
    // target P0's Reckoner, and so on.
    cast_new(&mut t, P1, "Shock", &[Entity::Object(mine)]);
    t.g.run_until(20_000, |g| g.result.is_some());
    assert_eq!(t.g.result, Some(mtg_engine::game::GameResult::Draw));
}

/// P0 casts Draconic Roar at P1's Hill Giant (revealing a Dragon card if `reveal`, while
/// controlling a Shivan Dragon), and copies it. Returns P1's life total afterward.
fn roar_and_copy(reveal: bool) -> i32 {
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Hill Giant");
    t.battlefield(P0, "Shivan Dragon");
    let card = t.hand(P0, "Shivan Dragon");
    t.lands(P0, "Mountain", 2);
    let roar = t.hand(P0, "Draconic Roar");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(reveal));
    if reveal {
        t.answer_choose(P0, &[Entity::Object(card)]);
    }
    let spell = t.cast(P0, roar).target(Entity::Object(giant)).go();
    // A copy, with a new target: the other Hill Giant.
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Object(other)]);
    run_effect(
        &mut t,
        None,
        P0,
        Effect::CopySpell {
            what: Sel::Target(0),
            count: Value::c(1),
            new_targets: true,
        },
        &[Entity::Object(spell)],
    );
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 0);
    t.life(P1)
}

#[test]
fn a_copy_gets_the_dragon_bonus_only_if_a_dragon_card_was_revealed() {
    cr!("707.10", "601.2b", "601.2i");
    ruling!(
        "Draconic Roar",
        "If one of these spells is copied, the controller of the copy will get the “Dragon bonus” only if a Dragon card was revealed as an additional cost. The copy wasn’t cast, so whether you controlled a Dragon won’t matter."
    );
    supported("Draconic Roar");
    // Controlling a Dragon (no reveal): only the original deals 3 damage to P1.
    assert_eq!(roar_and_copy(false), 17);
    // Revealing a Dragon card: the copy has the bonus too.
    assert_eq!(roar_and_copy(true), 14);
}

/// P0 casts Scaleguard Sentinels (revealing a Dragon card from hand if `reveal`,
/// controlling a Shivan Dragon if `control`, with `in_hand` Dragon cards in hand) and, if
/// `copy`, copies it. Returns the +1/+1 counters on each Scaleguard Sentinels that
/// entered (the card first).
fn sentinels(reveal: bool, control: bool, in_hand: usize, copy: bool) -> Vec<u32> {
    let mut t = TestGame::new(2);
    if control {
        t.battlefield(P0, "Shivan Dragon");
    }
    let cards: Vec<ObjectId> = (0..in_hand)
        .map(|_| t.hand(P0, "Shivan Dragon"))
        .collect();
    t.lands(P0, "Forest", 2);
    let card = t.hand(P0, "Scaleguard Sentinels");
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(reveal));
    if reveal {
        let all: Vec<Entity> = cards.iter().map(|c| Entity::Object(*c)).collect();
        t.answer_choose(P0, &all);
    }
    let spell = t.cast(P0, card).go();
    if copy {
        run_effect(
            &mut t,
            None,
            P0,
            Effect::CopySpell {
                what: Sel::Target(0),
                count: Value::c(1),
                new_targets: false,
            },
            &[Entity::Object(spell)],
        );
    }
    t.resolve_all();
    let mut ids = t.named_on_battlefield("Scaleguard Sentinels");
    ids.sort_by_key(|id| t.obj(*id).is_token());
    ids.iter().map(|id| t.counters(*id, counters::PLUS1)).collect()
}

#[test]
fn scaleguard_sentinels_a_copy_gets_the_bonus_only_for_a_revealed_card() {
    cr!("707.10", "601.2b", "601.2i", "707.10f");
    ruling!(
        "Scaleguard Sentinels",
        "If one of these spells is copied, the controller of the copy will get the \"Dragon bonus\" only if a Dragon card was revealed as an additional cost. The copy wasn't cast, so whether you controlled a Dragon won't matter."
    );
    supported("Scaleguard Sentinels");
    assert_eq!(sentinels(false, true, 0, true), vec![1, 0]);
    assert_eq!(sentinels(true, false, 1, true), vec![1, 1]);
    assert_eq!(sentinels(false, false, 0, true), vec![0, 0]);
}

#[test]
fn scaleguard_sentinels_revealing_and_controlling_dont_add_up() {
    cr!("601.2b", "601.2i", "614.1c");
    ruling!(
        "Scaleguard Sentinels",
        "You can't reveal more than one Dragon card to multiply the bonus. There is also no additional benefit for both revealing a Dragon card as an additional cost and controlling a Dragon as you cast the spell."
    );
    assert_eq!(sentinels(false, false, 1, false), vec![0]);
    assert_eq!(sentinels(true, false, 2, false), vec![1]);
    assert_eq!(sentinels(false, true, 0, false), vec![1]);
    assert_eq!(sentinels(true, true, 2, false), vec![1]);
}

#[test]
fn the_dragon_must_be_controlled_as_the_spell_finishes_being_cast() {
    cr!("601.2i", "601.2g", "605.3a");
    ruling!(
        "Scaleguard Sentinels",
        "If you don't reveal a Dragon card from your hand, you must control a Dragon as you are finished casting the spell to get the bonus. For example, if you lose control of your only Dragon while casting the spell (because, for example, you sacrificed it to activate a mana ability), you won't get the bonus."
    );
    ruling!(
        "Draconic Roar",
        "If you don’t reveal a Dragon card from your hand, you must control a Dragon as you are finished casting the spell to get the bonus. For example, if you lose control of your only Dragon while casting the spell (because, for example, you sacrificed it to activate a mana ability), you won’t get the bonus."
    );
    supported("Ashnod's Altar");
    // P0's only Dragon is sacrificed to Ashnod's Altar ("Sacrifice a creature: Add
    // {C}{C}.") to pay for Draconic Roar ({1}{R}) while casting it.
    let roar = |sacrifice: bool| {
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P1, "Hill Giant");
        let dragon = t.battlefield(P0, "Shivan Dragon");
        t.battlefield(P0, "Ashnod's Altar");
        t.lands(P0, "Mountain", 1);
        if !sacrifice {
            t.lands(P0, "Wastes", 1);
        } else {
            t.answer_choose(P0, &[Entity::Object(dragon)]);
        }
        let roar = t.hand(P0, "Draconic Roar");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
        t.cast(P0, roar).target(Entity::Object(giant)).go();
        assert_eq!(t.on_battlefield(dragon), !sacrifice);
        t.resolve_all();
        assert!(t.in_graveyard(P1, "Hill Giant"));
        t.life(P1)
    };
    assert_eq!(roar(false), 17);
    assert_eq!(roar(true), 20);
    // Scaleguard Sentinels ({G}{G}), paid with a Forest and P0's only Dragon, sacrificed
    // for {G} by its own mana ability while the spell is being cast.
    let sentinels = |sacrifice: bool| {
        let mut t = TestGame::new(2);
        let drake = custom_card(
            "Sapling Drake",
            "Creature — Dragon",
            "{2}{G}",
            Some((2, 2)),
            "Sacrifice this creature: Add {G}.",
        );
        let dragon = t.custom(P0, drake, Zone::Battlefield);
        t.lands(P0, "Forest", 1);
        if !sacrifice {
            t.lands(P0, "Forest", 1);
        }
        let card = t.hand(P0, "Scaleguard Sentinels");
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(false));
        t.cast(P0, card).go();
        assert_eq!(t.on_battlefield(dragon), !sacrifice);
        t.resolve_all();
        t.counters(t.g.current(card), counters::PLUS1)
    };
    assert_eq!(sentinels(false), 1);
    assert_eq!(sentinels(true), 0);
}

#[test]
fn korlessa_doesnt_change_when_dragon_spells_can_be_cast() {
    cr!("302.1", "117.1a", "601.3");
    ruling!(
        "Korlessa, Scale Singer",
        "Korlessa doesn't change when you can cast Dragon spells. Normally, this means during your main phase when the stack is empty, although flash may change this."
    );
    supported("Korlessa, Scale Singer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Korlessa, Scale Singer");
    let dragon = t.library_top(P0, "Slumbering Dragon");
    t.lands(P0, "Mountain", 1);
    t.set_step(P0, Step::PrecombatMain);
    assert!(castable(&mut t, P0, dragon));
    t.set_step(P0, Step::Upkeep);
    assert!(!castable(&mut t, P0, dragon));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!castable(&mut t, P0, dragon));
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 1);
    cast_new(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    assert!(!castable(&mut t, P0, dragon));
    t.resolve_all();
    assert!(castable(&mut t, P0, dragon));
}

#[test]
fn fearsome_awakening_checks_for_a_dragon_right_as_it_returns() {
    cr!("608.2c", "117.1");
    ruling!(
        "Fearsome Awakening",
        "No player can cast spells or activate abilities between returning the creature card to the battlefield and checking whether it’s a Dragon."
    );
    supported("Fearsome Awakening");
    for (name, n) in [("Shivan Dragon", 2), ("Hill Giant", 0)] {
        let mut t = TestGame::new(2);
        let card = t.graveyard(P0, name);
        cast_new(&mut t, P0, "Fearsome Awakening", &[Entity::Object(card)]);
        t.resolve();
        let back = t.g.current(card);
        assert!(t.on_battlefield(back), "{name}");
        assert_eq!(t.counters(back, counters::PLUS1), n, "{name}");
        // Nothing else happened in between: the spell resolved as one action.
        assert_eq!(t.stack_len(), 0);
    }
}

#[test]
fn dracogenesis_its_caster_gets_priority_first_after_it_resolves() {
    cr!("117.3b", "608.2n");
    ruling!(
        "Dracogenesis",
        "Once you cast Dracogenesis, if it’s your turn, you’ll have priority immediately after it resolves. You can cast another spell before any player can attempt to remove Dracogenesis with spells or abilities."
    );
    supported("Dracogenesis");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let draco = cast_new(&mut t, P0, "Dracogenesis", &[]);
    let shivan = t.hand(P0, "Shivan Dragon");
    // Both players pass; Dracogenesis resolves, and P0 receives priority.
    let ok = t.g.run_until(1_000, |g| {
        g.stack.is_empty() && g.turn.stage == Stage::Priority && g.turn.priority.is_some()
    });
    assert!(ok);
    assert!(t.on_battlefield(t.g.current(draco)));
    assert_eq!(t.g.turn.priority, Some(P0));
    // P0 may cast a Dragon spell for free right away.
    assert!(cast_methods(&mut t, P0, shivan).contains(&CastMethod::Free));
}

#[test]
fn dracogenesis_follows_normal_timing() {
    cr!("601.3", "117.1a", "118.9");
    ruling!(
        "Dracogenesis",
        "You must follow the normal timing permissions and restrictions of each Dragon spell you cast this way."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dracogenesis");
    let shivan = t.hand(P0, "Shivan Dragon");
    t.set_step(P0, Step::PrecombatMain);
    assert!(castable(&mut t, P0, shivan));
    t.set_step(P0, Step::Upkeep);
    assert!(!castable(&mut t, P0, shivan));
    t.set_step(P1, Step::PrecombatMain);
    assert!(!castable(&mut t, P0, shivan));
}

/// P0's Ojutai, Soul of Winter attacks; its trigger targets P1's `target`.
fn ojutai_attack(t: &mut TestGame, extra_dragon: bool, target: ObjectId) {
    let ojutai = t.battlefield(P0, "Ojutai, Soul of Winter");
    let mut attackers = vec![(ojutai, Entity::Player(P1))];
    if extra_dragon {
        let shivan = t.battlefield(P0, "Shivan Dragon");
        attackers.push((shivan, Entity::Player(P1)));
        t.answer_targets(P0, &[Entity::Object(target)]);
    }
    t.answer_targets(P0, &[Entity::Object(target)]);
    attack_with(t, &attackers);
    t.resolve_all();
}

#[test]
fn ojutai_the_permanent_stays_tapped_for_one_untap_step_only() {
    cr!("502.3", "603.2");
    ruling!(
        "Ojutai, Soul of Winter",
        "The ability only applies during the controller’s next untap step, even if the permanent has been the target of Ojutai’s triggered ability more than once. The permanent won’t stay tapped through multiple untap steps."
    );
    ruling!(
        "Ojutai, Soul of Winter",
        "The triggered ability can target a nonland permanent that’s already tapped. That permanent won’t untap during its controller’s next untap step."
    );
    supported("Ojutai, Soul of Winter");
    // Targeted twice (two attacking Dragons).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    ojutai_attack(&mut t, true, bears);
    assert!(t.obj_now(bears).tapped);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped, "not untapped in P1's next untap step");
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped, "untapped in the one after");
    // Already tapped when targeted.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.objects[bears.0 as usize].tapped = true;
    ojutai_attack(&mut t, false, bears);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(bears).tapped);
    t.advance_to(P0, Step::Upkeep);
    t.advance_to(P1, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn ojutai_the_effect_follows_the_permanent_to_its_new_controller() {
    cr!("502.3", "611.2c");
    ruling!(
        "Ojutai, Soul of Winter",
        "The ability tracks the permanent, but not its controller. If the permanent changes controllers before its first controller’s next untap step, then it won’t untap during its new controller’s next untap step."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    ojutai_attack(&mut t, false, bears);
    assert!(t.obj_now(bears).tapped);
    // P0 gains control of the Bears before P1's next untap step.
    give_control(&mut t, bears, P0);
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(bears).tapped, "not untapped in P0's next untap step");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn imperial_hellkite_searching_is_optional() {
    cr!("701.23a", "603.5", "702.37e");
    ruling!(
        "Imperial Hellkite",
        "You do not have to find a Dragon card if you do not want to, even if you have one in your library."
    );
    supported("Imperial Hellkite");
    let mut t = TestGame::new(2);
    t.library_top(P0, "Shivan Dragon");
    let hellkite = crate::r_s12_common::morph(&mut t, P0, "Imperial Hellkite");
    t.lands(P0, "Mountain", 8);
    let hand = t.hand_size(P0);
    let library = t.library_size(P0);
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[]);
    assert!(crate::r_s11_common::turn_face_up(&mut t, P0, hellkite));
    t.resolve_all();
    assert!(!t.obj_now(hellkite).face_down);
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.library_size(P0), library);
}
