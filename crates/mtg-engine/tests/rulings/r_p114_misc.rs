//! Rulings batch P114 — other partner commanders' abilities: Brallin and Shabraz, Abdel
//! Adrian, The Fourth Doctor, Tevesh Szat, Blue, Ich-Tekik, Pir, Ardenn, Glacian,
//! Donatello, Silvar, Jeska.

use crate::r_p114_common::*;
use crate::r_s01_common::watch;
use crate::r_s02_common::destroy;
use crate::r_s06_common::{attach_new, attached_to, give_control};
use crate::r_s24_common::enter_together;
use crate::r_s25_common::{abilities_from, lands_for_cost};
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::facedown::can_look_at;
use mtg_engine::game::Game;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

// --- Brallin, Skyshark Rider: "Whenever you discard a card, put a +1/+1 counter on Brallin
// and it deals 1 damage to each opponent." ----------------------------------------------

#[test]
fn brallin_deals_damage_even_without_its_counter_and_one_counter_per_resolution() {
    cr!("608.2b", "608.2h", "113.7a");
    ruling!(
        "Brallin, Skyshark Rider",
        "Brallin's middle ability deals 1 damage to each opponent even if you can't put a +1/+1 counter on Brallin, most likely because Brallin has left the battlefield."
    );
    ruling!(
        "Brallin, Skyshark Rider",
        "Brallin's middle ability gives it only one +1/+1 counter each time it resolves, no matter how many opponents are dealt damage."
    );
    supported("Brallin, Skyshark Rider");
    for leaves in [false, true] {
        let mut t = TestGame::new(3);
        let brallin = t.battlefield(P0, "Brallin, Skyshark Rider");
        let bears = t.battlefield(P1, "Grizzly Bears");
        t.hand(P0, "Island");
        // Lightning Axe: "As an additional cost to cast this spell, discard a card or pay
        // {5}." One card discarded.
        t.answer_yes(P0, true);
        cast_instant(&mut t, P0, "Lightning Axe", &[ent(bears)]);
        assert_eq!(abilities_from(&t, brallin).len(), 1);
        if leaves {
            destroy(&mut t, brallin);
        }
        t.resolve_all();
        assert_eq!(t.life(P1), 19, "leaves {leaves}");
        assert_eq!(t.life(P2), 19, "leaves {leaves}");
        if !leaves {
            assert_eq!(t.counters(brallin, counters::PLUS1), 1);
        }
    }
}

#[test]
fn brallin_resolves_before_the_spell_discarded_for_or_after_the_resolving_one() {
    cr!("601.2c", "601.2h", "603.3", "608.2");
    ruling!(
        "Brallin, Skyshark Rider",
        "If you discard a card as a cost to cast a spell or activate an ability, Brallin's middle ability resolves before that spell or ability but after you've chosen targets for it. If you discard a card while a spell or ability is resolving, that spell or ability finishes resolving before Brallin's triggered ability does."
    );
    // Discarding as a cost: Lightning Axe has its target, and Brallin's trigger is above it.
    let mut t = TestGame::new(2);
    let brallin = t.battlefield(P0, "Brallin, Skyshark Rider");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.hand(P0, "Island");
    t.answer_yes(P0, true);
    let axe = cast_instant(&mut t, P0, "Lightning Axe", &[ent(bears)]);
    assert_eq!(crate::r_s25_common::targets_of(&t, axe), vec![ent(bears)]);
    assert_eq!(
        abilities_from(&t, brallin),
        vec![*t.g.stack.last().unwrap()]
    );
    t.resolve();
    assert_eq!(t.life(P1), 19);
    assert!(t.on_battlefield(bears) && t.g.stack.contains(&axe));
    // Discarding during a resolution: Faithless Looting ("Draw two cards. Then discard two
    // cards.") finishes first.
    let mut t = TestGame::new(2);
    let brallin = t.battlefield(P0, "Brallin, Skyshark Rider");
    lands_for_cost(&mut t, P0, "Faithless Looting");
    let looting = t.hand(P0, "Faithless Looting");
    t.cast(P0, looting).go();
    t.resolve();
    assert!(t.in_graveyard(P0, "Faithless Looting"));
    assert_eq!(abilities_from(&t, brallin).len(), 2);
    t.resolve_all();
    assert_eq!(t.counters(brallin, counters::PLUS1), 2);
}

#[test]
fn shabraz_gains_life_even_without_its_counter() {
    cr!("608.2b", "113.7a");
    ruling!(
        "Shabraz, the Skyshark",
        "Shabraz's middle ability has you gain 1 life even if you can't put a +1/+1 counter on Shabraz, most likely because Shabraz has left the battlefield."
    );
    supported("Shabraz, the Skyshark");
    let mut t = TestGame::new(2);
    let shabraz = t.battlefield(P0, "Shabraz, the Skyshark");
    t.g.draw_cards(P0, 1);
    t.g.flush_events();
    t.settle();
    assert_eq!(abilities_from(&t, shabraz).len(), 1);
    destroy(&mut t, shabraz);
    t.resolve_all();
    assert_eq!(t.life(P0), 21);
}

// --- Abdel Adrian, Gorion's Ward: "When Abdel Adrian enters, exile any number of other
// nonland permanents you control until Abdel Adrian leaves the battlefield. Create a 1/1
// white Soldier creature token for each permanent exiled this way." ---------------------

#[test]
fn abdel_adrian_returns_cards_to_their_owners_and_the_tokens_stay() {
    cr!("610.3c", "610.3", "111.1");
    ruling!(
        "Abdel Adrian, Gorion's Ward",
        "Cards that return to the battlefield after Abdel Adrian leaves the battlefield return under their owners' control."
    );
    ruling!(
        "Abdel Adrian, Gorion's Ward",
        "The Soldier creature tokens stay on the battlefield even after the exiled cards return."
    );
    supported("Abdel Adrian, Gorion's Ward");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.battlefield(P1, "Hill Giant");
    give_control(&mut t, giant, P0);
    t.answer_choose(P0, &[ent(bears), ent(giant)]);
    let abdel = t.enter(P0, "Abdel Adrian, Gorion's Ward");
    t.resolve_all();
    assert_eq!(t.zone(t.g.current(bears)), Zone::Exile);
    assert_eq!(t.zone(t.g.current(giant)), Zone::Exile);
    assert_eq!(tokens_named(&t, P0, "Soldier").len(), 2);
    destroy(&mut t, abdel);
    t.resolve_all();
    let giant = t.g.current(giant);
    assert!(t.on_battlefield(giant));
    assert_eq!(t.obj(giant).controller, P1);
    assert_eq!(t.obj_now(bears).controller, P0);
    assert_eq!(tokens_named(&t, P0, "Soldier").len(), 2);
}

// --- The Fourth Doctor: "You may look at the top card of your library any time." -------

fn p0_sees_top(g: &Game) -> bool {
    g.library_top(P0).is_some_and(|top| can_look_at(g, P0, top))
}

#[test]
fn the_fourth_doctor_looks_at_the_top_card_any_time() {
    cr!("401.5");
    ruling!(
        "The Fourth Doctor",
        "Essentially, The Fourth Doctor lets you play with the top card of your library revealed only to you. Knowing what that card is becomes part of the information you have access to, just like you can look at the cards in your hand. You may look at the top card of your library whenever you want, even if you don't have priority. This action doesn't use the stack."
    );
    supported("The Fourth Doctor");
    let mut t = TestGame::new(2);
    let top = t.library_top(P0, "Grizzly Bears");
    t.battlefield(P0, "The Fourth Doctor");
    t.g.recompute();
    t.set_step(P1, Step::DeclareAttackers);
    assert_eq!(t.g.turn.priority, Some(P1));
    assert!(can_look_at(&t.g, P0, top));
    assert!(!can_look_at(&t.g, P1, top));
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn the_fourth_doctor_new_top_card_hidden_while_casting() {
    cr!("401.5", "601.2i");
    ruling!(
        "The Fourth Doctor",
        "If the top card of your library changes during the process of casting a spell or activating an ability, you can't look at the new top card until the process of casting the spell or activating the ability ends (all targets are chosen, all costs are paid, and so on)."
    );
    // Stonecoil Serpent ({X} artifact creature, historic) cast from the top of the library.
    let mut t = TestGame::new(2);
    let next = t.library_top(P0, "Hill Giant");
    let serpent = t.library_top(P0, "Stonecoil Serpent");
    t.battlefield(P0, "The Fourth Doctor");
    t.lands(P0, "Wastes", 2);
    t.g.recompute();
    let seen = watch(
        &mut t,
        P0,
        |d| matches!(d, Decision::ChooseX { .. }),
        p0_sees_top,
    );
    t.answer(P0, DecisionKind::X, Answer::Number(2));
    t.cast(P0, serpent).go();
    assert_eq!(t.g.library_top(P0), Some(next));
    assert_eq!(*seen.lock().unwrap(), vec![false]);
    assert!(p0_sees_top(&t.g));
}

// --- Tevesh Szat, Doom of Fools -------------------------------------------------------------

#[test]
fn tevesh_szat_chooses_the_sacrifice_as_the_ability_resolves() {
    cr!("608.2c", "115.10");
    ruling!(
        "Tevesh Szat, Doom of Fools",
        "For the second ability of Tevesh Szat, Doom of Fools, you choose whether to sacrifice anything and what to sacrifice as the ability resolves."
    );
    supported("Tevesh Szat, Doom of Fools");
    let mut t = commander_game_n(2);
    let tevesh = t.battlefield(P0, "Tevesh Szat, Doom of Fools");
    let from = t.asked().len();
    t.activate(P0, tevesh, 1, &[]).expect("+1");
    assert!(
        !t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, Decision::ChooseEntities { .. } | Decision::YesNo { .. })),
        "nothing chosen on activation"
    );
    // A commander that entered after the activation is sacrificed: three cards.
    let gut = commander_on_battlefield(&mut t, P0, "Gut, True Soul Zealot");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[ent(gut)]);
    t.answer_yes(P0, false); // Gut stays in the graveyard.
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Gut, True Soul Zealot"));
    assert_eq!(t.hand_size(P0), hand + 3);
}

#[test]
fn tevesh_szat_commanders_leave_with_the_player() {
    cr!("800.4a", "903.9a");
    ruling!(
        "Tevesh Szat, Doom of Fools",
        "If you leave the game, each commander you gained control of is returned to its previous controller's control. Each commander that entered the battlefield under your control is exiled (from there, its owner may return it to the command zone)."
    );
    let mut t = commander_game_n(3);
    let tevesh = t.battlefield(P0, "Tevesh Szat, Doom of Fools");
    t.g.objects[tevesh.0 as usize]
        .counters
        .insert(counters::LOYALTY.into(), 10);
    let ganax = commander_on_battlefield(&mut t, P1, "Ganax, Astral Hunter");
    let gut = {
        let id = t.command(P2, "Gut, True Soul Zealot");
        make_commander(&mut t, id);
        id
    };
    t.activate(P0, tevesh, 2, &[]).expect("-10");
    t.resolve_all();
    assert_eq!(t.obj_now(ganax).controller, P0);
    let gut_now = t.g.current(gut);
    assert!(t.on_battlefield(gut_now));
    assert_eq!(t.obj(gut_now).controller, P0);
    // P0 leaves the game. P2 leaves Gut in exile.
    t.answer_yes(P2, false);
    t.g.lose_game(P0);
    t.settle();
    assert_eq!(t.obj_now(ganax).controller, P1);
    assert!(t.on_battlefield(t.g.current(ganax)));
    assert_eq!(t.zone(t.g.current(gut)), Zone::Exile);
}

// --- Blue, Loyal Raptor: "For each kind of counter on Blue, Loyal Raptor, each other
// Dinosaur you control enters with a counter of that kind on it." -----------------------

#[test]
fn blue_doesnt_add_counters_to_dinosaurs_entering_with_it() {
    cr!("614.12", "614.1c");
    ruling!(
        "Blue, Loyal Raptor",
        "If Blue, Loyal Raptor enters the battlefield with one or more kinds of counters on it at the same time as other Dinosaurs you control, those other Dinosaurs won't get additional counters from Blue, Loyal Raptor's last ability."
    );
    supported("Blue, Loyal Raptor");
    // Grumgully: "Each other non-Human creature you control enters with an additional +1/+1
    // counter on it."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grumgully, the Generous");
    let ids = enter_together(
        &mut t,
        &[(P0, "Blue, Loyal Raptor"), (P0, "Colossal Dreadmaw")],
    );
    assert_eq!(t.counters(ids[0], counters::PLUS1), 1);
    assert_eq!(t.counters(ids[1], counters::PLUS1), 1, "Grumgully's only");
    let later = t.enter(P0, "Colossal Dreadmaw");
    assert_eq!(
        t.counters(later, counters::PLUS1),
        2,
        "Grumgully's and Blue's"
    );
}

// --- Ich-Tekik, Salvage Splicer -----------------------------------------------------------

#[test]
fn ich_tekik_golems_get_counters_even_if_it_left() {
    cr!("608.2b", "603.10a");
    ruling!(
        "Ich-Tekik, Salvage Splicer",
        "If Ich-Tekik leaves the battlefield before its middle ability resolves, you'll still put a +1/+1 counter on each Golem you control."
    );
    let mut t = TestGame::new(2);
    let ich = t.enter(P0, "Ich-Tekik, Salvage Splicer");
    t.resolve_all();
    let golems = tokens_named(&t, P0, "Golem");
    assert_eq!(golems.len(), 1);
    let stone = t.battlefield(P0, "Mind Stone");
    destroy(&mut t, stone);
    assert_eq!(abilities_from(&t, ich).len(), 1);
    destroy(&mut t, ich);
    t.resolve_all();
    assert_eq!(t.counters(golems[0], counters::PLUS1), 1);
}

// --- Pir, Imaginative Rascal: "If one or more counters would be put on a permanent your team
// controls, that many plus one of each of those kinds of counters are put on that permanent
// instead." --------------------------------------------------------------------------------

#[test]
fn pir_adds_one_to_counters_entering_and_on_teammates_permanents() {
    cr!("614.1c", "122.6", "810.2");
    ruling!(
        "Pir, Imaginative Rascal",
        "If a permanent your team controls would enter the battlefield with one or more counters of any kind on it, it enters with that many plus one instead."
    );
    supported("Pir, Imaginative Rascal");
    let mut t = two_headed_giant();
    t.battlefield(P0, "Pir, Imaginative Rascal");
    // Walking Ballista for X=2 enters with three.
    t.lands(P0, "Wastes", 4);
    let ballista = t.hand(P0, "Walking Ballista");
    t.cast(P0, ballista).x(2).go();
    t.resolve_all();
    let ballista = t.g.current(ballista);
    assert_eq!(t.counters(ballista, counters::PLUS1), 3);
    // A counter put on P0's teammate's creature: two.
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.g.add_counters(ent(bears), counters::PLUS1, 1, None);
    t.g.flush_events();
    assert_eq!(t.counters(bears, counters::PLUS1), 2);
    // An opponent's isn't affected.
    let theirs = t.battlefield(P2, "Grizzly Bears");
    t.g.add_counters(ent(theirs), counters::PLUS1, 1, None);
    assert_eq!(t.counters(theirs, counters::PLUS1), 1);
}

#[test]
fn pir_applies_to_each_instruction() {
    cr!("614.1c", "608.2c");
    ruling!(
        "Pir, Imaginative Rascal",
        "If an effect includes multiple instructions to put one or more counters on a permanent, such as Lifecrafter's Gift does, Pir's effect applies to each of those instructions."
    );
    // Lifecrafter's Gift: "Put a +1/+1 counter on target creature, then put a +1/+1 counter
    // on each creature you control with a +1/+1 counter on it."
    supported("Lifecrafter's Gift");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Pir, Imaginative Rascal");
    let bears = t.battlefield(P0, "Grizzly Bears");
    cast_instant(&mut t, P0, "Lifecrafter's Gift", &[ent(bears)]);
    t.resolve_all();
    assert_eq!(t.counters(bears, counters::PLUS1), 4);
}

#[test]
fn pir_doesnt_apply_to_permanents_entering_with_it() {
    cr!("614.12", "614.1c");
    ruling!(
        "Pir, Imaginative Rascal",
        "Pir's effect can't apply to itself as it's entering the battlefield or to any other permanent entering the battlefield at the same time as it."
    );
    // Renata, Called to the Hunt: "Each other creature you control enters with an
    // additional +1/+1 counter on it."
    supported("Renata, Called to the Hunt");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Renata, Called to the Hunt");
    let ids = enter_together(
        &mut t,
        &[(P0, "Pir, Imaginative Rascal"), (P0, "Grizzly Bears")],
    );
    assert_eq!(t.counters(ids[0], counters::PLUS1), 1, "Pir itself");
    assert_eq!(t.counters(ids[1], counters::PLUS1), 1, "entering with Pir");
    let later = t.enter(P0, "Grizzly Bears");
    assert_eq!(t.counters(later, counters::PLUS1), 2);
}

// --- Ardenn, Intrepid Archaeologist: "At the beginning of combat on your turn, you may attach
// any number of Auras and Equipment you control to target permanent or player." -----------

#[test]
fn ardenn_can_attach_to_an_opponents_creature_but_not_illegally() {
    cr!("301.5", "303.4", "701.3b");
    ruling!(
        "Ardenn, Intrepid Archaeologist",
        "Other players and permanents controlled by other players can be targeted by Ardenn's ability. This can cause Equipment under your control to be attached to a creature an opponent controls. Auras and Equipment that say \"you\" still refer to you, as you still control them. Ardenn's ability can't cause an Aura or Equipment to be attached to something it couldn't be attached to."
    );
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ardenn, Intrepid Archaeologist");
    let mine = t.battlefield(P0, "Hill Giant");
    let splitter = attach_new(&mut t, P0, "Bonesplitter", ent(mine));
    // Spirit Link: "Whenever enchanted creature deals damage, you gain that much life."
    let link = attach_new(&mut t, P0, "Spirit Link", ent(mine));
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[ent(bears)]);
    t.answer_choose(P0, &[ent(splitter), ent(link)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), Some(ent(bears)));
    assert_eq!(attached_to(&t, link), Some(ent(bears)));
    assert_eq!(t.obj(splitter).controller, P0);
    assert_eq!(t.pt(bears), (4, 2));
    deal(&mut t, bears, 4, Entity::Player(P0));
    t.resolve_all();
    assert_eq!(t.life(P0), 20, "P0 lost 4 and gained 4 from Spirit Link");
    // Target a player: neither can be attached to a player.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Ardenn, Intrepid Archaeologist");
    let mine = t.battlefield(P0, "Hill Giant");
    let splitter = attach_new(&mut t, P0, "Bonesplitter", ent(mine));
    let link = attach_new(&mut t, P0, "Spirit Link", ent(mine));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_choose(P0, &[ent(splitter), ent(link)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), Some(ent(mine)));
    assert_eq!(attached_to(&t, link), Some(ent(mine)));
}

// --- Glacian, Powerstone Engineer -----------------------------------------------------------

#[test]
fn glacian_tapping_equipment_doesnt_unequip() {
    cr!("301.5", "701.26a");
    ruling!(
        "Glacian, Powerstone Engineer",
        "Tapping an Equipment doesn't cause it to stop affecting the creature it equips."
    );
    supported("Glacian, Powerstone Engineer");
    let mut t = TestGame::new(2);
    let glacian = t.battlefield(P0, "Glacian, Powerstone Engineer");
    let splitter = attach_new(&mut t, P0, "Bonesplitter", ent(glacian));
    assert_eq!(t.pt(glacian), (5, 6));
    t.answer(P0, DecisionKind::X, Answer::Number(1));
    t.answer_choose(P0, &[ent(splitter)]);
    t.activate(P0, glacian, 0, &[]).expect("Glacian");
    assert!(t.obj(splitter).tapped);
    t.resolve_all();
    assert_eq!(attached_to(&t, splitter), Some(ent(glacian)));
    assert_eq!(t.pt(glacian), (5, 6));
}

// --- Donatello, the Brains ------------------------------------------------------------------

#[test]
fn donatello_mutagen_shares_the_creation_instructions_but_not_abilities() {
    cr!("614.1a", "111.1");
    ruling!(
        "Donatello, the Brains",
        "The additional Mutagen token won't have any abilities the other tokens were created with. Anything else specified in the effect creating the token (such as tapped or \"Exile that token at end of combat\") applies to both the original tokens and the Mutagen token."
    );
    supported("Donatello, the Brains");
    // Geist of Saint Traft: "Whenever this creature attacks, create a 4/4 white Angel
    // creature token with flying that's tapped and attacking. Exile that token at end of
    // combat."
    supported("Geist of Saint Traft");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Donatello, the Brains");
    let geist = t.battlefield(P0, "Geist of Saint Traft");
    attack_with(&mut t, &[(geist, Entity::Player(P1))]);
    t.resolve_all();
    let mutagen = tokens_named(&t, P0, "Mutagen");
    assert_eq!(mutagen.len(), 1);
    assert!(t.obj(mutagen[0]).tapped);
    assert!(!t
        .obj(mutagen[0])
        .has_keyword(mtg_engine::keywords::KeywordKind::Flying));
    assert_eq!(tokens_named(&t, P0, "Angel").len(), 1);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    t.advance_to(P0, Step::EndOfCombat);
    t.resolve_all();
    t.advance_to(P0, Step::PostcombatMain);
    assert!(tokens_named(&t, P0, "Angel").is_empty());
    assert!(tokens_named(&t, P0, "Mutagen").is_empty());
}

// --- Silvar, Devourer of the Free -----------------------------------------------------------

#[test]
fn silvar_activates_while_already_indestructible() {
    cr!("602.2", "702.12a");
    ruling!(
        "Silvar, Devourer of the Free",
        "You can activate Silvar's last ability even if it already has indestructible. It'll still get a +1/+1 counter."
    );
    supported("Silvar, Devourer of the Free");
    let mut t = TestGame::new(2);
    let silvar = t.battlefield(P0, "Silvar, Devourer of the Free");
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    let human = |t: &mut TestGame, id| {
        crate::r_s26_common::modify_until_eot(
            t,
            id,
            vec![mtg_engine::ability::Modification::AddSubtypes(vec![
                "Human".into(),
            ])],
        )
    };
    human(&mut t, a);
    human(&mut t, b);
    t.answer_choose(P0, &[ent(a)]);
    t.activate(P0, silvar, 0, &[]).expect("first");
    t.resolve_all();
    assert!(t
        .obj(silvar)
        .has_keyword(mtg_engine::keywords::KeywordKind::Indestructible));
    t.answer_choose(P0, &[ent(b)]);
    t.activate(P0, silvar, 0, &[]).expect("again");
    t.resolve_all();
    assert_eq!(t.counters(silvar, counters::PLUS1), 2);
}

// --- Jeska, Thrice Reborn -------------------------------------------------------------------

#[test]
fn jeska_cast_as_a_commander_counts_herself() {
    cr!("903.8", "614.1c");
    ruling!(
        "Jeska, Thrice Reborn",
        "If you cast Jeska, Thrice Reborn and she is your commander, that casting counts for her first ability."
    );
    let mut t = commander_game_n(2);
    let jeska = t.command(P0, "Jeska, Thrice Reborn");
    make_commander(&mut t, jeska);
    lands_for_cost(&mut t, P0, "Jeska, Thrice Reborn");
    t.cast(P0, jeska).go();
    t.resolve_all();
    let jeska = t.g.current(jeska);
    assert!(t.on_battlefield(jeska));
    assert_eq!(t.counters(jeska, counters::LOYALTY), 1);
}
