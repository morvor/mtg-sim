//! Rulings batch P115 — cards that name a card after looking at or revealing a hand
//! (Anointed Peacekeeper, Alhammarret, High Arbiter, Thought Hemorrhage, Brain Pry) and
//! Quash's search for every card with the countered spell's name (CR 201.4, 701.23).

use crate::r_p115_common::*;
use crate::r_s02_common::create_token;
use crate::r_s04_common::add_mana;
use mtg_engine::decision::{Action, Decision};
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::*;

fn name(t: &mut TestGame, p: PlayerId, card: &str) {
    t.answer(p, DecisionKind::Name, Answer::Text(card.into()));
}

/// Anointed Peacekeeper enters under P0's control naming `card` (P0 looks at P1's hand).
fn peacekeeper(t: &mut TestGame, card: &str) -> ObjectId {
    name(t, P0, card);
    let id = t.enter(P0, "Anointed Peacekeeper");
    t.settle();
    id
}

/// Whether `p` can activate the `index`th activated ability of `src` with exactly
/// `generic` colorless mana (plus one of each color for colored costs) in their pool.
fn activates_with(
    t: &mut TestGame,
    p: PlayerId,
    src: ObjectId,
    index: usize,
    generic: u32,
) -> bool {
    t.g.players[p.idx()].mana_pool = Default::default();
    add_mana(t, p, ManaType::C, generic);
    let ok = t.activate(p, src, index, &[]).is_ok();
    t.g.players[p.idx()].mana_pool = Default::default();
    ok
}

#[test]
fn anointed_peacekeeper_taxes_activated_abilities_not_triggered_or_mana_abilities() {
    cr!("602.1", "605.1a", "601.2f");
    ruling!(
        "Anointed Peacekeeper",
        "Some keyword abilities (such as equip) are activated abilities and will have colons in their reminder text. Triggered abilities (starting with “when,” “whenever,” or “at”) are unaffected by the last ability of Anointed Peacekeeper."
    );
    supported("Anointed Peacekeeper");
    // Equip {1} costs {3}.
    let mut t = TestGame::new(2);
    peacekeeper(&mut t, "Bonesplitter");
    let split = t.battlefield(P0, "Bonesplitter");
    t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    assert!(!activates_with(&mut t, P0, split, 0, 1));
    assert!(!activates_with(&mut t, P0, split, 0, 2));
    let mut t = TestGame::new(2);
    peacekeeper(&mut t, "Bonesplitter");
    let split = t.battlefield(P0, "Bonesplitter");
    let bears = t.battlefield(P0, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::C, 3);
    t.activate(P0, split, 0, &[obj(bears)])
        .expect("equip for {3}");
    t.resolve_all();
    assert_eq!(t.pt(bears), (4, 2));
    // Mind Stone's mana ability is untaxed: it still adds {C} for {T}.
    let mut t = TestGame::new(2);
    peacekeeper(&mut t, "Mind Stone");
    let stone = t.battlefield(P0, "Mind Stone");
    assert!(activates_with(&mut t, P0, stone, 0, 0));
    // A named card's triggered ability still triggers and resolves with no extra payment
    // (Raging Regisaur's attack trigger).
    let mut t = TestGame::new(2);
    peacekeeper(&mut t, "Raging Regisaur");
    let rex = t.battlefield(P0, "Raging Regisaur");
    t.answer_targets(P0, &[pl(P1)]);
    attack_with(&mut t, &[(rex, pl(P1))]);
    t.resolve_all();
    assert_eq!(t.life(P1), 19);
}

#[test]
fn anointed_peacekeeper_taxes_abilities_in_every_zone() {
    cr!("602.2b", "601.2f");
    ruling!(
        "Anointed Peacekeeper",
        "Anointed Peacekeeper’s last ability affects cards regardless of what zone they’re in. This includes cards in hands, cards in the graveyards, and exiled cards."
    );
    // Cycling from the hand: Krosan Tusker's cycling {2}{G} costs {4}{G}.
    for (generic, ok) in [(2u32, false), (4, true)] {
        let mut t = TestGame::new(2);
        peacekeeper(&mut t, "Krosan Tusker");
        let tusker = t.hand(P0, "Krosan Tusker");
        add_mana(&mut t, P0, ManaType::G, 1);
        assert_eq!(
            activates_with_keep(&mut t, P0, tusker, generic),
            ok,
            "{generic}"
        );
    }
    // Unearth from the graveyard: Hellspark Elemental's unearth {1}{R} costs {3}{R}.
    for (generic, ok) in [(1u32, false), (3, true)] {
        let mut t = TestGame::new(2);
        peacekeeper(&mut t, "Hellspark Elemental");
        let elemental = t.graveyard(P0, "Hellspark Elemental");
        add_mana(&mut t, P0, ManaType::R, 1);
        assert_eq!(
            activates_with_keep(&mut t, P0, elemental, generic),
            ok,
            "{generic}"
        );
    }
}

/// Like [`activates_with`] for ability 0, keeping the colored mana already in the pool.
fn activates_with_keep(t: &mut TestGame, p: PlayerId, src: ObjectId, generic: u32) -> bool {
    add_mana(t, p, ManaType::C, generic);
    t.activate(p, src, 0, &[]).is_ok()
}

#[test]
fn anointed_peacekeepers_spell_tax_applies_to_every_opponent() {
    cr!("601.2f");
    ruling!(
        "Anointed Peacekeeper",
        "Anointed Peacekeeper’s third ability applies to all opponents, not just the opponent whose hand you looked at."
    );
    let mut t = TestGame::new(3);
    peacekeeper(&mut t, "Grizzly Bears");
    for p in [P1, P2] {
        let bears = t.hand(p, "Grizzly Bears");
        // (Making `p` the active player is a test shortcut so that the creature spell's
        // timing is legal and only its cost decides.)
        t.g.turn.active = p;
        t.g.turn.priority = Some(p);
        // {1}{G} costs {3}{G}: {1}{G} isn't enough, {3}{G} is.
        add_mana(&mut t, p, ManaType::G, 1);
        add_mana(&mut t, p, ManaType::C, 1);
        assert!(
            t.g.cast_spell(p, bears, CastMethod::Normal).is_err(),
            "{p:?}"
        );
        add_mana(&mut t, p, ManaType::C, 2);
        t.g.turn.priority = Some(p);
        assert!(
            t.g.cast_spell(p, bears, CastMethod::Normal).is_ok(),
            "{p:?}"
        );
        t.resolve_all();
        t.g.turn.active = P0;
    }
    // Its controller's own spells aren't taxed.
    let bears = t.hand(P0, "Grizzly Bears");
    add_mana(&mut t, P0, ManaType::G, 2);
    t.g.turn.priority = Some(P0);
    assert!(t.g.cast_spell(P0, bears, CastMethod::Normal).is_ok());
}

#[test]
fn anointed_peacekeeper_names_any_card_but_not_a_token_only_name() {
    cr!("201.4", "111.4");
    ruling!(
        "Anointed Peacekeeper",
        "You can choose any card name, even if that card doesn’t normally have an activated ability. You’re not limited to the names of cards you saw in the opponent’s hand."
    );
    ruling!(
        "Anointed Peacekeeper",
        "You can’t choose the name of a token unless that token has the same name as a card."
    );
    // P1's hand holds only Hill Giant; P0 names Grizzly Bears (no activated abilities).
    let mut t = TestGame::new(2);
    t.hand(P1, "Hill Giant");
    let pk = peacekeeper(&mut t, "Grizzly Bears");
    assert_eq!(
        t.obj_now(pk).choices.card_name.as_deref(),
        Some("Grizzly Bears")
    );
    // A Clue token is named "Clue Token" (CR 111.4, 111.10f), the name of no card: it can't
    // be chosen, so the token's "{2}, Sacrifice this token: Draw a card." costs only {2}.
    let mut t = TestGame::new(2);
    let pk = peacekeeper(&mut t, "Clue Token");
    let clue = create_token(&mut t, P0, "Clue");
    assert_eq!(t.obj_now(clue).chars.name, "Clue Token");
    assert_ne!(
        t.obj_now(pk).choices.card_name.as_deref(),
        Some("Clue Token")
    );
    assert!(activates_with(&mut t, P0, clue, 0, 2));
    // A token with a card's name can be named: a token copy of Mogg Fanatic (Cackling
    // Counterpart) is named Mogg Fanatic, and its ability costs {2} more.
    let mut t = TestGame::new(2);
    peacekeeper(&mut t, "Mogg Fanatic");
    let fanatic = t.battlefield(P0, "Mogg Fanatic");
    cast_at(&mut t, P0, "Cackling Counterpart", &[obj(fanatic)]);
    t.resolve_all();
    let copy =
        t.g.permanents()
            .find(|o| o.is_token() && o.chars.name == "Mogg Fanatic")
            .map(|o| o.id)
            .expect("the token copy");
    t.answer_targets(P0, &[pl(P1)]);
    assert!(!activates_with(&mut t, P0, copy, 0, 1));
    assert!(t.on_battlefield(copy));
    t.answer_targets(P0, &[pl(P1)]);
    assert!(activates_with(&mut t, P0, copy, 0, 2));
}

/// Alhammarret enters under P0's control; P0 chooses `card` (from P1's revealed hand).
fn alhammarret(t: &mut TestGame, card: &str) -> ObjectId {
    let c =
        t.g.player(P1)
            .hand
            .iter()
            .copied()
            .find(|c| t.g.obj(*c).chars.name == card);
    if let Some(c) = c {
        t.answer_choose(P0, &[obj(c)]);
    }
    name(t, P0, card);
    t.answer(P0, DecisionKind::Option, Answer::Text(card.into()));
    let id = t.enter(P0, "Alhammarret, High Arbiter");
    t.settle();
    id
}

#[test]
fn alhammarrets_name_is_chosen_as_it_enters_with_no_window() {
    cr!("614.1c", "614.12", "117.3d");
    ruling!(
        "Alhammarret, High Arbiter",
        "No one can cast spells or activate abilities between the time a card is named and the time that Alhammarret's last ability starts to work."
    );
    supported("Alhammarret, High Arbiter");
    // Through the real priority loop: P0 casts Alhammarret; P1, holding Shock and mana,
    // passes while it's on the stack and tries to cast Shock the next time it gets
    // priority. That is after Alhammarret has entered and Shock has been named, so the
    // attempt fails (with a window between them, as an "enters" trigger naming the card
    // would leave, Shock would be cast).
    let mut t = TestGame::new(2);
    let shock = t.hand(P1, "Shock");
    lots(&mut t, P1);
    t.answer(P1, DecisionKind::Priority, Answer::Action(Action::Pass));
    t.answer(
        P1,
        DecisionKind::Priority,
        Answer::Action(Action::Cast {
            card: shock,
            method: CastMethod::Normal,
        }),
    );
    t.answer_targets(P1, &[pl(P0)]);
    t.answer_choose(P0, &[obj(shock)]);
    name(&mut t, P0, "Shock");
    t.answer(P0, DecisionKind::Option, Answer::Text("Shock".into()));
    // P1's priority: the stack's size and whether Alhammarret is on the battlefield.
    let prio = watch(
        &mut t,
        P1,
        |d| matches!(d, Decision::Priority { .. }),
        |g| {
            (
                g.stack.len(),
                g.permanents()
                    .any(|o| o.chars.name == "Alhammarret, High Arbiter"),
            )
        },
    );
    lots(&mut t, P0);
    let card = t.hand(P0, "Alhammarret, High Arbiter");
    t.cast(P0, card).go();
    pass_until_step_ends(&mut t);
    assert_eq!(t.named_on_battlefield("Alhammarret, High Arbiter").len(), 1);
    let prio = prio.lock().unwrap().clone();
    // P1 could respond to the spell; then, with Alhammarret on the battlefield, P1 tried
    // Shock (refused) and was asked again.
    assert!(prio.contains(&(1, false)), "{prio:?}");
    assert!(
        prio.iter().filter(|&&(n, on)| n == 0 && on).count() >= 2,
        "{prio:?}"
    );
    assert_eq!(t.zone(shock), Zone::Hand(P1));
    assert_eq!(t.life(P0), 20);
    // The prohibition applies to the named card only.
    let mut t = TestGame::new(2);
    let bears = t.hand(P1, "Grizzly Bears");
    alhammarret(&mut t, "Grizzly Bears");
    lots(&mut t, P1);
    t.g.turn.priority = Some(P1);
    t.g.turn.active = P1;
    assert!(t.g.cast_spell(P1, bears, CastMethod::Normal).is_err());
    // Another name can still be cast.
    let giant = t.hand(P1, "Hill Giant");
    assert!(t.g.cast_spell(P1, giant, CastMethod::Normal).is_ok());
}

#[test]
fn alhammarret_doesnt_stop_putting_the_card_onto_the_battlefield() {
    cr!("101.2", "601.3");
    ruling!(
        "Alhammarret, High Arbiter",
        "Although spells with the chosen name can't be cast, permanent cards with that name can still be put onto the battlefield by a spell or ability."
    );
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P1, "Grizzly Bears");
    alhammarret(&mut t, "Grizzly Bears");
    let dead = t.graveyard(P1, "Grizzly Bears");
    t.g.turn.active = P1;
    lots(&mut t, P1);
    t.g.turn.priority = Some(P1);
    assert!(t.g.cast_spell(P1, in_hand, CastMethod::Normal).is_err());
    cast_at(&mut t, P1, "Reanimate", &[obj(dead)]);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert_eq!(t.zone(in_hand), Zone::Hand(P1));
}

#[test]
fn alhammarret_doesnt_stop_face_down_spells_or_manifest() {
    cr!("708.2", "701.40a");
    ruling!(
        "Alhammarret, High Arbiter",
        "Your opponents can still cast a card with the chosen name face down if it has a morph ability. A card with the chosen name can also be manifested."
    );
    let mut t = TestGame::new(2);
    let angel = t.hand(P1, "Exalted Angel");
    alhammarret(&mut t, "Exalted Angel");
    t.g.turn.active = P1;
    lots(&mut t, P1);
    t.g.turn.priority = Some(P1);
    assert!(t.g.cast_spell(P1, angel, CastMethod::Normal).is_err());
    lots(&mut t, P1);
    assert!(t
        .g
        .cast_spell(P1, angel, CastMethod::FaceDown(KeywordKind::Morph))
        .is_ok());
    t.resolve_all();
    // Manifest: Soul Summons puts the top card (another Exalted Angel) face down.
    t.library_top(P1, "Exalted Angel");
    cast_at(&mut t, P1, "Soul Summons", &[]);
    t.resolve_all();
    let face_down =
        t.g.permanents()
            .filter(|o| o.controller == P1 && o.face_down)
            .count();
    assert_eq!(face_down, 2);
}

#[test]
fn thought_hemorrhage_damage_is_one_instance() {
    cr!("609.7", "609.7a", "615.1");
    ruling!(
        "Thought Hemorrhage",
        "the damage dealt by Thought Hemorrhage is all considered to be the same instance of damage"
    );
    supported("Thought Hemorrhage");
    supported("Circle of Protection: Red");
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    name(&mut t, P0, "Grizzly Bears");
    let from = t.g.turn_events.len();
    cast_at(&mut t, P0, "Thought Hemorrhage", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 14);
    let hits: Vec<u32> = t.g.turn_events[from..]
        .iter()
        .filter_map(|e| match e {
            Event::Damage { amount, .. } => Some(*amount),
            _ => None,
        })
        .collect();
    assert_eq!(hits, vec![6]);
    // The ruling's example: one effect preventing "the next time a source would deal
    // damage" prevents all of it. P1 answers with Circle of Protection: Red, choosing the
    // spell on the stack as the source; with 3 damage dealt twice, it would stop only 3.
    let mut t = TestGame::new(2);
    t.hand(P1, "Grizzly Bears");
    t.hand(P1, "Grizzly Bears");
    let circle = t.battlefield(P1, "Circle of Protection: Red");
    name(&mut t, P0, "Grizzly Bears");
    let spell = cast_at(&mut t, P0, "Thought Hemorrhage", &[pl(P1)]);
    lots(&mut t, P1);
    t.answer_choose(P1, &[obj(spell)]);
    t.activate(P1, circle, 0, &[])
        .expect("Circle of Protection: Red");
    t.resolve_all();
    assert_eq!(t.life(P1), 20);
    assert!(!t.in_hand(P1, "Grizzly Bears"));
}

#[test]
fn naming_happens_as_the_spell_resolves() {
    cr!("608.2c", "608.2d", "117.3d");
    ruling!(
        "Thought Hemorrhage",
        "The card is named as Thought Hemorrhage resolves. Spells and abilities can’t be cast or activated between the time a card is named and the time that Thought Hemorrhage has the rest of its effect."
    );
    ruling!(
        "Brain Pry",
        "You name the card during Brain Pry’s resolution, then the player reveals their hand and discards if appropriate. There is no way for the player to do anything between you naming the card and the player discarding."
    );
    supported("Brain Pry");
    for spell in ["Thought Hemorrhage", "Brain Pry"] {
        let mut t = TestGame::new(2);
        t.hand(P1, "Grizzly Bears");
        // When P0 names the card, and P1's priority: when, whether the spell is still on
        // the stack and whether the Bears are still in P1's hand.
        let named = watch(
            &mut t,
            P0,
            |d| matches!(d, Decision::NameCard { .. }),
            |_| next_seq(),
        );
        let prio = watch(
            &mut t,
            P1,
            |d| matches!(d, Decision::Priority { .. }),
            |g| {
                (
                    next_seq(),
                    !g.stack.is_empty(),
                    g.player(P1)
                        .hand
                        .iter()
                        .any(|c| g.obj(*c).chars.name == "Grizzly Bears"),
                )
            },
        );
        name(&mut t, P0, "Grizzly Bears");
        cast_at(&mut t, P0, spell, &[pl(P1)]);
        assert!(
            named.lock().unwrap().is_empty(),
            "{spell}: named on casting"
        );
        // Through the real priority loop: both pass, it resolves, both pass again.
        pass_until_step_ends(&mut t);
        let named = named.lock().unwrap().clone();
        assert_eq!(named.len(), 1, "{spell}");
        let prio = prio.lock().unwrap().clone();
        // P1 could respond before it resolved; the next time P1 had priority, after the
        // card was named, the spell had finished and the Bears were gone.
        assert!(
            prio.iter()
                .any(|&(s, on_stack, _)| s < named[0] && on_stack),
            "{spell}: {prio:?}"
        );
        let after: Vec<_> = prio.iter().filter(|(s, ..)| *s > named[0]).collect();
        assert!(!after.is_empty(), "{spell}: {prio:?}");
        assert!(
            after
                .iter()
                .all(|&&(_, on_stack, bears)| !on_stack && !bears),
            "{spell}: {prio:?}"
        );
        assert!(!t.in_hand(P1, "Grizzly Bears"), "{spell}");
    }
}

#[test]
fn thought_hemorrhage_finds_all_in_the_graveyard_optionally_elsewhere() {
    cr!("701.23b", "400.2");
    ruling!(
        "Thought Hemorrhage",
        "The cards you’re searching for must be found if they’re in the graveyard, because that’s a zone everyone can see. Finding those cards in the hand and library is optional, though. This is true even though the targeted player has revealed their hand."
    );
    ruling!(
        "Thought Hemorrhage",
        "Thought Hemorrhage has no effect on cards on the battlefield that have the same name as the named card."
    );
    let mut t = TestGame::new(2);
    let in_hand = t.hand(P1, "Grizzly Bears");
    let in_gy = t.graveyard(P1, "Grizzly Bears");
    let in_lib = t.library_top(P1, "Grizzly Bears");
    let on_bf = t.battlefield(P1, "Grizzly Bears");
    name(&mut t, P0, "Grizzly Bears");
    // P0 finds none in the hand or the library.
    t.answer_choose(P0, &[]);
    t.answer_choose(P0, &[]);
    cast_at(&mut t, P0, "Thought Hemorrhage", &[pl(P1)]);
    t.resolve_all();
    assert_eq!(t.life(P1), 17);
    assert_eq!(t.zone(in_gy), Zone::Exile);
    assert_eq!(t.zone(in_hand), Zone::Hand(P1));
    assert_eq!(t.zone(in_lib), Zone::Library(P1));
    assert!(t.on_battlefield(on_bf));
}

/// P1 casts `spell` (targets given); P0 casts Quash targeting it; everything resolves.
fn quash(t: &mut TestGame, spell: ObjectId) {
    cast_at(t, P0, "Quash", &[obj(spell)]);
    t.resolve_all();
}

#[test]
fn quash_exiles_the_countered_spell_and_the_other_copies() {
    cr!("701.6a", "608.2c");
    ruling!(
        "Quash",
        "It exiles the countered spell. This is because the first sentence puts the spell into the graveyard before you continue to the second sentence."
    );
    supported("Quash");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.hand(P1, "Lightning Bolt");
    t.library_top(P1, "Lightning Bolt");
    t.graveyard(P1, "Lightning Bolt");
    let bolt = cast_at(&mut t, P1, "Lightning Bolt", &[obj(bears)]);
    quash(&mut t, bolt);
    assert!(t.on_battlefield(bears));
    let in_zones = |t: &TestGame, z: fn(PlayerId) -> Zone| {
        t.g.objects
            .iter()
            .filter(|o| o.chars.name == "Lightning Bolt" && o.zone == z(P1) && t.g.is_live(o.id))
            .count()
    };
    assert_eq!(in_zones(&t, Zone::Graveyard), 0);
    assert_eq!(in_zones(&t, Zone::Hand), 0);
    assert_eq!(in_zones(&t, Zone::Library), 0);
    let exiled =
        t.g.exile
            .iter()
            .filter(|c| t.g.obj(**c).chars.name == "Lightning Bolt")
            .count();
    assert_eq!(exiled, 4);
}

#[test]
fn quash_still_searches_when_the_spell_cant_be_countered() {
    cr!("113.6g", "101.2", "608.2c");
    ruling!(
        "Quash",
        "If the spell is not countered (because the spell it targets can’t be countered), then it does not get exiled but all the other copies in the graveyard, hand, and library are exiled."
    );
    let mut t = TestGame::new(2);
    let stone = t.battlefield(P0, "Mind Stone");
    t.hand(P1, "Abrupt Decay");
    t.graveyard(P1, "Abrupt Decay");
    let decay = cast_at(&mut t, P1, "Abrupt Decay", &[obj(stone)]);
    quash(&mut t, decay);
    assert!(!t.on_battlefield(stone));
    assert!(t.in_graveyard(P1, "Abrupt Decay"));
    let gy = gy_names(&t, P1)
        .iter()
        .filter(|n| *n == "Abrupt Decay")
        .count();
    assert_eq!(gy, 1);
    assert!(!t.in_hand(P1, "Abrupt Decay"));
    let exiled =
        t.g.exile
            .iter()
            .filter(|c| t.g.obj(**c).chars.name == "Abrupt Decay")
            .count();
    assert_eq!(exiled, 2);
}

#[test]
fn quash_exiles_only_the_base_spells_name_when_text_is_spliced() {
    cr!("702.47c", "201.2a");
    ruling!(
        "Quash",
        "If Quash counters a spell that has had text spliced onto it, only the base spell may be exiled."
    );
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P1, "Grizzly Bears");
    lots(&mut t, P1);
    let might = t.hand(P1, "Kodama's Might");
    t.hand(P1, "Glacial Ray");
    t.hand(P1, "Kodama's Might");
    t.answer(P1, DecisionKind::OptionalCost, Answer::Bool(true));
    let spell = t.cast(P1, might).target(bears).target(P0).go();
    quash(&mut t, spell);
    assert_eq!(t.life(P0), 20);
    assert!(t.in_hand(P1, "Glacial Ray"));
    assert!(!t.in_hand(P1, "Kodama's Might"));
    assert!(!t.in_graveyard(P1, "Kodama's Might"));
}

#[test]
fn quash_exiles_cards_named_like_a_countered_copy() {
    cr!("707.10", "707.12");
    ruling!(
        "Quash",
        "Quash can exile cards even if the spell countered was a copy of a spell."
    );
    supported("Isochron Scepter");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    // P1's Isochron Scepter with Boomerang imprinted; another Boomerang in hand.
    let imprinted = t.hand(P1, "Boomerang");
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[obj(imprinted)]);
    let scepter = t.enter(P1, "Isochron Scepter");
    t.resolve_all();
    assert_eq!(t.zone(imprinted), Zone::Exile);
    t.hand(P1, "Boomerang");
    lots(&mut t, P1);
    t.answer_yes(P1, true);
    t.answer_yes(P1, true);
    t.answer_targets(P1, &[obj(bears)]);
    t.activate(P1, scepter, 0, &[]).expect("Scepter");
    t.resolve();
    // The cast copy of Boomerang is on the stack.
    assert_eq!(t.stack_len(), 1);
    let copy = t.g.stack[0];
    assert_eq!(t.g.obj(copy).chars.name, "Boomerang");
    quash(&mut t, copy);
    assert!(t.on_battlefield(bears));
    assert!(!t.in_hand(P1, "Boomerang"));
}
