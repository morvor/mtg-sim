//! Rulings batch P054 — targets of pump spells and abilities: a spell or ability whose
//! targets are all illegal doesn't resolve and none of its effects happen (CR 608.2b),
//! partial illegality, "up to one" targets, choices made on resolution rather than on
//! casting (CR 601.2c vs 608.2), and abilities that resolve once activated whatever
//! happens to their activation condition or source (CR 113.7a, 602.5).

use crate::r_p054_common::*;
use crate::r_s02_common::{create_token, target_candidates};
use crate::r_s06_common::{attach_new, give_control};
use crate::r_s07_common::damage_on;
use crate::r_s15_common::tokens_created_by;
use crate::r_s17_common::transform;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn energy(t: &TestGame, p: PlayerId) -> u32 {
    t.g.player(p).counter("energy")
}

fn foods(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents()
        .filter(|o| o.controller == p && o.chars.has_subtype("Food"))
        .count()
}

#[test]
fn a_fizzled_pump_spell_has_none_of_its_other_effects() {
    cr!("608.2b");
    ruling!(
        "Highspire Infusion",
        "If the target creature becomes an illegal target, the spell doesn't resolve and none of its effects happen. You won't get {E}{E}."
    );
    ruling!(
        "Thoughtweft Charge",
        "If the target creature is an illegal target as Thoughtweft Charge tries to resolve, it won't resolve and none of its effects will happen. You won't draw a card."
    );
    ruling!(
        "Strength of the Coalition",
        "If the target creature is an illegal target as Strength of the Coalition tries to resolve, none of its effects will occur. You won't put a +1/+1 counter on any creatures, even if it was kicked."
    );
    // Highspire Infusion: the target leaves.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Highspire Infusion", bear);
    move_to(&mut t, bear, Zone::Exile);
    t.resolve_all();
    assert_eq!(energy(&t, P0), 0);
    // Thoughtweft Charge: a creature entered this turn, but the target leaves.
    let mut t = TestGame::new(2);
    t.library_top(P0, "Island");
    enter(&mut t, P0, "Grizzly Bears");
    let bear = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Thoughtweft Charge", bear);
    move_to(&mut t, bear, Zone::Exile);
    let hand = t.hand_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand, "no card drawn");
    // Control: it does draw when it resolves.
    let bear = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Thoughtweft Charge", bear);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 1);
    // Strength of the Coalition, kicked: "target creature you control" changes control.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    let s = t.hand(P0, "Strength of the Coalition");
    pool(&mut t, P0);
    t.cast(P0, s).target(bear).kicked(true).go();
    give_control(&mut t, bear, P1);
    t.resolve_all();
    assert_eq!(t.counters(other, "+1/+1"), 0);
    assert_eq!(t.counters(bear, "+1/+1"), 0);
    assert_eq!(t.pt(bear), (2, 2));
}

#[test]
fn strength_of_the_coalition_kicked_gives_both_the_counter_and_the_bonus() {
    cr!("702.33d", "608.2c");
    ruling!(
        "Strength of the Coalition",
        "If the spell is kicked, the target creature will get both a +1/+1 counter and +2/+2 until end of turn."
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Hill Giant");
    let s = t.hand(P0, "Strength of the Coalition");
    pool(&mut t, P0);
    t.cast(P0, s).target(bear).kicked(true).go();
    t.resolve_all();
    assert_eq!(t.counters(bear, "+1/+1"), 1);
    assert_eq!(t.pt(bear), (5, 5));
    assert_eq!(t.pt(other), (4, 4));
}

/// Insatiable Appetite / Pippin's Bravery: the target is chosen on casting, the Food is
/// sacrificed (or not) on resolution, and a fizzled spell sacrifices nothing.
#[test]
fn food_pumps_choose_the_sacrifice_on_resolution_and_not_when_fizzled() {
    cr!("601.2c", "608.2b", "608.2c");
    ruling!(
        "Insatiable Appetite",
        "If the target creature is an illegal target by the time Insatiable Appetite tries to resolve, the spell won't resolve. You won't sacrifice a Food."
    );
    ruling!(
        "Pippin's Bravery",
        "If the target creature is an illegal target by the time Pippin's Bravery tries to resolve, Pippin's Bravery is removed from the stack. You won't be able to sacrifice a Food to it."
    );
    ruling!(
        "Insatiable Appetite",
        "You choose a target creature as you cast Insatiable Appetite, but you don't choose whether to sacrifice a Food until the spell resolves."
    );
    ruling!(
        "Pippin's Bravery",
        "You choose a target creature as you cast Pippin's Bravery, but you don't choose whether to sacrifice a Food until the spell resolves."
    );
    for (name, big, small) in [("Insatiable Appetite", 5, 3), ("Pippin's Bravery", 4, 2)] {
        // Fizzled: the Food stays.
        let mut t = TestGame::new(2);
        create_token(&mut t, P0, "Food");
        let bear = t.battlefield(P0, "Grizzly Bears");
        t.answer_yes(P0, true);
        cast_at(&mut t, P0, name, bear);
        assert_eq!(foods(&t, P0), 1, "{name}: nothing sacrificed on casting");
        move_to(&mut t, bear, Zone::Exile);
        t.resolve_all();
        assert_eq!(foods(&t, P0), 1, "{name}: fizzled");
        // Resolving: a Food created after casting can be sacrificed.
        let mut t = TestGame::new(2);
        let bear = t.battlefield(P0, "Grizzly Bears");
        cast_at(&mut t, P0, name, bear);
        create_token(&mut t, P0, "Food");
        t.answer_yes(P0, true);
        t.resolve_all();
        assert_eq!(foods(&t, P0), 0, "{name}: sacrificed on resolution");
        assert_eq!(t.pt(bear), (2 + big, 2 + big), "{name}");
        // Declining: the smaller bonus.
        let mut t = TestGame::new(2);
        create_token(&mut t, P0, "Food");
        let bear = t.battlefield(P0, "Grizzly Bears");
        cast_at(&mut t, P0, name, bear);
        t.answer_yes(P0, false);
        t.resolve_all();
        assert_eq!(foods(&t, P0), 1, "{name}");
        assert_eq!(t.pt(bear), (2 + small, 2 + small), "{name}");
    }
}

#[test]
fn pippins_bravery_can_sacrifice_a_nontoken_food() {
    cr!("205.3g", "111.10b");
    ruling!(
        "Pippin's Bravery",
        "If an effect refers to a Food, it means any Food artifact, not just a Food artifact token. For example, you can sacrifice Lembas to Pippin's Bravery."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Lembas");
    let bear = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Pippin's Bravery", bear);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.named_on_battlefield("Lembas").is_empty());
    assert_eq!(t.pt(bear), (6, 6));
}

/// Primal Might: the pump needs only the creature you control; the fight needs both.
#[test]
fn primal_might_with_illegal_or_missing_targets() {
    cr!("608.2b", "701.14b", "115.1");
    ruling!(
        "Primal Might",
        "If the creature you control is an illegal target as Primal Might tries to resolve, no creature gets +X/+X. If that creature is a legal target but the creature you don't control isn't, it still gets +X/+X."
    );
    ruling!(
        "Primal Might",
        "If you choose two target creatures and either target is an illegal target as Primal Might tries to resolve, neither creature will deal or be dealt damage."
    );
    ruling!(
        "Primal Might",
        "You can cast Primal Might targeting only the creature you control."
    );
    let primal = |t: &mut TestGame, mine: ObjectId, theirs: Option<ObjectId>| {
        let second: Vec<Entity> = theirs.map(o).into_iter().collect();
        cast_how(
            t,
            P0,
            "Primal Might",
            CastMethod::Normal,
            &[&[o(mine)], &second],
            Some(2),
            None,
        )
    };
    // Only the creature you control.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    primal(&mut t, bear, None);
    t.resolve_all();
    assert_eq!(t.pt(bear), (4, 4));
    // Its own creature becomes illegal (changes control): no pump, no fight.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    primal(&mut t, bear, Some(giant));
    give_control(&mut t, bear, P1);
    t.resolve_all();
    assert_eq!(t.pt(bear), (2, 2));
    assert_eq!(damage_on(&t, bear), 0);
    assert_eq!(damage_on(&t, giant), 0);
    // The other creature becomes illegal (gains hexproof... leaves): pump, no fight.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let other = t.battlefield(P1, "Grizzly Bears");
    primal(&mut t, bear, Some(giant));
    give_control(&mut t, giant, P0);
    t.resolve_all();
    assert_eq!(t.pt(bear), (4, 4));
    assert_eq!(damage_on(&t, bear), 0);
    assert_eq!(damage_on(&t, giant), 0);
    assert_eq!(damage_on(&t, other), 0);
}

#[test]
fn razorgrass_invoker_with_no_target_or_an_illegal_target() {
    cr!("608.2b", "115.1");
    ruling!(
        "Razorgrass Invoker",
        "You don't have to choose a target for Razorgrass Invoker's last ability. However, if you do and the target creature is an illegal target as the ability tries to resolve, it won't resolve and none of its effects will happen. Razorgrass Invoker won't get +3/+3."
    );
    let mut t = TestGame::new(2);
    let inv = t.battlefield(P0, "Razorgrass Invoker");
    pool(&mut t, P0);
    t.activate(P0, inv, 0, &[]).expect("no target");
    t.resolve_all();
    assert_eq!(t.pt(inv), (7, 6));
    let mut t = TestGame::new(2);
    let inv = t.battlefield(P0, "Razorgrass Invoker");
    let bear = t.battlefield(P0, "Grizzly Bears");
    pool(&mut t, P0);
    t.activate(P0, inv, 0, &[o(bear)]).expect("a target");
    move_to(&mut t, bear, Zone::Exile);
    t.resolve_all();
    assert_eq!(t.pt(inv), (4, 3));
}

#[test]
fn authority_without_targets_or_with_all_targets_illegal() {
    cr!("608.2b", "702.127a");
    ruling!(
        "Appeal // Authority",
        "You may cast Authority without choosing any target creatures. Creatures you control will still gain vigilance until end of turn. However, if you choose any targets and each of those targets become illegal before Authority resolves, the spell won’t resolve and your creatures won’t gain vigilance."
    );
    let authority = |t: &mut TestGame, targets: &[Entity]| {
        crate::r_s01_common::supported("Appeal // Authority");
        pool(t, P0);
        let c = t.graveyard(P0, "Appeal // Authority");
        t.answer_targets(P0, targets);
        t.cast(P0, c)
            .method(CastMethod::Keyword(KeywordKind::Aftermath))
            .go();
    };
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    authority(&mut t, &[]);
    t.resolve_all();
    assert!(t.obj_now(bear).has_keyword(KeywordKind::Vigilance));
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Grizzly Bears");
    authority(&mut t, &[o(theirs)]);
    move_to(&mut t, theirs, Zone::Exile);
    t.resolve_all();
    assert!(!t.obj_now(bear).has_keyword(KeywordKind::Vigilance));
}

#[test]
fn ulrich_alphas_fight_trigger_with_an_illegal_target_or_ulrich_gone() {
    cr!("608.2b", "701.14b");
    ruling!(
        "Ulrich of the Krallenhorde // Ulrich, Uncontested Alpha",
        "If the targeted creature is an illegal target when Ulrich, Uncontested Alpha's first ability tries to resolve, the ability doesn't resolve."
    );
    ruling!(
        "Ulrich of the Krallenhorde // Ulrich, Uncontested Alpha",
        "If Ulrich is no longer on the battlefield as Ulrich, Uncontested Alpha's first ability resolves, the targeted creature won't deal or be dealt damage."
    );
    let setup = |t: &mut TestGame| -> (ObjectId, ObjectId) {
        crate::r_s01_common::supported("Ulrich of the Krallenhorde // Ulrich, Uncontested Alpha");
        let u = t.battlefield(
            P0,
            "Ulrich of the Krallenhorde // Ulrich, Uncontested Alpha",
        );
        let bear = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[o(bear)]);
        t.answer_yes(P0, true);
        transform(t, u);
        assert_eq!(t.stack_len(), 1, "the fight trigger");
        (u, bear)
    };
    // Control: it fights.
    let mut t = TestGame::new(2);
    let (_u, bear) = setup(&mut t);
    t.resolve_all();
    assert!(!t.on_battlefield(bear));
    // The target becomes illegal (comes under P0's control): no fight.
    let mut t = TestGame::new(2);
    let (u, bear) = setup(&mut t);
    give_control(&mut t, bear, P0);
    t.resolve_all();
    assert_eq!(damage_on(&t, bear), 0);
    assert_eq!(damage_on(&t, u), 0);
    // Ulrich leaves: the target isn't dealt damage.
    let mut t = TestGame::new(2);
    let (u, bear) = setup(&mut t);
    move_to(&mut t, u, Zone::Exile);
    t.resolve_all();
    assert_eq!(damage_on(&t, bear), 0);
}

#[test]
fn ulrich_alpha_transforms_only_if_a_single_player_cast_two_spells() {
    cr!("701.27a");
    ruling!(
        "Ulrich of the Krallenhorde // Ulrich, Uncontested Alpha",
        "Ulrich, Uncontested Alpha's last ability only triggers if a single player has cast two or more spells during the previous turn. If multiple players each cast just one spell during the previous turn, the ability won't trigger."
    );
    for (p0_spells, p1_spells, flips) in [(1, 1, false), (2, 0, true), (0, 2, true)] {
        let mut t = TestGame::new(2);
        let u = t.battlefield(
            P0,
            "Ulrich of the Krallenhorde // Ulrich, Uncontested Alpha",
        );
        transform(&mut t, u);
        t.clear_answers();
        t.resolve_all(); // the (optional) fight trigger: declined by default
        let name = |t: &TestGame| t.obj_now(u).chars.name.to_string();
        assert_eq!(name(&t), "Ulrich, Uncontested Alpha");
        let bear = t.battlefield(P0, "Grizzly Bears");
        for (p, n) in [(P0, p0_spells), (P1, p1_spells)] {
            for _ in 0..n {
                cast_at(&mut t, p, "Giant Growth", bear);
                t.resolve_all();
            }
        }
        t.advance_to(P1, Step::Upkeep);
        t.resolve_all();
        let flipped = name(&t) == "Ulrich of the Krallenhorde";
        assert_eq!(flipped, flips, "{p0_spells}/{p1_spells}");
    }
}

#[test]
fn angelic_destiny_fizzles_to_the_graveyard_if_its_creature_dies_first() {
    cr!("608.2b", "303.4a");
    ruling!(
        "Angelic Destiny",
        "If the creature dies before the Angelic Destiny spell resolves, Angelic Destiny will go to its owner's graveyard. It won't return to its owner's hand."
    );
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Angelic Destiny", bear);
    move_to(&mut t, bear, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Angelic Destiny"));
    assert!(!t.in_hand(P0, "Angelic Destiny"));
}

#[test]
fn arm_the_cathars_targets_must_be_different_creatures() {
    cr!("115.3", "601.2c");
    ruling!(
        "Arm the Cathars",
        "If you choose three targets as you cast Arm the Cathars, all three targets must be different creatures."
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let mark = t.asked().len();
    // Trying to target the bear three times: only the first slot takes it.
    cast_how(
        &mut t,
        P0,
        "Arm the Cathars",
        CastMethod::Normal,
        &[&[o(a)], &[o(a)], &[o(a)]],
        None,
        None,
    );
    let cands = target_candidates(&t, P0, mark);
    assert!(cands.len() >= 2);
    assert!(cands[1].contains(&o(b)) && !cands[1].contains(&o(a)));
    t.resolve_all();
    assert_eq!(t.pt(a), (5, 5));
    assert_eq!(t.pt(b), (3, 3), "the bear can't get the other bonuses");
}

/// An activated ability resolves once activated even if its activation condition no
/// longer holds (CR 602.5b only restricts activation) and even if the ability's source
/// loses it (CR 113.7a).
#[test]
fn abilities_resolve_even_if_their_activation_condition_stops_holding() {
    cr!("602.5b", "113.7a");
    ruling!(
        "Greenhilt Trainee",
        "Once Greenhilt Trainee’s ability is activated, it will resolve as normal even if Greenhilt Trainee’s power is less than 4 at that time (perhaps because an Equipment attached to it was destroyed)."
    );
    ruling!(
        "Vigilant Sentry",
        "Once you activate the ability Vigilant Sentry gives itself, it doesn’t matter what happens to the number of cards in your graveyard. That ability will still resolve even if that number drops below seven in response."
    );
    // Greenhilt Trainee pumped by an Equipment that is then destroyed.
    let mut t = TestGame::new(2);
    let gt = t.battlefield(P0, "Greenhilt Trainee");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let sword = attach_new(&mut t, P0, "Bonesplitter", gt);
    assert!(t.activate(P0, gt, 0, &[o(bear)]).is_ok());
    move_to(&mut t, sword, Zone::Graveyard(P0));
    assert_eq!(t.pt(gt), (2, 3));
    t.resolve_all();
    assert_eq!(t.pt(bear), (6, 6));
    // Vigilant Sentry: the graveyard drops below seven in response.
    let mut t = TestGame::new(2);
    let gy: Vec<_> = (0..7).map(|_| t.graveyard(P0, "Island")).collect();
    let vs = t.battlefield(P0, "Vigilant Sentry");
    let bear = t.battlefield(P0, "Grizzly Bears");
    crate::r_s01_common::attack_with(&mut t, &[(bear, pl(P1))]);
    assert!(t.activate(P0, vs, 0, &[o(bear)]).is_ok());
    move_to(&mut t, gy[0], Zone::Exile);
    t.resolve_all();
    assert_eq!(t.pt(bear), (5, 5));
}

#[test]
fn batterskull_stays_where_it_went_if_it_left_before_its_return_ability_resolved() {
    cr!("400.7", "602.1");
    ruling!(
        "Batterskull",
        "The ability to return Batterskull to its owner's hand can be activated only if Batterskull is on the battlefield. If Batterskull is no longer on the battlefield when the ability resolves, Batterskull remains in its new zone and isn't returned to its owner's hand."
    );
    let mut t = TestGame::new(2);
    let bs = t.battlefield(P0, "Batterskull");
    pool(&mut t, P0);
    let idx = t
        .obj_now(bs)
        .chars
        .abilities
        .iter()
        .filter(|a| matches!(a.kind, mtg_engine::ability::AbilityKind::Activated(_)))
        .position(|a| a.text.contains("Return"))
        .unwrap();
    t.activate(P0, bs, idx, &[]).expect("return");
    move_to(&mut t, bs, Zone::Graveyard(P0));
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Batterskull"));
    assert!(!t.in_hand(P0, "Batterskull"));
    // In the graveyard it can't be activated.
    let gy = t.g.current(bs);
    pool(&mut t, P0);
    assert!(t.activate(P0, gy, idx, &[]).is_err());
}

#[test]
fn garruk_with_two_loyalty_dies_before_his_minus_two_resolves() {
    cr!("704.5i", "606.4");
    ruling!(
        "Garruk, Unleashed",
        "If Garruk has only two loyalty counters, activating his second loyalty ability results in him dying before it resolves. He won't be there to get a loyalty counter."
    );
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Garruk, Unleashed");
    let have = t.counters(g, "loyalty");
    if have > 2 {
        t.g.remove_counters(o(g), "loyalty", have - 2);
    } else if have < 2 {
        t.g.add_counters(o(g), "loyalty", 2 - have, None);
    }
    for _ in 0..3 {
        t.battlefield(P1, "Grizzly Bears");
    }
    t.activate(P0, g, 1, &[]).expect("-2");
    t.settle();
    assert!(t.in_graveyard(P0, "Garruk, Unleashed"));
    t.resolve_all();
    assert_eq!(tokens_created_by(&t, P0), 1, "the Beast is still created");
    assert!(t.in_graveyard(P0, "Garruk, Unleashed"));
}

/// Cards whose rulings say they can target an untapped (or already tapped) creature, or a
/// creature with a matching subtype or keyword.
#[test]
fn targets_that_are_legal_though_the_effect_seems_to_assume_otherwise() {
    cr!("115.1", "205.3m", "702.14c");
    ruling!(
        "Gerrard's Command",
        "This can target a creature which is already untapped."
    );
    ruling!(
        "Raggadragga, Goreguts Boss",
        "Raggadragga's last ability can target any creature, including one that's untapped."
    );
    ruling!(
        "Assembly-Worker",
        "Assembly-Worker's ability can target any creature with the Assembly-Worker subtype, not only creatures named Assembly-Worker."
    );
    // Gerrard's Command on an untapped creature.
    let mut t = TestGame::new(2);
    let bear = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Gerrard's Command", bear);
    t.resolve_all();
    assert_eq!(t.pt(bear), (5, 5));
    // Raggadragga: a seven-mana spell untaps an untapped creature and pumps it.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Raggadragga, Goreguts Boss");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let mark = t.asked().len();
    t.answer_targets(P0, &[o(bear)]);
    cast_how(
        &mut t,
        P0,
        "Vitalizing Wind",
        CastMethod::Normal,
        &[],
        None,
        None,
    );
    t.settle();
    assert!(target_candidates(&t, P0, mark)
        .iter()
        .any(|c| c.contains(&o(bear))));
    t.resolve_all();
    assert_eq!(t.pt(bear), (16, 16));
    // Assembly-Worker: a changeling is an Assembly-Worker; a Grizzly Bears isn't.
    let mut t = TestGame::new(2);
    let aw = t.battlefield(P0, "Assembly-Worker");
    let ch = t.battlefield(P0, "Woodland Changeling");
    let bear = t.battlefield(P0, "Grizzly Bears");
    let mark = t.asked().len();
    t.activate(P0, aw, 0, &[o(ch)])
        .expect("target the changeling");
    let cands = target_candidates(&t, P0, mark);
    assert!(cands[0].contains(&o(ch)) && cands[0].contains(&o(aw)));
    assert!(!cands[0].contains(&o(bear)));
    t.resolve_all();
    assert_eq!(t.pt(ch), (3, 3));
}

/// Shielding Plax / Vines of Vastwood aren't hexproof: on an opponent's creature, that
/// opponent can't target their own creature.
#[test]
fn opponent_targeting_protection_on_an_opponents_creature_stops_its_controller() {
    cr!("115.1", "702.11b");
    ruling!(
        "Shielding Plax",
        "This is not the same as hexproof. If, for example, you target one of your opponent's creatures, your opponents won't be able to target their own creature with spells or abilities."
    );
    ruling!(
        "Vines of Vastwood",
        "This is not the same as hexproof. If, for example, you target one of your opponent's creatures, your opponents won't be able to target their own creature with spells or abilities."
    );
    for name in ["Shielding Plax", "Vines of Vastwood"] {
        let mut t = TestGame::new(2);
        t.library_top(P0, "Island");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        cast_at(&mut t, P0, name, theirs);
        t.resolve_all();
        // P0 still can.
        let mark = t.asked().len();
        let gg = t.hand(P1, "Giant Growth");
        pool(&mut t, P1);
        t.answer_targets(P1, &[o(theirs)]);
        let r = t.cast(P1, gg).try_go();
        let cands = target_candidates(&t, P1, mark);
        assert!(
            r.is_err() || cands.iter().all(|c| !c.contains(&o(theirs))),
            "{name}: P1 can't target their own creature"
        );
        t.resolve_all();
        assert_eq!(t.pt(theirs), (2, 2), "{name}");
    }
}
