//! Rulings batch P126 — casting spells from graveyards and exile: timing permissions
//! (CR 601.3, 305.2), costs (CR 601.2b, 601.2f, 118.9), "if a card would be put into your
//! graveyard, exile it instead" (CR 614.1a, 700.4), linked abilities and new objects
//! (CR 607.2a, 400.7), and "attacks alone" (CR 506.5).

use crate::r_p076_common::mana;
use crate::r_s01_common::{attack_with, supported};
use crate::r_s02_common::{can_activate, can_cast, can_play_land};
use crate::r_s05_common::{enter, move_to};
use crate::r_s08_common::legal_cast_methods;
use mtg_engine::casting::cast_during_resolution;
use mtg_engine::decision::Answer;
use mtg_engine::mana::ManaType;
use mtg_engine::object::{CastMethod, Zone};
use mtg_engine::rooms;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

fn obj(id: ObjectId) -> Entity {
    Entity::Object(id)
}

// ---------------------------------------------------------------------------
// Attacks alone
// ---------------------------------------------------------------------------

#[test]
fn yamazakis_trigger_only_when_attacking_alone() {
    cr!("506.5", "508.1");
    ruling!(
        "Heiko Yamazaki, the General",
        "Heiko Yamazaki's triggered ability won't trigger if you attack with multiple creatures and all but one of them are removed from combat."
    );
    ruling!(
        "Norika Yamazaki, the Poet",
        "Norika Yamazaki's triggered ability won't trigger if you attack with multiple creatures and all but one of them are removed from combat."
    );
    for (name, card) in [
        ("Heiko Yamazaki, the General", "Bonesplitter"),
        ("Norika Yamazaki, the Poet", "Pacifism"),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let y = t.battlefield(P0, name);
        let bears = t.battlefield(P0, "Grizzly Bears");
        t.graveyard(P0, card);
        attack_with(&mut t, &[(y, Entity::Player(P1)), (bears, Entity::Player(P1))]);
        assert_eq!(t.stack_len(), 0, "{name}");
        move_to(&mut t, bears, Zone::Graveyard(P0));
        t.settle();
        assert_eq!(t.stack_len(), 0, "{name}");
        // Alone: it triggers.
        let mut t = TestGame::new(2);
        let y = t.battlefield(P0, name);
        let c = t.graveyard(P0, card);
        t.answer_targets(P0, &[obj(c)]);
        attack_with(&mut t, &[(y, Entity::Player(P1))]);
        assert_eq!(t.stack_len(), 1, "{name}");
    }
}

#[test]
fn yamazakis_cast_with_normal_timing_and_costs() {
    cr!("601.3", "307.1", "601.2f");
    ruling!(
        "Heiko Yamazaki, the General",
        "You must still pay all costs and follow all normal timing rules when casting an artifact from your graveyard this way. Normally, this means you will not be able to cast that card until your post-combat main phase."
    );
    ruling!(
        "Norika Yamazaki, the Poet",
        "You must still follow all normal timing rules and pay all costs for the enchantment card you cast this way."
    );
    for (name, card, color) in [
        ("Heiko Yamazaki, the General", "Bonesplitter", None),
        ("Norika Yamazaki, the Poet", "Pacifism", Some(ManaType::W)),
    ] {
        let mut t = TestGame::new(2);
        let y = t.battlefield(P0, name);
        let c = t.graveyard(P0, card);
        t.answer_targets(P0, &[obj(c)]);
        attack_with(&mut t, &[(y, Entity::Player(P1))]);
        t.resolve_all();
        if let Some(col) = color {
            mana(&mut t, P0, col, 1);
            mana(&mut t, P0, ManaType::C, 1);
        } else {
            mana(&mut t, P0, ManaType::C, 1);
        }
        assert!(legal_cast_methods(&mut t, P0, c).is_empty(), "{name}: not during combat");
        t.advance_to(P0, Step::PostcombatMain);
        // Mana empties between steps: no mana, no cast.
        assert!(legal_cast_methods(&mut t, P0, c).is_empty(), "{name}: costs are paid");
        if let Some(col) = color {
            mana(&mut t, P0, col, 1);
        }
        mana(&mut t, P0, ManaType::C, 1);
        t.battlefield(P1, "Grizzly Bears");
        assert!(can_cast(&mut t, P0, c, CastMethod::Normal), "{name}");
    }
}

// ---------------------------------------------------------------------------
// "Exile it instead"
// ---------------------------------------------------------------------------

fn magus_activated(t: &mut TestGame) {
    let magus = t.battlefield(P0, "Magus of the Will");
    mana(t, P0, ManaType::B, 1);
    mana(t, P0, ManaType::C, 2);
    t.activate(P0, magus, 0, &[]).unwrap();
    t.resolve_all();
}

fn forgotten_cellar_unlocked(t: &mut TestGame) {
    let room = t.battlefield(P0, "Walk-In Closet // Forgotten Cellar");
    assert!(rooms::unlock(&mut t.g, room, 1, P0));
    t.g.flush_events();
    t.settle();
    assert_eq!(t.stack_len(), 1, "Forgotten Cellar triggered");
    t.resolve_all();
}

/// A Walk-In Closet // Forgotten Cellar with the Walk-In Closet door unlocked.
fn walk_in_closet(t: &mut TestGame) {
    let room = t.battlefield(P0, "Walk-In Closet // Forgotten Cellar");
    rooms::unlock(&mut t.g, room, 0, P0);
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(rooms::unlocked(&t.g, room), [true, false]);
}

#[test]
fn exile_instead_effects_apply_to_cards_you_didnt_play() {
    cr!("614.1a", "700.4", "111.7");
    ruling!(
        "Magus of the Will",
        "Cards that would be put into your graveyard are exiled even if you didn't play them this turn, such as a nontoken creature that would die."
    );
    ruling!(
        "Walk-In Closet // Forgotten Cellar",
        "Cards that would be put into your graveyard on the turn Forgotten Cellar's ability resolves are exiled even if you didn't play them this turn."
    );
    ruling!(
        "Walk-In Closet // Forgotten Cellar",
        "Forgotten Cellar's ability won't stop tokens from going to your graveyard as normal"
    );
    ruling!(
        "Magus of the Will",
        "Tokens are put into your graveyard as normal (and cease to exist soon after). Abilities that trigger when a creature dies can trigger on token creatures being put into your graveyard but won't trigger on nontoken creatures being exiled instead of being put into your graveyard."
    );
    ruling!(
        "Gaea's Will",
        "After Gaea's Will resolves, nontoken creature you own that would die will be exiled instead. Abilities that would trigger when those creatures die won't trigger. Token creatures you own will still die as normal after Gaea's Will resolves."
    );
    supported("Magus of the Will");
    supported("Walk-In Closet // Forgotten Cellar");
    supported("Gaea's Will");
    for which in 0..3 {
        let mut t = TestGame::new(2);
        let bears = t.battlefield(P0, "Grizzly Bears");
        match which {
            0 => magus_activated(&mut t),
            1 => forgotten_cellar_unlocked(&mut t),
            _ => {
                let will = t.hand(P0, "Gaea's Will");
                cast_during_resolution(&mut t.g, P0, will, CastMethod::Free).unwrap();
                t.resolve_all();
            }
        }
        t.battlefield(P0, "Blood Artist");
        let life = t.life(P1);
        t.g.destroy(bears, None);
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(t.zone(bears), Zone::Exile, "case {which}");
        assert_eq!(t.life(P1), life, "case {which}: no dies trigger");
        // A token dies as normal.
        let token = crate::r_s02_common::create_token(&mut t, P0, "Soldier");
        t.answer_targets(P0, &[Entity::Player(P1)]);
        t.g.destroy(token, None);
        t.g.flush_events();
        t.resolve_all();
        assert_eq!(t.life(P1), life - 1, "case {which}: the token died");
    }
}

#[test]
fn gaeas_will_exiles_itself() {
    cr!("608.2n", "614.1a");
    ruling!(
        "Gaea's Will",
        "Assuming it's not a copy, Gaea's Will will be exiled after it finishes resolving."
    );
    let mut t = TestGame::new(2);
    let will = t.hand(P0, "Gaea's Will");
    cast_during_resolution(&mut t.g, P0, will, CastMethod::Free).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(will), Zone::Exile);
}

#[test]
fn yawgmoths_agenda_exiles_itself() {
    cr!("614.1a", "614.6");
    ruling!(
        "Yawgmoth's Agenda",
        "It will exile itself if it is going to the graveyard from the battlefield."
    );
    supported("Yawgmoth's Agenda");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Yawgmoth's Agenda");
    t.g.destroy(a, None);
    t.g.flush_events();
    assert_eq!(t.zone(a), Zone::Exile);
}

#[test]
fn yawgmoths_agenda_counts_spells_cast_before_it_entered() {
    cr!("604.1", "101.2");
    ruling!(
        "Yawgmoth's Agenda",
        "You count spells that were cast this turn before this card enters."
    );
    let mut t = TestGame::new(2);
    let bears = t.hand(P0, "Ornithopter");
    t.cast(P0, bears).go();
    t.resolve_all();
    t.battlefield(P0, "Yawgmoth's Agenda");
    let o2 = t.hand(P0, "Ornithopter");
    assert!(!can_cast(&mut t, P0, o2, CastMethod::Normal));
}

#[test]
fn graveyard_cards_arent_in_your_hand() {
    cr!("701.9a", "402.1", "702.29a");
    ruling!(
        "Yawgmoth's Agenda",
        "The cards in your graveyard are not considered to be in your hand for any reason. For example, you can't discard them."
    );
    ruling!(
        "Walk-In Closet // Forgotten Cellar",
        "Walk-In Closet's ability doesn't allow you to activate abilities (such as cycling) of land cards in your graveyard."
    );
    // Yawgmoth's Agenda: a discard effect finds no cards with an empty hand.
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Yawgmoth's Agenda");
    let g = t.graveyard(P1, "Grizzly Bears");
    let mind_rot = t.hand(P0, "Mind Rot");
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, mind_rot).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.zone(g), Zone::Graveyard(P1));
    // Walk-In Closet: a cycling land in the graveyard can be played, not cycled.
    let mut t = TestGame::new(2);
    walk_in_closet(&mut t);
    let steppe = t.graveyard(P0, "Secluded Steppe");
    mana(&mut t, P0, ManaType::W, 1);
    assert!(!can_activate(&mut t, P0, steppe));
    assert!(can_play_land(&mut t, P0, steppe));
}

#[test]
fn magus_of_the_will_with_buyback_you_choose() {
    cr!("616.1", "702.27a");
    ruling!(
        "Magus of the Will",
        "If another effect tries to change where a spell is put as it resolves, such as that of a rebound or buyback ability, you may choose whether to exile the card to Magus of the Will's effect or to apply the other effect."
    );
    let mut zones = Vec::new();
    for choice in [0, 1] {
        let mut t = TestGame::new(2);
        magus_activated(&mut t);
        t.library_top(P0, "Island");
        let w = t.hand(P0, "Whispers of the Muse");
        mana(&mut t, P0, ManaType::U, 1);
        mana(&mut t, P0, ManaType::C, 5);
        t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
        t.cast(P0, w).go();
        let from = t.asked().len();
        t.answer(P0, DecisionKind::Replacement, Answer::Index(choice));
        t.resolve_all();
        let asked = t.asked()[from..]
            .iter()
            .any(|(_, d)| matches!(d, mtg_engine::decision::Decision::ChooseReplacement { .. }));
        assert!(asked, "the controller chooses");
        zones.push(t.zone(w));
    }
    // One choice returns it to its owner's hand (buyback), the other exiles it.
    zones.sort_by_key(|z| format!("{z:?}"));
    assert_eq!(zones, vec![Zone::Exile, Zone::Hand(P0)]);
}

// ---------------------------------------------------------------------------
// Timing and costs of permissions
// ---------------------------------------------------------------------------

#[test]
fn graveyard_permissions_dont_change_timing() {
    cr!("601.3", "307.1", "305.2", "117.1a");
    ruling!(
        "Magus of the Will",
        "Magus of the Will doesn't change when you can play the cards in your graveyard. For example, if you have a creature card without flash in your graveyard, you can cast it only during your main phase while the stack is empty."
    );
    ruling!(
        "Magus of the Will",
        "You may play a land card from your graveyard only if you have an available land play."
    );
    ruling!(
        "Walk-In Closet // Forgotten Cellar",
        "Walk-In Closet's ability doesn't change the times when you can play those land cards. You can still play only one land per turn, and only during your main phase when you have priority and the stack is empty."
    );
    ruling!(
        "Conduit of Worlds",
        "The first ability of Conduit of Worlds doesn't change the times when you can play those land cards. You can still play only one land per turn, and only during your main phase when you have priority and the stack is empty."
    );
    ruling!(
        "Chainer, Nightmare Adept",
        "Chainer's first ability doesn't change when you can cast the creature card. You can normally only cast it during your turn while the stack is empty, but if the creature card has flash, you can cast it at other times."
    );
    // Lands: Magus of the Will, Walk-In Closet, Conduit of Worlds.
    for which in 0..3 {
        let mut t = TestGame::new(2);
        match which {
            0 => magus_activated(&mut t),
            1 => walk_in_closet(&mut t),
            _ => {
                t.battlefield(P0, "Conduit of Worlds");
            }
        }
        let f1 = t.graveyard(P0, "Forest");
        let f2 = t.graveyard(P0, "Forest");
        // Not during combat, not with the stack nonempty.
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(!can_play_land(&mut t, P0, f1), "case {which}");
        t.set_step(P0, Step::PostcombatMain);
        let bolt = t.hand(P0, "Ornithopter");
        t.cast(P0, bolt).go();
        assert!(!can_play_land(&mut t, P0, f1), "case {which}: stack");
        t.resolve_all();
        assert!(can_play_land(&mut t, P0, f1), "case {which}");
        t.play_land(P0, f1).unwrap();
        assert!(!can_play_land(&mut t, P0, f2), "case {which}: one land per turn");
    }
    // Spells: Magus of the Will and Chainer.
    for which in 0..2 {
        let mut t = TestGame::new(2);
        if which == 0 {
            magus_activated(&mut t);
        } else {
            let chainer = t.battlefield(P0, "Chainer, Nightmare Adept");
            t.hand(P0, "Island");
            t.activate(P0, chainer, 0, &[]).unwrap();
            t.resolve_all();
        }
        let bears = t.graveyard(P0, "Grizzly Bears");
        let ambusher = t.graveyard(P0, "Ambush Viper");
        mana(&mut t, P0, ManaType::G, 2);
        mana(&mut t, P0, ManaType::C, 2);
        t.set_step(P0, Step::BeginningOfCombat);
        assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal), "case {which}");
        assert!(can_cast(&mut t, P0, ambusher, CastMethod::Normal), "case {which}: flash");
    }
}

#[test]
fn permissions_from_exile_and_graveyards_follow_timing_and_costs() {
    cr!("601.3", "601.2f", "307.1");
    ruling!(
        "Rona, Disciple of Gix",
        "You must follow the normal timing permissions and restrictions of the cards you cast from exile."
    );
    ruling!(
        "Tlincalli Hunter // Retrieve Prey",
        "You must pay all costs and follow all normal timing rules when casting a creature card from exile with Retrieve Prey."
    );
    ruling!(
        "Zul Ashur, Lich Lord",
        "You pay all costs and follow all normal timing rules for Zombie creature cards cast with the permission granted by Zul Ashur's last ability."
    );
    supported("Rona, Disciple of Gix");
    supported("Tlincalli Hunter // Retrieve Prey");
    supported("Zul Ashur, Lich Lord");
    // Rona: an exiled creature card (from the top of the library).
    let mut t = TestGame::new(2);
    let rona = t.battlefield(P0, "Rona, Disciple of Gix");
    let bears = t.library_top(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::C, 4);
    t.activate(P0, rona, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    // (The card exiled is face up.)
    assert!(!t.obj_now(bears).face_down);
    mana(&mut t, P0, ManaType::G, 1);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal), "{{1}}{{G}} not paid");
    mana(&mut t, P0, ManaType::C, 1);
    t.set_step(P0, Step::BeginningOfCombat);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 1);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal), "not during combat");
    t.set_step(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 1);
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
    // Retrieve Prey.
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let tl = t.hand(P0, "Tlincalli Hunter // Retrieve Prey");
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 1);
    t.cast(P0, tl).method(CastMethod::Half(1)).target(bears).go();
    t.resolve_all();
    assert_eq!(t.zone(bears), Zone::Exile);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal), "costs are paid");
    t.set_step(P0, Step::BeginningOfCombat);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 1);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal), "not during combat");
    t.set_step(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 1);
    assert!(can_cast(&mut t, P0, bears, CastMethod::Normal));
    // Zul Ashur.
    let mut t = TestGame::new(2);
    let zul = t.battlefield(P0, "Zul Ashur, Lich Lord");
    let corpse = t.graveyard(P0, "Walking Corpse");
    t.activate(P0, zul, 0, &[obj(corpse)]).unwrap();
    t.resolve_all();
    assert!(!can_cast(&mut t, P0, corpse, CastMethod::Normal), "costs are paid");
    t.set_step(P0, Step::BeginningOfCombat);
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::C, 1);
    assert!(!can_cast(&mut t, P0, corpse, CastMethod::Normal), "not during combat");
    t.set_step(P0, Step::PostcombatMain);
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::C, 1);
    assert!(can_cast(&mut t, P0, corpse, CastMethod::Normal));
}

#[test]
fn graveyard_permissions_allow_alternative_and_additional_costs() {
    cr!("601.2b", "601.2f", "118.9", "702.96a");
    ruling!(
        "Magus of the Will",
        "You pay the costs for a card in your graveyard if you cast it. You may pay alternative costs such as overload rather than the card's mana cost."
    );
    ruling!(
        "Walk-In Closet // Forgotten Cellar",
        "You pay all costs and follow all timing rules for spells cast from your graveyard with the permission granted by Forgotten Cellar's ability. You may pay alternative costs, such as impending costs, rather than the spell's mana cost. You may pay optional additional costs, such as kicker"
    );
    // Magus of the Will: Vandalblast overloaded from the graveyard.
    let mut t = TestGame::new(2);
    magus_activated(&mut t);
    let vb = t.graveyard(P0, "Vandalblast");
    let a1 = t.battlefield(P1, "Ornithopter");
    let a2 = t.battlefield(P1, "Ornithopter");
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 4);
    assert!(legal_cast_methods(&mut t, P0, vb).contains(&CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Overload)));
    t.cast(P0, vb).method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Overload)).go();
    t.resolve_all();
    assert!(!t.on_battlefield(a1) && !t.on_battlefield(a2));
    assert_eq!(t.zone(vb), Zone::Exile);
    // Forgotten Cellar: a kicked spell from the graveyard.
    let mut t = TestGame::new(2);
    forgotten_cellar_unlocked(&mut t);
    let kb = t.graveyard(P0, "Burst Lightning");
    mana(&mut t, P0, ManaType::R, 1);
    mana(&mut t, P0, ManaType::C, 4);
    t.answer(P0, DecisionKind::OptionalCost, Answer::Bool(true));
    t.cast(P0, kb).target(Entity::Player(P1)).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 16);
    assert_eq!(t.zone(kb), Zone::Exile);
}

#[test]
fn rona_pays_all_costs_and_losing_rona_doesnt_matter() {
    cr!("601.2h", "601.2b", "607.2a");
    ruling!(
        "Rona, Disciple of Gix",
        "You'll still pay all costs for that spell, including additional costs."
    );
    ruling!(
        "Rona, Disciple of Gix",
        "Once you begin to cast a card, losing control of Rona won't affect the spell."
    );
    let mut t = TestGame::new(2);
    let rona = t.battlefield(P0, "Rona, Disciple of Gix");
    let reap = t.library_top(P0, "Altar's Reap");
    mana(&mut t, P0, ManaType::C, 4);
    t.activate(P0, rona, 0, &[]).unwrap();
    t.resolve_all();
    assert_eq!(t.zone(reap), Zone::Exile);
    // Altar's Reap: sacrifice a creature as an additional cost. Sacrifice Rona itself.
    mana(&mut t, P0, ManaType::B, 1);
    mana(&mut t, P0, ManaType::C, 1);
    let hand = t.hand_size(P0);
    t.answer_choose(P0, &[obj(rona)]);
    let reap = t.g.current(reap);
    t.cast(P0, reap).go();
    assert!(!t.on_battlefield(rona));
    t.resolve_all();
    assert_eq!(t.hand_size(P0), hand + 2);
}

#[test]
fn rona_exiles_the_top_card_as_the_ability_resolves() {
    cr!("608.2c", "406.3");
    ruling!(
        "Rona, Disciple of Gix",
        "The card you'll exile from the top of your library is determined as Rona's last ability resolves."
    );
    ruling!("Rona, Disciple of Gix", "The cards exiled from your library are exiled face up.");
    let mut t = TestGame::new(2);
    let rona = t.battlefield(P0, "Rona, Disciple of Gix");
    let first = t.library_top(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::C, 4);
    t.activate(P0, rona, 0, &[]).unwrap();
    let second = t.library_top(P0, "Hill Giant");
    t.resolve_all();
    assert_eq!(t.zone(second), Zone::Exile);
    assert!(!t.obj_now(second).face_down);
    assert_eq!(t.zone(first), Zone::Library(P0));
}

#[test]
fn rona_exiled_cards_arent_in_your_hand() {
    cr!("702.29a", "602.2");
    ruling!(
        "Rona, Disciple of Gix",
        "The exiled cards aren't in your hand, so you can't cycle them, discard them, or activate any of their activated abilities."
    );
    let mut t = TestGame::new(2);
    let rona = t.battlefield(P0, "Rona, Disciple of Gix");
    let steppe = t.library_top(P0, "Secluded Steppe");
    mana(&mut t, P0, ManaType::C, 4);
    t.activate(P0, rona, 0, &[]).unwrap();
    t.resolve_all();
    mana(&mut t, P0, ManaType::W, 1);
    assert!(!can_activate(&mut t, P0, steppe));
}

#[test]
fn a_new_rona_isnt_linked_to_the_old_ronas_cards() {
    cr!("607.2a", "400.7");
    ruling!(
        "Rona, Disciple of Gix",
        "If Rona leaves the battlefield, the exiled cards will remain exiled indefinitely. If Rona enters the battlefield again, it won't be associated with the cards the \"other\" Rona exiled."
    );
    let mut t = TestGame::new(2);
    let rona = t.battlefield(P0, "Rona, Disciple of Gix");
    let bears = t.library_top(P0, "Grizzly Bears");
    mana(&mut t, P0, ManaType::C, 4);
    t.activate(P0, rona, 0, &[]).unwrap();
    t.resolve_all();
    move_to(&mut t, rona, Zone::Graveyard(P0));
    let rona2 = move_to(&mut t, rona, Zone::Battlefield).unwrap();
    t.resolve_all();
    assert!(t.on_battlefield(rona2));
    assert_eq!(t.zone(bears), Zone::Exile);
    mana(&mut t, P0, ManaType::G, 1);
    mana(&mut t, P0, ManaType::C, 1);
    assert!(!can_cast(&mut t, P0, bears, CastMethod::Normal));
}

#[test]
fn rona_you_have_priority_after_its_ability_resolves() {
    cr!("117.3b", "608.2n");
    ruling!(
        "Rona, Disciple of Gix",
        "After Rona's last ability resolves, you have priority if it's your turn. You can cast the exiled card before any player can take other actions if it's legal to do so."
    );
    let mut t = TestGame::new(2);
    let rona = t.battlefield(P0, "Rona, Disciple of Gix");
    let bolt = t.library_top(P0, "Lightning Bolt");
    mana(&mut t, P0, ManaType::C, 4);
    t.activate(P0, rona, 0, &[]).unwrap();
    // P0 casts Lightning Bolt at the first priority after the ability resolves.
    let b = t.g.current(bolt);
    let _ = b;
    t.resolve();
    assert_eq!(t.zone(bolt), Zone::Exile);
    assert_eq!(t.g.turn.priority, Some(P0));
    mana(&mut t, P0, ManaType::R, 1);
    assert!(can_cast(&mut t, P0, bolt, CastMethod::Normal));
}

// ---------------------------------------------------------------------------
// Chainer
// ---------------------------------------------------------------------------

#[test]
fn chainer_gives_haste_when_it_enters_without_being_cast_from_hand() {
    cr!("603.4", "702.10b");
    ruling!(
        "Chainer, Nightmare Adept",
        "Chainer's last ability triggers if Chainer enters the battlefield without being cast from your hand."
    );
    let mut t = TestGame::new(2);
    let chainer = enter(&mut t, P0, "Chainer, Nightmare Adept");
    assert_eq!(t.stack_len(), 1);
    t.resolve_all();
    assert!(t.obj_now(chainer).has_keyword(mtg_engine::keywords::KeywordKind::Haste));
}

#[test]
fn chainer_permissions_stack_up() {
    cr!("601.3", "400.7", "602.5b");
    ruling!(
        "Chainer, Nightmare Adept",
        "If you somehow activate Chainer's first ability more than once during a turn (perhaps because Chainer left the battlefield and returned), you can cast a creature card for each ability's permission."
    );
    let mut t = TestGame::new(2);
    let chainer = t.battlefield(P0, "Chainer, Nightmare Adept");
    t.hand(P0, "Island");
    t.hand(P0, "Island");
    let b1 = t.graveyard(P0, "Grizzly Bears");
    let b2 = t.graveyard(P0, "Grizzly Bears");
    t.activate(P0, chainer, 0, &[]).unwrap();
    t.resolve_all();
    mana(&mut t, P0, ManaType::G, 2);
    mana(&mut t, P0, ManaType::C, 2);
    t.cast(P0, b1).go();
    t.resolve_all();
    assert!(!can_cast(&mut t, P0, b2, CastMethod::Normal));
    // Chainer leaves and returns: a new object, a new activation.
    move_to(&mut t, chainer, Zone::Exile);
    let c2 = move_to(&mut t, chainer, Zone::Battlefield).unwrap();
    t.resolve_all();
    t.activate(P0, c2, 0, &[]).unwrap();
    t.resolve_all();
    assert!(can_cast(&mut t, P0, b2, CastMethod::Normal));
}

// ---------------------------------------------------------------------------
// Scholar of the Lost Trove, Horde of Notions, Bilbo
// ---------------------------------------------------------------------------

#[test]
fn scholar_casts_now_without_alternative_costs_but_with_additional_costs() {
    cr!("118.9a", "601.2b", "608.2g");
    ruling!(
        "Scholar of the Lost Trove",
        "If you cast a spell “without paying its mana cost,” you can’t choose to cast it for any alternative costs. You can, however, pay additional costs. If the card has any mandatory additional costs, those must be paid to cast the spell."
    );
    ruling!(
        "Scholar of the Lost Trove",
        "If you wish to cast the target card, you must cast it while the triggered ability is resolving. You can’t cast it later in the turn."
    );
    supported("Scholar of the Lost Trove");
    // Altar's Reap: sacrifice a creature as it's cast.
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    let reap = t.graveyard(P0, "Altar's Reap");
    t.answer_targets(P0, &[obj(reap)]);
    let scholar = enter(&mut t, P0, "Scholar of the Lost Trove");
    let hand = t.hand_size(P0);
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[obj(bears)]);
    t.resolve_all();
    assert!(!t.on_battlefield(bears));
    assert!(t.on_battlefield(scholar));
    assert_eq!(t.hand_size(P0), hand + 2);
    assert_eq!(t.zone(reap), Zone::Exile);
    // Declining: it can't be cast later.
    let mut t = TestGame::new(2);
    let growth = t.graveyard(P0, "Giant Growth");
    t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[obj(growth)]);
    enter(&mut t, P0, "Scholar of the Lost Trove");
    t.answer_yes(P0, false);
    t.resolve_all();
    mana(&mut t, P0, ManaType::G, 1);
    assert!(legal_cast_methods(&mut t, P0, growth).is_empty());
}

#[test]
fn scholar_artifact_isnt_exiled() {
    cr!("614.1a", "608.2n");
    ruling!(
        "Scholar of the Lost Trove",
        "If you cast an artifact spell this way, it won’t be exiled if it would be put into your graveyard this turn."
    );
    let mut t = TestGame::new(2);
    let orn = t.graveyard(P0, "Ornithopter");
    t.answer_targets(P0, &[obj(orn)]);
    enter(&mut t, P0, "Scholar of the Lost Trove");
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(t.on_battlefield(orn));
    t.g.destroy(t.g.current(orn), None);
    t.g.flush_events();
    assert_eq!(t.zone(orn), Zone::Graveyard(P0));
}

#[test]
fn horde_of_notions_cant_pay_evoke() {
    cr!("118.9a", "702.74a");
    ruling!(
        "Horde of Notions",
        "This ability does not allow you to pay the evoke cost of the targeted Elemental card."
    );
    supported("Horde of Notions");
    let mut t = TestGame::new(2);
    let horde = t.battlefield(P0, "Horde of Notions");
    let shriek = t.graveyard(P0, "Shriekmaw");
    t.battlefield(P1, "Hill Giant");
    for ty in [ManaType::W, ManaType::U, ManaType::B, ManaType::R, ManaType::G] {
        mana(&mut t, P0, ty, 1);
    }
    t.activate(P0, horde, 0, &[obj(shriek)]).unwrap();
    t.answer_yes(P0, true);
    t.resolve_all();
    // It was cast without paying its mana cost, not evoked: it stays on the battlefield.
    assert!(t.on_battlefield(shriek));
}

#[test]
fn bilbo_reduces_the_total_cost_not_the_mana_value() {
    cr!("601.2f", "202.3", "118.7");
    ruling!(
        "Bilbo, Thief in the Night",
        "To determine the total cost of a spell, start with the mana cost or alternative cost you're paying, add any cost increases, then apply any cost reductions (such as that of Bilbo, Thief in the Night). The mana value of the spell is determined only by its mana cost"
    );
    supported("Bilbo, Thief in the Night");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Bilbo, Thief in the Night");
    t.library_top(P0, "Island");
    let tt = t.graveyard(P0, "Think Twice");
    // Flashback {2}{U} - {1}.
    mana(&mut t, P0, ManaType::U, 1);
    mana(&mut t, P0, ManaType::C, 1);
    let spell = t
        .cast(P0, tt)
        .method(CastMethod::Keyword(mtg_engine::keywords::KeywordKind::Flashback))
        .go();
    assert_eq!(t.g.player(P0).mana_pool.total(), 0);
    assert_eq!(t.g.mana_value_of(spell), 2);
}

#[test]
fn bilbo_adventure_from_the_graveyard() {
    cr!("715.4", "614.1a", "608.2b");
    ruling!(
        "Bilbo, Thief in the Night",
        "You can choose to cast an Adventure instant or sorcery spell from your graveyard as Bilbo's ability resolves. If that Adventure spell resolves, you can exile it using the replacement effect associated with the Adventure, and you can cast the permanent spell later from exile. If that Adventure spell fails to resolve (because it's countered or its targets become illegal), that card is exiled by the replacement effect created by Bilbo's ability; you can't cast the permanent spell later from exile."
    );
    for resolves in [true, false] {
        let mut t = TestGame::new(2);
        let bilbo = t.battlefield(P0, "Bilbo, Thief in the Night");
        let giant = t.graveyard(P0, "Bonecrusher Giant // Stomp");
        let target = t.battlefield(P1, "Grizzly Bears");
        t.answer_choose(P0, &[obj(giant)]);
        t.answer_targets(P0, &[obj(target)]);
        attack_with(&mut t, &[(bilbo, Entity::Player(P1))]);
        // Stomp ({1}{R}) costs {1} less cast from the graveyard.
        mana(&mut t, P0, ManaType::R, 1);
        t.resolve();
        assert_eq!(t.stack_len(), 1, "Stomp was cast");
        if !resolves {
            move_to(&mut t, target, Zone::Exile);
        }
        t.resolve_all();
        assert_eq!(t.zone(giant), Zone::Exile);
        t.set_step(P0, Step::PostcombatMain);
        mana(&mut t, P0, ManaType::R, 1);
        mana(&mut t, P0, ManaType::C, 2);
        assert_eq!(can_cast(&mut t, P0, giant, CastMethod::Normal), resolves);
    }
}
