//! Rulings batch P064 — combat, multiplayer and other rulings of cards that gain flying or
//! haste: staying blocked (CR 509.1h), attacking after losing haste (506.4), Alexios's
//! restrictions, multiplayer conditions (Two-Headed Giant 810), skipped draws (614.10),
//! additional land plays (305.2), casting from the graveyard (601.3), "can't be
//! countered" (101.2), regeneration (701.19) and the legend rule (704.5j).

use crate::r_p064_common::*;
use crate::r_p076_common::unblockable_after_blocked;
use crate::r_s01_common::supported;
use crate::r_s09_common::legal_attack;
use mtg_engine::mana::ManaType;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------------------
// Combat
// ---------------------------------------------------------------------------------------

#[test]
fn gaining_flying_after_being_blocked_doesnt_unblock() {
    cr!("509.1h", "506.4");
    ruling!(
        "Kargan Dragonrider",
        "Once Kargan Dragonrider has been blocked, causing it to gain flying by putting a Dragon on the battlefield won't cause it to become unblocked."
    );
    ruling!(
        "Obstinate Gargoyle",
        "Once Obstinate Gargoyle has been blocked, causing it to gain flying by modifying it won't cause it to stop being blocked."
    );
    supported("Kargan Dragonrider");
    supported("Obstinate Gargoyle");
    let mut t = TestGame::new(2);
    let k = t.battlefield(P0, "Kargan Dragonrider");
    let b = t.battlefield(P1, "Hill Giant");
    unblockable_after_blocked(&mut t, k, b, |t| {
        t.battlefield(P0, "Shivan Dragon");
        assert!(flies(t, k));
    });
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Obstinate Gargoyle");
    let b = t.battlefield(P1, "Hill Giant");
    unblockable_after_blocked(&mut t, g, b, |t| {
        counters(t, g, "+1/+1", 1);
        assert!(flies(t, g));
    });
}

#[test]
fn lightning_mauler_partner_losing_haste_keeps_attacking() {
    cr!("506.4", "702.95c", "302.6");
    ruling!(
        "Lightning Mauler",
        "A creature that loses haste after it’s been declared as an attacking creature doesn’t stop attacking."
    );
    supported("Lightning Mauler");
    let mut t = TestGame::new(2);
    let bears = t.battlefield_sick(P0, "Grizzly Bears");
    let m = t.enter(P0, "Lightning Mauler");
    t.settle();
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve();
    assert!(has(&mut t, bears, mtg_engine::keywords::KeywordKind::Haste));
    crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    crate::r_s02_common::destroy(&mut t, m);
    assert!(!has(&mut t, bears, mtg_engine::keywords::KeywordKind::Haste));
    assert!(crate::r_s10_common::attacking(&t, bears));
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn alexios_can_attack_planeswalkers_its_owner_controls() {
    cr!("506.3", "508.1b");
    ruling!(
        "Alexios, Deimos of Kosmos",
        "Alexios can attack planeswalkers its owner controls and battles its owner protects."
    );
    supported("Alexios, Deimos of Kosmos");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Alexios, Deimos of Kosmos");
    let jace = t.battlefield(P0, "Jace Beleren");
    // P1's upkeep: P1 gains control of Alexios, and it gains haste.
    crate::r_s04_common::next_upkeep(&mut t, P1);
    t.resolve_all();
    assert_eq!(t.obj_now(a).controller, P1);
    crate::r_s20_common::to_beginning_of_combat(&mut t, P1);
    assert!(legal_attack(&mut t, &[(a, obj(jace))]));
    assert!(!legal_attack(&mut t, &[(a, Entity::Player(P0))]), "not its owner");
}

#[test]
fn alexios_cant_be_sacrificed() {
    cr!("701.21a", "101.3");
    ruling!(
        "Alexios, Deimos of Kosmos",
        "Alexios can't be sacrificed for any reason."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Alexios, Deimos of Kosmos");
    t.set_step(P1, Step::PrecombatMain);
    crate::r_s29_common::cast_and_resolve(&mut t, P1, "Diabolic Edict", &[Entity::Player(P0)]);
    assert!(t.on_battlefield(a));
    // Nor sacrificed to pay a cost.
    t.set_step(P0, Step::PrecombatMain);
    let f = t.battlefield(P0, "Falkenrath Torturer");
    let from = t.asked().len();
    crate::r_s06_common::activate_containing(&mut t, P0, f, "Sacrifice a creature").unwrap();
    for cands in crate::r_s03_common::choice_candidates(&t, from, "") {
        assert!(!cands.contains(&obj(a)), "Alexios was offered as the sacrifice");
    }
    assert!(t.on_battlefield(a));
}

// ---------------------------------------------------------------------------------------
// Multiplayer
// ---------------------------------------------------------------------------------------

#[test]
fn escaped_shapeshifter_looks_at_every_opponent() {
    cr!("611.3a", "613.1f");
    ruling!(
        "Escaped Shapeshifter",
        "In a multiplayer game, it works if any opponent controls creatures with the appropriate abilities."
    );
    supported("Escaped Shapeshifter");
    let mut t = TestGame::new(3);
    let s = t.battlefield(P0, "Escaped Shapeshifter");
    assert!(!flies(&mut t, s));
    t.battlefield(P2, "Serra Angel");
    assert!(flies(&mut t, s));
    assert!(!has(&mut t, s, mtg_engine::keywords::KeywordKind::FirstStrike));
}

#[test]
fn blood_baron_in_two_headed_giant_uses_team_life_totals() {
    cr!("810.9", "810.9a");
    ruling!(
        "Blood Baron of Vizkopa",
        "In a Two-Headed Giant game, Blood Baron of Vizkopa gets +6/+6 and has flying as long as your team has 30 or more life and an opposing team has 10 or less life."
    );
    supported("Blood Baron of Vizkopa");
    let mut t = crate::r_s33_common::two_headed_giant();
    let b = t.battlefield(P0, "Blood Baron of Vizkopa");
    assert_eq!(t.life(P0), 30);
    t.g.lose_life(P2, 19);
    t.settle();
    assert!(!flies(&mut t, b), "11 life");
    t.g.lose_life(P3, 1);
    t.settle();
    assert_eq!(t.life(P2), 10);
    assert!(flies(&mut t, b), "the opposing team has 10");
    assert_eq!(t.pt(b), (10, 10));
}

// ---------------------------------------------------------------------------------------
// Other
// ---------------------------------------------------------------------------------------

#[test]
fn living_conundrum_draws_what_it_can_then_skips() {
    cr!("121.4", "614.10");
    ruling!(
        "Living Conundrum",
        "If you're instructed to draw more than one card and you have fewer cards in your library, you draw each card in your library, then skip the remaining draws."
    );
    supported("Living Conundrum");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Living Conundrum");
    crate::r_s11_common::empty_library(&mut t, P0);
    t.library_top(P0, "Forest");
    t.library_top(P0, "Forest");
    let hand = t.hand_size(P0);
    t.g.draw_cards(P0, 3);
    t.g.flush_events();
    t.settle();
    assert_eq!(t.hand_size(P0), hand + 2);
    assert!(!t.has_lost(P0), "the third draw was skipped");
    assert_eq!(t.pt(c), (10, 10));
    assert!(flies(&mut t, c));
}

#[test]
fn hearthhull_extra_land_play_adds_to_others() {
    cr!("305.2", "305.2a");
    ruling!(
        "Hearthhull, the Worldseed",
        "The effect of Hearthhull’s second ability is cumulative with similar effects."
    );
    supported("Hearthhull, the Worldseed");
    supported("Icetill Explorer");
    let mut t = TestGame::new(2);
    let h = t.battlefield(P0, "Hearthhull, the Worldseed");
    counters(&mut t, h, "charge", 2);
    t.battlefield(P0, "Icetill Explorer");
    let sac = t.battlefield(P0, "Swamp");
    pool(&mut t, P0, ManaType::C, 1);
    t.answer_choose(P0, &[obj(sac)]);
    activate_and_resolve(&mut t, P0, h, "Draw two");
    let lands: Vec<ObjectId> = (0..4).map(|_| t.hand(P0, "Forest")).collect();
    for l in &lands[..3] {
        t.play_land(P0, *l).expect("a land play available");
    }
    assert!(t.play_land(P0, lands[3]).is_err(), "only three land plays");
}

#[test]
fn flutterfox_counts_your_aura_on_an_opponents_creature() {
    cr!("303.4e", "110.2");
    ruling!(
        "Flutterfox",
        "You control Auras that you put onto the battlefield attached to a permanent you don't control."
    );
    supported("Flutterfox");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Flutterfox");
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(!flies(&mut t, f));
    crate::r_s06_common::attach_new(&mut t, P0, "Pacifism", bears);
    assert!(flies(&mut t, f));
}

#[test]
fn croakid_amphibonaut_counts_itself_or_another_stickered_permanent() {
    cr!("123.3", "123.9");
    ruling!(
        "Croakid Amphibonaut",
        "The stickered permanent could be Croakid Amphibonaut itself or another permanent you control."
    );
    supported("Croakid Amphibonaut");
    use mtg_engine::stickers::{put_sticker, StickerKind};
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Croakid Amphibonaut");
    assert!(!flies(&mut t, c));
    assert!(put_sticker(&mut t.g, P0, c, StickerKind::Art));
    assert!(flies(&mut t, c), "itself");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Croakid Amphibonaut");
    let b = t.battlefield(P0, "Grizzly Bears");
    assert!(put_sticker(&mut t.g, P0, b, StickerKind::Art));
    assert!(flies(&mut t, c), "another permanent");
}

#[test]
fn exploration_broodship_graveyard_casts_follow_timing_rules() {
    cr!("601.3", "117.1a");
    ruling!(
        "Exploration Broodship",
        "You pay all costs and follow all timing rules for cards cast with the permission granted by Exploration Broodship’s last ability."
    );
    supported("Exploration Broodship");
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Exploration Broodship");
    // Its last ability is part of its 8+ station section.
    counters(&mut t, b, "charge", 8);
    t.lands(P0, "Forest", 3);
    let bears = t.graveyard(P0, "Grizzly Bears");
    assert!(crate::r_s21_common::castable(&mut t, P0, bears));
    t.set_step(P0, Step::Upkeep);
    assert!(
        !crate::r_s21_common::castable(&mut t, P0, bears),
        "a creature spell only in a main phase"
    );
}

#[test]
fn shifting_ceratops_can_be_targeted_by_counterspells_whose_other_effects_happen() {
    cr!("101.2", "608.2b");
    ruling!(
        "Shifting Ceratops",
        "A spell or ability that counters spells can still target Shifting Ceratops."
    );
    supported("Shifting Ceratops");
    let mut t = TestGame::new(2);
    let c = crate::r_s25_common::cast_new(&mut t, P0, "Shifting Ceratops", &[]);
    crate::r_s25_common::cast_new(&mut t, P1, "Dissolve", &[obj(c)]);
    let from = t.asked().len();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Shifting Ceratops").len(), 1);
    assert!(
        t.asked()[from..]
            .iter()
            .any(|(p, d)| *p == P1 && matches!(d, mtg_engine::decision::Decision::Scry { .. })),
        "Dissolve's scry still happened"
    );
}

#[test]
fn skithiryx_regeneration_shield_stops_destruction_and_lethal_damage() {
    cr!("701.19a", "701.19b", "614.8");
    ruling!(
        "Skithiryx, the Blight Dragon",
        "Activating the regeneration ability creates a replacement effect that acts like a shield"
    );
    supported("Skithiryx, the Blight Dragon");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Skithiryx, the Blight Dragon");
    pool(&mut t, P0, ManaType::B, 4);
    activate_and_resolve(&mut t, P0, s, "Regenerate");
    t.set_step(P1, Step::PrecombatMain);
    crate::r_s29_common::cast_and_resolve(&mut t, P1, "Murder", &[obj(s)]);
    assert!(t.on_battlefield(s), "regenerated");
    assert!(t.obj_now(s).tapped);
    t.set_step(P0, Step::PrecombatMain);
    activate_and_resolve(&mut t, P0, s, "Regenerate");
    let giant = t.battlefield(P1, "Hill Giant");
    crate::r_s06_common::damage(&mut t, giant, 5, s);
    assert!(t.on_battlefield(s), "regenerated from lethal damage");
    assert_eq!(t.obj_now(s).damage, 0);
}

#[test]
fn zurgo_bellstriker_and_helmsmasher_have_different_names() {
    cr!("704.5j", "201.2");
    ruling!(
        "Zurgo Bellstriker",
        "Controlling both of them won't invoke the \"legend rule.\""
    );
    supported("Zurgo Bellstriker");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Zurgo Bellstriker");
    let b = t.battlefield(P0, "Zurgo Helmsmasher");
    t.settle();
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
}

#[test]
fn soulflayer_delve_exiles_at_most_its_generic_requirement() {
    cr!("702.66a");
    ruling!(
        "Soulflayer",
        "You can’t exile more cards from your graveyard than you’d need to pay Soulflayer’s generic mana requirement."
    );
    // Soulflayer's keyword-granting ability isn't compiled; its delve (the subject of this
    // ruling) is.
    let mut t = TestGame::new(2);
    let cards: Vec<ObjectId> = (0..6).map(|_| t.graveyard(P0, "Wind Drake")).collect();
    t.lands(P0, "Swamp", 2);
    let s = t.hand(P0, "Soulflayer");
    let all: Vec<Entity> = cards.iter().map(|c| obj(*c)).collect();
    t.answer_choose(P0, &all);
    let from = t.asked().len();
    t.cast(P0, s).go();
    let maxes: Vec<usize> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            mtg_engine::decision::Decision::ChooseEntities { prompt, max, .. }
                if prompt.contains("delve") =>
            {
                Some(*max as usize)
            }
            _ => None,
        })
        .collect();
    assert_eq!(maxes, vec![4], "at most four cards for its {{4}}");
    assert!(t.graveyard_size(P0) >= 2, "two cards stayed in the graveyard");
}
