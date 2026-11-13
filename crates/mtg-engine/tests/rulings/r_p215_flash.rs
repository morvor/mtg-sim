//! Rulings batch P215 — flanking (CR 702.25) and flash (CR 702.8): multiple flanking
//! instances, "can't be countered" for spells with flash you cast, casting "as though it
//! had flash", and timing permissions for cards cast from other zones.

use crate::r_p215_fight::grant;
use crate::r_s01_common::*;
use crate::r_s02_common::*;
use crate::r_s04_common::{add_mana, untapped_lands};
use crate::r_s05_common::*;
use crate::r_s07_common::*;
use crate::r_s21_common::castable;
use crate::r_s25_common::lands_for_cost;
use mtg_engine::ability::*;
use mtg_engine::decision::{Action, Answer, SpecialAction};
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

const FLASHBACK: CastMethod = CastMethod::Keyword(KeywordKind::Flashback);
const FORETELL: CastMethod = CastMethod::Keyword(KeywordKind::Foretell);

/// `id` loses `kw` until end of turn.
pub(crate) fn lose(t: &mut TestGame, id: ObjectId, kw: KeywordKind) {
    run_from(
        t,
        P0,
        None,
        Effect::Modify {
            what: Sel::Target(0),
            mods: vec![Modification::RemoveKeyword(kw)],
            duration: Duration::EndOfTurn,
        },
        &[Entity::Object(id)],
    );
}

/// P0 attacks with `attacker`; P1 blocks it with `blocker`; the game advances to the
/// declare blockers step and resolves the triggers there.
fn blocked_by(t: &mut TestGame, attacker: ObjectId, blocker: ObjectId) {
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    t.answer(
        P1,
        DecisionKind::Blockers,
        Answer::Blockers(vec![(blocker, attacker)]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    t.resolve_all();
}

#[test]
fn cavalry_master_adds_an_instance_only_to_creatures_that_have_flanking() {
    cr!("702.25a", "702.25b", "613.1f");
    ruling!(
        "Cavalry Master",
        "If a creature without flanking is given flanking by another effect, then this ability will grant a second instance. Similarly, if a creature loses flanking, then it won’t get the Calvary Master instance either."
    );
    supported("Cavalry Master");
    // A creature given flanking: two instances, the blocker gets -2/-2.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cavalry Master");
    let bears = t.battlefield(P0, "Grizzly Bears");
    grant(&mut t, P0, bears, Keyword::new(KeywordKind::Flanking));
    let giant = t.battlefield(P1, "Hill Giant");
    blocked_by(&mut t, bears, giant);
    assert_eq!(t.pt(giant), (1, 1));
    // A creature that lost flanking: no instance at all.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cavalry Master");
    let knight = t.battlefield(P0, "Knight of Valor");
    lose(&mut t, knight, KeywordKind::Flanking);
    let giant = t.battlefield(P1, "Hill Giant");
    blocked_by(&mut t, knight, giant);
    assert_eq!(t.pt(giant), (3, 3));
}

#[test]
fn knight_of_valors_ability_isnt_flanking() {
    cr!("702.25a", "602.2");
    ruling!(
        "Knight of Valor",
        "The second ability is not Flanking and is not removed by an effect that removes Flanking."
    );
    supported("Knight of Valor");
    let mut t = TestGame::new(2);
    let knight = t.battlefield(P0, "Knight of Valor");
    lose(&mut t, knight, KeywordKind::Flanking);
    let giant = t.battlefield(P1, "Hill Giant");
    blocked_by(&mut t, knight, giant);
    assert_eq!(t.pt(giant), (3, 3));
    t.lands(P0, "Plains", 2);
    crate::r_s06_common::activate_containing(&mut t, P0, knight, "without flanking")
        .expect("activate");
    t.resolve_all();
    assert_eq!(t.pt(giant), (2, 2));
}

/// P0 controls Cunning Nightbonder (optional) and casts Ambush Viper (flash); returns it.
fn viper(t: &mut TestGame) -> ObjectId {
    t.lands(P0, "Forest", 2);
    let v = t.hand(P0, "Ambush Viper");
    t.cast(P0, v).go()
}

/// P1 casts `counter` targeting `spell`.
fn p1_counters(t: &mut TestGame, counter: &str, spell: ObjectId) {
    lands_for_cost(t, P1, counter);
    let c = t.hand(P1, counter);
    t.cast(P1, c).target(Entity::Object(spell)).go();
}

#[test]
fn nightbonder_spells_can_be_targeted_by_counterspells_whose_other_effects_happen() {
    cr!("101.2", "608.2c", "702.8a");
    ruling!(
        "Cunning Nightbonder",
        "A spell or ability that counters spells can still target spells with flash you cast. When that spell or ability resolves, your spell won't be countered, but any additional effects of the countering spell or ability will still happen."
    );
    supported("Cunning Nightbonder");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cunning Nightbonder");
    let v = viper(&mut t);
    let hand = t.hand_size(P1);
    // Exclude: "Counter target creature spell. Draw a card."
    p1_counters(&mut t, "Exclude", v);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(v)));
    assert_eq!(t.hand_size(P1), hand + 1);
}

#[test]
fn nightbonder_entering_after_the_spell_was_cast_still_protects_it() {
    cr!("101.2", "702.8a", "611.3a");
    ruling!(
        "Cunning Nightbonder",
        "If Cunning Nightbonder enters the battlefield while a spell with flash that you cast is on the stack, that spell can't be countered, even though you don't get the {1} discount."
    );
    supported("Cunning Nightbonder");
    let mut t = TestGame::new(2);
    let v = viper(&mut t);
    // Both Forests paid the full {1}{G}.
    assert_eq!(untapped_lands(&t, P0), 0);
    enter(&mut t, P0, "Cunning Nightbonder");
    p1_counters(&mut t, "Cancel", v);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(v)));
}

#[test]
fn nightbonder_doesnt_protect_flash_spells_you_didnt_cast() {
    cr!("101.2", "702.8a", "707.10");
    ruling!(
        "Cunning Nightbonder",
        "Spells with flash that you control but didn't cast (perhaps because they're copies of a spell or because you took control of them from another player) can be countered."
    );
    supported("Cunning Nightbonder");
    // A copy of P0's Ambush Viper spell.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cunning Nightbonder");
    let v = viper(&mut t);
    run_from(
        &mut t,
        P0,
        None,
        Effect::CopySpell {
            what: Sel::Target(0),
            count: Value::c(1),
            new_targets: false,
        },
        &[Entity::Object(v)],
    );
    assert_eq!(t.stack_len(), 2);
    let copy = *t.g.stack.last().unwrap();
    assert_ne!(copy, v);
    p1_counters(&mut t, "Cancel", copy);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ambush Viper").len(), 1);
    // P1's Ambush Viper spell, now controlled by P0.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Cunning Nightbonder");
    t.lands(P1, "Forest", 2);
    let v = t.hand(P1, "Ambush Viper");
    let v = t.cast(P1, v).go();
    run_from(
        &mut t,
        P0,
        None,
        Effect::GainControl {
            what: Sel::Target(0),
            who: PlayerRef::You,
            duration: Duration::Permanent,
        },
        &[Entity::Object(v)],
    );
    assert_eq!(t.obj_now(v).controller, P0);
    p1_counters(&mut t, "Cancel", v);
    t.resolve_all();
    assert!(t.named_on_battlefield("Ambush Viper").is_empty());
}

#[test]
fn annie_flash_triggers_only_if_cast() {
    cr!("603.4", "601.2");
    ruling!(
        "Annie Flash, the Veteran",
        "Annie Flash's second ability triggers if you cast it from any zone. It doesn't trigger if you put Annie Flash onto the battlefield without casting it."
    );
    supported("Annie Flash, the Veteran");
    // Put onto the battlefield: no trigger.
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Grizzly Bears");
    enter(&mut t, P0, "Annie Flash, the Veteran");
    assert_eq!(t.stack_len(), 0);
    // Cast: it returns the Bears.
    let mut t = TestGame::new(2);
    let bears2 = t.graveyard(P0, "Grizzly Bears");
    cast_from_hand(&mut t, "Annie Flash, the Veteran");
    t.answer_targets(P0, &[Entity::Object(bears2)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

fn cast_from_hand(t: &mut TestGame, name: &str) -> ObjectId {
    lands_for_cost(t, P0, name);
    let c = t.hand(P0, name);
    t.cast(P0, c).go()
}

#[test]
fn annie_flash_exiled_cards_follow_normal_timing_and_costs() {
    cr!("305.1", "305.2", "601.2f");
    ruling!(
        "Annie Flash, the Veteran",
        "You pay all costs and follow all normal timing rules for cards played from exile with Annie Flash's last ability. For example, if the exiled card is a land card, you may play it only during your main phase while the stack is empty."
    );
    supported("Annie Flash, the Veteran");
    let mut t = TestGame::new(2);
    let annie = t.battlefield(P0, "Annie Flash, the Veteran");
    let cards = stack_library(&mut t, P0, &["Forest", "Grizzly Bears"]);
    t.g.tap(annie);
    t.g.flush_events();
    t.resolve_all();
    let forest = t.g.current(cards[0]);
    let bears = t.g.current(cards[1]);
    assert_eq!(t.zone(forest), Zone::Exile);
    // Main phase, empty stack: the land can be played; the Bears needs its mana.
    assert!(can_play_land(&mut t, P0, forest));
    assert!(!castable(&mut t, P0, bears));
    // With a spell on the stack: no land.
    t.lands(P1, "Mountain", 1);
    let shock = t.hand(P1, "Shock");
    t.cast(P1, shock).target(Entity::Player(P0)).go();
    assert!(!can_play_land(&mut t, P0, forest));
    t.resolve_all();
    // In combat: no land, and no sorcery-speed creature.
    t.lands(P0, "Forest", 2);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(&mut t, P0, forest));
    assert!(!castable(&mut t, P0, bears));
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_play_land(&mut t, P0, forest));
    assert!(castable(&mut t, P0, bears));
}

#[test]
fn foretold_green_slime_can_be_cast_on_the_next_players_turn() {
    cr!("702.143a", "702.8a");
    ruling!(
        "Green Slime",
        "Casting a foretold card from exile follows the timing rules for that card. If you foretell Green Slime (or an instant card), you can cast it as soon as the next player's turn."
    );
    supported("Green Slime");
    let mut t = TestGame::new(2);
    t.lands(P0, "Forest", 1);
    let slime = t.hand(P0, "Green Slime");
    add_mana(&mut t, P0, ManaType::C, 2);
    t.g.turn.priority = Some(P0);
    t.g.take_action(P0, Action::Special(SpecialAction::Foretell { card: slime }));
    t.g.flush_events();
    let slime = t.g.current(slime);
    assert_eq!(t.zone(slime), Zone::Exile);
    assert!(!can_cast(&mut t, P0, slime, FORETELL));
    t.advance_to(P1, Step::Upkeep);
    assert!(can_cast(&mut t, P0, slime, FORETELL));
}

#[test]
fn venser_returning_a_flashback_spell_exiles_it() {
    cr!("702.34a");
    ruling!(
        "Venser, Shaper Savant",
        "If Venser's triggered ability targets a spell cast with flashback, that spell will be exiled instead of returning to its owner's hand."
    );
    supported("Venser, Shaper Savant");
    let mut t = TestGame::new(2);
    t.lands(P1, "Island", 3);
    let tt = t.graveyard(P1, "Think Twice");
    let spell = t.cast(P1, tt).method(FLASHBACK).go();
    t.answer_targets(P0, &[Entity::Object(spell)]);
    enter(&mut t, P0, "Venser, Shaper Savant");
    t.resolve();
    assert!(t.in_exile("Think Twice"));
    assert!(!t.in_hand(P1, "Think Twice"));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn slitherwisp_doesnt_trigger_for_spells_cast_as_though_they_had_flash() {
    cr!("702.8a", "609.4", "603.2");
    ruling!(
        "Slitherwisp",
        "If an effect allows you to cast a spell “as though” it has flash, Slitherwisp’s last ability won’t trigger."
    );
    supported("Slitherwisp");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Slitherwisp");
    t.battlefield(P0, "Valley Floodcaller");
    t.set_step(P1, Step::Upkeep);
    // A sorcery cast as though it had flash: no trigger.
    t.lands(P0, "Island", 3);
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    // A spell that has flash: a trigger.
    viper(&mut t);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn weeping_angel_stops_being_a_creature_but_the_attacker_stays_blocked() {
    cr!("506.4", "509.1h", "510.1c");
    ruling!(
        "Weeping Angel",
        "If an opponent casts a creature spell with flash while Weeping Angel is attacking or blocking, it is removed from combat because it stops being a creature. However, any creatures that it was blocking are still considered blocked."
    );
    supported("Weeping Angel");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let angel = t.battlefield(P1, "Weeping Angel");
    blocked_by(&mut t, bears, angel);
    viper(&mut t);
    t.resolve_all();
    assert!(!t.obj_now(angel).is(CardType::Creature));
    t.advance_to(P0, Step::EndOfCombat);
    // The Bears stayed blocked: no damage to P1, and the Angel dealt none.
    assert_eq!(t.life(P1), 20);
    assert!(t.on_battlefield(bears));
    assert_eq!(damage_on(&t, angel), 0);
}

#[test]
fn valley_floodcaller_lets_flashback_sorceries_be_cast_at_instant_speed() {
    cr!("702.8a", "702.34a", "307.1");
    ruling!(
        "Valley Floodcaller",
        "Valley Floodcaller’s second ability applies to noncreature cards in any zone, provided something is allowing you to cast them. For example, you could cast a sorcery with flashback as though it had flash."
    );
    supported("Valley Floodcaller");
    let mut t = TestGame::new(2);
    let fc = t.battlefield(P0, "Valley Floodcaller");
    t.lands(P0, "Island", 2);
    // Deep Analysis: sorcery, flashback—{1}{U}, Pay 3 life.
    let da = t.graveyard(P0, "Deep Analysis");
    t.set_step(P1, Step::Upkeep);
    assert!(can_cast(&mut t, P0, da, FLASHBACK));
    destroy(&mut t, fc);
    assert!(!can_cast(&mut t, P0, da, FLASHBACK));
}

#[test]
fn yeva_lets_green_creatures_on_top_of_the_library_be_cast_with_flash() {
    cr!("702.8a", "609.4");
    ruling!(
        "Yeva, Nature's Herald",
        "Yeva's ability applies to green creature cards in any zone, provided something is allowing you to cast them. For example, if the top card of your library is a green creature card and you control Garruk's Horde, you can cast that card as though it had flash."
    );
    supported("Yeva, Nature's Herald");
    supported("Garruk's Horde");
    let mut t = TestGame::new(2);
    let yeva = t.battlefield(P0, "Yeva, Nature's Herald");
    t.battlefield(P0, "Garruk's Horde");
    t.lands(P0, "Forest", 2);
    let bears = stack_library(&mut t, P0, &["Grizzly Bears"])[0];
    t.set_step(P1, Step::Upkeep);
    assert!(castable(&mut t, P0, bears));
    destroy(&mut t, yeva);
    assert!(!castable(&mut t, P0, bears));
}
