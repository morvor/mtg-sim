//! Rulings batch P066 — static abilities and triggers of creatures that gain menace,
//! trample, vigilance or protection: conditions checked continuously (CR 611.3a),
//! stickered permanents, "as enters" choices (CR 614.12), die rolls counted (CR 706),
//! damage replaced with -1/-1 counters (CR 614.1a), mana abilities (CR 605) and more.

use crate::r_p066_common::*;
use crate::r_s01_common::{supported, with_subtype};
use crate::r_s02_common::{create_token, destroy};
use crate::r_s06_common::damage;
use crate::r_s25_common::cast_new;
use crate::r_s29_common::put_counters;
use mtg_engine::ability::AbilityKind;
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::stickers::{put_sticker, StickerKind};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn has_kw(t: &TestGame, id: ObjectId, kw: KeywordKind) -> bool {
    t.obj_now(id).has_keyword(kw)
}

// ---------------------------------------------------------------------------
// Stickered permanents
// ---------------------------------------------------------------------------

#[test]
fn a_stickered_permanent_can_be_itself_or_another_permanent_you_control() {
    cr!("123.9", "611.3a");
    ruling!(
        "Scared Stiff",
        "The stickered permanent could be Scared Stiff itself or another permanent you control."
    );
    ruling!(
        "Big Winner",
        "The stickered permanent could be Big Winner itself or another permanent you control."
    );
    ruling!(
        "Grabby Tabby",
        "The stickered permanent could be Grabby Tabby itself or another permanent you control."
    );
    for (name, kw) in [
        ("Scared Stiff", KeywordKind::Menace),
        ("Big Winner", KeywordKind::Trample),
        ("Grabby Tabby", KeywordKind::Vigilance),
    ] {
        supported(name);
        // Itself.
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        assert!(!has_kw(&t, c, kw), "{name}");
        assert!(put_sticker(&mut t.g, P0, c, StickerKind::Art));
        t.g.recompute();
        assert!(has_kw(&t, c, kw), "{name}: itself");
        // Another permanent you control (a land).
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        let land = t.battlefield(P0, "Forest");
        assert!(put_sticker(&mut t.g, P0, land, StickerKind::Art));
        t.g.recompute();
        assert!(has_kw(&t, c, kw), "{name}: another permanent");
        // Not an opponent's stickered permanent.
        let mut t = TestGame::new(2);
        let c = t.battlefield(P0, name);
        let theirs = t.battlefield(P1, "Forest");
        assert!(put_sticker(&mut t.g, P1, theirs, StickerKind::Art));
        t.g.recompute();
        assert!(!has_kw(&t, c, kw), "{name}: an opponent's");
    }
}

// ---------------------------------------------------------------------------
// Conditions checked continuously
// ---------------------------------------------------------------------------

#[test]
fn bellowing_elk_counts_creatures_that_entered_before_it_and_left() {
    cr!("611.3a");
    ruling!(
        "Bellowing Elk",
        "Bellowing Elk’s effect applies even if the other creature entered the battlefield under your control before Bellowing Elk entered the battlefield, and it continues to apply even if that creature leaves the battlefield."
    );
    supported("Bellowing Elk");
    let mut t = TestGame::new(2);
    let bears = t.enter(P0, "Grizzly Bears");
    let elk = t.enter(P0, "Bellowing Elk");
    t.settle();
    assert!(has_kw(&t, elk, KeywordKind::Trample));
    assert!(has_kw(&t, elk, KeywordKind::Indestructible));
    destroy(&mut t, bears);
    assert!(has_kw(&t, elk, KeywordKind::Trample));
    assert!(has_kw(&t, elk, KeywordKind::Indestructible));
    // A creature entering under an opponent's control doesn't count.
    let mut t = TestGame::new(2);
    let elk = t.battlefield(P0, "Bellowing Elk");
    t.enter(P1, "Grizzly Bears");
    t.settle();
    assert!(!has_kw(&t, elk, KeywordKind::Trample));
}

#[test]
fn bellowing_elk_checks_each_turn() {
    cr!("611.3a");
    ruling!(
        "Bellowing Elk",
        "Bellowing Elk’s effect checks each turn whether it applies during that turn. It doesn’t check only during the turn it enters the battlefield."
    );
    supported("Bellowing Elk");
    let mut t = TestGame::new(2);
    let elk = t.enter(P0, "Bellowing Elk");
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert!(has_kw(&t, elk, KeywordKind::Trample));
    t.advance_to(P1, Step::Upkeep);
    assert!(!has_kw(&t, elk, KeywordKind::Trample));
    t.advance_to(P0, Step::PrecombatMain);
    assert!(!has_kw(&t, elk, KeywordKind::Trample));
    t.enter(P0, "Hill Giant");
    t.settle();
    assert!(has_kw(&t, elk, KeywordKind::Trample));
    assert!(has_kw(&t, elk, KeywordKind::Indestructible));
}

#[test]
fn hungry_ridgewolf_gets_plus_one_once_for_a_wolf_and_a_werewolf() {
    cr!("611.3a");
    ruling!(
        "Hungry Ridgewolf",
        "If you control both a Wolf and a Werewolf, Hungry Ridgewolf still only gets +1/+0."
    );
    supported("Hungry Ridgewolf");
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Hungry Ridgewolf");
    assert_eq!(t.pt(w), (2, 2));
    let wolf = t.battlefield(P0, "Russet Wolves");
    assert_eq!(t.pt(w), (3, 2));
    assert!(has_kw(&t, w, KeywordKind::Trample));
    t.battlefield(P0, "Village Ironsmith // Ironfang");
    assert_eq!(t.pt(w), (3, 2));
    // The Werewolf alone also counts.
    destroy(&mut t, wolf);
    assert_eq!(t.pt(w), (3, 2));
    assert!(has_kw(&t, w, KeywordKind::Trample));
}

#[test]
fn charging_war_boar_marked_damage_becomes_lethal_when_domri_leaves() {
    cr!("120.6", "704.5g", "611.3a");
    ruling!(
        "Charging War Boar",
        "Because damage remains marked on a creature until it’s removed as the turn ends, nonlethal damage dealt to Charging War Boar may become lethal if Domri leaves the battlefield during that turn."
    );
    supported("Charging War Boar");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Charging War Boar");
    let domri = t.battlefield(P0, "Domri Rade");
    assert_eq!(t.pt(b), (4, 2));
    let src = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, src, 1, b);
    assert!(t.on_battlefield(b));
    destroy(&mut t, domri);
    assert!(t.in_graveyard(P0, "Charging War Boar"));
}

// ---------------------------------------------------------------------------
// Faceless Haven
// ---------------------------------------------------------------------------

/// Animates Faceless Haven with three snow lands' mana.
fn animate_haven(t: &mut TestGame, haven: ObjectId) {
    // Snow mana from three snow lands (not from the Haven itself, a snow land too).
    for f in t.lands(P0, "Snow-Covered Forest", 3) {
        assert!(crate::r_s20_common::tap_for_mana(t, P0, f, "Add {G}"));
    }
    t.activate(P0, haven, 1, &[]).unwrap();
    assert!(!tapped(t, haven));
    t.resolve_all();
    assert!(t.obj_now(haven).is(CardType::Creature));
}

#[test]
fn faceless_haven_has_all_creature_types_but_not_changeling() {
    cr!("205.3m", "702.73a");
    ruling!(
        "Faceless Haven",
        "Although Faceless Haven will have all creature types after its last activated ability resolves, it will not have the changeling keyword ability."
    );
    supported("Faceless Haven");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Faceless Haven");
    animate_haven(&mut t, h);
    for s in ["Elf", "Sliver", "Zombie", "Dragon"] {
        assert!(t.obj_now(h).chars.has_subtype(s), "{s} missing");
    }
    assert!(has_kw(&t, h, KeywordKind::Vigilance));
    assert!(!has_kw(&t, h, KeywordKind::Changeling));
    assert_eq!(t.pt(h), (4, 3));
}

#[test]
fn a_new_faceless_haven_creature_cant_attack_or_tap_for_mana() {
    cr!("302.6", "508.1a");
    ruling!(
        "Faceless Haven",
        "If Faceless Haven becomes a creature but hasn't been under your control since the beginning of your most recent turn, it can't attack or be tapped for mana."
    );
    supported("Faceless Haven");
    let mut t = TestGame::new(2);
    let h = t.battlefield_sick(P0, "Faceless Haven");
    // As a land, it can be tapped for mana the turn it came under your control.
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, h, "Add {C}"));
    t.g.untap(h);
    t.g.players[0].mana_pool.empty();
    animate_haven(&mut t, h);
    assert!(!crate::r_s20_common::tap_for_mana(&mut t, P0, h, "Add {C}"));
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!crate::r_s02_common::can_attack(&mut t, h));
    // An older Faceless Haven can.
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Faceless Haven");
    animate_haven(&mut t, h);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(crate::r_s02_common::can_attack(&mut t, h));
}

// ---------------------------------------------------------------------------
// Lavabrink Venturer
// ---------------------------------------------------------------------------

/// Whether `p` could target `what` with the card `spell` (it's among the candidates).
fn could_target(t: &mut TestGame, p: PlayerId, spell: &str, what: ObjectId) -> bool {
    crate::r_s04_common::spell_targets(t, p, spell).contains(&o(t.g.current(what)))
}

#[test]
fn lavabrink_venturer_with_no_choice_has_no_protection() {
    cr!("702.16b");
    ruling!(
        "Lavabrink Venturer",
        "If Lavabrink Venturer somehow has no choice made for its ability, its last ability grants it no protection abilities."
    );
    supported("Lavabrink Venturer");
    // A creature that becomes a copy of it never made the choice.
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Lavabrink Venturer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s17_common::become_copy(&mut t, bears, v);
    assert_eq!(t.obj_now(bears).chars.name, "Lavabrink Venturer");
    // Lightning Bolt (mana value 1) and Searing Spear (mana value 2) can both target it.
    assert!(could_target(&mut t, P1, "Lightning Bolt", bears));
    assert!(could_target(&mut t, P1, "Searing Spear", bears));
}

#[test]
fn lavabrink_venturer_has_protection_as_soon_as_it_enters() {
    cr!("614.12", "702.16b");
    ruling!(
        "Lavabrink Venturer",
        "You choose odd or even for Lavabrink Venturer before it enters the battlefield. There's no point at which players can target it before it has gained the appropriate protection abilities."
    );
    supported("Lavabrink Venturer");
    for (choice, bolt_ok, spear_ok) in [("odd", false, true), ("even", true, false)] {
        let mut t = TestGame::new(2);
        t.lands(P0, "Plains", 3);
        let v = t.hand(P0, "Lavabrink Venturer");
        // When the choice is asked, the Venturer isn't on the battlefield yet.
        let seen = crate::r_s01_common::watch(
            &mut t,
            P0,
            |d| matches!(d, Decision::ChooseOption { .. }),
            |g| g.find_in_zone(Zone::Battlefield, "Lavabrink Venturer").len(),
        );
        crate::r_s35_common::pick_options(&mut t, P0, &[choice]);
        t.cast(P0, v).go();
        // Resolving the spell: the permanent's protection is there before any player
        // gets priority.
        t.g.resolve_top();
        assert_eq!(*seen.lock().unwrap(), vec![0], "{choice}");
        let v = t.g.current(v);
        assert!(t.on_battlefield(v));
        assert_eq!(could_target(&mut t, P1, "Lightning Bolt", v), bolt_ok, "{choice}");
        assert_eq!(could_target(&mut t, P1, "Searing Spear", v), spear_ok, "{choice}");
    }
}

// ---------------------------------------------------------------------------
// Baylen, the Haymaker
// ---------------------------------------------------------------------------

fn sick_tokens(t: &mut TestGame, n: usize) -> Vec<ObjectId> {
    (0..n)
        .map(|_| {
            let id = create_token(t, P0, "Soldier");
            t.g.objects[id.0 as usize].summoning_sick = true;
            id
        })
        .collect()
}

fn activated_uid(t: &TestGame, src: ObjectId, idx: usize) -> u64 {
    t.obj_now(src)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, AbilityKind::Activated(_)))
        .nth(idx)
        .map(|a| a.uid)
        .unwrap()
}

#[test]
fn baylens_first_ability_is_a_mana_ability() {
    cr!("605.1a", "605.3a");
    ruling!(
        "Baylen, the Haymaker",
        "Baylen's first ability is a mana ability. It doesn't use the stack and can't be responded to."
    );
    supported("Baylen, the Haymaker");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Baylen, the Haymaker");
    let toks = sick_tokens(&mut t, 2);
    let uid = activated_uid(&t, b, 0);
    assert!(matches!(
        &t.obj_now(b).chars.abilities.iter().find(|a| a.uid == uid).unwrap().kind,
        AbilityKind::Activated(x) if x.is_mana_ability
    ));
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, b, "Add one mana"));
    assert_eq!(t.stack_len(), 0);
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
    assert!(toks.iter().all(|x| tapped(&t, *x)));
}

#[test]
fn baylen_can_tap_tokens_that_entered_this_turn() {
    cr!("302.6", "602.5a");
    ruling!(
        "Baylen, the Haymaker",
        "You may tap any untapped tokens you control to activate Baylen's abilities, including creature tokens that entered this turn, even if they don't have haste."
    );
    supported("Baylen, the Haymaker");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Baylen, the Haymaker");
    // A noncreature token counts too.
    let toks = sick_tokens(&mut t, 3);
    let treasure = create_token(&mut t, P0, "Treasure");
    t.activate(P0, b, 2, &[]).unwrap();
    assert!(toks.iter().all(|x| tapped(&t, *x)));
    assert!(tapped(&t, treasure));
    t.resolve_all();
    assert_eq!(t.counters(b, counters::PLUS1), 3);
    assert!(has_kw(&t, b, KeywordKind::Trample));
    // Three summoning-sick creature tokens: draw a card.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Baylen, the Haymaker");
    sick_tokens(&mut t, 3);
    let hand = t.hand_size(P0);
    t.activate(P0, b, 1, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
}

// ---------------------------------------------------------------------------
// Dice
// ---------------------------------------------------------------------------

#[test]
fn rolling_several_dice_at_once_counts_as_several_rolls() {
    cr!("706.1");
    ruling!(
        "The Space Family Goblinson",
        "If you roll more than one die at a time, however, that does count as multiple die rolls."
    );
    supported("The Space Family Goblinson");
    supported("Circuits Act");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "The Space Family Goblinson");
    assert!(!has_kw(&t, g, KeywordKind::Trample));
    crate::r_s35_common::load_dice(&mut t, &[1, 2, 3]);
    cast_new(&mut t, P0, "Circuits Act", &[]);
    t.resolve_all();
    assert_eq!(crate::r_s35_common::die_rolls(&t).len(), 3);
    assert!(has_kw(&t, g, KeywordKind::Trample));
    assert_eq!(t.counters(g, counters::PLUS1), 3);
}

#[test]
fn rerolled_and_ignored_dice_dont_count_as_rolls() {
    cr!("706.6", "706.2b");
    ruling!(
        "The Space Family Goblinson",
        "Remember that if a die is rerolled, that counts only as a single roll. The original roll doesn’t count. Similarly, any die roll that’s ignored doesn’t count."
    );
    ruling!(
        "Resolute Veggiesaur",
        "Remember that if a die is rerolled, that counts only as a single roll. The original roll doesn’t count. Similarly, any die roll that’s ignored doesn’t count."
    );
    supported("The Space Family Goblinson");
    supported("Resolute Veggiesaur");
    supported("Clam-I-Am");
    supported("Pixie Guide");
    supported("Pair o' Dice Lost");
    // A reroll (Clam-I-Am rerolls a 3): two dice rolled, not three.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "The Space Family Goblinson");
    let v = t.battlefield(P0, "Resolute Veggiesaur");
    t.battlefield(P0, "Clam-I-Am");
    t.answer_yes(P0, true);
    crate::r_s35_common::load_dice(&mut t, &[3, 5, 4]);
    cast_new(&mut t, P0, "Pair o' Dice Lost", &[]);
    t.resolve_all();
    assert_eq!(t.counters(g, counters::PLUS1), 2);
    assert!(!has_kw(&t, g, KeywordKind::Trample));
    assert_eq!(t.counters(v, counters::PLUS1), 0);
    // A third die: now it's three.
    crate::r_s35_common::load_dice(&mut t, &[1, 2]);
    cast_new(&mut t, P0, "Pair o' Dice Lost", &[]);
    t.resolve_all();
    assert!(has_kw(&t, g, KeywordKind::Trample));
    assert_eq!(t.counters(v, counters::PLUS1), 1);
    // An ignored roll (Pixie Guide: roll one more and ignore the lowest): two dice
    // rolled, not three.
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "The Space Family Goblinson");
    let v = t.battlefield(P0, "Resolute Veggiesaur");
    t.battlefield(P0, "Pixie Guide");
    crate::r_s35_common::load_dice(&mut t, &[1, 5, 4]);
    cast_new(&mut t, P0, "Pair o' Dice Lost", &[]);
    t.resolve_all();
    assert_eq!(t.counters(g, counters::PLUS1), 2);
    assert!(!has_kw(&t, g, KeywordKind::Trample));
    assert_eq!(t.counters(v, counters::PLUS1), 0);
}

// ---------------------------------------------------------------------------
// Replacement of damage, triggers, sacrifice on resolution
// ---------------------------------------------------------------------------

#[test]
fn lichenthrope_damage_is_replaced_so_its_side_effects_dont_happen() {
    cr!("614.1a", "702.15b", "702.2b");
    ruling!(
        "Lichenthrope",
        "This card’s effect replaces damage with placement of -1/-1 counters, so side-effects of damage to it will not trigger."
    );
    supported("Lichenthrope");
    supported("Spiteful Shadows");
    // A lifelink and deathtouch source: no life is gained and Lichenthrope isn't
    // destroyed, since it was dealt no damage.
    let mut t = TestGame::new(2);
    let l = t.battlefield(P0, "Lichenthrope");
    let n = t.battlefield(P1, "Vampire Nighthawk");
    // Spiteful Shadows: "Enchanted creature has 'Whenever this creature is dealt damage,
    // it deals that much damage to its controller.'"
    crate::r_s06_common::attach_new(&mut t, P1, "Spiteful Shadows", l);
    damage(&mut t, n, 2, l);
    assert_eq!(t.stack_len(), 0, "a damage trigger triggered");
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(l, counters::MINUS1), 2);
    assert_eq!(t.obj_now(l).damage, 0);
    assert_eq!(t.pt(l), (3, 3));
    assert!(t.on_battlefield(l));
    assert_eq!(t.life(P1), 20);
    // No damage event happened, so nothing that triggers on damage being dealt can.
    let dealt = t
        .g
        .turn_events
        .iter()
        .filter(|e| matches!(e, mtg_engine::events::Event::Damage { .. }))
        .count();
    assert_eq!(dealt, 0);
}

#[test]
fn reluctant_dounguard_triggers_for_each_creature_entering_with_it() {
    cr!("603.6a");
    ruling!(
        "Reluctant Dounguard",
        "If Reluctant Dounguard enters at the same time as one or more other creatures you control, its last ability will trigger for each of those creatures (unless it somehow entered without -1/-1 counters on it.)"
    );
    supported("Reluctant Dounguard");
    let mut t = TestGame::new(2);
    let ids = crate::r_s24_common::enter_together(
        &mut t,
        &[
            (P0, "Reluctant Dounguard"),
            (P0, "Grizzly Bears"),
            (P0, "Grizzly Bears"),
        ],
    );
    let d = ids[0];
    assert_eq!(t.counters(d, counters::MINUS1), 2);
    t.resolve_all();
    assert_eq!(t.counters(d, counters::MINUS1), 0);
    assert_eq!(t.pt(d), (4, 4));
    // Without -1/-1 counters (Solemnity: counters can't be put on creatures), it doesn't
    // trigger.
    supported("Solemnity");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Solemnity");
    let ids = crate::r_s24_common::enter_together(
        &mut t,
        &[(P0, "Reluctant Dounguard"), (P0, "Grizzly Bears")],
    );
    assert_eq!(t.counters(ids[0], counters::MINUS1), 0);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn faunsbane_troll_exiles_the_creature_whenever_it_would_die_that_turn() {
    cr!("614.1a", "701.14a");
    ruling!(
        "Faunsbane Troll",
        "If Faunsbane Troll's activated ability resolves, its effect causes the creature to be exiled any time it would die that turn, not just if it would die as a result of damage dealt by Faunsbane Troll."
    );
    supported("Faunsbane Troll");
    let mut t = TestGame::new(2);
    let troll = t.enter(P0, "Faunsbane Troll");
    t.resolve_all();
    assert_eq!(t.pt(troll), (5, 5));
    let role = with_subtype(&t, P0, "Role")[0];
    let wurm = t.battlefield(P1, "Colossal Dreadmaw");
    mana(&mut t, P0, ManaType::C, 1);
    t.answer_choose(P0, &[o(role)]);
    t.activate(P0, troll, 0, &[o(wurm)]).unwrap();
    t.resolve_all();
    // The 4/4 Troll dealt 4 to the 6/6 Dreadmaw, which survived.
    assert!(t.on_battlefield(wurm));
    assert_eq!(t.obj_now(wurm).damage, 4);
    destroy(&mut t, wurm);
    assert!(t.in_exile("Colossal Dreadmaw"));
    assert!(!t.in_graveyard(P1, "Colossal Dreadmaw"));
}

#[test]
fn person_of_interest_creates_the_detective_even_if_it_left() {
    cr!("113.7a", "603.6a");
    ruling!(
        "Person of Interest",
        "If Person of Interest is no longer on the battlefield when its triggered ability resolves, you'll still create a Detective token."
    );
    supported("Person of Interest");
    let mut t = TestGame::new(2);
    let p = t.enter(P0, "Person of Interest");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    destroy(&mut t, p);
    t.resolve_all();
    let det = with_subtype(&t, P0, "Detective");
    assert_eq!(det.len(), 1);
    assert_eq!(t.pt(det[0]), (2, 2));
    // The token isn't suspected (only the Person would have been).
    assert!(!has_kw(&t, det[0], KeywordKind::Menace));
}

/// Dire Fleet Warmonger's trigger resolves with `others` other creatures; P0 answers the
/// sacrifice choice with `answer`.
fn warmonger(others: usize, answer: impl Fn(&[ObjectId]) -> Vec<Entity>) -> (TestGame, ObjectId, Vec<ObjectId>) {
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "Dire Fleet Warmonger");
    let os: Vec<ObjectId> = (0..others).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &answer(&os));
    t.advance_to(P0, Step::BeginningOfCombat);
    t.settle();
    (t, w, os)
}

#[test]
fn dire_fleet_warmonger_sacrifices_one_creature_on_resolution() {
    cr!("608.2d", "608.2c");
    ruling!(
        "Dire Fleet Warmonger",
        "While resolving Dire Fleet Warmonger's ability, you can't sacrifice multiple creatures to give it more than +2/+2 and trample."
    );
    ruling!(
        "Dire Fleet Warmonger",
        "You choose whether to sacrifice a creature and which to sacrifice as Dire Fleet Warmonger's ability resolves. No player can take action between the time you make that choice and the time it gets +2/+2 and has trample, if appropriate."
    );
    supported("Dire Fleet Warmonger");
    let (mut t, w, os) = warmonger(2, |os| os.iter().map(|x| o(*x)).collect());
    assert_eq!(t.stack_len(), 1);
    let from = t.asked().len();
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::YesNo { .. } | Decision::ChooseEntities { .. }),
        |g| g.stack.len(),
    );
    // Nothing was chosen when it triggered.
    assert!(t.asked()[..from]
        .iter()
        .all(|(_, d)| !matches!(d, Decision::ChooseEntities { .. })));
    t.g.resolve_top();
    // The choices were made during the resolution, and the +2/+2 and trample were
    // applied in the same resolution (no priority in between).
    assert!(!seen.lock().unwrap().is_empty());
    assert_eq!(t.pt(w), (5, 5));
    assert!(has_kw(&t, w, KeywordKind::Trample));
    t.settle();
    let gone = os.iter().filter(|x| !t.on_battlefield(**x)).count();
    assert_eq!(gone, 1, "sacrificed more than one creature");
    assert_eq!(t.pt(w), (5, 5));
}

#[test]
fn sami_gives_spells_affinity_for_artifacts() {
    cr!("702.41a", "601.2f");
    ruling!(
        "Sami, Wildcat Captain",
        "Affinity for artifacts means “This spell costs {1} less to cast for each artifact you control.”"
    );
    supported("Sami, Wildcat Captain");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sami, Wildcat Captain");
    t.battlefield(P0, "Bonesplitter");
    t.battlefield(P0, "Bonesplitter");
    // An opponent's artifact doesn't count.
    t.battlefield(P1, "Bonesplitter");
    // Hill Giant ({3}{R}) costs {1}{R}.
    mana(&mut t, P0, ManaType::R, 2);
    let g = t.hand(P0, "Hill Giant");
    t.cast(P0, g).go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    t.resolve_all();
    assert!(t.on_battlefield(t.g.current(g)));
    // With one mana fewer it can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sami, Wildcat Captain");
    t.battlefield(P0, "Bonesplitter");
    t.battlefield(P0, "Bonesplitter");
    mana(&mut t, P0, ManaType::R, 1);
    let g = t.hand(P0, "Hill Giant");
    assert!(t.cast(P0, g).try_go().is_err());
}
