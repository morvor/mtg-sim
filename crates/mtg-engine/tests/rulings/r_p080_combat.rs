//! Rulings batch P080 — combat bookkeeping: creatures put onto the battlefield blocking
//! (CR 509.4, 509.1h), reselecting what an attacker attacks (CR 508.7), extra combat
//! phases (CR 500.8, 505.1a), "doesn't untap" on already tapped creatures, static -1/-0
//! while untapped, and Two-Headed Giant teams (CR 810).

use crate::r_p079_common::*;
use crate::r_s01_common::{attack_with, supported, tokens};
use crate::r_s03_common::to_blockers;
use crate::r_s09_common::{attacks_now, declare_in_next_combat};
use crate::r_s10_common::{attacking, blocking};
use crate::r_s21_common::go_to;
use crate::r_s33_common::two_headed_giant;
use mtg_engine::combat::{player_has_attacked, player_is_attacking};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::CastMethod;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::*;

/// The creatures blocking `attacker` now.
fn blockers_of(t: &TestGame, attacker: ObjectId) -> Vec<ObjectId> {
    crate::r_s21_common::blocks_now(t)
        .into_iter()
        .filter(|(_, a)| *a == attacker)
        .map(|(b, _)| b)
        .collect()
}

/// P1 casts Flash Foliage at `attacker` (in the declare blockers step) and resolves it.
/// Returns the Saproling, if it's still on the battlefield.
fn foliage(t: &mut TestGame, attacker: ObjectId) -> Option<ObjectId> {
    let before = tokens(t, P1);
    cast_card(t, P1, "Flash Foliage", &[obj(attacker)]).unwrap();
    t.resolve_all();
    tokens(t, P1).into_iter().find(|x| !before.contains(x))
}

#[test]
fn flash_foliage_saproling_blocks_ignoring_restrictions_and_block_triggers() {
    cr!("509.4", "509.1h");
    ruling!(
        "Flash Foliage",
        "Putting a blocking creature onto the battlefield doesn't trigger \"When this creature blocks\" abilities. It also won't check blocking restrictions, costs, or requirements."
    );
    ruling!(
        "Flash Foliage",
        "The Saproling token is blocking the attacking creature, even if the block couldn't legally be declared (for example, if the attacking creature has flying, or is Vindictive Mob)."
    );
    ruling!("Flash Foliage", "The target creature can be blocked by other creatures.");
    supported("Flash Foliage");
    supported("Noble Stand");
    // P0's Serra Angel (flying) attacks unblocked and P0's Craw Wurm is blocked by a
    // Grizzly Bears; P1 controls Noble Stand ("Whenever a creature you control blocks,
    // you gain 2 life").
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Noble Stand");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let angel = t.battlefield(P0, "Serra Angel");
    let wurm = t.battlefield(P0, "Craw Wurm");
    to_blockers(
        &mut t,
        &[(angel, Entity::Player(P1)), (wurm, Entity::Player(P1))],
        &[(bears, wurm)],
    );
    t.resolve_all();
    assert_eq!(t.life(P1), 22, "the Bears' block");
    let s1 = foliage(&mut t, angel).unwrap();
    assert!(blocking(&t, s1));
    assert_eq!(blockers_of(&t, angel), vec![s1]);
    let s2 = foliage(&mut t, wurm).unwrap();
    let mut b = blockers_of(&t, wurm);
    b.sort();
    assert_eq!(b, vec![bears, s2]);
    t.resolve_all();
    assert_eq!(t.life(P1), 22, "no block triggers for the Saprolings");
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 22, "the Angel is blocked: no damage to P1");
}

#[test]
fn flash_foliage_triggers_becomes_blocked_by_a_creature_and_first_becomes_blocked() {
    cr!("509.4", "509.3c", "509.3d");
    ruling!(
        "Flash Foliage",
        "Putting a blocking creature onto the battlefield will trigger \"When this creature becomes blocked by a creature\" abilities. It may trigger \"When this creature becomes blocked\" abilities, but only if the creature the Saproling is blocking had not yet been blocked that combat."
    );
    // Engulfing Slagwurm: "Whenever this creature blocks or becomes blocked by a creature,
    // destroy that creature."
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P0, "Engulfing Slagwurm");
    to_blockers(&mut t, &[(wurm, Entity::Player(P1))], &[]);
    assert_eq!(foliage(&mut t, wurm), None, "destroyed by the Slagwurm's trigger");
    assert_eq!(t.life(P0), 21, "the Slagwurm's controller gained 1 life");
    // Elvish Berserker: "Whenever this creature becomes blocked, it gets +1/+1 ...":
    // unblocked before, it triggers; already blocked, it doesn't.
    for already in [false, true] {
        let mut t = TestGame::new(2);
        let bers = t.battlefield(P0, "Elvish Berserker");
        let wall = t.battlefield(P1, "Wall of Omens");
        let blocks: &[(ObjectId, ObjectId)] = if already { &[(wall, bers)] } else { &[] };
        to_blockers(&mut t, &[(bers, Entity::Player(P1))], blocks);
        t.resolve_all();
        let pt = t.pt(bers);
        foliage(&mut t, bers);
        let gained = t.pt(bers).0 - pt.0;
        assert_eq!(gained, if already { 0 } else { 1 }, "already: {already}");
    }
}

#[test]
fn mirror_match_noncreature_copies_dont_block() {
    cr!("509.4", "509.1h", "707.2");
    ruling!(
        "Mirror Match",
        "If the tokens aren’t creatures (perhaps because the attacking creature was an animated land), they’ll enter the battlefield but they won’t be blocking. Attacking creatures that weren’t otherwise blocked will remain unblocked."
    );
    supported("Mirror Match");
    // P0's animated Mutavault and Grizzly Bears attack P1.
    let mut t = TestGame::new(2);
    let vault = t.battlefield(P0, "Mutavault");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.g.combat = None;
    t.set_step(P0, Step::BeginningOfCombat);
    add(&mut t, P0, mtg_engine::mana::ManaType::C, 1);
    crate::r_s06_common::activate_containing(&mut t, P0, vault, "becomes").expect("animated");
    t.resolve_all();
    to_blockers(
        &mut t,
        &[(vault, Entity::Player(P1)), (bears, Entity::Player(P1))],
        &[],
    );
    cast_card(&mut t, P1, "Mirror Match", &[]).unwrap();
    t.resolve_all();
    let toks = tokens(&t, P1);
    assert_eq!(toks.len(), 2);
    let land_copy = *toks
        .iter()
        .find(|x| t.obj_now(**x).chars.name == "Mutavault")
        .unwrap();
    assert!(!blocking(&t, land_copy));
    assert!(blockers_of(&t, vault).is_empty());
    assert_eq!(blockers_of(&t, bears).len(), 1);
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18, "the Mutavault stayed unblocked");
}

/// P0 controls Leyline of Anticipation (cast spells as though they had flash) and has
/// Response // Resurgence in hand with mana for Resurgence; casts Resurgence now.
fn cast_resurgence(t: &mut TestGame) {
    t.battlefield(P0, "Leyline of Anticipation");
    let c = t.hand(P0, "Response // Resurgence");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Wastes", 3);
    t.cast(P0, c).method(CastMethod::Half(1)).go();
    t.resolve_all();
}

#[test]
fn resurgence_outside_a_main_phase_or_in_an_opponents() {
    cr!("500.8", "505.1a", "506.1");
    ruling!(
        "Response // Resurgence",
        "If you cast Resurgence when it's not a main phase (perhaps with Omnispell Adept's ability), creatures you control gain first strike and vigilance, but there's no additional combat phase or main phase. If you cast it during an opponent's main phase, there are additional combat and main phases, but that player attacks during that combat phase, not you."
    );
    supported("Response // Resurgence");
    supported("Leyline of Anticipation");
    // In P0's end step: first strike, no extra phases.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::End);
    cast_resurgence(&mut t);
    assert!(t.obj_now(bears).has_keyword(KeywordKind::FirstStrike));
    assert!(!t.g.turn.schedule.contains(&Step::BeginningOfCombat));
    // In P1's postcombat main phase: P1 gets an additional combat phase and attacks.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PostcombatMain);
    t.g.turn.combat_phases = 1;
    cast_resurgence(&mut t);
    let attacks = declare_in_next_combat(&mut t, P1, &[(theirs, Entity::Player(P0))]);
    assert_eq!(attacks, vec![(theirs, Entity::Player(P0))]);
    assert_eq!(t.g.turn.active, P1);
}

#[test]
fn resurgence_affects_only_creatures_you_control_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Response // Resurgence",
        "Resurgence affects only creatures you control at the time it resolves. Creatures that come under your control later in the turn won't gain first strike and vigilance."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_resurgence(&mut t);
    let later = t.battlefield(P0, "Hill Giant");
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Vigilance));
    assert!(!t.obj_now(later).has_keyword(KeywordKind::FirstStrike));
    assert!(!t.obj_now(later).has_keyword(KeywordKind::Vigilance));
}

#[test]
fn take_the_bait_adds_a_combat_phase_without_a_main_phase() {
    cr!("500.8", "505.1a");
    ruling!(
        "Take the Bait",
        "Take the Bait doesn't give the opponent whose turn it is an additional main phase. They will move directly from the end of combat step of one combat phase to the beginning of combat step of the next one."
    );
    supported("Take the Bait");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(bears, Entity::Player(P0))]);
    cast_card(&mut t, P0, "Take the Bait", &[]).unwrap();
    t.resolve_all();
    go_to(&mut t, Step::EndOfCombat);
    assert_eq!(t.life(P0), 20, "prevented");
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step != Step::EndOfCombat && g.turn.stage == Stage::Priority
    });
    assert!(ok);
    assert_eq!(t.g.turn.step, Step::BeginningOfCombat);
    assert_eq!(t.g.turn.active, P1);
}

#[test]
fn watchdog_penalty_follows_its_tapped_status() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Watchdog",
        "The second ability is no longer triggered. It continuously checks the tapped/untapped status of the Watchdog, and gives them -1/-0 as appropriate."
    );
    supported("Watchdog");
    let mut t = TestGame::new(2);
    let dog = t.battlefield(P1, "Watchdog");
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 0, "nothing triggered");
    assert_eq!(t.pt(bears), (1, 2));
    t.g.objects[dog.0 as usize].tapped = true;
    t.g.recompute();
    assert_eq!(t.pt(bears), (2, 2));
    t.g.objects[dog.0 as usize].tapped = false;
    t.g.recompute();
    assert_eq!(t.pt(bears), (1, 2));
}

#[test]
fn clinging_mists_already_tapped_attackers_still_dont_untap() {
    cr!("502.3", "506.4b");
    ruling!(
        "Clinging Mists",
        "In most cases, attacking creatures will already be tapped. They still won’t untap during their controller’s next untap step."
    );
    supported("Clinging Mists");
    let mut t = TestGame::new(2);
    t.g.players[1].life = 5;
    let bears = t.battlefield(P0, "Grizzly Bears");
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(t.obj_now(bears).tapped);
    cast_card(&mut t, P1, "Clinging Mists", &[]).unwrap();
    t.resolve_all();
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(bears).tapped, "didn't untap");
    assert_eq!(t.life(P1), 5);
}

#[test]
fn two_headed_giant_metzali_and_cleaving_skyrider() {
    cr!("810.9", "810.7");
    ruling!(
        "Path of Mettle // Metzali, Tower of Triumph",
        "In a Two-Headed Giant game, Metzali's second ability causes the opposing team to lose 4 life."
    );
    ruling!(
        "Cleaving Skyrider",
        "In a Two-Headed Giant game, this triggered ability counts all attacking creatures that each attacking team controls."
    );
    supported("Cleaving Skyrider");
    let mut t = two_headed_giant();
    let start = t.life(P2);
    let metzali = back_face(&mut t, P0, "Path of Mettle // Metzali, Tower of Triumph");
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 1);
    crate::r_s06_common::activate_containing(&mut t, P0, metzali, "each opponent")
        .expect("activated");
    t.resolve_all();
    assert_eq!((t.life(P2), t.life(P3)), (start - 4, start - 4));
    // P2 and P3 each attack with a creature; P0's kicked Skyrider deals 2.
    let mut t = two_headed_giant();
    let a = t.battlefield(P2, "Grizzly Bears");
    let b = t.battlefield(P3, "Savannah Lions");
    t.set_step(P2, Step::BeginningOfCombat);
    attack_with(&mut t, &[(a, Entity::Player(P0)), (b, Entity::Player(P0))]);
    t.resolve_all();
    assert_eq!(attacks_now(&t).len(), 2);
    let sky = t.hand(P0, "Cleaving Skyrider");
    t.lands(P0, "Plains", 1);
    t.lands(P0, "Mountain", 1);
    t.lands(P0, "Wastes", 4);
    t.answer_targets(P0, &[obj(a)]);
    t.cast(P0, sky).kicked(true).go();
    t.resolve_all();
    assert!(!t.on_battlefield(a), "2 damage to the 2/2");
}

#[test]
fn sandswirl_wanderglyph_attack_prohibition_outlasts_it() {
    cr!("611.2a", "508.1c");
    ruling!(
        "Unstable Glyphbridge // Sandswirl Wanderglyph",
        "Once Sandswirl Wanderglyph's middle ability resolves, that opponent won't be able to attack you or planeswalkers you control that turn, even if Sandswirl Wanderglyph is no longer on the battlefield."
    );
    let mut t = TestGame::new(3);
    let glyph = back_face(&mut t, P0, "Unstable Glyphbridge // Sandswirl Wanderglyph");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    cast_card(&mut t, P1, "Shock", &[Entity::Player(P2)]).unwrap();
    t.resolve_all();
    crate::r_s02_common::destroy(&mut t, glyph);
    t.g.combat = None;
    t.set_step(P1, Step::BeginningOfCombat);
    assert!(!crate::r_s09_common::legal_attack(
        &mut t,
        &[(bears, Entity::Player(P0))]
    ));
    assert!(crate::r_s09_common::legal_attack(
        &mut t,
        &[(bears, Entity::Player(P2))]
    ));
}

/// P0's creature `attacker` attacks P1 (three players); P0 flashes in Misleading Signpost
/// and sends it at P2.
fn signpost(t: &mut TestGame, attacker: ObjectId) {
    t.set_step(P0, Step::BeginningOfCombat);
    attack_with(t, &[(attacker, Entity::Player(P1))]);
    t.resolve_all();
    crate::r_s01_common::give_mana_for(t, P0, "Misleading Signpost");
    let c = t.hand(P0, "Misleading Signpost");
    t.answer_targets(P0, &[obj(attacker)]);
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.cast(P0, c).go();
    t.resolve();
    t.resolve();
}

fn target_of(t: &TestGame, a: ObjectId) -> Option<Entity> {
    attacks_now(t)
        .into_iter()
        .find(|(x, _)| *x == a)
        .map(|(_, e)| e)
}

#[test]
fn misleading_signpost_reselects_without_attacking_again() {
    cr!("508.7", "508.7a", "508.7b", "508.7c");
    ruling!(
        "Misleading Signpost",
        "If you reselect which player or permanent an attacking creature is attacking, that creature is still considered to have attacked the player or permanent as declared, but it is now attacking the new player or permanent."
    );
    ruling!(
        "Misleading Signpost",
        "Reselecting which player or permanent a creature is attacking doesn't cause \"whenever this creature attacks\" abilities to trigger."
    );
    ruling!(
        "Windshaper Planetar",
        "Reselecting which player or planeswalker a creature is attacking doesn't cause “whenever this creature attacks” abilities to trigger."
    );
    ruling!(
        "Portal Mage",
        "Reselecting which player or planeswalker a creature is attacking doesn’t cause “whenever this creature attacks” abilities to trigger. Notably, the Curses in this set (such as Curse of Vitality) won’t trigger while Portal Mage’s ability is resolving."
    );
    supported("Misleading Signpost");
    supported("Hero of Bladehold");
    supported("Curse of Vitality");
    // P0's Hero of Bladehold ("Whenever this creature attacks, create two 1/1 ...
    // Soldiers ... tapped and attacking") attacks P1, then is sent at P2, who is enchanted
    // by P1's Curse of Vitality ("Whenever enchanted player is attacked, you gain 2 life").
    let mut t = TestGame::new(3);
    let hero = t.battlefield(P0, "Hero of Bladehold");
    crate::r_s06_common::attach_new(&mut t, P1, "Curse of Vitality", Entity::Player(P2));
    signpost(&mut t, hero);
    let soldiers = tokens(&t, P0).len();
    assert_eq!(soldiers, 2, "the Hero's declared attack");
    let life = t.life(P1);
    t.resolve_all();
    assert_eq!(target_of(&t, hero), Some(Entity::Player(P2)));
    assert_eq!(tokens(&t, P0).len(), 2, "no new attack trigger");
    assert_eq!(t.life(P1), life, "the Curse didn't trigger");
    assert!(player_has_attacked(&t.g, P0, P1));
    assert!(!player_has_attacked(&t.g, P0, P2));
    assert!(player_is_attacking(&t.g, P0, P2));
    assert!(attacking(&t, hero));
    // The same with Windshaper Planetar and Portal Mage.
    for name in ["Windshaper Planetar", "Portal Mage"] {
        supported(name);
        let mut t = TestGame::new(3);
        let hero = t.battlefield(P0, "Hero of Bladehold");
        t.set_step(P0, Step::BeginningOfCombat);
        attack_with(&mut t, &[(hero, Entity::Player(P1))]);
        t.resolve_all();
        let before = tokens(&t, P0).len();
        crate::r_s01_common::give_mana_for(&mut t, P0, name);
        let c = t.hand(P0, name);
        if name == "Portal Mage" {
            t.answer_targets(P0, &[obj(hero)]);
        }
        t.answer_choose(P0, &[Entity::Player(P2)]);
        t.cast(P0, c).go();
        t.resolve_all();
        assert_eq!(target_of(&t, hero), Some(Entity::Player(P2)), "{name}");
        assert_eq!(tokens(&t, P0).len(), before, "{name}");
    }
}

#[test]
fn misleading_signpost_ignores_attack_costs_and_restrictions() {
    cr!("508.7b", "508.1h");
    ruling!(
        "Misleading Signpost",
        "Reselecting which player or permanent a creature is attacking ignores all requirements, restrictions, and costs associated with attacking."
    );
    supported("Ghostly Prison");
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P2, "Ghostly Prison");
    t.lands(P0, "Wastes", 2);
    signpost(&mut t, bears);
    assert_eq!(target_of(&t, bears), Some(Entity::Player(P2)));
    assert_eq!(crate::r_s04_common::untapped_lands(&t, P0), 2, "nothing paid");
}
