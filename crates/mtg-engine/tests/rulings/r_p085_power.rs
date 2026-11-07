//! Rulings batch P085 — hosers of high or low power and toughness: targets whose power or
//! toughness is checked on targeting and on resolution, "exile until ~ leaves the
//! battlefield", blocks that power changes don't undo, Azure Beastbinder's layered effect,
//! and Mob Rule.

use crate::r_p076_common::unblockable_after_blocked;
use crate::r_p085_common::*;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

/// `p` casts `spell` (from hand, paid with `types`) targeting `target`.
fn cast_at(
    t: &mut TestGame,
    p: PlayerId,
    spell: &str,
    types: &[(ManaType, u32)],
    target: ObjectId,
) {
    let id = t.hand(p, spell);
    pool(t, p, types);
    t.cast(p, id).target(target).go();
}

/// P1 casts Giant Growth on `target` (+3/+3).
fn giant_growth(t: &mut TestGame, target: ObjectId) {
    cast_at(t, P1, "Giant Growth", &[(ManaType::G, 1)], target);
}

/// P1 casts Disfigure on `target` (-2/-2).
fn disfigure(t: &mut TestGame, target: ObjectId) {
    cast_at(t, P1, "Disfigure", &[(ManaType::B, 1)], target);
}

// ---------------------------------------------------------------------------------------
// Targets checked on resolution
// ---------------------------------------------------------------------------------------

#[test]
fn power_checked_on_targeting_and_resolution() {
    cr!("115.1", "608.2b");
    ruling!(
        "Intrepid Hero",
        "The power of the creature is checked on activation and on resolution."
    );
    ruling!(
        "Bala Ged Scorpion",
        "The power of the targeted creature is checked both as you target it and as the ability resolves."
    );
    ruling!(
        "Kor Line-Slinger",
        "The power of the targeted creature is checked both as you target it and as the ability resolves."
    );
    ruling!(
        "Triumphant Surge",
        "If the target creature is an illegal target by the time Triumphant Surge tries to resolve, the spell won’t resolve. You won’t gain 3 life."
    );
    for n in [
        "Intrepid Hero",
        "Bala Ged Scorpion",
        "Kor Line-Slinger",
        "Triumphant Surge",
    ] {
        supported(n);
    }
    // Intrepid Hero: a 3/3 isn't a legal target; a 4/4 shrunk in response survives.
    let mut t = TestGame::new(2);
    let hero = t.battlefield(P0, "Intrepid Hero");
    let giant = t.battlefield(P1, "Hill Giant");
    let air = t.battlefield(P1, "Air Elemental");
    t.activate(P0, hero, 0, &[Entity::Object(air)]).unwrap();
    // The 3/3 wasn't among the possible targets.
    assert!(t.asked().iter().any(|(_, d)| matches!(d,
        decision::Decision::ChooseTargets { candidates, .. }
            if candidates.contains(&Entity::Object(air))
                && !candidates.contains(&Entity::Object(giant)))));
    disfigure(&mut t, air);
    t.resolve_all();
    assert!(t.on_battlefield(air));
    assert!(t.on_battlefield(giant));
    // Bala Ged Scorpion: a 1/1 pumped in response survives.
    let mut t = TestGame::new(2);
    let elves = t.battlefield(P1, "Llanowar Elves");
    t.answer_targets(P0, &[Entity::Object(elves)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Bala Ged Scorpion");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    giant_growth(&mut t, elves);
    t.resolve_all();
    assert!(t.on_battlefield(elves));
    // Kor Line-Slinger: a 2/2 pumped to 5/5 isn't tapped.
    let mut t = TestGame::new(2);
    let slinger = t.battlefield(P0, "Kor Line-Slinger");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, slinger, 0, &[Entity::Object(bears)])
        .unwrap();
    giant_growth(&mut t, bears);
    t.resolve_all();
    assert!(!t.obj_now(bears).tapped);
    // Triumphant Surge: no life gain when the target shrinks.
    let mut t = TestGame::new(2);
    let air = t.battlefield(P1, "Air Elemental");
    cast_at(&mut t, P0, "Triumphant Surge", &[(ManaType::W, 4)], air);
    disfigure(&mut t, air);
    t.resolve_all();
    assert!(t.on_battlefield(air));
    assert_eq!(t.life(P0), 20);
}

#[test]
fn fleshpulper_giant_toughness_checks() {
    cr!("608.2b", "603.3d", "115.1");
    ruling!(
        "Fleshpulper Giant",
        "If the target creature’s toughness is greater than 2 when the ability tries to resolve"
    );
    ruling!(
        "Fleshpulper Giant",
        "If you control the only creature with toughness 2 or less, you must choose it as the target, though you can choose to not destroy it."
    );
    supported("Fleshpulper Giant");
    // Toughness raised in response: not destroyed.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    t.enter(P0, "Fleshpulper Giant");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    giant_growth(&mut t, bears);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // No legal target: the ability isn't put on the stack.
    let mut t = TestGame::new(2);
    t.enter(P0, "Fleshpulper Giant");
    t.settle();
    assert_eq!(t.stack_len(), 0);
    // Only P0's own Bears: it's the target, and P0 may decline.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    t.answer_yes(P0, false);
    t.enter(P0, "Fleshpulper Giant");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.on_battlefield(mine));
}

#[test]
fn mandate_of_abaddon_illegal_chosen_creature() {
    cr!("608.2b");
    ruling!(
        "Mandate of Abaddon",
        "If the chosen creature is an illegal target as Mandate of Abaddon tries to resolve"
    );
    supported("Mandate of Abaddon");
    let mut t = TestGame::new(2);
    let air = t.battlefield(P0, "Air Elemental");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_at(&mut t, P0, "Mandate of Abaddon", &[(ManaType::B, 4)], air);
    // P1 gains control of the Air Elemental: it's no longer a creature P0 controls.
    crate::r_s06_common::give_control(&mut t, air, P1);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn fights_with_illegal_target() {
    cr!("701.14a", "701.14b", "608.2b");
    ruling!(
        "Clash of Titans",
        "Clash of Titans can target two creatures controlled by the same player."
    );
    ruling!(
        "Clash of Titans",
        "If either target is an illegal target as Clash of Titans tries to resolve, neither creature will deal or be dealt damage."
    );
    supported("Clash of Titans");
    // Two of P1's creatures fight.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Clash of Titans");
    pool(&mut t, P0, &[(ManaType::R, 5)]);
    t.cast(P0, spell).target(giant).target(bears).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.obj_now(giant).damage, 2);
    // One target leaves: no damage at all.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Clash of Titans");
    pool(&mut t, P0, &[(ManaType::R, 5)]);
    t.cast(P0, spell).target(giant).target(bears).go();
    let b = t.g.current(bears);
    t.g.move_object(b, Zone::Hand(P1), events::MoveCause::Effect, None);
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 0);
}

#[test]
fn getaway_glamer_power_checked_after_first_mode() {
    cr!("702.172a", "608.2c");
    ruling!(
        "Getaway Glamer",
        "Getaway Glamer’s last mode can target any creature, even a creature that doesn’t have the undisputed greatest power"
    );
    supported("Getaway Glamer");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Getaway Glamer");
    pool(&mut t, P0, &[(ManaType::W, 4)]);
    t.cast(P0, spell)
        .modes(&[0, 1])
        .target(giant)
        .target(bears)
        .go();
    t.resolve_all();
    // The Giant was exiled first, so the Bears had the greatest power.
    assert_eq!(t.zone(giant), Zone::Exile);
    assert!(!t.on_battlefield(bears));
    // Only the second mode, with a bigger creature around: nothing happens.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Getaway Glamer");
    pool(&mut t, P0, &[(ManaType::W, 3)]);
    t.cast(P0, spell).modes(&[1]).target(bears).go();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn mistmeadow_skulk_x_counts_on_stack() {
    cr!("202.3e", "702.16b");
    ruling!(
        "Mistmeadow Skulk",
        "Mistmeadow Skulk can be chosen as a target for a Blaze (which has mana cost {X}{R}) if X is 0 or 1"
    );
    supported("Mistmeadow Skulk");
    for (x, ok) in [(1, true), (2, false)] {
        let mut t = TestGame::new(2);
        let skulk = t.battlefield(P1, "Mistmeadow Skulk");
        let blaze = t.hand(P0, "Blaze");
        pool(&mut t, P0, &[(ManaType::R, 3)]);
        let r = t.cast(P0, blaze).x(x).target(skulk).try_go();
        if ok {
            r.unwrap();
            t.resolve_all();
            assert!(!t.on_battlefield(skulk));
        } else {
            // Either illegal, or the engine picked another legal target (P0 or P1).
            t.resolve_all();
            assert!(t.on_battlefield(skulk), "x={x}");
        }
    }
}

// ---------------------------------------------------------------------------------------
// Nightmare Unmaking and Mob Rule: checked on resolution
// ---------------------------------------------------------------------------------------

#[test]
fn nightmare_unmaking_counts_hand_on_resolution() {
    cr!("608.2h", "700.2a");
    ruling!(
        "Nightmare Unmaking",
        "Because Nightmare Unmaking is on the stack while it's resolving, it won't be counted among the cards in your hand."
    );
    ruling!(
        "Nightmare Unmaking",
        "The power of creatures and the number of cards in your hand are checked only as Nightmare Unmaking resolves."
    );
    supported("Nightmare Unmaking");
    // Two other cards in hand: power greater than 2 is exiled.
    let mut t = TestGame::new(2);
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let spell = t.hand(P0, "Nightmare Unmaking");
    pool(&mut t, P0, &[(ManaType::B, 5)]);
    t.cast(P0, spell).modes(&[0]).go();
    t.resolve_all();
    assert_eq!(t.zone(giant), Zone::Exile);
    assert!(t.on_battlefield(bears));
    // P0 draws in response: three cards, the Giant isn't exiled.
    let mut t = TestGame::new(2);
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    let giant = t.battlefield(P1, "Hill Giant");
    let spell = t.hand(P0, "Nightmare Unmaking");
    pool(&mut t, P0, &[(ManaType::B, 5)]);
    t.cast(P0, spell).modes(&[0]).go();
    t.g.draw_cards(P0, 1);
    t.resolve_all();
    assert!(t.on_battlefield(giant));
}

#[test]
fn mob_rule_mode_fixed_set_on_resolution() {
    cr!("700.2a", "608.2h", "611.2c");
    ruling!(
        "Mob Rule",
        "You choose which mode you're using as you cast Mob Rule. Once it's cast, you can't change its mode"
    );
    ruling!(
        "Mob Rule",
        "Once you gain control of a creature, it doesn't matter what happens to its power."
    );
    ruling!(
        "Mob Rule",
        "Mob Rule can affect creatures you already control or ones that are already untapped."
    );
    supported("Mob Rule");
    let mut t = TestGame::new(2);
    let air = t.battlefield(P1, "Air Elemental");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let craw = t.battlefield(P1, "Craw Wurm");
    let mine = t.battlefield_sick(P0, "Serra Angel");
    t.g.tap(mine);
    let spell = t.hand(P0, "Mob Rule");
    pool(&mut t, P0, &[(ManaType::R, 6)]);
    t.cast(P0, spell).modes(&[0]).go();
    // In response, the Bears grow to 5/5 and the Air Elemental shrinks to 2/2.
    giant_growth(&mut t, bears);
    disfigure(&mut t, air);
    t.resolve_all();
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(t.obj_now(craw).controller, P0);
    assert_eq!(t.obj_now(air).controller, P1);
    // P0's own tapped, summoning-sick Serra Angel is untapped and has haste.
    assert!(!t.obj_now(mine).tapped);
    assert!(t.obj_now(mine).has_keyword(keywords::KeywordKind::Haste));
    // Shrinking a gained creature doesn't return it.
    disfigure(&mut t, craw);
    t.resolve_all();
    assert_eq!(t.obj_now(craw).controller, P0);
}

// ---------------------------------------------------------------------------------------
// Exile until ~ leaves the battlefield
// ---------------------------------------------------------------------------------------

#[test]
fn aligned_hedron_network_checks_power_on_resolution() {
    cr!("608.2h", "610.3");
    ruling!(
        "Aligned Hedron Network",
        "Check the power of each creature as Aligned Hedron Network’s ability resolves to determine if it’s exiled."
    );
    ruling!(
        "Aligned Hedron Network",
        "If Aligned Hedron Network exiles multiple creatures, those cards all return to the battlefield at the same time."
    );
    supported("Aligned Hedron Network");
    supported("Hamletback Goliath");
    let mut t = TestGame::new(2);
    let goliath = t.battlefield(P0, "Hamletback Goliath");
    let giant = t.battlefield(P1, "Hill Giant");
    let craw = t.battlefield(P1, "Craw Wurm");
    let net = t.enter(P0, "Aligned Hedron Network");
    t.settle();
    // The Hill Giant becomes 6/6 in response.
    giant_growth(&mut t, giant);
    t.resolve_all();
    for c in [goliath, giant, craw] {
        assert_eq!(t.zone(c), Zone::Exile);
    }
    // They return together: the Goliath sees the other two enter.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    let net = t.g.current(net);
    t.g.destroy(net, None);
    t.resolve_all();
    let craw_now = t.g.current(craw);
    assert_eq!(t.g.obj(craw_now).controller, P1);
    assert_eq!(t.counters(goliath, "+1/+1"), 6 + 3);
}

#[test]
fn until_leaves_source_gone_before_resolution() {
    cr!("610.3c");
    ruling!(
        "Aligned Hedron Network",
        "If Aligned Hedron Network leaves the battlefield before its triggered ability resolves, no creatures will be exiled."
    );
    ruling!(
        "Suspension Field",
        "If Suspension Field leaves the battlefield before its enters-the-battlefield ability resolves, the target creature won’t be exiled."
    );
    supported("Suspension Field");
    let mut t = TestGame::new(2);
    let craw = t.battlefield(P1, "Craw Wurm");
    let net = t.enter(P0, "Aligned Hedron Network");
    t.settle();
    t.g.destroy(net, None);
    t.resolve_all();
    assert!(t.on_battlefield(craw));
    let mut t = TestGame::new(2);
    let craw = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(craw)]);
    t.answer_yes(P0, true);
    let field = t.enter(P0, "Suspension Field");
    t.settle();
    t.g.destroy(field, None);
    t.resolve_all();
    assert!(t.on_battlefield(craw));
}

#[test]
fn suspension_field_target_and_token() {
    cr!("608.2b", "111.8", "610.3");
    ruling!(
        "Suspension Field",
        "If the target creature becomes an illegal target before the ability resolves, perhaps because its toughness was lowered"
    );
    ruling!(
        "Suspension Field",
        "If a token is exiled this way, it ceases to exist. It won’t be returned to the battlefield."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer_yes(P0, true);
    let field = t.enter(P0, "Suspension Field");
    t.settle();
    disfigure(&mut t, giant);
    t.resolve_all();
    assert!(t.on_battlefield(field));
    // (Hill Giant 3/3 -2/-2 = 1/1 survives and wasn't exiled.)
    assert!(t.on_battlefield(giant));
    // A token creature: exiled, gone for good.
    let mut t = TestGame::new(2);
    let herd = t.hand(P0, "Call of the Herd");
    pool(&mut t, P0, &[(ManaType::G, 3)]);
    t.cast(P0, herd).go();
    t.resolve_all();
    let token = crate::r_s01_common::tokens(&t, P0)[0];
    t.answer_targets(P0, &[Entity::Object(token)]);
    t.answer_yes(P0, true);
    let field = t.enter(P0, "Suspension Field");
    t.resolve_all();
    assert!(!t.on_battlefield(token));
    t.g.destroy(field, None);
    t.resolve_all();
    assert!(crate::r_s01_common::tokens(&t, P0).is_empty());
}

#[test]
fn until_leaves_return_is_immediate() {
    cr!("610.3a", "610.3");
    ruling!(
        "Suspension Field",
        "Suspension Field’s ability causes a zone change with a duration."
    );
    ruling!(
        "Suspension Field",
        "The exiled card returns to the battlefield immediately after Suspension Field leaves the battlefield. Nothing happens between these two events"
    );
    ruling!(
        "Aligned Hedron Network",
        "The exiled cards return to the battlefield immediately after Aligned Hedron Network leaves the battlefield. Nothing happens between the two events"
    );
    // Suspension Field: the card is back as part of the destruction, before any SBA or
    // trigger, and the Field is gone by then.
    let mut t = TestGame::new(2);
    let craw = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(craw)]);
    t.answer_yes(P0, true);
    let field = t.enter(P0, "Suspension Field");
    t.resolve_all();
    assert_eq!(t.zone(craw), Zone::Exile);
    t.g.destroy(field, None);
    t.g.flush_events();
    assert!(t.on_battlefield(craw));
    assert_eq!(t.zone(field), Zone::Graveyard(P0));
    assert_eq!(t.stack_len(), 0);
    // Aligned Hedron Network likewise.
    let mut t = TestGame::new(2);
    let craw = t.battlefield(P1, "Craw Wurm");
    let net = t.enter(P0, "Aligned Hedron Network");
    t.resolve_all();
    assert_eq!(t.zone(craw), Zone::Exile);
    t.g.destroy(net, None);
    t.g.flush_events();
    assert!(t.on_battlefield(craw));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn until_leaves_owner_leaves_the_game() {
    cr!("800.4a", "610.3");
    ruling!(
        "Aligned Hedron Network",
        "In a multiplayer game, if Aligned Hedron Network’s owner leaves the game, the exiled cards will return to the battlefield."
    );
    ruling!(
        "Suspension Field",
        "In a multiplayer game, if Suspension Field’s owner leaves the game, the exiled card will return to the battlefield."
    );
    let mut t = TestGame::new(3);
    let craw = t.battlefield(P1, "Craw Wurm");
    t.enter(P0, "Aligned Hedron Network");
    t.resolve_all();
    assert_eq!(t.zone(craw), Zone::Exile);
    t.g.player_loses(P0);
    t.settle();
    assert!(t.on_battlefield(craw));
    let mut t = TestGame::new(3);
    let craw = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[Entity::Object(craw)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Suspension Field");
    t.resolve_all();
    assert_eq!(t.zone(craw), Zone::Exile);
    t.g.player_loses(P0);
    t.settle();
    assert!(t.on_battlefield(craw));
}

#[test]
fn aligned_hedron_network_exiling_itself_loops() {
    cr!("104.4b", "610.3");
    ruling!(
        "Aligned Hedron Network",
        "If this causes a loop with Aligned Hedron Network continually exiling and returning itself, the game will be a draw"
    );
    // March of the Machines makes it a 4/4, Glorious Anthem a 5/5.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "March of the Machines");
    t.battlefield(P0, "Glorious Anthem");
    t.enter(P0, "Aligned Hedron Network");
    t.g.run_until(4000, |g| g.is_over());
    assert_eq!(t.g.result, Some(GameResult::Draw));
}

// ---------------------------------------------------------------------------------------
// Blocks that power changes don't undo
// ---------------------------------------------------------------------------------------

/// P0 attacks with `attacker`; P1 blocks with `blocker`; then P1 changes the blocker's
/// power with `change` (Giant Growth or Disfigure). The attacker stays blocked.
fn stays_blocked(attacker: &str, blocker: &str, change: fn(&mut TestGame, ObjectId)) {
    supported(attacker);
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, attacker);
    let b = t.battlefield(P1, blocker);
    unblockable_after_blocked(&mut t, a, b, |t| change(t, b));
}

#[test]
fn blocks_stay_after_power_changes() {
    cr!("509.1h", "506.4");
    ruling!(
        "Goldmeadow Dodger",
        "After Goldmeadow Dodger becomes blocked, increasing the blocking creature’s power to 4 or greater has no effect on the block."
    );
    ruling!(
        "Azure Beastbinder",
        "Once Azure Beastbinder has been blocked, increasing the blocking creature's power to 2 or greater won't cause Azure Beastbinder to become unblocked."
    );
    ruling!(
        "Duskmantle Operative",
        "Once a creature with power 3 or less has blocked this creature, changing the power of the blocking creature won’t cause this creature to become unblocked."
    );
    ruling!(
        "War-Name Aspirant",
        "Once a creature legally blocks War-Name Aspirant, changing that creature's power to 1 or less won't change or undo that block."
    );
    stays_blocked("Goldmeadow Dodger", "Grizzly Bears", giant_growth);
    stays_blocked("Azure Beastbinder", "Llanowar Elves", giant_growth);
    stays_blocked("Duskmantle Operative", "Grizzly Bears", giant_growth);
    stays_blocked("War-Name Aspirant", "Hill Giant", disfigure);
}

#[test]
fn squeak_by_block_stays() {
    cr!("509.1h", "715.3");
    ruling!(
        "Cheeky House-Mouse // Squeak By",
        "Increasing a creature's power after it has blocked a creature affected by Squeak By will not remove that blocking creature from combat"
    );
    let mut t = TestGame::new(2);
    let mouse = t.hand(P0, "Cheeky House-Mouse");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Grizzly Bears");
    pool(&mut t, P0, &[(ManaType::W, 1)]);
    t.cast(P0, mouse)
        .method(CastMethod::Half(1))
        .target(bears)
        .go();
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 3));
    unblockable_after_blocked(&mut t, bears, blocker, |t| giant_growth(t, blocker));
}

#[test]
fn immobilizer_eldrazi_blocks() {
    cr!("509.1b", "509.1h");
    ruling!(
        "Immobilizer Eldrazi",
        "If a creature has already been legally declared as a blocker, Immobilizer Eldrazi’s ability won’t change or undo that block."
    );
    ruling!(
        "Immobilizer Eldrazi",
        "You compare the power and toughness as you declare blockers, not as Immobilizer Eldrazi’s ability resolves."
    );
    supported("Immobilizer Eldrazi");
    supported("Iron Will");
    // Activated after blocks: the Horned Turtle (1/4) keeps blocking.
    let mut t = TestGame::new(2);
    let eldrazi = t.battlefield(P0, "Immobilizer Eldrazi");
    let giant = t.battlefield(P0, "Hill Giant");
    let turtle = t.battlefield(P1, "Horned Turtle");
    unblockable_after_blocked(&mut t, giant, turtle, |t| {
        pool(t, P0, &[(ManaType::C, 3)]);
        t.activate(P0, eldrazi, 0, &[]).unwrap();
    });
    // Activated in the main phase; the Bears get +0/+4 later: they can't block. The
    // Elvish Warrior (2/3) gets +2/+0 (Rally the Peasants): it can.
    for bears_block in [true, false] {
        let mut t = TestGame::new(2);
        let eldrazi = t.battlefield(P0, "Immobilizer Eldrazi");
        let giant = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        let warrior = t.battlefield(P1, "Elvish Warrior");
        pool(&mut t, P0, &[(ManaType::C, 3)]);
        t.activate(P0, eldrazi, 0, &[]).unwrap();
        t.resolve_all();
        cast_at(&mut t, P1, "Iron Will", &[(ManaType::W, 1)], bears);
        t.resolve_all();
        let rally = t.hand(P1, "Rally the Peasants");
        pool(&mut t, P1, &[(ManaType::W, 3)]);
        t.cast(P1, rally).go();
        t.resolve_all();
        assert_eq!(t.pt(warrior), (4, 3));
        assert_eq!(t.pt(bears), (4, 6));
        let blocker = if bears_block { bears } else { warrior };
        crate::r_s03_common::to_blockers(
            &mut t,
            &[(giant, Entity::Player(P1))],
            &[(blocker, giant)],
        );
        let blocks = crate::r_s21_common::blocks_now(&t);
        assert_eq!(
            blocks.iter().any(|(b, _)| *b == blocker),
            !bears_block,
            "bears_block={bears_block}"
        );
    }
}

// ---------------------------------------------------------------------------------------
// Azure Beastbinder's effect
// ---------------------------------------------------------------------------------------

/// P0 attacks with Azure Beastbinder targeting `target` (P1's); returns the game in the
/// declare attackers step after the trigger resolved.
fn beastbind(t: &mut TestGame, target: ObjectId) -> ObjectId {
    supported("Azure Beastbinder");
    let bb = t.battlefield(P0, "Azure Beastbinder");
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer_targets(P0, &[Entity::Object(target)]);
    t.answer(
        P0,
        DecisionKind::Attackers,
        decision::Answer::Attackers(vec![(bb, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    bb
}

#[test]
fn azure_beastbinder_layers() {
    cr!("613.4b", "613.4c", "613.7", "613.1f");
    ruling!(
        "Azure Beastbinder",
        "A permanent that loses all abilities because of Azure Beastbinder's last ability may later gain abilities."
    );
    ruling!(
        "Azure Beastbinder",
        "Azure Beastbinder's last ability overwrites all previous effects that set the creature's base power and toughness to specific values."
    );
    ruling!(
        "Azure Beastbinder",
        "Effects that modify the creature's power and/or toughness, such as the effect of Overprotect, will apply to the creature no matter when they started to take effect."
    );
    supported("Turn to Frog");
    supported("Jump");
    let mut t = TestGame::new(2);
    let air = t.battlefield(P1, "Air Elemental");
    t.g.add_counters(Entity::Object(air), "+1/+1", 1, None);
    // An earlier base-setting effect (Turn to Frog: 1/1) is overwritten: 2/2 plus the
    // counter.
    cast_at(&mut t, P1, "Turn to Frog", &[(ManaType::U, 2)], air);
    t.resolve_all();
    assert_eq!(t.pt(air), (2, 2));
    beastbind(&mut t, air);
    assert_eq!(t.pt(air), (3, 3));
    assert!(!t.obj_now(air).has_keyword(keywords::KeywordKind::Flying));
    // It may later gain abilities (Jump: flying).
    cast_at(&mut t, P1, "Jump", &[(ManaType::U, 1)], air);
    t.resolve_all();
    assert!(t.obj_now(air).has_keyword(keywords::KeywordKind::Flying));
    // A later base-setting effect overwrites the 2/2.
    cast_at(&mut t, P1, "Turn to Frog", &[(ManaType::U, 2)], air);
    t.resolve_all();
    assert_eq!(t.pt(air), (2, 2));
}
