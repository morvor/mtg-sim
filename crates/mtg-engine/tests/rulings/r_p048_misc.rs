//! Rulings batch P048 — the other cards that give tokens or permanents to other players:
//! Fractured Identity (token copies, CR 707.2, 111.5, 903.3), Declaration in Stone (names,
//! CR 201.2, 708.2), Oversimplify (last known power, CR 608.2h), Dark Salvation, Baffling
//! End, Awaken the Erstwhile, Strix Serenade and Swan Song (can't be countered, CR 101.2),
//! Unyielding Gatekeeper; "doesn't untap" and {Q} (Alirios, Farmstead Gleaner); and target
//! legality on resolution (Molten Frame, Petty Revenge, Go for the Throat).

use crate::r_p048_common::*;
use crate::r_s02_common::create_token;
use crate::r_s25_common::name_now;
use mtg_engine::ability::{Modification, Value};
use mtg_engine::decision::Answer;
use mtg_engine::keywords::{Keyword, KeywordKind};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// P0 casts Fractured Identity on `target` (P1's) in a three-player game; returns the
/// tokens P0 and P2 create.
fn fracture(t: &mut TestGame, target: ObjectId) -> Vec<ObjectId> {
    supported("Fractured Identity");
    let before: Vec<ObjectId> = t.g.battlefield.clone();
    cast_new(t, P0, "Fractured Identity", &[obj(target)]);
    t.resolve_all();
    assert!(!t.on_battlefield(target));
    let new: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.is_token() && !before.contains(&o.id))
            .map(|o| o.id)
            .collect();
    let owners: Vec<PlayerId> = new.iter().map(|o| t.obj(*o).controller).collect();
    assert_eq!(owners, vec![P0, P2], "one token each for P0 and P2");
    new
}

#[test]
fn fractured_identitys_tokens_have_enters_abilities_and_replacements() {
    cr!("707.2", "603.6a", "614.1c");
    ruling!(
        "Fractured Identity",
        "Any enters-the-battlefield abilities of the copied permanent will trigger when the tokens enter the battlefield."
    );
    // Elvish Visionary: "When this creature enters, draw a card."
    let mut t = TestGame::new(3);
    let v = t.battlefield(P1, "Elvish Visionary");
    let hands = (t.hand_size(P0), t.hand_size(P2));
    fracture(&mut t, v);
    assert_eq!(
        (t.hand_size(P0), t.hand_size(P2)),
        (hands.0 + 1, hands.1 + 1)
    );
    // Spike Feeder: "This creature enters with two +1/+1 counters on it."
    supported("Spike Feeder");
    let mut t = TestGame::new(3);
    let f = t.enter(P1, "Spike Feeder");
    t.resolve_all();
    assert_eq!(t.counters(f, "+1/+1"), 2);
    for tok in fracture(&mut t, f) {
        assert_eq!(t.counters(tok, "+1/+1"), 2);
    }
}

#[test]
fn fractured_identity_copies_only_the_copiable_values() {
    cr!("707.2", "707.3", "111.5", "903.3");
    ruling!(
        "Fractured Identity",
        "The tokens copy exactly what was printed on the permanent and nothing else"
    );
    ruling!(
        "Fractured Identity",
        "If the copied permanent is a token, the tokens created by Fractured Identity copy the original characteristics of that token as stated by the effect that put it onto the battlefield."
    );
    ruling!(
        "Fractured Identity",
        "If the copied permanent was copying something else, the tokens enter the battlefield as whatever that permanent was copying."
    );
    ruling!(
        "Fractured Identity",
        "If the exiled permanent was a commander, the tokens won't be commanders."
    );
    // Tapped Grizzly Bears with a +1/+1 counter, Holy Strength, and +3/+3 until end of
    // turn: the tokens are plain untapped 2/2 Grizzly Bears.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 1);
    attach_new(&mut t, P1, "Holy Strength", obj(bears));
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::ModifyPT(Value::c(3), Value::c(3))],
    );
    tap(&mut t, bears);
    for tok in fracture(&mut t, bears) {
        assert_eq!(name_now(&t, tok), "Grizzly Bears");
        assert_eq!(t.pt(tok), (2, 2));
        assert!(!t.obj(tok).tapped);
        assert_eq!(t.counters(tok, "+1/+1"), 0);
    }
    // A 1/1 Zombie token, pumped: the tokens are 1/1 Zombies.
    let mut t = TestGame::new(3);
    let z = create_token(&mut t, P1, "Zombie");
    modify_until_eot(
        &mut t,
        z,
        vec![Modification::ModifyPT(Value::c(2), Value::c(2))],
    );
    for tok in fracture(&mut t, z) {
        assert!(t.obj(tok).chars.has_subtype("Zombie"));
        assert_eq!(t.pt(tok), (1, 1));
    }
    // A Clone copying Hill Giant: the tokens are Hill Giants.
    let mut t = TestGame::new(3);
    let giant = t.battlefield(P0, "Hill Giant");
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.answer_choose(P1, &[obj(giant)]);
    let clone = t.enter(P1, "Clone");
    t.resolve_all();
    assert_eq!(name_now(&t, clone), "Hill Giant");
    for tok in fracture(&mut t, clone) {
        assert_eq!(name_now(&t, tok), "Hill Giant");
        assert_eq!(t.pt(tok), (3, 3));
    }
    // P1's commander: the tokens aren't commanders.
    let mut t = TestGame::with_config(
        3,
        mtg_engine::game::GameConfig {
            variant: mtg_engine::game::Variant::Commander,
            ..Default::default()
        },
    );
    let cmdr = t.battlefield(P1, "Craw Wurm");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    t.g.players[1].commander_names.push("Craw Wurm".into());
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(false));
    for tok in fracture(&mut t, cmdr) {
        assert!(!t.obj(tok).is_commander);
    }
}

#[test]
fn declaration_in_stone_exiles_only_creatures_sharing_the_targets_name() {
    cr!("201.2", "708.2", "115.1", "109.2");
    ruling!(
        "Declaration in Stone",
        "Declaration in Stone has only one target. The other creatures with that name aren't targeted. For example, a creature with hexproof will be exiled if it has the same name as the target creature."
    );
    ruling!(
        "Declaration in Stone",
        "It's possible to have a creature and a noncreature permanent with the same name, such as two copies of the same land, one of which has become a land creature. Only creatures with that name are exiled by Declaration in Stone."
    );
    ruling!(
        "Declaration in Stone",
        "A face-down creature, such as one cast with a megamorph ability from the Dragons of Tarkir set, has no name and can't share a name with any other creatures."
    );
    supported("Declaration in Stone");
    // Two Bears, one with hexproof: both exiled; P1 investigates twice.
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Grizzly Bears");
    modify_until_eot(
        &mut t,
        b,
        vec![Modification::AddKeyword(Keyword::new(
            KeywordKind::Hexproof,
        ))],
    );
    cast_new(&mut t, P0, "Declaration in Stone", &[obj(a)]);
    t.resolve_all();
    assert!(!t.on_battlefield(a) && !t.on_battlefield(b));
    assert_eq!(tokens_of(&t, P1, "Clue"), 2);
    // Two Mutavaults, one animated (a 2/2 land creature): only that one is exiled.
    let mut t = TestGame::new(2);
    let m1 = t.battlefield(P1, "Mutavault");
    let m2 = t.battlefield(P1, "Mutavault");
    mana(&mut t, P1, mtg_engine::mana::ManaType::C, 1);
    activate_with(&mut t, P1, m1, "becomes", &[]);
    t.resolve_all();
    assert!(t.obj_now(m1).is(CardType::Creature));
    cast_new(&mut t, P0, "Declaration in Stone", &[obj(m1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(m1));
    assert!(t.on_battlefield(m2));
    // Two face-down creatures (manifested Grizzly Bears): only the target is exiled.
    let mut t = TestGame::new(2);
    let f1 = crate::r_s11_common::manifest_card(&mut t, P1, "Grizzly Bears");
    let f2 = crate::r_s11_common::manifest_card(&mut t, P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Declaration in Stone", &[obj(f1)]);
    t.resolve_all();
    assert!(!t.on_battlefield(f1));
    assert!(t.on_battlefield(f2));
}

#[test]
fn crib_swap_is_a_kindred_instant() {
    cr!("308.1", "205.2a", "205.4a");
    ruling!(
        "Crib Swap",
        "Kindred is a card type (like creature or instant), not a supertype (like legendary)."
    );
    let mut t = TestGame::new(2);
    let c = t.hand(P0, "Crib Swap");
    let ch = &t.obj(c).chars;
    assert!(ch.card_types.contains(CardType::Kindred));
    assert!(ch.card_types.contains(CardType::Instant));
    assert_eq!(ch.supertypes.iter().count(), 0);
}

#[test]
fn oversimplify_counts_power_as_the_creatures_last_existed() {
    cr!("608.2h", "903.9a");
    ruling!(
        "Oversimplify",
        "Oversimplify counts the total power of the creatures as they last existed on the battlefield, not the power of those cards in exile."
    );
    ruling!(
        "Oversimplify",
        "If a commander is exiled this way, its power will be counted along with the other creatures. Once Oversimplify finishes resolving, that commander's owner may move it to the command zone."
    );
    supported("Oversimplify");
    let mut t = TestGame::with_config(
        2,
        mtg_engine::game::GameConfig {
            variant: mtg_engine::game::Variant::Commander,
            ..Default::default()
        },
    );
    // P1: Grizzly Bears with a +1/+1 counter (3) and its commander Craw Wurm, +2/+0 until
    // end of turn (8); P0: Hill Giant (3).
    let bears = t.battlefield(P1, "Grizzly Bears");
    put_counters(&mut t, bears, "+1/+1", 1);
    let cmdr = t.battlefield(P1, "Craw Wurm");
    t.g.objects[cmdr.0 as usize].is_commander = true;
    t.g.players[1].commander_names.push("Craw Wurm".into());
    modify_until_eot(
        &mut t,
        cmdr,
        vec![Modification::ModifyPT(Value::c(2), Value::c(0))],
    );
    t.battlefield(P0, "Hill Giant");
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    cast_new(&mut t, P0, "Oversimplify", &[]);
    t.resolve_all();
    let fractal = |t: &TestGame, p| {
        t.g.permanents()
            .find(|o| o.controller == p && o.chars.has_subtype("Fractal"))
            .map(|o| o.id)
            .expect("fractal")
    };
    assert_eq!(t.counters(fractal(&t, P1), "+1/+1"), 11);
    assert_eq!(t.counters(fractal(&t, P0), "+1/+1"), 3);
    assert_eq!(t.zone(t.g.current(cmdr)), Zone::Command);
}

/// P0 casts Dark Salvation with X = `x`, targeting `player` and (if any) `creature`.
fn dark_salvation(t: &mut TestGame, x: i64, player: PlayerId, creature: Option<ObjectId>) {
    supported("Dark Salvation");
    t.lands(P0, "Swamp", 1 + 2 * x as usize);
    let c = t.hand(P0, "Dark Salvation");
    t.answer_targets(P0, &[Entity::Player(player)]);
    t.answer_targets(P0, &creature.map(obj).into_iter().collect::<Vec<_>>());
    t.cast(P0, c).x(x).go();
}

#[test]
fn dark_salvation_targets_up_to_one_creature_and_works_in_parts() {
    cr!("601.2c", "608.2b", "115.1");
    ruling!(
        "Dark Salvation",
        "You choose up to one target creature total, not a separate target for each Zombie that player controls."
    );
    ruling!(
        "Dark Salvation",
        "You may cast Dark Salvation with X as 0 just to have a creature get -1/-1 for each Zombie a player controls"
    );
    ruling!(
        "Dark Salvation",
        "If the targeted creature becomes an illegal target, the targeted player will still get Zombie tokens."
    );
    // X = 2 for P0: two Zombies, and P1's Craw Wurm (6/4) gets -2/-2.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    dark_salvation(&mut t, 2, P0, Some(wurm));
    let spell = *t.g.stack.last().unwrap();
    let slots = crate::r_s25_common::targets_of(&t, spell);
    assert_eq!(slots.len(), 2, "a player and one creature: {slots:?}");
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Zombie"), 2);
    assert_eq!(t.pt(wurm), (4, 2));
    // X = 0, P1 controls two Zombies: the Wurm gets -2/-2.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    create_token(&mut t, P1, "Zombie");
    create_token(&mut t, P1, "Zombie");
    dark_salvation(&mut t, 0, P1, Some(wurm));
    t.resolve_all();
    assert_eq!(t.pt(wurm), (4, 2));
    // X = 2, no target creature: just the Zombies.
    let mut t = TestGame::new(2);
    dark_salvation(&mut t, 2, P1, None);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P1, "Zombie"), 2);
    // X = 0, no creature: nothing.
    let mut t = TestGame::new(2);
    let w = t.battlefield(P1, "Craw Wurm");
    dark_salvation(&mut t, 0, P1, None);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P1, "Zombie"), 0);
    assert_eq!(t.pt(w), (6, 4));
    // The creature leaves in response: the player still gets the Zombies.
    let mut t = TestGame::new(2);
    let wurm = t.battlefield(P1, "Craw Wurm");
    dark_salvation(&mut t, 2, P0, Some(wurm));
    bounce(&mut t, wurm);
    t.resolve_all();
    assert_eq!(tokens_of(&t, P0, "Zombie"), 2);
}

#[test]
fn baffling_ends_abilities_are_independent() {
    cr!("603.2", "608.2b");
    ruling!(
        "Baffling End",
        "In a multiplayer game, the target opponent who creates a Dinosaur token doesn’t have to be the same one whose creature was exiled."
    );
    ruling!(
        "Baffling End",
        "If the first ability doesn’t resolve (perhaps because the target gained hexproof), the second will still create a Dinosaur token when Baffling End leaves the battlefield."
    );
    supported("Baffling End");
    // Three players: P1's Bears exiled; when Baffling End leaves, P2 gets the Dinosaur.
    let mut t = TestGame::new(3);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let end = t.enter(P0, "Baffling End");
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    t.answer_targets(P0, &[Entity::Player(P2)]);
    destroy(&mut t, end);
    t.resolve_all();
    assert_eq!(tokens_by_player(&t, "Dinosaur"), vec![0, 0, 1]);
    // The first ability's target gains hexproof in response: it does nothing; the second
    // still makes a Dinosaur.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.answer_targets(P0, &[obj(bears)]);
    let end = t.enter(P0, "Baffling End");
    t.settle();
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::AddKeyword(Keyword::new(
            KeywordKind::Hexproof,
        ))],
    );
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    t.answer_targets(P0, &[Entity::Player(P1)]);
    destroy(&mut t, end);
    t.resolve_all();
    assert_eq!(tokens_by_player(&t, "Dinosaur"), vec![0, 1]);
}

#[test]
fn awaken_the_erstwhile_gives_each_player_zombies_for_their_own_discards() {
    cr!("608.2c", "701.9a");
    ruling!(
        "Awaken the Erstwhile",
        "The number of Zombie tokens each player creates is equal to the number of cards that player discarded, not the total number of cards discarded."
    );
    supported("Awaken the Erstwhile");
    let mut t = TestGame::new(3);
    t.hand(P0, "Grizzly Bears");
    for _ in 0..3 {
        t.hand(P1, "Grizzly Bears");
    }
    cast_new(&mut t, P0, "Awaken the Erstwhile", &[]);
    t.resolve_all();
    assert_eq!(tokens_by_player(&t, "Zombie"), vec![1, 3, 0]);
}

#[test]
fn countering_a_spell_that_cant_be_countered_still_gives_the_bird() {
    cr!("101.2", "701.6b");
    ruling!(
        "Strix Serenade",
        "Strix Serenade can target a spell that can't be countered. That spell won't be countered when Strix Serenade resolves, but its controller will get a Bird token."
    );
    ruling!(
        "Swan Song",
        "Swan Song can target a spell that can't be countered. That spell won't be countered when Swan Song resolves, but its controller will get a Bird token."
    );
    // P1 casts Carnage Tyrant ("This spell can't be countered.") / Abrupt Decay (likewise)
    // and P0 responds.
    for (counter, spell, target) in [
        ("Strix Serenade", "Carnage Tyrant", None),
        ("Swan Song", "Abrupt Decay", Some("Sol Ring")),
    ] {
        supported(counter);
        supported(spell);
        let mut t = TestGame::new(2);
        t.set_step(P1, Step::PrecombatMain);
        let targets: Vec<Entity> = target
            .map(|n| obj(t.battlefield(P0, n)))
            .into_iter()
            .collect();
        let s = cast_new(&mut t, P1, spell, &targets);
        cast_new(&mut t, P0, counter, &[obj(s)]);
        t.resolve();
        assert!(t.g.stack.contains(&s), "{spell} stays on the stack");
        assert_eq!(tokens_by_player(&t, "Bird"), vec![0, 1], "{counter}");
        t.resolve_all();
        if target.is_none() {
            assert_eq!(t.named_on_battlefield(spell).len(), 1);
        } else {
            assert!(t.in_graveyard(P0, "Sol Ring"));
        }
    }
}

/// P0 casts Unyielding Gatekeeper face down (disguise) and it resolves.
fn face_down_gatekeeper(t: &mut TestGame) -> ObjectId {
    supported("Unyielding Gatekeeper");
    t.lands(P0, "Wastes", 4);
    t.lands(P0, "Plains", 1);
    let card = t.hand(P0, "Unyielding Gatekeeper");
    let spell = t
        .cast(P0, card)
        .method(CastMethod::FaceDown(KeywordKind::Disguise))
        .go();
    t.resolve_all();
    t.g.current(spell)
}

#[test]
fn unyielding_gatekeeper_asks_who_controlled_the_target_as_it_resolves() {
    cr!("608.2h", "110.2");
    ruling!(
        "Unyielding Gatekeeper",
        "If the controller of Unyielding Gatekeeper's last ability controlled the target permanent at the time the ability began to resolve, the permanent will return to the battlefield tapped under that player's control, regardless of who its owner is."
    );
    ruling!(
        "Unyielding Gatekeeper",
        "If you controlled the target permanent earlier in the turn but not at the time Unyielding Gatekeeper's last ability resolves, you won't return it to the battlefield tapped. The player who controlled that permanent at the time it was exiled will create a Detective."
    );
    // P1's Bears that P0 controls: back tapped under P0's control.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    crate::r_s06_common::give_control(&mut t, bears, P0);
    let g = face_down_gatekeeper(&mut t);
    t.answer_targets(P0, &[obj(bears)]);
    assert!(crate::r_s11_common::turn_face_up(&mut t, P0, g));
    t.resolve_all();
    let back = t.named_on_battlefield("Grizzly Bears");
    assert_eq!(back.len(), 1);
    assert_eq!(t.obj(back[0]).controller, P0);
    assert_eq!(t.obj(back[0]).owner, P1);
    assert!(t.obj(back[0]).tapped);
    assert_eq!(tokens_by_player(&t, "Detective"), vec![0, 0]);
    // P0's own Bears, given to P1 in response: exiled, and P1 creates a Detective.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let g = face_down_gatekeeper(&mut t);
    t.answer_targets(P0, &[obj(bears)]);
    assert!(crate::r_s11_common::turn_face_up(&mut t, P0, g));
    t.settle();
    crate::r_s06_common::give_control(&mut t, bears, P1);
    t.resolve_all();
    assert!(t.in_exile("Grizzly Bears"));
    assert_eq!(tokens_by_player(&t, "Detective"), vec![0, 1]);
}

#[test]
fn alirios_untaps_only_in_untap_steps_without_a_reflection_or_by_effects() {
    cr!("502.3", "701.26b");
    ruling!(
        "Alirios, Enraptured",
        "Once you no longer control a Reflection, Alirios doesn't immediately become untapped. It'll be untapped during your next untap step."
    );
    ruling!(
        "Alirios, Enraptured",
        "Spells and abilities can untap Alirios, even if it doesn't untap during your untap step."
    );
    supported("Alirios, Enraptured");
    let mut t = TestGame::new(2);
    let alirios = t.enter(P0, "Alirios, Enraptured");
    t.resolve_all();
    assert!(t.obj_now(alirios).tapped);
    let reflection = with_reflection(&t);
    // With the Reflection, it stays tapped through P0's untap step.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(t.obj_now(alirios).tapped);
    // An effect untaps it (Kiora's Follower: "{T}: Untap another target permanent.").
    let follower = t.battlefield(P0, "Kiora's Follower");
    activate_with(
        &mut t,
        P0,
        follower,
        "Untap another target permanent",
        &[obj(alirios)],
    );
    t.resolve_all();
    assert!(!t.obj_now(alirios).tapped);
    // Tapped again, then the Reflection leaves: still tapped until the next untap step.
    tap(&mut t, alirios);
    destroy(&mut t, reflection);
    assert!(t.obj_now(alirios).tapped);
    t.advance_to(P1, Step::Upkeep);
    assert!(t.obj_now(alirios).tapped);
    t.advance_to(P0, Step::Upkeep);
    assert!(!t.obj_now(alirios).tapped);
}

/// P0's Reflection token.
fn with_reflection(t: &TestGame) -> ObjectId {
    t.g.permanents()
        .find(|o| o.controller == P0 && o.chars.has_subtype("Reflection"))
        .map(|o| o.id)
        .expect("a Reflection")
}

#[test]
fn farmstead_gleaners_untap_symbol_cost() {
    cr!("107.6", "302.6", "602.5a");
    ruling!(
        "Farmstead Gleaner",
        "You can’t pay the {Q} cost if Farmstead Gleaner is already untapped, and you must have controlled the creature continuously since your most recent turn began in order to pay {Q}."
    );
    supported("Farmstead Gleaner");
    for (tapped, sick, ok) in [
        (false, false, false),
        (true, true, false),
        (true, false, true),
    ] {
        let mut t = TestGame::new(2);
        let g = if sick {
            t.battlefield_sick(P0, "Farmstead Gleaner")
        } else {
            t.battlefield(P0, "Farmstead Gleaner")
        };
        if tapped {
            tap(&mut t, g);
        }
        t.lands(P0, "Wastes", 2);
        let r = activate_containing(&mut t, P0, g, "+1/+1 counter");
        assert_eq!(r.is_ok(), ok, "tapped {tapped}, sick {sick}");
        if ok {
            t.resolve_all();
            assert!(!t.obj_now(g).tapped);
            assert_eq!(t.counters(g, "+1/+1"), 1);
        }
    }
}

#[test]
fn removal_whose_target_changes_before_resolution() {
    cr!("608.2b", "115.1");
    ruling!(
        "Molten Frame",
        "Any permanent that's both an artifact and a creature is a legal target for Molten Frame."
    );
    ruling!(
        "Disruptive Stormbrood // Petty Revenge",
        "If the target creature’s power becomes greater than 3 before Petty Revenge resolves, the target is illegal and the spell is put into its owner’s graveyard."
    );
    ruling!(
        "Go for the Throat",
        "If the targeted creature is also an artifact when Go for the Throat tries to resolve, it won't resolve."
    );
    // Molten Frame: Ornithopter and Iron Golem (artifact creatures) are legal targets;
    // Grizzly Bears and Sol Ring aren't.
    supported("Molten Frame");
    let mut t = TestGame::new(2);
    let thopter = t.battlefield(P1, "Ornithopter");
    let bears = t.battlefield(P1, "Grizzly Bears");
    let ring = t.battlefield(P1, "Sol Ring");
    let golem = t.battlefield(P1, "Iron Golem");
    let legal = crate::r_s04_common::spell_targets(&mut t, P0, "Molten Frame");
    assert!(legal.contains(&obj(thopter)) && legal.contains(&obj(golem)));
    assert!(!legal.contains(&obj(bears)) && !legal.contains(&obj(ring)));
    // Petty Revenge on Grizzly Bears pumped to 5 power in response: it doesn't resolve,
    // and the card goes to the graveyard instead of being shuffled into the library.
    supported("Disruptive Stormbrood // Petty Revenge");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.lands(P0, "Swamp", 2);
    let card = t.hand(P0, "Disruptive Stormbrood // Petty Revenge");
    t.answer_targets(P0, &[obj(bears)]);
    t.cast(P0, card).method(CastMethod::Half(1)).go();
    let lib = t.library_size(P0);
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::ModifyPT(Value::c(3), Value::c(3))],
    );
    t.resolve_all();
    assert!(t.on_battlefield(bears));
    assert_eq!(t.zone(t.g.current(card)), Zone::Graveyard(P0));
    assert_eq!(t.library_size(P0), lib);
    // Go for the Throat on Grizzly Bears that becomes an artifact in response.
    supported("Go for the Throat");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    cast_new(&mut t, P0, "Go for the Throat", &[obj(bears)]);
    modify_until_eot(
        &mut t,
        bears,
        vec![Modification::AddTypes(vec![CardType::Artifact])],
    );
    t.resolve_all();
    assert!(t.on_battlefield(bears));
}
