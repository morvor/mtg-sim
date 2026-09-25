//! CR 702.26 Phasing.

use crate::common_k702_011_017::*;
use crate::common_k702_018_026::*;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kw::phasing::{phase_in, phase_out};
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn phased_out(t: &TestGame, id: ObjectId) -> bool {
    t.g.obj(id).phased_out
}

/// Advances to `p`'s next turn, just past its untap step (state-based actions checked).
fn next_turn(t: &mut TestGame, p: PlayerId) {
    if t.g.turn.active == p {
        let other = t.g.next_player(p);
        t.advance_to(other, Step::Upkeep);
    }
    t.advance_to(p, Step::Upkeep);
    t.settle();
}

/// Power and toughness with characteristics up to date.
fn pt(t: &mut TestGame, id: ObjectId) -> (i32, i32) {
    t.g.recompute();
    t.pt(id)
}

/// Casts Reality Ripple for `p` ("Target artifact, creature, or land phases out.").
fn ripple(t: &mut TestGame, p: PlayerId, target: ObjectId) {
    t.lands(p, "Island", 2);
    let spell = t.hand(p, "Reality Ripple");
    t.cast(p, spell).target(target).go();
    t.resolve_all();
    assert!(phased_out(t, target));
}

/// `p` casts Sower of Temptation in their main phase, gaining control of `victim` for as
/// long as the Sower remains on the battlefield. Returns the Sower.
fn steal(t: &mut TestGame, p: PlayerId, victim: ObjectId) -> ObjectId {
    t.set_step(p, Step::PrecombatMain);
    t.lands(p, "Island", 4);
    let sower = t.hand(p, "Sower of Temptation");
    t.cast(p, sower).go();
    t.answer_targets(p, &[Entity::Object(victim)]);
    t.resolve_all();
    t.g.recompute();
    assert_eq!(t.obj_now(victim).controller, p);
    t.g.current(sower)
}

#[test]
fn phasing_toggles_during_its_controllers_untap_step() {
    cr!("702.26", "702.26a", "702.26c");
    assert_supported("Breezekeeper");
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Breezekeeper");
    let other = t.battlefield(P0, "Breezekeeper");
    next_turn(&mut t, P1);
    // Not during another player's untap step.
    assert!(!phased_out(&t, keeper));
    // Put the other one out of phase: at P0's untap step both switch, simultaneously.
    t.g.objects[other.0 as usize].phased_out = true;
    t.g.objects[other.0 as usize].phased_out_under = Some(P0);
    t.g.tap(keeper);
    next_turn(&mut t, P0);
    assert!(phased_out(&t, keeper));
    assert!(!phased_out(&t, other));
    // Phasing happens before untapping: the one that phased out stays tapped.
    assert!(t.obj_now(keeper).tapped);
    next_turn(&mut t, P0);
    assert!(!phased_out(&t, keeper));
    assert!(phased_out(&t, other));
    // It phased in before P0 untapped, so it untapped.
    assert!(!t.obj_now(keeper).tapped);
}

#[test]
fn a_creature_that_phases_in_can_attack() {
    cr!("702.26a", "702.26c", "702.26d");
    ruling!(
        "Slip Out the Back",
        "Creatures that phase in this way are able to attack and pay a cost of {T} during that turn"
    );
    assert_supported("Slip Out the Back");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Island", 1);
    let slip = t.hand(P0, "Slip Out the Back");
    t.cast(P0, slip).target(bears).go();
    t.resolve_all();
    assert!(phased_out(&t, bears));
    next_turn(&mut t, P1);
    assert!(phased_out(&t, bears));
    next_turn(&mut t, P0);
    assert!(!phased_out(&t, bears));
    // Same object, with its counter, not summoning sick.
    assert!(t.g.is_live(bears));
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert!(!t.obj_now(bears).summoning_sick);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(t.g.can_attack(bears));
}

#[test]
fn a_phased_out_permanent_is_treated_as_though_it_doesnt_exist() {
    cr!("702.26b");
    ruling!(
        "Clever Concealment",
        "Phased out permanents are treated as though they don’t exist"
    );
    let mut t = TestGame::new(2);
    let anthem = t.battlefield(P0, "Glorious Anthem");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let warden = t.battlefield(P0, "Soul Warden");
    assert_eq!(pt(&mut t, bears), (3, 3));
    phase_out(&mut t.g, vec![anthem, warden]);
    // Its static abilities have no effect.
    assert_eq!(pt(&mut t, bears), (2, 2));
    // It can't be targeted.
    assert!(!spell_can_target(&mut t, P1, "Lightning Bolt", warden));
    // Its triggered abilities don't trigger.
    let life = t.life(P0);
    t.battlefield(P1, "Grizzly Bears");
    t.settle();
    t.resolve_all();
    assert_eq!(t.life(P0), life);
    // It isn't affected by "destroy all creatures".
    t.lands(P0, "Plains", 4);
    let wrath = t.hand(P0, "Wrath of God");
    t.cast(P0, wrath).go();
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(warden));
    assert!(t.g.is_live(warden));
}

#[test]
fn a_creature_that_phases_out_is_removed_from_combat() {
    cr!("702.26b");
    ruling!(
        "Slip Out the Back",
        "An attacking or blocking creature that phases out is removed from combat"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(t.g.is_attacking(bears));
    t.lands(P0, "Island", 1);
    let slip = t.hand(P0, "Slip Out the Back");
    t.cast(P0, slip).target(bears).go();
    t.resolve_all();
    assert!(!t.g.is_attacking(bears));
    block_and_finish(&mut t, P1, &[]);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn phasing_doesnt_trigger_enters_or_leaves_abilities() {
    cr!("702.26d");
    ruling!(
        "Guardian of Faith",
        "Phasing out doesn't cause any \"leaves the battlefield\" abilities to trigger. Similarly, phasing in won't cause any \"enters the battlefield\" abilities to trigger."
    );
    let mut t = TestGame::new(2);
    let fox = t.battlefield(P0, "Clockwork Fox");
    let elf = t.battlefield(P0, "Elvish Visionary");
    let hand = t.hand_size(P0);
    let hand1 = t.hand_size(P1);
    phase_out(&mut t.g, vec![fox, elf]);
    t.settle();
    t.resolve_all();
    phase_in(&mut t.g, fox);
    phase_in(&mut t.g, elf);
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand);
    assert_eq!(t.hand_size(P1), hand1);
    assert!(t.g.is_live(fox) && t.g.is_live(elf));
    assert_eq!(t.zone(fox), Zone::Battlefield);
}

#[test]
fn tokens_and_counters_survive_phasing() {
    cr!("702.26d");
    ruling!(
        "Teferi's Protection",
        "If a token is phased out, it will phase in as your next untap step begins"
    );
    ruling!(
        "Teferi's Protection",
        "Permanents that phase out with counters phase in with those counters"
    );
    let mut t = TestGame::new(2);
    t.lands(P0, "Plains", 2);
    let alarm = t.hand(P0, "Raise the Alarm");
    t.cast(P0, alarm).go();
    t.resolve_all();
    let token = t
        .g
        .battlefield
        .iter()
        .copied()
        .find(|id| t.g.obj(*id).is_token())
        .expect("a Soldier token");
    assert!(t.obj_now(token).is_token());
    t.g.add_counters(Entity::Object(token), "+1/+1", 2, None);
    phase_out(&mut t.g, vec![token]);
    t.settle();
    // State-based actions don't remove the phased-out token.
    assert!(t.g.is_live(token));
    next_turn(&mut t, P0);
    assert!(!phased_out(&t, token));
    assert_eq!(t.counters(token, "+1/+1"), 2);
}

#[test]
fn a_resolving_effect_doesnt_include_phased_out_permanents() {
    cr!("702.26e");
    assert_supported("Inspired Charge");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    phase_out(&mut t.g, vec![a]);
    t.lands(P0, "Plains", 4);
    let charge = t.hand(P0, "Inspired Charge");
    t.cast(P0, charge).go();
    t.resolve_all();
    assert_eq!(pt(&mut t, b), (4, 3));
    // It phases back in this turn: it isn't affected.
    phase_in(&mut t.g, a);
    t.g.recompute();
    assert_eq!(pt(&mut t, a), (2, 2));
}

#[test]
fn effects_can_expire_while_phased_out() {
    cr!("702.26f");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 1);
    let growth = t.hand(P0, "Giant Growth");
    t.cast(P0, growth).target(bears).go();
    t.resolve_all();
    assert_eq!(pt(&mut t, bears), (5, 5));
    ripple(&mut t, P0, bears);
    next_turn(&mut t, P0);
    assert!(!phased_out(&t, bears));
    assert_eq!(pt(&mut t, bears), (2, 2));
}

#[test]
fn for_as_long_as_durations_end_when_the_permanent_phases_out() {
    cr!("702.26f");
    ruling!(
        "Slip Out the Back",
        "Any continuous effects with a “for as long as” duration"
    );
    assert_supported("Sower of Temptation");
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P1, "Gray Ogre");
    let sower = steal(&mut t, P0, ogre);
    ripple(&mut t, P0, sower);
    t.g.recompute();
    assert_eq!(t.obj_now(ogre).controller, P1);
    // Phasing back in doesn't bring the effect back.
    phase_in(&mut t.g, sower);
    t.g.recompute();
    assert_eq!(t.obj_now(ogre).controller, P1);
}

#[test]
fn attachments_phase_out_indirectly_and_in_attached() {
    cr!("702.26g");
    ruling!(
        "Slip Out the Back",
        "As a creature is phased out, Auras and Equipment attached to it also phase out at the same time"
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = t.battlefield(P0, "Bonesplitter");
    let rancor = t.battlefield(P0, "Rancor");
    t.g.attach(splitter, Entity::Object(bears));
    t.g.attach(rancor, Entity::Object(bears));
    ripple(&mut t, P0, bears);
    for a in [splitter, rancor] {
        assert!(phased_out(&t, a));
        assert!(t.g.obj(a).phased_out_indirectly);
    }
    next_turn(&mut t, P0);
    for a in [splitter, rancor] {
        assert!(!phased_out(&t, a));
        assert_eq!(t.obj_now(a).attached_to, Some(Entity::Object(bears)));
    }
    assert_eq!(pt(&mut t, bears), (6, 2));
}

#[test]
fn phasing_out_directly_and_indirectly_is_indirectly() {
    cr!("702.26h");
    let aura = custom_card(
        "Phasing Veil",
        "Enchantment — Aura",
        None,
        "Enchant creature\nPhasing",
    );
    let mut t = TestGame::new(2);
    // The Aura is first in the battlefield's order, so it's considered first.
    let veil = t.custom(P0, aura, Zone::Battlefield);
    let keeper = t.battlefield(P0, "Breezekeeper");
    t.g.attach(veil, Entity::Object(keeper));
    assert!(t.g.battlefield.iter().position(|x| *x == veil) < t.g.battlefield.iter().position(|x| *x == keeper));
    next_turn(&mut t, P0);
    assert!(phased_out(&t, keeper));
    assert!(phased_out(&t, veil));
    assert!(t.g.obj(veil).phased_out_indirectly);
    next_turn(&mut t, P0);
    assert!(!phased_out(&t, keeper));
    assert!(!phased_out(&t, veil));
    assert_eq!(t.obj_now(veil).attached_to, Some(Entity::Object(keeper)));
}

#[test]
fn an_aura_that_phased_out_directly_phases_in_attached_if_it_can() {
    cr!("702.26i");
    ruling!(
        "Teferi's Protection",
        "If not, it phases in unattached. An Aura that phases in unattached will be put into its owner's graveyard as a state-based action"
    );
    assert_supported("Clever Concealment");
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P1, "Gray Ogre");
    let pacifism = t.battlefield(P0, "Pacifism");
    t.g.attach(pacifism, Entity::Object(ogre));
    let bears = t.battlefield(P1, "Grizzly Bears");
    let pac2 = t.battlefield(P0, "Pacifism");
    t.g.attach(pac2, Entity::Object(bears));
    t.lands(P0, "Plains", 4);
    let conceal = t.hand(P0, "Clever Concealment");
    t.cast(P0, conceal)
        .targets(&[Entity::Object(pacifism), Entity::Object(pac2)])
        .go();
    t.resolve_all();
    assert!(phased_out(&t, pacifism) && phased_out(&t, pac2));
    assert!(!t.g.obj(pacifism).phased_out_indirectly);
    // One enchanted creature leaves the battlefield meanwhile.
    t.g.destroy(bears, None);
    next_turn(&mut t, P0);
    assert!(!phased_out(&t, pacifism));
    assert_eq!(t.obj_now(pacifism).attached_to, Some(Entity::Object(ogre)));
    assert!(t.in_graveyard(P0, "Pacifism"));
}

#[test]
fn an_equipment_that_phased_out_directly_phases_in_unattached_if_it_must() {
    cr!("702.26i");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let splitter = t.battlefield(P0, "Bonesplitter");
    t.g.attach(splitter, Entity::Object(bears));
    ripple(&mut t, P0, splitter);
    assert!(!t.g.obj(splitter).phased_out_indirectly);
    assert_eq!(pt(&mut t, bears), (2, 2));
    t.g.destroy(bears, None);
    next_turn(&mut t, P0);
    assert!(!phased_out(&t, splitter));
    assert!(t.on_battlefield(splitter));
    assert_eq!(t.obj_now(splitter).attached_to, None);
}

#[test]
fn phasing_doesnt_trigger_attach_or_unattach_abilities() {
    cr!("702.26j");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let exo = t.battlefield(P0, "Grafted Exoskeleton");
    t.g.attach(exo, Entity::Object(bears));
    // The Equipment phases out directly, then in.
    ripple(&mut t, P0, exo);
    next_turn(&mut t, P0);
    t.settle();
    t.resolve_all();
    assert!(!phased_out(&t, exo));
    assert!(t.on_battlefield(bears));
    assert_eq!(t.obj_now(exo).attached_to, Some(Entity::Object(bears)));
    // The creature phases out, taking the Equipment with it, then both phase in.
    ripple(&mut t, P0, bears);
    next_turn(&mut t, P0);
    t.settle();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(pt(&mut t, bears), (4, 4));
}

#[test]
fn phased_out_permanents_leave_with_their_owner() {
    cr!("702.26k");
    let watcher = custom_card(
        "Departure Watcher",
        "Creature — Spirit",
        Some((1, 1)),
        "Whenever another creature leaves the battlefield, you gain 1 life.",
    );
    let mut t = TestGame::new(3);
    bf(&mut t, P0, watcher);
    let fox = t.battlefield(P2, "Clockwork Fox");
    phase_out(&mut t.g, vec![fox]);
    let life = t.life(P0);
    t.g.player_loses(P2);
    t.settle();
    t.resolve_all();
    assert_eq!(t.g.obj(fox).zone, Zone::Nowhere);
    assert!(!t.g.battlefield.contains(&fox));
    // Leaving the game this way isn't a zone change that triggers anything.
    assert_eq!(t.life(P0), life);
}

#[test]
fn a_skipped_untap_step_has_no_phasing() {
    cr!("702.26m");
    ruling!(
        "Teferi's Protection",
        "If your untap step is somehow skipped as your next turn begins, your phased-out permanents won't phase in until the next untap step you actually have"
    );
    assert_supported("Stasis");
    let mut t = TestGame::new(2);
    let keeper = t.battlefield(P0, "Breezekeeper");
    let bears = t.battlefield(P0, "Grizzly Bears");
    ripple(&mut t, P0, bears);
    // (P0 controls it, so its upkeep trigger comes after the skipped untap step.)
    t.battlefield(P0, "Stasis");
    next_turn(&mut t, P0);
    assert!(!phased_out(&t, keeper));
    assert!(phased_out(&t, bears));
}

#[test]
fn control_ending_while_phased_out_phases_in_at_that_players_untap_step() {
    cr!("702.26a");
    ruling!(
        "Guardian of Faith",
        "that creature phases in under your control as that opponent's next untap step begins"
    );
    let mut t = TestGame::new(2);
    let ogre = t.battlefield(P0, "Gray Ogre");
    let sower = steal(&mut t, P1, ogre);
    ripple(&mut t, P1, ogre);
    t.g.destroy(sower, None);
    t.g.recompute();
    assert_eq!(t.obj_now(ogre).controller, P0);
    // Not during its owner's untap step, but during the untap step of the player it phased
    // out under.
    next_turn(&mut t, P0);
    assert!(phased_out(&t, ogre));
    next_turn(&mut t, P1);
    assert!(!phased_out(&t, ogre));
    assert_eq!(t.obj_now(ogre).controller, P0);
}

#[test]
fn phased_out_under_a_player_who_left_the_game() {
    cr!("702.26n");
    let mut t = TestGame::new(3);
    let ogre = t.battlefield(P0, "Gray Ogre");
    steal(&mut t, P1, ogre);
    ripple(&mut t, P1, ogre);
    // P1 leaves the game during P2's turn: the control effect ends, so the Ogre isn't
    // exiled, and it stays phased out.
    next_turn(&mut t, P2);
    t.g.player_loses(P1);
    t.g.recompute();
    assert!(t.g.is_live(ogre));
    assert_eq!(t.obj_now(ogre).controller, P0);
    assert!(phased_out(&t, ogre));
    // P1's next turn would have begun after P0's: the Ogre phases in during the next
    // untap step after that, P2's, not during P0's.
    next_turn(&mut t, P0);
    assert!(phased_out(&t, ogre));
    next_turn(&mut t, P2);
    assert!(!phased_out(&t, ogre));
    assert_eq!(t.obj_now(ogre).controller, P0);
}

#[test]
fn multiple_instances_of_phasing_are_redundant() {
    cr!("702.26p");
    let def = custom_card(
        "Double Phaser",
        "Creature — Illusion",
        Some((2, 2)),
        "Phasing\nPhasing",
    );
    let mut t = TestGame::new(2);
    let c = bf(&mut t, P0, def);
    assert_eq!(keyword_count(&t, c, KeywordKind::Phasing), 2);
    next_turn(&mut t, P0);
    assert!(phased_out(&t, c));
    next_turn(&mut t, P0);
    assert!(!phased_out(&t, c));
}
