//! Rulings batch P113 — one-sided fights: "target creature you control deals damage equal
//! to its power to ..." spells and abilities. Illegal targets as they resolve (CR 608.2b),
//! the power used (CR 608.2h), the creature as the source of the damage, and the targets
//! needed to cast them (CR 601.2c).

use crate::r_p113_common::*;
use crate::r_s01_common::supported;
use crate::r_s02_common::destroy;
use crate::r_s07_common::{damage_on, resolved};
use crate::r_s11_common::manifest_card;
use crate::r_s25_common::{cast_new, lands_for_cost};
use mtg_engine::decision::Decision;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

/// Who is made an illegal target in a two-target bite scenario.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Bad {
    None,
    Mine(Spoil),
    Theirs(Spoil),
    Both,
}

/// P0's Hill Giant (3/3) and P1's Serra Angel (4/4 flier); `start` puts the bite on the
/// stack targeting them (in that order), `bad` happens in response, then everything
/// resolves. Returns (game, mine, theirs, spell-or-None).
fn scenario(
    start: impl Fn(&mut TestGame, ObjectId, ObjectId) -> Option<ObjectId>,
    bad: Bad,
) -> (TestGame, ObjectId, ObjectId, Option<ObjectId>) {
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Serra Angel");
    let spell = start(&mut t, mine, theirs);
    match bad {
        Bad::None => {}
        Bad::Mine(s) => spoil(&mut t, mine, s),
        Bad::Theirs(s) => spoil(&mut t, theirs, s),
        Bad::Both => {
            spoil(&mut t, mine, Spoil::Hexproof);
            spoil(&mut t, theirs, Spoil::Hexproof);
        }
    }
    t.resolve_all();
    (t, mine, theirs, spell)
}

fn spell_start(name: &'static str, modes: &'static [usize]) -> impl Fn(&mut TestGame, ObjectId, ObjectId) -> Option<ObjectId> {
    move |t, m, o| Some(cast_modes(t, P0, name, modes, &[ent(m), ent(o)]))
}

fn vivien_start(t: &mut TestGame, m: ObjectId, o: ObjectId) -> Option<ObjectId> {
    supported("Vivien of the Arkbow");
    let v = t.battlefield(P0, "Vivien of the Arkbow");
    t.activate(P0, v, 1, &[ent(m), ent(o)]).expect("-3");
    None
}

fn talent_start(t: &mut TestGame, m: ObjectId, o: ObjectId) -> Option<ObjectId> {
    supported("Hunter's Talent");
    t.answer_targets(P0, &[ent(m)]);
    t.answer_targets(P0, &[ent(o)]);
    t.enter(P0, "Hunter's Talent");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    None
}

#[test]
fn bites_deal_no_damage_if_either_target_is_illegal() {
    cr!("608.2b", "115.1");
    ruling!(
        "Tail Slash",
        "If either creature is an illegal target as Tail Slash tries to resolve, the creature you control won’t deal damage."
    );
    ruling!(
        "Vivien of the Arkbow",
        "If either creature is an illegal target as Vivien’s second ability tries to resolve, the creature you control won’t deal damage."
    );
    ruling!(
        "Assert Perfection",
        "If either target is an illegal target as Assert Perfection tries to resolve, the creature you control won't deal damage."
    );
    ruling!(
        "Tenderize",
        "If either target is an illegal target as Tenderize resolves, the creature you control won't deal damage."
    );
    ruling!(
        "Giantfall",
        "If the first mode is chosen and either target is an illegal target as Giantfall tries to resolve, the creature you control won't deal damage."
    );
    ruling!(
        "Hunter's Talent",
        "If either target is illegal as the level 1 class ability resolves, no damage will be dealt."
    );
    ruling!(
        "Wing Puncture",
        "If the first target is illegal, it can’t perform any actions, so it can’t deal damage. If the second target is illegal, there’s nothing for the first target to deal damage to."
    );
    type Start = Box<dyn Fn(&mut TestGame, ObjectId, ObjectId) -> Option<ObjectId>>;
    // (name, start, damage dealt with both targets legal)
    let cases: Vec<(&str, Start, u32)> = vec![
        ("Tail Slash", Box::new(spell_start("Tail Slash", &[])), 3),
        ("Tenderize", Box::new(spell_start("Tenderize", &[])), 3),
        ("Giantfall", Box::new(spell_start("Giantfall", &[0])), 3),
        ("Wing Puncture", Box::new(spell_start("Wing Puncture", &[])), 3),
        ("Assert Perfection", Box::new(spell_start("Assert Perfection", &[])), 4),
        ("Vivien", Box::new(vivien_start), 3),
        ("Hunter's Talent", Box::new(talent_start), 3),
    ];
    for (name, start, dmg) in &cases {
        let (t, mine, theirs, _) = scenario(start, Bad::None);
        if *dmg >= 4 {
            assert!(!t.on_battlefield(theirs), "{name}");
        } else {
            assert_eq!(damage_on(&t, theirs), *dmg, "{name}");
        }
        assert_eq!(damage_on(&t, mine), 0, "{name}: one-sided");
        for s in [Spoil::Gone, Spoil::Stolen, Spoil::Hexproof] {
            let (t, _, theirs, _) = scenario(start, Bad::Mine(s));
            assert_eq!(damage_on(&t, theirs), 0, "{name} {s:?}");
            assert!(t.on_battlefield(theirs), "{name} {s:?}");
        }
        for s in [Spoil::Gone, Spoil::Hexproof] {
            let (t, mine, theirs, _) = scenario(start, Bad::Theirs(s));
            assert_eq!(damage_on(&t, theirs), 0, "{name} {s:?}");
            assert_eq!(damage_on(&t, mine), 0, "{name} {s:?}");
            assert_eq!(t.life(P1), 20, "{name} {s:?}");
        }
    }
}

#[test]
fn with_both_targets_illegal_the_spell_or_ability_doesnt_resolve() {
    cr!("608.2b");
    ruling!(
        "Wing Puncture",
        "If either target has become illegal by the time Wing Puncture resolves, the spell will still resolve but have no effect."
    );
    ruling!(
        "Hunter's Talent",
        "If both targets are illegal, the ability won't resolve at all."
    );
    // One illegal target: Wing Puncture resolves (without effect).
    let (t, _, theirs, spell) = scenario(spell_start("Wing Puncture", &[]), Bad::Mine(Spoil::Hexproof));
    assert!(resolved(&t, spell.unwrap()));
    assert_eq!(damage_on(&t, theirs), 0);
    // Both illegal: it doesn't resolve.
    let (t, _, theirs, spell) = scenario(spell_start("Wing Puncture", &[]), Bad::Both);
    assert!(!resolved(&t, spell.unwrap()));
    assert_eq!(damage_on(&t, theirs), 0);
    assert!(t.in_graveyard(P0, "Wing Puncture"));
    // Hunter's Talent's trigger with both targets illegal is removed without effect.
    let (t, mine, theirs, _) = scenario(talent_start, Bad::Both);
    assert_eq!(damage_on(&t, theirs), 0);
    assert_eq!(damage_on(&t, mine), 0);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn bonus_then_damage_spells_with_one_illegal_target() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Rabid Gnaw",
        "If the creature you control is a legal target but the creature you don’t control isn’t as Rabid Gnaw resolves, the creature you control will get +1/+0 until end of turn, but no damage will be dealt. If instead the creature you control is an illegal target but the creature you don’t control is still a legal target, nothing will happen when Rabid Gnaw resolves."
    );
    ruling!(
        "Ambuscade",
        "If the creature you control leaves the battlefield before Ambuscade resolves, Ambuscade has no effect and no damage is dealt. If the creature an opponent controls leaves the battlefield instead, the creature you control gets +1/+0 even though it won't deal any damage."
    );
    ruling!(
        "Hunter's Edge",
        "If the creature you don't control is an illegal target as Hunter's Edge tries to resolve but the creature you control is a legal target, you just put a +1/+1 counter on the creature you control."
    );
    // (spell, Hill Giant's P/T after it with only the other target illegal)
    for (spell, pt) in [
        ("Rabid Gnaw", (4, 3)),
        ("Ambuscade", (4, 3)),
        ("Hunter's Edge", (4, 4)),
    ] {
        // The other creature leaves or gains hexproof: the bonus, no damage.
        for s in [Spoil::Gone, Spoil::Hexproof] {
            let (t, mine, theirs, _) = scenario(spell_start(spell, &[]), Bad::Theirs(s));
            assert_eq!(t.pt(mine), pt, "{spell} {s:?}");
            assert_eq!(damage_on(&t, theirs), 0, "{spell} {s:?}");
            assert_eq!(damage_on(&t, mine), 0, "{spell} {s:?}");
        }
        // The creature you control is illegal (still on the battlefield, with hexproof or
        // stolen): nothing happens.
        for s in [Spoil::Hexproof, Spoil::Stolen] {
            let (t, mine, theirs, _) = scenario(spell_start(spell, &[]), Bad::Mine(s));
            assert_eq!(t.pt(mine), (3, 3), "{spell} {s:?}");
            assert_eq!(damage_on(&t, theirs), 0, "{spell} {s:?}");
        }
        // Gone: no damage.
        let (t, _, theirs, _) = scenario(spell_start(spell, &[]), Bad::Mine(Spoil::Gone));
        assert_eq!(damage_on(&t, theirs), 0, "{spell}");
    }
}

/// Casts a spell whose first target slot is "up to/two target creatures you control" and
/// whose second is the creature dealt damage. P0 has Hill Giant (3/3) and Grizzly Bears
/// (2/2); P1 has Craw Wurm (6/4). `bad` lists which of (giant, bears, wurm) get hexproof or
/// shroud in response. Returns (game, giant, bears, wurm).
fn gang_up(spell: &str, bad: &[usize]) -> (TestGame, ObjectId, ObjectId, ObjectId) {
    supported(spell);
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    lands_for_cost(&mut t, P0, spell);
    let card = t.hand(P0, spell);
    t.answer_targets(P0, &[ent(giant), ent(bears)]);
    t.answer_targets(P0, &[ent(wurm)]);
    t.cast(P0, card).go();
    for i in bad {
        spoil(&mut t, [giant, bears, wurm][*i], Spoil::Hexproof);
    }
    t.resolve_all();
    (t, giant, bears, wurm)
}

#[test]
fn two_creatures_dealing_damage_with_one_illegal() {
    cr!("608.2b", "601.2c");
    ruling!(
        "Band Together",
        "If one of the two target creatures you control is an illegal target as Band Together resolves, the other will still deal damage equal to its power."
    );
    ruling!(
        "Band Together",
        "If the last target creature is an illegal target as Band Together resolves, or if both of the first targets are illegal targets, no creature deals or is dealt damage."
    );
    ruling!(
        "Combo Attack",
        "You can’t cast Combo Attack without targeting two creatures your team controls. If one of those creatures is an illegal target as Combo Attack resolves, the other will still deal damage equal to its power."
    );
    for spell in ["Band Together", "Combo Attack"] {
        let (t, _, _, wurm) = gang_up(spell, &[]);
        assert!(!t.on_battlefield(wurm), "{spell}: 3 + 2 damage");
        let (t, _, _, wurm) = gang_up(spell, &[0]);
        assert_eq!(damage_on(&t, wurm), 2, "{spell}: only the Bears");
        let (t, _, _, wurm) = gang_up(spell, &[1]);
        assert_eq!(damage_on(&t, wurm), 3, "{spell}: only the Giant");
    }
    for bad in [&[2usize][..], &[0, 1]] {
        let (t, giant, bears, wurm) = gang_up("Band Together", bad);
        for id in [giant, bears, wurm] {
            assert_eq!(damage_on(&t, id), 0, "{bad:?}");
        }
    }
    // Combo Attack needs two creatures your team controls.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let wurm = t.battlefield(P1, "Craw Wurm");
    lands_for_cost(&mut t, P0, "Combo Attack");
    let card = t.hand(P0, "Combo Attack");
    t.answer_targets(P0, &[ent(giant)]);
    t.answer_targets(P0, &[ent(wurm)]);
    assert!(t.cast(P0, card).try_go().is_err());
    assert_eq!(damage_on(&t, wurm), 0);
}

#[test]
fn fall_of_the_hammer_uses_the_power_as_it_resolves() {
    cr!("608.2h");
    ruling!(
        "Fall of the Hammer",
        "The amount of damage dealt is based on the first target creature’s power as Fall of the Hammer resolves."
    );
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Craw Wurm");
    cast_modes(&mut t, P0, "Fall of the Hammer", &[], &[ent(mine), ent(theirs)]);
    // In response, the Bears gets +3/+3.
    cast_new(&mut t, P0, "Giant Growth", &[ent(mine)]);
    t.resolve_all();
    assert!(!t.on_battlefield(theirs), "5 damage to the 6/4 Wurm");
}

#[test]
fn wolf_strike_deals_damage_even_if_its_day() {
    cr!("608.2c", "731.1");
    ruling!(
        "Wolf Strike",
        "The target creature you control deals damage equal to its power to the target creature you don't control even if it's day."
    );
    for (day, dmg) in [(true, 3), (false, 5)] {
        let mut t = TestGame::new(2);
        t.g.set_day(day);
        let mine = t.battlefield(P0, "Hill Giant");
        let theirs = t.battlefield(P1, "Colossal Dreadmaw");
        cast_modes(&mut t, P0, "Wolf Strike", &[], &[ent(mine), ent(theirs)]);
        t.resolve_all();
        assert_eq!(damage_on(&t, theirs), dmg, "day: {day}");
    }
}

#[test]
fn each_other_creature_uses_the_power_as_it_resolves() {
    cr!("608.2h", "608.2b");
    ruling!(
        "Chandra's Ignition",
        "Use the power of the target creature as Chandra's Ignition resolves to determine how much damage it deals to each other creature and each opponent."
    );
    ruling!(
        "Nibelheim Aflame",
        "Use the power of the target creature as Nibelheim Aflame resolves to determine how much damage it deals to each other creature."
    );
    for spell in ["Chandra's Ignition", "Nibelheim Aflame"] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Grizzly Bears");
        let theirs = t.battlefield(P1, "Craw Wurm");
        cast_modes(&mut t, P0, spell, &[], &[ent(mine)]);
        cast_new(&mut t, P0, "Giant Growth", &[ent(mine)]);
        t.resolve_all();
        assert!(!t.on_battlefield(theirs), "{spell}: 5 damage");
        if spell == "Chandra's Ignition" {
            assert_eq!(t.life(P1), 15);
        }
    }
}

#[test]
fn each_other_creature_spells_with_an_illegal_target_dont_resolve() {
    cr!("608.2b", "702.34a");
    ruling!(
        "Chandra's Ignition",
        "If the creature becomes an illegal target by the time Chandra's Ignition tries to resolve (perhaps because another player controls it or it's left the battlefield), Chandra's Ignition won't resolve and none of its effects will happen. No damage will be dealt."
    );
    ruling!(
        "Nibelheim Aflame",
        "If the target creature is an illegal target as Nibelheim Aflame tries to resolve, it won't resolve and none of its effects will happen. No damage will be dealt, and you won't discard and draw even if you cast Nibelheim Aflame from a graveyard."
    );
    for s in [Spoil::Stolen, Spoil::Gone] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Hill Giant");
        let other = t.battlefield(P1, "Grizzly Bears");
        let spell = cast_modes(&mut t, P0, "Chandra's Ignition", &[], &[ent(mine)]);
        spoil(&mut t, mine, s);
        t.resolve_all();
        assert!(!resolved(&t, spell));
        assert!(t.on_battlefield(other), "{s:?}");
        assert_eq!(t.life(P1), 20, "{s:?}");
    }
    // Nibelheim Aflame cast with flashback: no damage, no discard, no draw.
    supported("Nibelheim Aflame");
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    let other = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Mountain", 7);
    t.hand(P0, "Grizzly Bears");
    t.hand(P0, "Grizzly Bears");
    let card = t.graveyard(P0, "Nibelheim Aflame");
    t.answer_targets(P0, &[ent(mine)]);
    let spell = t
        .cast(P0, card)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    let (hand, library) = (t.hand_size(P0), t.library_size(P0));
    spoil(&mut t, mine, Spoil::Hexproof);
    t.resolve_all();
    assert!(!resolved(&t, spell));
    assert!(t.on_battlefield(other));
    assert_eq!((t.hand_size(P0), t.library_size(P0)), (hand, library));
    // With a legal target, it does discard and draw four.
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 7);
    t.hand(P0, "Grizzly Bears");
    let card = t.graveyard(P0, "Nibelheim Aflame");
    t.answer_targets(P0, &[ent(mine)]);
    t.cast(P0, card)
        .method(CastMethod::Keyword(KeywordKind::Flashback))
        .go();
    let library = t.library_size(P0);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 4);
    assert_eq!(t.library_size(P0), library - 4);
}

#[test]
fn the_creature_is_the_source_of_the_damage() {
    cr!("609.7", "702.16e", "120.1");
    ruling!(
        "Chandra's Ignition",
        "The creature is the source of the damage, not Chandra's Ignition. For example, Chandra's Ignition can have a white creature deal damage to a creature with protection from red."
    );
    ruling!(
        "Nibelheim Aflame",
        "The creature is the source of the damage, not Nibelheim Aflame. For example, Nibelheim Aflame can have a white creature deal damage to a creature with protection from red."
    );
    for spell in ["Chandra's Ignition", "Nibelheim Aflame"] {
        let mut t = TestGame::new(2);
        // Serra Angel: a white 4/4. Silver Knight: a 2/2 with protection from red.
        let angel = t.battlefield(P0, "Serra Angel");
        let knight = t.battlefield(P1, "Silver Knight");
        cast_modes(&mut t, P0, spell, &[], &[ent(angel)]);
        t.resolve_all();
        assert!(!t.on_battlefield(knight), "{spell}");
        // A red creature's damage to it is prevented.
        let mut t = TestGame::new(2);
        let giant = t.battlefield(P0, "Hill Giant");
        let knight = t.battlefield(P1, "Silver Knight");
        cast_modes(&mut t, P0, spell, &[], &[ent(giant)]);
        t.resolve_all();
        assert!(t.on_battlefield(knight), "{spell}");
        assert_eq!(damage_on(&t, knight), 0, "{spell}");
    }
}

#[test]
fn too_greedily_too_deep_a_card_that_isnt_a_creature_on_the_battlefield_deals_no_damage() {
    cr!("608.2h", "614.1a");
    ruling!(
        "Too Greedily, Too Deep",
        "If the card you return to the battlefield isn't a creature on the battlefield, it has no power and therefore deals no damage."
    );
    // Containment Priest exiles the Hill Giant instead.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Containment Priest");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.graveyard(P1, "Hill Giant");
    cast_modes(&mut t, P0, "Too Greedily, Too Deep", &[], &[ent(giant)]);
    t.resolve_all();
    assert!(t.in_exile("Hill Giant"));
    assert!(t.on_battlefield(bears));
    assert_eq!(damage_on(&t, bears), 0);
    // Without the Priest, the Giant deals 3 damage to each other creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.graveyard(P1, "Hill Giant");
    cast_modes(&mut t, P0, "Too Greedily, Too Deep", &[], &[ent(giant)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
}

#[test]
fn showstopping_surprise_with_a_face_down_creature_that_cant_be_turned_up() {
    cr!("708.2", "701.40a", "608.2h");
    ruling!(
        "Showstopping Surprise",
        "If the target creature is a face-down creature that can’t be turned face up for any reason, it will remain face down on the battlefield and still deal damage equal to its power (which is very likely to be 2) to each other creature."
    );
    let mut t = TestGame::new(2);
    // A manifested instant can't be turned face up.
    let m = manifest_card(&mut t, P0, "Shock");
    let giant = t.battlefield(P1, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_modes(&mut t, P0, "Showstopping Surprise", &[], &[ent(m)]);
    t.resolve_all();
    assert!(t.on_battlefield(m));
    assert!(t.obj_now(m).face_down);
    assert_eq!(damage_on(&t, giant), 2);
    assert!(!t.on_battlefield(bears));
}

#[test]
fn electropotence_targets_on_trigger_pays_on_resolution_and_uses_last_known_power() {
    cr!("603.3d", "608.2h", "603.5", "117.3a");
    ruling!(
        "Electropotence",
        "You target a permanent or player when the ability triggers. You decide whether or not to pay {2}{R} as the ability resolves."
    );
    ruling!(
        "Electropotence",
        "If you pay {2}{R}, the creature that entered deals damage equal to its current power to the targeted permanent or player. If it's no longer on the battlefield, its last known existence on the battlefield is checked to determine its power."
    );
    ruling!(
        "Electropotence",
        "Once you decide to pay {2}{R}, it's too late for anyone to respond"
    );
    supported("Electropotence");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Electropotence");
    t.lands(P0, "Mountain", 3);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    let giant = t.enter(P0, "Hill Giant");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let before = t.asked().len();
    // The target was chosen as the ability was put on the stack; no payment yet.
    assert!(t.asked()[..before]
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. })));
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 0);
    // The Giant leaves; the ability resolves and P0 pays.
    destroy(&mut t, giant);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 3);
    assert_eq!(t.life(P1), 17, "the Giant's last known power");
    // No new target was chosen on resolution, and no player got priority between the
    // decision to pay and the damage (the whole thing happens as the ability resolves).
    let after = t.asked()[before..].to_vec();
    assert!(!after
        .iter()
        .any(|(_, d)| matches!(d, Decision::ChooseTargets { .. } | Decision::Priority { .. })));
    // Declining to pay: no damage.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Electropotence");
    t.lands(P0, "Mountain", 3);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Hill Giant");
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
}

#[test]
fn allies_at_last_cost_reduction_doesnt_change_mana_value() {
    cr!("601.2f", "202.3", "702.41a");
    ruling!(
        "Allies at Last",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Allies at Last's first ability). The mana value of the spell is determined only by its mana cost, no matter what the total cost to cast that spell was."
    );
    supported("Allies at Last");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Hada Freeblade");
    let b = t.battlefield(P0, "Hada Freeblade");
    let wurm = t.battlefield(P1, "Craw Wurm");
    // Only {G} available: {2}{G} minus {2} for two Allies.
    t.lands(P0, "Forest", 1);
    let card = t.hand(P0, "Allies at Last");
    t.answer_targets(P0, &[ent(a), ent(b)]);
    t.answer_targets(P0, &[ent(wurm)]);
    let spell = t.cast(P0, card).go();
    assert_eq!(t.g.mana_value_of(spell), 3);
    t.resolve_all();
    assert!(resolved(&t, spell));
}

#[test]
fn bites_need_both_targets_to_be_cast() {
    cr!("601.2c", "115.1");
    ruling!(
        "Clear Shot",
        "You can't cast Clear Shot unless you choose a creature you control and a creature you don't control as targets."
    );
    ruling!(
        "Felling Blow",
        "You can't cast Felling Blow unless you choose a creature you control and a creature an opponent controls as targets."
    );
    ruling!(
        "Hunter's Edge",
        "You can't cast Hunter's Edge unless you choose both a creature you control and a creature you don't control as targets."
    );
    ruling!(
        "Knockout Maneuver",
        "You can't cast Knockout Maneuver unless you choose a creature you control and a creature an opponent controls as targets."
    );
    ruling!(
        "Finishing Move",
        "You must choose a creature you control and a creature you don’t control as targets to cast Finishing Move."
    );
    for spell in [
        "Clear Shot",
        "Felling Blow",
        "Hunter's Edge",
        "Knockout Maneuver",
        "Finishing Move",
    ] {
        supported(spell);
        for (mine, theirs) in [(true, false), (false, true), (true, true)] {
            let mut t = TestGame::new(2);
            // P0's own second creature can't stand in for the opponent's.
            let a = t.battlefield(P0, "Grizzly Bears");
            let b = t.battlefield(P0, "Hill Giant");
            let o = t.battlefield(P1, "Craw Wurm");
            if !mine {
                crate::r_s06_common::give_control(&mut t, a, P1);
                crate::r_s06_common::give_control(&mut t, b, P1);
            }
            if !theirs {
                destroy(&mut t, o);
            }
            lands_for_cost(&mut t, P0, spell);
            let card = t.hand(P0, spell);
            let first = if mine { a } else { o };
            let second = if theirs { o } else { b };
            t.answer_targets(P0, &[ent(first)]);
            t.answer_targets(P0, &[ent(second)]);
            let ok = t.cast(P0, card).try_go().is_ok();
            assert_eq!(ok, mine && theirs, "{spell} mine:{mine} theirs:{theirs}");
            if !ok {
                assert_eq!(t.zone(card), Zone::Hand(P0), "{spell}");
            }
        }
    }
}

#[test]
fn finishing_move_sticker_is_chosen_on_resolution_and_may_go_on_the_creature() {
    cr!("123.3", "608.2c", "601.2c");
    ruling!(
        "Finishing Move",
        "The nonland permanent you put a sticker on, if any, isn’t a target of the spell and is chosen as the spell resolves. It can be the creature that’s about to deal damage, but it doesn’t have to be."
    );
    use mtg_engine::stickers::{self, SheetFormat, StickerDef, StickerKind, StickerSheet};
    let mut t = TestGame::new(2);
    // P0's sticker sheet has a 5/5 power and toughness sticker.
    let sheet = StickerSheet {
        name: "Sheet".into(),
        stickers: vec![StickerDef {
            kind: StickerKind::PowerToughness(5, 5),
            ticket_cost: 1,
        }],
    };
    stickers::choose_sheets(&mut t.g, P0, vec![sheet], SheetFormat::Limited).unwrap();
    let mine = t.battlefield(P0, "Grizzly Bears");
    let theirs = t.battlefield(P1, "Craw Wurm");
    let spell = cast_modes(&mut t, P0, "Finishing Move", &[], &[ent(mine), ent(theirs)]);
    assert_eq!(
        crate::r_s25_common::targets_of(&t, spell),
        vec![ent(mine), ent(theirs)],
        "only the two creatures are targets"
    );
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[ent(mine)]);
    t.resolve_all();
    let asked: Vec<_> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| !matches!(d, Decision::Priority { .. }))
        .cloned()
        .collect();
    assert!(
        asked.iter().any(|(_, d)| matches!(
            d,
            Decision::ChooseEntities { candidates, .. } if candidates.contains(&ent(mine))
        )),
        "the sticker's permanent is chosen as it resolves"
    );
    assert!(stickers::is_stickered(&t.g, t.g.current(mine)));
    assert_eq!(t.pt(mine), (5, 5));
    assert!(!t.on_battlefield(theirs), "5 damage to the 6/4 Wurm");
}

/// P0's Hill Giant (3/3) targeted by Betrayal at the Vault, with P1's Grizzly Bears and
/// Craw Wurm (6/4) as the two other targets; `gone` (0 = the Giant, 1 = the Bears) leaves
/// the battlefield in response. Returns (game, giant, bears, wurm).
fn betrayal(gone: Option<usize>) -> (TestGame, ObjectId, ObjectId, ObjectId) {
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    supported("Betrayal at the Vault");
    lands_for_cost(&mut t, P0, "Betrayal at the Vault");
    let card = t.hand(P0, "Betrayal at the Vault");
    t.answer_targets(P0, &[ent(giant)]);
    t.answer_targets(P0, &[ent(bears), ent(wurm)]);
    t.cast(P0, card).go();
    if let Some(i) = gone {
        destroy(&mut t, [giant, bears][i]);
    }
    t.resolve_all();
    (t, giant, bears, wurm)
}

#[test]
fn betrayal_at_the_vault_with_a_target_gone() {
    cr!("608.2b", "601.2c", "115.3");
    ruling!(
        "Betrayal at the Vault",
        "If one of the two other target creatures leaves the battlefield before Betrayal at the Vault resolves, the creature you control will still deal damage to the remaining other target creature."
    );
    ruling!(
        "Betrayal at the Vault",
        "If the creature you control leaves the battlefield before Betrayal at the Vault resolves, Betrayal at the Vault has no effect and no damage is dealt."
    );
    let (t, giant, bears, wurm) = betrayal(None);
    assert!(!t.on_battlefield(bears));
    assert_eq!(damage_on(&t, wurm), 3);
    assert_eq!(damage_on(&t, giant), 0);
    let (t, _, _, wurm) = betrayal(Some(1));
    assert_eq!(damage_on(&t, wurm), 3);
    let (t, _, bears, wurm) = betrayal(Some(0));
    assert!(t.on_battlefield(bears));
    assert_eq!(damage_on(&t, bears), 0);
    assert_eq!(damage_on(&t, wurm), 0);
    // Exactly two other targets: not the creature you control, and not just one.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    lands_for_cost(&mut t, P0, "Betrayal at the Vault");
    let card = t.hand(P0, "Betrayal at the Vault");
    t.answer_targets(P0, &[ent(giant)]);
    t.answer_targets(P0, &[ent(bears), ent(giant)]);
    assert!(t.cast(P0, card).try_go().is_err());
    assert_eq!(t.zone(card), Zone::Hand(P0));
    t.answer_targets(P0, &[ent(giant)]);
    t.answer_targets(P0, &[ent(bears)]);
    assert!(t.cast(P0, card).try_go().is_err(), "only one other target");
}

#[test]
fn spinning_wheel_kick_deals_damage_to_each_of_x_targets() {
    cr!("601.2c", "107.3a");
    supported("Spinning Wheel Kick");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    let jace = t.battlefield(P1, "Jace Beleren");
    // X = 3: {3}{3}{G}{G}.
    t.lands(P0, "Forest", 8);
    let card = t.hand(P0, "Spinning Wheel Kick");
    t.answer_targets(P0, &[ent(giant)]);
    t.answer_targets(P0, &[ent(bears), ent(wurm), ent(jace)]);
    t.cast(P0, card).x(3).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(damage_on(&t, wurm), 3);
    assert!(!t.on_battlefield(jace), "3 damage to a 3-loyalty Jace");
    assert_eq!(damage_on(&t, giant), 0);
    // X = 4 needs exactly four different targets: with only three creatures, it can't be
    // cast.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.lands(P0, "Forest", 10);
    let card = t.hand(P0, "Spinning Wheel Kick");
    t.answer_targets(P0, &[ent(giant)]);
    t.answer_targets(P0, &[ent(bears), ent(wurm), ent(giant)]);
    assert!(t.cast(P0, card).x(4).try_go().is_err());
    assert_eq!(t.zone(card), Zone::Hand(P0));
}
