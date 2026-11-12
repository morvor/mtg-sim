//! CR 702.26 audit (review): instructions that name a phased-out permanent without
//! targeting it — a delayed "sacrifice the creature" or "destroy that creature", an
//! ability's "this creature" — do nothing to it. A phased-out permanent is treated as
//! though it doesn't exist and can't be affected by anything (702.26b), and a delayed
//! triggered ability doesn't affect an object that isn't where it's expected to be
//! (603.7c, with 702.26d's "treated as though it's not on the battlefield"): it isn't
//! sacrificed (and nothing sees a sacrifice), its regeneration shield isn't used, and it
//! doesn't become monstrous, renowned or suspected, flip, turn face down or gain a level.
//! An Aura or Equipment phased out by itself doesn't make the permanent it stays attached
//! to enchanted or equipped until it phases in (702.26i).

use mtg_engine::card::card;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::*;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Asserts the compiler fully supports `name`.
fn supported(name: &str) {
    let c = card(name);
    assert!(
        c.is_fully_supported(),
        "{name}: unsupported {:?}",
        c.unsupported_text()
    );
}

fn phased_out(t: &TestGame, id: ObjectId) -> bool {
    t.g.obj(t.g.current(id)).phased_out
}

fn pt(t: &mut TestGame, id: ObjectId) -> (i32, i32) {
    t.g.recompute();
    t.pt(id)
}

/// `p` casts Reality Ripple ("Target artifact, creature, or land phases out.") on
/// `target`, on top of whatever is on the stack, and it resolves.
fn ripple(t: &mut TestGame, p: PlayerId, target: ObjectId) {
    supported("Reality Ripple");
    t.lands(p, "Island", 2);
    let spell = t.hand(p, "Reality Ripple");
    t.cast(p, spell).target(target).go();
    t.resolve();
    assert!(phased_out(t, target));
}

/// Advances to `p`'s next upkeep (its untap step's phasing done), triggered abilities not
/// yet put on the stack.
fn to_upkeep(t: &mut TestGame, p: PlayerId) {
    if t.g.turn.active == p {
        let other = t.g.next_player(p);
        t.advance_to(other, Step::Upkeep);
    }
    t.advance_to(p, Step::Upkeep);
}

/// P0 attacks P1 with `attacker`, unblocked, to the combat damage step (the damage dealt,
/// its triggers on the stack).
fn hit(t: &mut TestGame, attacker: ObjectId) {
    if t.g.turn.active != P0 || t.g.turn.step > Step::BeginningOfCombat {
        to_upkeep(t, P0);
    }
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(attacker, Entity::Player(P1))]),
    );
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
}

/// P0 puts `name` from their hand onto the battlefield with Sneak Attack ("{R}: You may
/// put a creature card from your hand onto the battlefield. That creature gains haste.
/// Sacrifice the creature at the beginning of the next end step.").
fn sneak(t: &mut TestGame, sneak_attack: ObjectId, name: &str) -> ObjectId {
    let c = t.hand(P0, name);
    t.lands(P0, "Mountain", 1);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(c)]);
    t.activate(P0, sneak_attack, 0, &[])
        .expect("activate Sneak Attack");
    t.resolve();
    t.clear_answers();
    let c = t.g.current(c);
    assert!(t.on_battlefield(c), "{name} was put onto the battlefield");
    c
}

/// Sneak Attack puts Ornithopter (an artifact creature) onto the battlefield, and it phases
/// out if `phase_out`; at the end step its delayed sacrifice resolves. Returns the game,
/// Juri ("Whenever you sacrifice a permanent, put a +1/+1 counter on Juri."), Detective's
/// Satchel ("{T}: Create a 1/1 colorless Thopter artifact creature token with flying.
/// Activate only if you've sacrificed an artifact this turn.") and the Ornithopter.
fn sneaked_ornithopter_at_end_step(phase_out: bool) -> (TestGame, ObjectId, ObjectId, ObjectId) {
    supported("Sneak Attack");
    supported("Juri, Master of the Revue");
    supported("Detective's Satchel");
    let mut t = TestGame::new(2);
    let juri = t.battlefield(P0, "Juri, Master of the Revue");
    let satchel = t.battlefield(P0, "Detective's Satchel");
    let sa = t.battlefield(P0, "Sneak Attack");
    let thopter = sneak(&mut t, sa, "Ornithopter");
    if phase_out {
        ripple(&mut t, P0, thopter);
    }
    t.advance_to(P0, Step::End);
    t.resolve_all();
    (t, juri, satchel, thopter)
}

#[test]
fn a_phased_out_creature_isnt_sacrificed() {
    cr!("702.26b", "603.7c", "701.21a");
    // Phased out, the Ornithopter isn't sacrificed: Juri sees no sacrifice, and P0 hasn't
    // sacrificed an artifact this turn, so the Satchel's ability can't be activated.
    let (mut t, juri, satchel, thopter) = sneaked_ornithopter_at_end_step(true);
    assert!(t.on_battlefield(thopter) && phased_out(&t, thopter));
    assert_eq!(t.counters(juri, counters::PLUS1), 0);
    assert!(t.activate(P0, satchel, 0, &[]).is_err());
    // It phases in during P0's next untap step and stays.
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(t.on_battlefield(thopter) && !phased_out(&t, thopter));
    assert_eq!(t.counters(juri, counters::PLUS1), 0);
    // Phased in, it's sacrificed: Juri gets a counter and the Satchel makes a Thopter.
    let (mut t, juri, satchel, thopter) = sneaked_ornithopter_at_end_step(false);
    assert!(!t.on_battlefield(thopter));
    assert!(t.in_graveyard(P0, "Ornithopter"));
    assert_eq!(t.counters(juri, counters::PLUS1), 1);
    t.activate(P0, satchel, 0, &[]).expect("an artifact was sacrificed");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Thopter Token").len(), 1);
}

#[test]
fn a_phased_out_creature_isnt_destroyed_and_keeps_its_regeneration_shield() {
    cr!("702.26b", "603.7c", "701.8a", "701.19a");
    supported("Stone Giant");
    supported("Drudge Skeletons");
    supported("The Moment");
    let mut t = TestGame::new(2);
    let stone_giant = t.battlefield(P0, "Stone Giant");
    let skeletons = t.battlefield(P0, "Drudge Skeletons");
    let moment = t.battlefield(P0, "The Moment");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Plains", 2);
    // "{T}: Target creature you control with toughness less than this creature's power
    // gains flying until end of turn. Destroy that creature at the beginning of the next
    // end step."
    t.activate(P0, stone_giant, 0, &[Entity::Object(skeletons)])
        .expect("Stone Giant");
    t.resolve();
    // "{B}: Regenerate this creature.", then The Moment: "{2}, {T}: Untap target creature
    // you control. It phases out until The Moment leaves the battlefield."
    t.activate(P0, skeletons, 0, &[]).expect("regenerate");
    t.resolve();
    t.activate(P0, moment, 0, &[Entity::Object(skeletons)])
        .expect("The Moment");
    t.resolve();
    assert!(phased_out(&t, skeletons));
    // The delayed "destroy that creature" finds the Skeletons phased out: they aren't
    // destroyed, and the shield isn't used up on them.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.stack_len(), 0);
    assert!(t.on_battlefield(skeletons) && phased_out(&t, skeletons));
    // The Moment leaves: they phase in, still shielded, and Murder only regenerates them.
    t.lands(P0, "Mountain", 2);
    let shatter = t.hand(P0, "Shatter");
    t.cast(P0, shatter).target(moment).go();
    t.resolve_all();
    assert!(!phased_out(&t, skeletons));
    assert!(!t.obj_now(skeletons).tapped);
    t.lands(P1, "Swamp", 3);
    let murder = t.hand(P1, "Murder");
    t.cast(P1, murder).target(skeletons).go();
    t.resolve_all();
    assert!(t.on_battlefield(skeletons), "regenerated");
    assert!(t.obj_now(skeletons).tapped, "regeneration taps it");
    assert!(t.in_graveyard(P1, "Murder"));
}

#[test]
fn a_phased_out_creature_doesnt_become_monstrous() {
    cr!("702.26b", "701.37a");
    supported("Nessian Asp");
    let mut t = TestGame::new(2);
    let asp = t.battlefield(P0, "Nessian Asp");
    t.lands(P0, "Forest", 7);
    // "{6}{G}: Monstrosity 4." The Asp phases out in response: no counters, and it doesn't
    // become monstrous.
    t.activate(P0, asp, 0, &[]).expect("monstrosity");
    ripple(&mut t, P0, asp);
    t.resolve_all();
    assert_eq!(t.counters(asp, counters::PLUS1), 0);
    assert!(!t.obj_now(asp).monstrous);
    // Phased in, it isn't monstrous: the same ability makes it monstrous now.
    to_upkeep(&mut t, P0);
    t.settle();
    assert!(!phased_out(&t, asp));
    assert!(!t.obj_now(asp).monstrous);
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, asp, 0, &[]).expect("monstrosity");
    t.resolve_all();
    assert!(t.obj_now(asp).monstrous);
    assert_eq!(t.counters(asp, counters::PLUS1), 4);
    assert_eq!(pt(&mut t, asp), (8, 9));
}

#[test]
fn a_phased_out_creature_doesnt_become_renowned() {
    cr!("702.26b", "702.112a", "603.4");
    supported("Topan Freeblade");
    let mut t = TestGame::new(2);
    // Renown 1: "When this creature deals combat damage to a player, if it isn't renowned,
    // put a +1/+1 counter on it and it becomes renowned." It phases out with the trigger
    // on the stack.
    let blade = t.battlefield(P0, "Topan Freeblade");
    hit(&mut t, blade);
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.stack_len(), 1);
    ripple(&mut t, P0, blade);
    t.resolve_all();
    assert_eq!(t.counters(blade, counters::PLUS1), 0);
    assert!(!t.obj_now(blade).renowned);
    // Phased in, it isn't renowned: its next combat damage to a player renowns it.
    hit(&mut t, blade);
    assert!(!phased_out(&t, blade));
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.obj_now(blade).renowned);
    assert_eq!(t.counters(blade, counters::PLUS1), 1);
}

#[test]
fn a_phased_out_flip_card_doesnt_flip() {
    cr!("702.26b", "710.2", "710.4");
    const LAVARUNNER: &str = "Akki Lavarunner // Tok-Tok, Volcano Born";
    supported(LAVARUNNER);
    let mut t = TestGame::new(2);
    // "Whenever this creature deals damage to an opponent, flip it." It phases out with
    // the trigger on the stack: it doesn't flip.
    let akki = t.battlefield(P0, LAVARUNNER);
    hit(&mut t, akki);
    assert_eq!(t.life(P1), 19);
    assert_eq!(t.stack_len(), 1);
    ripple(&mut t, P0, akki);
    t.resolve_all();
    assert!(!t.obj_now(akki).flipped);
    // It phases in unflipped (a 1/1 Akki Lavarunner); phased in, the trigger flips it.
    hit(&mut t, akki);
    assert!(!phased_out(&t, akki));
    assert_eq!(t.obj_now(akki).chars.name.as_str(), "Akki Lavarunner");
    assert_eq!(t.life(P1), 18);
    t.resolve_all();
    assert!(t.obj_now(akki).flipped);
    assert_eq!(t.obj_now(akki).chars.name.as_str(), "Tok-Tok, Volcano Born");
}

#[test]
fn a_phased_out_permanent_isnt_turned_face_down() {
    cr!("702.26b", "708.2");
    supported("Mischievous Quanar");
    let mut t = TestGame::new(2);
    let quanar = t.battlefield(P0, "Mischievous Quanar");
    t.lands(P0, "Island", 5);
    // "{3}{U}{U}: Turn this creature face down." It phases out in response.
    t.activate(P0, quanar, 0, &[]).expect("turn face down");
    ripple(&mut t, P0, quanar);
    t.resolve_all();
    assert!(!t.obj_now(quanar).face_down);
    // It phases in face up, a 3/3 Mischievous Quanar; phased in, the ability turns it face
    // down (a 2/2 with no name).
    to_upkeep(&mut t, P0);
    t.settle();
    assert!(!phased_out(&t, quanar));
    assert!(!t.obj_now(quanar).face_down);
    assert_eq!(pt(&mut t, quanar), (3, 3));
    t.advance_to(P0, Step::PrecombatMain);
    t.activate(P0, quanar, 0, &[]).expect("turn face down");
    t.resolve_all();
    assert!(t.obj_now(quanar).face_down);
    assert_eq!(pt(&mut t, quanar), (2, 2));
}

#[test]
fn a_phased_out_class_doesnt_gain_a_level() {
    cr!("702.26b", "716.2a");
    supported("Druid Class");
    supported("Clever Concealment");
    let mut t = TestGame::new(2);
    let class = t.battlefield(P0, "Druid Class");
    t.lands(P0, "Forest", 3);
    // "{2}{G}: Level 2" is on the stack when Clever Concealment ("Any number of target
    // nonland permanents you control phase out.") phases the Class out: it stays level 1.
    t.activate(P0, class, 0, &[]).expect("level 2");
    t.lands(P0, "Plains", 4);
    let conceal = t.hand(P0, "Clever Concealment");
    t.cast(P0, conceal)
        .targets(&[Entity::Object(class)])
        .go();
    t.resolve();
    assert!(phased_out(&t, class));
    t.resolve_all();
    assert!(t.obj_now(class).class_level <= 1);
    // It phases in at level 1 ("You may play an additional land on each of your turns" not
    // in effect); phased in, it gains level 2.
    to_upkeep(&mut t, P0);
    t.settle();
    assert!(!phased_out(&t, class));
    assert!(t.obj_now(class).class_level <= 1);
    t.advance_to(P0, Step::PrecombatMain);
    t.g.recompute();
    assert_eq!(t.g.player(P0).land_plays, 1);
    t.activate(P0, class, 0, &[]).expect("level 2");
    t.resolve_all();
    assert_eq!(t.obj_now(class).class_level, 2);
    t.g.recompute();
    assert_eq!(t.g.player(P0).land_plays, 2);
}

#[test]
fn a_phased_out_creature_doesnt_become_suspected() {
    cr!("702.26b", "701.60a");
    supported("Frantic Scapegoat");
    let mut t = TestGame::new(2);
    // "When this creature enters, suspect it." It phases out in response.
    let goat = t.enter(P0, "Frantic Scapegoat");
    t.settle();
    assert_eq!(t.stack_len(), 1);
    ripple(&mut t, P0, goat);
    t.resolve_all();
    assert!(!t.obj_now(goat).suspected);
    // It phases in unsuspected.
    to_upkeep(&mut t, P0);
    t.settle();
    assert!(!phased_out(&t, goat));
    assert!(!t.obj_now(goat).suspected);
    // One that stays phased in is suspected.
    let other = t.enter(P0, "Frantic Scapegoat");
    t.resolve_all();
    assert!(t.obj_now(other).suspected);
}

#[test]
fn a_creature_whose_aura_phased_out_isnt_enchanted() {
    cr!("702.26b", "702.26i", "603.4");
    supported("Dreampod Druid");
    supported("Clever Concealment");
    let mut t = TestGame::new(2);
    // "At the beginning of each upkeep, if this creature is enchanted, create a 1/1 green
    // Saproling creature token." Clever Concealment phases out its Aura alone: while the
    // Aura is phased out, the Druid isn't enchanted.
    let druid = t.battlefield(P0, "Dreampod Druid");
    let aura = t.battlefield(P0, "Holy Strength");
    t.g.attach(aura, Entity::Object(druid));
    assert_eq!(pt(&mut t, druid), (3, 4));
    t.lands(P0, "Plains", 4);
    let conceal = t.hand(P0, "Clever Concealment");
    t.cast(P0, conceal)
        .targets(&[Entity::Object(aura)])
        .go();
    t.resolve_all();
    assert!(phased_out(&t, aura) && !phased_out(&t, druid));
    assert_eq!(pt(&mut t, druid), (2, 2));
    to_upkeep(&mut t, P1);
    t.resolve_all();
    assert!(t.named_on_battlefield("Saproling Token").is_empty());
    // In P0's untap step the Aura phases in still attached (it phased out directly), and
    // the Druid is enchanted again.
    to_upkeep(&mut t, P0);
    t.resolve_all();
    assert!(!phased_out(&t, aura));
    assert_eq!(t.obj_now(aura).attached_to, Some(Entity::Object(druid)));
    assert_eq!(t.named_on_battlefield("Saproling Token").len(), 1);
}

#[test]
fn a_creature_whose_equipment_phased_out_isnt_equipped() {
    cr!("702.26b", "702.26i");
    supported("Kor Duelist");
    let mut t = TestGame::new(2);
    // "As long as this creature is equipped, it has double strike." Its Bonesplitter
    // ("Equipped creature gets +2/+0.") phases out alone.
    let duelist = t.battlefield(P0, "Kor Duelist");
    let splitter = t.battlefield(P0, "Bonesplitter");
    t.g.attach(splitter, Entity::Object(duelist));
    t.g.recompute();
    assert!(t.obj_now(duelist).has_keyword(KeywordKind::DoubleStrike));
    ripple(&mut t, P0, splitter);
    t.g.recompute();
    assert!(!t.obj_now(duelist).has_keyword(KeywordKind::DoubleStrike));
    assert_eq!(pt(&mut t, duelist), (1, 1));
    // It phases in still attached: the Duelist is equipped again.
    to_upkeep(&mut t, P0);
    t.settle();
    t.g.recompute();
    assert_eq!(t.obj_now(splitter).attached_to, Some(Entity::Object(duelist)));
    assert!(t.obj_now(duelist).has_keyword(KeywordKind::DoubleStrike));
    assert_eq!(pt(&mut t, duelist), (3, 1));
}
