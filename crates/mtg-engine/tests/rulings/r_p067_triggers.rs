//! Rulings batch P067 — the triggered abilities of cards that let a player cast cards
//! from exile: "becomes prepared" triggers (CR 722.3), intervening "if" clauses (CR
//! 603.4), "second card each turn", "one or more" and simultaneous events (CR 603.2c),
//! last known information (CR 608.2h), and dies triggers (CR 603.6c, 603.10a).

use crate::r_p067_common::*;
use crate::r_s01_common::{attack_with, block_and_finish, stack_library, supported, triggers_on_stack};
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn prepared(t: &TestGame, id: ObjectId) -> bool {
    t.obj_now(id).prepared.is_some()
}

/// Casts `name` from P0's hand with mana in the pool for it (one of each of its colored
/// symbols, colorless for the rest), targeting `targets`.
fn cast_spell(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    let c = mtg_engine::card::card(name);
    let cost = c.front().chars.mana_cost.clone().unwrap_or_default();
    let text = format!("{cost}");
    for sym in text.split('}').filter_map(|s| s.strip_prefix('{')) {
        let ty = match sym {
            "W" => ManaType::W,
            "U" => ManaType::U,
            "B" => ManaType::B,
            "R" => ManaType::R,
            "G" => ManaType::G,
            n => {
                mana(t, p, ManaType::C, n.parse::<u32>().unwrap_or(0));
                continue;
            }
        };
        mana(t, p, ty, 1);
    }
    let card = t.hand(p, name);
    t.g.turn.priority = Some(p);
    t.cast_with(p, card, targets)
        .unwrap_or_else(|e| panic!("casting {name} failed: {e:?}"))
}

/// P1 casts Counterspell targeting `spell`.
fn counter(t: &mut TestGame, spell: ObjectId) {
    cast_spell(t, P1, "Counterspell", &[o(spell)]);
}

// ---------------------------------------------------------------------------
// Becoming prepared
// ---------------------------------------------------------------------------

#[test]
fn defacing_duskmage_triggers_on_the_second_draw_even_if_it_arrived_after_the_first() {
    cr!("603.2", "603.2h");
    ruling!(
        "Defacing Duskmage // Vandal's Edit",
        "Defacing Duskmage doesn't need to have been under your control when the first card is drawn for its ability to trigger. As long as you control it when an opponent draws their second card in a turn, that ability will trigger."
    );
    supported("Defacing Duskmage // Vandal's Edit");
    let mut t = TestGame::new(2);
    t.g.draw_cards(P1, 1);
    t.settle();
    let d = t.battlefield(P0, "Defacing Duskmage // Vandal's Edit");
    assert!(!prepared(&t, d));
    t.g.draw_cards(P1, 1);
    t.settle();
    t.resolve_all();
    assert!(prepared(&t, d), "{}", t.dump_log());
}

#[test]
fn ian_malcolm_triggers_once_each_turn_for_each_player_on_their_second_draw() {
    cr!("603.2", "603.2h");
    ruling!(
        "Ian Malcolm, Chaotician",
        "Ian Malcolm, Chaotician's first ability can trigger only once each turn for each player. It doesn't matter if Ian was on the battlefield when the first card was drawn. If you control it when a player draws their second card in a turn, that ability will trigger."
    );
    supported("Ian Malcolm, Chaotician");
    let mut t = TestGame::new(2);
    t.g.draw_cards(P1, 1);
    t.settle();
    t.battlefield(P0, "Ian Malcolm, Chaotician");
    let l1 = t.library_size(P1);
    t.g.draw_cards(P1, 1);
    t.settle();
    t.resolve_all();
    // The second card: P1 exiles its top card.
    assert_eq!(t.library_size(P1), l1 - 2, "{}", t.dump_log());
    assert_eq!(t.g.exile.len(), 1);
    // The third: nothing.
    t.g.draw_cards(P1, 1);
    t.settle();
    t.resolve_all();
    assert_eq!(t.g.exile.len(), 1);
    // P0's second card this turn triggers it too.
    t.g.draw_cards(P0, 2);
    t.settle();
    t.resolve_all();
    assert_eq!(t.g.exile.len(), 2);
    assert_eq!(t.obj(t.g.exile[1]).owner, P0);
}

#[test]
fn dirgur_focusmage_and_emeritus_of_conflict_triggers_resolve_first_even_if_the_spell_is_countered() {
    cr!("603.3", "608.1", "722.3a");
    ruling!(
        "Dirgur Focusmage // Braingeyser",
        "Dirgur Focusmage's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    ruling!(
        "Emeritus of Conflict // Lightning Bolt",
        "Emeritus of Conflict's last ability resolves before the spell that caused it to trigger. It resolves even if that spell is countered or otherwise leaves the stack without resolving."
    );
    supported("Dirgur Focusmage // Braingeyser");
    supported("Emeritus of Conflict // Lightning Bolt");
    // Dirgur Focusmage: Overrun (mana value 5) from hand.
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Dirgur Focusmage // Braingeyser");
    let spell = cast_spell(&mut t, P0, "Overrun", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    counter(&mut t, spell);
    // Counterspell resolves; the trigger still resolves.
    t.resolve();
    assert!(t.in_graveyard(P0, "Overrun"));
    t.resolve_all();
    assert!(prepared(&t, d), "{}", t.dump_log());
    // Emeritus of Conflict: the third spell is countered.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Emeritus of Conflict // Lightning Bolt");
    for _ in 0..2 {
        cast_spell(&mut t, P0, "Shock", &[Entity::Player(P1)]);
        t.resolve_all();
    }
    let third = cast_spell(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    // The trigger is above the spell: it resolves first.
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert!(prepared(&t, e));
    assert_eq!(t.zone(third), Zone::Stack);
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Emeritus of Conflict // Lightning Bolt");
    for _ in 0..2 {
        cast_spell(&mut t, P0, "Shock", &[Entity::Player(P1)]);
        t.resolve_all();
    }
    let third = cast_spell(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    counter(&mut t, third);
    t.resolve_all();
    assert!(prepared(&t, e));
    assert_eq!(t.life(P1), 16);
}

#[test]
fn emeritus_of_conflict_counts_every_spell_cast_this_turn_itself_included() {
    cr!("603.2", "603.6a");
    ruling!(
        "Emeritus of Conflict // Lightning Bolt",
        "Emeritus of Conflict's last ability will count any spells you've cast this turn, which may include Emeritus of Conflict itself. It doesn't matter if the other spells resolved, didn't resolve, were countered, or are still on the stack. If Emeritus of Conflict was the second spell you cast this turn, the next spell you cast this turn is your third spell. If Emeritus of Conflict is your third spell in a turn, it's not on the battlefield for its triggered ability to trigger, and it will not trigger for further spells cast after it that turn."
    );
    const E: &str = "Emeritus of Conflict // Lightning Bolt";
    // A countered first spell, then Emeritus (second): the next spell is the third.
    let mut t = TestGame::new(2);
    let s = cast_spell(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    counter(&mut t, s);
    t.resolve_all();
    let e = cast_spell(&mut t, P0, E, &[]);
    t.resolve_all();
    assert!(t.on_battlefield(e));
    assert!(!prepared(&t, e));
    cast_spell(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    t.resolve_all();
    assert!(prepared(&t, e), "{}", t.dump_log());
    // Emeritus as the third spell: no trigger then or for the fourth spell.
    let mut t = TestGame::new(2);
    for _ in 0..2 {
        cast_spell(&mut t, P0, "Shock", &[Entity::Player(P1)]);
        t.resolve_all();
    }
    let e = cast_spell(&mut t, P0, E, &[]);
    t.resolve_all();
    assert!(t.on_battlefield(e));
    cast_spell(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(!prepared(&t, e));
    // A first spell still on the stack counts too.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, E);
    cast_spell(&mut t, P0, "Lightning Bolt", &[Entity::Player(P1)]);
    cast_spell(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    cast_spell(&mut t, P0, "Shock", &[Entity::Player(P1)]);
    t.settle();
    t.resolve();
    assert!(prepared(&t, e));
}

#[test]
fn end_step_prepared_triggers_check_as_the_end_step_starts() {
    cr!("603.4", "513.1");
    ruling!(
        "Eccentric Pestfinder // Turn Stones",
        "Eccentric Pestfinder's last ability will check as your end step starts to see if you gained life this turn. If you didn't, the ability won't trigger at all. It's not possible to gain life during your end step in time to have the ability trigger."
    );
    ruling!(
        "Emeritus of Woe // Demonic Tutor",
        "Emeritus of Woe's last ability will check as your end step starts to see if two or more creatures have died this turn. If they haven't, the ability won't trigger at all. It's not possible to cause creatures to die during your end step in time to have the ability trigger."
    );
    supported("Eccentric Pestfinder // Turn Stones");
    supported("Emeritus of Woe // Demonic Tutor");
    for before in [true, false] {
        // Eccentric Pestfinder: life gained before the end step, or during it.
        let mut t = TestGame::new(2);
        let p = t.battlefield(P0, "Eccentric Pestfinder // Turn Stones");
        if before {
            t.g.gain_life(P0, 1);
        }
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(t.stack_len(), usize::from(before), "{before}");
        t.g.gain_life(P0, 1);
        t.resolve_all();
        assert_eq!(prepared(&t, p), before, "{before}");
        // Emeritus of Woe: two creatures die before the end step, or during it.
        let mut t = TestGame::new(2);
        let e = t.battlefield(P0, "Emeritus of Woe // Demonic Tutor");
        let bears: Vec<ObjectId> = (0..2).map(|_| t.battlefield(P1, "Grizzly Bears")).collect();
        if before {
            for b in &bears {
                crate::r_s02_common::destroy(&mut t, *b);
            }
        }
        t.advance_to(P0, Step::End);
        t.settle();
        assert_eq!(t.stack_len(), usize::from(before), "{before}");
        if !before {
            for b in &bears {
                crate::r_s02_common::destroy(&mut t, *b);
            }
        }
        t.resolve_all();
        assert_eq!(prepared(&t, e), before, "{before}");
    }
}

#[test]
fn emeritus_of_abundance_checks_eight_lands_as_it_triggers_and_resolves() {
    cr!("603.4");
    ruling!(
        "Emeritus of Abundance // Regrowth",
        "Emeritus of Abundance's last ability checks at the moment it would trigger to see if you control eight or more lands. If you don't, the ability won't trigger at all. It's not possible to put lands onto the battlefield during your declare attackers step in time to have the ability trigger. If it does trigger, the ability will check again as it tries to resolve. If you don't control eight or more lands at that time, the ability won't resolve and none of its effects will happen."
    );
    supported("Emeritus of Abundance // Regrowth");
    const E: &str = "Emeritus of Abundance // Regrowth";
    // Seven lands: no trigger; an eighth land in the declare attackers step is too late.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, E);
    t.lands(P0, "Forest", 7);
    attack_with(&mut t, &[(e, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 0);
    t.lands(P0, "Forest", 1);
    t.resolve_all();
    assert!(!prepared(&t, e));
    // Eight lands: it triggers; one leaves before it resolves: nothing happens.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, E);
    let lands = t.lands(P0, "Forest", 8);
    attack_with(&mut t, &[(e, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    crate::r_s02_common::destroy(&mut t, lands[0]);
    t.resolve_all();
    assert!(!prepared(&t, e));
    // Control: eight lands throughout.
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, E);
    t.lands(P0, "Forest", 8);
    attack_with(&mut t, &[(e, Entity::Player(P1))]);
    t.resolve_all();
    assert!(prepared(&t, e));
}

#[test]
fn kirol_triggers_once_for_cards_leaving_the_graveyard_together() {
    cr!("603.2c", "603.10a");
    ruling!(
        "Kirol, History Buff // Pack a Punch",
        "If multiple cards leave your graveyard at the same time, Kirol's first ability will trigger only once."
    );
    supported("Kirol, History Buff // Pack a Punch");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Kirol, History Buff // Pack a Punch");
    crate::r_s07_common::graveyard_n(&mut t, P0, "Grizzly Bears", 3);
    let crypt = t.battlefield(P1, "Tormod's Crypt");
    t.activate(P1, crypt, 0, &[Entity::Player(P0)]).unwrap();
    t.resolve();
    assert_eq!(t.graveyard_size(P0), 0);
    assert_eq!(triggers_on_stack(&t, ""), 1, "{}", t.dump_log());
}

// ---------------------------------------------------------------------------
// Other triggers
// ---------------------------------------------------------------------------

#[test]
fn digsite_conservator_triggers_however_it_dies() {
    cr!("700.4", "603.6c");
    ruling!(
        "Digsite Conservator",
        "Digsite Conservator's triggered ability will trigger if it dies for any reason, not only because you sacrificed it due to its activated ability."
    );
    supported("Digsite Conservator");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Digsite Conservator");
    t.lands(P0, "Wastes", 4);
    let giant = t.library_top(P0, "Hill Giant");
    t.answer_yes(P0, true);
    crate::r_s02_common::destroy(&mut t, d);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.zone(giant) != Zone::Library(P0), "{}", t.dump_log());
    assert!(t.on_battlefield(giant) || t.in_hand(P0, "Hill Giant"));
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 4);
}

#[test]
fn geological_appraiser_triggers_when_cast_from_any_zone_but_not_when_put_onto_the_battlefield() {
    cr!("603.4", "601.2");
    ruling!(
        "Geological Appraiser",
        "Geological Appraiser's triggered ability triggers if you cast it from any zone. It doesn't trigger if you put Geological Appraiser onto the battlefield without casting it."
    );
    supported("Geological Appraiser");
    // Put onto the battlefield: no trigger.
    let mut t = TestGame::new(2);
    t.enter(P0, "Geological Appraiser");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Cast from exile (Release to the Wind lets its owner cast it).
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Geological Appraiser");
    crate::r_s25_common::cast_new(&mut t, P0, "Release to the Wind", &[o(a)]);
    t.resolve_all();
    assert!(exiled(&t, a));
    let shock = t.library_top(P0, "Shock");
    cast_free(&mut t, P0, a);
    t.resolve();
    // The discover trigger.
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert!(t.zone(shock) != Zone::Library(P0));
}

#[test]
fn caparocti_sunborn_untapped_as_its_ability_resolves_can_tap_itself() {
    cr!("508.1f", "702.20b", "701.26a");
    ruling!(
        "Caparocti Sunborn",
        "If Caparocti Sunborn is untapped as its ability is resolving (for example, if it has vigilance), it can be tapped for its own ability."
    );
    supported("Caparocti Sunborn");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Always Watching");
    let c = t.battlefield(P0, "Caparocti Sunborn");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let shock = t.library_top(P0, "Shock");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(c), o(bears)]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(c, Entity::Player(P1))]);
    assert!(!t.obj_now(c).tapped, "vigilance");
    t.resolve_all();
    assert!(t.obj_now(c).tapped && t.obj_now(bears).tapped, "{}", t.dump_log());
    assert!(t.zone(shock) != Zone::Library(P0));
}

#[test]
fn daxos_stays_blocked_when_the_blockers_power_rises_to_3() {
    cr!("509.1h", "506.4");
    ruling!(
        "Daxos of Meletis",
        "If Daxos is blocked by a creature, raising that creature's power to 3 or greater won't change or undo the block."
    );
    let mut t = TestGame::new(2);
    let daxos = t.battlefield(P0, "Daxos of Meletis");
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(daxos, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(bears, daxos)]));
    t.advance_to(P0, Step::DeclareBlockers);
    cast_spell(&mut t, P1, "Giant Growth", &[o(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
    let c = t.g.combat.as_ref().unwrap();
    assert!(c.blockers_of(daxos).contains(&bears));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.in_graveyard(P0, "Daxos of Meletis"));
    assert_eq!(t.life(P1), 20);
}

/// Puts `card` into exile as if `by` had exiled it, with `n` counters of `kind` on it.
fn exile_by(t: &mut TestGame, card: ObjectId, by: PlayerId, kind: &str, n: u32) -> ObjectId {
    let x = t
        .g
        .move_object(card, Zone::Exile, mtg_engine::events::MoveCause::Exile, Some(by))
        .unwrap();
    t.g.obj_mut(x).counters.insert(kind.into(), n);
    x
}

#[test]
fn dragon_kamis_egg_triggers_for_each_and_casts_once_per_resolution() {
    cr!("603.2", "603.10a", "608.2g");
    ruling!(
        "The Dragon-Kami Reborn // Dragon-Kami's Egg",
        "If Dragon-Kami's Egg and one or more Dragons you control die at the same time, the ability of Dragon-Kami's Egg will trigger once for each of them. Each time one of those abilities resolves, you'll be able to cast a creature spell from among cards you own in exile with hatchling counters on them."
    );
    // The ruling says "hatchling"; the card's counters are hatching counters.
    supported("The Dragon-Kami Reborn // Dragon-Kami's Egg");
    let mut t = TestGame::new(2);
    let egg = crate::r_s17_common::enter_transformed(
        &mut t,
        P0,
        "The Dragon-Kami Reborn // Dragon-Kami's Egg",
    );
    assert_eq!(t.obj_now(egg).chars.name, "Dragon-Kami's Egg");
    t.battlefield(P0, "Shivan Dragon");
    let a = t.library_top(P0, "Hill Giant");
    let a = exile_by(&mut t, a, P0, "hatching", 1);
    let b = t.library_top(P0, "Serra Angel");
    let b = exile_by(&mut t, b, P0, "hatching", 1);
    t.answer_choose_moved(P0, &[o(a)]);
    t.answer_yes(P0, true);
    t.answer_choose_moved(P0, &[o(b)]);
    t.answer_yes(P0, true);
    cast_spell(&mut t, P0, "Wrath of God", &[]);
    t.resolve();
    assert_eq!(triggers_on_stack(&t, ""), 2, "{}", t.dump_log());
    t.resolve_all();
    assert!(t.on_battlefield(a) && t.on_battlefield(b), "{}", t.dump_log());
}

#[test]
fn keldon_flamesage_uses_its_last_known_power_for_x() {
    cr!("608.2h", "113.7a");
    ruling!(
        "Keldon Flamesage",
        "If Keldon Flamesage leaves the battlefield in response to its triggered ability, use its power as it last existed on the battlefield to determine the value of X."
    );
    supported("Keldon Flamesage");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Keldon Flamesage");
    t.g.players[0].library.clear();
    let lib = stack_library(
        &mut t,
        P0,
        &["Hill Giant", "Hill Giant", "Hill Giant", "Hill Giant", "Lava Axe", "Hill Giant"],
    );
    t.answer(P0, DecisionKind::Number, Answer::Number(0));
    attack_with(&mut t, &[(k, Entity::Player(P1))]);
    // In response: Giant Growth (power 5), then it leaves.
    cast_spell(&mut t, P0, "Giant Growth", &[o(k)]);
    t.resolve();
    assert_eq!(t.pt(k), (5, 6));
    crate::r_s02_common::destroy(&mut t, k);
    t.answer_choose_moved(P0, &[o(lib[4])]);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    // X = 5: Lava Axe (mana value 5), the fifth card, was found and cast.
    assert_eq!(t.life(P1), 15, "{}", t.dump_log());
}

#[test]
fn maralen_triggers_for_each_elf_or_faerie_entering_with_it() {
    cr!("603.6a", "603.2");
    ruling!(
        "Maralen, Fae Ascendant",
        "If Maralen enters at the same time as one or more other Elves or Faeries you control, its first ability will trigger for each of those other Elves and Faeries as well as itself."
    );
    let mut t = TestGame::new(2);
    crate::r_s24_common::enter_together(
        &mut t,
        &[
            (P0, "Maralen, Fae Ascendant"),
            (P0, "Llanowar Elves"),
            (P0, "Faerie Miscreant"),
        ],
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, ""), 3, "{}", t.dump_log());
    let l = t.library_size(P1);
    t.resolve_all();
    assert_eq!(t.library_size(P1), l - 6);
}

#[test]
fn primeval_spawn_exiled_by_its_own_replacement_never_leaves_the_battlefield() {
    cr!("614.6", "603.6c");
    ruling!(
        "Primeval Spawn",
        "If Primeval Spawn is exiled by its first ability as it tries to enter the battlefield, it goes directly to the exile zone without entering the battlefield first. Since it never enters or leaves the battlefield, its last ability won’t trigger."
    );
    supported("Primeval Spawn");
    let mut t = TestGame::new(2);
    let l = t.library_size(P0);
    let id = t.g.create_card_object(mtg_engine::card::card("Primeval Spawn"), P0, Zone::Graveyard(P0));
    t.g.players[0].graveyard.push(id);
    let r = t
        .g
        .move_object(id, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, Some(P0));
    t.settle();
    let now = t.g.current(id);
    assert_eq!(t.zone(now), Zone::Exile, "{r:?}");
    assert!(!t.g.turn_events.iter().any(|e| matches!(
        e,
        mtg_engine::events::Event::ZoneChange { to: Zone::Battlefield, .. }
    )));
    assert_eq!(t.stack_len(), 0);
    t.resolve_all();
    assert_eq!(t.library_size(P0), l);
}

#[test]
fn primeval_spawn_entering_as_a_copy_of_another_permanent_isnt_exiled() {
    cr!("614.12", "707.9", "614.6");
    ruling!(
        "Primeval Spawn",
        "Conversely, if an effect would cause Primeval Spawn to enter the battlefield as a copy of another permanent, it will enter the battlefield as a copy of that permanent whether or not it was cast or mana was spent to cast it."
    );
    supported("Essence of the Wild");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Essence of the Wild");
    let s = t.enter(P0, "Primeval Spawn");
    assert!(t.on_battlefield(s), "{}", t.dump_log());
    assert_eq!(t.obj_now(s).chars.name, "Essence of the Wild");
    assert_eq!(t.pt(s), (6, 6));
}

#[test]
fn sanwell_as_an_attacking_artifact_creature_prevents_damage_to_itself() {
    cr!("615.1", "506.4");
    ruling!(
        "Sanwell, Avenger Ace",
        "If Sanwell, Avenger Ace is somehow an artifact creature, its first ability will see itself while it is attacking."
    );
    supported("Sanwell, Avenger Ace");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Sanwell, Avenger Ace");
    let coat = t.battlefield(P0, "Liquimetal Coating");
    t.activate(P0, coat, 0, &[o(s)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(s).is(mtg_engine::types::CardType::Artifact));
    t.answer_choose_moved(P0, &[]);
    attack_with(&mut t, &[(s, Entity::Player(P1))]);
    t.resolve_all();
    cast_spell(&mut t, P1, "Shock", &[o(s)]);
    t.resolve_all();
    assert!(t.on_battlefield(s), "{}", t.dump_log());
    assert_eq!(t.obj_now(s).damage, 0);
}

#[test]
fn kaya_two_ghostform_abilities_return_the_card_once_and_make_two_spirits() {
    cr!("603.2", "603.10a", "608.2b");
    ruling!(
        "Kaya the Inexorable",
        "If a nontoken creature gains more than one instance of that triggered ability, each of them will trigger separately. You can return the card to its owner’s hand only once, but you’ll create a Spirit creature token for each of those abilities."
    );
    supported("Kaya the Inexorable");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    for _ in 0..2 {
        let k = t.battlefield(P0, "Kaya the Inexorable");
        t.activate(P0, k, 0, &[o(bears)]).unwrap();
        t.resolve_all();
    }
    assert_eq!(t.counters(bears, "ghostform"), 2);
    crate::r_s02_common::destroy(&mut t, bears);
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(crate::r_s01_common::tokens(&t, P0).len(), 2);
}

#[test]
fn robber_of_the_rich_checks_hand_sizes_on_attack_and_on_resolution_only() {
    cr!("603.4");
    ruling!(
        "Robber of the Rich",
        "If the defending player doesn't have more cards in hand than you immediately after Robber of the Rich attacks, its ability doesn't trigger at all. If that player doesn't have more cards in hand as the ability resolves, the ability does nothing. It doesn't matter if that player has fewer or the same number of cards in hand in between these two times, or if they have fewer or the same number of cards in hand later."
    );
    // Equal hands: no trigger.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Robber of the Rich");
    t.hand(P0, "Forest");
    t.hand(P1, "Forest");
    attack_with(&mut t, &[(r, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 0);
    // More, then not as it resolves: nothing.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Robber of the Rich");
    let f = t.hand(P1, "Forest");
    let top = t.library_top(P1, "Grizzly Bears");
    attack_with(&mut t, &[(r, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, f, Zone::Graveyard(P1));
    t.resolve_all();
    assert_eq!(t.zone(top), Zone::Library(P1));
    // More at both times, fewer in between: it exiles the card.
    let mut t = TestGame::new(2);
    let r = t.battlefield(P0, "Robber of the Rich");
    let f = t.hand(P1, "Forest");
    let top = t.library_top(P1, "Grizzly Bears");
    attack_with(&mut t, &[(r, Entity::Player(P1))]);
    let f2 = move_to(&mut t, f, Zone::Graveyard(P1));
    move_to(&mut t, f2, Zone::Hand(P1));
    t.resolve_all();
    assert!(exiled(&t, top));
    // Fewer later: the permission doesn't depend on hand sizes.
    let f = t.g.player(P1).hand[0];
    move_to(&mut t, f, Zone::Graveyard(P1));
    block_and_finish(&mut t, P1, &[]);
    t.advance_to(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::R, 2);
    assert!(castable(&mut t, P0, top));
}
