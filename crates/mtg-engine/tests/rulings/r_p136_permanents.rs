//! Rulings batch P136 (removal) — removal on permanents: hybrid mana (CR 107.4e, 202.2d,
//! 202.3f), copies entering (CR 707.2), phasing (CR 702.26d), attack triggers that exile
//! (CR 508.1m, 506.4), delayed triggers (CR 603.7), type-changing Auras (CR 205.1a,
//! 704.5m, 704.5n), effects performed in order (CR 608.2), costs that exile their source
//! (CR 603.2, 405.5), losing abilities (CR 613.1f), mana spent on a spell (CR 107.4h,
//! 707.10) and linked "until" exile (CR 610.3c).

use crate::r_p108_common::end_step;
use crate::r_p136_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, destroy};
use crate::r_s04_common::add_mana;
use crate::r_s06_common::attached_to;
use crate::r_s21_common::legal_blocks;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

// ---------------------------------------------------------------------------
// Reaper King
// ---------------------------------------------------------------------------

/// P0 casts Reaper King paying with exactly the mana `pool` (type, amount).
fn cast_reaper_king(pool: &[(ManaType, u32)]) -> (TestGame, Result<ObjectId, String>) {
    let mut t = TestGame::new(2);
    for (ty, n) in pool {
        add_mana(&mut t, P0, *ty, *n);
    }
    let rk = t.hand(P0, "Reaper King");
    t.g.turn.priority = Some(P0);
    let r = t
        .g
        .cast_spell(P0, rk, mtg_engine::object::CastMethod::Normal)
        .map_err(|e| format!("{e:?}"));
    t.g.flush_events();
    (t, r)
}

#[test]
fn reaper_king_is_every_color_and_mana_value_ten_however_it_was_paid() {
    cr!("105.2", "202.2d", "202.3f", "107.4e");
    ruling!(
        "Reaper King",
        "Thus, Reaper King is all colors even if you spend ten colorless mana to cast it."
    );
    ruling!(
        "Reaper King",
        "Thus, Reaper King has a mana value of 10, even if you spend {W}{U}{B}{R}{G} to cast it."
    );
    supported("Reaper King");
    let (mut t, spell) = cast_reaper_king(&[(ManaType::C, 10)]);
    let spell = spell.unwrap();
    assert_eq!(t.g.obj(spell).chars.colors, ColorSet::ALL);
    t.resolve_all();
    let rk = t.named_on_battlefield("Reaper King")[0];
    assert_eq!(t.g.obj(rk).chars.colors, ColorSet::ALL);
    let (t, spell) = cast_reaper_king(&[
        (ManaType::W, 1),
        (ManaType::U, 1),
        (ManaType::B, 1),
        (ManaType::R, 1),
        (ManaType::G, 1),
    ]);
    assert_eq!(t.g.mana_value_of(spell.unwrap()), 10);
}

#[test]
fn reaper_king_each_hybrid_symbol_is_paid_its_own_way() {
    cr!("107.4e", "601.2h");
    ruling!(
        "Reaper King",
        "If a cost includes more than one monocolored hybrid mana symbol, you can choose a different way to pay for each symbol."
    );
    // {2} and four colors; {6} and two colors.
    for pool in [
        vec![
            (ManaType::C, 2),
            (ManaType::W, 1),
            (ManaType::U, 1),
            (ManaType::B, 1),
            (ManaType::R, 1),
        ],
        vec![(ManaType::C, 6), (ManaType::G, 1), (ManaType::U, 1)],
    ] {
        let (t, spell) = cast_reaper_king(&pool);
        assert!(spell.is_ok(), "{pool:?}");
        assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    }
    // Nine mana including one color isn't enough ({8} and one color is).
    let (_, spell) = cast_reaper_king(&[(ManaType::C, 8)]);
    assert!(spell.is_err());
    let (_, spell) = cast_reaper_king(&[(ManaType::C, 8), (ManaType::G, 1)]);
    assert!(spell.is_ok());
}

// ---------------------------------------------------------------------------
// Eye of Singularity
// ---------------------------------------------------------------------------

#[test]
fn eye_of_singularity_a_copy_entering_destroys_what_it_copies() {
    cr!("707.2", "603.6a");
    ruling!(
        "Eye of Singularity",
        "If a permanent enters as a copy of another permanent, the permanent it is copying is destroyed and the copy stays on the battlefield."
    );
    supported("Eye of Singularity");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eye of Singularity");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[o(bears)]);
    t.answer_yes(P0, true);
    let clone = t.enter(P0, "Clone");
    t.resolve_all();
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Grizzly Bears");
    assert!(t.on_battlefield(clone));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
}

#[test]
fn eye_of_singularity_phasing_in_doesnt_trigger_it() {
    cr!("702.26d");
    ruling!(
        "Eye of Singularity",
        "Since phasing in does not cause enters abilities to trigger"
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Eye of Singularity");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    mtg_engine::kw::phasing::phase_out(&mut t.g, vec![b]);
    t.settle();
    mtg_engine::kw::phasing::phase_in(&mut t.g, b);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_on_stack(&t, "destroy all other permanents"), 0);
    t.resolve_all();
    assert!(t.on_battlefield(a) && t.on_battlefield(b));
    // Control: one entering does trigger.
    t.enter(P0, "Grizzly Bears");
    t.settle();
    assert_eq!(triggers_on_stack(&t, "destroy all other permanents"), 1);
}

// ---------------------------------------------------------------------------
// Bane of Bala Ged
// ---------------------------------------------------------------------------

#[test]
fn bane_of_bala_ged_exiles_before_blocks_and_keeps_attacking() {
    cr!("508.1m", "506.4", "509.1");
    ruling!(
        "Bane of Bala Ged",
        "If Bane of Bala Ged attacks a planeswalker, the defending player may exile that planeswalker. Bane of Bala Ged will still be attacking and may be blocked as normal. If it’s not blocked, it won’t deal combat damage."
    );
    ruling!("Bane of Bala Ged", "The permanents are exiled before blockers are chosen.");
    supported("Bane of Bala Ged");
    let mut t = TestGame::new(2);
    let bane = t.battlefield(P0, "Bane of Bala Ged");
    let walker = t.battlefield(P1, "Liliana, Dreadhorde General");
    t.g.objects[walker.0 as usize]
        .counters
        .insert("loyalty".into(), 6);
    let blocker = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P1, &[o(walker), o(blocker)]);
    attack_with(&mut t, &[(bane, o(walker))]);
    t.resolve_all();
    assert!(t.in_exile("Liliana, Dreadhorde General") && t.in_exile("Hill Giant"));
    // Still attacking; the exiled Giant can't block.
    assert!(t.g.is_attacking(bane));
    assert!(legal_blocks(&mut t, P1, &[(other, bane)]));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20);
    assert!(t.on_battlefield(other));
}

#[test]
fn bane_of_bala_ged_exiles_the_only_permanent() {
    cr!("508.1m", "608.2d");
    ruling!(
        "Bane of Bala Ged",
        "If the defending player controls only one permanent, that permanent will be exiled."
    );
    let mut t = TestGame::new(2);
    let bane = t.battlefield(P0, "Bane of Bala Ged");
    t.battlefield(P1, "Forest");
    attack_with(&mut t, &[(bane, Entity::Player(P1))]);
    t.resolve_all();
    assert!(t.in_exile("Forest"));
    assert_eq!(t.g.permanents().filter(|p| p.controller == P1).count(), 0);
}

// ---------------------------------------------------------------------------
// Bearer of the Heavens
// ---------------------------------------------------------------------------

#[test]
fn bearer_of_the_heavens_dying_in_an_end_step_waits_for_the_next_one() {
    cr!("603.7a", "513.1");
    ruling!(
        "Bearer of the Heavens",
        "If Bearer of the Heavens dies during an end step, the delayed triggered ability will trigger at the beginning of the next turn's end step."
    );
    supported("Bearer of the Heavens");
    let mut t = TestGame::new(2);
    let bearer = t.battlefield(P0, "Bearer of the Heavens");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.advance_to(P0, Step::End);
    destroy(&mut t, bearer);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    t.advance_to(P1, Step::Upkeep);
    assert!(t.on_battlefield(bears));
    end_step(&mut t, P1);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn bearer_of_the_heavens_destroys_everything_there_as_it_resolves() {
    cr!("603.7c", "608.2h");
    ruling!(
        "Bearer of the Heavens",
        "The effect of the ability will destroy all permanents on the battlefield as it resolves. It doesn't matter if those permanents were on the battlefield when Bearer of the Heavens died."
    );
    let mut t = TestGame::new(2);
    let bearer = t.battlefield(P0, "Bearer of the Heavens");
    destroy(&mut t, bearer);
    t.resolve_all();
    let later = t.battlefield(P1, "Grizzly Bears");
    let land = t.battlefield(P0, "Forest");
    end_step(&mut t, P0);
    assert!(!t.on_battlefield(later) && !t.on_battlefield(land));
}

// ---------------------------------------------------------------------------
// Song of the Dryads
// ---------------------------------------------------------------------------

#[test]
fn song_of_the_dryads_on_an_aura_or_equipment_unattaches_it() {
    cr!("205.1a", "704.5m", "704.5n", "301.5c");
    ruling!(
        "Song of the Dryads",
        "If the permanent was an Aura or an Equipment, it becomes unattached from whatever it was attached to."
    );
    supported("Song of the Dryads");
    for attachment in ["Bonesplitter", "Holy Strength"] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P1, "Grizzly Bears");
        let a = t.battlefield(P1, attachment);
        assert!(t.g.attach(a, o(bears)));
        cast(&mut t, P0, "Song of the Dryads", &[&[o(a)]]);
        t.resolve_all();
        assert!(t.on_battlefield(a), "{attachment}");
        assert!(t.obj_now(a).chars.has_subtype("Forest"));
        assert_eq!(attached_to(&t, a), None, "{attachment}");
        assert_eq!(t.pt(bears), (2, 2));
    }
}

// ---------------------------------------------------------------------------
// Spine of Ish Sah
// ---------------------------------------------------------------------------

#[test]
fn spine_of_ish_sah_offers_no_way_to_sacrifice_itself() {
    cr!("603.6c");
    ruling!(
        "Spine of Ish Sah",
        "Spine of Ish Sah's last ability doesn't allow you to sacrifice it."
    );
    supported("Spine of Ish Sah");
    let mut t = TestGame::new(2);
    t.lands(P0, "Wastes", 8);
    let spine = t.battlefield(P0, "Spine of Ish Sah");
    assert!(!can_activate(&mut t, P0, spine));
    // Put into a graveyard some other way, it returns to its owner's hand.
    destroy(&mut t, spine);
    t.resolve_all();
    assert!(t.in_hand(P0, "Spine of Ish Sah"));
}

// ---------------------------------------------------------------------------
// Worldfire
// ---------------------------------------------------------------------------

#[test]
fn worldfire_in_order_life_loss_and_triggers_after() {
    cr!("608.2", "119.5", "603.3");
    ruling!(
        "Worldfire",
        "If your life total is greater than 1 when Worldfire resolves, the change in life total will count as life loss."
    );
    ruling!(
        "Worldfire",
        "These actions are performed in order, but no triggered abilities resolve in between them"
    );
    ruling!(
        "Worldfire",
        "Worldfire won’t be exiled. It will go to your graveyard as the final part of its resolution."
    );
    supported("Worldfire");
    let mut t = TestGame::new(2);
    // Exquisite Blood would gain P0 life as P1 loses life, but it's exiled first.
    t.battlefield(P0, "Exquisite Blood");
    // Leaves-the-battlefield triggers wait until Worldfire has finished.
    supported("Aven Riftwatcher");
    t.battlefield(P1, "Aven Riftwatcher");
    t.hand(P1, "Lightning Bolt");
    t.graveyard(P1, "Shock");
    let wf = cast(&mut t, P0, "Worldfire", &[]);
    t.g.resolve_top();
    t.g.flush_events();
    let lost: Vec<(PlayerId, u32)> = t
        .g
        .turn_events
        .iter()
        .filter_map(|e| match e {
            Event::LifeLost { player, amount } => Some((*player, *amount)),
            _ => None,
        })
        .collect();
    assert!(lost.contains(&(P1, 19)) && lost.contains(&(P0, 19)));
    assert_eq!((t.life(P0), t.life(P1)), (1, 1));
    assert!(t.g.permanents().next().is_none());
    assert_eq!(t.hand_size(P1), 0);
    assert_eq!(t.zone(wf), Zone::Graveyard(P0));
    assert_eq!(t.graveyard_size(P0), 1);
    assert_eq!(t.graveyard_size(P1), 0);
    t.settle();
    assert_eq!(triggers_on_stack(&t, "gain 2 life"), 1);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (1, 3));
}

// ---------------------------------------------------------------------------
// Urza's Sylex
// ---------------------------------------------------------------------------

#[test]
fn urzas_sylex_trigger_resolves_before_its_activated_ability() {
    cr!("603.3", "405.5", "602.2");
    ruling!(
        "Urza's Sylex",
        "The triggered ability of Urza's Sylex will resolve before the activated ability."
    );
    supported("Urza's Sylex");
    let mut t = TestGame::new(2);
    let sylex = t.battlefield(P0, "Urza's Sylex");
    t.lands(P0, "Plains", 6);
    let walker = t.library_top(P0, "Liliana, Dreadhorde General");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P1, "Forest", 6);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(walker)]);
    // When P1 chooses its lands, P0 already has the planeswalker card.
    let seen = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::ChooseEntities { .. }),
        |g| {
            g.player(PlayerId(0))
                .hand
                .iter()
                .any(|c| g.obj(*c).chars.name.as_str() == "Liliana, Dreadhorde General")
        },
    );
    t.activate(P0, sylex, 0, &[]).unwrap();
    t.settle();
    assert_eq!(t.g.stack.len(), 2);
    assert_eq!(triggers_on_stack(&t, "planeswalker card"), 1);
    t.resolve_all();
    assert_eq!(t.zone(walker), Zone::Hand(P0));
    assert_eq!(seen.lock().unwrap().first().copied(), Some(true));
    assert!(!t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------
// Tamiyo's Compleation
// ---------------------------------------------------------------------------

#[test]
fn tamiyos_compleation_planeswalker_can_still_be_attacked() {
    cr!("506.3", "120.3c");
    ruling!(
        "Tamiyo's Compleation",
        "A planeswalker with Tamiyo's Compleation attached to it may still be attacked"
    );
    supported("Tamiyo's Compleation");
    let mut t = TestGame::new(2);
    let walker = t.battlefield(P1, "Liliana, Dreadhorde General");
    t.g.objects[walker.0 as usize]
        .counters
        .insert("loyalty".into(), 6);
    cast(&mut t, P0, "Tamiyo's Compleation", &[&[o(walker)]]);
    t.resolve_all();
    let tc = t.named_on_battlefield("Tamiyo's Compleation")[0];
    assert_eq!(attached_to(&t, tc), Some(o(walker)));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.attack(&[(bears, o(walker))], &[]);
    assert_eq!(t.counters(walker, "loyalty"), 4);
}

/// Lizard Blades (Equipment creature, reconfigure) attached to P1's Bears, and P0's
/// Tamiyo's Compleation cast on it, its trigger on the stack.
fn compleation_on_attached_blades() -> (TestGame, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let blades = t.battlefield(P1, "Lizard Blades");
    assert!(t.g.attach(blades, o(bears)));
    t.g.recompute();
    assert!(!t.obj_now(blades).is(CardType::Creature));
    cast(&mut t, P0, "Tamiyo's Compleation", &[&[o(blades)]]);
    t.resolve();
    (t, bears, blades)
}

#[test]
fn tamiyos_compleation_attached_reconfigure_equipment_stays_attached_until_the_trigger() {
    cr!("702.151b", "613.1d", "613.1f");
    ruling!(
        "Tamiyo's Compleation",
        "An attached Equipment with reconfigure that loses its reconfigure ability doesn't become a creature again until it becomes unattached"
    );
    supported("Lizard Blades");
    supported("Stifle");
    let (mut t, bears, blades) = compleation_on_attached_blades();
    assert_eq!(triggers_on_stack(&t, "unattach"), 1);
    t.g.recompute();
    assert!(!t.obj_now(blades).is(CardType::Creature));
    assert_eq!(attached_to(&t, blades), Some(o(bears)));
    // The trigger resolves: unattached, it's a creature again.
    t.resolve_all();
    assert_eq!(attached_to(&t, blades), None);
    assert!(t.obj_now(blades).is(CardType::Creature));
    // If the trigger is countered, it stays attached.
    let (mut t, bears, blades) = compleation_on_attached_blades();
    let trigger = *t.g.stack.last().unwrap();
    cast(&mut t, P1, "Stifle", &[&[o(trigger)]]);
    t.resolve_all();
    assert_eq!(attached_to(&t, blades), Some(o(bears)));
    assert!(!t.obj_now(blades).is(CardType::Creature));
}

#[test]
fn tamiyos_compleation_equipment_creature_without_reconfigure_cant_be_attached() {
    cr!("301.5c");
    ruling!(
        "Tamiyo's Compleation",
        "An Equipment creature that loses its reconfigure ability cannot be attached to a creature by any effect."
    );
    let mut t = TestGame::new(2);
    let blades = t.battlefield(P1, "Lizard Blades");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast(&mut t, P0, "Tamiyo's Compleation", &[&[o(blades)]]);
    t.resolve_all();
    assert!(t.obj_now(blades).is(CardType::Creature));
    assert!(!t.g.attach(blades, o(bears)));
    assert_eq!(attached_to(&t, blades), None);
}

// ---------------------------------------------------------------------------
// Blood on the Snow
// ---------------------------------------------------------------------------

/// P0 casts Blood on the Snow (destroy all creatures) paying with `snow` snow and the rest
/// non-snow mana.
fn blood_on_the_snow(t: &mut TestGame, snow: usize) -> ObjectId {
    t.lands(P0, "Snow-Covered Swamp", snow);
    t.lands(P0, "Swamp", 6 - snow);
    let c = t.hand(P0, "Blood on the Snow");
    t.cast(P0, c).modes(&[0]).go()
}

#[test]
fn blood_on_the_snow_returns_a_card_chosen_on_resolution() {
    cr!("608.2c", "107.4h");
    ruling!(
        "Blood on the Snow",
        "You choose which one to return to the battlefield, if any, as Blood on the Snow resolves. Notably, you could return one of the creatures or planeswalkers that was just destroyed"
    );
    supported("Blood on the Snow");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    blood_on_the_snow(&mut t, 6);
    t.resolve_all();
    // The Giant was destroyed and returned (as a new object).
    assert!(!t.g.is_live(giant));
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn blood_on_the_snow_without_snow_mana_returns_only_mana_value_zero() {
    cr!("107.4h", "202.3");
    ruling!(
        "Blood on the Snow",
        "If you cast Blood on the Snow without spending snow mana, you can return only a creature or planeswalker card with mana value 0."
    );
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Ornithopter");
    blood_on_the_snow(&mut t, 0);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
}

#[test]
fn blood_on_the_snow_copy_had_no_mana_spent() {
    cr!("707.10", "107.4h");
    ruling!(
        "Blood on the Snow",
        "If Blood on the Snow is copied, no mana was spent to cast the copy, so you can return only a creature or planeswalker card with mana value 0."
    );
    supported("Twincast");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    t.graveyard(P0, "Ornithopter");
    let spell = blood_on_the_snow(&mut t, 6);
    t.lands(P0, "Island", 2);
    let tw = t.hand(P0, "Twincast");
    t.cast_with(P0, tw, &[o(spell)]).unwrap();
    // Twincast resolves; its copy of Blood on the Snow resolves next.
    t.resolve();
    t.resolve();
    assert_eq!(t.g.stack, vec![spell]);
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
    assert!(t.named_on_battlefield("Hill Giant").is_empty());
    // The original (six snow mana spent) can return the Giant.
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

// ---------------------------------------------------------------------------
// Citizen's Arrest and Ossification
// ---------------------------------------------------------------------------

#[test]
fn until_leaves_exile_never_happens_if_the_enchantment_left_first() {
    cr!("610.3c");
    ruling!(
        "Citizen's Arrest",
        "If Citizen’s Arrest leaves the battlefield before its triggered ability resolves, the target permanent won’t be exiled."
    );
    ruling!(
        "Ossification",
        "If Ossification leaves the battlefield before its triggered ability resolves, the target permanent won't be exiled."
    );
    supported("Citizen's Arrest");
    supported("Ossification");
    // Citizen's Arrest.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[o(bears)]);
    let ca = t.enter(P0, "Citizen's Arrest");
    t.settle();
    assert_eq!(t.g.stack.len(), 1);
    destroy(&mut t, ca);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // Ossification (enchanting a basic land P0 controls).
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let plains = t.battlefield(P0, "Plains");
    t.answer_targets(P0, &[o(bears)]);
    cast(&mut t, P0, "Ossification", &[&[o(plains)]]);
    t.resolve();
    let oss = t.named_on_battlefield("Ossification")[0];
    assert_eq!(attached_to(&t, oss), Some(o(plains)));
    assert_eq!(t.g.stack.len(), 1);
    destroy(&mut t, oss);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

// ---------------------------------------------------------------------------
// The Most Dangerous Gamer
// ---------------------------------------------------------------------------

#[test]
fn most_dangerous_gamer_only_prize_attractions_trigger_it() {
    cr!("702.159b");
    ruling!(
        "The Most Dangerous Gamer",
        "Only certain Attractions allow you to claim a prize."
    );
    supported("The Most Dangerous Gamer");
    let mut t = TestGame::new(2);
    // Information Booth (2 and 6 lit up): "Visit — Draw a card." No prize.
    t.battlefield(P0, "Information Booth");
    t.battlefield(P0, "The Most Dangerous Gamer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.dice.loaded.push_back(6);
    mtg_engine::variants::roll_to_visit(&mut t.g, P0);
    t.g.flush_events();
    t.settle();
    t.answer_targets(P0, &[o(bears)]);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    assert!(t.on_battlefield(bears));
}
