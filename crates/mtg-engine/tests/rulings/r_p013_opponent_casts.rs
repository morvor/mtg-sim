//! Rulings batch P013 — abilities that trigger when an opponent casts a spell (CR 601.2i,
//! 603.2): what the spell is as it becomes cast (a face-down spell has no name; delve
//! exiles cards before the trigger checks the graveyard), counting spells per player and
//! per turn, "if no mana was spent", spells that are already cast when an ability starts
//! to exist, and Perplexing Chimera's exchange of control with the triggering spell.

use crate::r_s01_common::{supported, watch};
use crate::r_s04_common::add_mana;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

/// `p` casts the card `name` from hand (lands for its mana cost added) with the given
/// targets, then triggers are put on the stack. Returns the spell.
fn cast_from_hand(t: &mut TestGame, p: PlayerId, name: &str, targets: &[Entity]) -> ObjectId {
    lands_for_cost(t, p, name);
    let card = t.hand(p, name);
    let spell = t.cast_with(p, card, targets).unwrap();
    t.settle();
    spell
}

fn controller(t: &TestGame, id: ObjectId) -> PlayerId {
    t.obj_now(id).controller
}

// ---------------------------------------------------------------------------------------
// Dragonlord Kolaghan
// ---------------------------------------------------------------------------------------

#[test]
fn kolaghan_doesnt_trigger_for_a_face_down_spell() {
    cr!("708.4", "708.2", "201.2");
    ruling!(
        "Dragonlord Kolaghan",
        "Casting a spell face down won’t cause Dragonlord Kolaghan’s last ability to trigger. A spell with no name can’t have the same name as any other card."
    );
    supported("Dragonlord Kolaghan");
    supported("Exalted Angel");
    for face_down in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Dragonlord Kolaghan");
        t.graveyard(P1, "Exalted Angel");
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Plains", 6);
        let angel = t.hand(P1, "Exalted Angel");
        let method = if face_down {
            CastMethod::FaceDown(KeywordKind::Morph)
        } else {
            CastMethod::Normal
        };
        t.cast(P1, angel).method(method).go();
        t.resolve_all();
        assert_eq!(t.life(P1), if face_down { 20 } else { 10 });
    }
}

#[test]
fn kolaghan_doesnt_trigger_if_delve_exiled_the_same_named_cards() {
    cr!("702.66a", "601.2h", "601.2i");
    ruling!(
        "Dragonlord Kolaghan",
        "If an opponent casts a creature spell with delve and exiles all cards from their graveyard with the same name to help pay its cost, Dragonlord Kolaghan’s ability won’t trigger."
    );
    supported("Gurmag Angler");
    for exile_angler in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Dragonlord Kolaghan");
        let gy_angler = t.graveyard(P1, "Gurmag Angler");
        let others: Vec<ObjectId> = (0..6).map(|_| t.graveyard(P1, "Grizzly Bears")).collect();
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Swamp", 1);
        // Exile six cards for the {6}: the Angler and five others, or six others.
        let mut exiled: Vec<Entity> = others.iter().map(|o| obj(*o)).collect();
        if exile_angler {
            exiled[0] = obj(gy_angler);
        }
        t.answer_choose(P1, &exiled);
        let angler = t.hand(P1, "Gurmag Angler");
        t.cast(P1, angler).go();
        assert_eq!(t.zone(gy_angler) == Zone::Exile, exile_angler);
        t.resolve_all();
        assert_eq!(t.life(P1), if exile_angler { 20 } else { 10 });
    }
}

// ---------------------------------------------------------------------------------------
// Lavinia, Azorius Renegade
// ---------------------------------------------------------------------------------------

#[test]
fn lavinia_compares_the_mana_value_not_the_cost_paid() {
    cr!("202.3", "118.9c", "702.34a");
    ruling!(
        "Lavinia, Azorius Renegade",
        "Effects that modify or replace the cost to cast a spell (such as flashback or casting a spell without paying its mana cost due to suspend) don't affect the spell's mana value, so they won't change whether Lavinia's first ability restricts that spell from being cast."
    );
    supported("Lavinia, Azorius Renegade");
    // Think Twice: mana value 2, flashback {2}{U} (three mana).
    for lands in [1usize, 2] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Lavinia, Azorius Renegade");
        t.set_step(P1, Step::PrecombatMain);
        t.lands(P1, "Island", lands);
        add_mana(&mut t, P1, ManaType::U, 3 - lands as u32);
        let tt = t.graveyard(P1, "Think Twice");
        let r = t
            .cast(P1, tt)
            .method(CastMethod::Keyword(KeywordKind::Flashback))
            .try_go();
        assert_eq!(r.is_ok(), lands == 2, "{lands} lands");
    }
}

#[test]
fn lavinia_doesnt_counter_a_free_spell_whose_additional_mana_cost_was_paid() {
    cr!("118.9", "601.2f", "702.33a");
    ruling!(
        "Lavinia, Azorius Renegade",
        "However, if that spell also has additional costs that require mana, paying that mana will stop Lavinia's last ability from triggering."
    );
    supported("Omniscience");
    // Burst Lightning, cast without paying its mana cost; kicked ({4}) or not.
    for kicked in [true, false] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Lavinia, Azorius Renegade");
        t.battlefield(P1, "Omniscience");
        t.lands(P1, "Mountain", 4);
        t.set_step(P1, Step::PrecombatMain);
        let bl = t.hand(P1, "Burst Lightning");
        t.cast(P1, bl)
            .method(CastMethod::Free)
            .kicked(kicked)
            .target(Entity::Player(P0))
            .go();
        t.resolve_all();
        assert_eq!(t.life(P0), if kicked { 16 } else { 20 }, "kicked {kicked}");
    }
}

#[test]
fn spells_lavinia_will_counter_still_trigger_and_count() {
    cr!("601.2i", "603.2", "701.6a");
    ruling!(
        "Lavinia, Azorius Renegade",
        "Players may cast spells that they know Lavinia's second ability will counter. Any abilities that trigger when spells are cast will trigger and resolve if appropriate, and any effects that count spells cast will count those spells if appropriate."
    );
    supported("Magebane Lizard");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lavinia, Azorius Renegade");
    t.battlefield(P0, "Magebane Lizard");
    t.battlefield(P1, "Omniscience");
    t.lands(P1, "Island", 3);
    t.set_step(P1, Step::PrecombatMain);
    for (n, life) in [(1, 19), (2, 17)] {
        let div = t.hand(P1, "Divination");
        t.cast(P1, div).method(CastMethod::Free).go();
        t.resolve_all();
        // Countered (no cards drawn), but Magebane Lizard counted it.
        assert_eq!(t.life(P1), life, "spell {n}");
        assert_eq!(t.hand_size(P1), 0);
    }
}

// ---------------------------------------------------------------------------------------
// Erayo, Soratami Ascendant // Erayo's Essence
// ---------------------------------------------------------------------------------------

/// Erayo, flipped, on `p`'s battlefield.
fn erayos_essence(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let e = t.battlefield(p, "Erayo, Soratami Ascendant");
    assert!(mtg_engine::flip::flip(&mut t.g, e));
    t.g.recompute();
    assert_eq!(t.obj_now(e).chars.name, "Erayo's Essence");
    e
}

#[test]
fn erayos_essence_counters_only_each_opponents_first_spell_each_turn() {
    cr!("603.2", "701.6a", "102.2");
    ruling!(
        "Erayo, Soratami Ascendant",
        "Erayo’s Essence triggers only for the first spell an opponent casts in a given turn. It doesn’t trigger off any other spell that player casts after the first."
    );
    ruling!(
        "Erayo, Soratami Ascendant",
        "If you have multiple opponents, Erayo’s Essence can trigger once for each opponent."
    );
    supported("Erayo, Soratami Ascendant");
    let mut t = TestGame::new(3);
    erayos_essence(&mut t, P0);
    // P1: the first Shock is countered, the second isn't.
    cast_from_hand(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    cast_from_hand(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    // P2's first spell is countered too; their second isn't.
    cast_from_hand(&mut t, P2, "Shock", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
    cast_from_hand(&mut t, P2, "Shock", &[Entity::Player(P0)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 16);
}

#[test]
fn erayo_flipping_while_the_first_spell_is_on_the_stack_doesnt_counter_it() {
    cr!("603.2", "603.3", "710.2");
    ruling!(
        "Erayo, Soratami Ascendant",
        "If Erayo, Soratami Ascendant flips while an opponent’s first spell for the turn is still on the stack, the ability of Erayo’s Essence doesn’t trigger because the opponent’s spell has already been cast. The spell isn’t countered."
    );
    let mut t = TestGame::new(2);
    let erayo = t.battlefield(P0, "Erayo, Soratami Ascendant");
    // Three spells this turn by P0.
    for _ in 0..3 {
        cast_from_hand(&mut t, P0, "Ornithopter", &[]);
        t.resolve_all();
    }
    // P1's first spell is the fourth spell of the turn: Erayo flips with it on the stack.
    lands_for_cost(&mut t, P1, "Shock");
    let shock = t.hand(P1, "Shock");
    let shock = t.cast_with(P1, shock, &[Entity::Player(P0)]).unwrap();
    t.settle();
    t.resolve();
    assert_eq!(t.obj_now(erayo).chars.name, "Erayo's Essence");
    assert_eq!(t.zone(shock), Zone::Stack);
    t.resolve_all();
    assert_eq!(t.life(P0), 18);
}

// ---------------------------------------------------------------------------------------
// Counting spells: Ichneumon Druid, Magebane Lizard, Mindsparker
// ---------------------------------------------------------------------------------------

#[test]
fn ichneumon_druid_counts_each_players_instants_separately() {
    cr!("603.2", "102.2");
    ruling!(
        "Ichneumon Druid",
        "It tracks the instant count for each player separately."
    );
    supported("Ichneumon Druid");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Ichneumon Druid");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P0, "Plains");
    for (p, life) in [(P1, 20), (P2, 20), (P1, 16), (P2, 16)] {
        cast_from_hand(&mut t, p, "Giant Growth", &[obj(bears)]);
        t.resolve_all();
        assert_eq!(t.life(p), life);
    }
}

#[test]
fn magebane_lizard_counts_spells_cast_before_it_entered() {
    cr!("603.2", "601.2i");
    ruling!(
        "Magebane Lizard",
        "Noncreature spells that were cast before Magebane Lizard entered the battlefield count."
    );
    supported("Magebane Lizard");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    cast_from_hand(&mut t, P1, "Divination", &[]);
    t.resolve_all();
    t.battlefield(P0, "Magebane Lizard");
    // A creature spell doesn't count or trigger.
    cast_from_hand(&mut t, P1, "Grizzly Bears", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    cast_from_hand(&mut t, P1, "Divination", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn mindsparker_triggers_once_for_a_white_and_blue_spell() {
    cr!("603.2", "105.2");
    ruling!(
        "Mindsparker",
        "If an opponent casts a spell that's both white and blue, Mindsparker's last ability triggers only once."
    );
    supported("Mindsparker");
    supported("Azorius Charm");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mindsparker");
    t.set_step(P1, Step::PrecombatMain);
    lands_for_cost(&mut t, P1, "Azorius Charm");
    let charm = t.hand(P1, "Azorius Charm");
    // "Draw a card."
    t.cast(P1, charm).modes(&[1]).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

// ---------------------------------------------------------------------------------------
// Swindler's Scheme
// ---------------------------------------------------------------------------------------

#[test]
fn swindlers_scheme_decides_to_reveal_without_looking() {
    cr!("401.2", "603.5");
    ruling!(
        "Swindler's Scheme",
        "You don't get to look at the top card of your library before choosing whether to reveal it."
    );
    supported("Swindler's Scheme");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Swindler's Scheme");
    let top = t.library_top(P0, "Lightning Bolt");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. }),
        |g| {
            g.players[0].library.iter().any(|id| {
                g.obj(*id).chars.name == "Lightning Bolt"
                    && mtg_engine::player_control::can_see(g, PlayerId(0), *id)
            })
        },
    );
    t.answer_yes(P0, true);
    t.answer_targets(P1, &[Entity::Player(P0)]);
    t.set_step(P1, Step::PrecombatMain);
    let shock = cast_from_hand(&mut t, P1, "Shock", &[Entity::Player(P0)]);
    t.resolve();
    // Asked once, without being able to see the card.
    assert_eq!(*seen.lock().unwrap(), vec![false]);
    // Revealed: it shares a card type with Shock, so Shock is countered.
    assert_eq!(t.zone(shock), Zone::Graveyard(P1));
    // P1 may cast the revealed card (not the countered Shock again): the Bolt, at P0.
    assert_eq!(t.zone(top), Zone::Stack);
    assert_eq!(t.obj_now(top).controller, P1);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert!(t.in_graveyard(P0, "Lightning Bolt"));
    assert!(t.in_graveyard(P1, "Shock"));
}

// ---------------------------------------------------------------------------------------
// Perplexing Chimera
// ---------------------------------------------------------------------------------------

#[test]
fn chimera_exchanges_with_any_spell_deciding_as_the_trigger_resolves() {
    cr!("701.12a", "603.5", "608.2");
    ruling!(
        "Perplexing Chimera",
        "You may exchange control of Perplexing Chimera and any spell cast by an opponent, not just one with targets."
    );
    ruling!(
        "Perplexing Chimera",
        "You make the decision whether to exchange control of Perplexing Chimera and the spell as the triggered ability resolves."
    );
    supported("Perplexing Chimera");
    let mut t = TestGame::new(2);
    let chimera = t.battlefield(P0, "Perplexing Chimera");
    t.set_step(P1, Step::PrecombatMain);
    let from = t.asked().len();
    let div = cast_from_hand(&mut t, P1, "Divination", &[]);
    let yes_nos = |t: &TestGame| {
        t.asked()[from..]
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, Decision::YesNo { .. }))
            .count()
    };
    // Triggered and on the stack: nothing asked yet.
    assert_eq!(t.stack_len(), 2);
    assert_eq!(yes_nos(&t), 0);
    t.answer_yes(P0, true);
    t.resolve();
    assert!(yes_nos(&t) >= 1);
    assert_eq!(controller(&t, chimera), P1);
    assert_eq!(controller(&t, div), P0);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn chimera_new_targets_must_be_legal_or_stay_the_same() {
    cr!("701.12a", "115.7", "115.7d");
    ruling!(
        "Perplexing Chimera",
        "You may change any of the spell’s targets. If you change a target, you must choose a legal target for the spell. If you can’t, you must leave the target the same (even if that target is now illegal)."
    );
    // An illegal new target (a land, for Lightning Bolt) isn't accepted: Bolt still
    // targets P0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Perplexing Chimera");
    let forest = t.battlefield(P1, "Forest");
    t.set_step(P1, Step::PrecombatMain);
    cast_from_hand(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(forest)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 17);
    assert!(t.on_battlefield(forest));
    // A legal new target is.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Perplexing Chimera");
    t.set_step(P1, Step::PrecombatMain);
    cast_from_hand(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 17);
    // No other legal target (P1's creature is the only creature): the target stays.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Perplexing Chimera");
    t.set_step(P1, Step::PrecombatMain);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let growth = cast_from_hand(&mut t, P1, "Giant Growth", &[obj(bears)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(controller(&t, growth), P0);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn chimera_changing_targets_doesnt_trigger_heroic() {
    cr!("701.12a", "115.7", "603.2", "207.2c");
    ruling!(
        "Perplexing Chimera",
        "Gaining control of a spell and changing its targets won’t cause any heroic abilities of the new targets to trigger."
    );
    supported("Phalanx Leader");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Perplexing Chimera");
    let leader = t.battlefield(P0, "Phalanx Leader");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_from_hand(&mut t, P1, "Giant Growth", &[obj(bears)]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[obj(leader)]);
    t.resolve_all();
    // Giant Growth went to Phalanx Leader, but heroic didn't trigger.
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.counters(leader, "+1/+1"), 0);
    assert_eq!(t.pt(leader), (4, 4));
}

#[test]
fn chimera_exchange_when_you_control_neither() {
    cr!("701.12a", "701.12b", "603.3");
    ruling!(
        "Perplexing Chimera",
        "In some unusual cases, you may not control Perplexing Chimera when its triggered ability resolves (perhaps because the triggered ability triggered again and resolved while the original ability was on the stack). In these cases, you can exchange control of Perplexing Chimera and the spell that causes the ability to trigger, even if you control neither of them. If you do, you’ll be able to change targets of the spell, not the spell’s new controller."
    );
    // P1 casts a Bolt at P0; with the trigger on the stack, P2 casts a Shock at P0 and
    // the second trigger resolves first: P2 takes the Chimera, P0 the Shock.
    let mut t = TestGame::new(3);
    let chimera = t.battlefield(P0, "Perplexing Chimera");
    t.set_step(P1, Step::PrecombatMain);
    let bolt = cast_from_hand(&mut t, P1, "Lightning Bolt", &[Entity::Player(P0)]);
    let shock = cast_from_hand(&mut t, P2, "Shock", &[Entity::Player(P0)]);
    assert_eq!(t.stack_len(), 4);
    // The Shock's trigger: exchange, and redirect the Shock at P1.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.resolve();
    assert_eq!(controller(&t, chimera), P2);
    assert_eq!(controller(&t, shock), P0);
    // The Shock resolves (at P1) before the Bolt's trigger.
    t.resolve();
    assert_eq!(t.life(P1), 18);
    // The Bolt's trigger: P0 controls neither the Chimera (P2) nor the Bolt (P1). P0
    // exchanges them and P0 (not the Bolt's new controller, P2) redirects the Bolt at
    // P2.
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[Entity::Player(P2)]);
    t.resolve();
    assert_eq!(controller(&t, chimera), P1);
    assert_eq!(controller(&t, bolt), P2);
    assert!(t.asked()[from..]
        .iter()
        .all(|(p, d)| *p == P0 || matches!(d, Decision::Priority { .. })));
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 17);
}
