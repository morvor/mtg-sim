//! Rulings batch P155 — interactions: a "blocks or becomes blocked" trigger with several
//! blockers (CR 509.1h, 603.2c), simultaneous life loss and a drawn game (CR 104.4a,
//! 704.5a), damage redirection with several objects (CR 615.6, 616.1), a state trigger
//! (CR 603.8), a permanent made a Vehicle (CR 205.1a, 613.1d), tokens copying printed
//! values (CR 707.2), spells whose only target became illegal (CR 608.2b), a cast trigger
//! outliving its spell (CR 603.3), an Aura that becomes an Equipment (CR 301.5, 704.5m).

use crate::r_p155_common::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::game::GameResult;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn karn_silver_golems_blocked_trigger_triggers_once_for_several_blockers() {
    cr!("509.1h", "603.2c");
    ruling!(
        "Karn, Silver Golem",
        "The blocking ability triggers only once no matter how many blockers are declared."
    );
    supported("Karn, Silver Golem");
    let mut t = TestGame::new(2);
    let karn = t.battlefield(P0, "Karn, Silver Golem");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    t.attack(&[(karn, pl(P1))], &[(a, karn), (b, karn)]);
    assert_eq!(t.pt(karn), (0, 8), "{}", t.dump_log());
    assert!(t.on_battlefield(karn));
    assert!(t.on_battlefield(a) && t.on_battlefield(b), "Karn dealt 0 damage");
}

#[test]
fn duskmantle_seers_life_loss_is_simultaneous_and_can_draw_the_game() {
    cr!("104.4a", "704.5a", "608.2f");
    ruling!(
        "Duskmantle Seer",
        "The loss of life is simultaneous. If this causes both players to end up at 0 or less life, the game will be a draw. In a multiplayer game, players with 0 or less life will lose the game."
    );
    supported("Duskmantle Seer");
    // Two players at 3 and 2 life, Hill Giants on top: both lose 4 at once: a draw.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Duskmantle Seer");
    t.library_top(P0, "Hill Giant");
    t.library_top(P1, "Hill Giant");
    set_life(&mut t, P0, 3);
    set_life(&mut t, P1, 2);
    t.set_step(P1, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert_eq!((t.life(P0), t.life(P1)), (-1, -2), "{}", t.dump_log());
    assert_eq!(t.g.result, Some(GameResult::Draw), "{}", t.dump_log());
    // Three players: P0 and P1 lose, P2 (at 20) wins.
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Duskmantle Seer");
    t.library_top(P0, "Hill Giant");
    t.library_top(P1, "Hill Giant");
    t.library_top(P2, "Hill Giant");
    set_life(&mut t, P0, 3);
    set_life(&mut t, P1, 2);
    t.set_step(P2, Step::End);
    t.advance_to(P0, Step::Upkeep);
    t.resolve_all();
    assert!(t.has_lost(P0) && t.has_lost(P1) && !t.has_lost(P2), "{}", t.dump_log());
    assert_eq!(t.g.result, Some(GameResult::Win(vec![P2])));
}

#[test]
fn duskmantle_seer_puts_cards_into_hands_without_drawing_them() {
    cr!("121.1", "702.94a");
    ruling!(
        "Duskmantle Seer",
        "The cards put into hands this way are not “drawn.” For example, you couldn’t reveal a card with miracle put into your hand this way (but it also won’t count as the first card you’ve drawn this turn either)."
    );
    supported("Duskmantle Seer");
    supported("Thunderous Wrath");
    // Thunderous Wrath ({4}{R}{R}, miracle {R}) twice on top of P0's library.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Duskmantle Seer");
    t.lands(P0, "Mountain", 1);
    t.library_top(P0, "Thunderous Wrath");
    t.library_top(P0, "Thunderous Wrath");
    t.library_top(P1, "Forest");
    t.set_step(P1, Step::End);
    let from = t.asked().len();
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    t.resolve_all();
    // The first Wrath went to hand without being drawn: no miracle trigger.
    assert_eq!(t.hand_size(P0), 1);
    assert_eq!(t.life(P0), 14, "{}", t.dump_log());
    assert_eq!(t.stack_len(), 0, "no miracle: {}", t.dump_log());
    assert_eq!(yes_nos_asked(&t, P0, from), 0);
    // The draw step's Wrath is the first card drawn this turn: miracle may be revealed.
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::Draw);
    t.settle();
    assert_eq!(t.hand_size(P0), 2);
    assert!(
        t.stack_len() == 1 || yes_nos_asked(&t, P0, from) >= 1,
        "miracle offered: {}",
        t.dump_log()
    );
}

#[test]
fn shining_shoal_redirects_damage_to_you_and_your_creatures_as_you_choose_up_to_x() {
    cr!("615.6", "616.1", "616.1c");
    ruling!(
        "Shining Shoal",
        "The redirection protects both you and your creatures from the source. If the source would deal more than X damage to multiple objects, you decide which damage is redirected until X is used up. For example, your opponent casts a spell that deals 1 damage to you and 1 damage to two of your creatures. If you cast Shining Shoal with X = 2, you can have damage to you and one creature redirected to the target, or you could have damage to both creatures redirected."
    );
    supported("Shining Shoal");
    supported("Arc Lightning");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P1, Step::PrecombatMain);
    // Arc Lightning: 1 damage to P0, Bears A and Bears B.
    lots(&mut t, P1);
    let arc = t.hand(P1, "Arc Lightning");
    t.answer(P1, DecisionKind::Divide, Answer::Numbers(vec![1, 1, 1]));
    let arc = t.cast(P1, arc).targets(&[pl(P0), obj(a), obj(b)]).go();
    t.settle();
    let shoal = t.hand(P0, "Shining Shoal");
    mana(&mut t, P0, "2WW");
    t.answer_choose(P0, &[obj(arc)]);
    t.cast(P0, shoal).x(2).target(pl(P1)).go();
    t.resolve();
    let from = t.asked().len();
    t.resolve_all();
    // Two of the three points were redirected to P1; one was dealt as normal.
    assert_eq!(t.life(P1), 18, "{}", t.dump_log());
    let dealt = (t.life(P0) < 20) as u32 + dmg(&t, a) + dmg(&t, b);
    assert_eq!(dealt, 1, "life {} damage {} {}: {}", t.life(P0), dmg(&t, a), dmg(&t, b), t.dump_log());
    // P0 chose which damage was redirected.
    assert!(
        t.asked()[from..].iter().any(|(p, _)| *p == P0),
        "no choice asked of P0: {:?}",
        &t.asked()[from..]
    );
}

#[test]
fn endrek_sahrs_state_trigger_triggers_once_until_it_leaves_the_stack_and_sacrifices_regardless() {
    cr!("603.8", "608.2c");
    ruling!(
        "Endrek Sahr, Master Breeder",
        "The second ability is a state trigger. Once it triggers, it won't trigger again as long as the ability is on the stack. If the ability is countered and the trigger condition is still true, it will immediately trigger again."
    );
    ruling!(
        "Endrek Sahr, Master Breeder",
        "When the second ability resolves, Endrek Sahr will be sacrificed regardless of how many Thrulls you control at that time, even if it's less than seven."
    );
    supported("Endrek Sahr, Master Breeder");
    supported("Stifle");
    let mut t = TestGame::new(2);
    let endrek = t.battlefield(P0, "Endrek Sahr, Master Breeder");
    let thrulls: Vec<ObjectId> = (0..7).map(|_| create_token(&mut t, P0, "Thrull")).collect();
    t.settle();
    assert_eq!(t.stack_len(), 1, "{}", t.dump_log());
    t.settle();
    assert_eq!(t.stack_len(), 1, "not again while on the stack");
    // Countered: it triggers again at once.
    let ability = top_of_stack(&t);
    cast(&mut t, P0, "Stifle", "U", &[obj(ability)]);
    t.resolve();
    assert!(!on_stack(&t, ability));
    assert_eq!(t.stack_len(), 1, "triggered again: {}", t.dump_log());
    assert!(top_is_trigger_of(&t, endrek));
    // A Thrull leaves in response: Endrek is sacrificed all the same.
    kill(&mut t, thrulls[0]);
    assert_eq!(with_subtype(&t, P0, "Thrull").len(), 6);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Endrek Sahr, Master Breeder"), "{}", t.dump_log());
}

#[test]
fn captain_rex_nebula_makes_a_vehicle_keeping_supertypes_and_losing_creature_types() {
    cr!("205.1a", "205.1b", "613.1d");
    ruling!(
        "Captain Rex Nebula",
        "The target of Rex Nebula's ability will become a Vehicle artifact until end of turn. It will lose all of its previous card types and subtypes, but it will keep any supertypes (such as legendary) it previously had. If the target of Rex Nebula's ability is a creature, it will lose all creature types (such as Zombie) it had until end of turn, and becoming crewed will not cause it to regain those subtypes."
    );
    supported("Captain Rex Nebula");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Captain Rex Nebula");
    let jedit = t.battlefield(P0, "Jedit Ojanen");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(jedit)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    let c = &t.obj_now(jedit).chars;
    assert!(c.card_types.contains(CardType::Artifact), "{c:?}");
    assert!(!c.card_types.contains(CardType::Creature));
    assert!(c.has_subtype("Vehicle"));
    assert!(!c.has_subtype("Cat") && !c.has_subtype("Warrior"));
    assert!(c.supertypes.contains(Supertype::Legendary));
    // Crewed: a 7/7 artifact creature, still without its creature types.
    assert!(crate::r_s04_common::crew(&mut t, P0, jedit, &[bears]), "{}", t.dump_log());
    t.resolve_all();
    let c = &t.obj_now(jedit).chars;
    assert!(c.card_types.contains(CardType::Creature));
    assert_eq!(t.pt(jedit), (7, 7));
    assert!(!c.has_subtype("Cat") && !c.has_subtype("Warrior"));
    assert!(c.has_subtype("Vehicle"));
}

#[test]
fn satyas_token_copies_the_printed_creature_and_chooses_what_it_attacks() {
    cr!("707.2", "707.3", "508.4");
    ruling!(
        "Satya, Aetherflux Genius",
        "The token copies exactly what was printed on the original card and nothing else (unless that creature is itself copying something else; see below). It doesn't copy whether that creature is tapped or untapped, whether it has any counters on it or Auras attached to it, or any non-copy effects that have changed its power, toughness, types, color, and so on."
    );
    ruling!(
        "Satya, Aetherflux Genius",
        "You choose which player, planeswalker, or battle the token is attacking as you create the token. It doesn't need to be attacking the same player, planeswalker, or battle that Satya is attacking."
    );
    supported("Satya, Aetherflux Genius");
    let mut t = TestGame::new(2);
    let satya = t.battlefield(P0, "Satya, Aetherflux Genius");
    let bears = t.battlefield(P0, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 1);
    cast_resolve(&mut t, P0, "Giant Growth", "G", &[obj(bears)]);
    t.g.tap(bears);
    assert_eq!(t.pt(bears), (6, 6));
    let jace = t.battlefield(P1, "Jace Beleren");
    let before = t.g.battlefield.clone();
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_choose(P0, &[obj(jace)]);
    attack_with(&mut t, &[(satya, pl(P1))]);
    t.resolve_all();
    let tok = *t
        .g
        .battlefield
        .iter()
        .find(|o| !before.contains(o) && t.g.obj(**o).is_token())
        .expect("the token");
    assert_eq!(t.obj_now(tok).chars.name, "Grizzly Bears");
    assert_eq!(t.pt(tok), (2, 2), "no counter, no Giant Growth");
    assert_eq!(t.counters(tok, counters::PLUS1), 0);
    assert!(t.obj_now(tok).tapped);
    assert!(crate::r_p195_common::attacking(&t, tok));
    let attacking = t
        .g
        .combat
        .as_ref()
        .and_then(|c| c.attackers.iter().find(|a| a.id == tok).map(|a| a.target));
    assert_eq!(attacking, Some(Some(obj(jace))), "attacks Jace: {}", t.dump_log());
    assert_eq!(t.g.player(P0).counter(counters::ENERGY), 2);
}

#[test]
fn will_of_the_temurs_token_copies_the_printed_permanent_as_a_4_4_flying_dragon() {
    cr!("707.2", "707.9b");
    ruling!(
        "Will of the Temur",
        "The token created by Will of the Temur’s first mode copies exactly what was printed on the original permanent and nothing else, with the listed exceptions (unless that permanent is copying something else or is a token; see below). It doesn’t copy whether that permanent is tapped or untapped, whether it has any counters on it or Auras and Equipment attached to it, or any non-copy effects that have changed its power, toughness, types, color, or so on."
    );
    supported("Will of the Temur");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    put(&mut t, bears, counters::PLUS1, 1);
    cast_resolve(&mut t, P1, "Giant Growth", "G", &[obj(bears)]);
    t.g.tap(bears);
    let before = t.g.battlefield.clone();
    lots(&mut t, P0);
    let will = t.hand(P0, "Will of the Temur");
    t.answer(P0, DecisionKind::Modes, Answer::Indices(vec![0]));
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, will).go();
    t.resolve_all();
    let tok = *t
        .g
        .battlefield
        .iter()
        .find(|o| !before.contains(o) && t.g.obj(**o).is_token())
        .expect("the token");
    let c = t.obj_now(tok).chars.clone();
    assert_eq!(c.name, "Grizzly Bears");
    assert_eq!(t.pt(tok), (4, 4), "{}", t.dump_log());
    assert!(c.has_subtype("Dragon") && c.has_subtype("Bear"));
    assert!(c.has_keyword(KeywordKind::Flying));
    assert_eq!(t.counters(tok, counters::PLUS1), 0);
    assert!(!t.obj_now(tok).tapped);
}

#[test]
fn mutation_spells_make_no_saprolings_when_their_target_is_illegal() {
    cr!("608.2b");
    ruling!(
        "Aura Mutation",
        "This spell doesn't resolve and you get no Saprolings if the enchantment is an illegal target on resolution."
    );
    ruling!(
        "Artifact Mutation",
        "This spell is countered and you get no Saprolings if the artifact is an illegal target on resolution."
    );
    ruling!(
        "Aether Mutation",
        "You only put the tokens onto the battlefield if the target creature is still a legal target on resolution."
    );
    for (spell, cost, target, gone) in [
        ("Aura Mutation", "GW", "Pacifism", false),
        ("Artifact Mutation", "RG", "Sol Ring", false),
        ("Aether Mutation", "3GU", "Hill Giant", true),
    ] {
        supported(spell);
        let mut t = TestGame::new(2);
        let perm = if target == "Pacifism" {
            let bears = t.battlefield(P1, "Grizzly Bears");
            crate::r_s06_common::attach_new(&mut t, P1, "Pacifism", bears)
        } else {
            t.battlefield(P1, target)
        };
        let s = cast(&mut t, P0, spell, cost, &[obj(perm)]);
        if gone {
            bounce(&mut t, perm);
        } else {
            kill(&mut t, perm);
        }
        t.resolve_all();
        assert_eq!(creature_tokens(&t, P0), 0, "{spell}: {}", t.dump_log());
        assert!(!resolved(&t, s) && t.in_graveyard(P0, spell), "{spell}");
        // With the target in place: tokens for its mana value.
        let mut t = TestGame::new(2);
        let perm = if target == "Pacifism" {
            let bears = t.battlefield(P1, "Grizzly Bears");
            crate::r_s06_common::attach_new(&mut t, P1, "Pacifism", bears)
        } else {
            t.battlefield(P1, target)
        };
        let mv = mana_value(&t, perm) as usize;
        cast_resolve(&mut t, P0, spell, cost, &[obj(perm)]);
        assert_eq!(creature_tokens(&t, P0), mv, "{spell}");
    }
}

#[test]
fn ulamogs_cast_trigger_resolves_before_it_and_even_if_its_countered() {
    cr!("603.3", "603.2", "405.5");
    ruling!(
        "Ulamog, the Defiler",
        "Ulamog, the Defiler's first ability will resolve before Ulamog does. If Ulamog is countered or otherwise leaves the stack in response to its triggered ability, the triggered ability will still resolve as normal."
    );
    supported("Ulamog, the Defiler");
    let mut t = TestGame::new(2);
    let u = t.hand(P0, "Ulamog, the Defiler");
    mana(&mut t, P0, "55");
    t.answer_targets(P0, &[pl(P1)]);
    t.cast(P0, u).go();
    t.settle();
    assert_eq!(t.stack_len(), 2);
    assert!(top_is_trigger_of(&t, u));
    cast(&mut t, P1, "Cancel", "1UU", &[obj(u)]);
    t.resolve();
    assert!(t.in_graveyard(P0, "Ulamog, the Defiler"));
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.library_size(P1), 15, "half of 30 exiled: {}", t.dump_log());
}

#[test]
fn bludgeon_brawl_makes_an_artifact_aura_an_equipment_too() {
    cr!("301.5", "303.4", "704.5m", "702.6a");
    ruling!(
        "Bludgeon Brawl",
        "While Bludgeon Brawl is on the battlefield, an Aura enchantment that somehow becomes an artifact in addition to its other types will also become an Equipment. Any of its abilities that refer to either “enchanted creature” or “equipped creature” refer to the creature it’s currently attached to. It can be attached to other creatures using its equip ability. If you try to attach the Aura Equipment to a creature it can’t legally be attached to, it remains where it is. If the creature it’s attached to becomes an illegal permanent for it to enchant, the Aura Equipment will be put into its owner’s graveyard as a state-based action."
    );
    supported("Mycosynth Lattice");
    supported("Holy Strength");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Mycosynth Lattice");
    t.battlefield(P0, "Bludgeon Brawl");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    let hs = crate::r_s06_common::attach_new(&mut t, P0, "Holy Strength", bears);
    t.settle();
    let c = t.obj_now(hs).chars.clone();
    assert!(c.card_types.contains(CardType::Artifact) && c.has_subtype("Equipment") && c.has_subtype("Aura"), "{c:?}");
    // "Enchanted creature gets +1/+2" and "equipped creature gets +1/+0" (X = 1).
    assert_eq!(t.pt(bears), (4, 4), "{}", t.dump_log());
    // Equip {1} moves it to the Giant.
    activate_resolve(&mut t, P0, hs, "Equip", "1", &[obj(giant)]);
    assert_eq!(crate::r_s06_common::attached_to(&t, hs), Some(obj(giant)), "{}", t.dump_log());
    assert_eq!(t.pt(giant), (5, 5));
    assert_eq!(t.pt(bears), (2, 2));
    // The creature it enchants leaves: an Aura not attached is put into the graveyard.
    kill(&mut t, giant);
    assert!(t.in_graveyard(P0, "Holy Strength"), "{}", t.dump_log());
}
