//! Rulings on meld cards (CR 712.4, 701.42): which cards meld, choosing among several
//! objects with a meld card's name, the melded permanent as a new object, its colors and
//! mana value, naming it, its color identity, and the Hanweir and Chittering Host meld
//! pairs' abilities.

use crate::r_s01_common::*;
use crate::r_s02_common::{can_activate, destroy, target_candidates};
use crate::r_s03_common::in_hand_with_mana;
use crate::r_s08_common::mana_value;
use crate::r_s07_common::graveyard_n;
use mtg_engine::card::{card, CardDef};
use mtg_engine::commander_rules::{color_identity, computed_color_identity};
use mtg_engine::decision::Answer;
use mtg_engine::deck::{check_commander, DeckProblem};
use mtg_engine::events::MoveCause;
use mtg_engine::merge;
use mtg_engine::object::{FaceState, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;
use std::sync::Arc;

const URZA: &str = "Urza, Lord Protector";
const STONES: &str = "The Mightstone and Weakstone";
const FANG: &str = "Fang, Fearless l'Cie";
const VANILLE: &str = "Vanille, Cheerful l'Cie";

/// Activates Urza, Lord Protector's "{7}: If you both own and control Urza, Lord Protector
/// and an artifact named The Mightstone and Weakstone, exile them, then meld them into
/// Urza, Planeswalker. Activate only as a sorcery." and resolves it.
fn activate_urza(t: &mut TestGame, urza: ObjectId) {
    t.lands(P0, "Wastes", 7);
    t.activate(P0, urza, 0, &[]).unwrap();
    t.resolve_all();
}

/// Melds Urza, Lord Protector and The Mightstone and Weakstone (owned by P0) with Urza's
/// ability, returning Urza, Planeswalker.
fn urza_planeswalker(t: &mut TestGame) -> ObjectId {
    let urza = t.battlefield(P0, URZA);
    t.battlefield(P0, STONES);
    activate_urza(t, urza);
    let pw = t.named_on_battlefield("Urza, Planeswalker");
    assert_eq!(pw.len(), 1);
    pw[0]
}

/// Vanille's "At the beginning of your first main phase, if you both own and control
/// Vanille and a creature named Fang, Fearless l'Cie, you may pay {3}{B}{G}. If you do,
/// exile them, then meld them into Ragnarok, Divine Deliverance.": advances P0 into the
/// first main phase with the mana, paying.
fn vanille_trigger(t: &mut TestGame) {
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.lands(P0, "Wastes", 3);
    t.answer_yes(P0, true);
    t.set_step(P0, Step::Draw);
    t.advance_to(P0, Step::PrecombatMain);
    t.resolve_all();
}

/// Melds Fang and Vanille (owned by P0) with Vanille's ability, returning Ragnarok.
fn ragnarok(t: &mut TestGame) -> ObjectId {
    t.battlefield(P0, FANG);
    t.battlefield(P0, VANILLE);
    vanille_trigger(t);
    let r = t.named_on_battlefield("Ragnarok, Divine Deliverance");
    assert_eq!(r.len(), 1, "{}", t.dump_log());
    r[0]
}

#[test]
fn cards_that_cant_be_melded_remain_in_exile() {
    cr!("701.42b", "701.42c");
    ruling!(
        "The Mightstone and Weakstone",
        "Tokens, cards that aren't meld cards, or meld cards that don't form a meld pair can't be melded. If an effect instructs a player to meld cards that can't be melded, those cards remain in exile."
    );
    supported(URZA);
    supported("Stolen Identity");
    supported("Sculpting Steel");
    // A token copy of The Mightstone and Weakstone (Stolen Identity: "Create a token
    // that's a copy of target artifact or creature."): both are exiled, nothing melds, and
    // Urza stays in exile.
    let mut t = TestGame::new(2);
    let urza = t.battlefield(P0, URZA);
    let theirs = t.battlefield(P1, STONES);
    let si = in_hand_with_mana(&mut t, P0, "Stolen Identity");
    t.cast(P0, si).target(theirs).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield(STONES).len(), 2);
    activate_urza(&mut t, urza);
    assert!(t.named_on_battlefield("Urza, Planeswalker").is_empty());
    assert_eq!(t.zone(urza), Zone::Exile);
    assert_eq!(t.named_on_battlefield(STONES), vec![theirs]);
    // Sculpting Steel as a copy of The Mightstone and Weakstone: a card that isn't a meld
    // card. Both cards remain in exile.
    let mut t = TestGame::new(2);
    let urza = t.battlefield(P0, URZA);
    let theirs = t.battlefield(P1, STONES);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(theirs)]);
    let steel = t.enter(P0, "Sculpting Steel");
    assert_eq!(t.obj(steel).chars.name, STONES);
    activate_urza(&mut t, urza);
    assert!(t.named_on_battlefield("Urza, Planeswalker").is_empty());
    assert_eq!(t.zone(urza), Zone::Exile);
    assert_eq!(t.zone(steel), Zone::Exile);
}

#[test]
fn with_several_objects_with_the_partners_name_the_player_chooses_one() {
    cr!("701.42a");
    ruling!(
        "The Mightstone and Weakstone",
        "If you control more than one object with one of those names, you select one object with that name to exile."
    );
    supported("Titania, Voice of Gaea");
    supported("Argoth, Sanctum of Nature");
    // Titania, Voice of Gaea: "At the beginning of your upkeep, if there are four or more
    // land cards in your graveyard and you both own and control Titania, Voice of Gaea and
    // a land named Argoth, Sanctum of Nature, exile them, then meld them into Titania,
    // Gaea Incarnate." P0 controls two Argoths (a nonlegendary land).
    for pick in [0usize, 1] {
        let mut t = TestGame::new(2);
        t.battlefield(P0, "Titania, Voice of Gaea");
        let argoths = [
            t.battlefield(P0, "Argoth, Sanctum of Nature"),
            t.battlefield(P0, "Argoth, Sanctum of Nature"),
        ];
        graveyard_n(&mut t, P0, "Forest", 4);
        t.answer_choose(P0, &[Entity::Object(argoths[pick])]);
        t.set_step(P1, Step::End);
        t.advance_to(P0, Step::Upkeep);
        t.resolve_all();
        assert_eq!(t.named_on_battlefield("Titania, Gaea Incarnate").len(), 1);
        // The chosen one was melded; the other one stays.
        assert_eq!(
            t.named_on_battlefield("Argoth, Sanctum of Nature"),
            vec![argoths[1 - pick]]
        );
    }
}

#[test]
fn with_several_objects_with_the_specified_name_the_player_chooses_one() {
    cr!("701.42a", "701.42c");
    ruling!(
        "Fang, Fearless l'Cie",
        "If you control more than one object with the specified name, you select one object with that name to exile."
    );
    supported(FANG);
    supported(VANILLE);
    supported("Quantum Misalignment");
    // Quantum Misalignment: "Create a token that's a copy of target creature you control,
    // except it isn't legendary." P0 controls Fang and a token named Fang.
    let setup = || {
        let mut t = TestGame::new(2);
        let fang = t.battlefield(P0, FANG);
        t.battlefield(P0, VANILLE);
        let qm = in_hand_with_mana(&mut t, P0, "Quantum Misalignment");
        t.cast(P0, qm).target(fang).go();
        t.resolve_all();
        let token = t
            .named_on_battlefield(FANG)
            .into_iter()
            .find(|x| *x != fang)
            .expect("token copy");
        (t, fang, token)
    };
    // Choosing the card: Ragnarok, and the token stays.
    let (mut t, fang, token) = setup();
    t.answer_choose(P0, &[Entity::Object(fang)]);
    vanille_trigger(&mut t);
    assert_eq!(t.named_on_battlefield("Ragnarok, Divine Deliverance").len(), 1);
    assert_eq!(t.named_on_battlefield(FANG), vec![token]);
    // Choosing the token: it and Vanille are exiled, and they can't be melded; the card
    // stays on the battlefield.
    let (mut t, fang, token) = setup();
    t.answer_choose(P0, &[Entity::Object(token)]);
    vanille_trigger(&mut t);
    assert!(t.named_on_battlefield("Ragnarok, Divine Deliverance").is_empty());
    assert_eq!(t.named_on_battlefield(FANG), vec![fang]);
    assert!(t.in_exile(VANILLE));
}

#[test]
fn a_meld_cards_commander_color_identity_is_its_front_faces() {
    cr!("903.4", "903.5c");
    ruling!(
        "The Mightstone and Weakstone",
        "In the Commander variant, a meld card's color identity is determined only by the mana costs and mana symbols in the rules text of its front face."
    );
    // The Mightstone and Weakstone melds into Urza, Planeswalker (white and blue), but is
    // colorless: it can be in the deck of a colorless commander.
    let deck = |commander: &str, card_name: &str| -> Vec<Arc<CardDef>> {
        let mut d = vec![card(commander), card(card_name)];
        d.extend((0..98).map(|_| card("Wastes")));
        d
    };
    assert!(color_identity(&card(STONES)).is_colorless());
    assert!(computed_color_identity(&card(STONES)).is_colorless());
    assert!(!color_identity(&card("Urza, Planeswalker")).is_colorless());
    let karn = card("Karn, Silver Golem");
    assert!(check_commander(&deck("Karn, Silver Golem", STONES), &karn, &[], false).is_empty());
    assert!(check_commander(&deck("Karn, Silver Golem", URZA), &karn, &[], false)
        .iter()
        .any(|p| matches!(p, DeckProblem::OutsideColorIdentity { .. })));
    // Phyrexian Dragon Engine (unearth {3}{R}{R}) is red only, though Mishra, Lost to
    // Phyrexia is black and red: it fits a mono-red commander's deck.
    let krenko = card("Krenko, Mob Boss");
    assert!(check_commander(
        &deck("Krenko, Mob Boss", "Phyrexian Dragon Engine"),
        &krenko,
        &[],
        false
    )
    .is_empty());
    assert_eq!(
        computed_color_identity(&card("Phyrexian Dragon Engine")),
        ColorSet::single(Color::Red)
    );
}

#[test]
fn a_player_may_name_the_combined_back_face() {
    cr!("712.4", "201.3");
    ruling!(
        "The Mightstone and Weakstone",
        "A player prompted to name a card may name the combined back face"
    );
    supported("Pithing Needle");
    // Pithing Needle: "As this artifact enters, choose a card name. Activated abilities of
    // sources with the chosen name can't be activated unless they're mana abilities."
    let mut t = TestGame::new(2);
    let pw = urza_planeswalker(&mut t);
    t.answer(P1, DecisionKind::Name, Answer::Text("Urza, Planeswalker".into()));
    let needle = t.enter(P1, "Pithing Needle");
    assert_eq!(
        t.obj(needle).choices.card_name.as_deref(),
        Some("Urza, Planeswalker")
    );
    // Its loyalty abilities can't be activated (they could without the Needle).
    t.set_step(P0, Step::PostcombatMain);
    assert!(!can_activate(&mut t, P0, pw));
    let mut t = TestGame::new(2);
    let pw = urza_planeswalker(&mut t);
    t.set_step(P0, Step::PostcombatMain);
    assert!(can_activate(&mut t, P0, pw));
}

#[test]
fn the_combined_back_face_has_a_color_indicator() {
    cr!("712.4", "204.1", "105.2");
    ruling!(
        "The Mightstone and Weakstone",
        "Note that the permanent represented by the combined back faces has a color indicator."
    );
    let mut t = TestGame::new(2);
    let pw = urza_planeswalker(&mut t);
    // Urza, Planeswalker has no mana cost; its color indicator makes it white and blue.
    let c = &t.obj(pw).chars;
    assert!(c.mana_cost.is_none());
    assert_eq!(c.colors, ColorSet::single(Color::White).union(ColorSet::single(Color::Blue)));
}

#[test]
fn the_combined_back_face_is_colorless_unless_it_has_a_color_indicator() {
    cr!("712.4", "204.1", "105.2c");
    ruling!(
        "Fang, Fearless l'Cie",
        "Note that the permanent represented by the combined back faces is colorless unless it has a color indicator."
    );
    // Ragnarok, Divine Deliverance has a black and green color indicator.
    let mut t = TestGame::new(2);
    let r = ragnarok(&mut t);
    let c = &t.obj(r).chars;
    assert!(c.mana_cost.is_none());
    assert_eq!(c.colors, ColorSet::single(Color::Black).union(ColorSet::single(Color::Green)));
    // Chittering Host has none: colorless.
    let rats = t.exile(P0, "Graf Rats");
    let scav = t.exile(P0, "Midnight Scavengers");
    let host = merge::meld(&mut t.g, rats, scav, card("Chittering Host"), P0).unwrap();
    t.g.recompute();
    assert!(t.obj(host).chars.colors.is_colorless());
}

#[test]
fn a_melded_permanent_is_a_new_object_without_the_cards_counters_auras_or_equipment() {
    cr!("701.42a", "400.7");
    ruling!(
        "The Mightstone and Weakstone",
        "When two cards are exiled and melded, they each leave the battlefield, then return together as one new object with no relation to either of the objects that left the battlefield."
    );
    supported("Holy Strength");
    supported("Bonesplitter");
    let mut t = TestGame::new(2);
    let urza = t.battlefield(P0, URZA);
    let stones = t.battlefield(P0, STONES);
    t.g.add_counters(Entity::Object(urza), counters::PLUS1, 2, None);
    t.g.add_counters(Entity::Object(stones), counters::CHARGE, 1, None);
    let aura = t.battlefield(P0, "Holy Strength");
    t.g.attach(aura, Entity::Object(urza));
    let sword = t.battlefield(P0, "Bonesplitter");
    t.g.attach(sword, Entity::Object(urza));
    t.g.recompute();
    assert_eq!(t.pt(urza), (7, 8));
    activate_urza(&mut t, urza);
    let pw = t.named_on_battlefield("Urza, Planeswalker")[0];
    // No +1/+1 or charge counters (only its starting loyalty), the Aura is put into the
    // graveyard, and the Equipment stays on the battlefield unattached.
    assert_eq!(t.counters(pw, counters::PLUS1), 0);
    assert_eq!(t.counters(pw, counters::CHARGE), 0);
    assert_eq!(t.counters(pw, counters::LOYALTY), 7);
    assert!(t.in_graveyard(P0, "Holy Strength"));
    assert!(t.on_battlefield(sword));
    assert_eq!(t.obj_now(sword).attached_to, None);
}

#[test]
fn a_melded_permanent_is_a_new_untapped_object() {
    cr!("701.42a", "400.7", "110.5b");
    ruling!(
        "Fang, Fearless l'Cie",
        "they each leave the battlefield, then return together as one new untapped object with no relation to either of the objects that left the battlefield"
    );
    let mut t = TestGame::new(2);
    let fang = t.battlefield(P0, FANG);
    t.battlefield(P0, VANILLE);
    t.g.objects[fang.0 as usize].tapped = true;
    t.g.add_counters(Entity::Object(fang), counters::PLUS1, 1, None);
    let aura = t.battlefield(P0, "Holy Strength");
    t.g.attach(aura, Entity::Object(fang));
    // Giant Growth on Fang until end of turn.
    t.lands(P0, "Forest", 1);
    let gg = t.hand(P0, "Giant Growth");
    t.cast(P0, gg).target(fang).go();
    t.resolve_all();
    vanille_trigger(&mut t);
    let r = t.named_on_battlefield("Ragnarok, Divine Deliverance");
    assert_eq!(r.len(), 1);
    let r = r[0];
    assert!(!t.obj(r).tapped);
    assert_eq!(t.counters(r, counters::PLUS1), 0);
    assert_eq!(t.pt(r), (7, 6));
    assert!(t.in_graveyard(P0, "Holy Strength"));
}

#[test]
fn a_melded_permanent_is_two_cards_when_it_leaves() {
    cr!("712.21", "712.21a", "701.42a");
    ruling!(
        "Fang, Fearless l'Cie",
        "If the melded permanent goes to your graveyard from the battlefield, both cards are put into your graveyard."
    );
    let mut t = TestGame::new(2);
    let r = ragnarok(&mut t);
    assert_eq!(merge::physical_components(&t.g, r).len(), 2);
    destroy(&mut t, r);
    t.resolve_all();
    assert!(t.in_graveyard(P0, FANG) && t.in_graveyard(P0, VANILLE));
    // Face up with their front faces.
    for id in t.g.player(P0).graveyard.clone() {
        if [FANG, VANILLE].contains(&t.obj(id).chars.name.as_str()) {
            assert_eq!(t.obj(id).face, FaceState::Front);
            assert!(!t.obj(id).face_down);
        }
    }
    // Put on top of a library: the owner orders them.
    let mut t = TestGame::new(2);
    let r = ragnarok(&mut t);
    t.answer(P0, DecisionKind::Order, Answer::Indices(vec![1, 0]));
    let from = t.asked().len();
    t.g.move_object(r, Zone::Library(P0), MoveCause::Effect, Some(P1));
    assert!(t.asked()[from..].iter().any(|(p, d)| *p == P0
        && matches!(d, mtg_engine::decision::Decision::Order { .. })));
    let lib = t.g.player(P0).library.clone();
    let top: Vec<&str> = lib[lib.len() - 2..]
        .iter()
        .map(|id| t.obj(*id).chars.name.as_str())
        .collect();
    assert!(top.contains(&FANG) && top.contains(&VANILLE));
}

#[test]
fn a_melded_permanents_mana_value_is_the_sum_and_a_copys_is_zero() {
    cr!("202.3a", "712.8g", "707.2");
    ruling!(
        "Fang, Fearless l'Cie",
        "A permanent that becomes a copy of a melded permanent has only the characteristics of that combined back face, and its mana value is 0."
    );
    supported("Clone");
    let mut t = TestGame::new(2);
    let r = ragnarok(&mut t);
    // {2}{B} + {3}{G}.
    assert_eq!(mana_value(&t, r), 7);
    // Clone: "You may have this creature enter as a copy of any creature on the
    // battlefield."
    t.answer_yes(P1, true);
    t.answer_choose(P1, &[Entity::Object(r)]);
    let clone = t.enter(P1, "Clone");
    let c = &t.obj(clone).chars;
    assert_eq!(c.name, "Ragnarok, Divine Deliverance");
    assert_eq!(t.pt(clone), (7, 6));
    assert_eq!(mana_value(&t, clone), 0);
}

#[test]
fn hanweir_garrisons_tokens_and_another_attack_trigger_resolve_in_either_order() {
    cr!("603.3b", "508.4");
    ruling!(
        "Hanweir Garrison",
        "you may have it resolve before or after that of Hanweir Garrison"
    );
    supported("Hanweir Garrison");
    supported("Hamlet Captain");
    // Hanweir Garrison: "Whenever this creature attacks, create two 1/1 red Human creature
    // tokens that are tapped and attacking." Hamlet Captain: "Whenever this creature
    // attacks or blocks, other Humans you control get +1/+1 until end of turn."
    let humans_pt = |order: Vec<usize>| {
        let mut t = TestGame::new(2);
        let garrison = t.battlefield(P0, "Hanweir Garrison");
        let captain = t.battlefield(P0, "Hamlet Captain");
        t.answer(P0, DecisionKind::Order, Answer::Indices(order));
        attack_with(
            &mut t,
            &[
                (garrison, Entity::Player(P1)),
                (captain, Entity::Player(P1)),
            ],
        );
        t.resolve_all();
        let tokens = tokens(&t, P0);
        assert_eq!(tokens.len(), 2);
        t.pt(tokens[0])
    };
    let a = humans_pt(vec![0, 1]);
    let b = humans_pt(vec![1, 0]);
    // Captain's ability resolving last gives the tokens +1/+1; first, it doesn't.
    let mut both = [a, b];
    both.sort();
    assert_eq!(both, [(1, 1), (2, 2)]);
}

#[test]
fn an_attack_triggers_targets_are_chosen_before_the_tokens_exist() {
    cr!("603.3d", "508.4");
    ruling!(
        "Hanweir Garrison",
        "If that ability has targets, it won’t be able to target the tokens."
    );
    supported("Hunter's Talent");
    // Hunter's Talent level 2: "Whenever you attack, target attacking creature gets +1/+0
    // and gains trample until end of turn."
    let mut t = TestGame::new(2);
    let garrison = t.battlefield(P0, "Hanweir Garrison");
    let talent = t.battlefield(P0, "Hunter's Talent");
    mtg_engine::classes::set_level(&mut t.g, talent, 2);
    let from = t.asked().len();
    attack_with(&mut t, &[(garrison, Entity::Player(P1))]);
    t.resolve_all();
    assert_eq!(tokens(&t, P0).len(), 2);
    let offered: Vec<Entity> = target_candidates(&t, P0, from).concat();
    assert!(offered.contains(&Entity::Object(garrison)));
    for tok in tokens(&t, P0) {
        assert!(!offered.contains(&Entity::Object(tok)));
    }
}

#[test]
fn each_token_may_attack_a_different_player() {
    cr!("508.4", "506.2");
    ruling!(
        "Hanweir Garrison",
        "the tokens don’t both have to attack the same player, planeswalker, or battle"
    );
    let mut t = TestGame::new(3);
    let garrison = t.battlefield(P0, "Hanweir Garrison");
    // The first token attacks P2, the second P1.
    t.answer_choose(P0, &[Entity::Player(P2)]);
    t.answer_choose(P0, &[Entity::Player(P1)]);
    attack_with(&mut t, &[(garrison, Entity::Player(P1))]);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 2);
    let combat = t.g.combat.as_ref().unwrap();
    let mut attacked: Vec<Option<Entity>> =
        toks.iter().map(|x| combat.attack_target(*x)).collect();
    attacked.sort();
    let mut expected = vec![Some(Entity::Player(P1)), Some(Entity::Player(P2))];
    expected.sort();
    assert_eq!(attacked, expected);
    assert!(toks.iter().all(|x| t.obj(*x).tapped));
}

#[test]
fn the_tokens_were_never_declared_as_attackers() {
    cr!("508.4", "508.3a");
    ruling!(
        "Hanweir Garrison",
        "Although the tokens created by the triggered ability are attacking, they were never declared as attacking creatures"
    );
    supported("Gleam of Battle");
    // Gleam of Battle: "Whenever a creature you control attacks, put a +1/+1 counter on
    // it." Only Hanweir Garrison attacked.
    let mut t = TestGame::new(2);
    let garrison = t.battlefield(P0, "Hanweir Garrison");
    t.battlefield(P0, "Gleam of Battle");
    attack_with(&mut t, &[(garrison, Entity::Player(P1))]);
    t.resolve_all();
    let toks = tokens(&t, P0);
    assert_eq!(toks.len(), 2);
    assert_eq!(t.counters(garrison, counters::PLUS1), 1);
    for tok in toks {
        assert_eq!(t.counters(tok, counters::PLUS1), 0);
        assert!(t.g.combat.as_ref().unwrap().attack_target(tok).is_some());
    }
}

#[test]
fn chittering_hosts_bonus_affects_only_the_creatures_there_as_it_resolves() {
    cr!("611.2c");
    ruling!(
        "Chittering Host",
        "Creatures you begin to control later in the turn won't get +1/+0 or gain menace."
    );
    supported("Graf Rats");
    supported("Midnight Scavengers");
    // Graf Rats: "At the beginning of combat on your turn, if you both own and control
    // this creature and a creature named Midnight Scavengers, exile them, then meld them
    // into Chittering Host." Chittering Host: "When this creature enters, other creatures
    // you control get +1/+0 and gain menace until end of turn."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Graf Rats");
    t.battlefield(P0, "Midnight Scavengers");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Chittering Host").len(), 1);
    assert_eq!(t.pt(bears), (3, 2));
    // A creature P0 begins to control later this turn: no bonus.
    let later = t.enter(P0, "Grizzly Bears");
    t.resolve_all();
    assert_eq!(t.pt(later), (2, 2));
    assert!(!t
        .obj(later)
        .chars
        .has_keyword(mtg_engine::keywords::KeywordKind::Menace));
}
