//! Rulings batch S13 — "partner with [name]" (CR 702.124j): a triggered ability, "When this
//! permanent enters, target player may search their library for a card named [name],
//! reveal it, put it into their hand, then shuffle", and a deck-construction rule letting
//! the two named legendary cards be a player's two commanders (CR 702.124a–d, 903.3).

use crate::r_s01_common::*;
use crate::r_s04_common::untapped_lands;
use crate::r_s13_common::*;
use mtg_engine::card::{card, CardDef};
use mtg_engine::deck::{check_constructed, DeckProblem};
use mtg_engine::decision::Answer;
use mtg_engine::events::Event;
use mtg_engine::game::{GameConfig, Variant};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::kw::partner::{
    can_be_commander, check_commander_deck, commanders_problem, ineligible_commander,
};
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;
use std::sync::Arc;

fn can_pair(a: &str, b: &str) -> bool {
    commanders_problem(&[&card(a), &card(b)]).is_none()
}

/// Whether `p` could begin casting `card` with `method` now (including paying for it).
fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId, method: CastMethod) -> bool {
    t.g.recompute();
    t.g.turn.priority = Some(p);
    let card = t.g.current(card);
    t.g.cast_options(p, card)
        .into_iter()
        .any(|o| o.method == method && t.g.can_begin_cast(p, card, &o))
}

/// The colors (as "WUBRG" letters) of the basic lands a 100-card Commander deck led by
/// `a` and `b` may contain.
fn identity_allows(a: &str, b: &str) -> String {
    let pair: Vec<Arc<CardDef>> = vec![card(a), card(b)];
    let mut out = String::new();
    for (land, c) in [
        ("Plains", 'W'),
        ("Island", 'U'),
        ("Swamp", 'B'),
        ("Mountain", 'R'),
        ("Forest", 'G'),
    ] {
        let mut d = pair.clone();
        d.push(card(land));
        d.extend((0..97).map(|_| card("Wastes")));
        if check_commander_deck(&d, &pair, &[], false).is_empty() {
            out.push(c);
        }
    }
    out
}

/// A legal 100-card Commander deck (the two commanders and 98 Wastes) for them.
fn legal_pair_deck(a: &str, b: &str) -> bool {
    let pair: Vec<Arc<CardDef>> = vec![card(a), card(b)];
    let mut d = pair.clone();
    d.extend((0..98).map(|_| card("Wastes")));
    check_commander_deck(&d, &pair, &[], false).is_empty()
}

/// `entering` enters under P0's control; its partner-with trigger targets `target`, who
/// answers `yes` to searching; `partner` is in `target`'s library under three other cards.
fn partner_trigger(entering: &str, partner: &str, target: PlayerId, yes: bool) -> TestGame {
    let mut t = TestGame::new(2);
    t.library_top(target, partner);
    for _ in 0..3 {
        t.library_top(target, "Island");
    }
    t.answer_targets(P0, &[Entity::Player(target)]);
    t.answer_yes(target, yes);
    t.enter(P0, entering);
    t.settle();
    assert_eq!(t.stack_len(), 1, "{entering}: its partner-with trigger");
    t.resolve_all();
    t
}

fn shuffled(t: &TestGame, p: PlayerId) -> bool {
    t.g.turn_events
        .iter()
        .any(|e| matches!(e, Event::Shuffled { player } if *player == p))
}

/// The names of the cards revealed this turn (CR 701.20a).
fn revealed_names(t: &TestGame) -> Vec<String> {
    t.g.turn_events
        .iter()
        .filter_map(|e| match e {
            Event::Custom { name, obj, .. } if name == mtg_engine::reveal::REVEALED => *obj,
            _ => None,
        })
        .map(|c| {
            let o = t.obj(c);
            o.card.as_ref().map_or(o.chars.name.to_string(), |d| d.name.to_string())
        })
        .collect()
}

#[test]
fn the_target_player_searches_and_reveals_the_card() {
    cr!("702.124j", "701.23a", "701.20a");
    ruling!(
        "Rocksteady, Mutant Marauder",
        "Note that the target player searches their library (which may be affected by effects such as that of Stranglehold) and that the card they find is revealed, even though these words aren't included in the ability's reminder text."
    );
    ruling!(
        "Okaun, Eye of Chaos",
        "Note that the target player searches their library (which may be affected by effects such as that of Stranglehold) and that the card they find is revealed, even though these words aren’t included in the ability’s reminder text."
    );
    supported("Rocksteady, Mutant Marauder");
    supported("Okaun, Eye of Chaos");
    supported("Stranglehold");
    // P1's Stranglehold ("Your opponents can't search libraries."): P0 can't search for
    // Bebop ...
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Stranglehold");
    t.library_top(P0, "Bebop, Skull & Crossbones");
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Rocksteady, Mutant Marauder");
    t.resolve_all();
    assert!(!t.in_hand(P0, "Bebop, Skull & Crossbones"));
    assert!(revealed_names(&t).is_empty());
    // ... but P1, the target player, isn't P1's opponent and can search their own library.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Stranglehold");
    t.library_top(P1, "Bebop, Skull & Crossbones");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P1, true);
    t.enter(P0, "Rocksteady, Mutant Marauder");
    t.resolve_all();
    assert!(t.in_hand(P1, "Bebop, Skull & Crossbones"));
    assert_eq!(revealed_names(&t), vec!["Bebop, Skull & Crossbones"]);
    // Okaun: the card found (Zndrsplt) is revealed.
    let t = partner_trigger("Okaun, Eye of Chaos", "Zndrsplt, Eye of Wisdom", P0, true);
    assert!(t.in_hand(P0, "Zndrsplt, Eye of Wisdom"));
    assert_eq!(revealed_names(&t), vec!["Zndrsplt, Eye of Wisdom"]);
    assert!(t.g.turn_events.iter().any(|e| matches!(e, Event::Searched { player } if *player == P0)));
}

#[test]
fn partner_with_is_an_enters_trigger_that_searches_for_the_partner() {
    cr!("702.124j", "603.6a", "701.23a", "701.24a");
    ruling!(
        "Pippin, Warden of Isengard",
        "\"Partner with [name]\" represents two abilities. The first is a triggered ability: \"When this permanent enters the battlefield, target player may search their library for a card named [name], reveal it, put it into their hand, then shuffle their library.\""
    );
    ruling!(
        "Sylvia Brightspear",
        "“Partner with [name]” represents two abilities. The first is a triggered ability: “When this permanent enters the battlefield, target player may search their library for a card named [name], reveal it, put it into their hand, then shuffle their library.”"
    );
    ruling!(
        "Madame Vastra",
        "\"Partner with [name]\" represents two abilities. The first is a triggered ability: \"When this permanent enters the battlefield, target player may search their library for a card named [name], reveal it, put it into their hand, then shuffle.\""
    );
    ruling!(
        "Bebop, Skull & Crossbones",
        "\"Partner with [name]\" represents two abilities. The first is a triggered ability: \"When this permanent enters, target player may search their library for a card named [name], reveal it, put it into their hand, then shuffle.\""
    );
    for (entering, partner) in [
        ("Pippin, Warden of Isengard", "Merry, Warden of Isengard"),
        ("Sylvia Brightspear", "Khorvath Brightflame"),
        ("Madame Vastra", "Jenny Flint"),
        ("Bebop, Skull & Crossbones", "Rocksteady, Mutant Marauder"),
    ] {
        supported(entering);
        // The target player may search: they find the partner, reveal it, put it into
        // their hand, and shuffle.
        let t = partner_trigger(entering, partner, P0, true);
        assert!(t.in_hand(P0, partner), "{entering}");
        assert_eq!(revealed_names(&t), vec![partner.to_string()]);
        assert!(shuffled(&t, P0));
        // ... or not ("may").
        let t = partner_trigger(entering, partner, P0, false);
        assert!(!t.in_hand(P0, partner), "{entering}");
        assert_eq!(t.library_size(P0), 30 + 4);
    }
}

#[test]
fn partner_with_still_triggers_in_a_commander_game() {
    cr!("702.124j", "903.3", "115.1");
    ruling!(
        "Merry, Warden of Isengard",
        "The triggered ability of the \"partner with\" keyword still triggers in a Commander game. If your other commander has somehow ended up in your library, you can find it. You can also target another player, whether or not they have that card in their library."
    );
    ruling!(
        "Okaun, Eye of Chaos",
        "The triggered ability of the “partner with” keyword still triggers in a Commander game. If your other commander has somehow ended up in your library, you can find it. You can also target another player who might have that card in their library."
    );
    supported("Merry, Warden of Isengard");
    supported("Okaun, Eye of Chaos");
    // Merry and Pippin are P0's commanders; Pippin ended up in P0's library.
    let mut t = commander_game();
    t.g.players[0].commander_names.push("Merry, Warden of Isengard".into());
    t.g.players[0].commander_names.push("Pippin, Warden of Isengard".into());
    let pippin = t.library_top(P0, "Pippin, Warden of Isengard");
    t.g.objects[pippin.0 as usize].is_commander = true;
    t.answer_targets(P0, &[Entity::Player(P0)]);
    t.answer_yes(P0, true);
    // (It stays in the hand rather than going to the command zone, CR 903.9b.)
    t.answer_yes(P0, false);
    let merry = t.enter(P0, "Merry, Warden of Isengard");
    t.g.objects[merry.0 as usize].is_commander = true;
    t.settle();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.in_hand(P0, "Pippin, Warden of Isengard"));
    // Targeting the opponent, who has no Pippin in their library: a legal target; they
    // search and find nothing.
    let mut t = commander_game();
    let from = t.asked().len();
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P1, true);
    t.enter(P0, "Merry, Warden of Isengard");
    t.resolve_all();
    let asked = asked_since(&t, from);
    assert!(asked
        .iter()
        .any(|(p, d)| *p == P1 && matches!(d, decision::Decision::YesNo { .. })));
    assert!(shuffled(&t, P1));
    assert_eq!(t.hand_size(P1), 0);
    // Okaun targeting the opponent, who has Zndrsplt in their library: they may find it.
    let mut t = commander_game();
    t.library_top(P1, "Zndrsplt, Eye of Wisdom");
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.answer_yes(P1, true);
    t.enter(P0, "Okaun, Eye of Chaos");
    t.resolve_all();
    assert!(t.in_hand(P1, "Zndrsplt, Eye of Wisdom"));
}

/// Plays out the rulings on two commanders being tracked separately (commander tax,
/// commander damage, Command Beacon) for the pair `a` / `b`, with `lands_a` / `lands_b`
/// paying their mana costs exactly.
fn tracked_separately(a: &str, lands_a: &[(&str, usize)], b: &str, lands_b: &[(&str, usize)]) {
    let add_lands = |t: &mut TestGame, lands: &[(&str, usize)]| {
        for (l, n) in lands {
            t.lands(P0, l, *n);
        }
    };
    // Commander tax: casting one doesn't make the other cost {2} more.
    let mut t = commander_game();
    let ca = commander(&mut t, P0, a);
    let cb = commander(&mut t, P0, b);
    add_lands(&mut t, lands_a);
    t.cast(P0, ca).go();
    t.resolve_all();
    assert!(t.on_battlefield(ca), "{a}");
    assert_eq!(untapped_lands(&t, P0), 0);
    t.answer_yes(P0, true);
    t.g.destroy(t.g.current(ca), None);
    t.settle();
    t.resolve_all();
    assert_eq!(t.zone(ca), Zone::Command);
    add_lands(&mut t, lands_b);
    t.cast(P0, cb).go();
    assert_eq!(untapped_lands(&t, P0), 0, "{b} cost no more");
    t.resolve_all();
    // The first one costs {2} more the second time.
    let ca = t.g.current(ca);
    add_lands(&mut t, lands_a);
    assert!(!castable(&mut t, P0, ca, CastMethod::Normal));
    t.lands(P0, "Wastes", 2);
    t.cast(P0, ca).go();
    assert_eq!(untapped_lands(&t, P0), 0);
    // Command Beacon: "{T}, Sacrifice this land: Put your commander into your hand from
    // the command zone." Only the chosen one.
    let mut t = commander_game();
    let ca = commander(&mut t, P0, a);
    let cb = commander(&mut t, P0, b);
    let beacon = t.battlefield(P0, "Command Beacon");
    t.answer_choose(P0, &[Entity::Object(cb)]);
    t.answer_yes(P0, false);
    t.activate(P0, beacon, 1, &[]).expect("Command Beacon");
    t.resolve_all();
    assert_eq!(t.zone(cb), Zone::Hand(P0));
    assert_eq!(t.zone(ca), Zone::Command);
    // Commander damage: 21 from one of them, not from both combined.
    let mut t = commander_game();
    t.g.players[1].life = 60;
    t.g.players[1].commander_damage.insert(a.into(), 20);
    let pa = t.battlefield(P0, a);
    t.g.objects[pa.0 as usize].is_commander = true;
    let pb = t.battlefield(P0, b);
    t.g.objects[pb.0 as usize].is_commander = true;
    attack_with(&mut t, &[(pb, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[]);
    t.settle();
    assert!(t.life(P1) < 60);
    assert!(!t.has_lost(P1), "damage from {b} doesn't add to {a}'s");
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::PrecombatMain);
    attack_with(&mut t, &[(pa, Entity::Player(P1))]);
    t.answer(P1, DecisionKind::Blockers, Answer::Blockers(vec![]));
    assert!(t.g.run_until(10_000, |g| g.result.is_some()));
    assert!(t.has_lost(P1));
    assert!(t.life(P1) > 0);
}

#[test]
fn two_commanders_are_tracked_separately() {
    cr!("702.124d", "903.8", "903.10a", "702.124e");
    ruling!(
        "Rocksteady, Mutant Marauder",
        "Once the game begins, your two commanders are tracked separately. If you cast one, you won't have to pay an additional {2} the first time you cast the other. A player loses the game after having been dealt 21 damage from one of them, not from both of them combined. Command Beacon's effect puts one into your hand from the command zone, not both."
    );
    ruling!(
        "Sylvia Brightspear",
        "Once the game begins, your two commanders are tracked separately. If you cast one, you won’t have to pay an additional {2} the first time you cast the other. A player loses the game after having been dealt 21 damage from one of them, not from both of them combined. Command Beacon’s effect puts one into your hand from the command zone, not both."
    );
    supported("Command Beacon");
    supported("Rocksteady, Mutant Marauder");
    supported("Bebop, Skull & Crossbones");
    supported("Sylvia Brightspear");
    supported("Khorvath Brightflame");
    // Rocksteady ({2}{G}) and Bebop ({1}{B}).
    tracked_separately(
        "Rocksteady, Mutant Marauder",
        &[("Forest", 1), ("Wastes", 2)],
        "Bebop, Skull & Crossbones",
        &[("Swamp", 1), ("Wastes", 1)],
    );
    // Sylvia ({2}{W}, double strike) and Khorvath ({5}{R}).
    tracked_separately(
        "Sylvia Brightspear",
        &[("Plains", 1), ("Wastes", 2)],
        "Khorvath Brightflame",
        &[("Mountain", 1), ("Wastes", 5)],
    );
}

#[test]
fn partner_with_lets_the_named_card_also_be_your_commander() {
    cr!("702.124a", "702.124j", "903.3");
    ruling!(
        "Brallin, Skyshark Rider",
        "The second ability represented by the \"partner with [name]\" keyword modifies the rules for deck construction in the Commander variant and has no function outside of that variant. If a legendary creature card with \"partner with [name]\" is designated as your commander, the named legendary creature card can also be designated as your commander."
    );
    ruling!(
        "Virtus the Veiled",
        "If a legendary creature card with \"partner with [name]\" is designated as your commander, the named legendary creature card can also be designated as your commander. For more information on the Commander variant, please visit Wizards.com/Commander."
    );
    ruling!(
        "Khorvath Brightflame",
        "The second ability represented by the “partner with [name]” keyword modifies the rules for deck construction in the Commander variant and has no function outside of that variant. If a legendary creature card with “partner with [name]” is designated as your commander, the named legendary creature card can also be designated as your commander."
    );
    ruling!(
        "Evie Frye",
        "The second ability represented by the “partner with [name]” keyword modifies the rules for deck construction in the Commander variant and has no function outside of that variant."
    );
    for (a, b) in [
        ("Brallin, Skyshark Rider", "Shabraz, the Skyshark"),
        ("Virtus the Veiled", "Gorm the Great"),
        ("Khorvath Brightflame", "Sylvia Brightspear"),
        ("Evie Frye", "Jacob Frye"),
    ] {
        // Either can be designated along with the other.
        assert!(can_pair(a, b) && can_pair(b, a), "{a} and {b}");
        assert!(legal_pair_deck(a, b), "{a} and {b}");
        // Not with another legendary creature.
        assert!(!can_pair(a, "Grizzly Bears"));
        assert!(!can_pair(a, "Isamaru, Hound of Konda"), "{a}");
        // Outside Commander, it doesn't matter: a 60-card deck with one of them but not
        // the other is fine.
        let mut d = vec![card(a)];
        d.extend((0..59).map(|_| card("Wastes")));
        assert!(check_constructed(&d).is_empty(), "{a}");
    }
    supported("Evie Frye");
}

#[test]
fn a_partner_with_creature_partners_only_with_its_partner_and_keeps_being_a_commander() {
    cr!("702.124a", "702.124j", "903.3", "903.9a", "613.1f");
    ruling!(
        "Pippin, Warden of Isengard",
        "To have two commanders, both must have the partner ability or corresponding \"partner with\" abilities as the game begins. A creature with a \"partner with\" ability can't partner with any creature other than its designated partner. Losing a partner ability during the game doesn't cause either to cease to be your commander."
    );
    ruling!(
        "Cazur, Ruthless Stalker",
        "A creature with a \"partner with\" ability can't partner with any creature other than its designated partner. Losing a partner ability during the game doesn't cause either to cease to be your commander."
    );
    ruling!(
        "Okaun, Eye of Chaos",
        "To have two commanders, both must have the partner ability (featured in the Magic: The Gathering—Commander™ (2016 Edition) set) or corresponding “partner with” abilities as the game begins. A creature with a “partner with” ability can’t partner with any creature other than its designated partner. Losing a partner ability during the game doesn’t cause either to cease to be your commander."
    );
    ruling!(
        "Evie Frye",
        "To have two commanders, both must have the partner ability or corresponding “partner with” abilities as the game begins. A creature with a “partner with” ability can’t partner with any creature other than its designated partner. Losing a partner ability during the game doesn’t cause either to cease to be your commander."
    );
    supported("Humility");
    for (a, b, other) in [
        (
            "Pippin, Warden of Isengard",
            "Merry, Warden of Isengard",
            "Frodo, Adventurous Hobbit",
        ),
        (
            "Cazur, Ruthless Stalker",
            "Ukkima, Stalking Shadow",
            "Brallin, Skyshark Rider",
        ),
        (
            "Okaun, Eye of Chaos",
            "Zndrsplt, Eye of Wisdom",
            "Sylvia Brightspear",
        ),
        ("Evie Frye", "Jacob Frye", "Rocksteady, Mutant Marauder"),
    ] {
        supported(a);
        // Only its designated partner: not another "partner with" creature, nor a creature
        // with partner.
        assert!(can_pair(a, b), "{a}");
        assert!(!can_pair(a, other), "{a} and {other}");
        assert!(!can_pair(a, "Kraum, Ludevic's Opus"), "{a}");
        // Designated as the game begins, both start in the command zone as commanders.
        let mut lib: Vec<Arc<CardDef>> = vec![card(a), card(b)];
        lib.extend((0..28).map(|_| card("Wastes")));
        let other_deck: Vec<Arc<CardDef>> = (0..30).map(|_| card("Wastes")).collect();
        let mut t = pregame(
            GameConfig {
                variant: Variant::Commander,
                skip_mulligans: true,
                ..Default::default()
            },
            vec![lib, other_deck],
        );
        assert!(t.g.designate_commander(P0, a));
        assert!(t.g.designate_commander(P0, b));
        t.g.start();
        for c in [a, b] {
            let ids = t.g.find_in_zone(Zone::Command, c);
            assert_eq!(ids.len(), 1, "{c} in the command zone");
            assert!(t.g.obj(ids[0]).is_commander);
        }
    }
    // Pippin loses its partner ability (Humility: "All creatures lose all abilities and
    // have base power and toughness 1/1.") but is still a commander: it can go to the
    // command zone, and it's taxed as a commander.
    let mut t = commander_game();
    let pippin = commander(&mut t, P0, "Pippin, Warden of Isengard");
    commander(&mut t, P0, "Merry, Warden of Isengard");
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    t.cast(P0, pippin).go();
    t.resolve_all();
    t.battlefield(P1, "Humility");
    t.g.recompute();
    let now = t.g.current(pippin);
    assert!(!t.obj(now).chars.has_keyword(KeywordKind::Partner));
    assert!(t.obj(now).is_commander);
    t.answer_yes(P0, true);
    t.g.destroy(now, None);
    t.settle();
    assert_eq!(t.zone(pippin), Zone::Command);
    assert!(t.obj_now(pippin).is_commander);
    t.lands(P0, "Swamp", 1);
    t.lands(P0, "Forest", 1);
    assert!(!castable(&mut t, P0, pippin, CastMethod::Normal));
    t.lands(P0, "Wastes", 2);
    assert!(castable(&mut t, P0, pippin, CastMethod::Normal));
}

#[test]
fn two_commanders_combined_color_identity() {
    cr!("702.124c", "903.4", "903.5c");
    ruling!(
        "Toothy, Imaginary Friend",
        "If Khorvath and Sylvia are your commanders, your deck may contain cards with red and/or white in their color identity, but not blue, black, or green."
    );
    ruling!(
        "Sylvia Brightspear",
        "If your Commander deck has two commanders, you can only include cards whose own color identities are also found in your commanders’ combined color identities. If Khorvath and Sylvia are your commanders"
    );
    ruling!(
        "Brallin, Skyshark Rider",
        "If Haldan and Pako are your commanders, your deck may contain cards with blue, red, and/or green in their color identity, but not cards with white or black."
    );
    ruling!(
        "Merry, Warden of Isengard",
        "If Frodo, Adventurous Hobbit and Sam, Loyal Attendant are your commanders, your deck may contain cards with white, black, and/or green in their color identity, but not blue or red."
    );
    ruling!(
        "Evie Frye",
        "If Evie Frye and Jacob Frye are your commanders, your deck may contain cards with blue and/or black in their color identity, but not white, red, or green."
    );
    ruling!(
        "Alphinaud Leveilleur",
        "If Alisaie Leveilleur and Alphinaud Leveilleur are your commanders, your deck may contain cards with white and/or blue in their color identity, but not black, red, or green."
    );
    ruling!(
        "Owen Grady, Raptor Trainer",
        "If Blue, Loyal Raptor and Owen Grady, Raptor Trainer are your commanders, your deck may contain cards with green, blue, and/or red in their color identity, but not white or black."
    );
    ruling!(
        "Bebop, Skull & Crossbones",
        "If Bebop, Skull and Crossbones and Rocksteady, Mutant Marauder are your commanders, your deck may contain cards with black and/or green in their color identities, but not white, blue, or red."
    );
    supported("Owen Grady, Raptor Trainer");
    supported("Evie Frye");
    let pairs: [(&str, &str, &str); 7] = [
        ("Khorvath Brightflame", "Sylvia Brightspear", "WR"),
        ("Haldan, Avid Arcanist", "Pako, Arcane Retriever", "URG"),
        (
            "Frodo, Adventurous Hobbit",
            "Sam, Loyal Attendant",
            "WBG",
        ),
        ("Evie Frye", "Jacob Frye", "UB"),
        ("Alisaie Leveilleur", "Alphinaud Leveilleur", "WU"),
        (
            "Blue, Loyal Raptor",
            "Owen Grady, Raptor Trainer",
            "URG",
        ),
        (
            "Bebop, Skull & Crossbones",
            "Rocksteady, Mutant Marauder",
            "BG",
        ),
    ];
    for (a, b, allowed) in pairs {
        assert!(can_pair(a, b), "{a} and {b}");
        assert_eq!(identity_allows(a, b), allowed, "{a} and {b}");
    }
    // "and/or": a card of both colors, or of just one of them.
    let pair = vec![card("Khorvath Brightflame"), card("Sylvia Brightspear")];
    let mut d = pair.clone();
    d.push(card("Lightning Helix"));
    d.push(card("Lightning Bolt"));
    d.extend((0..96).map(|_| card("Wastes")));
    assert!(check_commander_deck(&d, &pair, &[], false).is_empty());
    d.pop();
    d.push(card("Counterspell"));
    assert!(check_commander_deck(&d, &pair, &[], false)
        .contains(&DeckProblem::OutsideColorIdentity { name: "Counterspell".into() }));
}

#[test]
fn a_nonlegendary_partner_with_creature_cant_be_your_commander() {
    cr!("903.3", "702.124j");
    ruling!(
        "Chakram Retriever",
        "A nonlegendary creature can’t be your commander, even if it has a “partner with” ability."
    );
    ruling!(
        "Chakram Slinger",
        "A nonlegendary creature can't be your commander, even if it has a \"partner with\" ability."
    );
    supported("Chakram Retriever");
    supported("Chakram Slinger");
    for c in ["Chakram Retriever", "Chakram Slinger"] {
        assert!(!can_be_commander(&card(c), false), "{c}");
        assert!(card(c).front().chars.has_keyword(KeywordKind::Partner));
    }
    let pair = [card("Chakram Retriever"), card("Chakram Slinger")];
    assert!(!can_pair("Chakram Retriever", "Chakram Slinger"));
    assert!(ineligible_commander(&[&pair[0], &pair[1]], false).is_some());
    let mut d = pair.to_vec();
    d.extend((0..98).map(|_| card("Wastes")));
    assert!(check_commander_deck(&d, &pair, &[], false)
        .iter()
        .any(|p| matches!(p, DeckProblem::InvalidCommanders { .. })));
    // Nor alone.
    let one = [card("Chakram Slinger")];
    let mut d = one.to_vec();
    d.extend((0..99).map(|_| card("Wastes")));
    assert!(!check_commander_deck(&d, &one, &[], false).is_empty());
}
