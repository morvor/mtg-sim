//! Rulings batch P110 — Saga chapters with optional targets (CR 714.2b, 603.3d, 608.2b),
//! values computed as an ability resolves (CR 608.2h), and "exile the top card ... you may
//! play that card": the card is played following the normal timing rules and paying its
//! costs (CR 601.2, 305.1, 305.2, 116.2a), even after the source is gone (CR 610.3).

use crate::r_p110_common::*;
use crate::r_s02_common::can_play_land;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::counters;
use mtg_engine::*;

/// Puts P0's Saga `name` onto the battlefield with `lore - 1` lore counters and adds one:
/// chapter `lore` triggers.
fn chapter(t: &mut TestGame, name: &str, lore: u32) -> ObjectId {
    supported(name);
    let saga = t.battlefield(P0, name);
    t.g.objects[saga.0 as usize]
        .counters
        .insert(counters::LORE.into(), lore - 1);
    add_lore(t, saga, 1);
    saga
}

/// Whether `p` could cast the card (followed across zone changes) now, any way.
fn castable(t: &mut TestGame, p: PlayerId, card: ObjectId) -> bool {
    t.g.turn.priority = Some(p);
    t.g.recompute();
    let card = t.g.current(card);
    t.g.legal_actions(p)
        .iter()
        .any(|a| matches!(a, mtg_engine::decision::Action::Cast { card: c, .. } if *c == card))
}

/// The checks shared by the "you may play that card" rulings, for a land card `land` and
/// a creature card `bears` (Grizzly Bears) exiled with that permission on P0's main phase
/// with the stack empty: the land only with a land play left and the stack empty, the
/// creature only by paying for it and at sorcery speed.
fn plays_follow_the_normal_rules(t: &mut TestGame, land: ObjectId, bears: ObjectId, what: &str) {
    assert_eq!(t.zone(land), Zone::Exile, "{what}");
    assert_eq!(t.zone(bears), Zone::Exile, "{what}");
    assert!(t.g.stack.is_empty(), "{what}");
    // No mana: the Bears can't be cast.
    assert!(!castable(t, P0, bears), "{what}: its cost must be paid");
    // With a spell on the stack, neither can be played.
    t.lands(P1, "Mountain", 1);
    cast_new(t, P1, "Shock", &[Entity::Player(P1)]);
    t.lands(P0, "Forest", 2);
    assert!(!can_play_land(t, P0, land), "{what}: the stack isn't empty");
    assert!(!castable(t, P0, bears), "{what}: sorcery timing");
    t.resolve_all();
    // During combat, neither.
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(!can_play_land(t, P0, land), "{what}: not a main phase");
    assert!(!castable(t, P0, bears), "{what}: not a main phase");
    t.set_step(P0, Step::PostcombatMain);
    assert!(castable(t, P0, bears), "{what}");
    // Having already played a land, the land can't be.
    t.g.players[0].lands_played_this_turn = 1;
    assert!(!can_play_land(t, P0, land), "{what}: no land play left");
    t.g.players[0].lands_played_this_turn = 0;
    assert!(can_play_land(t, P0, land), "{what}");
    t.play_land(P0, t.g.current(land)).unwrap();
    t.cast(P0, t.g.current(bears)).go();
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1, "{what}");
}

#[test]
fn the_flux_plays_follow_the_normal_rules() {
    cr!("305.2", "305.1", "307.1", "601.2f");
    ruling!(
        "The Flux",
        "You must pay all costs and follow all normal timing rules to play a card with The Flux's middle chapter abilities. For example, you can play a land exiled this way only during your main phase while the stack is empty."
    );
    let mut t = TestGame::new(2);
    let land = t.library_top(P0, "Forest");
    chapter(&mut t, "The Flux", 2);
    // While the chapter ability is still on the stack, nothing is exiled yet.
    t.resolve_all();
    let bears = t.library_top(P0, "Grizzly Bears");
    let saga = t.named_on_battlefield("The Flux")[0];
    add_lore(&mut t, saga, 1);
    t.resolve_all();
    plays_follow_the_normal_rules(&mut t, land, bears, "The Flux");
}

#[test]
fn zukos_plays_follow_the_normal_rules() {
    cr!("305.2", "307.1", "601.2f");
    ruling!(
        "Zuko, Exiled Prince",
        "You must follow all normal timing rules for a card you play using Zuko, Exiled Prince's last ability and, if it's a spell, you must pay its costs to cast it."
    );
    supported("Zuko, Exiled Prince");
    let mut t = TestGame::new(2);
    let zuko = t.battlefield(P0, "Zuko, Exiled Prince");
    let land = t.library_top(P0, "Forest");
    mana(&mut t, P0, ManaType::C, 3);
    act(&mut t, P0, zuko, "Exile the top card", &[]).unwrap();
    t.resolve_all();
    let bears = t.library_top(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::C, 3);
    act(&mut t, P0, zuko, "Exile the top card", &[]).unwrap();
    t.resolve_all();
    plays_follow_the_normal_rules(&mut t, land, bears, "Zuko");
}

#[test]
fn chandras_exiled_cards_follow_the_normal_rules() {
    cr!("305.2", "307.1", "601.2f", "606.3");
    ruling!(
        "Chandra, Heart of Fire",
        "You must follow the normal timing permissions and restrictions for the exiled cards for Chandra's first and last abilities. If one is a land, you can't play it unless you have land plays available."
    );
    ruling!(
        "Chandra, Heart of Fire",
        "If you have no cards in hand, you still exile three cards while resolving Chandra's first ability and you may play them this turn."
    );
    supported("Chandra, Heart of Fire");
    // P0's hand is empty.
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Heart of Fire");
    let cards = stack_library(&mut t, P0, &["Forest", "Grizzly Bears", "Hill Giant"]);
    assert_eq!(t.hand_size(P0), 0);
    act(&mut t, P0, chandra, "Discard your hand", &[]).unwrap();
    t.resolve_all();
    assert!(cards.iter().all(|c| t.zone(*c) == Zone::Exile));
    plays_follow_the_normal_rules(&mut t, cards[0], cards[1], "Chandra");
}

#[test]
fn chandras_unplayed_cards_remain_exiled() {
    cr!("611.2a", "400.7");
    ruling!(
        "Chandra, Heart of Fire",
        "If you don't play the exiled cards, they remain exiled."
    );
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Heart of Fire");
    let cards = stack_library(&mut t, P0, &["Forest", "Grizzly Bears", "Hill Giant"]);
    act(&mut t, P0, chandra, "Discard your hand", &[]).unwrap();
    t.resolve_all();
    assert!(can_play_land(&mut t, P0, cards[0]));
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    assert!(cards.iter().all(|c| t.zone(*c) == Zone::Exile));
    assert!(!can_play_land(&mut t, P0, cards[0]), "only that turn");
}

#[test]
fn chandras_ultimate_is_a_loyalty_ability() {
    cr!("606.3", "605.1a", "606.6");
    ruling!(
        "Chandra, Heart of Fire",
        "Because it's a loyalty ability, Chandra's last ability isn't a mana ability. It can be activated only any time you could cast a sorcery. It uses the stack and can be responded to."
    );
    let mut t = TestGame::new(2);
    let chandra = t.battlefield(P0, "Chandra, Heart of Fire");
    put_counters(&mut t, chandra, counters::LOYALTY, 5);
    t.set_step(P0, Step::BeginningOfCombat);
    assert!(act(&mut t, P0, chandra, "Search your graveyard", &[]).is_err());
    t.set_step(P0, Step::PrecombatMain);
    // Not with a spell on the stack either.
    t.lands(P1, "Mountain", 1);
    cast_new(&mut t, P1, "Shock", &[Entity::Player(P1)]);
    assert!(act(&mut t, P0, chandra, "Search your graveyard", &[]).is_err());
    t.resolve_all();
    act(&mut t, P0, chandra, "Search your graveyard", &[]).unwrap();
    assert_eq!(t.stack_len(), 1);
    assert_eq!(pool_total(&t, P0), 0);
    t.resolve_all();
    assert_eq!(pool_of(&t, P0, ManaType::R), 6);
}

#[test]
fn the_legend_of_roku_cards_can_be_played_without_the_saga() {
    cr!("305.2", "601.2f", "610.3", "714.2b");
    ruling!(
        "The Legend of Roku // Avatar Roku",
        "You pay all costs and follow all timing rules for cards played from exile using The Legend of Roku's first chapter ability. For example, if one of the exiled cards is a land card, you may play it only during your main phase while the stack is empty. You may play those exiled cards even if you no longer control The Legend of Roku."
    );
    let mut t = TestGame::new(2);
    let cards = stack_library(&mut t, P0, &["Forest", "Grizzly Bears", "Hill Giant"]);
    supported("The Legend of Roku // Avatar Roku");
    let saga = t.enter(P0, "The Legend of Roku // Avatar Roku");
    t.resolve_all();
    assert!(cards.iter().all(|c| t.zone(*c) == Zone::Exile));
    destroy(&mut t, saga);
    assert!(!t.on_battlefield(saga));
    plays_follow_the_normal_rules(&mut t, cards[0], cards[1], "Roku");
    // On P0's next turn, the Hill Giant can still be cast.
    t.advance_to(P1, Step::PrecombatMain);
    t.advance_to(P0, Step::PrecombatMain);
    t.lands(P0, "Mountain", 4);
    assert!(castable(&mut t, P0, cards[2]));
}

// --- Arni Slays the Troll ----------------------------------------------------------------

#[test]
fn arni_chapter_one_with_an_illegal_target_no_fight() {
    cr!("701.14b", "608.2b", "714.2b");
    ruling!(
        "Arni Slays the Troll",
        "For the chapter I ability, if either target is an illegal target as that ability resolves, neither creature will deal or be dealt damage."
    );
    for gone in [0usize, 1] {
        let mut t = TestGame::new(2);
        let mine = t.battlefield(P0, "Hill Giant");
        let theirs = t.battlefield(P1, "Grizzly Bears");
        t.answer_targets(P0, &[obj(mine)]);
        t.answer_targets(P0, &[obj(theirs)]);
        chapter(&mut t, "Arni Slays the Troll", 1);
        // One target leaves (bounced) in response.
        let leaving = [mine, theirs][gone];
        let staying = [mine, theirs][1 - gone];
        t.lands(P1, "Island", 1);
        cast_new(&mut t, P1, "Unsummon", &[obj(leaving)]);
        t.resolve_all();
        assert_eq!(t.obj_now(staying).damage, 0, "gone={gone}");
        assert!(t.on_battlefield(staying));
    }
}

#[test]
fn arni_chapter_two_targets() {
    cr!("714.2b", "603.3d", "608.2b");
    ruling!(
        "Arni Slays the Troll",
        "If you choose no targets for the chapter II ability, you’ll simply add {R} as the ability resolves. However, if you do choose a target creature you control, and that creature is an illegal target as the ability resolves, the ability won’t resolve. You won’t add {R} in that case."
    );
    // No target: {R}.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[]);
    chapter(&mut t, "Arni Slays the Troll", 2);
    t.resolve_all();
    assert_eq!(pool_of(&t, P0, ManaType::R), 1);
    // A target that's gone: nothing.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(b)]);
    chapter(&mut t, "Arni Slays the Troll", 2);
    destroy(&mut t, b);
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 0);
    // A legal target: {R} and the counters.
    let mut t = TestGame::new(2);
    let b = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(b)]);
    chapter(&mut t, "Arni Slays the Troll", 2);
    t.resolve_all();
    assert_eq!(pool_of(&t, P0, ManaType::R), 1);
    assert_eq!(t.counters(b, counters::PLUS1), 2);
}

#[test]
fn arni_chapter_three_uses_the_greatest_power_on_resolution() {
    cr!("608.2h", "714.2b");
    ruling!(
        "Arni Slays the Troll",
        "For the chapter III ability, use the greatest power among creatures you control as the ability resolves to determine how much life you will gain. If you control no creatures at that time, or if all creatures you control have power 0 or less, you won’t gain any life."
    );
    // Hill Giant (3) and Grizzly Bears; the Giant dies in response: 2 life.
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    t.battlefield(P0, "Grizzly Bears");
    chapter(&mut t, "Arni Slays the Troll", 3);
    destroy(&mut t, giant);
    t.resolve_all();
    assert_eq!(t.life(P0), 22);
    // Only a 0-power creature: nothing. No creatures: nothing.
    for with_thopter in [true, false] {
        let mut t = TestGame::new(2);
        if with_thopter {
            t.battlefield(P0, "Ornithopter");
        }
        chapter(&mut t, "Arni Slays the Troll", 3);
        t.resolve_all();
        assert_eq!(t.life(P0), 20);
    }
}

// --- The Bath Song -----------------------------------------------------------------------

#[test]
fn the_bath_song_final_chapter_targets() {
    cr!("714.2b", "603.3d", "608.2b");
    ruling!(
        "The Bath Song",
        "You can choose no targets for The Bath Song's final chapter ability just to add {U}{U}. However, if you do choose targets, and all of those targets are illegal at the time the ability tries to resolve, the ability won't resolve and none of its effects will happen. You won't add {U}{U}."
    );
    // No targets.
    let mut t = TestGame::new(2);
    t.answer_targets(P0, &[]);
    chapter(&mut t, "The Bath Song", 3);
    t.resolve_all();
    assert_eq!(pool_of(&t, P0, ManaType::U), 2);
    // A target card exiled in response: nothing.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(card)]);
    chapter(&mut t, "The Bath Song", 3);
    let c = t.g.current(card);
    t.g.move_object(
        c,
        Zone::Exile,
        mtg_engine::events::MoveCause::Effect,
        Some(P1),
    );
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 0);
    // A legal target: shuffled in, {U}{U}.
    let mut t = TestGame::new(2);
    let card = t.graveyard(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(card)]);
    chapter(&mut t, "The Bath Song", 3);
    t.resolve_all();
    assert_eq!(t.zone(card), Zone::Library(P0));
    assert_eq!(pool_of(&t, P0, ManaType::U), 2);
}

// --- Niko Defies Destiny -----------------------------------------------------------------

#[test]
fn niko_mana_pays_for_a_spell_with_foretell_from_hand() {
    cr!("106.6", "702.143a");
    ruling!(
        "Niko Defies Destiny",
        "The mana added by the chapter II ability can be spent to cast a spell that has foretell no matter where you’re casting it from. It doesn’t have to be a foretold card you’re casting from exile."
    );
    supported("Battle Mammoth");
    // A card with foretell cast from hand: Battle Mammoth ({3}{G}{G}) with {W}{U} from
    // Niko and three Forests.
    let mut t = TestGame::new(2);
    chapter(&mut t, "Niko Defies Destiny", 2);
    t.resolve_all();
    assert_eq!(pool_total(&t, P0), 2);
    t.lands(P0, "Forest", 3);
    let mammoth = t.hand(P0, "Battle Mammoth");
    assert!(castable(&mut t, P0, mammoth));
    t.cast(P0, mammoth).method(CastMethod::Normal).go();
    assert_eq!(pool_total(&t, P0), 0);
    // A spell without foretell can't use it: Craw Wurm ({4}{G}{G}) with the same mana.
    let mut t = TestGame::new(2);
    chapter(&mut t, "Niko Defies Destiny", 2);
    t.resolve_all();
    t.lands(P0, "Forest", 4);
    let wurm = t.hand(P0, "Craw Wurm");
    assert!(t.cast(P0, wurm).try_go().is_err());
    assert_eq!(pool_total(&t, P0), 2);
}

// --- Clive, Ifrit's Dominant -------------------------------------------------------------

#[test]
fn clive_counts_devotion_as_its_ability_resolves() {
    cr!("700.5", "608.2h");
    ruling!(
        "Clive, Ifrit's Dominant // Ifrit, Warden of Inferno",
        "Count the number of red mana symbols among the mana costs of permanents you control as Clive's triggered ability resolves to determine your devotion to red. If Clive is still on the battlefield at that time, it will be included in that count."
    );
    supported("Clive, Ifrit's Dominant // Ifrit, Warden of Inferno");
    // Clive ({4}{R}{R}) enters; `respond` acts before its ability resolves; returns the
    // cards drawn.
    let drawn = |respond: fn(&mut TestGame, ObjectId)| {
        let mut t = TestGame::new(2);
        t.answer_yes(P0, true);
        let clive = t.enter(P0, "Clive, Ifrit's Dominant // Ifrit, Warden of Inferno");
        t.settle();
        assert_eq!(t.stack_len(), 1);
        respond(&mut t, clive);
        let library = t.library_size(P0);
        t.resolve_all();
        library - t.library_size(P0)
    };
    assert_eq!(drawn(|_, _| {}), 2);
    // A Raging Goblin ({R}) arrives in response: 3.
    assert_eq!(
        drawn(|t, _| {
            t.battlefield(P0, "Raging Goblin");
        }),
        3
    );
    // Clive is bounced in response: 0.
    assert_eq!(
        drawn(|t, clive| {
            t.lands(P1, "Island", 1);
            cast_new(t, P1, "Unsummon", &[obj(clive)]);
            t.resolve();
        }),
        0
    );
}
