//! Rulings batch P198 — permanents with upkeep costs (cumulative upkeep, "sacrifice ~
//! unless you pay"), tested for their other abilities: mana replacement effects (CR
//! 106.12, 614), untap and phasing with no untap step (502), simultaneous tokens, coin
//! flips (705), and cumulative upkeep costs paid in multiplayer games.

use crate::r_p194_common::damage_events;
use crate::r_p198_common::*;
use crate::r_s01_common::stack_library;
use mtg_engine::ability::LibraryPosition;
use mtg_engine::events::{Event, MoveCause};
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::replacement::{EtbInfo, MoveEv};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn pool(t: &TestGame, p: PlayerId) -> Vec<ManaType> {
    let mut v: Vec<ManaType> = t.g.player(p).mana_pool.mana.iter().map(|m| m.ty).collect();
    v.sort_by_key(|m| format!("{m:?}"));
    v
}

/// Attaches the Aura `aura` (on P0's battlefield) to `to`.
fn attach(t: &mut TestGame, aura: ObjectId, to: ObjectId) {
    t.g.objects[aura.0 as usize].attached_to = Some(obj(to));
    t.g.recompute();
}

#[test]
fn drought_counts_a_black_hybrid_symbol_however_it_is_paid() {
    cr!("107.4e", "601.2f");
    ruling!(
        "Drought",
        "A hybrid symbol that is both black and another type is a black mana symbol, regardless of what cost is paid for it."
    );
    // Deathrite Shaman ({B/G}) paid with green still needs a Swamp sacrificed.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Drought");
    let swamp = t.battlefield(P0, "Swamp");
    let ds = t.hand(P0, "Deathrite Shaman");
    mana(&mut t, P0, ManaType::G, 1);
    t.cast(P0, ds).go();
    assert!(!t.on_battlefield(swamp));
    assert!(t.in_graveyard(P0, "Swamp"));
    // Without a Swamp it can't be cast.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Drought");
    let ds = t.hand(P0, "Deathrite Shaman");
    mana(&mut t, P0, ManaType::G, 1);
    assert!(t.cast(P0, ds).try_go().is_err());
}

#[test]
fn tombstone_stairwell_creates_its_tokens_at_once_and_only_if_still_there() {
    cr!("603.4", "111.1", "603.2c");
    ruling!(
        "Tombstone Stairwell",
        "All the tokens are put onto the battlefield simultaneously."
    );
    ruling!(
        "Tombstone Stairwell",
        "Does not put the Zombies onto the battlefield if this card is not still on the battlefield when its ability resolves."
    );
    supported("Tombstone Stairwell");
    supported("Cloakwood Swarmkeeper");
    // Three Tombspawn at once: "Whenever one or more tokens you control enter"
    // triggers once.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tombstone Stairwell");
    let sk = t.battlefield(P0, "Cloakwood Swarmkeeper");
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Tombspawn").len(), 3);
    assert_eq!(t.counters(sk, "+1/+1"), 1);
    // Destroyed in response: no Zombies.
    let mut t = TestGame::new(2);
    let ts = t.battlefield(P0, "Tombstone Stairwell");
    t.graveyard(P0, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, ts);
    t.resolve_all();
    assert!(t.named_on_battlefield("Tombspawn").is_empty());
}

#[test]
fn tombstone_stairwell_destroys_its_tokens_even_if_renamed() {
    cr!("603.7c", "201.2");
    ruling!(
        "Tombstone Stairwell",
        "If something changes the names of the tokens, then the \"at end of turn\" ability will still destroy them."
    );
    supported("Cytoshape");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Tombstone Stairwell");
    let giant = t.battlefield(P1, "Hill Giant");
    t.graveyard(P0, "Grizzly Bears");
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    let ts = t.named_on_battlefield("Tombspawn");
    assert_eq!(ts.len(), 1);
    t.answer_choose(P0, &[obj(giant)]);
    cast_at(&mut t, P0, "Cytoshape", &[obj(ts[0])]);
    t.resolve_all();
    assert_eq!(t.obj_now(ts[0]).chars.name, "Hill Giant");
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(!t.on_battlefield(ts[0]));
    assert!(t.on_battlefield(giant));
}

#[test]
fn braid_of_fires_mana_empties_before_the_draw_step() {
    cr!("106.4", "500.4", "702.24a");
    ruling!(
        "Braid of Fire",
        "Any of this mana that isn't spent during your upkeep step will be removed before your draw step begins. You cannot save it to be spent on the card you will draw this turn."
    );
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Braid of Fire");
    to_upkeep(&mut t, P0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(b));
    assert_eq!(pool(&t, P0), vec![ManaType::R]);
    t.advance_to(P0, Step::Draw);
    assert!(pool(&t, P0).is_empty());
}

#[test]
fn illusionary_presence_offers_only_existing_land_subtypes() {
    cr!("205.3i", "205.4a", "702.14c");
    ruling!(
        "Illusionary Presence",
        "Can only give landwalk of land sub-types that exist (so you can no longer give \"Island of Wak-Wak walk\")."
    );
    ruling!(
        "Illusionary Presence",
        "Can't give Snow landwalk, because Snow is supertype, not a subtype."
    );
    let mut t = TestGame::new(2);
    let ip = t.battlefield(P0, "Illusionary Presence");
    t.lands(P0, "Island", 2);
    t.battlefield(P1, "Desert");
    let seen = choose_words(&mut t, P0, &["Desert"]);
    t.answer_yes(P0, true);
    to_upkeep(&mut t, P0);
    t.resolve_all();
    let lists = seen.lock().unwrap().clone();
    let types = lists
        .iter()
        .find(|l| l.iter().any(|w| w == "Island"))
        .expect("land type choice");
    assert!(types.iter().any(|w| w == "Desert"));
    assert!(!types.iter().any(|w| w == "Snow" || w.contains("Wak-Wak")));
    assert!(t
        .obj_now(ip)
        .chars
        .keywords()
        .any(|k| k.kind == mtg_engine::keywords::KeywordKind::Landwalk));
}

/// `by` puts a Swamp onto the battlefield under `controller`'s control (as an effect
/// would), then everything resolves.
fn put_swamp(t: &mut TestGame, by: PlayerId, controller: PlayerId) -> ObjectId {
    let swamp =
        t.g.create_card_object(mtg_engine::card::card("Swamp"), by, Zone::Nowhere);
    let s =
        t.g.move_object_ev(MoveEv {
            obj: swamp,
            to: Zone::Battlefield,
            pos: LibraryPosition::Top,
            cause: MoveCause::Effect,
            by: Some(by),
            etb: EtbInfo {
                controller: Some(controller),
                ..Default::default()
            },
            source: None,
        })
        .unwrap();
    t.g.flush_events();
    t.resolve_all();
    s
}

#[test]
fn thelons_chant_damages_the_player_who_put_the_swamp_onto_the_battlefield() {
    cr!("110.2a", "603.2");
    ruling!(
        "Thelon's Chant",
        "Damages the player putting the land onto the battlefield, even if they are not the controller of the land after it is on the battlefield."
    );
    supported("Thelon's Chant");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thelon's Chant");
    let s = put_swamp(&mut t, P0, P1);
    assert_eq!(ctl(&t, s), P1);
    assert_eq!(t.life(P0), 17);
    assert_eq!(t.life(P1), 20);
    // That player may instead put a -1/-1 counter on a creature they control (the
    // Chant's controller here is the other player).
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Thelon's Chant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    put_swamp(&mut t, P0, P1);
    assert_eq!(t.life(P0), 20);
    assert_eq!(t.counters(bears, "-1/-1"), 1);
    assert_eq!(t.counters(giant, "-1/-1"), 0);
}

#[test]
fn tourachs_chant_damages_the_player_whose_dryad_put_the_forest_onto_the_battlefield() {
    cr!("110.2a", "603.2");
    supported("Tourach's Chant");
    supported("Yavimaya Dryad");
    // Yavimaya Dryad: "When this creature enters, you may search your library for a Forest
    // card and put it onto the battlefield tapped under target player's control."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Tourach's Chant");
    let forest = stack_library(&mut t, P0, &["Forest"])[0];
    t.answer_targets(P0, &[pl(P1)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(forest)]);
    // Declines to put a -1/-1 counter on its Dryad.
    t.answer_yes(P0, false);
    t.enter(P0, "Yavimaya Dryad");
    t.resolve_all();
    let f = t.named_on_battlefield("Forest");
    assert_eq!(f.len(), 1);
    assert_eq!(ctl(&t, f[0]), P1);
    // P0 put it there: P0, not the Forest's controller, is dealt the damage.
    assert_eq!(t.life(P1), 20);
    assert_eq!(t.life(P0), 17);
}

#[test]
fn stasis_skips_the_untap_step_but_not_other_untapping_and_stops_phasing() {
    cr!("502.1", "502.3", "702.26a");
    ruling!(
        "Stasis",
        "Does not prevent cards from being untapped outside the untap step."
    );
    ruling!(
        "Stasis",
        "Since there is no untap step, Phasing in/out won't happen."
    );
    supported("Stasis");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Stasis");
    t.lands(P0, "Island", 2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bk = t.battlefield(P0, "Breezekeeper");
    t.g.tap(bears);
    t.answer_yes(P0, true);
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.obj_now(bears).tapped);
    assert!(!t.obj_now(bk).phased_out);
    t.advance_to(P0, Step::PrecombatMain);
    cast_at(&mut t, P0, "Vitalize", &[]);
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
}

#[test]
fn contamination_changes_land_mana_but_not_mana_from_other_effects() {
    cr!("106.12", "614.1a", "605.1b");
    ruling!(
        "Contamination",
        "External effects that generate more mana still generate their additional mana as directed without being affected by this card. Effects which generate more mana based on what type of mana the land produces will be affected in color but not in amount."
    );
    supported("Contamination");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Contamination");
    t.battlefield(P1, "Mana Flare");
    let forest = t.battlefield(P0, "Forest");
    let wg = t.battlefield(P0, "Wild Growth");
    attach(&mut t, wg, forest);
    t.activate(P0, forest, 0, &[]).unwrap();
    t.settle();
    // The Forest's {B}, Mana Flare's copy of that type ({B}), Wild Growth's {G}.
    assert_eq!(pool(&t, P0), vec![ManaType::B, ManaType::B, ManaType::G]);
}

#[test]
fn naked_singularity_replaces_the_mana_the_land_produces() {
    cr!("614.1a", "106.12");
    ruling!(
        "Naked Singularity",
        "The ability generates a replacement effect."
    );
    supported("Naked Singularity");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Naked Singularity");
    t.battlefield(P1, "Mana Flare");
    let plains = t.battlefield(P0, "Plains");
    let wg = t.battlefield(P0, "Wild Growth");
    attach(&mut t, wg, plains);
    t.activate(P0, plains, 0, &[]).unwrap();
    t.settle();
    // {R} instead of {W}; Mana Flare adds the type produced; Wild Growth's {G} isn't the
    // land's mana.
    assert_eq!(pool(&t, P0), vec![ManaType::G, ManaType::R, ManaType::R]);
}

#[test]
fn herald_of_leshrac_returns_every_land_you_control_but_dont_own() {
    cr!("613.1b", "613.7");
    ruling!(
        "Herald of Leshrac",
        "Herald of Leshrac's leaves-the-battlefield ability affects all lands you control but don't own, not just the ones you gained control of with Herald of Leshrac. For example, if you had gained control of an opponent's land with Annex, its owner will regain control of that land. Annex will remain attached to it, but its effect will be overridden."
    );
    supported("Herald of Leshrac");
    let mut t = TestGame::new(2);
    let herald = t.battlefield(P0, "Herald of Leshrac");
    let mine = t.battlefield(P0, "Forest");
    let island = t.battlefield(P1, "Island");
    let annex = cast_at(&mut t, P0, "Annex", &[obj(island)]);
    t.resolve_all();
    let annex = t.g.current(annex);
    assert_eq!(ctl(&t, island), P0);
    assert_eq!(t.pt(herald), (3, 5));
    kill(&mut t, herald);
    t.resolve_all();
    assert_eq!(ctl(&t, island), P1);
    assert_eq!(ctl(&t, mine), P0);
    assert_eq!(t.obj_now(annex).attached_to, Some(obj(island)));
    pass_turn(&mut t);
    assert_eq!(ctl(&t, island), P1);
}

#[test]
fn justice_adds_up_the_damage_of_one_source_dealt_at_once() {
    cr!("120.2", "603.2c");
    ruling!(
        "Justice",
        "If a single source does damage to multiple targets at once, Justice will add up all the damage done and deal damage to the source’s controller at one time (not multiple separate damagings)."
    );
    supported("Justice");
    let mut t = TestGame::new(2);
    let justice = t.battlefield(P0, "Justice");
    for _ in 0..3 {
        t.battlefield(P0, "Hill Giant");
    }
    t.set_step(P1, Step::PrecombatMain);
    cast_at(&mut t, P1, "Pyroclasm", &[]);
    t.resolve_all();
    let from_justice: Vec<u32> = damage_events(&t)
        .into_iter()
        .filter(|(s, _, _)| *s == justice)
        .map(|(_, _, n)| n)
        .collect();
    assert_eq!(from_justice, vec![6]);
    assert_eq!(t.life(P1), 14);
}

#[test]
fn karplusan_minotaur_triggers_on_any_won_or_lost_flip_but_not_heads_or_tails() {
    cr!("705.2", "705.3");
    ruling!(
        "Karplusan Minotaur",
        "The Minotaur’s last two abilities will trigger whenever you win or lose any coin flip. For example, if you cast Stitch in Time, one of the Minotaur’s abilities will trigger."
    );
    ruling!(
        "Karplusan Minotaur",
        "If a spell or ability uses a coin flip to determine what happens on a heads result versus a tails result, the flipping player doesn’t call “heads” or “tails.” Such flips have no winner or loser."
    );
    supported("Karplusan Minotaur");
    supported("Molten Sentry");
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Karplusan Minotaur");
    cast_at(&mut t, P0, "Stitch in Time", &[]);
    t.resolve();
    assert_eq!(crate::r_p189_common::triggers_of(&t, m), 1);
    t.resolve_all();
    // Molten Sentry's flip (heads or tails) has no winner or loser.
    let mut t = TestGame::new(2);
    let m = t.battlefield(P0, "Karplusan Minotaur");
    let s = t.enter(P0, "Molten Sentry");
    t.settle();
    let p = t.pt(s);
    assert!(p == (5, 2) || p == (2, 5));
    assert_eq!(crate::r_p189_common::triggers_of(&t, m), 0);
}

#[test]
fn varchilds_war_riders_tokens_can_go_to_different_opponents() {
    cr!("702.24a", "800.4a");
    ruling!(
        "Varchild's War-Riders",
        "In a multiplayer game, you can put the Survivor tokens onto the battlefield under the control of different players."
    );
    supported("Varchild's War-Riders");
    let mut t = TestGame::new(3);
    let vw = t.battlefield(P0, "Varchild's War-Riders");
    t.g.add_counters(obj(vw), "age", 1, None);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[pl(P1)]);
    t.answer_choose(P0, &[pl(P2)]);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.on_battlefield(vw));
    assert_eq!(t.counters(vw, "age"), 2);
    assert_eq!(with_subtype(&t, P1, "Survivor").len(), 1);
    assert_eq!(with_subtype(&t, P2, "Survivor").len(), 1);
}

#[test]
fn survivor_of_the_unseen_puts_a_card_back_even_if_both_draws_were_replaced() {
    cr!("614.11", "702.52a");
    ruling!(
        "Survivor of the Unseen",
        "If you don’t draw a card because all the draws were replaced (with dredge, for example), you still have to put a card from your hand on top of your library."
    );
    supported("Survivor of the Unseen");
    let mut t = TestGame::new(2);
    let s = t.battlefield(P0, "Survivor of the Unseen");
    t.graveyard(P0, "Golgari Thug");
    t.graveyard(P0, "Darkblast");
    let island = t.hand(P0, "Island");
    stack_library(&mut t, P0, &["Forest"; 8]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(island)]);
    let before = t.g.turn_events.len();
    act(&mut t, P0, s, 0, &[]);
    let draws = t.g.turn_events[before..]
        .iter()
        .filter(|e| matches!(e, Event::Drew { .. }))
        .count();
    assert_eq!(draws, 0);
    assert!(t.in_hand(P0, "Golgari Thug") && t.in_hand(P0, "Darkblast"));
    assert_eq!(t.hand_size(P0), 2);
    let top = *t.g.player(P0).library.last().unwrap();
    assert_eq!(t.g.obj(top).chars.name, "Island");
}

#[test]
fn peacekeeper_doesnt_skip_the_combat_phase() {
    cr!("506.1", "508.1");
    ruling!(
        "Peacekeeper",
        "Peacekeeper’s ability doesn’t cause the combat phase or the declare attackers step to be skipped."
    );
    supported("Peacekeeper");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Peacekeeper");
    t.battlefield(P0, "Legion Warboss");
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let gob = with_subtype(&t, P0, "Goblin");
    assert_eq!(gob.len(), 2);
    assert!(gob.iter().all(|g| !t.g.can_attack(*g)));
    t.advance_to(P0, Step::DeclareAttackers);
}

#[test]
fn conversion_turns_a_mountain_into_a_plains_at_once() {
    cr!("305.7", "613.1d");
    ruling!(
        "Conversion",
        "The Conversion effect is a continuous effect. There is no chance to tap a just-played mountain for red mana before it becomes a plains."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Conversion");
    let m = t.hand(P0, "Mountain");
    t.play_land(P0, m).unwrap();
    let m = t.named_on_battlefield("Mountain")[0];
    assert!(!has_ability(&t, m, "Add {R}"));
    t.activate(P0, m, 0, &[]).unwrap();
    assert_eq!(pool(&t, P0), vec![ManaType::W]);
}
