//! Rulings batch P063 — creatures that gain first strike, double strike, deathtouch or
//! flash: what happens between the first-strike and regular combat damage steps (CR
//! 510.4, 702.7c, 702.4d), conditions checked as a triggered ability resolves (CR
//! 603.4), "another [type] entered this turn", and flash from a static ability (CR
//! 601.3, 601.2).

use crate::r_p063_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s05_common::{enter, move_to};
use crate::r_s06_common::give_control;
use crate::r_s21_common::castable;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

#[test]
fn alpharael_discards_one_artifact_or_two_cards_of_any_kind() {
    cr!("118.12", "608.2c");
    ruling!(
        "Alpharael, Dreaming Acolyte",
        "As Alpharael, Dreaming Acolyte’s first ability resolves, you can discard either one artifact card or two cards which may or may not be artifacts. If you really want to, you can discard two artifact cards."
    );
    supported("Alpharael, Dreaming Acolyte");
    // One artifact card.
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Ornithopter");
    t.hand(P0, "Ornithopter");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a)]);
    enter(&mut t, P0, "Alpharael, Dreaming Acolyte");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.graveyard_size(P0), 1);
    // Two artifact cards (declining to discard just one).
    let mut t = TestGame::new(2);
    let a = t.hand(P0, "Ornithopter");
    let b = t.hand(P0, "Ornithopter");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, false);
    t.answer_choose(P0, &[obj(a), obj(b)]);
    enter(&mut t, P0, "Alpharael, Dreaming Acolyte");
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.graveyard_size(P0), 2);
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert!(!t.in_hand(P0, "Ornithopter"));
}

#[test]
fn gloom_stalker_has_double_strike_even_if_it_arrived_after_the_dungeon() {
    cr!("309.7", "702.4a");
    ruling!(
        "Gloom Stalker",
        "Gloom Stalker will have double strike even if it wasn't on the battlefield as you completed a dungeon."
    );
    supported("Gloom Stalker");
    let mut t = TestGame::new(2);
    let before = t.battlefield(P0, "Gloom Stalker");
    assert!(!has(&mut t, before, KeywordKind::DoubleStrike));
    t.g.players[0].dungeons_completed = 1;
    t.g.players[0].completed_dungeons.push("Lost Mine of Phandelver".into());
    t.g.recompute();
    let after = t.battlefield(P0, "Gloom Stalker");
    assert!(has(&mut t, after, KeywordKind::DoubleStrike));
    assert!(has(&mut t, before, KeywordKind::DoubleStrike));
}

/// The creature dealt first-strike damage to P1 (unblocked), then loses its first or
/// double strike via `lose`: it deals no regular combat damage.
fn loses_strike_after_first_damage(
    t: &mut TestGame,
    c: ObjectId,
    kw: KeywordKind,
    lose: impl FnOnce(&mut TestGame),
) {
    assert!(has(t, c, kw));
    let power = t.pt(c).0;
    to_first_strike_damage(t, &[c], &[]);
    assert_eq!(t.g.turn.step, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 20 - power, "first-strike damage");
    lose(t);
    assert!(!has(t, c, kw));
    finish_combat(t);
    assert_eq!(t.life(P1), 20 - power, "no regular combat damage");
}

#[test]
fn spiraling_duelist_losing_double_strike_skips_its_second_hit() {
    cr!("510.4", "702.4c", "702.4d");
    ruling!(
        "Spiraling Duelist",
        "If Spiraling Duelist has double strike as the combat damage step begins, there will be a second combat damage step. As that second combat damage step begins, if Spiraling Duelist no longer has double strike (perhaps because an artifact creature you controlled was destroyed), Spiraling Duelist will not assign combat damage a second time."
    );
    supported("Spiraling Duelist");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Spiraling Duelist");
    let arts = [
        t.battlefield(P0, "Ornithopter"),
        t.battlefield(P0, "Ornithopter"),
        t.battlefield(P0, "Ornithopter"),
    ];
    loses_strike_after_first_damage(&mut t, d, KeywordKind::DoubleStrike, |t| {
        destroy(t, arts[0])
    });
}

#[test]
fn kithkin_greatheart_losing_first_strike_deals_no_regular_damage() {
    cr!("510.4", "702.7c");
    ruling!(
        "Kithkin Greatheart",
        "If, after dealing first-strike combat damage, Kithkin Greatheart loses first strike (perhaps because the Giant you controlled was destroyed by first-strike combat damage), it won’t deal regular combat damage."
    );
    supported("Kithkin Greatheart");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kithkin Greatheart");
    let giant = t.battlefield(P0, "Hill Giant");
    loses_strike_after_first_damage(&mut t, k, KeywordKind::FirstStrike, |t| destroy(t, giant));
}

#[test]
fn apostle_of_invasion_loses_double_strike_when_the_poisoned_opponent_leaves() {
    cr!("510.4", "702.4d", "800.4a");
    ruling!(
        "Apostle of Invasion",
        "In a multiplayer game, Apostle of Invasion can lose double strike after dealing damage in the first combat damage step if all the opponents that had three or more poison counters left the game."
    );
    supported("Apostle of Invasion");
    let mut t = TestGame::new(3);
    let apostle = t.battlefield(P0, "Apostle of Invasion");
    t.g.add_counters(Entity::Player(P2), "poison", 3, None);
    t.g.recompute();
    loses_strike_after_first_damage(&mut t, apostle, KeywordKind::DoubleStrike, |t| {
        crate::r_p079_common::concede(t, P2)
    });
}

#[test]
fn boros_reckoner_triggers_on_lethal_damage_and_its_damage_isnt_combat_damage() {
    cr!("603.10a", "120.2a", "510.2");
    ruling!(
        "Boros Reckoner",
        "Boros Reckoner's first ability will trigger even if it is dealt lethal damage. For example, if it blocks a 7/7 creature, its ability will trigger and Boros Reckoner will deal 7 damage to the target."
    );
    ruling!(
        "Boros Reckoner",
        "Damage dealt by Boros Reckoner due to its first ability isn't combat damage, even if it was combat damage that caused that ability to trigger."
    );
    supported("Boros Reckoner");
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let reckoner = t.battlefield(P1, "Boros Reckoner");
    t.answer_targets(P1, &[Entity::Player(P0)]);
    to_first_strike_damage(&mut t, &[wurm], &[(reckoner, wurm)]);
    assert!(t.in_graveyard(P1, "Boros Reckoner"));
    assert_eq!(t.life(P0), 14, "it dealt the 6 damage it was dealt");
    let to_p0: Vec<_> = crate::r_s30_common::damage_events(&t)
        .into_iter()
        .filter(|(_, to, _, _)| *to == Entity::Player(P0))
        .collect();
    assert_eq!(to_p0.len(), 1);
    assert!(!to_p0[0].3, "not combat damage");
}

#[test]
fn flamespeaker_adept_triggers_once_per_scry() {
    cr!("701.22a", "603.2");
    ruling!(
        "Flamespeaker Adept",
        "Flamespeaker Adept's ability triggers once for each scry instruction, no matter how many cards you're scrying."
    );
    supported("Flamespeaker Adept");
    let mut t = TestGame::new(2);
    let adept = t.battlefield(P0, "Flamespeaker Adept");
    mtg_engine::library::scry(&mut t.g, P0, 3);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.pt(adept), (4, 3));
    assert!(has(&mut t, adept, KeywordKind::FirstStrike));
}

#[test]
fn soltari_lancer_has_first_strike_as_soon_as_it_attacks() {
    cr!("611.3a", "508.1", "702.7a");
    ruling!(
        "Soltari Lancer",
        "There is never a time when the Lancer is attacking but doesn't have first strike."
    );
    supported("Soltari Lancer");
    let mut t = TestGame::new(2);
    let lancer = t.battlefield(P0, "Soltari Lancer");
    assert!(!has(&mut t, lancer, KeywordKind::FirstStrike));
    // Right after attackers are declared, before anything resolves.
    crate::r_s01_common::attack_with(&mut t, &[(lancer, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 0);
    assert!(has(&mut t, lancer, KeywordKind::FirstStrike));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn gaining_first_strike_after_first_strike_damage_means_regular_damage_once() {
    cr!("510.4", "702.7c");
    ruling!(
        "Bloodhaze Wolverine",
        "If Bloodhaze Wolverine gains first strike after first-strike combat damage has been dealt, it will just deal regular combat damage."
    );
    supported("Bloodhaze Wolverine");
    // A first striker (White Knight) makes a first-strike damage step; the Wolverine gains
    // first strike in it (P0 draws a second card).
    let mut t = TestGame::new(2);
    let wolv = t.battlefield(P0, "Bloodhaze Wolverine");
    let knight = t.battlefield(P0, "White Knight");
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    to_first_strike_damage(&mut t, &[wolv, knight], &[]);
    assert_eq!(t.g.turn.step, Step::FirstStrikeDamage);
    assert_eq!(t.life(P1), 18, "only the Knight so far");
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.resolve_all();
    assert!(has(&mut t, wolv, KeywordKind::FirstStrike));
    assert_eq!(t.pt(wolv), (3, 2));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 15, "the Wolverine dealt regular damage once");
}

#[test]
fn paranoid_parish_blade_deals_combat_damage_exactly_once() {
    cr!("510.4", "702.7c");
    ruling!(
        "Paranoid Parish-Blade",
        "Losing or gaining first strike after first-strike damage has been dealt won't cause Paranoid Parish-Blade to deal combat damage twice or to not deal combat damage."
    );
    supported("Paranoid Parish-Blade");
    let delirium = ["Forest", "Grizzly Bears", "Shock", "Lava Axe"];
    // Losing it: delirium as first-strike damage is dealt, then the graveyard is exiled.
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Paranoid Parish-Blade");
    let gy: Vec<ObjectId> = delirium.iter().map(|n| t.graveyard(P0, n)).collect();
    t.g.recompute();
    assert!(has(&mut t, blade, KeywordKind::FirstStrike));
    to_first_strike_damage(&mut t, &[blade], &[]);
    assert_eq!(t.life(P1), 16);
    for c in gy {
        move_to(&mut t, c, Zone::Exile);
    }
    assert!(!has(&mut t, blade, KeywordKind::FirstStrike));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 16, "not twice");
    // Gaining it: no delirium until after first-strike damage (a White Knight makes the
    // first-strike step).
    let mut t = TestGame::new(2);
    let blade = t.battlefield(P0, "Paranoid Parish-Blade");
    let knight = t.battlefield(P0, "White Knight");
    to_first_strike_damage(&mut t, &[blade, knight], &[]);
    assert_eq!(t.life(P1), 18);
    for n in delirium {
        t.graveyard(P0, n);
    }
    t.g.recompute();
    assert!(has(&mut t, blade, KeywordKind::FirstStrike));
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 14, "it still deals its (now 4) damage");
}

#[test]
fn swordsworn_cavalier_counts_its_former_self_as_another_knight() {
    cr!("400.7", "611.3a");
    ruling!(
        "Swordsworn Cavalier",
        "If Swordsworn Cavalier enters the battlefield, leaves the battlefield, and returns to the battlefield all in the same turn, it will have first strike that turn."
    );
    supported("Swordsworn Cavalier");
    let mut t = TestGame::new(2);
    let first = enter(&mut t, P0, "Swordsworn Cavalier");
    assert!(!has(&mut t, first, KeywordKind::FirstStrike));
    let card = move_to(&mut t, first, Zone::Hand(P0)).unwrap();
    // The card enters again (a new object, CR 400.7).
    move_to(&mut t, card, Zone::Battlefield);
    let now = t.g.current(card);
    assert!(has(&mut t, now, KeywordKind::FirstStrike));
}

#[test]
fn swordsworn_cavalier_keeps_first_strike_after_the_other_knight_leaves() {
    cr!("611.3a", "700.4");
    ruling!(
        "Swordsworn Cavalier",
        "Swordsworn Cavalier will continue to have first strike even if the other Knight that entered the battlefield that turn has since left the battlefield, has since stopped being a Knight, or another player has gained control of it."
    );
    for how in 0..2 {
        let mut t = TestGame::new(2);
        let cav = t.battlefield(P0, "Swordsworn Cavalier");
        assert!(!has(&mut t, cav, KeywordKind::FirstStrike));
        let knight = enter(&mut t, P0, "White Knight");
        assert!(has(&mut t, cav, KeywordKind::FirstStrike));
        match how {
            0 => destroy(&mut t, knight),
            _ => give_control(&mut t, knight, P1),
        }
        assert!(has(&mut t, cav, KeywordKind::FirstStrike), "case {how}");
    }
}

#[test]
fn toolcraft_exemplar_intervening_if_and_the_count_on_resolution() {
    cr!("603.4", "608.2h");
    ruling!(
        "Toolcraft Exemplar",
        "If you control no artifacts as your combat phase begins, Toolcraft Exemplar's triggered ability doesn't trigger at all. If you control no artifacts as the triggered ability resolves, it has no effect."
    );
    ruling!(
        "Toolcraft Exemplar",
        "Whether or not you control three or more artifacts is checked only as the triggered ability resolves."
    );
    supported("Toolcraft Exemplar");
    let to_combat = |t: &mut TestGame| {
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::BeginningOfCombat);
        t.settle();
    };
    // No artifacts: no trigger.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Toolcraft Exemplar");
    to_combat(&mut t);
    assert_eq!(t.stack_len(), 0);
    // An artifact as combat begins, gone as it resolves: no effect.
    let mut t = TestGame::new(2);
    let ex2 = t.battlefield(P0, "Toolcraft Exemplar");
    let art = t.battlefield(P0, "Ornithopter");
    to_combat(&mut t);
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, art);
    t.resolve_all();
    assert_eq!(t.pt(ex2), (1, 1));
    // One artifact as it triggers, three as it resolves: +2/+1 and first strike.
    let mut t = TestGame::new(2);
    let ex3 = t.battlefield(P0, "Toolcraft Exemplar");
    t.battlefield(P0, "Ornithopter");
    to_combat(&mut t);
    assert_eq!(t.stack_len(), 1);
    t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Ornithopter");
    t.resolve_all();
    assert_eq!(t.pt(ex3), (3, 2));
    assert!(has(&mut t, ex3, KeywordKind::FirstStrike));
}

#[test]
fn opal_champion_becomes_a_creature_before_the_spell_resolves() {
    cr!("603.2", "601.2i", "608.3");
    ruling!(
        "Opal Champion",
        "It triggers when the spell is cast, so it becomes a creature before the spell resolves. A creature that would destroy an enchantment when it enters can't be used to destroy this card, since it will no longer be an enchantment when the creature enters."
    );
    supported("Opal Champion");
    let mut t = TestGame::new(2);
    let opal = t.battlefield(P0, "Opal Champion");
    t.set_step(P1, Step::PrecombatMain);
    crate::r_s25_common::lands_for_cost(&mut t, P1, "Cloudchaser Kestrel");
    let kestrel = t.hand(P1, "Cloudchaser Kestrel");
    t.cast(P1, kestrel).go();
    t.settle();
    // The trigger resolves first.
    t.resolve();
    assert!(t.obj_now(opal).is(mtg_engine::types::CardType::Creature));
    assert!(!t.obj_now(opal).is(mtg_engine::types::CardType::Enchantment));
    assert_eq!(t.pt(opal), (3, 3));
    t.resolve_all();
    assert!(t.on_battlefield(opal), "the Kestrel had no enchantment to destroy");
}

#[test]
fn moorland_inquisitor_extra_first_strike_is_redundant() {
    cr!("702.7d", "510.4");
    ruling!(
        "Moorland Inquisitor",
        "Multiple instances of first strike on the same creature are redundant."
    );
    supported("Moorland Inquisitor");
    let mut t = TestGame::new(2);
    let inq = t.battlefield(P0, "Moorland Inquisitor");
    for _ in 0..2 {
        pool(&mut t, P0, ManaType::W, 3);
        t.activate(P0, inq, 0, &[]).unwrap();
        t.resolve_all();
    }
    assert!(has(&mut t, inq, KeywordKind::FirstStrike));
    to_first_strike_damage(&mut t, &[inq], &[]);
    assert_eq!(t.life(P1), 18);
    finish_combat(&mut t);
    assert_eq!(t.life(P1), 18, "it dealt combat damage once");
}

#[test]
fn monoxa_activated_die_roll_triggers_its_roll_ability() {
    cr!("706.2", "603.2");
    ruling!(
        "Monoxa, Midway Manager",
        "The die roll caused by the activated ability has no inherent effect: you just roll a six-sided die, but any ability that triggers whenever you roll a die will trigger. Abilities that trigger whenever you roll a specific result, such as Monoxa's first ability, may also trigger."
    );
    supported("Monoxa, Midway Manager");
    let mut t = TestGame::new(2);
    let mono = t.battlefield(P0, "Monoxa, Midway Manager");
    crate::r_s35_common::load_dice(&mut t, &[5]);
    pool(&mut t, P0, ManaType::C, 6);
    t.activate(P0, mono, 0, &[]).unwrap();
    t.resolve_all();
    assert!(has(&mut t, mono, KeywordKind::FirstStrike));
    assert!(has(&mut t, mono, KeywordKind::Menace));
    assert!(has(&mut t, mono, KeywordKind::Lifelink));
    // A 2: nothing.
    let mut t = TestGame::new(2);
    let mono = t.battlefield(P0, "Monoxa, Midway Manager");
    crate::r_s35_common::load_dice(&mut t, &[2]);
    pool(&mut t, P0, ManaType::C, 6);
    t.activate(P0, mono, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!has(&mut t, mono, KeywordKind::FirstStrike));
}

#[test]
fn slaughter_priest_triggers_on_sacrifices_made_some_other_way() {
    cr!("603.2", "602.1");
    ruling!(
        "Slaughter-Priest of Mogis",
        "The first ability of Slaughter-Priest of Mogis is a triggered ability, not an activated ability. It doesn't allow you to sacrifice a permanent whenever you want; rather, you need some other way of sacrificing permanents, such as its second ability."
    );
    supported("Slaughter-Priest of Mogis");
    let mut t = TestGame::new(2);
    let priest = t.battlefield(P0, "Slaughter-Priest of Mogis");
    let bears = t.battlefield(P0, "Grizzly Bears");
    // Its only activated ability is the first strike one.
    let n_activated = t
        .obj_now(priest)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .count();
    assert_eq!(n_activated, 1);
    pool(&mut t, P0, ManaType::C, 2);
    t.answer_choose(P0, &[obj(bears)]);
    t.activate(P0, priest, 0, &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.pt(priest), (4, 2));
    assert!(has(&mut t, priest, KeywordKind::FirstStrike));
}

/// P0 controls Ambush Viper (flash) and has P1's turn's priority with a Swamp: P0 casts
/// Mutual Destruction sacrificing `sac`, targeting P1's Bears. Returns the spell.
fn mutual_destruction(t: &mut TestGame, sac: &str) -> (ObjectId, ObjectId, ObjectId) {
    supported("Mutual Destruction");
    let viper = t.battlefield(P0, "Ambush Viper");
    let sac = if sac == "Ambush Viper" {
        viper
    } else {
        t.battlefield(P0, sac)
    };
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 1);
    let md = t.hand(P0, "Mutual Destruction");
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(castable(t, P0, md), "it has flash");
    t.g.turn.priority = Some(P0);
    t.answer_choose(P0, &[obj(sac)]);
    let spell = t.cast(P0, md).target(bears).go();
    (spell, viper, bears)
}

#[test]
fn mutual_destruction_resolves_even_if_it_loses_flash_after_being_cast() {
    cr!("601.3", "601.2", "608.2");
    ruling!(
        "Mutual Destruction",
        "If it loses flash after it’s been cast, it will still resolve if able."
    );
    let mut t = TestGame::new(2);
    let (_, viper, bears) = mutual_destruction(&mut t, "Hill Giant");
    assert!(t.in_graveyard(P0, "Hill Giant"));
    destroy(&mut t, viper);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn mutual_destruction_may_sacrifice_the_creature_that_gave_it_flash() {
    cr!("601.3", "601.2h");
    ruling!(
        "Mutual Destruction",
        "If you sacrifice a creature with flash to cast Mutual Destruction and then have no other permanents with flash, you’ll legally finish casting Mutual Destruction even though it no longer has flash."
    );
    let mut t = TestGame::new(2);
    let (spell, _, _) = mutual_destruction(&mut t, "Ambush Viper");
    assert!(t.in_graveyard(P0, "Ambush Viper"));
    assert!(!t.obj_now(spell).has_keyword(KeywordKind::Flash));
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}
