//! CR 115: targets.

use super::r114_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::mana::ManaType;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Adds plenty of mana of each type to `p`'s pool.
fn lots(t: &mut TestGame, p: PlayerId) {
    for ty in [
        ManaType::W,
        ManaType::U,
        ManaType::B,
        ManaType::R,
        ManaType::G,
        ManaType::C,
    ] {
        t.g.players[p.idx()].mana_pool.add_type(ty, 3);
    }
}

fn targets_of(t: &TestGame, id: ObjectId) -> Vec<Entity> {
    t.obj(id)
        .stack
        .as_ref()
        .unwrap()
        .chosen
        .iter()
        .flat_map(|cm| cm.targets.iter().flatten().copied())
        .collect()
}

/// The candidates offered by the last target choice `p` was asked to make.
fn last_candidates(t: &TestGame, p: PlayerId) -> Vec<Entity> {
    t.asked()
        .into_iter()
        .rev()
        .find_map(|(q, d)| match d {
            Decision::ChooseTargets { candidates, .. } if q == p => Some(candidates),
            _ => None,
        })
        .expect("no target choice")
}

fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

#[test]
fn targets_are_chosen_as_a_spell_is_cast_and_make_it_targeted() {
    cr!("115.1", "115.1a");
    let mut t = TestGame::new(2);
    // Heroic: "Whenever you cast a spell that targets this creature, put two +1/+1
    // counters on it."
    let warrior = t.battlefield(P0, "Staunch-Hearted Warrior");
    lots(&mut t, P0);
    let growth = t.hand(P0, "Giant Growth");
    let s = t.cast(P0, growth).target(warrior).go();
    // The target was declared as part of casting it.
    assert_eq!(targets_of(&t, s), vec![o(warrior)]);
    t.resolve_all();
    assert_eq!(t.counters(warrior, counters::PLUS1), 2);
    // A spell that doesn't say "target" isn't targeted, even if it affects the creature.
    let mut t = TestGame::new(2);
    let warrior = t.battlefield(P0, "Staunch-Hearted Warrior");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(warrior, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::DeclareBlockers);
    lots(&mut t, P0);
    let blast = t.hand(P0, "Trumpet Blast");
    let s = t.cast(P0, blast).go();
    assert!(targets_of(&t, s).is_empty());
    t.resolve_all();
    assert_eq!(t.counters(warrior, counters::PLUS1), 0);
    assert_eq!(t.pt(warrior), (4, 2));
}

#[test]
fn aura_spells_are_targeted_but_aura_permanents_are_not() {
    cr!("115.1b");
    let mut t = TestGame::new(2);
    let warrior = t.battlefield(P0, "Staunch-Hearted Warrior");
    lots(&mut t, P0);
    let pacifism = t.hand(P0, "Pacifism");
    let s = t.cast(P0, pacifism).target(warrior).go();
    assert_eq!(targets_of(&t, s), vec![o(warrior)]);
    t.resolve_all();
    // The Aura spell targeted the creature: heroic triggered.
    assert_eq!(t.counters(warrior, counters::PLUS1), 2);
    let aura = t.named_on_battlefield("Pacifism")[0];
    assert_eq!(t.obj(aura).attached_to, Some(o(warrior)));
    // The creature gains shroud. The Aura permanent doesn't target it, so it stays.
    let greaves = t.battlefield(P0, "Lightning Greaves");
    t.activate(P0, greaves, 0, &[o(warrior)]).unwrap();
    t.resolve_all();
    assert!(t
        .obj(warrior)
        .has_keyword(mtg_engine::keywords::KeywordKind::Shroud));
    assert_eq!(t.obj(aura).attached_to, Some(o(warrior)));
    assert!(t.on_battlefield(aura));
}

#[test]
fn activated_abilities_choose_targets_as_they_are_activated() {
    cr!("115.1c");
    ruling!(
        "Phantasmal Bear",
        "its ability triggers and goes on the stack on top of that spell or ability"
    );
    let mut t = TestGame::new(2);
    let pyro = t.battlefield(P0, "Prodigal Pyromancer");
    // "When this creature becomes the target of a spell or ability, sacrifice it."
    t.battlefield(P1, "Phantasmal Bear");
    let bear = t.named_on_battlefield("Phantasmal Bear")[0];
    let ab = t.activate(P0, pyro, 0, &[o(bear)]).unwrap().unwrap();
    assert_eq!(targets_of(&t, ab), vec![o(bear)]);
    // The bear's trigger resolves first: it's gone before the ability resolves.
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Phantasmal Bear"));
    assert_eq!(t.life(P1), 20);
}

#[test]
fn triggered_abilities_choose_targets_as_they_are_put_on_the_stack() {
    cr!("115.1d");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Phantasmal Bear");
    let bear = t.named_on_battlefield("Phantasmal Bear")[0];
    t.answer_targets(P0, &[o(bear)]);
    t.enter(P0, "Flametongue Kavu");
    t.settle();
    // The Kavu's trigger is on the stack with its target; the bear's trigger is above it.
    assert_eq!(t.stack_len(), 2);
    let kavu_trigger = t.stack[0];
    assert_eq!(targets_of(&t, kavu_trigger), vec![o(bear)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Phantasmal Bear"));
}

#[test]
fn keyword_abilities_such_as_equip_are_targeted() {
    cr!("115.1e");
    let mut t = TestGame::new(2);
    let greaves = t.battlefield(P0, "Lightning Greaves");
    t.battlefield(P0, "Phantasmal Bear");
    let bear = t.named_on_battlefield("Phantasmal Bear")[0];
    // Equip is a targeted activated ability: the bear becomes its target.
    t.activate(P0, greaves, 0, &[o(bear)]).unwrap();
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Phantasmal Bear"));
}

#[test]
fn only_permanents_are_legal_targets_unless_the_spell_says_otherwise() {
    cr!("115.2");
    let mut t = TestGame::new(2);
    let dead = t.graveyard(P0, "Grizzly Bears");
    let in_hand = t.hand(P0, "Hill Giant");
    lots(&mut t, P0);
    let shock = t.hand(P0, "Shock");
    t.cast(P0, shock).target(P1).go();
    let cands = last_candidates(&t, P0);
    assert!(!cands.contains(&o(dead)) && !cands.contains(&o(in_hand)));
    t.resolve_all();
    // "Target creature card from your graveyard" can target the card in the graveyard.
    lots(&mut t, P0);
    let raise = t.hand(P0, "Raise Dead");
    t.cast(P0, raise).target(dead).go();
    t.resolve_all();
    assert!(t.in_hand(P0, "Grizzly Bears"));
}

#[test]
fn the_same_target_cant_be_chosen_twice_for_one_instance_of_target() {
    cr!("115.3");
    ruling!(
        "Seeds of Strength",
        "You may choose the same creature as a target multiple times"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // "Two target creatures each get +2/+2": two different creatures are needed.
    lots(&mut t, P0);
    let symbiosis = t.hand(P0, "Symbiosis");
    assert!(t
        .cast(P0, symbiosis)
        .targets(&[o(bears), o(bears)])
        .try_go()
        .is_err());
    t.clear_answers();
    // Three instances of the word "target": the same creature can be chosen for each.
    lots(&mut t, P0);
    let seeds = t.hand(P0, "Seeds of Strength");
    t.answer_targets(P0, &[o(bears)]);
    t.answer_targets(P0, &[o(bears)]);
    let s = t.cast(P0, seeds).target(bears).go();
    assert_eq!(targets_of(&t, s), vec![o(bears); 3]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (5, 5));
}

#[test]
fn any_target_is_a_creature_player_planeswalker_or_battle() {
    cr!("115.4");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    let battle = t.battlefield(P1, "Invasion of Tarkir");
    let ring = t.battlefield(P1, "Sol Ring");
    let enchantment = t.battlefield(P1, "Pacifism");
    lots(&mut t, P0);
    let first = t.hand(P0, "Shock");
    let s1 = t.cast(P0, first).target(P1).go();
    let second = t.hand(P0, "Shock");
    t.cast(P0, second).target(P1).go();
    let cands = last_candidates(&t, P0);
    for e in [
        o(bears),
        o(jace),
        o(battle),
        Entity::Player(P0),
        Entity::Player(P1),
    ] {
        assert!(cands.contains(&e), "{e:?} should be a legal target");
    }
    // Noncreature artifacts, enchantments and spells can't be chosen.
    for e in [o(ring), o(enchantment), o(s1)] {
        assert!(!cands.contains(&e), "{e:?} shouldn't be a legal target");
    }
}

#[test]
fn a_spell_is_an_illegal_target_for_itself() {
    cr!("115.5");
    ruling!("Redirect", "you can’t change that spell’s target to itself");
    let mut t = TestGame::new(2);
    // Nothing else on the stack: Counterspell has no legal target.
    lots(&mut t, P0);
    let counterspell = t.hand(P0, "Counterspell");
    assert!(t.cast(P0, counterspell).try_go().is_err());
    // When choosing new targets for a spell, it can't become its own target.
    let mut t = TestGame::new(2);
    lots(&mut t, P1);
    let shock = t.hand(P1, "Shock");
    t.set_step(P1, Step::PrecombatMain);
    let shock = t.cast(P1, shock).target(P0).go();
    lots(&mut t, P0);
    let counterspell = t.hand(P0, "Counterspell");
    let cs = t.cast(P0, counterspell).target(shock).go();
    lots(&mut t, P1);
    let redirect = t.hand(P1, "Redirect");
    t.cast(P1, redirect).target(cs).go();
    t.answer_yes(P1, true);
    t.resolve();
    let cands = last_candidates(&t, P1);
    assert!(!cands.contains(&o(cs)));
}

#[test]
fn a_spell_may_allow_zero_targets_and_then_isnt_targeted() {
    cr!("115.6");
    ruling!(
        "Plunge into Winter",
        "You can cast Plunge into Winter without a target just to scry 1 and draw a card."
    );
    // "Tap up to one target creature. Scry 1, then draw a card." with no target chosen:
    // it isn't targeted, so it resolves.
    let mut t = TestGame::new(2);
    lots(&mut t, P0);
    let plunge = t.hand(P0, "Plunge into Winter");
    let s = t.cast(P0, plunge).targets(&[]).go();
    assert!(targets_of(&t, s).is_empty());
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1);
    // With a target chosen that becomes illegal, it doesn't resolve.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P0);
    let plunge = t.hand(P0, "Plunge into Winter");
    t.cast(P0, plunge).target(bears).go();
    t.g.destroy(bears, None);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
}

/// P1 casts Lightning Bolt at `target`; returns the spell.
fn bolt(t: &mut TestGame, target: Entity) -> ObjectId {
    t.set_step(P1, Step::PrecombatMain);
    lots(t, P1);
    let b = t.hand(P1, "Lightning Bolt");
    t.cast(P1, b).targets(&[target]).go()
}

#[test]
fn change_the_target_moves_each_target_to_another_legal_target_or_none() {
    cr!("115.7", "115.7a");
    ruling!(
        "Deflection",
        "If there is no other legal target for the spell, this does not change the target."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    let s = bolt(&mut t, o(bears));
    lots(&mut t, P0);
    let deflection = t.hand(P0, "Deflection");
    t.cast(P0, deflection).target(s).go();
    t.answer_targets(P0, &[o(giant)]);
    t.resolve();
    assert_eq!(targets_of(&t, s), vec![o(giant)]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Hill Giant"));
    assert!(t.on_battlefield(bears));
    // If the target can't be changed to another legal target, it's unchanged.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Vampire Nighthawk");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let blade = t.hand(P1, "Doom Blade");
    let s = t.cast(P1, blade).target(bears).go();
    lots(&mut t, P0);
    let deflection = t.hand(P0, "Deflection");
    t.cast(P0, deflection).target(s).go();
    t.resolve();
    assert_eq!(targets_of(&t, s), vec![o(bears)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn change_a_target_changes_only_one_of_them() {
    cr!("115.7b");
    ruling!(
        "Spellskite",
        "you can only change one of the targets to Spellskite"
    );
    let mut t = TestGame::new(2);
    let skite = t.battlefield(P0, "Spellskite");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let symbiosis = t.hand(P1, "Symbiosis");
    let s = t.cast(P1, symbiosis).targets(&[o(a), o(b)]).go();
    // "Change a target of target spell or ability to this creature."
    lots(&mut t, P0);
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.activate(P0, skite, 0, &[o(s)]).unwrap();
    t.resolve();
    let now = targets_of(&t, s);
    assert!(now.contains(&o(skite)));
    assert_eq!(now.iter().filter(|e| **e != o(skite)).count(), 1);
    t.resolve();
    assert_eq!(t.pt(skite), (2, 6));
}

#[test]
fn change_any_targets_may_change_some_or_none() {
    cr!("115.7c");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let ray = t.hand(P1, "Glacial Ray");
    let s = t.cast(P1, ray).target(bears).go();
    // "You may change any targets of target Arcane spell." P0 leaves it.
    lots(&mut t, P0);
    let swipe = t.hand(P0, "Sideswipe");
    t.cast(P0, swipe).target(s).go();
    t.answer_targets(P0, &[]);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(targets_of(&t, s), vec![o(bears)]);
    // Another Sideswipe changes it.
    lots(&mut t, P0);
    let swipe = t.hand(P0, "Sideswipe");
    t.cast(P0, swipe).target(s).go();
    t.answer_targets(P0, &[o(giant)]);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(targets_of(&t, s), vec![o(giant)]);
}

#[test]
fn choose_new_targets_may_leave_illegal_targets_unchanged() {
    cr!("115.7d");
    ruling!(
        "Redirect",
        "then it remains unchanged (even if the current target is illegal)"
    );
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    let c = t.battlefield(P1, "Vampire Nighthawk");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let symbiosis = t.hand(P1, "Symbiosis");
    let s = t.cast(P1, symbiosis).targets(&[o(a), o(b)]).go();
    // The Bears leave the battlefield: now an illegal target.
    t.g.destroy(a, None);
    // P0 chooses new targets: the Bears stay (illegal), the Giant becomes the Nighthawk.
    lots(&mut t, P0);
    let redirect = t.hand(P0, "Redirect");
    t.cast(P0, redirect).target(s).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[]);
    t.answer_targets(P0, &[o(c)]);
    t.resolve();
    assert_eq!(targets_of(&t, s), vec![o(a), o(c)]);
    // The new target can't make an unchanged one illegal: "another target creature".
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P1, "Hill Giant");
    let theirs = t.battlefield(P0, "Grizzly Bears");
    let other = t.battlefield(P0, "Vampire Nighthawk");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let hammer = t.hand(P1, "Fall of the Hammer");
    t.answer_targets(P1, &[o(mine)]);
    let s = t.cast(P1, hammer).target(theirs).go();
    lots(&mut t, P0);
    let redirect = t.hand(P0, "Redirect");
    t.cast(P0, redirect).target(s).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[]);
    // Changing "another target creature" to the Giant would make it both targets.
    t.answer_targets(P0, &[o(mine)]);
    t.resolve();
    assert!(last_candidates(&t, P0).contains(&o(mine)));
    assert_eq!(targets_of(&t, s), vec![o(mine), o(theirs)]);
    let _ = other;
}

#[test]
fn only_the_final_set_of_targets_is_evaluated() {
    cr!("115.7e");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let symbiosis = t.hand(P1, "Symbiosis");
    let s = t.cast(P1, symbiosis).targets(&[o(a), o(b)]).go();
    // Swapping the two targets passes through a state where both are the Giant; only the
    // final set counts.
    lots(&mut t, P0);
    let redirect = t.hand(P0, "Redirect");
    t.cast(P0, redirect).target(s).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[o(b)]);
    t.answer_targets(P0, &[o(a)]);
    t.resolve();
    assert_eq!(targets_of(&t, s), vec![o(b), o(a)]);
}

#[test]
fn the_original_division_is_kept_when_targets_change() {
    cr!("115.7f");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let their_bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let arc = t.hand(P1, "Arc Lightning");
    // 2 to the Hill Giant, 1 to P0.
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![2, 1]));
    let s = t
        .cast(P1, arc)
        .targets(&[o(giant), Entity::Player(P0)])
        .go();
    // P0 moves the 2 damage from the Giant to P1's Bears; the division stays.
    lots(&mut t, P0);
    let redirect = t.hand(P0, "Redirect");
    t.cast(P0, redirect).target(s).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[o(their_bears)]);
    t.answer_targets(P0, &[]);
    t.resolve();
    let si = t.obj(s).stack.as_ref().unwrap();
    assert_eq!(si.chosen[0].divided[0], vec![2, 1]);
    t.resolve();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert!(t.on_battlefield(giant));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.life(P0), 19);
}

#[test]
fn changing_the_targets_of_a_modal_spell_doesnt_change_its_mode() {
    cr!("115.8");
    ruling!("Redirect", "you can’t choose a different mode");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let charm = t.hand(P1, "Izzet Charm");
    // Mode 2: "Izzet Charm deals 2 damage to target creature."
    let s = t.cast(P1, charm).modes(&[1]).target(bears).go();
    lots(&mut t, P0);
    let redirect = t.hand(P0, "Redirect");
    t.cast(P0, redirect).target(s).go();
    t.answer_yes(P0, true);
    t.answer_targets(P0, &[o(giant)]);
    t.resolve();
    let si = t.obj(s).stack.as_ref().unwrap();
    assert_eq!(si.chosen.len(), 1);
    assert_eq!(si.chosen[0].mode, Some(1));
    t.resolve();
    assert_eq!(t.obj(giant).damage, 2);
    assert_eq!(t.obj(bears).damage, 0);
}

#[test]
fn with_a_single_target_counts_the_times_targets_were_chosen() {
    cr!("115.9", "115.9a");
    ruling!(
        "Deflection",
        "If a spell targets the same player or object multiple times, you can’t target it with Deflection."
    );
    ruling!(
        "Deflection",
        "This does not check if the current target is legal. It just checks if the spell has a single target."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let seeds = t.hand(P1, "Seeds of Strength");
    t.answer_targets(P1, &[o(bears)]);
    t.answer_targets(P1, &[o(bears)]);
    let s = t.cast(P1, seeds).target(bears).go();
    // One creature chosen three times: three targets, not a single target.
    lots(&mut t, P0);
    let deflection = t.hand(P0, "Deflection");
    assert!(t.cast(P0, deflection).target(s).try_go().is_err());
    t.clear_answers();
    t.resolve();
    // A spell whose single target is no longer legal still has a single target.
    let mine = t.battlefield(P0, "Hill Giant");
    let s = bolt(&mut t, o(bears));
    t.g.destroy(bears, None);
    lots(&mut t, P0);
    let deflection = t.hand(P0, "Deflection");
    t.cast(P0, deflection).target(s).go();
    t.answer_targets(P0, &[o(mine)]);
    t.resolve();
    assert_eq!(targets_of(&t, s), vec![o(mine)]);
}

#[test]
fn that_targets_checks_the_current_state_of_the_targets() {
    cr!("115.9b");
    ruling!(
        "Intervene",
        "Intervene won’t resolve since it no longer targets a spell that targets a creature"
    );
    // "Counter target spell that targets a creature."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let s = bolt(&mut t, o(bears));
    lots(&mut t, P0);
    let intervene = t.hand(P0, "Intervene");
    let i = t.cast(P0, intervene).target(s).go();
    let ctx = mtg_engine::eval::Ctx::new(Some(i), P0);
    let that_targets_a_creature = Filter::StackTargets(Box::new(TargetsFilter::Targets {
        objects: Some(Filter::creature()),
        players: None,
    }));
    assert!(t.g.matches(s, &that_targets_a_creature, &ctx));
    // A target that left the battlefield is ignored; its last known information isn't
    // used.
    t.g.destroy(bears, None);
    assert!(!t.g.matches(s, &that_targets_a_creature, &ctx));
    t.resolve();
    // Intervene's target is illegal now: it doesn't resolve, the Bolt does.
    assert!(t.in_graveyard(P0, "Intervene"));
    assert!(t.stack.contains(&s));
}

#[test]
fn that_targets_only_counts_different_objects_chosen() {
    cr!("115.9c");
    ruling!(
        "Muck Drubb",
        "Seeds of Strength targeting the same creature three times"
    );
    // Muck Drubb: "change the target of target spell that targets only a single creature
    // to this creature." Seeds of Strength targeting the same creature three times
    // targets only that creature.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    lots(&mut t, P1);
    let seeds = t.hand(P1, "Seeds of Strength");
    t.answer_targets(P1, &[o(bears)]);
    t.answer_targets(P1, &[o(bears)]);
    let s = t.cast(P1, seeds).target(bears).go();
    let ctx = mtg_engine::eval::Ctx::new(None, P0);
    let only_a_creature = Filter::StackTargets(Box::new(TargetsFilter::Only {
        objects: Some(Filter::creature()),
        players: None,
    }));
    assert!(t.g.matches(s, &only_a_creature, &ctx));
    // A spell with two different targets doesn't target only a single creature.
    let giant = t.battlefield(P1, "Hill Giant");
    lots(&mut t, P1);
    let symbiosis = t.hand(P1, "Symbiosis");
    let s2 = t.cast(P1, symbiosis).targets(&[o(bears), o(giant)]).go();
    assert!(!t.g.matches(s2, &only_a_creature, &ctx));
    // Muck Drubb can target the Seeds but not Symbiosis.
    t.answer_targets(P0, &[o(s)]);
    let drubb = t.enter(P0, "Muck Drubb");
    t.settle();
    let trigger = *t.stack.last().unwrap();
    assert_eq!(targets_of(&t, trigger), vec![o(s)]);
    let cands = last_candidates(&t, P0);
    assert!(!cands.contains(&o(s2)));
    t.resolve();
    assert_eq!(targets_of(&t, s), vec![o(drubb); 3]);
}

#[test]
fn spells_can_affect_what_they_dont_target() {
    cr!("115.10", "115.10a");
    let mut t = TestGame::new(2);
    // Hexproof doesn't stop an untargeted effect; the Phantasmal Bear isn't targeted.
    let troll = t.battlefield(P1, "Troll Ascetic");
    t.battlefield(P1, "Phantasmal Bear");
    lots(&mut t, P0);
    let pyroclasm = t.hand(P0, "Pyroclasm");
    t.cast(P0, pyroclasm).go();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.obj(troll).damage, 2);
    assert!(t.in_graveyard(P1, "Phantasmal Bear"));
}

#[test]
fn you_doesnt_indicate_a_target() {
    cr!("115.10b");
    // True Believer: "You have shroud."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "True Believer");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    t.library_top(P0, "Island");
    lots(&mut t, P0);
    let div = t.hand(P0, "Divination");
    t.cast(P0, div).go();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 2);
    // "Target player draws two cards" can't target P0.
    lots(&mut t, P0);
    let insp = t.hand(P0, "Inspiration");
    t.cast(P0, insp).target(P1).go();
    let cands = last_candidates(&t, P0);
    assert_eq!(cands, vec![Entity::Player(P1)]);
}
