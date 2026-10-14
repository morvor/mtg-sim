//! Rulings batch P172 — cards that care about white (and other colors): devotion
//! (CR 700.5), multicolored spells and cost reductions (CR 601.2f, 105.2), effects locked
//! in as they resolve (CR 611.2c), damage that becomes lethal later (CR 120.6, 704.5g),
//! "activated this turn" (CR 700.10), and choices made on resolution (CR 608.2c).

use crate::r_p017_common::choice_candidates;
use crate::r_p076_common::mana;
use crate::r_p116_common::enter_together;
use crate::r_p117_common::modify;
use crate::r_s01_common::{attack_with, supported, with_subtype};
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s04_common::spell_targets;
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::{activate_containing, attach_new, damage, give_control};
use crate::r_s08_common::mana_value;
use crate::r_s15_common::library_top_n;
use crate::r_s17_common::enter_transformed;
use mtg_engine::ability::*;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn add_loyalty(t: &mut TestGame, id: ObjectId, n: u32) {
    t.g.add_counters(obj(id), "loyalty", n, None);
    t.g.recompute();
}

#[test]
fn mistveil_plains_counts_white_permanents_only_on_activation() {
    cr!("105.2c", "602.5b", "608.2b");
    ruling!(
        "Mistveil Plains",
        "A land card that produces white mana, even a Plains, normally has no color."
    );
    ruling!(
        "Mistveil Plains",
        "Once you activate Mistveil Plains's last ability, it doesn't matter if you lose control of some of your white permanents."
    );
    supported("Mistveil Plains");
    // Plains aren't white: one Savannah Lions isn't enough.
    let mut t = TestGame::new(2);
    let mp = t.battlefield(P0, "Mistveil Plains");
    let plains = t.battlefield(P0, "Plains");
    assert_eq!(t.obj(plains).chars.colors, ColorSet::NONE);
    assert_eq!(t.obj(mp).chars.colors, ColorSet::NONE);
    t.battlefield(P0, "Savannah Lions");
    let card = t.graveyard(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::W, 1);
    t.answer_targets(P0, &[obj(card)]);
    assert!(activate_containing(&mut t, P0, mp, "graveyard").is_err());
    // Two: activated; losing one in response doesn't matter.
    let mut t = TestGame::new(2);
    let mp = t.battlefield(P0, "Mistveil Plains");
    t.battlefield(P0, "Savannah Lions");
    let lions = t.battlefield(P0, "Savannah Lions");
    let card = t.graveyard(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::W, 1);
    t.answer_targets(P0, &[obj(card)]);
    activate_containing(&mut t, P0, mp, "graveyard").unwrap();
    give_control(&mut t, lions, P1);
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Library(P0));
    assert_eq!(*t.g.player(P0).library.first().unwrap(), t.g.current(card));
}

#[test]
fn cut_short_planeswalkers_activated_this_turn() {
    cr!("700.10", "115.1");
    ruling!(
        "Cut Short",
        "A planeswalker was \"activated\" during a turn if one of its abilities was activated that turn."
    );
    ruling!(
        "Cut Short",
        "Once an ability of a planeswalker is activated, it doesn't matter if the planeswalker loses that ability. That planeswalker has still been \"activated\" that turn."
    );
    supported("Cut Short");
    supported("Jace Beleren");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let jace = t.battlefield(P1, "Jace Beleren");
    let idle = t.battlefield(P1, "Liliana of the Veil");
    t.activate(P1, jace, 0, &[]).unwrap();
    // The ability is still on the stack.
    assert_eq!(t.stack_len(), 1);
    let cands = spell_targets(&mut t, P0, "Cut Short");
    assert!(cands.contains(&obj(jace)));
    assert!(!cands.contains(&obj(idle)));
    // Countered (removed from the stack), and Jace loses all abilities: still activated.
    let ability = *t.g.stack.last().unwrap();
    move_to(&mut t, ability, Zone::Exile);
    t.g.stack.retain(|s| *s != ability);
    modify(&mut t, jace, vec![Modification::RemoveAllAbilities]);
    let cands = spell_targets(&mut t, P0, "Cut Short");
    assert!(cands.contains(&obj(jace)));
}

#[test]
fn cut_short_activating_an_emblems_ability_doesnt_activate_a_planeswalker() {
    cr!("700.10", "114.4");
    ruling!(
        "Cut Short",
        "Emblems are separate from the planeswalkers you got them from. Activating an ability of an emblem doesn't cause any planeswalker to be \"activated.\""
    );
    supported("Ob Nixilis of the Black Oath");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let first = t.battlefield(P0, "Ob Nixilis of the Black Oath");
    add_loyalty(&mut t, first, 8);
    t.activate(P0, first, 2, &[]).unwrap();
    t.resolve_all();
    let emblem = t
        .g
        .objects
        .iter()
        .find(|o| o.zone == Zone::Command && o.controller == P0)
        .map(|o| o.id)
        .expect("emblem");
    // Another Ob Nixilis (not activated this turn); the emblem's ability is activated.
    let ob = t.battlefield(P0, "Ob Nixilis of the Black Oath");
    let bears = t.battlefield(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::B, 2);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, emblem, 0, &[]).unwrap();
    assert!(!t.on_battlefield(bears));
    let cands = spell_targets(&mut t, P1, "Cut Short");
    assert!(!cands.contains(&obj(ob)));
}

#[test]
fn agrus_kos_pumps_attacking_creatures_by_color() {
    cr!("105.2", "611.2c");
    ruling!(
        "Agrus Kos, Wojek Veteran",
        "Attacking creatures (including Agrus Kos) that are both red and white get +2/+2 until end of turn."
    );
    supported("Agrus Kos, Wojek Veteran");
    let mut t = TestGame::new(2);
    let agrus = t.battlefield(P0, "Agrus Kos, Wojek Veteran");
    let recruit = t.battlefield(P0, "Boros Recruit");
    let lions = t.battlefield(P0, "Savannah Lions");
    let piker = t.battlefield(P0, "Goblin Piker");
    let home = t.battlefield(P0, "Boros Recruit");
    let p1 = Entity::Player(P1);
    attack_with(
        &mut t,
        &[(agrus, p1), (recruit, p1), (lions, p1), (piker, p1)],
    );
    t.resolve_all();
    assert_eq!(t.pt(agrus), (5, 5));
    assert_eq!(t.pt(recruit), (3, 3));
    assert_eq!(t.pt(lions), (2, 3));
    assert_eq!(t.pt(piker), (4, 1));
    assert_eq!(t.pt(home), (1, 1));
}

#[test]
fn nonlethal_damage_becomes_lethal_when_toughness_drops() {
    cr!("120.6", "704.5g");
    ruling!(
        "Daxos, Blessed by the Sun",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to Daxos may become lethal if your devotion to white decreases during that turn."
    );
    ruling!(
        "Celestial Crusader",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a white creature may become lethal if Celestial Crusader leaves the battlefield during that turn."
    );
    ruling!(
        "Sulfur Elemental",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to white creatures may become lethal if Sulfur Elemental enters the battlefield during that turn."
    );
    ruling!(
        "Divine Sacrament",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to white creatures you control may become lethal if the number of cards in your graveyard falls below seven or if Divine Sacrament leaves the battlefield."
    );
    // Daxos: devotion falls.
    let mut t = TestGame::new(2);
    let src = t.battlefield(P1, "Hill Giant");
    let daxos = t.battlefield(P0, "Daxos, Blessed by the Sun");
    let angel = t.battlefield(P0, "Serra Angel");
    assert_eq!(t.pt(daxos), (2, 4));
    damage(&mut t, src, 3, daxos);
    assert!(t.on_battlefield(daxos));
    move_to(&mut t, angel, Zone::Hand(P0));
    assert!(!t.on_battlefield(daxos));
    // Celestial Crusader leaves.
    let mut t = TestGame::new(2);
    let src = t.battlefield(P1, "Hill Giant");
    let cc = t.battlefield(P0, "Celestial Crusader");
    let lions = t.battlefield(P0, "Savannah Lions");
    damage(&mut t, src, 1, lions);
    assert!(t.on_battlefield(lions));
    move_to(&mut t, cc, Zone::Graveyard(P0));
    assert!(!t.on_battlefield(lions));
    // Sulfur Elemental enters.
    let mut t = TestGame::new(2);
    let src = t.battlefield(P1, "Hill Giant");
    let angel = t.battlefield(P0, "Serra Angel");
    damage(&mut t, src, 3, angel);
    assert!(t.on_battlefield(angel));
    enter(&mut t, P1, "Sulfur Elemental");
    assert!(!t.on_battlefield(angel));
    // Divine Sacrament: the graveyard falls below seven, or it leaves.
    for leaves in [false, true] {
        let mut t = TestGame::new(2);
        let src = t.battlefield(P1, "Hill Giant");
        let ds = t.battlefield(P0, "Divine Sacrament");
        let lions = t.battlefield(P0, "Savannah Lions");
        let gy: Vec<ObjectId> = (0..7).map(|_| t.graveyard(P0, "Grizzly Bears")).collect();
        t.g.recompute();
        assert_eq!(t.pt(lions), (4, 3));
        damage(&mut t, src, 2, lions);
        assert!(t.on_battlefield(lions));
        if leaves {
            move_to(&mut t, ds, Zone::Graveyard(P1));
        } else {
            move_to(&mut t, gy[0], Zone::Exile);
        }
        assert!(!t.on_battlefield(lions), "leaves: {leaves}");
    }
}

#[test]
fn sulfur_elementals_toughness_change_is_checked_before_anyone_can_act() {
    cr!("704.3", "117.5");
    ruling!(
        "Sulfur Elemental",
        "Players can't take actions between the time that Sulfur Elemental enters the battlefield and the time state-based actions are performed."
    );
    supported("Sulfur Elemental");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Savannah Lions");
    let se = in_hand_with_mana(&mut t, P0, "Sulfur Elemental");
    t.cast(P0, se).go();
    // Resolving it: the state-based action is performed before anyone gets priority.
    t.resolve();
    assert!(t.in_graveyard(P1, "Savannah Lions"));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn elspeths_emblem_effect_is_locked_in_as_it_resolves() {
    cr!("611.2c", "608.2h", "700.5");
    ruling!(
        "Elspeth, Undaunted Hero",
        "Elspeth’s last ability affects only creatures you control at the time it resolves. Creatures you begin to control later in the turn won’t get +X/+X or gain flying."
    );
    ruling!(
        "Elspeth, Undaunted Hero",
        "The value of X is determined only as Elspeth’s last ability resolves. Once that happens, the value of X won’t change later in the turn even if your devotion to white changes."
    );
    supported("Elspeth, Undaunted Hero");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let elspeth = t.battlefield(P0, "Elspeth, Undaunted Hero");
    add_loyalty(&mut t, elspeth, 5);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.activate(P0, elspeth, 2, &[]).unwrap();
    t.resolve_all();
    // Devotion 3 (Elspeth's {W}{W}{W}).
    assert_eq!(t.pt(bears), (5, 5));
    assert!(t.obj(bears).has_keyword(KeywordKind::Flying));
    let angel = t.battlefield(P0, "Serra Angel");
    let later = t.battlefield(P0, "Hill Giant");
    t.g.recompute();
    assert_eq!(t.pt(bears), (5, 5));
    assert_eq!(t.pt(angel), (4, 4));
    assert_eq!(t.pt(later), (3, 3));
    assert!(!t.obj(later).has_keyword(KeywordKind::Flying));
}

#[test]
fn guardians_pledge_is_locked_in_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Guardians' Pledge",
        "Guardians' Pledge only gives a bonus to white creatures you control when it resolves. Creatures that enter later in the turn and nonwhite creatures that become white later in the turn will not get the bonus."
    );
    ruling!(
        "Guardians' Pledge",
        "If a white creature that gets the bonus stops being white later in the turn, it will continue to get +2/+2 until end of turn."
    );
    supported("Guardians' Pledge");
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P0, "Savannah Lions");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let gp = t.hand(P0, "Guardians' Pledge");
    mana(&mut t, P0, ManaType::W, 3);
    t.cast(P0, gp).go();
    t.resolve_all();
    assert_eq!(t.pt(lions), (4, 3));
    assert_eq!(t.pt(bears), (2, 2));
    let later = t.battlefield(P0, "Savannah Lions");
    modify(&mut t, lions, vec![Modification::SetColors(ColorSet::single(Color::Blue))]);
    modify(&mut t, bears, vec![Modification::SetColors(ColorSet::single(Color::White))]);
    assert_eq!(t.pt(lions), (4, 3));
    assert_eq!(t.pt(bears), (2, 2));
    assert_eq!(t.pt(later), (2, 1));
}

#[test]
fn battle_brawler_losing_first_strike_doesnt_deal_damage_twice() {
    cr!("702.7c", "510.4");
    ruling!(
        "Battle Brawler",
        "If Battle Brawler assigns combat damage during the first combat damage step, and then it loses first strike before the second combat damage step"
    );
    supported("Battle Brawler");
    let mut t = TestGame::new(2);
    let bb = t.battlefield(P0, "Battle Brawler");
    let piker = t.battlefield(P0, "Goblin Piker");
    attack_with(&mut t, &[(bb, Entity::Player(P1))]);
    t.advance_to(P0, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 17);
    move_to(&mut t, piker, Zone::Graveyard(P0));
    assert!(!t.obj(bb).has_keyword(KeywordKind::FirstStrike));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 17);
}

#[test]
fn conclave_tribunal_leaving_first_exiles_nothing() {
    cr!("610.3c", "603.2");
    ruling!(
        "Conclave Tribunal",
        "If Conclave Tribunal leaves the battlefield before its triggered ability resolves, the target permanent won't be exiled."
    );
    supported("Conclave Tribunal");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let ct = enter(&mut t, P0, "Conclave Tribunal");
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, ct, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn view_from_above_returns_to_hand_only_if_it_resolves() {
    cr!("608.2b", "608.2n");
    ruling!(
        "View from Above",
        "If View from Above doesn’t resolve for any reason (for example, if its target has become illegal), none of its effects happen. It will be put into your graveyard as normal."
    );
    ruling!(
        "View from Above",
        "If you control a white permanent, View from Above moves from the stack to your hand as part of its resolution. It never hits the graveyard. If you don’t control a white permanent, View from Above is put into the graveyard from the stack as normal."
    );
    supported("View from Above");
    for (white, bounce, hand) in [(true, false, true), (false, false, false), (true, true, false)] {
        let mut t = TestGame::new(2);
        if white {
            t.battlefield(P0, "Savannah Lions");
        }
        let bears = t.battlefield(P0, "Grizzly Bears");
        let vfa = t.hand(P0, "View from Above");
        mana(&mut t, P0, ManaType::U, 2);
        t.cast(P0, vfa).target(bears).go();
        if bounce {
            move_to(&mut t, bears, Zone::Hand(P0));
        }
        t.resolve_all();
        assert_eq!(t.in_hand(P0, "View from Above"), hand);
        assert_eq!(t.in_graveyard(P0, "View from Above"), !hand);
        let to_graveyard = t.turn_events.iter().chain(t.g.events.iter()).any(|e| {
            matches!(e, mtg_engine::events::Event::ZoneChange { to: Zone::Graveyard(_), .. })
        });
        assert_eq!(to_graveyard, !hand);
        if !bounce {
            assert!(t.obj(bears).has_keyword(KeywordKind::Flying));
        }
    }
}

#[test]
fn angels_feather_triggers_for_any_players_white_spell_and_resolves_first() {
    cr!("603.3", "601.2i");
    ruling!(
        "Angel's Feather",
        "If a player casts a white spell, Angel’s Feather’s ability triggers and is put on the stack on top of that spell."
    );
    ruling!(
        "Angel's Feather",
        "The ability triggers whenever any player, not just you, casts a white spell."
    );
    supported("Angel's Feather");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    t.battlefield(P0, "Angel's Feather");
    let lions = t.hand(P1, "Savannah Lions");
    mana(&mut t, P1, ManaType::W, 1);
    t.answer_yes(P0, true);
    let spell = t.cast(P1, lions).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve();
    assert_eq!(t.life(P0), 21);
    assert!(t.g.stack.contains(&spell));
    // A nonwhite spell doesn't trigger it.
    let bears = t.hand(P1, "Grizzly Bears");
    mana(&mut t, P1, ManaType::G, 2);
    t.resolve_all();
    t.cast(P1, bears).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
}

/// Whether `p` can cast the real card `spell` with a pool of `pool` while controlling
/// `permanents`.
fn castable(permanents: &[(PlayerId, &str)], spell: &str, pool: &[(ManaType, u32)]) -> bool {
    let mut t = TestGame::new(2);
    for (p, n) in permanents {
        t.battlefield(*p, n);
    }
    let card = t.hand(P0, spell);
    for (ty, n) in pool {
        mana(&mut t, P0, *ty, *n);
    }
    t.cast(P0, card).try_go().is_ok()
}

#[test]
fn familiars_reduce_a_multicolored_spell_only_once() {
    cr!("601.2f", "105.2");
    ruling!(
        "Thornscape Familiar",
        "If a spell is both red and white, you pay {1} less, not {2} less."
    );
    ruling!(
        "Stormscape Familiar",
        "If a spell is both white and black, you pay {1} less, not {2} less."
    );
    supported("Thornscape Familiar");
    supported("Stormscape Familiar");
    use ManaType::*;
    let th = [(P0, "Thornscape Familiar")];
    let helix_like = "Agrus Kos, Wojek Veteran"; // {3}{R}{W}
    assert!(castable(&th, helix_like, &[(R, 1), (W, 1), (C, 2)]));
    assert!(!castable(&th, helix_like, &[(R, 1), (W, 1), (C, 1)]));
    let st = [(P0, "Stormscape Familiar")];
    assert!(castable(&st, "Utter End", &[(W, 1), (B, 1), (C, 1)]));
    assert!(!castable(&st, "Utter End", &[(W, 1), (B, 1)]));
}

#[test]
fn grand_arbiter_reduces_generic_mana_of_white_and_blue_spells_by_two() {
    cr!("601.2f", "118.7c");
    ruling!(
        "Grand Arbiter Augustin IV",
        "Spells you cast that are both blue and white cost {2} less to cast."
    );
    ruling!(
        "Grand Arbiter Augustin IV",
        "This reduces only generic mana portion of mana costs. A spell you cast with mana cost {1}{W}{U} will cost {W}{U} to cast."
    );
    supported("Grand Arbiter Augustin IV");
    use ManaType::*;
    let ga = [(P0, "Grand Arbiter Augustin IV")];
    // {2}{W}{U}
    assert!(castable(&ga, "Sawtooth Loon", &[(W, 1), (U, 1)]));
    // {1}{W}{U}: costs {W}{U}, not less.
    assert!(castable(&ga, "Silver Drake", &[(W, 1), (U, 1)]));
    assert!(!castable(&ga, "Silver Drake", &[(W, 1), (C, 2)]));
    assert!(!castable(&ga, "Savannah Lions", &[]));
}

#[test]
fn daxos_gains_life_for_creatures_entering_or_dying_with_it() {
    cr!("603.6a", "603.10a");
    ruling!(
        "Daxos, Blessed by the Sun",
        "If another creature you control enters the battlefield at the same time as Daxos, you gain 1 life for that creature. Similarly, if another creature you control dies at the same time as Daxos, you gain 1 life for that creature."
    );
    supported("Day of Judgment");
    let mut t = TestGame::new(2);
    enter_together(&mut t, P0, &["Daxos, Blessed by the Sun", "Grizzly Bears"]);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
    let doj = t.hand(P0, "Day of Judgment");
    mana(&mut t, P0, ManaType::W, 4);
    t.cast(P0, doj).go();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn multicolored_spells_trigger_color_cast_triggers_once() {
    cr!("603.2c", "105.2");
    ruling!(
        "Quirion Dryad",
        "If you cast a spell that's more than one color, Quirion Dryad's ability triggers just once."
    );
    ruling!(
        "Questing Druid // Seek the Beast",
        "If you cast a spell that's more than one of the listed colors, Questing Druid's ability still triggers only once."
    );
    for name in ["Quirion Dryad", "Questing Druid // Seek the Beast"] {
        supported(name);
        let mut t = TestGame::new(2);
        let d = t.battlefield(P0, name);
        let helix = t.hand(P0, "Lightning Helix");
        mana(&mut t, P0, ManaType::R, 1);
        mana(&mut t, P0, ManaType::W, 1);
        t.cast(P0, helix).target(Entity::Player(P1)).go();
        t.resolve_all();
        assert_eq!(t.counters(d, "+1/+1"), 1, "{name}");
    }
}

#[test]
fn devotion_counts_battles_and_auras_you_control() {
    cr!("700.5", "310.8");
    ruling!(
        "Daxos, Blessed by the Sun",
        "If you control a battle and an opponent is its protector, you still control the battle, and mana symbols in its mana cost count toward your devotion."
    );
    let mut t = TestGame::new(2);
    let daxos = t.battlefield(P0, "Daxos, Blessed by the Sun");
    t.answer_choose(P0, &[Entity::Player(P1)]);
    let battle = t.enter(P0, "Invasion of Gobakhan // Lightshield Array");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(
        mtg_engine::battle::protector(&t.g, battle),
        Some(P1)
    );
    assert_eq!(t.obj_now(battle).controller, P0);
    assert_eq!(t.pt(daxos), (2, 3));
}

/// P0 has March of the Canonized, Serra Angel and Daxos (devotion to white and black 6),
/// plus `extra`; returns whether its upkeep trigger made a Vampire Demon.
fn march_upkeep(extra: fn(&mut TestGame), before_resolving: fn(&mut TestGame)) -> bool {
    let mut t = TestGame::new(2);
    t.battlefield(P0, "March of the Canonized");
    t.battlefield(P0, "Serra Angel");
    t.battlefield(P0, "Daxos, Blessed by the Sun");
    extra(&mut t);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    before_resolving(&mut t);
    t.resolve_all();
    !with_subtype(&t, P0, "Demon").is_empty()
}

#[test]
fn march_of_the_canonized_devotion() {
    cr!("700.5", "603.4", "712.8e");
    ruling!(
        "March of the Canonized",
        "If you put an Aura on an opponent's permanent, you still control the Aura, and mana symbols in its mana cost count toward your devotion."
    );
    ruling!(
        "March of the Canonized",
        "March of the Canonized's last ability will trigger only if your devotion to white and black is seven or greater at the beginning of your upkeep. As the triggered ability tries to resolve, if your devotion to white and black is no longer at least seven, the ability won't do anything."
    );
    ruling!(
        "March of the Canonized",
        "The back face of a transforming double-faced permanent has no mana cost. Mana symbols on a transforming double-faced permanent's front face don't count toward your devotion to any color while it's on the battlefield with its back face up."
    );
    supported("March of the Canonized");
    fn nothing(_: &mut TestGame) {}
    assert!(!march_upkeep(nothing, nothing));
    fn pacifism(t: &mut TestGame) {
        let bears = t.battlefield(P1, "Grizzly Bears");
        attach_new(t, P0, "Pacifism", bears);
    }
    assert!(march_upkeep(pacifism, nothing));
    fn lose_angel(t: &mut TestGame) {
        assert_eq!(t.stack_len(), 1);
        let angel = t.named_on_battlefield("Serra Angel")[0];
        move_to(t, angel, Zone::Hand(P0));
    }
    assert!(!march_upkeep(pacifism, lose_angel));
    fn cathar(t: &mut TestGame) {
        t.battlefield(P0, "Hanweir Militia Captain // Westvale Cult Leader");
    }
    assert!(march_upkeep(cathar, nothing));
    fn brute(t: &mut TestGame) {
        enter_transformed(t, P0, "Hanweir Militia Captain // Westvale Cult Leader");
    }
    assert!(!march_upkeep(brute, nothing));
}

#[test]
fn knight_of_malice_gets_plus_one_once_for_any_white_permanent() {
    cr!("613.4c");
    ruling!(
        "Knight of Malice",
        "Knight of Malice only gets +1/+0 if any player controls a white permanent, no matter how many white permanents players control."
    );
    supported("Knight of Malice");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Knight of Malice");
    assert_eq!(t.pt(k), (2, 2));
    t.battlefield(P1, "Savannah Lions");
    t.battlefield(P1, "Savannah Lions");
    t.battlefield(P0, "Savannah Lions");
    t.g.recompute();
    assert_eq!(t.pt(k), (3, 2));
}

#[test]
fn daybreak_chimera_cost_reduction() {
    cr!("601.2f", "202.3", "601.2h");
    ruling!(
        "Daybreak Chimera",
        "The cost reduction ability reduces only the generic mana in Daybreak Chimera's cost. The colored mana must still be paid."
    );
    ruling!(
        "Daybreak Chimera",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Daybreak Chimera). The mana value of the spell remains unchanged, no matter what the total cost to cast it was."
    );
    ruling!(
        "Daybreak Chimera",
        "Once you announce that you're casting a spell, no player may take actions until the spell has been paid for."
    );
    supported("Daybreak Chimera");
    use ManaType::*;
    let angels = [(P0, "Serra Angel"), (P0, "Serra Angel")];
    assert!(castable(&angels, "Daybreak Chimera", &[(W, 2)]));
    assert!(!castable(&angels, "Daybreak Chimera", &[(W, 1), (C, 4)]));
    // P1's Grand Arbiter's {1} more is added first, then devotion 4 takes off all four.
    let taxed = [
        (P0, "Serra Angel"),
        (P0, "Serra Angel"),
        (P1, "Grand Arbiter Augustin IV"),
    ];
    assert!(castable(&taxed, "Daybreak Chimera", &[(W, 2)]));
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Serra Angel");
    t.battlefield(P0, "Serra Angel");
    t.battlefield(P1, "Grand Arbiter Augustin IV");
    let dc = t.hand(P0, "Daybreak Chimera");
    mana(&mut t, P0, W, 2);
    let from = t.asked().len();
    let spell = t.cast(P0, dc).go();
    // No player got priority while it was being cast.
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::Priority { .. })));
    assert_eq!(mana_value(&t, spell), 5);
}

#[test]
fn daxos_toughness_works_in_every_zone() {
    cr!("604.3", "208.2a");
    ruling!(
        "Daxos, Blessed by the Sun",
        "The ability that defines Daxos's toughness works in all zones, not just the battlefield."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Serra Angel");
    let in_hand = t.hand(P0, "Daxos, Blessed by the Sun");
    let in_gy = t.graveyard(P0, "Daxos, Blessed by the Sun");
    t.battlefield(P0, "Savannah Lions");
    t.g.recompute();
    assert_eq!(t.obj(in_hand).chars.toughness, Some(3));
    assert_eq!(t.obj(in_gy).chars.toughness, Some(3));
}

#[test]
fn heralds_need_three_different_creatures() {
    cr!("118.3", "602.2b");
    ruling!(
        "Angel's Herald",
        "To activate the ability, you must sacrifice three different creatures. You can’t sacrifice just one three-colored creature, for example."
    );
    supported("Angel's Herald");
    supported("Rhox War Monk");
    let mut t = TestGame::new(2);
    let herald = t.battlefield(P0, "Angel's Herald");
    t.battlefield(P0, "Rhox War Monk");
    mana(&mut t, P0, ManaType::W, 3);
    assert!(t.activate(P0, herald, 0, &[]).is_err());
    let mut t = TestGame::new(2);
    let herald = t.battlefield(P0, "Angel's Herald");
    let monk = t.battlefield(P0, "Rhox War Monk");
    let elves = t.battlefield(P0, "Llanowar Elves");
    mana(&mut t, P0, ManaType::W, 3);
    // Green, white and blue.
    t.answer_choose(P0, &[obj(elves)]);
    t.answer_choose(P0, &[obj(herald)]);
    t.answer_choose(P0, &[obj(monk)]);
    let r = t.activate(P0, herald, 0, &[]);
    assert!(r.is_ok(), "{r:?}");
    assert_eq!(t.g.player(P0).graveyard.len(), 3);
}

#[test]
fn blinding_light_doesnt_tap_white_multicolored_creatures() {
    cr!("105.2", "105.4");
    ruling!(
        "Blinding Light",
        "Will not tap a creature that is both white and another color."
    );
    supported("Blinding Light");
    let mut t = TestGame::new(2);
    let recruit = t.battlefield(P1, "Boros Recruit");
    let lions = t.battlefield(P1, "Savannah Lions");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let bl = t.hand(P0, "Blinding Light");
    mana(&mut t, P0, ManaType::W, 3);
    t.cast(P0, bl).go();
    t.resolve_all();
    assert!(!t.obj(recruit).tapped);
    assert!(!t.obj(lions).tapped);
    assert!(t.obj(bears).tapped);
}

#[test]
fn jasmine_seer_can_reveal_nothing() {
    cr!("701.20a");
    ruling!(
        "Jasmine Seer",
        "You can reveal zero cards and gain zero life."
    );
    supported("Jasmine Seer");
    let mut t = TestGame::new(2);
    let js = t.battlefield(P0, "Jasmine Seer");
    t.hand(P0, "Savannah Lions");
    mana(&mut t, P0, ManaType::W, 3);
    t.answer_choose(P0, &[]);
    t.activate(P0, js, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 20);
    // Revealing one: 2 life.
    let mut t = TestGame::new(2);
    let js = t.battlefield(P0, "Jasmine Seer");
    let lions = t.hand(P0, "Savannah Lions");
    mana(&mut t, P0, ManaType::W, 3);
    t.answer_choose(P0, &[obj(lions)]);
    t.activate(P0, js, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
}

#[test]
fn ajanis_influence_needs_a_target_and_does_nothing_if_it_becomes_illegal() {
    cr!("601.2c", "608.2b");
    ruling!(
        "Ajani's Influence",
        "You can’t cast Ajani’s Influence without a target creature. If the target creature is an illegal target when Ajani’s Influence tries to resolve, the spell doesn’t resolve. You won’t look at the top five cards of your library."
    );
    supported("Ajani's Influence");
    let mut t = TestGame::new(2);
    let ai = t.hand(P0, "Ajani's Influence");
    mana(&mut t, P0, ManaType::W, 4);
    assert!(t.cast(P0, ai).try_go().is_err());
    let bears = t.battlefield(P0, "Grizzly Bears");
    let top = library_top_n(&t, P0, 5);
    t.cast(P0, ai).target(bears).go();
    move_to(&mut t, bears, Zone::Hand(P0));
    let from = t.asked().len();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Ajani's Influence"));
    assert_eq!(library_top_n(&t, P0, 5), top);
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseEntities { .. })));
}

#[test]
fn bounce_creatures_choose_what_to_return_on_resolution() {
    cr!("608.2c", "115.1");
    ruling!(
        "Cavern Harpy",
        "You choose the creature to return on resolution of the triggered ability. This is not targeted."
    );
    ruling!(
        "Sawtooth Loon",
        "You choose which white or blue creature you control to return to its owner’s hand as the first triggered ability resolves. That creature isn’t targeted by the ability. You may choose Sawtooth Loon itself. If Sawtooth Loon is the only white or blue creature you control at that time, you must return it to its owner’s hand."
    );
    // Alone, each returns itself.
    for name in [
        "Cavern Harpy",
        "Doomsday Specter",
        "Fleetfoot Panther",
        "Horned Kavu",
        "Lava Zombie",
        "Marsh Crocodile",
        "Sawtooth Loon",
        "Silver Drake",
        "Shivan Wurm",
        "Steel Leaf Paladin",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let c = enter(&mut t, P0, name);
        t.resolve_all();
        assert!(!t.on_battlefield(c), "{name}");
        assert!(t.in_hand(P0, name), "{name}");
    }
    // With a hexproof... a shroud creature: it can still be chosen, as the trigger
    // resolves.
    let mut t = TestGame::new(2);
    let crow = t.battlefield(P0, "Storm Crow");
    modify(&mut t, crow, vec![Modification::AddKeyword(KeywordKind::Shroud.into())]);
    let from = t.asked().len();
    t.answer_choose(P0, &[obj(crow)]);
    let harpy = enter(&mut t, P0, "Cavern Harpy");
    assert!(t.asked()[from..]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseTargets { .. })));
    t.resolve_all();
    assert!(t.on_battlefield(harpy));
    assert!(t.in_hand(P0, "Storm Crow"));
    // Sawtooth Loon may return itself even with another choice: both are candidates.
    let mut t = TestGame::new(2);
    let lions = t.battlefield(P0, "Savannah Lions");
    let from = t.asked().len();
    let loon = enter(&mut t, P0, "Sawtooth Loon");
    t.resolve_all();
    let offered = choice_candidates(&t, from)
        .into_iter()
        .any(|c| c.contains(&obj(loon)) && c.contains(&obj(lions)) && c.len() == 2);
    assert!(offered);
}

#[test]
fn teysa_sacrificing_itself_doesnt_trigger_its_own_ability() {
    cr!("603.10a", "118.3");
    ruling!(
        "Teysa, Orzhov Scion",
        "You may sacrifice Teysa itself to help pay for its first ability, but Teysa dying won't cause its second ability to trigger. Any other white and black creatures you sacrifice to pay for the first ability will cause the second ability to trigger."
    );
    supported("Teysa, Orzhov Scion");
    supported("Tithe Drinker");
    let mut t = TestGame::new(2);
    let teysa = t.battlefield(P0, "Teysa, Orzhov Scion");
    let drinker = t.battlefield(P0, "Tithe Drinker");
    let lions = t.battlefield(P0, "Savannah Lions");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(teysa), obj(drinker), obj(lions)]);
    t.activate(P0, teysa, 0, &[obj(bears)]).unwrap();
    t.resolve_all();
    assert!(!t.on_battlefield(teysa));
    assert!(!t.on_battlefield(bears));
    assert_eq!(with_subtype(&t, P0, "Spirit").len(), 1);
}

