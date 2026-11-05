//! Rulings batch P195 — effects that untap other permanents: what they can target (an
//! untapped permanent, an opponent's land, two different lands), that they use the stack
//! (not mana abilities), and untapping or giving flying to an attacking creature during
//! combat; plus combat rulings of creatures that make tokens or assign damage
//! (Ghalta and Mavren, Kjeldoran Home Guard, Deathcoil Wurm, Shriekwood Devourer).

use crate::r_p195_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Moves to P0's declare attackers step, declaring `attackers` at P1.
fn declare(t: &mut TestGame, attackers: &[ObjectId]) {
    let a = attackers.iter().map(|a| (*a, pl(P1))).collect();
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(a));
    t.advance_to(P0, Step::DeclareAttackers);
}

#[test]
fn untap_effects_can_target_untapped_permanents() {
    cr!("115.1", "701.26b", "608.2b");
    ruling!(
        "Hyrax Tower Scout",
        "Hyrax Tower Scout’s ability can target an untapped creature, including Hyrax Tower Scout itself, if there are no tapped creatures you wish to untap."
    );
    ruling!(
        "Vault Skyward",
        "Vault Skyward can target any creature, including one that already has flying or is already untapped."
    );
    ruling!(
        "Wings of the Cosmos",
        "Wings of the Cosmos can target a creature that’s already untapped. It will still get +1/+3 and gain flying until end of turn."
    );
    ruling!("Argothian Elder", "The ability can target an already untapped land.");
    ruling!("Candelabra of Tawnos", "Can be used on an untapped land.");
    ruling!(
        "Candelabra of Tawnos",
        "You may untap your opponent's lands if desired."
    );
    // Hyrax Tower Scout can target itself as it enters.
    supported("Hyrax Tower Scout");
    let mut t = TestGame::new(2);
    let from = t.asked().len();
    let h = t.enter(P0, "Hyrax Tower Scout");
    t.answer_targets(P0, &[obj(h)]);
    t.settle();
    assert!(target_choices(&t, P0, from, "")
        .iter()
        .any(|c| c.contains(&obj(h))));
    t.resolve_all();
    assert!(!tapped(&t, h));
    // Vault Skyward on an untapped creature with flying; Wings of the Cosmos on an
    // untapped creature.
    supported("Vault Skyward");
    let mut t = TestGame::new(2);
    let angel = t.battlefield(P0, "Serra Angel");
    cast_at(&mut t, P0, "Vault Skyward", &[obj(angel)]);
    t.resolve_all();
    assert!(!tapped(&t, angel));
    assert!(t.obj_now(angel).has_keyword(KeywordKind::Flying));
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_at(&mut t, P0, "Wings of the Cosmos", &[obj(bears)]);
    t.resolve_all();
    assert_eq!(t.pt(bears), (3, 5));
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    // Argothian Elder on an untapped land and a tapped one.
    supported("Argothian Elder");
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Argothian Elder");
    let a = t.battlefield(P0, "Forest");
    let b = t.battlefield(P0, "Forest");
    tap(&mut t, b);
    t.activate(P0, e, 0, &[obj(a), obj(b)]).unwrap();
    t.resolve_all();
    assert!(!tapped(&t, a) && !tapped(&t, b));
    // Candelabra of Tawnos: an untapped land of yours and an opponent's tapped land.
    supported("Candelabra of Tawnos");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Candelabra of Tawnos");
    let mine = t.battlefield(P0, "Forest");
    let theirs = t.battlefield(P1, "Island");
    tap(&mut t, theirs);
    mana(&mut t, P0, mtg_engine::mana::ManaType::C, 2);
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.answer_targets(P0, &[obj(mine), obj(theirs)]);
    t.activate(P0, c, 0, &[]).unwrap();
    t.resolve_all();
    assert!(!tapped(&t, mine) && !tapped(&t, theirs));
}

#[test]
fn argothian_elder_needs_two_different_lands() {
    cr!("115.3", "601.2c", "602.2b");
    ruling!(
        "Argothian Elder",
        "The ability must target two different lands. It can’t be activated if there is just one land to target."
    );
    let mut t = TestGame::new(2);
    let e = t.battlefield(P0, "Argothian Elder");
    let a = t.battlefield(P0, "Forest");
    assert!(t.activate(P0, e, 0, &[obj(a), obj(a)]).is_err());
    assert!(!tapped(&t, e));
}

#[test]
fn land_untappers_are_not_mana_abilities() {
    cr!("605.1a", "602.2", "405.1");
    ruling!(
        "Candelabra of Tawnos",
        "This is not mana ability. It is a normal ability and it will resolve along with other spells and abilities on the stack. The lands untap during resolution."
    );
    ruling!(
        "Greenside Watcher",
        "Greenside Watcher’s ability is not a mana ability. It uses the stack and can be responded to."
    );
    ruling!(
        "Ley Druid",
        "The land is untapped as a normal ability that goes on the stack. It is not a mana ability, so it can’t be used at times that only mana abilities can be used."
    );
    // (card, land it untaps)
    for (name, land) in [
        ("Candelabra of Tawnos", "Forest"),
        ("Greenside Watcher", "Azorius Guildgate"),
        ("Ley Druid", "Forest"),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let src = t.battlefield(P0, name);
        let l = t.battlefield(P0, land);
        tap(&mut t, l);
        assert!(!t
            .obj_now(src)
            .chars
            .abilities
            .iter()
            .any(|a| a.is_mana_ability()));
        if name == "Candelabra of Tawnos" {
            mana(&mut t, P0, mtg_engine::mana::ManaType::C, 1);
            t.answer(P0, DecisionKind::X, Answer::Number(1));
        }
        t.activate(P0, src, 0, &[obj(l)]).unwrap();
        assert_eq!(t.stack_len(), 1, "{name}");
        assert!(tapped(&t, l));
        t.resolve();
        assert!(!tapped(&t, l), "{name}");
    }
}

#[test]
fn untapping_an_attacking_creature_keeps_it_in_combat() {
    cr!("506.4", "508.1f", "702.20b");
    ruling!(
        "Bazaar Krovod",
        "Bazaar Krovod’s ability can target an attacking creature that’s untapped (such as one with vigilance)."
    );
    ruling!(
        "Bazaar Krovod",
        "Untapping an attacking creature doesn’t cause it to stop attacking or remove it from combat."
    );
    supported("Bazaar Krovod");
    for other in ["Serra Angel", "Grizzly Bears"] {
        let mut t = TestGame::new(2);
        let k = t.battlefield(P0, "Bazaar Krovod");
        let c = t.battlefield(P0, other);
        t.answer_targets(P0, &[obj(c)]);
        declare(&mut t, &[k, c]);
        t.resolve_all();
        assert!(!tapped(&t, c), "{other}");
        assert!(attacking(&t, c), "{other}");
        let toughness = t.pt(c).1;
        assert_eq!(toughness, card_toughness(other) + 2);
        t.advance_to(P0, Step::EndOfCombat);
        assert_eq!(t.life(P1), 20 - 2 - t.pt(c).0);
    }
}

fn card_toughness(name: &str) -> i32 {
    match name {
        "Serra Angel" => 4,
        _ => 2,
    }
}

#[test]
fn all_attackers_are_declared_at_once() {
    cr!("508.1a", "508.1f");
    ruling!(
        "Dauntless Aven",
        "All attackers are chosen at once. You can’t attack with Dauntless Aven, untap a tapped creature, and then attack with that creature."
    );
    supported("Dauntless Aven");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Dauntless Aven");
    let bears = t.battlefield(P0, "Grizzly Bears");
    tap(&mut t, bears);
    let from = t.asked().len();
    t.answer_targets(P0, &[obj(bears)]);
    declare(&mut t, &[a]);
    // The tapped Bears couldn't be declared as an attacker.
    let options = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::DeclareAttackers { options } => Some(options.clone()),
            _ => None,
        })
        .unwrap();
    assert!(!options.iter().any(|(id, _)| *id == bears));
    t.resolve_all();
    assert!(!tapped(&t, bears));
    assert!(!attacking(&t, bears));
}

#[test]
fn flying_after_blocks_does_not_undo_the_block() {
    cr!("509.1b", "506.4", "702.9b");
    ruling!(
        "Vault Skyward",
        "For flying to work as an evasion ability, Vault Skyward must be cast before the declare blockers step begins. Once a creature has become blocked, giving it flying won’t change that."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![(bears, pl(P1))]));
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(blocker, bears)]));
    t.advance_to(P0, Step::DeclareBlockers);
    cast_at(&mut t, P0, "Vault Skyward", &[obj(bears)]);
    t.resolve_all();
    assert!(t.obj_now(bears).has_keyword(KeywordKind::Flying));
    t.advance_to(P0, Step::EndOfCombat);
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
    assert_eq!(t.life(P1), 20);
    // Cast before blockers are declared, the Giant can't block it.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let blocker = t.battlefield(P1, "Hill Giant");
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![(bears, pl(P1))]));
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(blocker, bears)]));
    t.advance_to(P0, Step::DeclareAttackers);
    cast_at(&mut t, P0, "Vault Skyward", &[obj(bears)]);
    t.resolve_all();
    t.advance_to(P0, Step::EndOfCombat);
    assert_eq!(t.life(P1), 18);
}

#[test]
fn clever_conjurer_can_target_itself_without_its_name() {
    cr!("201.2", "707.9a", "115.1");
    ruling!(
        "Clever Conjurer",
        "If Clever Conjurer somehow loses the name Clever Conjurer, perhaps because you mutated another creature on top of it or copied it with Sakashima the Impostor, it can target itself with its Mage Hand ability. How clever!"
    );
    supported("Clever Conjurer");
    supported("Sakashima the Impostor");
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Clever Conjurer");
    // The Conjurer itself can't be targeted.
    lots(&mut t, P0);
    assert!(t.activate(P0, c, 0, &[obj(c)]).is_err());
    // Sakashima copying it (named Sakashima the Impostor) can target itself.
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(c)]);
    let s = t.enter(P0, "Sakashima the Impostor");
    t.settle();
    t.g.objects[s.0 as usize].summoning_sick = false;
    assert_eq!(t.obj_now(s).chars.name, "Sakashima the Impostor");
    t.activate(P0, s, 0, &[obj(s)]).unwrap();
    t.resolve_all();
    assert!(!tapped(&t, s));
}

#[test]
fn ghalta_dinosaur_attacks_what_you_choose() {
    cr!("508.4", "506.3");
    ruling!(
        "Ghalta and Mavren",
        "You choose which player, planeswalker, or battle the Dinosaur token is attacking. It doesn’t have to be the same player, planeswalker, or battle that Ghalta and Mavren is attacking."
    );
    supported("Ghalta and Mavren");
    let mut t = TestGame::new(2);
    let gm = t.battlefield(P0, "Ghalta and Mavren");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let jace = t.battlefield(P1, "Jace Beleren");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_choose(P0, &[obj(jace)]);
    declare(&mut t, &[gm, bears]);
    t.resolve_all();
    let dino = with_subtype(&t, P0, "Dinosaur")
        .into_iter()
        .find(|d| *d != gm)
        .expect("no Dinosaur token");
    assert_eq!(t.pt(dino), (2, 2));
    assert!(tapped(&t, dino));
    let target = t
        .g
        .combat
        .as_ref()
        .unwrap()
        .attackers
        .iter()
        .find(|a| a.id == dino)
        .and_then(|a| a.target);
    assert_eq!(target, Some(obj(jace)));
}

#[test]
fn kjeldoran_home_guard_makes_the_token_even_if_it_left() {
    cr!("608.2c", "603.4");
    ruling!(
        "Kjeldoran Home Guard",
        "You put the Deserter token onto the battlefield even if you can’t put a -0/-1 counter on Kjeldoran Home Guard (because it left the battlefield after its ability triggered but before it resolved, for example)."
    );
    supported("Kjeldoran Home Guard");
    let mut t = TestGame::new(2);
    let g = t.battlefield(P0, "Kjeldoran Home Guard");
    declare(&mut t, &[g]);
    t.advance_to(P0, Step::EndOfCombat);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    kill(&mut t, g);
    t.resolve_all();
    let deserters = with_subtype(&t, P0, "Deserter");
    assert_eq!(deserters.len(), 1);
    assert_eq!(t.pt(deserters[0]), (0, 1));
}

#[test]
fn deathcoil_wurm_all_or_nothing_and_banding() {
    cr!("510.1c", "702.22j");
    ruling!(
        "Deathcoil Wurm",
        "When assigning combat damage, you choose whether you want to assign all damage to blocking creatures, or if you want to assign all of it to the player or planeswalker this creature is attacking. You can't split the damage assignment between them."
    );
    ruling!(
        "Deathcoil Wurm",
        "If blocked by a creature with banding, the defending player decides whether or not the damage is assigned \"as though it weren't blocked\"."
    );
    supported("Deathcoil Wurm");
    supported("Benalish Hero");
    // (asked player, as though unblocked) -> P1 life, blocker dies
    for (banding, yes) in [(false, true), (false, false), (true, true), (true, false)] {
        let mut t = TestGame::new(2);
        let w = t.battlefield(P0, "Deathcoil Wurm");
        let blocker = t.battlefield(P1, if banding { "Benalish Hero" } else { "Grizzly Bears" });
        let decider = if banding { P1 } else { P0 };
        t.answer_yes(decider, yes);
        t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![(w, pl(P1))]));
        t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![(blocker, w)]));
        let from = t.asked().len();
        t.advance_to(P0, Step::EndOfCombat);
        let asked: Vec<PlayerId> = t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::YesNo { .. }))
            .map(|(p, _)| *p)
            .collect();
        assert_eq!(asked, vec![decider], "banding={banding}");
        if yes {
            assert_eq!(t.life(P1), 13, "banding={banding}");
            assert!(t.on_battlefield(blocker));
        } else {
            assert_eq!(t.life(P1), 20, "banding={banding}");
            assert!(!t.on_battlefield(blocker));
        }
    }
}

#[test]
fn shriekwood_devourer_counts_declared_attackers_on_resolution() {
    cr!("508.4", "608.2h", "603.2");
    ruling!(
        "Shriekwood Devourer",
        "Creatures that enter attacking were never declared as attackers, so Shriekwood Devourer's last ability won't take them into account."
    );
    ruling!(
        "Shriekwood Devourer",
        "The value of X is calculated only once, as Shriekwood Devourer's last ability resolves. This means that the value of X may be different than you had hoped when attackers were declared if attacking creatures have left the battlefield or their power has changed."
    );
    supported("Shriekwood Devourer");
    // Bears (2) attack with Ghalta-made Dinosaur... use a plain case: the Bears attack; X
    // is read on resolution after they get +3/+3.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shriekwood Devourer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lands = t.lands(P0, "Forest", 6);
    for l in &lands {
        tap(&mut t, *l);
    }
    declare(&mut t, &[bears]);
    t.settle();
    assert_eq!(t.stack_len(), 1);
    cast_at(&mut t, P0, "Giant Growth", &[obj(bears)]);
    t.resolve();
    assert_eq!(untap_up_to(&mut t, &lands, 5), 5);
    // A creature put onto the battlefield attacking isn't counted: Ghalta's Dinosaur
    // token (X/X) enters after the trigger; only the Bears count.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Shriekwood Devourer");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let lands = t.lands(P0, "Forest", 6);
    for l in &lands {
        tap(&mut t, *l);
    }
    declare(&mut t, &[bears]);
    t.settle();
    let token = t.custom(
        P0,
        mtg_engine::card::CardDef::custom(mtg_engine::object::Characteristics {
            name: "Big".into(),
            card_types: [CardType::Creature].into_iter().collect(),
            power: Some(6),
            toughness: Some(6),
            ..Default::default()
        }),
        mtg_engine::object::Zone::Battlefield,
    );
    mtg_engine::combat::put_onto_battlefield_attacking(&mut t.g, token, pl(P1));
    assert!(attacking(&t, token));
    assert_eq!(untap_up_to(&mut t, &lands, 2), 2);
}

/// Resolves Shriekwood Devourer's trigger (and the rest of the stack), choosing the
/// first `n` of `lands` to untap. The number it let P0 choose (X).
fn untap_up_to(t: &mut TestGame, lands: &[ObjectId], n: usize) -> u32 {
    let from = t.asked().len();
    let ids: Vec<Entity> = lands[..n].iter().map(|l| obj(*l)).collect();
    t.answer_choose(P0, &ids);
    t.resolve_all();
    let max = t.asked()[from..]
        .iter()
        .find_map(|(_, d)| match d {
            Decision::ChooseEntities { max, .. } => Some(*max),
            _ => None,
        })
        .expect("no land choice");
    let untapped = lands.iter().filter(|l| !tapped(t, **l)).count() as u32;
    assert_eq!(untapped, max);
    max
}
