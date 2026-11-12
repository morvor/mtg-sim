//! Rulings batch P196 — characteristic-changing effects with no duration (CR 611.2,
//! 613): "becomes" effects that overwrite creature types and base power and toughness
//! (613.1d, 613.4b), abilities gained afterwards (613.7), effects that apply only to the
//! objects there as they began (611.2c), new objects (400.7), and copies that don't enter
//! (707.4).

use crate::r_p196_common::*;
use crate::r_s02_common::can_attack;
use crate::r_s20_common::tap_for_mana;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::{CardType, ColorSet, Supertype};
use mtg_engine::*;

/// Adds `types` to the permanent's creature types until end of turn.
fn add_types(t: &mut TestGame, id: ObjectId, types: &[&str]) {
    crate::r_s26_common::modify_until_eot(
        t,
        id,
        vec![mtg_engine::ability::Modification::AddSubtypes(
            types.iter().map(|s| (*s).into()).collect(),
        )],
    );
}

fn has(t: &TestGame, id: ObjectId, kw: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(kw)
}

#[test]
fn evolved_sleeper_first_ability_again_overwrites_the_later_step() {
    cr!("613.1d", "613.4b", "613.7");
    ruling!(
        "Evolved Sleeper",
        "Evolved Sleeper's first two abilities overwrite its power, toughness, and creature types. Typically, those abilities are activated in the order they appear on the card. However, if Evolved Sleeper is a 3/3 Phyrexian Human Cleric and you activate its first ability again, it will become a 2/2 Human Cleric as that ability resolves."
    );
    supported("Evolved Sleeper");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Evolved Sleeper");
    act(&mut t, P0, s, 0, &[]);
    act(&mut t, P0, s, 1, &[]);
    assert_eq!(t.pt(s), (3, 3));
    assert_eq!(creature_types(&t, s), vec!["Cleric", "Human", "Phyrexian"]);
    act(&mut t, P0, s, 0, &[]);
    assert_eq!(t.pt(s), (2, 2));
    assert_eq!(creature_types(&t, s), vec!["Cleric", "Human"]);
    // The deathtouch counter stays.
    assert_eq!(t.counters(s, "deathtouch"), 1);
}

#[test]
fn evolved_sleeper_checks_its_types_however_it_got_them() {
    cr!("608.2c", "613.1d");
    ruling!(
        "Evolved Sleeper",
        "Evolved Sleeper's second ability checks whether it's a Cleric, and its third ability checks whether it's a Phyrexian. It doesn't matter how it became the appropriate creature type."
    );
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Evolved Sleeper");
    add_types(&mut t, s, &["Cleric"]);
    act(&mut t, P0, s, 1, &[]);
    assert_eq!(t.pt(s), (3, 3));
    assert_eq!(t.counters(s, "deathtouch"), 1);
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Evolved Sleeper");
    add_types(&mut t, s, &["Phyrexian"]);
    let hand = t.hand_size(P0);
    act(&mut t, P0, s, 2, &[]);
    assert_eq!(t.counters(s, "+1/+1"), 1);
    assert_eq!(t.hand_size(P0), hand + 1);
    assert_eq!(t.life(P0), 19);
}

#[test]
fn figure_of_destiny_first_ability_keeps_flying_and_first_strike() {
    cr!("613.1d", "613.4b", "611.2a");
    ruling!(
        "Figure of Destiny",
        "Figure of Destiny's abilities overwrite its power, toughness, and creature types. Typically, those abilities are activated in the order they appear on the card. However, if Figure of Destiny is an 8/8 Kithkin Spirit Warrior Avatar with flying and first strike, and you activate its first ability, it will become a 2/2 Kithkin Spirit that still has flying and first strike."
    );
    supported("Figure of Destiny");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Figure of Destiny");
    for i in 0..3 {
        act(&mut t, P0, f, i, &[]);
    }
    assert_eq!(t.pt(f), (8, 8));
    act(&mut t, P0, f, 0, &[]);
    assert_eq!(t.pt(f), (2, 2));
    assert_eq!(creature_types(&t, f), vec!["Kithkin", "Spirit"]);
    assert!(has(&t, f, KeywordKind::Flying) && has(&t, f, KeywordKind::FirstStrike));
}

#[test]
fn figure_of_destiny_checks_its_types_however_it_got_them() {
    cr!("608.2c", "613.1d");
    ruling!(
        "Figure of Destiny",
        "Figure of Destiny's second ability checks whether it's a Spirit, and its third ability checks whether it's a Warrior. It doesn't matter how it became the appropriate creature type."
    );
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Figure of Destiny");
    add_types(&mut t, f, &["Spirit"]);
    act(&mut t, P0, f, 1, &[]);
    assert_eq!(t.pt(f), (4, 4));
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Figure of Destiny");
    add_types(&mut t, f, &["Warrior"]);
    act(&mut t, P0, f, 2, &[]);
    assert_eq!(t.pt(f), (8, 8));
    assert!(has(&t, f, KeywordKind::Flying));
}

#[test]
fn figure_of_fable_overwrites_its_creature_types() {
    cr!("613.1d", "205.1a");
    ruling!(
        "Figure of Fable",
        "Figure of Fable's abilities overwrite its existing creature types. For example, once Figure of Fable's second ability resolves, if it was a Scout when the ability resolves, Figure of Fable will be a Kithkin Soldier. It won't have the Scout creature type."
    );
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Figure of Fable");
    act(&mut t, P0, f, 0, &[]);
    act(&mut t, P0, f, 1, &[]);
    assert_eq!(t.pt(f), (4, 5));
    assert_eq!(creature_types(&t, f), vec!["Kithkin", "Soldier"]);
}

/// Gornog attacks P1; its trigger makes `target` (P1's) a Coward.
fn gornog_cowardice(t: &mut TestGame, g: ObjectId, target: ObjectId) {
    t.answer_targets(P0, &[obj(target)]);
    crate::r_s01_common::attack_with(t, &[(g, pl(P1))]);
    t.resolve_all();
}

#[test]
fn gornog_coward_overwrites_creature_types_only_and_lasts() {
    cr!("611.2a", "613.1d", "205.1a");
    ruling!(
        "Gornog, the Red Reaper",
        "Gornog's third ability overwrites the target's existing creature types. Once the ability resolves, that creature will be a Coward and won't have its previous creature types. It retains any other types and supertypes it has as well as any subtypes that aren't creature types."
    );
    ruling!(
        "Gornog, the Red Reaper",
        "Gornog's third ability doesn't have a duration. It will remain in effect until the game ends, the target creature leaves the battlefield, or some subsequent effect changes the creature's characteristics, whichever comes first."
    );
    supported("Gornog, the Red Reaper");
    supported("Dryad Arbor");
    let mut t = TestGame::new(2);
    let arbor = t.battlefield(P1, "Dryad Arbor");
    let legend = t.battlefield(P1, "Drana, the Last Bloodchief");
    let g = t.battlefield(P0, "Gornog, the Red Reaper");
    gornog_cowardice(&mut t, g, arbor);
    let o = t.obj_now(arbor);
    assert!(o.is(CardType::Land) && o.is(CardType::Creature));
    assert!(o.chars.has_subtype("Forest") && o.chars.has_subtype("Coward"));
    assert!(!o.chars.has_subtype("Dryad"));
    // Supertypes stay: a legendary creature is still legendary.
    pass_turn(&mut t);
    pass_turn(&mut t);
    gornog_cowardice(&mut t, g, legend);
    let o = t.obj_now(legend);
    assert!(o.chars.supertypes.contains(Supertype::Legendary));
    assert_eq!(creature_types(&t, legend), vec!["Coward"]);
    // Turns later, still Cowards; a later effect changes it.
    pass_turn(&mut t);
    pass_turn(&mut t);
    assert_eq!(creature_types(&t, arbor), vec!["Coward"]);
    let oko = t.battlefield(P0, "Oko, Thief of Crowns");
    t.set_step(P0, Step::PostcombatMain);
    act(&mut t, P0, oko, 1, &[obj(arbor)]);
    assert_eq!(creature_types(&t, arbor), vec!["Elk"]);
}

#[test]
fn oko_toughness_change_makes_marked_damage_lethal() {
    cr!("704.5g", "120.6", "613.4b");
    ruling!(
        "Oko, Thief of Crowns",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a creature may become lethal if Oko's second ability changes its toughness during that turn."
    );
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, "Oko, Thief of Crowns");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let src = t.battlefield(P0, "Hill Giant");
    crate::r_s06_common::damage(&mut t, src, 3, wurm);
    assert!(t.on_battlefield(wurm));
    act(&mut t, P0, oko, 1, &[obj(wurm)]);
    assert!(!t.on_battlefield(wurm));
    assert!(t.in_graveyard(P1, "Craw Wurm"));
}

#[test]
fn abilities_gained_after_oko_or_abigale_are_kept() {
    cr!("613.1f", "613.7", "122.1b");
    ruling!(
        "Oko, Thief of Crowns",
        "If the affected creature gains an ability after Oko's second ability resolves, it will keep that ability."
    );
    ruling!(
        "Abigale, Eloquent First-Year",
        "If the affected creature gains an ability after Abigale's last ability resolves, it will keep that ability."
    );
    supported("Abigale, Eloquent First-Year");
    // Oko's +1 on Serra Angel, then Rancor: trample, but no flying or vigilance.
    let mut t = TestGame::new(2);
    let oko = t.battlefield(P0, "Oko, Thief of Crowns");
    let angel = t.battlefield(P0, "Serra Angel");
    act(&mut t, P0, oko, 1, &[obj(angel)]);
    assert!(!has(&t, angel, KeywordKind::Flying));
    attach_new(&mut t, P0, "Rancor", angel);
    assert!(has(&t, angel, KeywordKind::Trample));
    assert!(!has(&t, angel, KeywordKind::Vigilance));
    assert_eq!(t.pt(angel), (5, 3));
    // Abigale on Serra Angel: it loses vigilance, gets the three keyword counters'
    // abilities, and keeps an ability gained later.
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P1, "Serra Angel");
    t.answer_targets(P0, &[obj(angel)]);
    t.enter(P0, "Abigale, Eloquent First-Year");
    t.g.flush_events();
    t.resolve_all();
    assert!(!has(&t, angel, KeywordKind::Vigilance));
    for kw in [
        KeywordKind::Flying,
        KeywordKind::FirstStrike,
        KeywordKind::Lifelink,
    ] {
        assert!(has(&t, angel, kw), "{kw:?}");
    }
    attach_new(&mut t, P1, "Rancor", angel);
    assert!(has(&t, angel, KeywordKind::Trample));
    pass_turn(&mut t);
    assert!(has(&t, angel, KeywordKind::Trample));
    assert!(!has(&t, angel, KeywordKind::Vigilance));
}

#[test]
fn hexproof_until_it_deals_damage_targets_chosen_after_the_damage() {
    cr!("603.3d", "702.11b", "603.2");
    ruling!(
        "Karakyk Guardian",
        "If a triggered ability triggers at the same time that Karakyk Guardian deals damage for the first time, targets for that ability are chosen after Karakyk Guardian no longer has hexproof."
    );
    ruling!(
        "Palladia-Mors, the Ruiner",
        "If a triggered ability triggers at the same time that Paladia-Mors deals damage for the first time, targets are chosen for that ability after Palladia-Mors no longer has hexproof."
    );
    // It deals damage to P1's Screaming Nemesis ("Whenever this creature is dealt
    // damage, it deals that much damage to any other target"), whose trigger P1 aims at
    // it.
    for name in ["Karakyk Guardian", "Palladia-Mors, the Ruiner"] {
        supported(name);
        let mut t = TestGame::new(2);
        let big = t.battlefield(P0, name);
        let nem = t.battlefield(P1, "Screaming Nemesis");
        assert!(has(&t, big, KeywordKind::Hexproof), "{name}");
        let from = t.asked().len();
        t.answer_targets(P1, &[obj(big)]);
        crate::r_s06_common::damage(&mut t, big, 1, nem);
        let cands = crate::r_s02_common::target_candidates(&t, P1, from);
        assert!(
            cands.iter().any(|c| c.contains(&obj(big))),
            "{name}: {cands:?}"
        );
        t.resolve_all();
        assert!(!has(&t, big, KeywordKind::Hexproof), "{name}");
        assert_eq!(t.obj_now(big).damage, 1, "{name}");
    }
}

#[test]
fn karakyk_guardian_has_hexproof_again_as_a_new_object() {
    cr!("400.7", "702.11b");
    ruling!(
        "Karakyk Guardian",
        "If Karakyk Guardian leaves the battlefield and returns, it’s considered a new object even if it’s represented by the same card. It has hexproof again until it deals damage."
    );
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Karakyk Guardian");
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::damage(&mut t, k, 1, bears);
    assert!(!has(&t, k, KeywordKind::Hexproof));
    crate::r_p189_common::bounce(&mut t, k);
    let card = t.g.find_in_zone(Zone::Hand(P0), "Karakyk Guardian")[0];
    t.g.move_object(card, Zone::Battlefield, events::MoveCause::Effect, Some(P0));
    t.g.recompute();
    let k2 = t.named_on_battlefield("Karakyk Guardian")[0];
    assert!(has(&t, k2, KeywordKind::Hexproof));
}

#[test]
fn moonlace_on_a_permanent_spell_keeps_the_permanent_colorless() {
    cr!("611.2a", "105.2c", "400.7a");
    ruling!(
        "Moonlace",
        "If Moonlace is cast on a spell that becomes a permanent (a creature spell, an artifact spell, or an enchantment spell), the permanent will enter colorless and will remain colorless until it leaves the battlefield (or until another effect changes its color)."
    );
    supported("Moonlace");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::PrecombatMain);
    let spell = cast_at(&mut t, P1, "Grizzly Bears", &[]);
    cast_at(&mut t, P0, "Moonlace", &[obj(spell)]);
    t.resolve_all();
    let bears = t.named_on_battlefield("Grizzly Bears")[0];
    assert_eq!(t.obj_now(bears).chars.colors, ColorSet::NONE);
    pass_turn(&mut t);
    pass_turn(&mut t);
    assert_eq!(t.obj_now(bears).chars.colors, ColorSet::NONE);
}

#[test]
fn ride_the_shoopuf_needs_continuous_control_to_attack() {
    cr!("302.6", "508.1a");
    ruling!(
        "Ride the Shoopuf",
        "If Ride the Shoopuf becomes a creature but you haven't controlled it continuously since your most recent turn began, you won't be able to attack with it that turn."
    );
    supported("Ride the Shoopuf");
    for fresh in [true, false] {
        let mut t = TestGame::new(2);
        let r = if fresh {
            crate::r_s20_common::entered_this_turn(&mut t, P0, "Ride the Shoopuf")
        } else {
            t.battlefield(P0, "Ride the Shoopuf")
        };
        act(&mut t, P0, r, 0, &[]);
        assert_eq!(t.pt(r), (7, 7));
        t.set_step(P0, Step::BeginningOfCombat);
        assert_eq!(can_attack(&mut t, r), !fresh, "fresh {fresh}");
    }
}

#[test]
fn screaming_nemesis_triggers_once_for_simultaneous_lethal_damage() {
    cr!("603.2c", "510.2", "119.5", "119.7");
    ruling!(
        "Screaming Nemesis",
        "If Screaming Nemesis is dealt damage by multiple sources at once, such as by two creatures blocking it, its last ability triggers once and one target is dealt that much damage."
    );
    ruling!(
        "Screaming Nemesis",
        "If lethal damage is dealt to Screaming Nemesis, its last ability still triggers."
    );
    ruling!(
        "Screaming Nemesis",
        "If an effect says to set a player's life total to a number that's higher than their current life total while they're affected by Screaming Nemesis's last ability, their life total won't change."
    );
    supported("Screaming Nemesis");
    supported("Blessed Wind");
    let mut t = TestGame::new(2);
    let nem = t.battlefield(P0, "Screaming Nemesis");
    let b1 = t.battlefield(P1, "Grizzly Bears");
    let b2 = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[pl(P1)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(nem, pl(P1))], &[(b1, nem), (b2, nem)]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Screaming Nemesis"));
    let hits: Vec<u32> = crate::r_p194_common::damage_events(&t)
        .into_iter()
        .filter(|(s, to, _)| *s == nem && *to == pl(P1))
        .map(|(_, _, n)| n)
        .collect();
    assert_eq!(hits, vec![4]);
    assert_eq!(t.life(P1), 16);
    // P1's life total can't be set higher.
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Blessed Wind", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
}

#[test]
fn stigma_lasher_player_cant_gain_life_in_any_way() {
    cr!("119.5", "119.7", "118.3", "614.1a");
    ruling!(
        "Stigma Lasher",
        "If a player has been dealt damage by Stigma Lasher, the following things are true: — Spells and abilities that would normally cause that player to gain life still resolve, but the life-gain part simply has no effect."
    );
    supported("Stigma Lasher");
    supported("Revitalize");
    supported("Tainted Remedy");
    // P1 at 25 is dealt 2 by the Lasher.
    let setup = |t: &mut TestGame| {
        t.g.players[1].life = 25;
        let lasher = t.battlefield(P0, "Stigma Lasher");
        damage(t, lasher, pl(P1), 2);
        t.resolve_all();
        assert_eq!(t.life(P1), 23);
    };
    // Revitalize resolves: P1 draws but gains nothing.
    let mut t = TestGame::new(2);
    setup(&mut t);
    let hand = t.hand_size(P1);
    cast_at(&mut t, P1, "Revitalize", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 23);
    assert_eq!(t.hand_size(P1), hand + 1);
    // Invigorate's alternative cost (an opponent gains 3 life) can't be paid; it can
    // when P1 hasn't been dealt damage by the Lasher.
    for lashed in [true, false] {
        let mut t = TestGame::new(2);
        if lashed {
            setup(&mut t);
        }
        t.battlefield(P0, "Forest");
        t.battlefield(P0, "Grizzly Bears");
        let inv = t.hand(P0, "Invigorate");
        let alt = crate::r_s07_common::cast_methods(&mut t, P0, inv)
            .into_iter()
            .any(|m| matches!(m, mtg_engine::object::CastMethod::Alternative(_)));
        assert_eq!(alt, !lashed, "lashed {lashed}");
    }
    // Tainted Remedy ("If an opponent would gain life, that player loses that much life
    // instead") has nothing to replace.
    let mut t = TestGame::new(2);
    setup(&mut t);
    t.battlefield(P0, "Tainted Remedy");
    cast_at(&mut t, P1, "Revitalize", &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 23);
    // Words of Worship replaces the draw with nothing.
    let mut t = TestGame::new(2);
    setup(&mut t);
    let words = t.battlefield(P1, "Words of Worship");
    act(&mut t, P1, words, 0, &[]);
    let hand = t.hand_size(P1);
    let lib = t.library_size(P1);
    t.g.draw_cards(P1, 1);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.hand_size(P1), hand);
    assert_eq!(t.library_size(P1), lib);
    assert_eq!(t.life(P1), 23);
    // Setting the life total: 20 (lower) works; then 20 again from 18 doesn't raise it.
    let mut t = TestGame::new(2);
    setup(&mut t);
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Blessed Wind", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    crate::r_p009_common::lose(&mut t, P1, 2);
    cast_at(&mut t, P1, "Blessed Wind", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
}

#[test]
fn tezzerets_minus_two_on_a_vehicle_isnt_crewing() {
    cr!("702.122b", "702.122e");
    ruling!(
        "Tezzeret, Betrayer of Flesh",
        "If Tezzeret's second loyalty ability causes a Vehicle to become an artifact creature, it doesn't count as \"crewing\" that Vehicle for any ability that would trigger off of a Vehicle becoming crewed."
    );
    supported("Mobilizer Mech");
    let mut t = TestGame::new(2);
    let tez = t.battlefield(P0, "Tezzeret, Betrayer of Flesh");
    let mech = t.battlefield(P0, "Mobilizer Mech");
    t.battlefield(P0, "Smuggler's Copter");
    act_only(&mut t, P0, tez, 1, &[obj(mech)]);
    t.resolve();
    assert!(t.obj_now(mech).is(CardType::Creature));
    // A Vehicle keeps its printed power and toughness.
    assert_eq!(t.pt(mech), (3, 4));
    t.settle();
    assert_eq!(t.stack_len(), 0, "no crewed trigger");
    // Crewing it does trigger.
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(crate::r_s04_common::crew(&mut t, P0, mech, &[giant]));
    t.resolve();
    assert!(crate::r_s14_common::triggers_from(&t, mech) >= 1);
}

#[test]
fn uncharted_haven_without_a_chosen_color_makes_no_mana() {
    cr!("106.5", "607.2d");
    ruling!(
        "Uncharted Haven",
        "If Uncharted Haven is somehow on the battlefield without a chosen color, its mana ability won't make any mana."
    );
    supported("Uncharted Haven");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Uncharted Haven");
    assert!(!tap_for_mana(&mut t, P0, h, "chosen color"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    // Chosen as it enters: it does.
    let mut t = TestGame::new(2);
    choose_color(&mut t, P0, mtg_engine::types::Color::Green);
    let h = t.enter(P0, "Uncharted Haven");
    t.g.untap(h);
    assert!(tap_for_mana(&mut t, P0, h, "chosen color"));
}

#[test]
fn avatar_kyoshis_mana_ability_needs_positive_power() {
    cr!("605.1a", "605.3b", "106.4", "107.1b");
    ruling!(
        "The Legend of Kyoshi // Avatar Kyoshi",
        "Avatar Kyoshi's last ability is a mana ability. It doesn't use the stack and can't be responded to. If the greatest power among creatures you control is 0 or less, no mana is added."
    );
    supported("The Legend of Kyoshi // Avatar Kyoshi");
    let mut t = TestGame::new(2);
    let k =
        crate::r_s17_common::enter_transformed(&mut t, P0, "The Legend of Kyoshi // Avatar Kyoshi");
    t.g.objects[k.0 as usize].summoning_sick = false;
    let (p, _) = t.pt(k);
    assert!(tap_for_mana(&mut t, P0, k, "Add X mana"));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P0).mana_pool.total() as i32, p);
    // Power 0 or less: nothing.
    t.g.players[0].mana_pool.empty();
    t.g.untap(k);
    pump(&mut t, k, -p - 2, 0);
    assert_eq!(t.pt(k).0, -2);
    assert!(!tap_for_mana(&mut t, P0, k, "Add X mana"));
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
}

#[test]
fn urzas_saga_is_sacrificed_after_its_last_chapter() {
    cr!("714.4", "704.5s");
    ruling!(
        "Urza's Saga",
        "Even though Urza's Saga is a land, it is also still a Saga, and it will be sacrificed after its last chapter ability resolves."
    );
    supported("Urza's Saga");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Urza's Saga");
    t.g.objects[saga.0 as usize]
        .counters
        .insert("lore".into(), 2);
    crate::r_s19_common::add_lore(&mut t, saga, 1);
    assert_eq!(t.stack_len(), 1);
    assert!(
        t.on_battlefield(saga),
        "not while the chapter is on the stack"
    );
    t.resolve_all();
    assert!(!t.on_battlefield(saga));
    assert!(t.in_graveyard(P0, "Urza's Saga"));
}

#[test]
fn druid_class_land_creature_tracks_the_number_of_lands() {
    cr!("604.3", "613.4a", "716.2a");
    ruling!(
        "Druid Class",
        "For the third ability, the power and toughness of the creature will change as the number of lands you control changes."
    );
    supported("Druid Class");
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Druid Class");
    let forest = t.battlefield(P0, "Forest");
    t.battlefield(P0, "Forest");
    lots(&mut t, P0);
    crate::r_s19_common::gain_level(&mut t, P0, class, 2).unwrap();
    t.resolve_all();
    lots(&mut t, P0);
    crate::r_s19_common::gain_level(&mut t, P0, class, 3).unwrap();
    t.answer_targets(P0, &[obj(forest)]);
    t.resolve_all();
    assert!(t.obj_now(forest).is(CardType::Creature));
    assert_eq!(t.pt(forest), (2, 2));
    t.battlefield(P0, "Island");
    assert_eq!(t.pt(forest), (3, 3));
    pass_turn(&mut t);
    assert_eq!(t.pt(forest), (3, 3));
    assert!(has(&t, forest, KeywordKind::Haste));
}

#[test]
fn opal_creatures_change_even_if_the_spell_is_countered_and_stay_changed() {
    cr!("603.2", "611.2a");
    ruling!(
        "Opal Acrolith",
        "Changes into a creature even if the spell is countered."
    );
    ruling!("Opal Guardian", "Guardian's effect doesn't wear off.");
    supported("Opal Acrolith");
    supported("Opal Guardian");
    let mut t = TestGame::new(2);
    let acro = t.battlefield(P0, "Opal Acrolith");
    let guard = t.battlefield(P0, "Opal Guardian");
    t.set_step(P1, Step::PrecombatMain);
    let spell = cast_at(&mut t, P1, "Grizzly Bears", &[]);
    t.settle();
    cast_at(&mut t, P0, "Counterspell", &[obj(spell)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.obj_now(acro).is(CardType::Creature));
    assert_eq!(t.pt(acro), (2, 4));
    assert!(t.obj_now(guard).is(CardType::Creature));
    pass_turn(&mut t);
    pass_turn(&mut t);
    assert!(t.obj_now(guard).is(CardType::Creature));
    assert_eq!(t.pt(guard), (3, 4));
    assert!(has(&t, guard, KeywordKind::Flying));
}

#[test]
fn cocoon_on_a_tapped_creature_and_moved_keeps_its_counters() {
    cr!("303.4", "122.1");
    ruling!("Cocoon", "Cocoon can be used on a tapped creature.");
    ruling!(
        "Cocoon",
        "If the Aura is moved to a different creature, the number of counters is unchanged."
    );
    supported("Cocoon");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.tap(bears);
    let c = cast_at(&mut t, P0, "Cocoon", &[obj(bears)]);
    t.resolve_all();
    let aura = t.g.current(c);
    assert_eq!(attached_to(&t, aura), Some(obj(bears)));
    assert_eq!(t.counters(aura, "pupa"), 3);
    let giant = t.battlefield(P0, "Hill Giant");
    assert!(t.g.attach(aura, obj(giant)));
    t.g.recompute();
    assert_eq!(attached_to(&t, aura), Some(obj(giant)));
    assert_eq!(t.counters(aura, "pupa"), 3);
}

#[test]
fn chance_for_glory_affects_only_creatures_there_and_lasts() {
    cr!("611.2c", "611.2a", "500.7");
    ruling!(
        "Chance for Glory",
        "Chance for Glory’s first effect affects only creatures you control at the time it resolves. Creatures you begin to control later in the game won’t gain indestructible."
    );
    ruling!(
        "Chance for Glory",
        "Creatures you control gain indestructible indefinitely. If you find a way to not lose the game during your next turn, they’ll continue to be indestructible for as long as they remain on the battlefield."
    );
    supported("Chance for Glory");
    supported("Platinum Angel");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Chance for Glory", &[]);
    t.resolve_all();
    assert!(has(&t, bears, KeywordKind::Indestructible));
    let later = t.battlefield(P0, "Hill Giant");
    assert!(!has(&t, later, KeywordKind::Indestructible));
    t.battlefield(P0, "Platinum Angel");
    // The extra turn, then P1's turn: P0 hasn't lost and the Bears are indestructible.
    let turn = t.g.turn.number;
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(t.g.turn.number, turn + 1, "the extra turn");
    t.advance_to(P1, Step::PrecombatMain);
    assert!(!t.has_lost(P0));
    assert!(has(&t, bears, KeywordKind::Indestructible));
}

#[test]
fn mirrorform_copies_dont_enter_and_copy_what_the_target_copies() {
    cr!("707.2", "707.4", "603.6a", "614.12");
    ruling!(
        "Mirrorform",
        "Because the permanents aren't entering the battlefield when they become copies of the target permanent, any \"When [this permanent] enters\" or \"[This permanent] enters with\" abilities of the copied permanent won't apply."
    );
    ruling!(
        "Mirrorform",
        "If the copied permanent is copying something else, then the permanents become copies of whatever that permanent copied."
    );
    supported("Mirrorform");
    supported("Spike Feeder");
    // Elvish Visionary (draw on entering): no draw.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let ring = t.battlefield(P0, "Sol Ring");
    let vis = t.battlefield(P1, "Elvish Visionary");
    cast_at(&mut t, P0, "Mirrorform", &[obj(vis)]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.obj_now(bears).chars.name, "Elvish Visionary");
    assert_eq!(t.obj_now(ring).chars.name, "Elvish Visionary");
    assert_eq!(t.stack_len(), 0);
    // Spike Feeder (enters with two +1/+1 counters): no counters, so the 0/0 copies die.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let feeder = t.battlefield(P1, "Spike Feeder");
    t.g.add_counters(obj(feeder), "+1/+1", 2, None);
    cast_at(&mut t, P0, "Mirrorform", &[obj(feeder)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    // A Clone copying Hill Giant: P0's permanents become Hill Giants.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_choose(P1, &[obj(giant)]);
    t.answer_yes(P1, true);
    let clone = t.enter(P1, "Clone");
    t.settle();
    cast_at(&mut t, P0, "Mirrorform", &[obj(clone)]);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).chars.name, "Hill Giant");
    assert_eq!(t.pt(bears), (3, 3));
}

#[test]
fn arcums_weathervane_snow_rules() {
    cr!("205.4g", "205.4a", "115.1");
    ruling!(
        "Arcum's Weathervane",
        "Can’t be used on a dual land to add the Snow Supertype even though it is a basic land subtype. You can use it on any land to remove the snow supertype."
    );
    supported("Arcum's Weathervane");
    supported("Tundra");
    supported("Mouth of Ronom");
    let mut t = TestGame::new(2);
    let vane = t.battlefield(P0, "Arcum's Weathervane");
    let tundra = t.battlefield(P0, "Tundra");
    let plains = t.battlefield(P0, "Plains");
    let mouth = t.battlefield(P1, "Mouth of Ronom");
    lots(&mut t, P0);
    let snowable = crate::r_s04_common::ability_targets(&mut t, vane, 1);
    assert!(snowable.contains(&obj(plains)));
    assert!(!snowable.contains(&obj(tundra)));
    // The nonbasic snow land loses snow.
    act(&mut t, P0, vane, 0, &[obj(mouth)]);
    assert!(!t.obj_now(mouth).chars.supertypes.contains(Supertype::Snow));
}

#[test]
fn kicked_kavus_keep_their_keyword() {
    cr!("611.2a", "702.33d", "614.1c");
    ruling!(
        "Kavu Titan",
        "If the Kavu Titan was kicked, it enters with trample and will keep this ability until it leaves the battlefield."
    );
    ruling!(
        "Pouncing Kavu",
        "If the Pouncing Kavu is kicked, it enters with haste and will keep this ability until it leaves the battlefield."
    );
    supported("Pouncing Kavu");
    for (name, kw, n) in [
        ("Kavu Titan", KeywordKind::Trample, 3),
        ("Pouncing Kavu", KeywordKind::Haste, 2),
    ] {
        let mut t = TestGame::new(2);
        lots(&mut t, P0);
        let c = t.hand(P0, name);
        t.cast(P0, c).kicked(true).go();
        t.resolve_all();
        let k = t.named_on_battlefield(name)[0];
        assert_eq!(t.counters(k, "+1/+1"), n, "{name}");
        pass_turn(&mut t);
        pass_turn(&mut t);
        assert!(has(&t, k, kw), "{name}");
    }
}

#[test]
fn praetors_counsel_and_another_hand_size_effect_apply_in_timestamp_order() {
    cr!("402.2", "613.11", "611.2a");
    ruling!(
        "Praetor's Counsel",
        "If multiple effects modify your hand size, apply them in timestamp order. For example, if you put Null Profusion (an enchantment that says your maximum hand size is two) onto the battlefield and then resolve Praetor's Counsel, you have no maximum hand size. However, if those events happened in the opposite order, your maximum hand size would be two."
    );
    supported("Praetor's Counsel");
    supported("Null Profusion");
    for profusion_first in [true, false] {
        let mut t = TestGame::new(2);
        if profusion_first {
            t.battlefield(P0, "Null Profusion");
        }
        cast_at(&mut t, P0, "Praetor's Counsel", &[]);
        t.resolve_all();
        if !profusion_first {
            t.battlefield(P0, "Null Profusion");
        }
        t.g.recompute();
        let want = if profusion_first { None } else { Some(2) };
        assert_eq!(t.g.player(P0).max_hand_size, want, "{profusion_first}");
    }
}

#[test]
fn donals_copy_becomes_a_token() {
    cr!("707.10", "111.1", "608.3f");
    ruling!(
        "Donal, Herald of Wings",
        "If a creature spell is copied, it's put onto the battlefield as a token as the spell resolves rather than putting the copy of the spell onto the battlefield. The rules that apply to a permanent spell becoming a permanent apply to a copy of a spell becoming a token."
    );
    supported("Donal, Herald of Wings");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Donal, Herald of Wings");
    t.answer_yes(P0, true);
    cast_at(&mut t, P0, "Serra Angel", &[]);
    t.resolve_all();
    let angels = t.named_on_battlefield("Serra Angel");
    assert_eq!(angels.len(), 2);
    let token: Vec<ObjectId> = angels
        .iter()
        .copied()
        .filter(|a| t.obj_now(*a).is_token())
        .collect();
    assert_eq!(token.len(), 1);
    assert_eq!(t.pt(token[0]), (1, 1));
    assert!(t.obj_now(token[0]).chars.has_subtype("Spirit"));
    assert!(t.obj_now(token[0]).summoning_sick);
}
