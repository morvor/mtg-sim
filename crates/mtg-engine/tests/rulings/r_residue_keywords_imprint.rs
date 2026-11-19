//! Item `rulings-residue-keywords` — imprint and delve interactions: linked abilities and
//! the cards "exiled with" an object (CR 607.2a, 607.2b, 406.6), cards that leave exile
//! before an ability resolves, face-down exile (406.3), several imprinted cards, tokens
//! that copy a card rather than casting it (111.1, 707.2), and abilities granted by cards
//! exiled with delve (702.66).

use crate::r_residue_keywords_common::*;
use mtg_engine::decision::Decision;
use mtg_engine::events::Event;
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::CardType;
use mtg_engine::*;

use crate::r_s06_common::activate_containing;

/// P0's Death-Mask Duplicant with the creature cards `names` (from P0's graveyard)
/// imprinted, one activation each.
fn duplicant(t: &mut TestGame, names: &[&str]) -> ObjectId {
    supported("Death-Mask Duplicant");
    let d = t.battlefield(P0, "Death-Mask Duplicant");
    for n in names {
        let c = t.graveyard(P0, n);
        t.lands(P0, "Wastes", 1);
        t.activate(P0, d, 0, &[obj(c)]).unwrap();
        t.resolve_all();
        assert_eq!(t.zone(c), Zone::Exile);
    }
    d
}

#[test]
fn death_mask_duplicant_gains_every_landwalk_and_protection() {
    cr!("702.14a", "702.16a", "702.16b", "607.2a");
    ruling!(
        "Death-Mask Duplicant",
        "Death-Mask Duplicant gains all landwalk and protection abilities of the imprinted creature cards. If Death-Mask Duplicant imprints a card with protection from red and a card with protection from creatures, it has protection from red and from creatures."
    );
    let mut t = TestGame::new(2);
    let d = duplicant(
        &mut t,
        &["Paladin en-Vec", "Beloved Chaplain", "Bog Wraith"],
    );
    t.g.recompute();
    let o = t.obj_now(d);
    // Protection from black, from red, and from creatures; swampwalk; first strike.
    assert!(o.chars.keyword_count(KeywordKind::Protection) >= 3);
    assert!(o.has_keyword(KeywordKind::Landwalk));
    assert!(o.has_keyword(KeywordKind::FirstStrike));
    // It can't be the target of a red spell.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    let from = t.asked().len();
    t.cast(P1, bolt).target(Entity::Player(P0)).go();
    let offered = crate::r_s02_common::target_candidates(&t, P1, from);
    assert!(!offered.is_empty() && !offered[0].contains(&obj(d)));
    t.resolve_all();
    // It can't be blocked by creatures.
    let blocker = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P0, Step::PrecombatMain);
    attack_with(&mut t, &[(d, Entity::Player(P1))]);
    block_and_finish(&mut t, P1, &[(blocker, d)]);
    assert_eq!(t.life(P1), 15);
}

/// P0's Spellweaver Helix with Divination and Concentrate (from P1's graveyard) imprinted.
fn helix(t: &mut TestGame) -> (ObjectId, ObjectId) {
    supported("Spellweaver Helix");
    let a = t.graveyard(P1, "Divination");
    let b = t.graveyard(P1, "Concentrate");
    t.answer_targets(P0, &[obj(a), obj(b)]);
    t.answer_yes(P0, true);
    t.enter(P0, "Spellweaver Helix");
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    (a, b)
}

#[test]
fn spellweaver_helix_copies_in_exile_and_casts_the_copy_for_free() {
    cr!("707.12", "601.2", "118.9");
    ruling!(
        "Spellweaver Helix",
        "Spellweaver Helix’s second ability creates a copy of the imprinted card in the Exile zone (that’s where the imprinted sorcery card is), then allows you to cast it without paying its mana cost."
    );
    let mut t = TestGame::new(2);
    let (_, b) = helix(&mut t);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Divination", &[]);
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve();
    // The copy of Concentrate was cast (no mana: P0 has no untapped land).
    assert_eq!(t.stack_len(), 2, "{}", t.dump_log());
    assert_eq!(crate::r_s01_common::tapped_lands(&t, P0), 3);
    let top = *t.g.stack.last().unwrap();
    assert_eq!(name_now(&t, top), "Concentrate");
    assert_ne!(top, t.g.current(b));
    assert_eq!(t.zone(b), Zone::Exile);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 3 + 2);
}

#[test]
fn spellweaver_helix_makes_no_copy_if_the_card_left_exile() {
    cr!("608.2h", "707.12", "400.7");
    ruling!(
        "Spellweaver Helix",
        "If the imprinted card leaves the exile zone while the ability to make a copy is on the stack, then no copy will be made."
    );
    let mut t = TestGame::new(2);
    let (_, b) = helix(&mut t);
    let hand = t.hand_size(P0);
    cast_new(&mut t, P0, "Divination", &[]);
    t.settle();
    assert_eq!(t.stack_len(), 2);
    move_to(&mut t, b, Zone::Graveyard(P1));
    t.answer_yes(P0, true);
    t.answer_yes(P0, true);
    t.resolve();
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

/// P0's Dermotaxi, which imprinted `card` as it entered.
fn dermotaxi(t: &mut TestGame, card: Option<ObjectId>) -> ObjectId {
    supported("Dermotaxi");
    if let Some(c) = card {
        t.answer_choose(P0, &[obj(c)]);
    }
    let taxi = t.enter(P0, "Dermotaxi");
    t.settle();
    t.clear_answers();
    t.g.current(taxi)
}

/// P0 taps two fresh Grizzly Bears for Dermotaxi's ability and resolves it.
fn taxi_ability(t: &mut TestGame, taxi: ObjectId) {
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_choose(P0, &[obj(a), obj(b)]);
    activate_containing(t, P0, taxi, "Tap two untapped").unwrap();
    t.resolve_all();
}

#[test]
fn dermotaxi_with_nothing_exiled_does_nothing() {
    cr!("607.2a", "608.2h");
    ruling!(
        "Dermotaxi",
        "If you activate the ability, but there's no exiled card to reference (perhaps because there wasn't one in any graveyard to imprint), the ability does nothing. Dermotaxi won't become a copy of anything."
    );
    let mut t = TestGame::new(2);
    let taxi = dermotaxi(&mut t, None);
    taxi_ability(&mut t, taxi);
    assert_eq!(name_now(&t, taxi), "Dermotaxi");
    t.g.recompute();
    assert!(!t.obj_now(taxi).is(CardType::Creature));
}

#[test]
fn dermotaxi_refers_only_to_the_card_its_imprint_ability_exiled() {
    cr!("607.2a", "607.2b", "707.2");
    ruling!(
        "Dermotaxi",
        "The imprint ability is linked to the activated ability that makes Dermotaxi a creature. This means that the second ability refers only to the card exiled by the first, even if it becomes a copy of something that later exiles other cards."
    );
    let mut t = TestGame::new(2);
    let dm = t.graveyard(P1, "Death-Mask Duplicant");
    let taxi = dermotaxi(&mut t, Some(dm));
    taxi_ability(&mut t, taxi);
    assert_eq!(name_now(&t, taxi), "Death-Mask Duplicant");
    // As a Duplicant, it exiles Wind Drake with its own imprint ability: it has flying.
    let drake = t.graveyard(P0, "Wind Drake");
    t.lands(P0, "Wastes", 1);
    t.activate(P0, taxi, 0, &[obj(drake)]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(drake), Zone::Exile);
    assert!(has(&mut t, taxi, KeywordKind::Flying));
    // Next turn, the Dermotaxi ability again copies the Duplicant card, not Wind Drake.
    t.advance_to(P1, Step::Upkeep);
    assert_eq!(name_now(&t, taxi), "Dermotaxi");
    t.set_step(P0, Step::PrecombatMain);
    taxi_ability(&mut t, taxi);
    assert_eq!(name_now(&t, taxi), "Death-Mask Duplicant");
}

#[test]
fn ugins_labyrinth_returns_every_card_exiled_with_it() {
    cr!("607.2a", "603.2c", "406.6");
    ruling!(
        "Ugin's Labyrinth",
        "In the rare case where more than one card is exiled with Ugin's Labyrinth's imprint ability (likely because the triggered ability was copied or the ability triggered a second time), its last ability will return all such cards to their owners' hands."
    );
    supported("Ugin's Labyrinth");
    let mut t = TestGame::new(2);
    // Yarok: the Labyrinth's enter trigger triggers an additional time.
    t.battlefield(P0, "Yarok, the Desecrated");
    let a = t.hand(P0, "Ulamog's Crusher");
    let b = t.hand(P0, "Artisan of Kozilek");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(a)]);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(b)]);
    let lab = t.enter(P0, "Ugin's Labyrinth");
    t.settle();
    assert_eq!(t.stack_len(), 2);
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Exile);
    assert_eq!(t.zone(b), Zone::Exile);
    activate_containing(&mut t, P0, lab, "Return").unwrap();
    t.resolve_all();
    assert_eq!(t.zone(a), Zone::Hand(P0));
    assert_eq!(t.zone(b), Zone::Hand(P0));
}

/// P0's Soul Foundry with `name` (from P0's hand) imprinted; returns (foundry, card).
fn foundry(t: &mut TestGame, name: &str) -> (ObjectId, ObjectId) {
    supported("Soul Foundry");
    let c = t.hand(P0, name);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(c)]);
    let f = t.enter(P0, "Soul Foundry");
    t.resolve_all();
    assert_eq!(t.zone(c), Zone::Exile);
    (t.g.current(f), c)
}

/// P0 activates Soul Foundry ({X}, {T}, X = the mana value of the exiled card).
fn foundry_token(t: &mut TestGame, f: ObjectId, mv: usize) {
    t.lands(P0, "Wastes", mv);
    t.answer(
        P0,
        DecisionKind::X,
        mtg_engine::decision::Answer::Number(mv as i64),
    );
    t.activate(P0, f, 0, &[]).unwrap();
    t.resolve_all();
}

#[test]
fn soul_foundry_token_isnt_cast_and_pays_no_optional_cost() {
    cr!("111.1", "707.2", "601.1", "702.33d");
    ruling!(
        "Soul Foundry",
        "Soul Foundry puts a token copy of the imprinted card onto the battlefield. The token is put onto the battlefield, not cast."
    );
    ruling!(
        "Soul Foundry",
        "You do not get a chance to pay any optional costs, such as Kicker, on the imprinted card."
    );
    let mut t = TestGame::new(2);
    // Beast Whisperer: "Whenever you cast a creature spell, draw a card."
    t.battlefield(P0, "Beast Whisperer");
    // Prison Barricade: "Kicker {1}{W} ... If this creature was kicked, it enters with a
    // +1/+1 counter on it ..."
    let (f, _) = foundry(&mut t, "Prison Barricade");
    let hand = t.hand_size(P0);
    let from = t.asked().len();
    foundry_token(&mut t, f, 2);
    let tok = t.named_on_battlefield("Prison Barricade")[0];
    assert!(t.obj_now(tok).is_token());
    assert_eq!(p1(&t, tok), 0);
    assert_eq!(t.hand_size(P0), hand);
    assert!(!was_asked(&t, from, |d| matches!(
        d,
        Decision::OptionalCost { .. }
    )));
    assert!(!t
        .g
        .turn_events
        .iter()
        .any(|e| matches!(e, Event::SpellCast { .. })));
}

#[test]
fn summoners_egg_imprints_face_down_where_others_imprint_face_up() {
    cr!("406.3", "406.6", "607.2a");
    ruling!(
        "Summoner's Egg",
        "The card is imprinted face down. This means that other players don’t know what the card is and you can’t look at the card once it’s imprinted (until it’s turned face up, of course). Note that effects that exile cards, including all previous imprint cards, exile those cards face up unless the effect says otherwise."
    );
    supported("Summoner's Egg");
    let mut t = TestGame::new(2);
    let c = t.hand(P0, "Hill Giant");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(c)]);
    let egg = t.enter(P0, "Summoner's Egg");
    t.resolve_all();
    let ex = t.g.current(c);
    assert_eq!(t.zone(ex), Zone::Exile);
    assert!(t.obj_now(ex).face_down);
    // Neither player may look at it, its owner included.
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P0, ex));
    assert!(!mtg_engine::facedown::can_look_at(&t.g, P1, ex));
    // When the Egg dies, the card is turned face up and put onto the battlefield.
    destroy(&mut t, egg);
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 1);
    // Other imprint cards exile face up: Soul Foundry's card is seen by both players.
    let mut t = TestGame::new(2);
    let (_, card) = foundry(&mut t, "Grizzly Bears");
    let ex = t.g.current(card);
    assert!(!t.obj_now(ex).face_down);
    assert!(mtg_engine::facedown::can_look_at(&t.g, P1, ex));
}

#[test]
fn soulflayer_ignores_keywords_a_card_grants_itself() {
    cr!("702.66a", "113.6", "611.3a");
    ruling!(
        "Soulflayer",
        "Creature cards that have abilities that grant the listed keyword abilities to themselves won’t count. For example, exiling a Battle Brawler (a creature with “As long as you control a red or white permanent, Battle Brawler gets +1/+0 and has first strike”) with Soulflayer’s delve ability won’t cause Soulflayer to have first strike, even if you control a red or white permanent."
    );
    supported("Soulflayer");
    let mut t = TestGame::new(2);
    // A red permanent: Battle Brawler's condition holds.
    t.battlefield(P0, "Mountain");
    let brawler = t.graveyard(P0, "Battle Brawler");
    let drake = t.graveyard(P0, "Wind Drake");
    t.lands(P0, "Swamp", 4);
    let s = t.hand(P0, "Soulflayer");
    t.answer_choose(P0, &[obj(brawler), obj(drake)]);
    t.cast(P0, s).go();
    t.resolve_all();
    assert_eq!(t.zone(brawler), Zone::Exile);
    let s = t.named_on_battlefield("Soulflayer")[0];
    assert!(has(&mut t, s, KeywordKind::Flying));
    assert!(!has(&mut t, s, KeywordKind::FirstStrike));
}

#[test]
fn ethereal_forager_finds_its_delved_cards_after_it_left() {
    cr!("607.2a", "702.66a", "603.10a");
    ruling!(
        "Ethereal Forager",
        "If Ethereal Forager leaves the battlefield while its triggered ability is on the stack, that ability can still find the cards exiled with Ethereal Forager’s delve ability."
    );
    supported("Ethereal Forager");
    let mut t = TestGame::new(2);
    let opt = t.graveyard(P0, "Opt");
    for _ in 0..3 {
        t.graveyard(P0, "Grizzly Bears");
    }
    t.lands(P0, "Island", 2);
    let f = t.hand(P0, "Ethereal Forager");
    t.cast(P0, f).go();
    t.resolve_all();
    assert_eq!(t.zone(opt), Zone::Exile);
    let f = t.named_on_battlefield("Ethereal Forager")[0];
    t.g.objects[f.0 as usize].summoning_sick = false;
    t.answer_yes(P0, true);
    attack_with(&mut t, &[(f, Entity::Player(P1))]);
    assert_eq!(t.stack_len(), 1);
    move_to(&mut t, f, Zone::Hand(P0));
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.in_hand(P0, "Opt"), "{}", t.dump_log());
}
