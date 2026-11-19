//! Rulings batch P106 — the other abilities of planeswalkers with a mana ability: lands
//! becoming creatures, emblems, "life total becomes", targets that become illegal, coin
//! flips, and putting creature cards onto the battlefield (CR 302.6, 613, 114, 119.5,
//! 608.2b, 705, 701.14).

use crate::r_p106_common::*;
use crate::r_s02_common::{can_attack, can_cast};
use crate::r_s06_common::give_control;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0's Koth of the Hammer emblem (from a Koth's −5).
fn koth_emblem(t: &mut TestGame) {
    let koth = t.battlefield(P0, "Koth of the Hammer");
    put_counters(t, koth, counters::LOYALTY, 2);
    act(t, P0, koth, "emblem", &[]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Koth of the Hammer"));
}

#[test]
fn koth_animating_a_mountain_that_just_came_under_your_control() {
    cr!("302.6", "508.1a");
    ruling!(
        "Koth of the Hammer",
        "If Koth’s first ability animates a Mountain that came under your control that turn, it will have “summoning sickness” and be unable to attack. It will also be unable to be tapped to activate an ability with the {T} symbol in its cost, such as the Mountain’s mana ability or the ability granted to it by Koth’s emblem."
    );
    let mut t = TestGame::new(2);
    koth_emblem(&mut t);
    let koth = t.battlefield(P0, "Koth of the Hammer");
    let old = t.battlefield(P0, "Mountain");
    let new = crate::r_s20_common::entered_this_turn(&mut t, P0, "Mountain");
    t.g.tap(new);
    act(&mut t, P0, koth, "Untap target Mountain", &[obj(new)]).unwrap();
    t.resolve_all();
    assert!(is_creature(&t, new));
    assert_eq!(t.pt(new), (4, 4));
    assert!(!t.obj_now(new).tapped);
    // No {T} abilities: not the mana ability, not the emblem's ability.
    assert!(!can_activate_containing(&mut t, P0, new, "deals 1 damage"));
    assert!(!tap_for_mana(&mut t, P0, new, "Add {R}"));
    // The Mountain controlled since the turn began (not a creature) can do both.
    assert!(can_activate_containing(&mut t, P0, old, "deals 1 damage"));
    // And it can't attack (a creature controlled since the turn began can).
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::DeclareAttackers);
    assert!(can_attack(&mut t, bears));
    assert!(!can_attack(&mut t, new));
}

#[test]
fn koths_emblem_follows_the_mountains_you_control() {
    cr!("114.4", "611.3a", "613.1f");
    ruling!(
        "Koth of the Hammer",
        "Koth’s emblem grants an activated ability to each Mountain you control at any given time for the rest of the game. It will continuously check which permanents you control are Mountains to determine what has the ability. For example, a Mountain that comes under your control later in the game will have the ability, while a Mountain you controlled at the time the emblem was created, but that later came under the control of another player, will no longer have the ability."
    );
    let mut t = TestGame::new(2);
    let early = t.battlefield(P0, "Mountain");
    koth_emblem(&mut t);
    let later = t.battlefield(P0, "Mountain");
    let theirs = t.battlefield(P1, "Mountain");
    assert!(can_activate_containing(&mut t, P0, early, "deals 1 damage"));
    assert!(can_activate_containing(&mut t, P0, later, "deals 1 damage"));
    // P1's Mountain doesn't have it; nor does P0's after P1 takes it.
    let has_ability = |t: &mut TestGame, id: ObjectId| {
        t.g.recompute();
        t.obj_now(id)
            .chars
            .abilities
            .iter()
            .any(|a| a.text.contains("deals 1 damage"))
    };
    assert!(!has_ability(&mut t, theirs));
    give_control(&mut t, early, P1);
    assert!(!has_ability(&mut t, early));
    assert!(has_ability(&mut t, later));
    // The Mountain P0 gains control of later does have it.
    give_control(&mut t, theirs, P0);
    assert!(has_ability(&mut t, theirs));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    act(&mut t, P0, later, "deals 1 damage", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

/// Activates Nissa of Shadowed Boughs's +1 on `land`, choosing to animate it.
fn nissa_plus_one(t: &mut TestGame, nissa: ObjectId, land: ObjectId) {
    t.answer_yes(P0, true);
    let r = act(t, P0, nissa, "Untap target land", &[obj(land)]);
    assert!(r.is_ok(), "{r:?}\n{}", t.dump_log());
    t.resolve_all();
}

#[test]
fn nissas_plus_one_on_an_untapped_land_or_a_land_creature() {
    cr!("115.1", "613.4b");
    ruling!(
        "Nissa of Shadowed Boughs",
        "Nissa's middle ability can target a land that's already untapped or that's already a creature."
    );
    ruling!(
        "Nissa of Shadowed Boughs",
        "Nissa's middle ability doesn't remove any abilities the land creature has."
    );
    // An untapped Forest: it becomes a 3/3 with haste and menace, keeping its mana ability.
    let mut t = TestGame::new(2);
    let nissa = t.battlefield(P0, "Nissa of Shadowed Boughs");
    let forest = crate::r_s20_common::entered_this_turn(&mut t, P0, "Forest");
    t.resolve_all(); // Nissa's landfall trigger
    nissa_plus_one(&mut t, nissa, forest);
    assert!(is_creature(&t, forest));
    assert_eq!(t.pt(forest), (3, 3));
    assert!(t.obj_now(forest).has_keyword(KeywordKind::Haste));
    assert!(t.obj_now(forest).has_keyword(KeywordKind::Menace));
    assert!(tap_for_mana(&mut t, P0, forest, "Add {G}"));
    // Dryad Arbor, already a land creature, keeps its abilities too.
    let mut t = TestGame::new(2);
    let nissa = t.battlefield(P0, "Nissa of Shadowed Boughs");
    let arbor = t.battlefield(P0, "Dryad Arbor");
    nissa_plus_one(&mut t, nissa, arbor);
    assert_eq!(t.pt(arbor), (3, 3));
    assert!(tap_for_mana(&mut t, P0, arbor, "Add {G}"));
}

#[test]
fn nissas_plus_one_overwrites_earlier_pt_setting_and_later_ones_overwrite_it() {
    cr!("613.4b", "613.7");
    ruling!(
        "Nissa of Shadowed Boughs",
        "Nissa's middle ability overwrites any previous effects that set the land creature's power and/or toughness to specific values. Other effects that set these characteristics to specific values that start to apply after the ability resolves will overwrite that part of the effect."
    );
    let mut t = TestGame::new(2);
    let nissa = t.battlefield(P0, "Nissa of Shadowed Boughs");
    let vault = t.battlefield(P0, "Mutavault");
    mana(&mut t, P0, ManaType::C, 1);
    act(&mut t, P0, vault, "2/2", &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.pt(vault), (2, 2));
    nissa_plus_one(&mut t, nissa, vault);
    assert_eq!(t.pt(vault), (3, 3));
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 1);
    cast_from_hand(&mut t, P0, "Turn to Frog", &[obj(vault)]);
    t.resolve_all();
    assert_eq!(t.pt(vault), (1, 1));
}

/// Nissa of Shadowed Boughs with enough loyalty for her −5.
fn nissa_ready(t: &mut TestGame) -> ObjectId {
    let nissa = t.battlefield(P0, "Nissa of Shadowed Boughs");
    put_counters(t, nissa, counters::LOYALTY, 1);
    nissa
}

/// The cards offered to P0 in entity choices since decision `from`.
fn offered_since(t: &TestGame, from: usize) -> Vec<Entity> {
    t.asked()[from..]
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseEntities { candidates, .. } if *p == P0 => Some(candidates.clone()),
            _ => None,
        })
        .flatten()
        .collect()
}

#[test]
fn nissas_ultimate_counts_x_as_zero_in_hand() {
    cr!("202.3e", "107.3b");
    ruling!(
        "Nissa of Shadowed Boughs",
        "If a card in a player's hand or graveyard has {X} in its mana cost, X is considered to be 0."
    );
    // Hydroid Krasis ({X}{G}{U}) has mana value 2 in hand: with two lands it can be put
    // onto the battlefield (a 0/0 with two +1/+1 counters); with one it isn't offered.
    for (lands, ok) in [(2, true), (1, false)] {
        let mut t = TestGame::new(2);
        let nissa = nissa_ready(&mut t);
        t.lands(P0, "Forest", lands);
        let krasis = t.hand(P0, "Hydroid Krasis");
        let from = t.asked().len();
        t.answer_choose(P0, &[obj(krasis)]);
        act(&mut t, P0, nissa, "You may put a creature card", &[]).unwrap();
        t.resolve_all();
        assert_eq!(offered_since(&t, from).contains(&obj(krasis)), ok, "{lands}");
        let on = t.named_on_battlefield("Hydroid Krasis");
        assert_eq!(on.len(), ok as usize, "{lands}");
        if ok {
            assert_eq!(t.counters(on[0], counters::PLUS1), 2);
            assert_eq!(t.pt(on[0]), (2, 2));
        }
    }
}

#[test]
fn nissas_ultimate_checks_mana_value_only_in_hand_or_graveyard() {
    cr!("202.3", "707.5");
    ruling!(
        "Nissa of Shadowed Boughs",
        "The creature card's mana value is checked only in your hand or graveyard. You may put a creature card onto the battlefield whose mana value becomes greater than the number of lands you control once it's on the battlefield, most likely because it enters as a copy of another permanent."
    );
    // Clone (mana value 4, four lands) enters as a copy of Craw Wurm (mana value 6).
    let mut t = TestGame::new(2);
    let nissa = nissa_ready(&mut t);
    t.lands(P0, "Forest", 4);
    let wurm = t.battlefield(P1, "Craw Wurm");
    let clone = t.graveyard(P0, "Clone");
    t.answer_choose(P0, &[obj(clone)]);
    t.answer_choose(P0, &[obj(wurm)]);
    act(&mut t, P0, nissa, "You may put a creature card", &[]).unwrap();
    t.resolve_all();
    let copy = t.g.current(clone);
    assert_eq!(t.zone(copy), Zone::Battlefield);
    assert_eq!(t.obj_now(copy).chars.name, "Craw Wurm");
    t.g.recompute();
    assert_eq!(t.g.mana_value_of(copy), 6);
    assert_eq!(t.counters(copy, counters::PLUS1), 2);
}

#[test]
fn ral_zareks_plus_one_on_a_tapped_and_an_untapped_permanent() {
    cr!("701.26a", "115.1");
    ruling!(
        "Ral Zarek",
        "Ral Zarek's first ability can target any two permanents. It doesn't matter whether they're tapped or untapped. If the first target is tapped as the ability resolves, it will just stay tapped. Similarly, if the second target is untapped as the ability resolves, it will stay untapped."
    );
    let mut t = TestGame::new(2);
    let ral = t.battlefield(P0, "Ral Zarek");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    t.g.tap(a);
    act(&mut t, P0, ral, "Tap target permanent", &[obj(a), obj(b)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(a).tapped);
    assert!(!t.obj_now(b).tapped);
    assert_eq!(t.counters(ral, counters::LOYALTY), 5);
}

#[test]
fn ral_zareks_coin_flips_are_not_won_or_lost() {
    cr!("705.2", "705.3");
    ruling!(
        "Ral Zarek",
        "The five coins you flip for Ral Zarek's last ability aren't called, and no one wins or loses these flips. Abilities that care about a player winning or losing a coin flip won't consider these flips."
    );
    let mut t = TestGame::new(2);
    let ral = t.battlefield(P0, "Ral Zarek");
    let ce = t.battlefield(P0, "Chance Encounter");
    put_counters(&mut t, ral, counters::LOYALTY, 3);
    let m = mark(&t);
    let from = t.asked().len();
    act(&mut t, P0, ral, "Flip five coins", &[]).unwrap();
    t.resolve_all();
    let flips: Vec<Event> = events_since(&t, m)
        .into_iter()
        .filter(|e| matches!(e, Event::CoinFlipped { .. }))
        .collect();
    assert_eq!(flips.len(), 5);
    assert!(flips
        .iter()
        .all(|e| matches!(e, Event::CoinFlipped { won: false, lost: false, .. })));
    assert_eq!(t.counters(ce, "luck"), 0);
    // Nobody was asked to call a coin.
    assert!(!t.asked()[from..]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseOption { .. })));
}

#[test]
fn teferis_plus_one_with_one_illegal_target_still_gains_life() {
    cr!("608.2b");
    ruling!(
        "Teferi, Who Slows the Sunset",
        "If one or more targets are illegal as the first ability resolves, but at least one target is still legal, the ability will affect only the remaining legal targets and you'll gain 2 life."
    );
    let mut t = TestGame::new(2);
    let teferi = t.battlefield(P0, "Teferi, Who Slows the Sunset");
    let ring = t.battlefield(P1, "Sol Ring");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let forest = t.battlefield(P0, "Forest");
    t.g.tap(forest);
    act(
        &mut t,
        P0,
        teferi,
        "Choose up to one target artifact",
        &[obj(ring), obj(bears), obj(forest)],
    )
    .unwrap();
    destroy(&mut t, bears);
    t.resolve_all();
    assert!(t.obj_now(ring).tapped);
    assert!(!t.obj_now(forest).tapped);
    assert_eq!(t.life(P0), 22);
}

/// P0's Tezzeret the Schemer emblem.
fn tezzeret_emblem(t: &mut TestGame) {
    let tez = t.battlefield(P0, "Tezzeret the Schemer");
    put_counters(t, tez, counters::LOYALTY, 2);
    act(t, P0, tez, "emblem", &[]).unwrap();
    t.resolve_all();
}

#[test]
fn tezzerets_emblem_unattaches_an_equipment_for_good() {
    cr!("301.5c", "704.5n", "611.2a");
    ruling!(
        "Tezzeret the Schemer",
        "If an Equipment becomes a creature, it becomes unattached and it can't be attached to a creature."
    );
    ruling!(
        "Tezzeret the Schemer",
        "The effect of the emblem's triggered ability lasts indefinitely."
    );
    let mut t = TestGame::new(2);
    tezzeret_emblem(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blade = crate::r_s06_common::attach_new(&mut t, P0, "Bonesplitter", bears);
    assert_eq!(t.pt(bears), (4, 2));
    t.answer_targets(P0, &[obj(blade)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(is_creature(&t, blade));
    assert_eq!(t.pt(blade), (5, 5));
    assert_eq!(t.obj_now(blade).attached_to, None);
    assert_eq!(t.pt(bears), (2, 2));
    // Equip can't attach it again.
    t.set_step(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::C, 1);
    let _ = act(&mut t, P0, blade, "Equip", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.obj_now(blade).attached_to, None);
    // Still a 5/5 artifact creature in later turns.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P1, Step::End);
    t.resolve_all();
    assert!(is_creature(&t, blade));
    assert_eq!(t.pt(blade), (5, 5));
}

#[test]
fn vraskas_ultimate_makes_the_player_lose_life() {
    cr!("119.5");
    ruling!(
        "Vraska, Relic Seeker",
        "For a player's life total to become 1, what actually happens is that the player loses the appropriate amount of life (or in some rare cases, gains the appropriate amount of life). For example, if the targeted player's life total is 4 when this ability resolves, that player loses 3 life. Other cards that interact with life loss will interact with this effect accordingly."
    );
    let mut t = TestGame::new(2);
    let vraska = t.battlefield(P0, "Vraska, Relic Seeker");
    put_counters(&mut t, vraska, counters::LOYALTY, 4);
    t.battlefield(P0, "Exquisite Blood");
    t.g.players[1].life = 4;
    let m = mark(&t);
    act(&mut t, P0, vraska, "life total becomes 1", &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(t.life(P1), 1);
    assert_eq!(life_lost_since(&t, m, P1), 3);
    // Exquisite Blood sees the loss.
    assert_eq!(t.life(P0), 23);
}

#[test]
fn power_four_enters_triggers_count_statics_counters_and_copies() {
    cr!("603.6a", "603.10", "614.1c");
    ruling!(
        "Kiora, Behemoth Beckoner",
        "The entering creature must have power 4 or greater as it enters the battlefield, or Kiora’s first ability won’t trigger. Static abilities that raise or lower a creature’s power are taken into account, as well as any +1/+1 counters the creature enters with. However, you can’t have a creature with power 3 or less enter the battlefield, raise its power with a spell, an activated ability, or a triggered ability, and have Kiora’s ability trigger."
    );
    ruling!(
        "Outcaster Trailblazer",
        "If one or more static abilities that apply to a creature entering the battlefield change its power, those abilities are considered when determining whether or not Outcaster Trailblazer’s second ability triggers. The same is true for replacement effects that apply to it, such as entering the battlefield with one or more +1/+1 counters or entering the battlefield as a copy of another creature."
    );
    for watcher in ["Kiora, Behemoth Beckoner", "Outcaster Trailblazer"] {
        // A 3/3 Hill Giant: no draw; with Glorious Anthem: a draw.
        let mut t = TestGame::new(2);
        t.battlefield(P0, watcher);
        let hand = t.hand_size(P0);
        t.enter(P0, "Hill Giant");
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand, "{watcher}");
        t.battlefield(P0, "Glorious Anthem");
        t.enter(P0, "Hill Giant");
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "{watcher}: anthem");
        // Endless One cast with X = 4 enters with four +1/+1 counters: a draw.
        let mut t = TestGame::new(2);
        t.battlefield(P0, watcher);
        let hand = t.hand_size(P0);
        mana(&mut t, P0, ManaType::C, 4);
        let one = t.hand(P0, "Endless One");
        t.cast(P0, one).x(4).go();
        t.resolve_all();
        // Endless One left the hand (it was put there after counting) and a card was
        // drawn.
        assert_eq!(t.hand_size(P0), hand + 1, "{watcher}: counters");
        assert_eq!(t.named_on_battlefield("Endless One").len(), 1);
        // Clone entering as a copy of Craw Wurm: a draw.
        let mut t = TestGame::new(2);
        t.battlefield(P0, watcher);
        let wurm = t.battlefield(P1, "Craw Wurm");
        let hand = t.hand_size(P0);
        t.answer_choose(P0, &[obj(wurm)]);
        t.enter(P0, "Clone");
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "{watcher}: clone");
        // A Hill Giant pumped with Giant Growth after entering: no draw.
        let mut t = TestGame::new(2);
        t.battlefield(P0, watcher);
        let giant = t.enter(P0, "Hill Giant");
        t.resolve_all();
        mana(&mut t, P0, ManaType::G, 1);
        cast_from_hand(&mut t, P0, "Giant Growth", &[obj(giant)]);
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.pt(giant), (6, 6));
        assert_eq!(t.hand_size(P0), hand, "{watcher}: pumped later");
    }
}

#[test]
fn power_four_enters_triggers_draw_even_if_power_drops_or_the_creature_leaves() {
    cr!("603.2", "603.10");
    ruling!(
        "Kiora, Behemoth Beckoner",
        "If the entering creature’s power changes to 3 or less after it has entered the battlefield, you’ll still draw a card."
    );
    ruling!(
        "Outcaster Trailblazer",
        "Once Outcaster Trailblazer’s second ability has triggered, lowering the power of the creature or removing it from the battlefield won’t stop you from drawing a card."
    );
    for watcher in ["Kiora, Behemoth Beckoner", "Outcaster Trailblazer"] {
        // Turned into a 1/1 Frog with the trigger on the stack.
        let mut t = TestGame::new(2);
        t.battlefield(P0, watcher);
        let wurm = t.enter(P0, "Craw Wurm");
        t.settle();
        assert_eq!(t.stack_len(), 1);
        mana(&mut t, P0, ManaType::U, 1);
        mana(&mut t, P0, ManaType::C, 1);
        cast_from_hand(&mut t, P0, "Turn to Frog", &[obj(wurm)]);
        t.resolve();
        assert_eq!(t.pt(wurm), (1, 1));
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "{watcher}: frog");
        // Destroyed with the trigger on the stack.
        let mut t = TestGame::new(2);
        t.battlefield(P0, watcher);
        let wurm = t.enter(P0, "Craw Wurm");
        t.settle();
        destroy(&mut t, wurm);
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + 1, "{watcher}: destroyed");
    }
}

#[test]
fn domris_fight_does_nothing_if_either_target_is_illegal() {
    cr!("701.14b", "608.2b");
    ruling!(
        "Domri, Anarch of Bolas",
        "If either target is an illegal target as Domri's last ability resolves, no creature will deal or be dealt damage."
    );
    // P0's creature is gone: P1's creature isn't dealt damage.
    let mut t = TestGame::new(2);
    let domri = t.battlefield(P0, "Domri, Anarch of Bolas");
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    act(&mut t, P0, domri, "fights", &[obj(mine), obj(theirs)]).unwrap();
    destroy(&mut t, mine);
    t.resolve_all();
    assert_eq!(t.obj_now(theirs).damage, 0);
    assert!(t.on_battlefield(theirs));
    // P1's creature came under P0's control ("you don't control" is false): neither is
    // dealt damage.
    let mut t = TestGame::new(2);
    let domri = t.battlefield(P0, "Domri, Anarch of Bolas");
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    act(&mut t, P0, domri, "fights", &[obj(mine), obj(theirs)]).unwrap();
    give_control(&mut t, theirs, P0);
    t.resolve_all();
    assert_eq!(t.obj_now(mine).damage, 0);
    assert_eq!(t.obj_now(theirs).damage, 0);
    assert!(t.on_battlefield(theirs));
}

const ROWAN: &str = "Rowan, Scholar of Sparks // Will, Scholar of Frost";

#[test]
fn chandras_middle_ability_checks_the_exiled_card_so_either_face_may_be_cast() {
    cr!("712.8a", "712.11b", "601.3e");
    ruling!(
        "Chandra, Dressed to Kill",
        "Chandra’s second ability checks the characteristics of the card you exiled to see if it’s red. If the card is red, but the spell that card becomes somehow isn’t red, you may still cast it. For example, Rowan, Scholar of Sparks, the front face of a modal double-faced card, is red. Will, Scholar of Frost, the back face of the same card, is blue. In exile, that card is red, so you may cast either face if you exile it with this ability."
    );
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Dressed to Kill");
    t.library_top(P0, ROWAN);
    act(&mut t, P0, chandra, "Exile the top card", &[]).unwrap();
    t.resolve_all();
    let card = t.g.find_in_zone(Zone::Exile, "Rowan, Scholar of Sparks")[0];
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 4);
    assert!(can_cast(&mut t, P0, card, CastMethod::Normal));
    assert!(can_cast(&mut t, P0, card, CastMethod::Half(1)));
    t.cast(P0, card).method(CastMethod::Half(1)).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Will, Scholar of Frost").len(), 1);
}

#[test]
fn chandras_ultimate_casts_only_spells_that_are_red() {
    cr!("712.8a", "601.3e");
    ruling!(
        "Chandra, Dressed to Kill",
        "Chandra’s last ability, by contrast, only allows you to cast spells that are red when you cast them, no matter what the card in exile is. If Rowan, Scholar of Sparks (a modal double-faced card) is exiled by this ability, you won’t be able to cast its back face, Will, Scholar of Frost, this way because it would be a blue spell."
    );
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Dressed to Kill");
    put_counters(&mut t, chandra, counters::LOYALTY, 4);
    stack_library(
        &mut t,
        P0,
        &[ROWAN, "Island", "Island", "Island", "Island"],
    );
    act(&mut t, P0, chandra, "emblem", &[]).unwrap();
    t.resolve_all();
    let card = t.g.find_in_zone(Zone::Exile, "Rowan, Scholar of Sparks")[0];
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 4);
    assert!(!can_cast(&mut t, P0, card, CastMethod::Half(1)));
    assert!(can_cast(&mut t, P0, card, CastMethod::Normal));
    t.cast(P0, card).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Rowan, Scholar of Sparks").len(), 1);
}
