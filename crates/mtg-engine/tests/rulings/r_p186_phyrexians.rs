//! Rulings batch P186 — Phyrexian, Human and Noble cards: targets of triggered abilities
//! (CR 603.3d, 115), exile-and-return with Auras (CR 303.4g, 303.4f), characteristics of
//! cards in exile (CR 604.3, 611.3), prohibitions on activated abilities (CR 602.5),
//! blocking requirements (CR 509.1c), copy effects (CR 707.2, 707.9b, 706), linked
//! trigger events and look-back (CR 603.10a), infect (CR 702.90), a game draw (CR
//! 104.4a), changing targets (CR 115.7), mana-ability triggers (CR 605.1b), "enters
//! tapped" for simultaneous entry (CR 614.12), token ownership (CR 111.2), postcombat main
//! phases (CR 505.1a), damage triggers per source (CR 603.2c), untap restrictions (CR
//! 502.3), and additional costs of spells cast without paying their mana cost (CR
//! 601.2b, 118.9).

use crate::r_p185_common::*;
use crate::r_p186_common::*;
use crate::r_p189_common::stack_triggers_from;
use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, destroy, target_candidates};
use crate::r_s04_common::add_mana;
use crate::r_s06_common::{attach_new, attached_to, damage};
use crate::r_s24_common::{enter_together, pool, tapped};
use crate::r_s25_common::cast_new;
use mtg_engine::ability::Modification;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::GameResult;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::{Stage, Step};
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn corrupt_eunuchs_alone_targets_itself() {
    cr!("603.3d", "115.1");
    ruling!(
        "Corrupt Eunuchs",
        "If there are no other creatures on the battlefield when Corrupt Eunuchs enters, it will have to target itself."
    );
    supported("Corrupt Eunuchs");
    let mut t = TestGame::new(2);
    let e = crate::r_s05_common::enter(&mut t, P0, "Corrupt Eunuchs");
    let trig = stack_triggers_from(&t, e);
    assert_eq!(trig.len(), 1);
    assert_eq!(
        crate::r_s25_common::targets_of(&t, trig[0]),
        vec![Entity::Object(e)]
    );
    t.resolve_all();
    assert!(!t.on_battlefield(e));
}

/// P0's Brago attacks P1 unblocked (with `others` attacking too, blocked as in `blocks`),
/// answering its trigger's targets with `targets`, and combat finishes.
fn brago_connects(
    t: &mut TestGame,
    brago: ObjectId,
    others: &[ObjectId],
    blocks: &[(ObjectId, ObjectId)],
    targets: &[Entity],
) {
    t.answer_targets(P0, targets);
    let mut attackers = vec![(brago, Entity::Player(P1))];
    attackers.extend(others.iter().map(|o| (*o, Entity::Player(P1))));
    attack_with(t, &attackers);
    block_and_finish(t, P1, blocks);
    t.resolve_all();
}

#[test]
fn brago_targets_only_what_survived_combat_damage() {
    cr!("510.2", "603.3d", "704.3");
    ruling!(
        "Brago, King Eternal",
        "Brago's last ability exiles and returns all the targets during the combat damage step, after combat damage is dealt. You can't target any creature that didn't survive combat."
    );
    supported("Brago, King Eternal");
    let mut t = TestGame::new(2);
    let brago = t.battlefield(P0, "Brago, King Eternal");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let survivor = t.battlefield(P0, "Hill Giant");
    let giant = t.battlefield(P1, "Hill Giant");
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseTargets { .. }),
        |g| g.turn.step,
    );
    let from = t.asked().len();
    brago_connects(&mut t, brago, &[bears, survivor], &[(giant, bears)], &[]);
    assert_eq!(t.life(P1), 18 - 3);
    assert_eq!(*seen.lock().unwrap(), vec![Step::CombatDamage]);
    let cands = target_candidates(&t, P0, from);
    assert_eq!(cands.len(), 1);
    assert!(!cands[0].contains(&Entity::Object(bears)));
    assert!(cands[0].iter().any(|e| *e == Entity::Object(survivor)));
}

#[test]
fn brago_auras_return_attached_without_targeting_or_stay_exiled() {
    cr!("303.4g", "303.4f", "610.3c", "400.7");
    ruling!(
        "Brago, King Eternal",
        "If you exile an Aura with Brago's last ability, the Aura's owner chooses what it will enchant as it comes back onto the battlefield."
    );
    ruling!(
        "Brago, King Eternal",
        "You may exile and return Brago using its own ability."
    );
    supported("Pacifism");
    supported("Gladecover Scout");
    // Pacifism on P1's hexproof Scout and Brago return: Brago enters with the Aura, so
    // the Scout is the only creature it can enchant.
    let mut t = TestGame::new(2);
    let brago = t.battlefield(P0, "Brago, King Eternal");
    let scout = t.battlefield(P1, "Gladecover Scout");
    let pac = attach_new(&mut t, P0, "Pacifism", scout);
    brago_connects(
        &mut t,
        brago,
        &[],
        &[],
        &[Entity::Object(pac), Entity::Object(brago)],
    );
    assert_ne!(t.g.current(brago), brago, "Brago is a new object");
    assert!(t.on_battlefield(brago) && t.on_battlefield(pac));
    assert_ne!(t.g.current(pac), pac);
    assert_eq!(attached_to(&t, pac), Some(Entity::Object(scout)));
    // No creature but those returning with it: the Aura stays exiled.
    let mut t = TestGame::new(2);
    let brago = t.battlefield(P0, "Brago, King Eternal");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let pac = attach_new(&mut t, P0, "Pacifism", bears);
    brago_connects(
        &mut t,
        brago,
        &[],
        &[],
        &[
            Entity::Object(pac),
            Entity::Object(brago),
            Entity::Object(bears),
        ],
    );
    assert!(t.on_battlefield(brago) && t.on_battlefield(bears));
    assert_eq!(t.zone(pac), Zone::Exile);
}

#[test]
fn phyrexian_ingester_uses_the_exiled_cards_defining_abilities_only() {
    cr!("604.3", "611.3a", "613.4a");
    ruling!(
        "Phyrexian Ingester",
        "Abilities that define a creature's power and toughness apply while that card is in exile, but abilities that add or subtract from it don't."
    );
    ruling!(
        "Phyrexian Ingester",
        "If the card in exile isn't a creature card (perhaps because it was a land that was temporarily a creature while on the battlefield), Phyrexian Ingester doesn't get a bonus."
    );
    supported("Phyrexian Ingester");
    supported("Battle Squadron");
    supported("Werebear");
    // Battle Squadron counts the creatures its owner controls, constantly.
    let mut t = TestGame::new(2);
    let sq = t.battlefield(P1, "Battle Squadron");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.answer_yes(P0, true);
    let ing = enter_with(&mut t, P0, "Phyrexian Ingester", &[Entity::Object(sq)]);
    t.resolve_all();
    assert_eq!(t.zone(sq), Zone::Exile);
    assert_eq!(t.pt(ing), (5, 5));
    t.battlefield(P1, "Grizzly Bears");
    assert_eq!(t.pt(ing), (6, 6));
    // Werebear's threshold bonus doesn't apply in exile.
    let mut t = TestGame::new(2);
    crate::r_p189_common::fill_graveyard(&mut t, P1, "Grizzly Bears", 7);
    let wb = t.battlefield(P1, "Werebear");
    assert_eq!(t.pt(wb), (4, 4));
    t.answer_yes(P0, true);
    let ing = enter_with(&mut t, P0, "Phyrexian Ingester", &[Entity::Object(wb)]);
    t.resolve_all();
    assert_eq!(t.pt(ing), (4, 4));
    // An animated land: a land card in exile, no bonus.
    let mut t = TestGame::new(2);
    crate::r_p189_common::fill_graveyard(&mut t, P0, "Grizzly Bears", 7);
    let m = t.battlefield(P0, "Nantuko Monastery");
    add_mana(&mut t, P0, ManaType::G, 1);
    add_mana(&mut t, P0, ManaType::W, 1);
    crate::r_s06_common::activate_containing(&mut t, P0, m, "{G}{W}").unwrap();
    t.resolve_all();
    t.answer_yes(P0, true);
    let ing = enter_with(&mut t, P0, "Phyrexian Ingester", &[Entity::Object(m)]);
    t.resolve_all();
    assert_eq!(t.zone(m), Zone::Exile);
    assert_eq!(t.pt(ing), (3, 3));
}

#[test]
fn phyrexian_revoker_stops_equip_but_not_static_abilities() {
    cr!("602.5", "702.6a");
    ruling!(
        "Phyrexian Revoker",
        "Some keywords are activated abilities (such as equip) and will have colons in their reminder texts. Static and triggered abilities of sources with the chosen name are unaffected."
    );
    supported("Phyrexian Revoker");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let staff = attach_new(&mut t, P0, "Wizard's Staff", bears);
    let other = t.battlefield(P0, "Hill Giant");
    t.lands(P0, "Wastes", 3);
    assert!(can_activate(&mut t, P0, staff));
    t.answer(P1, DecisionKind::Name, Answer::Text("Wizard's Staff".into()));
    crate::r_s05_common::enter(&mut t, P1, "Phyrexian Revoker");
    assert!(!can_activate(&mut t, P0, staff), "equip can't be activated");
    // The prowess it grants still works (static, then triggered).
    cast_new(&mut t, P0, "Opt", &[]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 4), "prowess twice from the Staff");
    assert_eq!(attached_to(&t, staff), Some(Entity::Object(bears)));
    let _ = other;
}

#[test]
fn tangle_angler_doesnt_force_it_to_attack() {
    cr!("509.1c", "508.1");
    ruling!(
        "Tangle Angler",
        "Activating Tangle Angler’s second ability doesn’t force you to attack with Tangle Angler that turn. If it doesn’t attack, the targeted creature is free to block whichever creature its controller chooses, or block no creatures at all."
    );
    supported("Tangle Angler");
    let setup = || {
        let mut t = TestGame::new(2);
        let angler = t.battlefield(P0, "Tangle Angler");
        let giant = t.battlefield(P0, "Hill Giant");
        let bears = t.battlefield(P1, "Grizzly Bears");
        add_mana(&mut t, P0, ManaType::G, 1);
        t.activate(P0, angler, 0, &[Entity::Object(bears)]).unwrap();
        t.resolve_all();
        crate::r_s09_common::to_combat(&mut t, P0);
        (t, angler, giant, bears)
    };
    let (mut t, angler, giant, bears) = setup();
    assert!(crate::r_s09_common::legal_attack(&mut t, &[(giant, Entity::Player(P1))]));
    attack_with(&mut t, &[(giant, Entity::Player(P1))]);
    assert!(crate::r_s21_common::legal_blocks(&mut t, P1, &[]));
    assert!(crate::r_s21_common::legal_blocks(&mut t, P1, &[(bears, giant)]));
    // When it attacks, the Bears must block it.
    let (mut t, angler2, giant2, bears2) = setup();
    attack_with(
        &mut t,
        &[(angler2, Entity::Player(P1)), (giant2, Entity::Player(P1))],
    );
    assert!(!crate::r_s21_common::legal_blocks(&mut t, P1, &[]));
    assert!(!crate::r_s21_common::legal_blocks(&mut t, P1, &[(bears2, giant2)]));
    assert!(crate::r_s21_common::legal_blocks(&mut t, P1, &[(bears2, angler2)]));
    let _ = angler;
}

#[test]
fn brudiclad_copies_only_the_chosen_tokens_copiable_values() {
    cr!("707.2", "707.9a");
    ruling!(
        "Brudiclad, Telchor Engineer",
        "All other tokens you control become a copy of the chosen token, even those that aren't of the same type."
    );
    ruling!(
        "Brudiclad, Telchor Engineer",
        "Brudiclad copies only the values defined as the original token was created (unless that token is copying something else; see below). It doesn't copy whether that token is tapped or untapped, whether it has any counters on it or any Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on."
    );
    supported("Brudiclad, Telchor Engineer");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Brudiclad, Telchor Engineer");
    let treasure = crate::r_s02_common::create_token(&mut t, P0, "Treasure");
    let soldier = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
    crate::r_s26_common::dress_up(&mut t, soldier);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(soldier)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    // The Treasure is now a 1/1 Soldier creature token, untapped, no counters.
    let o = t.obj_now(treasure);
    assert!(o.is(CardType::Creature) && o.chars.has_subtype("Soldier"));
    assert!(!o.chars.has_subtype("Treasure"));
    assert_eq!(t.pt(treasure), (1, 1));
    // So is the new Myr.
    let myr: Vec<_> = tokens(&t, P0)
        .into_iter()
        .filter(|id| ![treasure, soldier].contains(id))
        .collect();
    assert_eq!(myr.len(), 1);
    assert!(t.obj_now(myr[0]).chars.has_subtype("Soldier"));
    assert!(crate::r_s26_common::fresh(&t, myr[0]));
    assert_eq!(t.pt(myr[0]), (1, 1));
    assert_eq!(t.pt(soldier), (6, 6), "the chosen token keeps its own changes");
}

#[test]
fn body_snatcher_targeting_itself_returns_nothing() {
    cr!("608.2b", "400.7");
    ruling!(
        "Body Snatcher",
        "Although you can choose Body Snatcher itself as the target of its last ability, it will be exiled before it can be returned to the battlefield."
    );
    supported("Body Snatcher");
    let mut t = TestGame::new(2);
    let bs = t.battlefield(P0, "Body Snatcher");
    destroy(&mut t, bs);
    t.resolve_all();
    assert_eq!(t.zone(bs), Zone::Exile);
    assert!(t.named_on_battlefield("Body Snatcher").is_empty());
    // Positive control: another creature card returns.
    let mut t = TestGame::new(2);
    let bs = t.battlefield(P0, "Body Snatcher");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    destroy(&mut t, bs);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.zone(bs), Zone::Exile);
}

/// `p` puts Phyrexian Metamorph onto the battlefield as a copy of `what`, then settles.
fn metamorph_copying(t: &mut TestGame, p: PlayerId, what: ObjectId) -> ObjectId {
    supported("Phyrexian Metamorph");
    t.answer_yes(p, true);
    crate::r_p023_common::enter_copying(t, p, "Phyrexian Metamorph", what)
}

#[test]
fn phyrexian_metamorph_gets_enters_abilities_and_replacements() {
    cr!("707.2", "707.9b", "614.1c", "603.6a");
    ruling!(
        "Phyrexian Metamorph",
        "Any \"enters\" abilities of the copied permanent will trigger when Phyrexian Metamorph enters. Any \"as [this] enters\" or \"[this] enters with\" abilities of the chosen permanent will also work."
    );
    let mut t = TestGame::new(2);
    let splicer = t.battlefield(P0, "Master Splicer");
    let m = metamorph_copying(&mut t, P0, splicer);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Golem").len(), 1);
    assert!(t.obj_now(m).is(CardType::Artifact));
    let mut t = TestGame::new(2);
    let tri = t.battlefield(P1, "Triskelion");
    let m = metamorph_copying(&mut t, P0, tri);
    assert_eq!(t.counters(m, counters::PLUS1), 3);
}

#[test]
fn phyrexian_metamorph_copies_printed_values_plus_artifact() {
    cr!("707.2", "707.9b", "707.3");
    ruling!(
        "Phyrexian Metamorph",
        "Except for also being an artifact, Phyrexian Metamorph copies exactly what was printed on the original permanent and nothing more (unless that creature is itself copying something or is a token; see below)."
    );
    ruling!(
        "Phyrexian Metamorph",
        "If Phyrexian Metamorph copies a noncreature artifact, it is no longer a creature."
    );
    ruling!(
        "Phyrexian Metamorph",
        "If the chosen creature is copying something else (for example, if the chosen creature is a Clone), then your Phyrexian Metamorph enters as whatever the chosen creature copied, except it's also an artifact."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s26_common::dress_up(&mut t, bears);
    let m = metamorph_copying(&mut t, P0, bears);
    assert!(crate::r_s26_common::fresh(&t, m));
    assert_eq!(t.pt(m), (2, 2));
    let o = t.obj_now(m);
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert_eq!(o.chars.name.as_str(), "Grizzly Bears");
    assert_eq!(o.chars.colors, crate::r_s17_common::color_set(&[Color::Green]));
    // A noncreature artifact.
    let statue = t.battlefield(P1, "Jade Statue");
    let m = metamorph_copying(&mut t, P0, statue);
    assert!(t.on_battlefield(m));
    assert!(!t.obj_now(m).is(CardType::Creature));
    assert!(t.obj_now(m).is(CardType::Artifact));
    // A Clone copying Hill Giant.
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P1, true);
    let clone = crate::r_p023_common::enter_copying(&mut t, P1, "Clone", giant);
    assert_eq!(t.obj_now(clone).chars.name.as_str(), "Hill Giant");
    let m = metamorph_copying(&mut t, P0, clone);
    let o = t.obj_now(m);
    assert_eq!(o.chars.name.as_str(), "Hill Giant");
    assert!(o.is(CardType::Artifact) && o.is(CardType::Creature));
    assert_eq!(t.pt(m), (3, 3));
}

#[test]
fn phyrexian_metamorph_cant_copy_what_enters_with_it() {
    cr!("707.9b", "614.12");
    ruling!(
        "Phyrexian Metamorph",
        "If Phyrexian Metamorph somehow enters at the same time as another permanent (due to Mass Polymorph or Liliana Vess's third ability, for example), Phyrexian Metamorph can't become a copy of that permanent."
    );
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_yes(P0, true);
    let from = t.asked().len();
    let ids = enter_together(
        &mut t,
        &[(P0, "Phyrexian Metamorph"), (P0, "Grizzly Bears")],
    );
    let offered: Vec<Vec<Entity>> = crate::r_p143_common::entity_choices(&t, P0, from)
        .into_iter()
        .map(|(_, _, c)| c)
        .collect();
    assert!(!offered.is_empty());
    for c in &offered {
        assert!(c.contains(&Entity::Object(giant)));
        assert!(!c.iter().any(|e| matches!(e, Entity::Object(o) if t.g.current(*o) == t.g.current(ids[1]))));
    }
}

#[test]
fn carnifex_demon_counters_every_other_creature() {
    cr!("122.1", "608.2c");
    ruling!(
        "Carnifex Demon",
        "As Carnifex Demon's last ability resolves, you'll put a -1/-1 counter on each creature on the battlefield — including creatures you control — except for that Carnifex Demon."
    );
    supported("Carnifex Demon");
    let mut t = TestGame::new(2);
    let d = t.battlefield(P0, "Carnifex Demon");
    crate::r_s29_common::put_counters(&mut t, d, counters::MINUS1, 2);
    let mine = t.battlefield(P0, "Hill Giant");
    let theirs = t.battlefield(P1, "Hill Giant");
    add_mana(&mut t, P0, ManaType::B, 1);
    t.activate(P0, d, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(d, counters::MINUS1), 1);
    assert_eq!(t.counters(mine, counters::MINUS1), 1);
    assert_eq!(t.counters(theirs, counters::MINUS1), 1);
}

#[test]
fn master_splicer_golem_damage_becomes_lethal_when_it_leaves() {
    cr!("120.6", "704.5g", "611.3a");
    ruling!(
        "Master Splicer",
        "Because damage remains marked on a creature until the damage is removed as the turn ends, nonlethal damage dealt to a Golem you control may become lethal if Master Splicer leaves the battlefield during that turn."
    );
    supported("Master Splicer");
    let mut t = TestGame::new(2);
    let splicer = crate::r_s05_common::enter(&mut t, P0, "Master Splicer");
    t.resolve_all();
    let golem = with_subtype(&t, P0, "Golem");
    assert_eq!(golem.len(), 1);
    assert_eq!(t.pt(golem[0]), (4, 4));
    lethal_when_it_leaves(&mut t, golem[0], splicer, "Master Splicer");
}

#[test]
fn cathedral_membrane_triggers_however_it_dies_in_combat() {
    cr!("603.10a", "506.1");
    ruling!(
        "Cathedral Membrane",
        "Cathedral Membrane's ability will trigger no matter why it's put into a graveyard during the combat phase, not just because of combat damage."
    );
    supported("Cathedral Membrane");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Cathedral Membrane");
    let attacker = t.battlefield(P1, "Craw Wurm");
    crate::r_s09_common::to_combat(&mut t, P1);
    attack_with(&mut t, &[(attacker, Entity::Player(P0))]);
    t.answer(P0, DecisionKind::Blockers, Answer::Blockers(vec![(wall, attacker)]));
    t.advance_to(P1, Step::DeclareBlockers);
    destroy(&mut t, wall);
    t.resolve_all();
    assert!(!t.on_battlefield(attacker), "6 damage to the Wurm it blocked");
}

#[test]
fn marauding_knight_counts_every_opponents_plains() {
    cr!("102.2", "611.3a");
    ruling!("Marauding Knight", "Counts plains controlled by all opponents.");
    supported("Marauding Knight");
    let mut t = TestGame::new(3);
    let k = t.battlefield(P0, "Marauding Knight");
    t.lands(P0, "Plains", 1);
    t.lands(P1, "Plains", 1);
    t.lands(P2, "Plains", 2);
    assert_eq!(t.pt(k), (5, 5));
}

#[test]
fn flesh_reaver_triggers_on_noncombat_damage() {
    cr!("603.2", "701.14a");
    ruling!(
        "Flesh Reaver",
        "Flesh Reaver’s ability triggers whenever it deals any damage to a creature or opponent, not only combat damage."
    );
    supported("Flesh Reaver");
    supported("Prey Upon");
    let mut t = TestGame::new(2);
    let fr = t.battlefield(P0, "Flesh Reaver");
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(
        &mut t,
        P0,
        "Prey Upon",
        &[Entity::Object(fr), Entity::Object(bears)],
    );
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert_eq!(t.life(P0), 16);
}

#[test]
fn fume_spitter_can_target_itself() {
    cr!("602.2b", "608.2b");
    ruling!(
        "Fume Spitter",
        "Fume Spitter can be the target of its own ability. The ability won’t resolve since it won’t have a legal target, and no counter will be put on any creature"
    );
    supported("Fume Spitter");
    let mut t = TestGame::new(2);
    let fs = t.battlefield(P0, "Fume Spitter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, fs, 0, &[Entity::Object(fs)]).unwrap();
    assert!(!t.on_battlefield(fs));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::MINUS1), 0);
    // Positive control.
    let mut t = TestGame::new(2);
    let fs = t.battlefield(P0, "Fume Spitter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.activate(P0, fs, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::MINUS1), 1);
}

#[test]
fn ascendant_evincar_nonblack_shrinks_itself() {
    cr!("611.3a", "613.4c");
    ruling!(
        "Ascendant Evincar",
        "If Ascendant Evincar stops being black, it gives itself -1/-1."
    );
    supported("Ascendant Evincar");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Ascendant Evincar");
    assert_eq!(t.pt(e), (3, 3));
    crate::r_s26_common::modify_until_eot(
        &mut t,
        e,
        vec![Modification::SetColors(crate::r_s17_common::color_set(&[
            Color::Red,
        ]))],
    );
    assert_eq!(t.pt(e), (2, 2));
}

#[test]
fn fallen_ferromancer_infect_damage() {
    cr!("702.90b", "702.90c");
    ruling!(
        "Fallen Ferromancer",
        "If Fallen Ferromancer's ability deals 1 damage to a creature, that damage will cause a -1/-1 counter to be placed on the creature. If it deals 1 damage to a player, that player gets a poison counter."
    );
    supported("Fallen Ferromancer");
    let mut t = TestGame::new(2);
    let f = t.battlefield(P0, "Fallen Ferromancer");
    let bears = t.battlefield(P1, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, f, 0, &[Entity::Object(bears)]).unwrap();
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::MINUS1), 1);
    assert_eq!(marked(&t, bears), 0);
    let f = t.g.current(f);
    t.g.objects[f.0 as usize].tapped = false;
    add_mana(&mut t, P0, ManaType::R, 1);
    add_mana(&mut t, P0, ManaType::C, 1);
    t.activate(P0, f, 0, &[Entity::Player(P1)]).unwrap();
    t.resolve_all();
    assert_eq!(crate::r_s10_common::poison(&t, P1), 1);
    assert_eq!(t.life(P1), 20);
}

#[test]
fn ichor_rats_can_make_the_game_a_draw() {
    cr!("104.4a", "704.5c");
    ruling!(
        "Ichor Rats",
        "If Ichor Rats's last ability causes each player to have ten poison counters, the game is a draw."
    );
    supported("Ichor Rats");
    let mut t = TestGame::new(2);
    for p in [P0, P1] {
        t.g.add_counters(Entity::Player(p), counters::POISON, 9, None);
    }
    crate::r_s05_common::enter(&mut t, P0, "Ichor Rats");
    t.resolve_all();
    assert!(t.has_lost(P0) && t.has_lost(P1));
    assert!(matches!(t.g.result, Some(GameResult::Draw)));
}

#[test]
fn rot_wolf_triggers_when_dying_with_the_creature_it_damaged() {
    cr!("603.10a", "702.90b");
    ruling!(
        "Rot Wolf",
        "If Rot Wolf and a creature dealt damage by Rot Wolf go to the graveyard at the same time, Rot Wolf’s ability will trigger."
    );
    supported("Rot Wolf");
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P0, "Rot Wolf");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(wolf, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(bears, wolf)]);
    t.resolve_all();
    assert!(!t.on_battlefield(wolf) && !t.on_battlefield(bears));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn spellskite_gone_before_resolution_changes_nothing() {
    cr!("608.2b", "115.7");
    ruling!(
        "Spellskite",
        "If Spellskite leaves the battlefield before its ability resolves or otherwise becomes an illegal target for the target spell or ability before its ability resolves, no targets are changed."
    );
    supported("Spellskite");
    let mut t = TestGame::new(2);
    let sk = t.battlefield(P0, "Spellskite");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Object(bears)]);
    add_mana(&mut t, P0, ManaType::U, 1);
    t.activate(P0, sk, 0, &[Entity::Object(bolt)]).unwrap();
    destroy(&mut t, sk);
    t.resolve_all();
    assert!(!t.on_battlefield(bears), "the Bolt still hits the Bears");
    // Positive control: with Spellskite still there, the Bolt is redirected.
    let mut t = TestGame::new(2);
    let sk = t.battlefield(P0, "Spellskite");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let bolt = cast_new(&mut t, P1, "Lightning Bolt", &[Entity::Object(bears)]);
    add_mana(&mut t, P0, ManaType::U, 1);
    t.activate(P0, sk, 0, &[Entity::Object(bolt)]).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(marked(&t, sk), 3);
}

#[test]
fn spellskite_cant_take_a_target_that_would_make_another_illegal() {
    cr!("115.7", "115.3");
    ruling!(
        "Spellskite",
        "If changing one target of a spell or ability to Spellskite would make other targets of that spell or ability illegal, that target can't be changed to Spellskite."
    );
    supported("Forked Bolt");
    let mut t = TestGame::new(2);
    let sk = t.battlefield(P0, "Spellskite");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    crate::r_s29_common::divide(&mut t, P1, &[1, 1]);
    let fb = crate::r_s33_common::cast_one_slot(
        &mut t,
        P1,
        "Forked Bolt",
        &[Entity::Object(sk), Entity::Object(bears)],
    );
    add_mana(&mut t, P0, ManaType::U, 1);
    t.activate(P0, sk, 0, &[Entity::Object(fb)]).unwrap();
    t.resolve_all();
    assert_eq!(marked(&t, sk), 1);
    assert_eq!(marked(&t, bears), 1, "the Bears is still a target");
    // Positive control: a Forked Bolt with the Bears as its only target is redirected.
    let mut t = TestGame::new(2);
    let sk = t.battlefield(P0, "Spellskite");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    let fb = crate::r_s33_common::cast_one_slot(&mut t, P1, "Forked Bolt", &[Entity::Object(bears)]);
    add_mana(&mut t, P0, ManaType::U, 1);
    t.activate(P0, sk, 0, &[Entity::Object(fb)]).unwrap();
    t.resolve_all();
    assert_eq!(marked(&t, sk), 2);
    assert!(t.on_battlefield(bears) && marked(&t, bears) == 0);
}

#[test]
fn vorinclex_untap_lock_resolves_without_it() {
    cr!("605.1b", "502.3", "608.2");
    ruling!(
        "Vorinclex, Voice of Hunger",
        "If Vorinclex leaves the battlefield after its second ability has triggered, that ability still resolves and the affected land won't untap during its controller's next untap step."
    );
    supported("Vorinclex, Voice of Hunger");
    let mut t = TestGame::new(2);
    let v = t.battlefield(P0, "Vorinclex, Voice of Hunger");
    let forest = t.battlefield(P1, "Forest");
    let other = t.battlefield(P1, "Forest");
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P1, forest, "{G}"));
    destroy(&mut t, v);
    t.resolve_all();
    t.g.tap(other);
    t.advance_to(P1, Step::Upkeep);
    assert!(tapped(&t, forest), "doesn't untap");
    assert!(!tapped(&t, other));
}

#[test]
fn vorinclex_adds_one_mana_of_one_type_the_land_produced() {
    cr!("605.1b", "106.12");
    ruling!(
        "Vorinclex, Voice of Hunger",
        "If a land you control produces multiple mana of more than one type, Vorinclex's first triggered ability adds one mana of only one of those types. You choose which of those types it adds."
    );
    supported("Azorius Chancery");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vorinclex, Voice of Hunger");
    let ch = t.battlefield(P0, "Azorius Chancery");
    let from = t.asked().len();
    assert!(crate::r_s20_common::tap_for_mana(&mut t, P0, ch, "{W}{U}"));
    t.resolve_all();
    let (w, u) = (pool(&t, P0, ManaType::W), pool(&t, P0, ManaType::U));
    assert_eq!(w + u, 3);
    assert!((w, u) == (2, 1) || (w, u) == (1, 2));
    let choices: Vec<Vec<String>> = t.asked()[from..]
        .iter()
        .filter_map(|(_, d)| match d {
            Decision::ChooseOption { options, .. } => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(choices.len(), 1, "P0 chooses the type");
    assert_eq!(choices[0].len(), 2);
}

#[test]
fn urabrask_doesnt_tap_creatures_entering_with_it() {
    cr!("614.12", "614.1c");
    ruling!(
        "Urabrask the Hidden",
        "If a creature an opponent controls enters the battlefield at the same time that Urabrask the Hidden enters the battlefield under your control, Urabrask's effect doesn't apply to your opponent's creature."
    );
    supported("Urabrask the Hidden");
    let mut t = TestGame::new(2);
    let ids = enter_together(
        &mut t,
        &[(P0, "Urabrask the Hidden"), (P1, "Grizzly Bears")],
    );
    assert!(!tapped(&t, ids[1]));
    let later = crate::r_s05_common::enter(&mut t, P1, "Grizzly Bears");
    assert!(tapped(&t, later));
}

#[test]
fn xantcha_token_copy_is_owned_by_its_creator() {
    cr!("111.2", "110.2a");
    ruling!(
        "Xantcha, Sleeper Agent",
        "If a player creates a token that's a copy of Xantcha, the player who creates the token is its owner, not the player under whose control it enters the battlefield."
    );
    let mut t = TestGame::new(3);
    let x = t.battlefield(P2, "Xantcha, Sleeper Agent");
    t.answer(P0, DecisionKind::Any, Answer::Entities(vec![Entity::Player(P1)]));
    crate::r_s17_common::token_copy(&mut t, P0, x);
    let tok: Vec<_> = t
        .named_on_battlefield("Xantcha, Sleeper Agent")
        .into_iter()
        .filter(|id| t.g.obj(*id).is_token())
        .collect();
    assert_eq!(tok.len(), 1);
    assert_eq!(t.g.obj(tok[0]).owner, P0);
    assert_eq!(t.g.obj(tok[0]).controller, P1);
}

/// P1 loses `n` life (as an effect would).
fn lose_life(t: &mut TestGame, p: PlayerId, n: i32) {
    crate::r_s04_common::run_with(
        t,
        P0,
        mtg_engine::ability::Effect::LoseLife {
            who: mtg_engine::ability::PlayerRef::Player(p),
            n: mtg_engine::ability::Value::c(n),
        },
        &[],
    );
}

#[test]
fn belbe_triggers_in_each_postcombat_main_phase() {
    cr!("505.1a", "500.8");
    ruling!(
        "Belbe, Corrupted Observer",
        "If an effect creates an additional combat phase in a turn, it may also create an additional main phase after that combat phase. Belbe's ability triggers at the beginning of each of these postcombat main phases."
    );
    supported("Belbe, Corrupted Observer");
    supported("Relentless Assault");
    let mut t = TestGame::new(2);
    let belbe = t.battlefield(P0, "Belbe, Corrupted Observer");
    lose_life(&mut t, P1, 1);
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(crate::r_p189_common::triggers_of(&t, belbe), 1);
    cast_new(&mut t, P0, "Relentless Assault", &[]);
    t.resolve_all();
    let ok = t.g.run_until(10_000, |g| {
        g.turn.combat_phases == 2
            && g.turn.step == Step::PostcombatMain
            && g.turn.stage == Stage::Priority
    });
    assert!(ok);
    t.settle();
    assert_eq!(crate::r_p189_common::triggers_of(&t, belbe), 2);
    t.resolve_all();
    assert_eq!(pool(&t, P0, ManaType::C), 2);
}

#[test]
fn belbe_rewards_the_active_opponent_who_lost_life() {
    cr!("505.1a", "106.4");
    ruling!(
        "Belbe, Corrupted Observer",
        "If an opponent loses life before their own postcombat main phase, Belbe will reward their suffering with mana."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Belbe, Corrupted Observer");
    t.set_step(P1, Step::PrecombatMain);
    lose_life(&mut t, P1, 1);
    t.advance_to(P1, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(pool(&t, P1, ManaType::C), 2);
    assert_eq!(pool(&t, P0, ManaType::C), 0);
}

#[test]
fn belbe_counts_an_opponent_who_lost_life_then_the_game() {
    cr!("800.4i", "505.1a");
    interp!("history-counts-players-who-left");
    ruling!(
        "Belbe, Corrupted Observer",
        "If an opponent lost life and subsequently lost the game, Belbe's triggered ability still counts that player to determine how much mana to add."
    );
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Belbe, Corrupted Observer");
    lose_life(&mut t, P1, 1);
    lose_life(&mut t, P2, 1);
    crate::r_p079_common::concede(&mut t, P1);
    assert!(t.has_lost(P1));
    t.advance_to(P0, Step::PostcombatMain);
    t.resolve_all();
    assert_eq!(pool(&t, P0, ManaType::C), 4);
}

#[test]
fn nested_ghoul_triggers_once_per_source() {
    cr!("603.2c", "510.2");
    ruling!(
        "Nested Ghoul",
        "If multiple sources deal damage to Nested Ghoul simultaneously (because it was blocked by multiple creatures, for example), the ability will trigger for each source dealing damage to Nested Ghoul."
    );
    supported("Nested Ghoul");
    let mut t = TestGame::new(2);
    let ghoul = t.battlefield(P0, "Nested Ghoul");
    let a = t.battlefield(P1, "Llanowar Elves");
    let b = t.battlefield(P1, "Llanowar Elves");
    attack_with(&mut t, &[(ghoul, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(a, ghoul), (b, ghoul)]);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
}

#[test]
fn phyrexian_gremlins_tapping_itself_as_an_artifact_never_untaps() {
    cr!("502.3", "611.2b");
    ruling!(
        "Phyrexian Gremlins",
        "If the Gremlins somehow become an artifact and then tap themselves, they can never be untapped during the untap phase."
    );
    supported("Phyrexian Gremlins");
    let mut t = TestGame::new(2);
    let gr = t.battlefield(P0, "Phyrexian Gremlins");
    let licid = t.battlefield(P0, "Transmogrifying Licid");
    add_mana(&mut t, P0, ManaType::C, 1);
    t.answer_yes(P0, false);
    t.activate(P0, licid, 0, &[Entity::Object(gr)]).unwrap();
    t.resolve_all();
    assert!(t.obj_now(gr).is(CardType::Artifact));
    t.activate(P0, gr, 0, &[Entity::Object(gr)]).unwrap();
    t.resolve_all();
    assert!(tapped(&t, gr));
    // P0 chooses to untap it.
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    assert!(tapped(&t, gr), "it stays tapped");
    // Positive control: tapped some other way, it untaps.
    let mut t = TestGame::new(2);
    let gr = t.battlefield(P0, "Phyrexian Gremlins");
    t.g.tap(gr);
    t.answer_yes(P0, true);
    crate::r_s04_common::next_upkeep(&mut t, P0);
    assert!(!tapped(&t, gr));
}

#[test]
fn chancellor_of_the_spires_spell_still_needs_its_additional_cost() {
    cr!("601.2b", "118.9d");
    ruling!(
        "Chancellor of the Spires",
        "If the card has any mandatory additional costs, then you must pay them in order to cast the spell."
    );
    supported("Chancellor of the Spires");
    supported("Bone Splinters");
    let mut t = TestGame::new(2);
    let splinters = t.graveyard(P1, "Bone Splinters");
    let fodder = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(splinters)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(fodder)]);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    crate::r_s05_common::enter(&mut t, P0, "Chancellor of the Spires");
    t.resolve_all();
    assert!(!t.on_battlefield(fodder), "sacrificed for the additional cost");
    assert!(!t.on_battlefield(giant));
}

#[test]
fn history_counts_include_opponents_who_left() {
    cr!("800.4i");
    interp!("history-counts-players-who-left");
    ruling!(
        "Teysa, Opulent Oligarch",
        "If an opponent lost life and subsequently lost the game, Teysa's second ability still counts that player when determining how many times to investigate."
    );
    supported("Teysa, Opulent Oligarch");
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Teysa, Opulent Oligarch");
    lose_life(&mut t, P1, 1);
    lose_life(&mut t, P2, 1);
    crate::r_p079_common::concede(&mut t, P1);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(with_subtype(&t, P0, "Clue").len(), 2);
}
