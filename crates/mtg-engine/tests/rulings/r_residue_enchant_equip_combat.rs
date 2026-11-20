//! Rulings residue (batches P207–P213, Auras and Equipment): combat — Curse of Hospitality,
//! Errantry, Chorale of the Void, Farrel's Mantle, Venom, Indrik Umbra, Infinite Authority,
//! Godsend, Nuka-Nuke Launcher, Ward of Piety, Hexplate Wallbreaker, Dire Blunderbuss,
//! Indoraptor — and tapping and upkeep triggers (Spirit Shackle, Artificer's Hex).

use crate::r_p056_common::steps_of_this_turn;
use crate::r_p209_common::{tap, untap};
use crate::r_p212_common::{back_face, boost};
use crate::r_residue_enchant_equip_common::*;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, can_play_land, create_token, destroy};
use crate::r_s03_common::to_blockers;
use crate::r_s06_common::*;
use crate::r_s26_common::modify_until_eot;
use mtg_engine::ability::Modification;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Whether `id` (followed) is blocking now.
fn blocking(t: &TestGame, id: ObjectId) -> bool {
    let id = t.g.current(id);
    t.g.combat.as_ref().is_some_and(|c| {
        c.blockers
            .iter()
            .any(|b| b.id == id && !b.blocking.is_empty())
    })
}

/// What `id` (followed) is attacking now.
fn attacking(t: &TestGame, id: ObjectId) -> Option<Entity> {
    let id = t.g.current(id);
    t.g.combat
        .as_ref()?
        .attackers
        .iter()
        .find(|a| a.id == id)
        .and_then(|a| a.target)
}

#[test]
fn curse_of_hospitality_triggers_for_each_creature() {
    cr!("510.2", "603.2c");
    ruling!(
        "Curse of Hospitality",
        "Curse of Hospitality's triggered ability triggers once for each creature that dealt combat damage to the enchanted player."
    );
    supported("Curse of Hospitality");
    let mut t = TestGame::new(2);
    aura_on(&mut t, P0, "Curse of Hospitality", P1);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    let lib = t.library_size(P1);
    t.attack(&[(a, Entity::Player(P1)), (b, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert_eq!(t.life(P1), 15);
    assert_eq!(t.library_size(P1), lib - 2, "{}", t.dump_log());
}

#[test]
fn errantry_another_creature_put_onto_the_battlefield_attacking_is_fine() {
    cr!("506.5", "508.4", "508.1c");
    ruling!(
        "Errantry",
        "If the enchanted creature attacks alone, then a spell or ability causes another creature to be put onto the battlefield attacking, that’s fine."
    );
    ruling!(
        "Chorale of the Void",
        "You choose the player, planeswalker, or battle the creature you put onto the battlefield is attacking. It doesn’t have to be the same player, planeswalker, or battle that the enchanted creature or any other attacking creatures are attacking."
    );
    supported("Errantry");
    supported("Chorale of the Void");
    supported("Liliana of the Veil");
    // Chorale of the Void: "Whenever enchanted creature attacks, put target creature card
    // from defending player's graveyard onto the battlefield under your control tapped and
    // attacking."
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    aura_on(&mut t, P0, "Errantry", bears);
    aura_on(&mut t, P0, "Chorale of the Void", bears);
    let lili = t.battlefield(P1, "Liliana of the Veil");
    let giant = t.graveyard(P1, "Hill Giant");
    t.answer_targets(P0, &[obj(giant)]);
    t.answer_choose(P0, &[obj(lili)]);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    let g = t.g.current(giant);
    assert!(t.on_battlefield(g), "{}", t.dump_log());
    assert_eq!(t.obj_now(g).controller, P0);
    assert_eq!(attacking(&t, g), Some(obj(lili)));
    assert_eq!(attacking(&t, bears), Some(Entity::Player(P1)));
}

#[test]
fn errantry_attacks_alone_including_a_two_headed_giant_teammate() {
    cr!("506.5", "508.1c", "805.10a");
    ruling!(
        "Errantry",
        "“Can only attack alone” means the enchanted creature can’t be declared as an attacker during the declare attackers step unless no other creatures are declared as attackers at that time (either by you or your Two-Headed Giant teammate)."
    );
    let mut t = crate::r_p014_common::thg();
    t.set_step(P0, Step::BeginningOfCombat);
    let bears = t.battlefield(P0, "Grizzly Bears");
    aura_on(&mut t, P0, "Errantry", bears);
    let mine = t.battlefield(P0, "Hill Giant");
    let mate = t.battlefield(P1, "Hill Giant");
    t.set_step(P0, Step::BeginningOfCombat);
    let options = mtg_engine::combat::attack_options(&t.g);
    let legal = |decl: &[(ObjectId, Entity)]| {
        mtg_engine::combat::attack_declaration_legal(&t.g, &options, decl)
    };
    assert!(legal(&[(bears, Entity::Player(P2))]));
    assert!(legal(&[
        (mine, Entity::Player(P2)),
        (mate, Entity::Player(P2))
    ]));
    assert!(!legal(&[
        (bears, Entity::Player(P2)),
        (mine, Entity::Player(P2))
    ]));
    assert!(!legal(&[
        (bears, Entity::Player(P2)),
        (mate, Entity::Player(P2))
    ]));
}

// --- Farrel's Mantle ----------------------------------------------------------------------

const FARREL: &str = "Farrel's Mantle";

#[test]
fn farrels_mantle_no_combat_damage_is_not_prevention() {
    cr!("510.1", "615.12");
    ruling!(
        "Farrel's Mantle",
        "If the enchanted creature’s controller chooses to have it deal no combat damage, that combat damage is not prevented. It’s simply not dealt. A creature whose damage can’t be prevented (such as Excruciator) would deal no combat damage this way."
    );
    supported(FARREL);
    supported("Excruciator");
    let mut t = TestGame::new(2);
    let ex = t.battlefield(P0, "Excruciator");
    aura_on(&mut t, P0, FARREL, ex);
    let wurm = t.battlefield(P1, "Craw Wurm");
    t.answer_targets(P0, &[obj(wurm)]);
    t.answer_yes(P0, true);
    t.attack(&[(ex, Entity::Player(P1))], &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(wurm), "{}", t.dump_log());
    assert_eq!(t.life(P1), 20);
}

#[test]
fn farrels_mantle_two_mantles_one_use_is_enough() {
    cr!("510.1", "603.2");
    ruling!(
        "Farrel's Mantle",
        "If the same creature is enchanted with more than one Farrel’s Mantle, they’ll each trigger if that creature attacks and isn’t blocked."
    );
    // Both trigger (two choices). Used once: no combat damage. Used twice: the damage is
    // dealt twice, and still no combat damage.
    for twice in [false, true] {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        aura_on(&mut t, P0, FARREL, bears);
        aura_on(&mut t, P0, FARREL, bears);
        let wall = t.battlefield(P1, "Wall of Stone");
        t.answer_targets(P0, &[obj(wall)]);
        t.answer_targets(P0, &[obj(wall)]);
        t.answer_yes(P0, true);
        t.answer_yes(P0, twice);
        let from = t.asked().len();
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        t.advance_to(P0, Step::EndOfCombat);
        t.resolve_all();
        let choices = t.asked()[from..]
            .iter()
            .filter(|(p, d)| *p == P0 && matches!(d, mtg_engine::decision::Decision::YesNo { .. }))
            .count();
        assert_eq!(choices, 2, "{}", t.dump_log());
        if twice {
            // 4 + 4 damage to the 0/8 Wall.
            assert!(!t.on_battlefield(wall), "{}", t.dump_log());
        } else {
            assert_eq!(t.obj_now(wall).damage, 4, "{}", t.dump_log());
        }
        assert_eq!(t.life(P1), 20);
    }
}

// --- Venom, Indrik Umbra, Infinite Authority, Godsend, Nuka-Nuke Launcher ------------------

#[test]
fn venom_moved_after_blocks_doesnt_affect_the_blockers() {
    cr!("509.3d", "603.2e");
    ruling!(
        "Venom",
        "If this Aura is moved onto a creature after blockers are declared, creatures blocking the newly-enchanted creature are not affected."
    );
    supported("Venom");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let wurm = t.battlefield(P0, "Craw Wurm");
    let venom = aura_on(&mut t, P0, "Venom", wurm);
    let bears = t.battlefield(P0, "Grizzly Bears");
    boost(&mut t, bears, 0, 5);
    let blocker = t.battlefield(P1, "Hill Giant");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(blocker, bears)]);
    assert!(t.g.attach(venom, obj(bears)));
    t.g.recompute();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(t.on_battlefield(blocker), "{}", t.dump_log());
}

#[test]
fn indrik_umbra_forces_only_creatures_able_to_block() {
    cr!("509.1c", "509.1b");
    ruling!(
        "Indrik Umbra",
        "Indrik Umbra does not give any creature the ability to block the enchanted creature. It just forces those creatures which are already able to block that creature to do so."
    );
    supported("Indrik Umbra");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let attacker = t.battlefield(P0, "Grizzly Bears");
    aura_on(&mut t, P0, "Indrik Umbra", attacker);
    let raider = t.battlefield(P1, "Goblin Raider");
    let bears = t.battlefield(P1, "Grizzly Bears");
    to_blockers(&mut t, &[(attacker, Entity::Player(P1))], &[]);
    assert!(blocking(&t, bears), "{}", t.dump_log());
    assert!(!blocking(&t, raider));
}

#[test]
fn infinite_authority_one_counter_for_each_destroyed_creature() {
    cr!("603.7", "509.3d");
    ruling!(
        "Infinite Authority",
        "The enchanted creature gets a +1/+1 counter for each creature that triggers the ability."
    );
    supported("Infinite Authority");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    // A 0/6 Craw Wurm: blocked by two Bears that survive the combat damage.
    let wurm = t.battlefield(P0, "Craw Wurm");
    aura_on(&mut t, P0, "Infinite Authority", wurm);
    boost(&mut t, wurm, -6, 2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    to_blockers(
        &mut t,
        &[(wurm, Entity::Player(P1))],
        &[(a, wurm), (b, wurm)],
    );
    t.resolve_all();
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert!(
        !t.on_battlefield(a) && !t.on_battlefield(b),
        "{}",
        t.dump_log()
    );
    assert_eq!(t.counters(wurm, counters::PLUS1), 2);
}

#[test]
fn godsend_triggers_once_however_many_blockers() {
    cr!("509.3e", "509.3a", "509.3c");
    ruling!(
        "Godsend",
        "Godsend's triggered ability will trigger only once no matter how many creatures the equipped creature blocks or becomes blocked by."
    );
    supported("Godsend");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let wurm = t.battlefield(P0, "Craw Wurm");
    attach_new(&mut t, P0, "Godsend", wurm);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    to_blockers(
        &mut t,
        &[(wurm, Entity::Player(P1))],
        &[(a, wurm), (b, wurm)],
    );
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
}

#[test]
fn hedron_blade_blocked_by_one_or_more_triggers_once_if_one_matches() {
    // "Becomes blocked by one or more colorless creatures" triggers once, as the creature
    // first has a colorless blocker (CR 509.3e), not once for each (CR 509.3d).
    cr!("509.3e", "509.3f");
    supported("Hedron Blade");
    for (blockers, n) in [
        (["Ornithopter", "Ornithopter"], 1),
        (["Grizzly Bears", "Ornithopter"], 1),
        (["Grizzly Bears", "Grizzly Bears"], 0),
    ] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::BeginningOfCombat);
        let wurm = t.battlefield(P0, "Craw Wurm");
        attach_new(&mut t, P0, "Hedron Blade", wurm);
        let a = t.battlefield(P1, blockers[0]);
        let b = t.battlefield(P1, blockers[1]);
        to_blockers(
            &mut t,
            &[(wurm, Entity::Player(P1))],
            &[(a, wurm), (b, wurm)],
        );
        assert_eq!(t.stack_len(), n, "{blockers:?}: {}", t.dump_log());
        t.resolve_all();
        assert_eq!(
            has_kw(&t, wurm, mtg_engine::keywords::KeywordKind::Deathtouch),
            n == 1
        );
    }
    // Blocked by a green creature, then an effect has a colorless creature block it too:
    // it triggers then (CR 509.3e); another colorless blocker doesn't trigger it again.
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let wurm = t.battlefield(P0, "Craw Wurm");
    attach_new(&mut t, P0, "Hedron Blade", wurm);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let a = t.battlefield(P1, "Ornithopter");
    let b = t.battlefield(P1, "Ornithopter");
    to_blockers(&mut t, &[(wurm, Entity::Player(P1))], &[(bears, wurm)]);
    assert_eq!(t.stack_len(), 0, "{}", t.dump_log());
    assert!(mtg_engine::combat::block_by_effect(&mut t.g, a, wurm));
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.resolve_all();
    assert!(has_kw(
        &t,
        wurm,
        mtg_engine::keywords::KeywordKind::Deathtouch
    ));
    assert!(mtg_engine::combat::block_by_effect(&mut t.g, b, wurm));
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 0, "{}", t.dump_log());
}

#[test]
fn godsend_exiled_land_card_can_still_be_played() {
    cr!("305.1", "601.3");
    ruling!(
        "Godsend",
        "opponents can still play land cards with the same name as the exiled card. Godsend stops only the casting of spells."
    );
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let village = t.battlefield(P1, "Treetop Village");
    t.lands(P1, "Forest", 2);
    activate_containing(&mut t, P1, village, "becomes").expect("animate");
    t.resolve_all();
    // (It may have tapped itself for mana.)
    t.g.obj_mut(village).tapped = false;
    let wurm = t.battlefield(P0, "Craw Wurm");
    attach_new(&mut t, P0, "Godsend", wurm);
    // P1 attacks with the Village; P0 blocks with the equipped Wurm and exiles it.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(village)]);
    to_blockers(&mut t, &[(village, Entity::Player(P0))], &[(wurm, village)]);
    t.resolve_all();
    assert!(t.in_exile("Treetop Village"), "{}", t.dump_log());
    // P1 can play another Treetop Village.
    t.set_step(P1, Step::PrecombatMain);
    let card = t.hand(P1, "Treetop Village");
    assert!(can_play_land(&mut t, P1, card));
}

#[test]
fn nuka_nuke_launcher_blocked_stays_blocked() {
    cr!("509.1h", "702.13b");
    ruling!(
        "Nuka-Nuke Launcher",
        "Once the equipped creature has been blocked, changing its color or the blocking creature’s color or stopping the blocking creature from being an artifact won’t cause the equipped creature to become unblocked."
    );
    supported("Nuka-Nuke Launcher");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Nuka-Nuke Launcher", bears);
    let thopter = t.battlefield(P1, "Ornithopter");
    to_blockers(&mut t, &[(bears, Entity::Player(P1))], &[(thopter, bears)]);
    assert!(blocking(&t, thopter));
    modify_until_eot(
        &mut t,
        thopter,
        vec![Modification::RemoveTypes(vec![CardType::Artifact])],
    );
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 20, "{}", t.dump_log());
    assert!(!t.on_battlefield(thopter));
}

// --- Ward of Piety ------------------------------------------------------------------------

#[test]
fn ward_of_piety_redirected_combat_damage_is_still_combat_damage() {
    cr!("614.9", "510.2");
    ruling!(
        "Ward of Piety",
        "Redirected damage retains its information."
    );
    supported("Ward of Piety");
    supported("Slith Firewalker");
    let mut t = TestGame::new(2);
    t.set_step(P1, Step::BeginningOfCombat);
    let slith = t.battlefield(P1, "Slith Firewalker");
    let bears = t.battlefield(P0, "Wall of Stone");
    let ward = aura_on(&mut t, P0, "Ward of Piety", bears);
    to_blockers(&mut t, &[(slith, Entity::Player(P0))], &[(bears, slith)]);
    t.lands(P0, "Plains", 2);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    activate_containing(&mut t, P0, ward, "next 1 damage").expect("activate");
    t.resolve_all();
    t.advance_to(P1, Step::EndOfCombat);
    t.resolve_all();
    assert_eq!(t.life(P1), 19, "{}", t.dump_log());
    assert_eq!(t.obj_now(bears).damage, 0);
    assert_eq!(t.counters(slith, counters::PLUS1), 1, "{}", t.dump_log());
}

// --- Hexplate Wallbreaker, Dire Blunderbuss, Indoraptor -----------------------------------

#[test]
fn hexplate_wallbreaker_no_main_phase_between_the_combats() {
    cr!("500.8", "602.5d", "307.1");
    ruling!(
        "Hexplate Wallbreaker",
        "There's no main phase between your combat phases, so you'll have no opportunity to cast spells or activate abilities that could only be activated any time you could cast a sorcery."
    );
    supported("Hexplate Wallbreaker");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::BeginningOfCombat);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let hex = attach_new(&mut t, P0, "Hexplate Wallbreaker", bears);
    t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Mountain", 4);
    attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    // The next step is the next combat's beginning of combat step, where the equip ability
    // (sorcery speed) can't be activated.
    crate::r_p056_common::next_step(&mut t, Step::BeginningOfCombat);
    assert!(!can_activate(&mut t, P0, hex));
    let steps = steps_of_this_turn(&mut t);
    let first_end = steps.iter().position(|s| *s == Step::EndOfCombat).unwrap();
    assert_eq!(steps[first_end + 1], Step::BeginningOfCombat, "{steps:?}");
}

#[test]
fn dire_blunderbuss_uses_the_power_as_the_reflexive_trigger_resolves() {
    cr!("603.12", "608.2h", "113.7a");
    ruling!(
        "Dire Flail // Dire Blunderbuss",
        "Use the equipped creature's power as the reflexive triggered ability granted by Dire Blunderbuss resolves to determine how much damage is dealt. If the equipped creature is no longer on the battlefield at that time, use its power as it last existed on the battlefield."
    );
    for gone in [false, true] {
        let mut t = TestGame::new(2);
        t.set_step(P0, Step::BeginningOfCombat);
        let bears = t.battlefield(P0, "Grizzly Bears");
        let blunderbuss = back_face(&mut t, P0, "Dire Flail // Dire Blunderbuss");
        assert_eq!(name(&t, blunderbuss), "Dire Blunderbuss");
        assert!(t.g.attach(blunderbuss, obj(bears)));
        t.g.recompute();
        let thopter = t.battlefield(P0, "Ornithopter");
        let wall = t.battlefield(P1, "Wall of Stone");
        t.answer_yes(P0, true);
        t.answer_choose(P0, &[obj(thopter)]);
        t.answer_targets(P0, &[obj(wall)]);
        attack_with(&mut t, &[(bears, Entity::Player(P1))]);
        // The attack trigger resolves: the Ornithopter is sacrificed; the reflexive
        // trigger waits.
        t.g.resolve_top();
        t.settle();
        assert!(!t.on_battlefield(thopter));
        assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
        boost(&mut t, bears, 2, 0);
        // A 7/2 now (2 + 3 from the Blunderbuss + 2): 5 as the ability triggered.
        if gone {
            // It leaves: its power as it last existed.
            destroy(&mut t, bears);
        }
        t.g.resolve_top();
        t.settle();
        assert_eq!(t.obj_now(wall).damage, 7, "gone {gone}: {}", t.dump_log());
    }
}

#[test]
fn indoraptor_lethal_damage_still_triggers_with_its_last_power() {
    cr!("603.10", "704.5g", "608.2h");
    ruling!(
        "Indoraptor, the Perfect Hybrid",
        "If lethal damage is dealt to Indoraptor, its enrage ability triggers. Use Indoraptor's power from when it was last on the battlefield to determine how much damage it deals to the chosen player if they don't sacrifice a nontoken creature."
    );
    supported("Indoraptor, the Perfect Hybrid");
    let mut t = TestGame::new(2);
    // A 5/3 Indoraptor is dealt 3 damage by P1's Rod of Ruin; P1 has only a token
    // creature, so can't sacrifice a nontoken one: Indoraptor (gone) deals 5 damage.
    let raptor = t.battlefield(P0, "Indoraptor, the Perfect Hybrid");
    t.g.add_counters(obj(raptor), counters::PLUS1, 2, None);
    let src = t.battlefield(P1, "Rod of Ruin");
    create_token(&mut t, P1, "Soldier");
    damage(&mut t, src, 3, raptor);
    assert!(!t.on_battlefield(raptor));
    t.resolve_all();
    assert_eq!(t.life(P1), 15, "{}", t.dump_log());
}

// --- Spirit Shackle, Artificer's Hex ------------------------------------------------------

#[test]
fn spirit_shackle_triggers_only_on_becoming_tapped() {
    cr!("603.2", "701.26a");
    ruling!(
        "Spirit Shackle",
        "Spirit Shackle's ability triggers only when the enchanted creature changes from untapped to tapped. If Spirit Shackle enters attached to an already tapped creature, it won't do anything"
    );
    supported("Spirit Shackle");
    let mut t = TestGame::new(2);
    t.set_step(P0, Step::PrecombatMain);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.obj_mut(giant).tapped = true;
    crate::r_p209_common::cast_aura(&mut t, P0, "Spirit Shackle", giant);
    t.resolve_all();
    assert_eq!(t.counters(giant, "-0/-2"), 0);
    untap(&mut t, giant);
    tap(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.counters(giant, "-0/-2"), 1, "{}", t.dump_log());
    assert_eq!(t.pt(giant), (3, 1));
}

#[test]
fn artificers_hex_destroys_the_creature_equipped_as_it_resolves() {
    cr!("608.2h", "603.4");
    ruling!(
        "Artificer's Hex",
        "In such a case, the creature the Equipment is attached to when the ability resolves is destroyed."
    );
    supported("Artificer's Hex");
    let mut t = TestGame::new(2);
    let first = t.battlefield(P1, "Grizzly Bears");
    let second = t.battlefield(P1, "Hill Giant");
    let split = attach_new(&mut t, P1, "Bonesplitter", first);
    aura_on(&mut t, P0, "Artificer's Hex", split);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    assert!(t.g.attach(split, obj(second)));
    t.g.recompute();
    t.resolve_all();
    assert!(t.on_battlefield(first));
    assert!(!t.on_battlefield(second));
}
