//! Rulings batch P165 — commander synergies (CR 903): the Will of the clans cycle,
//! Jeska's Will, Guardian Augmenter, Commander's Insignia, Road of Return, Crimson Honor
//! Guard, Geode Golem, Jirina Kudro, Myth Unbound, Thunderclap Drake, Forge of Heroes,
//! Chain Assassination, Tome of Legends, Drillworks Mole, Sanctum of Eternity, Flamekin
//! Herald, Skyhunter Strike Force, Vexilus Praetor, Hellkite Courser, and token names
//! (Neriv, Crackling Vanguard).

use crate::r_p189_common::{bounce, triggers_of};
use crate::r_s01_common::{asked_since, attack_with, supported, tokens};
use crate::r_s06_common::{activate_containing, give_control, has_kw};
use crate::r_s25_common::{cast_new, change_copy_targets, lands_for_cost, spell_copies, x_of};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn o(id: ObjectId) -> Entity {
    Entity::Object(id)
}

fn pl(p: PlayerId) -> Entity {
    Entity::Player(p)
}

/// A Commander game (20 life, to keep the numbers small) of `n` players.
fn commander_game(n: usize) -> TestGame {
    TestGame::with_config(
        n,
        GameConfig {
            variant: Variant::Commander,
            ..Default::default()
        },
    )
}

/// Makes `id` one of its owner's commanders (CR 903.3).
fn make_commander(t: &mut TestGame, id: ObjectId) {
    t.g.objects[id.0 as usize].is_commander = true;
    let owner = t.g.obj(id).owner;
    let name = t.g.obj(id).chars.name.clone();
    t.g.players[owner.idx()].commander_names.push(name);
    t.g.dirty = true;
    t.g.recompute();
}

fn commander_in_zone(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.command(p, name);
    make_commander(t, id);
    id
}

fn commander_on_battlefield(t: &mut TestGame, p: PlayerId, name: &str) -> ObjectId {
    let id = t.battlefield(p, name);
    make_commander(t, id);
    id
}

/// Records that `p` has cast the commander `name` from the command zone `n` times.
fn set_casts(t: &mut TestGame, p: PlayerId, name: &str, n: u32) {
    t.g.players[p.idx()]
        .commander_casts
        .insert(name.into(), n);
    t.g.dirty = true;
    t.g.recompute();
}

fn casts(t: &TestGame, p: PlayerId, name: &str) -> u32 {
    t.g.player(p)
        .commander_casts
        .get(name)
        .copied()
        .unwrap_or(0)
}

/// Casts the commander card `id` from the command zone (with lands for its cost and
/// `extra` more generic mana).
fn cast_commander(t: &mut TestGame, p: PlayerId, id: ObjectId, extra: usize) -> ObjectId {
    let name = t.g.obj(id).chars.name.to_string();
    lands_for_cost(t, p, &name);
    t.lands(p, "Wastes", extra);
    t.cast(p, id).go()
}

// ---------------------------------------------------------------------------------------
// The Will cycle, Jeska's Will
// ---------------------------------------------------------------------------------------

#[test]
fn will_of_the_jeskai_a_player_may_discard_an_empty_hand() {
    cr!("701.9a", "608.2c");
    ruling!(
        "Will of the Jeskai",
        "A player may discard their hand even if their hand contains zero cards."
    );
    supported("Will of the Jeskai");
    let mut t = TestGame::new(2);
    assert_eq!(t.hand_size(P1), 0);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_yes(P0, false);
    t.answer_yes(P1, true);
    cast_new(&mut t, P0, "Will of the Jeskai", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P1), 5);
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn will_cycle_your_commander_is_not_required_and_choices_are_locked_at_announcement() {
    cr!("700.2", "903.3", "601.2b");
    ruling!(
        "Will of the Abzan",
        "The commander you control doesn’t have to be your commander."
    );
    ruling!(
        "Will of the Abzan",
        "Once you’ve announced that you’re casting this spell, players can’t take any actions until you’ve finished doing so. Notably, opponents can’t try to remove your commander to change how many modes you may choose."
    );
    supported("Will of the Mardu");
    for name in [
        "Will of the Abzan",
        "Will of the Jeskai",
        "Will of the Mardu",
        "Will of the Sultai",
        "Will of the Temur",
    ] {
        supported(name);
    }
    // P1's commander under P0's control: P0 may choose both modes of Will of the Mardu;
    // nobody gets priority while it's being cast.
    let mut t = commander_game(2);
    let cmdr = commander_on_battlefield(&mut t, P1, "Hill Giant");
    give_control(&mut t, cmdr, P0);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0, 1]));
    cast_new(&mut t, P0, "Will of the Mardu", &[pl(P1), o(bears)]);
    let during = asked_since(&t, from);
    let max: Vec<u32> = during
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseModes { max, .. } => Some(*max),
            _ => None,
        })
        .collect();
    assert_eq!(max, vec![2]);
    assert!(!during
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    // Removing the commander now doesn't change the modes chosen.
    bounce(&mut t, cmdr);
    t.resolve_all();
    // Both modes: one Warrior (P1 controls one creature), then 1 damage (P0 controls
    // just the Warrior) to the Bears.
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Warrior").len(), 1);
    assert_eq!(t.obj_now(bears).damage, 1);
}

#[test]
fn jeskas_will_cards_you_dont_play_stay_exiled() {
    cr!("611.2a", "400.7");
    ruling!(
        "Jeska's Will",
        "Any of the exiled cards you don't play will remain exiled."
    );
    supported("Jeska's Will");
    let mut t = TestGame::new(2);
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    cast_new(&mut t, P0, "Jeska's Will", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.g.exile.len(), 3);
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(t.g.exile.len(), 3);
}

// ---------------------------------------------------------------------------------------
// Guardian Augmenter, Vexilus Praetor
// ---------------------------------------------------------------------------------------

#[test]
fn guardian_augmenter_doesnt_affect_copies_of_a_commander() {
    cr!("707.2", "903.3");
    ruling!(
        "Guardian Augmenter",
        "Being a commander is not a copiable characteristic; these effects do not apply to copies of a player's commander."
    );
    supported("Guardian Augmenter");
    let mut t = commander_game(2);
    t.battlefield(P0, "Guardian Augmenter");
    let cmdr = commander_on_battlefield(&mut t, P0, "Hill Giant");
    assert_eq!(t.pt(cmdr), (5, 5));
    assert!(has_kw(&t, cmdr, KeywordKind::Hexproof));
    t.answer_choose(P0, &[o(cmdr)]);
    t.answer_yes(P0, true);
    let clone = t.enter(P0, "Clone");
    t.resolve_all();
    let clone = t.g.current(clone);
    assert_eq!(t.obj_now(clone).chars.name, "Hill Giant");
    assert_eq!(t.pt(clone), (3, 3));
    assert!(!has_kw(&t, clone, KeywordKind::Hexproof));
}

#[test]
fn vexilus_praetor_protection_from_everything_on_commanders() {
    cr!("702.16j", "702.16b", "702.16e");
    ruling!(
        "Vexilus Praetor",
        "Protection from everything means that commanders you control can't be the target of any spells or abilities, they can't be blocked, nothing can be attached to them, and all damage that would be dealt to them is prevented."
    );
    supported("Vexilus Praetor");
    let mut t = commander_game(2);
    t.battlefield(P0, "Vexilus Praetor");
    let cmdr = commander_on_battlefield(&mut t, P0, "Hill Giant");
    // Not targetable (even by its controller).
    t.lands(P0, "Mountain", 1);
    let bolt = t.hand(P0, "Lightning Bolt");
    if let Ok(spell) = t.cast(P0, bolt).target(cmdr).try_go() {
        // The illegal choice was replaced by a legal target.
        assert!(!crate::r_s25_common::targets_of(&t, spell).contains(&o(cmdr)));
        t.resolve_all();
    }
    assert_eq!(t.obj_now(cmdr).damage, 0);
    // Not blockable.
    let bears = t.battlefield(P1, "Grizzly Bears");
    attack_with(&mut t, &[(cmdr, pl(P1))]);
    assert!(!crate::r_s21_common::legal_blocks(&mut t, P1, &[(bears, cmdr)]));
    // Damage is prevented: Pyroclasm.
    t.g.combat = None;
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Pyroclasm", &[]);
    t.resolve_all();
    assert_eq!(t.obj_now(cmdr).damage, 0);
    assert!(!t.on_battlefield(bears));
    // But it can still be destroyed by an effect that doesn't target: Day of Judgment.
    t.answer_yes(P0, true);
    cast_new(&mut t, P1, "Day of Judgment", &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(cmdr));
}

#[test]
fn vexilus_praetor_nothing_can_be_attached() {
    cr!("702.16c", "303.4d");
    ruling!(
        "Vexilus Praetor",
        "They can still be affected in other ways (such as by Their Name is Death, for example)."
    );
    let mut t = commander_game(2);
    t.battlefield(P0, "Vexilus Praetor");
    let cmdr = commander_on_battlefield(&mut t, P0, "Hill Giant");
    let sword = t.battlefield(P0, "Bonesplitter");
    t.lands(P0, "Wastes", 1);
    let _ = t.activate(P0, sword, 0, &[o(cmdr)]);
    t.resolve_all();
    assert_ne!(t.obj_now(sword).attached_to, Some(o(cmdr)));
    // Affected in other ways: Wrath of God destroys it.
    t.answer_yes(P0, false);
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Wrath of God", &[]);
    t.resolve_all();
    assert!(!t.on_battlefield(cmdr));
}

// ---------------------------------------------------------------------------------------
// Commander's Insignia, Jirina Kudro
// ---------------------------------------------------------------------------------------

#[test]
fn commanders_insignia_counts_casts_of_each_commander_including_a_spell_on_the_stack() {
    cr!("903.8", "601.2i");
    ruling!(
        "Commander's Insignia",
        "Certain cards in other sets with the partner keyword may allow you to have more than one commander. If you do, Commander’s Insignia counts the total number of times you’ve cast each of your commanders."
    );
    ruling!(
        "Commander's Insignia",
        "Commander’s Insignia counts each time you’ve cast your commander, even the times it was countered or when your commander spell is still on the stack."
    );
    supported("Commander's Insignia");
    let mut t = commander_game(2);
    t.battlefield(P0, "Commander's Insignia");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let a = commander_in_zone(&mut t, P0, "Hill Giant");
    let b = commander_in_zone(&mut t, P0, "Llanowar Elves");
    set_casts(&mut t, P0, "Llanowar Elves", 2);
    assert_eq!(t.pt(bears), (4, 4));
    // Hill Giant on the stack already counts.
    let spell = cast_commander(&mut t, P0, a, 0);
    assert_eq!(casts(&t, P0, "Hill Giant"), 1);
    assert_eq!(t.pt(bears), (5, 5));
    // Countered: still counts.
    cast_new(&mut t, P1, "Counterspell", &[o(spell)]);
    t.resolve_all();
    assert!(!t.on_battlefield(spell));
    assert_eq!(t.pt(bears), (5, 5));
    let _ = b;
}

#[test]
fn jirina_kudro_counts_its_own_cast_and_pumps_its_tokens() {
    cr!("903.8", "611.3a");
    ruling!(
        "Jirina Kudro",
        "If Jirina is your commander, the number of times you cast it includes the time you cast Jirina just before it entered the battlefield."
    );
    ruling!(
        "Jirina Kudro",
        "Jirina’s last ability gives +2/+0 to all Humans you control other than Jirina, including the Human tokens created by its triggered ability."
    );
    supported("Jirina Kudro");
    let mut t = commander_game(2);
    let jirina = commander_in_zone(&mut t, P0, "Jirina Kudro");
    let spell = cast_commander(&mut t, P0, jirina, 0);
    t.resolve_all();
    let jirina = t.g.current(spell);
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 1);
    assert_eq!(t.pt(toks[0]), (3, 1));
    assert_eq!(t.pt(jirina), (3, 3));
}

// ---------------------------------------------------------------------------------------
// Road of Return, Sanctum of Eternity
// ---------------------------------------------------------------------------------------

#[test]
fn road_of_return_returns_one_chosen_commander_or_does_nothing() {
    cr!("903.8", "608.2c");
    ruling!(
        "Road of Return",
        "Certain cards in other sets with the partner keyword may allow you to have more than one commander. If you have two commanders in the command zone, Road of Return’s second mode puts one of your choice into your hand, not both."
    );
    ruling!(
        "Road of Return",
        "If your commander isn’t in the command zone (or if you’re not playing the Commander variant), Road of Return’s second mode does nothing."
    );
    supported("Road of Return");
    let mut t = commander_game(2);
    commander_in_zone(&mut t, P0, "Hill Giant");
    let b = commander_in_zone(&mut t, P0, "Llanowar Elves");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    t.answer_choose(P0, &[o(b)]);
    // The owner may put it into the command zone instead (CR 903.9b); P0 doesn't.
    t.answer_yes(P0, false);
    let from = t.asked().len();
    cast_new(&mut t, P0, "Road of Return", &[]);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 1, "{:?}\n{}", asked_since(&t, from), t.dump_log());
    assert_eq!(t.g.command.len(), 1);
    // A commander on the battlefield: nothing.
    let mut t = commander_game(2);
    let c = commander_on_battlefield(&mut t, P0, "Hill Giant");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![1]));
    cast_new(&mut t, P0, "Road of Return", &[]);
    t.resolve_all();
    assert!(t.on_battlefield(c));
    assert_eq!(t.hand_size(P0), 0);
}

#[test]
fn sanctum_of_eternity_does_nothing_if_the_commander_left() {
    cr!("608.2b", "903.3");
    ruling!(
        "Sanctum of Eternity",
        "If your commander isn’t on the battlefield (or if you’re not playing the Commander variant), Sanctum of Eternity’s second ability does nothing."
    );
    supported("Sanctum of Eternity");
    let mut t = commander_game(2);
    let sanctum = t.battlefield(P0, "Sanctum of Eternity");
    let c = commander_on_battlefield(&mut t, P0, "Hill Giant");
    t.lands(P0, "Wastes", 2);
    activate_containing(&mut t, P0, sanctum, "Return target commander")
        .expect("sanctum")
        .expect("on the stack");
    // It leaves for the command zone in response.
    t.answer_yes(P0, true);
    t.g.destroy(c, None);
    t.g.flush_events();
    t.settle();
    t.resolve_all();
    assert_eq!(t.hand_size(P0), 0);
    assert!(!t.g.find_in_zone(Zone::Command, "Hill Giant").is_empty());
    // Not a Commander game: no commander to target.
    let mut t = TestGame::new(2);
    let sanctum = t.battlefield(P0, "Sanctum of Eternity");
    t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Wastes", 2);
    assert!(activate_containing(&mut t, P0, sanctum, "Return target commander").is_err());
}

// ---------------------------------------------------------------------------------------
// Crimson Honor Guard
// ---------------------------------------------------------------------------------------

#[test]
fn crimson_honor_guard_triggers_every_end_step_and_checks_on_resolution() {
    cr!("603.2", "608.2c");
    ruling!(
        "Crimson Honor Guard",
        "Crimson Honor Guard's last ability triggers at the beginning of each player's end step whether that player controls a commander or not. Whether that player controls a commander is checked only as the ability resolves."
    );
    supported("Crimson Honor Guard");
    for commander_in_response in [false, true] {
        let mut t = commander_game(2);
        let chg = t.battlefield(P0, "Crimson Honor Guard");
        t.set_step(P1, Step::PostcombatMain);
        let c = t.command(P1, "Hill Giant");
        make_commander(&mut t, c);
        t.advance_to(P1, Step::End);
        t.settle();
        assert_eq!(triggers_of(&t, chg), 1);
        if commander_in_response {
            lands_for_cost(&mut t, P1, "Hill Giant");
            // Put onto the battlefield in response (as an effect would).
            t.g.move_object(c, Zone::Battlefield, mtg_engine::events::MoveCause::Effect, None);
            t.g.flush_events();
            t.g.recompute();
        }
        let life = t.life(P1);
        t.resolve_all();
        assert_eq!(
            life - t.life(P1),
            if commander_in_response { 0 } else { 4 },
            "commander: {commander_in_response}"
        );
    }
}

// ---------------------------------------------------------------------------------------
// Geode Golem
// ---------------------------------------------------------------------------------------

/// P0's Geode Golem attacks P1; `blocker` (if any) blocks it.
fn golem_hits(t: &mut TestGame, blocker: Option<ObjectId>) -> ObjectId {
    supported("Geode Golem");
    let golem = t.battlefield(P0, "Geode Golem");
    let blocks: Vec<(ObjectId, ObjectId)> = blocker.map(|b| (b, golem)).into_iter().collect();
    crate::r_s03_common::to_blockers(t, &[(golem, pl(P1))], &blocks);
    t.advance_to(P0, Step::CombatDamage);
    t.settle();
    golem
}

#[test]
fn geode_golem_triggers_even_if_dealt_lethal_damage_at_the_same_time() {
    cr!("603.10", "510.2");
    ruling!(
        "Geode Golem",
        "Geode Golem's triggered ability triggers even if it's dealt lethal damage at the same time it deals combat damage to a player."
    );
    let mut t = commander_game(2);
    let c = commander_in_zone(&mut t, P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    // Trample: 3 to the Hill Giant, 2 to P1; the Giant deals 3 to the Golem.
    t.answer_yes(P0, true);
    let life = t.life(P1);
    let golem = golem_hits(&mut t, Some(giant));
    assert_eq!(life - t.life(P1), 2);
    assert!(!t.on_battlefield(golem));
    assert_eq!(triggers_of(&t, golem), 1);
    t.resolve_all();
    assert!(t.on_battlefield(c));
}

#[test]
fn geode_golem_can_cast_a_commander_that_returned_before_it_resolves() {
    cr!("608.2c", "903.9b");
    ruling!(
        "Geode Golem",
        "Geode Golem's triggered ability triggers even if your commander isn't in the command zone at the time Geode Golem deals combat damage to a player. If your commander finds its way there before the trigger resolves, you can cast it."
    );
    let mut t = commander_game(2);
    let c = commander_on_battlefield(&mut t, P0, "Grizzly Bears");
    let golem = golem_hits(&mut t, None);
    assert_eq!(triggers_of(&t, golem), 1);
    // The commander dies and goes to the command zone in response.
    t.answer_yes(P0, true);
    t.g.destroy(c, None);
    t.g.flush_events();
    t.settle();
    let in_zone = t.g.find_in_zone(Zone::Command, "Grizzly Bears");
    assert_eq!(in_zone.len(), 1);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
}

#[test]
fn geode_golem_free_cast_still_pays_the_commander_tax_and_only_during_resolution() {
    cr!("118.9a", "118.9d", "903.8", "608.2g");
    ruling!(
        "Geode Golem",
        "If you cast a card “without paying its mana cost,” you can't choose to cast it for any alternative costs, such as bestow costs. You can, however, pay additional costs. If the card has any mandatory additional costs, such as the commander tax, those must be paid to cast the card."
    );
    ruling!(
        "Geode Golem",
        "If you wish to cast your commander without paying its mana cost, you must cast it while the triggered ability of Geode Golem is resolving. You can't cast for free it later in the turn. It may be cast at a time you normally wouldn't be able to cast a creature spell, but other restrictions (such as “Cast this card only from your graveyard”) are enforced."
    );
    // Cast once before: the tax is {2}.
    let mut t = commander_game(2);
    let c = commander_in_zone(&mut t, P0, "Grizzly Bears");
    set_casts(&mut t, P0, "Grizzly Bears", 1);
    let lands = t.lands(P0, "Wastes", 2);
    golem_hits(&mut t, None);
    // In the combat damage step (not a main phase), during the resolution.
    assert_eq!(t.g.turn.step, Step::CombatDamage);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(lands.iter().all(|l| t.obj_now(*l).tapped));
    assert_eq!(casts(&t, P0, "Grizzly Bears"), 2);
    let _ = c;
    // Without the mana for the tax, it can't be cast.
    let mut t = commander_game(2);
    let c = commander_in_zone(&mut t, P0, "Grizzly Bears");
    set_casts(&mut t, P0, "Grizzly Bears", 1);
    golem_hits(&mut t, None);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.zone(c), Zone::Command);
    // Declining during the resolution: it can't be cast for free later.
    let mut t = commander_game(2);
    let c = commander_in_zone(&mut t, P0, "Grizzly Bears");
    golem_hits(&mut t, None);
    t.answer_yes(P0, false);
    t.resolve_all();
    assert_eq!(t.zone(c), Zone::Command);
    t.advance_to(P0, Step::PostcombatMain);
    assert!(t.cast(P0, c).try_go().is_err());
}

// ---------------------------------------------------------------------------------------
// Myth Unbound
// ---------------------------------------------------------------------------------------

#[test]
fn myth_unbound_doesnt_trigger_if_it_leaves_with_the_commander() {
    cr!("603.6a", "603.10");
    ruling!(
        "Myth Unbound",
        "If Myth Unbound leaves the battlefield at the same time as your commander is put into the command zone, its last ability won’t trigger."
    );
    supported("Myth Unbound");
    supported("Planar Cleansing");
    let mut t = commander_game(2);
    let mu = t.battlefield(P0, "Myth Unbound");
    commander_on_battlefield(&mut t, P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.set_step(P1, Step::PrecombatMain);
    cast_new(&mut t, P1, "Planar Cleansing", &[]);
    t.resolve_all();
    assert!(!t.g.find_in_zone(Zone::Command, "Hill Giant").is_empty());
    assert_eq!(triggers_of(&t, mu), 0);
    // Alone, it does trigger.
    let mut t = commander_game(2);
    let mu = t.battlefield(P0, "Myth Unbound");
    let c = commander_on_battlefield(&mut t, P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.g.destroy(c, None);
    t.g.flush_events();
    t.settle();
    assert_eq!(triggers_of(&t, mu), 1);
}

#[test]
fn myth_unbound_reduces_after_the_tax_and_from_any_zone() {
    cr!("601.2f", "903.8");
    ruling!(
        "Myth Unbound",
        "Myth Unbound’s cost reduction applies after you apply the commander tax. In effect, this reduces the commander tax to {1}."
    );
    ruling!(
        "Myth Unbound",
        "Myth Unbound’s cost reduction applies even if you’re casting your commander from a zone other than the command zone."
    );
    // Hill Giant ({3}{R}) cast once before: {3}{R} + {2} − {1} = 5 mana.
    for (lands, ok) in [(4usize, false), (5, true)] {
        let mut t = commander_game(2);
        t.battlefield(P0, "Myth Unbound");
        let c = commander_in_zone(&mut t, P0, "Hill Giant");
        set_casts(&mut t, P0, "Hill Giant", 1);
        t.lands(P0, "Mountain", lands);
        assert_eq!(t.cast(P0, c).try_go().is_ok(), ok, "{lands} lands");
    }
    // From the hand: {3}{R} − {1} = 3 mana.
    let mut t = commander_game(2);
    t.battlefield(P0, "Myth Unbound");
    let c = t.hand(P0, "Hill Giant");
    make_commander(&mut t, c);
    set_casts(&mut t, P0, "Hill Giant", 1);
    t.lands(P0, "Mountain", 3);
    t.cast(P0, c).go();
}

// ---------------------------------------------------------------------------------------
// Thunderclap Drake
// ---------------------------------------------------------------------------------------

/// P0's Thunderclap Drake's ability resolves (P0 has cast commanders `casts` times).
fn drake(t: &mut TestGame, casts: &[(&str, u32)]) {
    supported("Thunderclap Drake");
    let d = t.battlefield(P0, "Thunderclap Drake");
    for (name, n) in casts {
        let c = commander_in_zone(t, P0, name);
        let _ = c;
        set_casts(t, P0, name, *n);
    }
    t.lands(P0, "Island", 1);
    t.lands(P0, "Wastes", 2);
    activate_containing(t, P0, d, "copy").expect("drake");
    t.resolve_all();
}

#[test]
fn thunderclap_drake_copies_keep_x_and_division() {
    cr!("707.10", "707.10c");
    ruling!(
        "Thunderclap Drake",
        "If a spell that's copied has an X whose value was determined as it was cast, the copies will have the same value of X."
    );
    ruling!(
        "Thunderclap Drake",
        "If the spell that's copied divides damage or distributes counters among a number of targets, the division and number of targets can't be changed."
    );
    // Blaze with X = 2.
    let mut t = commander_game(2);
    drake(&mut t, &[("Hill Giant", 1)]);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let giant = t.battlefield(P1, "Wall of Stone");
    lands_for_cost(&mut t, P0, "Blaze");
    t.lands(P0, "Wastes", 2);
    let blaze = t.hand(P0, "Blaze");
    t.cast(P0, blaze).x(2).target(bears).go();
    change_copy_targets(&mut t, P0, &[Some(o(giant))]);
    t.resolve();
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 1);
    assert_eq!(x_of(&t, copies[0]), Some(2));
    t.resolve_all();
    assert_eq!(t.obj_now(giant).damage, 2);
    // Forked Bolt divided 1/1: the copy keeps it.
    let mut t = commander_game(2);
    drake(&mut t, &[("Hill Giant", 1)]);
    let a = t.battlefield(P1, "Wall of Stone");
    t.answer(P0, DecisionKind::Divide, Answer::Numbers(vec![1, 1]));
    t.answer_yes(P0, false);
    let life = t.life(P1);
    lands_for_cost(&mut t, P0, "Forked Bolt");
    let fb = t.hand(P0, "Forked Bolt");
    t.cast(P0, fb).targets(&[o(a), pl(P1)]).go();
    t.resolve_all();
    assert_eq!(t.obj_now(a).damage, 2);
    assert_eq!(life - t.life(P1), 2);
}

#[test]
fn thunderclap_drake_counts_casts_of_each_commander_and_copies_arent_cast() {
    cr!("707.10", "707.12", "903.8");
    ruling!(
        "Thunderclap Drake",
        "If you have multiple commanders, you'll copy the next instant or sorcery spell you cast this turn for each time you've cast either of them. For example, if you've cast one commander once and one twice, you'll copy that spell three times."
    );
    ruling!(
        "Thunderclap Drake",
        "The copies made by Thunderclap Drake's delayed triggered ability are created on the stack, so they're not \"cast.\""
    );
    let mut t = commander_game(2);
    drake(&mut t, &[("Hill Giant", 1), ("Llanowar Elves", 2)]);
    let gad = t.battlefield(P0, "Gadwick, the Wizened");
    t.battlefield(P1, "Grizzly Bears");
    for _ in 0..3 {
        t.answer_yes(P0, false);
    }
    let life = t.life(P1);
    cast_new(&mut t, P0, "Lightning Bolt", &[pl(P1)]);
    t.resolve();
    // Not blue, so Gadwick doesn't care; check casts with the spell history instead.
    let copies = spell_copies(&t);
    assert_eq!(copies.len(), 3);
    t.resolve_all();
    assert_eq!(life - t.life(P1), 12);
    let _ = gad;
    // Copies of a blue spell don't trigger "whenever you cast a blue spell".
    let mut t = commander_game(2);
    drake(&mut t, &[("Hill Giant", 1)]);
    let gad = t.battlefield(P0, "Gadwick, the Wizened");
    t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, false);
    cast_new(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    assert_eq!(triggers_of(&t, gad), 1);
}

// ---------------------------------------------------------------------------------------
// Forge of Heroes
// ---------------------------------------------------------------------------------------

#[test]
fn forge_of_heroes_gives_both_counters_or_none() {
    cr!("122.1", "903.3");
    ruling!(
        "Forge of Heroes",
        "If the target commander is somehow a creature and a planeswalker, most likely because it's Gideon, it receives both kinds of counters."
    );
    ruling!(
        "Forge of Heroes",
        "If the target commander is somehow neither a creature nor a planeswalker (such as Arixmethes, Slumbering Isle may be), it receives no counters."
    );
    supported("Forge of Heroes");
    supported("Gideon Blackblade");
    supported("Arixmethes, Slumbering Isle");
    // Gideon Blackblade on P0's turn: a creature and a planeswalker.
    let mut t = commander_game(2);
    let forge = t.battlefield(P0, "Forge of Heroes");
    let gid = t.enter(P0, "Gideon Blackblade");
    make_commander(&mut t, gid);
    t.resolve_all();
    let loyalty = t.counters(gid, "loyalty");
    assert!(t.obj_now(gid).is_creature());
    activate_containing(&mut t, P0, forge, "Choose target commander")
        .expect("forge")
        .expect("on the stack");
    t.resolve_all();
    assert_eq!(t.counters(gid, "+1/+1"), 1);
    assert_eq!(t.counters(gid, "loyalty"), loyalty + 1);
    // Arixmethes with slumber counters: a land.
    let mut t = commander_game(2);
    let forge = t.battlefield(P0, "Forge of Heroes");
    let arix = t.enter(P0, "Arixmethes, Slumbering Isle");
    make_commander(&mut t, arix);
    t.resolve_all();
    assert!(!t.obj_now(arix).is_creature());
    activate_containing(&mut t, P0, forge, "Choose target commander")
        .expect("forge")
        .expect("on the stack");
    t.resolve_all();
    assert_eq!(t.counters(arix, "+1/+1"), 0);
    assert_eq!(t.counters(arix, "loyalty"), 0);
}

// ---------------------------------------------------------------------------------------
// Chain Assassination
// ---------------------------------------------------------------------------------------

#[test]
fn chain_assassination_draws_if_the_target_survives_but_not_if_it_fizzles() {
    cr!("608.2b", "702.12b");
    ruling!(
        "Chain Assassination",
        "If the target creature is a legal target but isn’t destroyed, most likely because it has indestructible, you’ll still draw a card if another creature died this turn."
    );
    ruling!(
        "Chain Assassination",
        "If the target creature is an illegal target by the time Chain Assassination tries to resolve, it won’t resolve and none of its effects will happen. You won’t draw a card even if another creature has died this turn."
    );
    supported("Chain Assassination");
    for fizzle in [false, true] {
        let mut t = TestGame::new(2);
        // Another creature died this turn.
        let elves = t.battlefield(P1, "Llanowar Elves");
        t.g.destroy(elves, None);
        t.g.flush_events();
        t.settle();
        let target = t.battlefield(P1, "Darksteel Myr");
        cast_new(&mut t, P0, "Chain Assassination", &[o(target)]);
        if fizzle {
            bounce(&mut t, target);
        }
        let hand = t.hand_size(P0);
        t.resolve_all();
        assert_eq!(t.hand_size(P0), hand + usize::from(!fizzle), "fizzle: {fizzle}");
    }
}

// ---------------------------------------------------------------------------------------
// Tome of Legends
// ---------------------------------------------------------------------------------------

#[test]
fn tome_of_legends_counts_your_commander_whoever_controls_it() {
    cr!("903.3", "508.1");
    ruling!(
        "Tome of Legends",
        "If you control another player's commander, you won't put a page counter on Tome of Legends when you attack with that creature. Similarly, if another player controls your commander, you'll still put a page counter on Tome of Legends if it attacks."
    );
    supported("Tome of Legends");
    // P0 attacks with P1's commander: no counter on P0's Tome.
    let mut t = commander_game(2);
    let tome = t.battlefield(P0, "Tome of Legends");
    let theirs = commander_on_battlefield(&mut t, P1, "Hill Giant");
    give_control(&mut t, theirs, P0);
    let theirs = t.g.current(theirs);
    t.g.objects[theirs.0 as usize].summoning_sick = false;
    let before = t.counters(tome, "page");
    attack_with(&mut t, &[(theirs, pl(P1))]);
    assert!(t.g.is_attacking(theirs));
    t.resolve_all();
    assert_eq!(t.counters(tome, "page"), before);
    // P1 attacks with P0's commander: P0's Tome gets one.
    let mut t = commander_game(2);
    let tome = t.battlefield(P0, "Tome of Legends");
    let mine = commander_on_battlefield(&mut t, P0, "Hill Giant");
    give_control(&mut t, mine, P1);
    let mine = t.g.current(mine);
    t.g.objects[mine.0 as usize].summoning_sick = false;
    let before = t.counters(tome, "page");
    t.set_step(P1, Step::BeginningOfCombat);
    attack_with(&mut t, &[(mine, pl(P0))]);
    assert!(t.g.is_attacking(mine));
    t.resolve_all();
    assert_eq!(t.counters(tome, "page"), before + 1);
}

// ---------------------------------------------------------------------------------------
// Drillworks Mole
// ---------------------------------------------------------------------------------------

#[test]
fn drillworks_mole_needs_a_legal_target_but_not_itself() {
    cr!("608.2b", "115.1");
    ruling!(
        "Drillworks Mole",
        "If you target a commander creature with Drillworks Mole's activated ability and that creature is no longer a legal target as the ability resolves, you will not put a +1/+1 counter on Drillworks Mole. Conversely, if the target is still legal, you will put a counter on the commander creature you control, even if Drillworks Mole is no longer on the battlefield as the ability resolves."
    );
    supported("Drillworks Mole");
    // The target leaves: no counter on the Mole.
    let mut t = commander_game(2);
    let mole = t.battlefield(P0, "Drillworks Mole");
    let c = commander_on_battlefield(&mut t, P0, "Hill Giant");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, mole, 0, &[o(c)]).expect("mole");
    bounce(&mut t, c);
    t.resolve_all();
    assert_eq!(t.counters(mole, "+1/+1"), 0);
    // The Mole leaves: the commander still gets one.
    let mut t = commander_game(2);
    let mole = t.battlefield(P0, "Drillworks Mole");
    let c = commander_on_battlefield(&mut t, P0, "Hill Giant");
    t.lands(P0, "Wastes", 2);
    t.activate(P0, mole, 0, &[o(c)]).expect("mole");
    bounce(&mut t, mole);
    t.resolve_all();
    assert_eq!(t.counters(c, "+1/+1"), 1);
}

// ---------------------------------------------------------------------------------------
// Flamekin Herald
// ---------------------------------------------------------------------------------------

#[test]
fn flamekin_herald_cascade_resolves_even_if_herald_leaves() {
    cr!("702.85a", "113.7a");
    ruling!(
        "Flamekin Herald",
        "Once you announce that you're casting a commander spell, players can't take action until you've finished casting it. Causing Flamekin Herald to leave the battlefield after the spell has been cast won't stop its cascade ability from resolving."
    );
    supported("Flamekin Herald");
    let mut t = commander_game(2);
    let herald = t.battlefield(P0, "Flamekin Herald");
    let c = commander_in_zone(&mut t, P0, "Hill Giant");
    t.library_top(P0, "Grizzly Bears");
    let from = t.asked().len();
    cast_commander(&mut t, P0, c, 0);
    assert!(!asked_since(&t, from)
        .iter()
        .any(|(_, d)| matches!(d, Decision::Priority { .. })));
    t.settle();
    assert_eq!(t.stack_len(), 2);
    bounce(&mut t, herald);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
}

#[test]
fn flamekin_herald_lost_during_casting_means_no_cascade() {
    cr!("702.85a", "601.2h", "601.2i");
    ruling!(
        "Flamekin Herald",
        "In some unusual situations, you may lose control of Flamekin Herald during the process of casting a commander spell. In those situations, the commander spell won't have cascade."
    );
    supported("Elder Deep-Fiend");
    // Elder Deep-Fiend's emerge sacrifices the Herald while it's being cast.
    let mut t = commander_game(2);
    let herald = t.battlefield(P0, "Flamekin Herald");
    let c = commander_in_zone(&mut t, P0, "Elder Deep-Fiend");
    t.library_top(P0, "Grizzly Bears");
    // Emerge {5}{U}{U} minus the Herald's mana value (3): {2}{U}{U}.
    t.lands(P0, "Island", 4);
    t.answer_choose(P0, &[o(herald)]);
    let from = t.asked().len();
    let spell = t
        .cast(P0, c)
        .method(mtg_engine::object::CastMethod::Keyword(KeywordKind::Emerge))
        .try_go();
    assert!(spell.is_ok(), "{spell:?} {:?}", asked_since(&t, from));
    assert!(!t.on_battlefield(herald));
    t.settle();
    t.resolve_all();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
}

// ---------------------------------------------------------------------------------------
// Skyhunter Strike Force, Hellkite Courser, Neriv
// ---------------------------------------------------------------------------------------

#[test]
fn skyhunter_strike_force_melee_counts_each_opponent_attacked_once() {
    cr!("702.121a", "508.6");
    ruling!(
        "Skyhunter Strike Force",
        "It doesn't matter how many creatures you attacked a player with, only that you attacked a player with at least one creature."
    );
    supported("Skyhunter Strike Force");
    let mut t = TestGame::new(3);
    let ssf = t.battlefield(P0, "Skyhunter Strike Force");
    let mut attackers = vec![(ssf, pl(P1))];
    for _ in 0..5 {
        let b = t.battlefield(P0, "Grizzly Bears");
        attackers.push((b, pl(P2)));
    }
    attack_with(&mut t, &attackers);
    t.resolve_all();
    assert_eq!(t.pt(ssf), (4, 4));
}

#[test]
fn hellkite_courser_isnt_casting_the_commander() {
    cr!("903.8", "601.1");
    ruling!(
        "Hellkite Courser",
        "Putting your commander onto the battlefield this way isn't the same as casting that commander. The \"commander tax\" won't increase."
    );
    supported("Hellkite Courser");
    let mut t = commander_game(2);
    let c = commander_in_zone(&mut t, P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[o(c)]);
    t.enter(P0, "Hellkite Courser");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    assert_eq!(casts(&t, P0, "Hill Giant"), 0);
    // It returns to the command zone at the next end step; casting it costs only {3}{R}.
    t.advance_to(P0, Step::End);
    t.resolve_all();
    let back = t.g.find_in_zone(Zone::Command, "Hill Giant");
    assert_eq!(back.len(), 1);
    t.advance_to(P1, Step::PrecombatMain);
    t.set_step(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 4);
    t.cast(P0, back[0]).go();
}

#[test]
fn neriv_goblin_tokens_are_named_goblin_token() {
    cr!("111.4");
    ruling!(
        "Neriv, Crackling Vanguard",
        "All other tokens have a name that matches the subtypes of the token that was created plus the word “Token.”"
    );
    let mut t = TestGame::new(2);
    t.enter(P0, "Neriv, Crackling Vanguard");
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 2);
    for tok in toks {
        assert_eq!(t.obj_now(tok).chars.name, "Goblin Token");
    }
}
