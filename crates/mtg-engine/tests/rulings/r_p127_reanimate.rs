//! Rulings batch P127 — putting cards from any graveyard (and lands and other permanent
//! cards) onto the battlefield: non-targeted choices made on resolution, Coffin Queen's
//! tracking delayed trigger, replacement effects on milled cards, leaving a multiplayer
//! game, Auras that can't be attached, simultaneous entry and sacrifice, and replacement
//! effects granted to returned permanents.

use crate::r_p127_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::types::*;
use mtg_engine::*;

#[test]
fn extract_from_darkness_chooses_on_resolution_including_a_just_milled_card() {
    cr!("608.2c", "701.17a");
    ruling!(
        "Extract from Darkness",
        "You can choose any creature card in a graveyard at that time, including one just put into a graveyard by Extract from Darkness."
    );
    supported("Extract from Darkness");
    let mut t = TestGame::new(2);
    t.graveyard(P0, "Hill Giant");
    let bears = t.library_top(P1, "Grizzly Bears");
    t.answer_choose_moved(P0, &[obj(bears)]);
    cast_new(&mut t, P0, "Extract from Darkness", &[]);
    t.resolve_all();
    assert_eq!(named_by(&t, P0, "Grizzly Bears").len(), 1);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    // No creature cards anywhere: it simply finishes resolving (both players milled).
    let mut t = TestGame::new(2);
    let spell = cast_new(&mut t, P0, "Extract from Darkness", &[]);
    t.resolve_all();
    assert!(t.in_graveyard(P0, "Extract from Darkness"));
    assert!(resolved(&t, spell));
    assert_eq!(t.library_size(P1), 28);
    assert!(creatures(&t, P0).is_empty());
}

/// Coffin Queen with mana for its ability; returns (queen, the creature card in P1's
/// graveyard).
fn coffin_queen() -> (TestGame, ObjectId, ObjectId) {
    supported("Coffin Queen");
    let mut t = TestGame::new(2);
    let queen = t.battlefield(P0, "Coffin Queen");
    t.lands(P0, "Swamp", 3);
    let bears = t.graveyard(P1, "Grizzly Bears");
    (t, queen, bears)
}

fn activate_queen(t: &mut TestGame, queen: ObjectId, target: ObjectId) {
    t.answer_targets(P0, &[obj(target)]);
    activate_containing(t, P0, queen, "Put target creature card").unwrap();
}

#[test]
fn coffin_queen_leaving_or_untapping_before_resolution() {
    cr!("603.7c", "603.7a");
    ruling!(
        "Coffin Queen",
        "If Coffin Queen leaves the battlefield before its ability resolves, the delayed trigger condition can never be fulfilled, so the creature will not be exiled. If Coffin Queen is untapped when its ability resolves, the creature will not be exiled until the next time Coffin Queen becomes untapped."
    );
    // Untapped in response: the creature stays until the Queen next becomes untapped.
    let (mut t, queen, bears) = coffin_queen();
    activate_queen(&mut t, queen, bears);
    untap(&mut t, queen);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    tap(&mut t, queen);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    untap(&mut t, queen);
    t.resolve_all();
    assert!(is_in(&t, bears, Zone::Exile));
    // The Queen leaves before the ability resolves: the creature is never exiled.
    let (mut t, queen, bears) = coffin_queen();
    activate_queen(&mut t, queen, bears);
    destroy(&mut t, queen);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    let queen2 = t.battlefield(P0, "Coffin Queen");
    tap(&mut t, queen2);
    untap(&mut t, queen2);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}

#[test]
fn coffin_queen_stops_tracking_a_creature_that_left() {
    cr!("400.7", "603.7c");
    ruling!(
        "Coffin Queen",
        "If the creature put onto the battlefield leaves the battlefield, the tracking effect ends and the creature will no longer be exiled if something happens to the Coffin Queen."
    );
    let (mut t, queen, bears) = coffin_queen();
    activate_queen(&mut t, queen, bears);
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    // It's bounced and replayed: a new object the Queen doesn't track.
    bounce(&mut t, bears);
    let bears = t.g.current(bears);
    let again = move_to(&mut t, bears, Zone::Battlefield).unwrap();
    untap(&mut t, queen);
    t.resolve_all();
    assert!(t.on_battlefield(again));
}

#[test]
fn coffin_queen_tracking_survives_phasing() {
    cr!("702.26d", "603.7c");
    ruling!(
        "Coffin Queen",
        "If the creature put onto the battlefield phases out, the tracking effect continues when it phases back in if the Coffin Queen is still on the battlefield. If the Queen left the battlefield while the creature was phased out, the creature phases in as normal."
    );
    // Phased out and back in: still tracked.
    let (mut t, queen, bears) = coffin_queen();
    activate_queen(&mut t, queen, bears);
    t.resolve_all();
    let b = t.g.current(bears);
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![b]);
    mtg_engine::keyword_impls::phase_in(&mut t.g, b);
    t.g.flush_events();
    t.settle();
    untap(&mut t, queen);
    t.resolve_all();
    assert!(is_in(&t, bears, Zone::Exile));
    // The Queen leaves while it's phased out: it phases in as normal and stays.
    let (mut t, queen, bears) = coffin_queen();
    activate_queen(&mut t, queen, bears);
    t.resolve_all();
    let b = t.g.current(bears);
    mtg_engine::keyword_impls::phase_out(&mut t.g, vec![b]);
    destroy(&mut t, queen);
    t.resolve_all();
    mtg_engine::keyword_impls::phase_in(&mut t.g, b);
    t.g.flush_events();
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert!(!t.obj_now(bears).phased_out);
}

#[test]
fn gyruda_can_choose_a_card_exiled_instead_of_milled() {
    cr!("614.1a", "608.2c");
    ruling!(
        "Gyruda, Doom of Depths",
        "If a replacement effect causes a player to exile the top four cards of their library instead of putting them into their graveyard as Gyruda's triggered ability resolves, the creature card you choose may be one of those cards in exile."
    );
    supported("Gyruda, Doom of Depths");
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Rest in Peace");
    let bears = t.library_top(P1, "Grizzly Bears");
    t.answer_choose_moved(P0, &[obj(bears)]);
    t.enter(P0, "Gyruda, Doom of Depths");
    t.resolve_all();
    assert_eq!(named_by(&t, P0, "Grizzly Bears").len(), 1);
}

#[test]
fn scythe_of_the_wretched_doesnt_move_onto_a_noncreature() {
    cr!("301.5c", "608.2b");
    ruling!(
        "Scythe of the Wretched",
        "If the card isn't a creature when it comes back onto the battlefield, then Scythe of the Wretched won't move onto it."
    );
    supported("Scythe of the Wretched");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let scythe = attach_new(&mut t, P0, "Scythe of the Wretched", bears);
    // An animated Mutavault (a land card) is dealt damage by the equipped creature and dies.
    let vault = t.battlefield(P1, "Mutavault");
    t.lands(P1, "Wastes", 1);
    t.g.turn.priority = Some(P1);
    activate_containing(&mut t, P1, vault, "becomes").unwrap();
    t.resolve_all();
    assert!(is_creature(&t, vault));
    damage(&mut t, bears, 2, vault);
    t.resolve_all();
    let back = named_by(&t, P0, "Mutavault");
    assert_eq!(back.len(), 1);
    assert!(!is_creature(&t, back[0]));
    assert_eq!(attached_to(&t, scythe), Some(obj(bears)));
}

#[test]
fn iridescent_drake_aura_that_cant_attach_stays_in_the_graveyard() {
    cr!("303.4a", "303.4f");
    ruling!(
        "Iridescent Drake",
        "However, if the Aura can’t be attached to Iridescent Drake, it remains in its owner’s graveyard and doesn’t enter the battlefield."
    );
    supported("Iridescent Drake");
    // An Aura from an opponent's graveyard that can enchant it is attached.
    let mut t = TestGame::new(2);
    let rancor = t.graveyard(P1, "Rancor");
    t.answer_targets(P0, &[obj(rancor)]);
    let drake = t.enter(P0, "Iridescent Drake");
    t.resolve_all();
    assert_eq!(attached_to(&t, rancor), Some(obj(drake)));
    assert_eq!(t.obj_now(rancor).controller, P0);
    // "Enchant land" can't be attached to it: it stays in the graveyard.
    let mut t = TestGame::new(2);
    let growth = t.graveyard(P0, "Wild Growth");
    t.answer_targets(P0, &[obj(growth)]);
    t.enter(P0, "Iridescent Drake");
    t.settle();
    assert_eq!(t.stack_len(), 1, "the trigger targets Wild Growth");
    t.resolve_all();
    assert!(is_in(&t, growth, Zone::Graveyard(P0)));
}

#[test]
fn leaving_the_game_exiles_reanimated_cards_of_other_owners() {
    cr!("800.4a");
    ruling!(
        "Iridescent Drake",
        "If you leave the game, the Aura you control from Iridescent Drake’s ability is exiled."
    );
    ruling!(
        "Boneyard Parley",
        "If you leave the game, any creatures you control from Boneyard Parley’s effect are exiled."
    );
    supported("Boneyard Parley");
    // Iridescent Drake: P1's Rancor, controlled by P0, is exiled as P0 leaves.
    let mut t = TestGame::new(3);
    let rancor = t.graveyard(P1, "Rancor");
    t.answer_targets(P0, &[obj(rancor)]);
    t.enter(P0, "Iridescent Drake");
    t.resolve_all();
    assert!(t.on_battlefield(rancor));
    concede(&mut t, P0);
    assert!(is_in(&t, rancor, Zone::Exile));
    // Boneyard Parley: P1's and P2's creature cards put onto the battlefield under P0's
    // control (P0 picks whichever pile isn't empty).
    for pile in [0usize, 1] {
        let mut t = TestGame::new(3);
        let bears = t.graveyard(P1, "Grizzly Bears");
        let giant = t.graveyard(P2, "Hill Giant");
        t.answer(P0, DecisionKind::Option, Answer::Index(pile));
        cast_new(&mut t, P0, "Boneyard Parley", &[obj(bears), obj(giant)]);
        t.resolve_all();
        let mine: Vec<ObjectId> = [bears, giant]
            .into_iter()
            .filter(|c| t.on_battlefield(*c))
            .collect();
        if mine.is_empty() {
            continue;
        }
        for c in &mine {
            assert_eq!(t.obj_now(*c).controller, P0);
        }
        concede(&mut t, P0);
        for c in mine {
            assert!(is_in(&t, c, Zone::Exile));
        }
        return;
    }
    panic!("no pile was put onto the battlefield");
}

#[test]
fn soul_collector_triggers_only_face_up() {
    cr!("708.2", "603.10a");
    ruling!(
        "Soul Collector",
        "The ability only triggers if this card is face up at the time the creature is put into the graveyard."
    );
    supported("Soul Collector");
    // Face down: no trigger.
    let mut t = TestGame::new(2);
    let fd = crate::r_s12_common::morph(&mut t, P0, "Soul Collector");
    let bears = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, fd, 2, bears);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    // Face up: it returns the creature under P0's control.
    let mut t = TestGame::new(2);
    let sc = t.battlefield(P0, "Soul Collector");
    let bears = t.battlefield(P1, "Grizzly Bears");
    damage(&mut t, sc, 2, bears);
    t.resolve_all();
    assert_eq!(named_by(&t, P0, "Grizzly Bears").len(), 1);
}

#[test]
fn teneb_targets_on_the_stack_and_pays_on_resolution() {
    cr!("603.3d", "603.5", "608.2b");
    ruling!(
        "Teneb, the Harvester",
        "You choose the target when the ability is put on the stack, and you choose whether to pay when it resolves."
    );
    supported("Teneb, the Harvester");
    let mut t = TestGame::new(2);
    let teneb = t.battlefield(P0, "Teneb, the Harvester");
    t.lands(P0, "Swamp", 3);
    let bears = t.graveyard(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    attack_with(&mut t, &[(teneb, Entity::Player(P1))]);
    let ap = P0;
    let ok = t.g.run_until(10_000, |g| {
        g.turn.step == mtg_engine::turn::Step::CombatDamage && !g.stack.is_empty()
    });
    assert!(ok && t.g.turn.active == ap);
    t.settle();
    // On the stack with its target; the payment hasn't been asked yet.
    assert_eq!(triggers_on_stack(&t, "you may pay"), 1);
    let from = t.asked().len();
    assert_eq!(yes_no_asked(&t, P0, 0), 0);
    t.answer_yes(P0, true);
    t.resolve_all();
    assert_eq!(yes_no_asked(&t, P0, from), 1);
    assert_eq!(named_by(&t, P0, "Grizzly Bears").len(), 1);
}

#[test]
fn sepulchral_primordial_creatures_enter_simultaneously() {
    cr!("603.6a", "608.2c");
    ruling!("Sepulchral Primordial", "The creature cards enter simultaneously.");
    supported("Sepulchral Primordial");
    // Two Soul Wardens entering at once each see the other enter.
    let mut t = TestGame::new(3);
    let w1 = t.graveyard(P1, "Soul Warden");
    let w2 = t.graveyard(P2, "Soul Warden");
    t.answer_targets(P0, &[obj(w1)]);
    t.answer_targets(P0, &[obj(w2)]);
    t.enter(P0, "Sepulchral Primordial");
    t.resolve_all();
    assert_eq!(named_by(&t, P0, "Soul Warden").len(), 2);
    assert_eq!(t.life(P0), 22);
}

#[test]
fn altanak_trigger_resolves_first_even_if_the_spell_is_countered() {
    cr!("603.3", "405.5");
    ruling!(
        "Altanak, the Thrice-Called",
        "It resolves even if that spell or ability is countered or otherwise leaves the stack without resolving."
    );
    supported("Altanak, the Thrice-Called");
    let mut t = TestGame::new(2);
    let altanak = t.battlefield(P0, "Altanak, the Thrice-Called");
    let shock = cast_new(&mut t, P1, "Shock", &[obj(altanak)]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Counterspell", &[obj(shock)]);
    t.resolve_all();
    assert!(t.in_graveyard(P1, "Shock"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn altanak_triggers_once_for_a_spell_targeting_it_twice() {
    cr!("603.2", "115.3");
    ruling!(
        "Altanak, the Thrice-Called",
        "If a spell or ability targets Altanak more than once, its second ability still triggers only once."
    );
    let mut t = TestGame::new(2);
    let altanak = t.battlefield(P0, "Altanak, the Thrice-Called");
    cast_new(
        &mut t,
        P1,
        "Seeds of Strength",
        &[obj(altanak), obj(altanak), obj(altanak)],
    );
    t.settle();
    assert_eq!(triggers_on_stack(&t, "draw a card"), 1);
    t.resolve_all();
    assert_eq!(t.pt(altanak), (12, 12));
}

#[test]
fn quarry_beetle_return_isnt_playing_a_land() {
    cr!("305.4", "305.2");
    ruling!(
        "Quarry Beetle",
        "It can return a land card to the battlefield even if you’ve already played as many lands as able this turn."
    );
    supported("Quarry Beetle");
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    t.play_land(P0, forest).unwrap();
    let swamp = t.graveyard(P0, "Swamp");
    t.answer_targets(P0, &[obj(swamp)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Quarry Beetle");
    t.resolve_all();
    assert!(t.on_battlefield(swamp));
    assert_eq!(t.g.player(P0).lands_played_this_turn, 1);
}

#[test]
fn vengeful_regrowth_makes_tokens_for_lands_actually_returned() {
    cr!("608.2b", "608.2c");
    ruling!(
        "Vengeful Regrowth",
        "you’ll return the remaining Forest card and create one Plant Warrior token."
    );
    supported("Vengeful Regrowth");
    let mut t = TestGame::new(2);
    let f: Vec<ObjectId> = (0..3).map(|_| t.graveyard(P0, "Forest")).collect();
    let es: Vec<Entity> = f.iter().map(|x| obj(*x)).collect();
    lands_for_cost(&mut t, P0, "Vengeful Regrowth");
    let card = t.hand(P0, "Vengeful Regrowth");
    t.cast(P0, card).targets(&es).go();
    exile_now(&mut t, f[0]);
    exile_now(&mut t, f[1]);
    t.resolve_all();
    assert!(t.on_battlefield(f[2]));
    assert_eq!(crate::r_p108_common::tokens_with(&t, P0, "Plant"), 1);
}

#[test]
fn obzedats_aid_targets_any_permanent_card() {
    cr!("110.4", "115.1");
    ruling!(
        "Obzedat's Aid",
        "A permanent card is an artifact card, a creature card, an enchantment card, a land card, or a planeswalker card."
    );
    supported("Obzedat's Aid");
    for (name, ok) in [
        ("Ornithopter", true),
        ("Grizzly Bears", true),
        ("Glorious Anthem", true),
        ("Forest", true),
        ("Ajani Goldmane", true),
        ("Shock", false),
        ("Divination", false),
    ] {
        let mut t = TestGame::new(2);
        let c = t.graveyard(P0, name);
        lands_for_cost(&mut t, P0, "Obzedat's Aid");
        let aid = t.hand(P0, "Obzedat's Aid");
        let r = t.cast(P0, aid).target(c).try_go();
        if ok {
            assert!(r.is_ok(), "{name} should be a legal target");
            t.resolve_all();
            assert!(t.on_battlefield(c), "{name}");
        } else {
            assert!(
                r.is_err() || !t.g.stack.iter().any(|s| crate::r_s25_common::targets_of(&t, *s).contains(&obj(c))),
                "{name} shouldn't be a legal target"
            );
        }
    }
}

#[test]
fn obzedats_aid_aura_chooses_on_resolution_or_stays() {
    cr!("303.4a", "303.4f");
    ruling!(
        "Obzedat's Aid",
        "If the target is an Aura card, you choose what it will enchant as Obzedat’s Aid resolves. If there’s nothing legal for it to enchant, it stays in the graveyard."
    );
    // Nothing to enchant: it stays.
    let mut t = TestGame::new(2);
    let rancor = t.graveyard(P0, "Rancor");
    cast_new(&mut t, P0, "Obzedat's Aid", &[obj(rancor)]);
    t.resolve_all();
    assert!(is_in(&t, rancor, Zone::Graveyard(P0)));
    // A creature to enchant, chosen as it resolves.
    let mut t = TestGame::new(2);
    let rancor = t.graveyard(P0, "Rancor");
    cast_new(&mut t, P0, "Obzedat's Aid", &[obj(rancor)]);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve_all();
    assert_eq!(attached_to(&t, rancor), Some(obj(bears)));
}

/// Spirit-Sister's Call returns a Grizzly Bears (sacrificing a Hill Giant) at P0's end step.
fn spirit_sisters_call() -> (TestGame, ObjectId, ObjectId) {
    supported("Spirit-Sister's Call");
    let mut t = TestGame::new(2);
    let call = t.battlefield(P0, "Spirit-Sister's Call");
    let bears = t.graveyard(P0, "Grizzly Bears");
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer_targets(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    end_step(&mut t, P0);
    assert!(t.on_battlefield(bears));
    assert!(t.in_graveyard(P0, "Hill Giant"));
    (t, call, bears)
}

#[test]
fn spirit_sisters_call_exiled_creature_doesnt_die() {
    cr!("700.4", "614.1a");
    ruling!(
        "Spirit-Sister's Call",
        "Because to die means to be put into a graveyard from the battlefield, a creature or planeswalker that is exiled instead doesn't \"die.\""
    );
    let (mut t, _call, bears) = spirit_sisters_call();
    t.battlefield(P0, "Blood Artist");
    destroy(&mut t, bears);
    assert!(is_in(&t, bears, Zone::Exile));
    assert_eq!(triggers_on_stack(&t, "loses 1 life"), 0);
}

#[test]
fn spirit_sisters_call_replacement_outlasts_the_enchantment() {
    cr!("613.1f", "614.1a");
    ruling!(
        "Spirit-Sister's Call",
        "If Spirit-Sister's Call leaves the battlefield, the replacement effect continues to apply."
    );
    let (mut t, call, bears) = spirit_sisters_call();
    destroy(&mut t, call);
    bounce(&mut t, bears);
    assert!(is_in(&t, bears, Zone::Exile));
}

#[test]
fn rise_of_the_witch_king_sacrifices_in_apnap_order_simultaneously() {
    cr!("101.4", "701.21a");
    ruling!(
        "Rise of the Witch-king",
        "First the active player chooses which creature they'll sacrifice, then each other player in turn order does the same, knowing choices made before their choice. Then all those creatures are sacrificed simultaneously."
    );
    supported("Rise of the Witch-king");
    let mut t = TestGame::new(2);
    let artist = t.battlefield(P0, "Blood Artist");
    t.battlefield(P0, "Hill Giant");
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Raging Goblin");
    t.answer_choose(P0, &[obj(artist)]);
    t.answer_choose(P1, &[obj(bears)]);
    t.answer_yes(P0, false);
    let from = t.asked().len();
    cast_new(&mut t, P0, "Rise of the Witch-king", &[]);
    t.resolve(); // the spell
    // The choices: P0 first, then P1.
    let order: Vec<PlayerId> = t.asked()[from..]
        .iter()
        .filter(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseEntities { .. }))
        .map(|(p, _)| *p)
        .collect();
    assert_eq!(&order[..2], &[P0, P1]);
    assert!(t.in_graveyard(P0, "Blood Artist") && t.in_graveyard(P1, "Grizzly Bears"));
    // Blood Artist saw the Bears die alongside it.
    assert_eq!(triggers_on_stack(&t, "loses 1 life"), 2);
}

#[test]
fn rise_of_the_witch_king_chooses_the_card_after_sacrificing() {
    cr!("608.2c", "701.21a");
    ruling!(
        "Rise of the Witch-king",
        "You choose which permanent card you're returning to the battlefield, if any, as Rise of the Witch-King resolves, after permanents are sacrificed."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let giant = t.graveyard(P0, "Hill Giant");
    t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[obj(bears)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(giant)]);
    // When P0 is asked about returning a card, the creatures have been sacrificed.
    let seen = crate::r_s01_common::watch(
        &mut t,
        P0,
        |d| matches!(d, mtg_engine::decision::Decision::YesNo { .. }),
        |g| g.permanents().filter(|o| o.is(CardType::Creature)).count(),
    );
    cast_new(&mut t, P0, "Rise of the Witch-king", &[]);
    t.resolve_all();
    assert_eq!(*seen.lock().unwrap(), vec![0]);
    assert!(t.on_battlefield(giant));
    assert!(t.in_graveyard(P0, "Grizzly Bears"));
}

#[test]
fn rooting_moloch_card_can_be_played_but_not_cycled() {
    cr!("305.1", "702.29a");
    ruling!(
        "Rooting Moloch",
        "Rooting Moloch gives you permission to play the card as a land or cast it as a spell. You can’t cycle the exiled card."
    );
    supported("Rooting Moloch");
    let mut t = TestGame::new(2);
    let sandbar = t.graveyard(P0, "Lonely Sandbar");
    t.lands(P0, "Island", 3);
    t.answer_targets(P0, &[obj(sandbar)]);
    t.enter(P0, "Rooting Moloch");
    t.resolve_all();
    assert!(is_in(&t, sandbar, Zone::Exile));
    assert!(!can_activate(&mut t, P0, sandbar));
    assert!(can_play_land(&mut t, P0, sandbar));
    let s = t.g.current(sandbar);
    t.play_land(P0, s).unwrap();
    assert!(t.on_battlefield(sandbar));
}
