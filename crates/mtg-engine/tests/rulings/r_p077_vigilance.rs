//! Rulings batch P077 — other rulings of cards that give vigilance: casting from exile and
//! the graveyard, untap restrictions that follow the creature, face-down manifests, block
//! limits, triggers of tokens, choices on resolution, targets and divisions on casting,
//! mana that isn't lost, and the "choose a creature type" choice.

use crate::r_p077_common::*;
use crate::r_s01_common::{stack_library, supported, with_subtype};
use crate::r_s02_common::{can_play_land, destroy};
use crate::r_s06_common::{activate_containing, attach_new, give_control, has_kw};
use crate::r_s08_common::is_tapped;
use crate::r_s21_common::castable;
use crate::r_s25_common::cast_new;
use crate::r_s29_common::put_counters;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::mana::ManaType;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::zones;
use mtg_engine::*;

/// P0 activates Vivien's −2 with `top_first` on top of their library, exiling the first.
fn vivien_exiles(t: &mut TestGame, top_first: &[&str]) -> (ObjectId, ObjectId) {
    let cards = stack_library(t, P0, top_first);
    let vivien = t.battlefield(P0, "Vivien, Champion of the Wilds");
    t.answer_choose(P0, &[Entity::Object(cards[0])]);
    activate_containing(t, P0, vivien, "Look at the top three").expect("−2");
    t.resolve_all();
    let exiled = t.g.current(cards[0]);
    assert_eq!(t.zone(exiled), Zone::Exile);
    assert!(t.g.obj(exiled).face_down);
    (vivien, exiled)
}

#[test]
fn vivien_must_exile_a_card_even_without_creatures_and_may_look_at_it() {
    cr!("406.3", "608.2c");
    ruling!(
        "Vivien, Champion of the Wilds",
        "While resolving Vivien's last ability, you must exile one of the three cards, even if none of them are creature cards. You may look at the exiled card at any time, even if it's not a creature card."
    );
    supported("Vivien, Champion of the Wilds");
    let mut t = TestGame::new(2);
    let size = t.library_size(P0) + 3;
    // No answer: the default still exiles one.
    stack_library(&mut t, P0, &["Shock", "Lightning Bolt", "Island"]);
    let vivien = t.battlefield(P0, "Vivien, Champion of the Wilds");
    activate_containing(&mut t, P0, vivien, "Look at the top three").expect("−2");
    t.resolve_all();
    assert_eq!(t.library_size(P0), size - 1);
    let exiled: Vec<ObjectId> = t.g.exile.clone();
    assert_eq!(exiled.len(), 1);
    assert!(t.g.obj(exiled[0]).face_down);
    assert!(zones::may_look(&t.g, P0, exiled[0]));
    assert!(!zones::may_look(&t.g, P1, exiled[0]));
    // It isn't a creature card: it can't be cast.
    t.lands(P0, "Mountain", 1);
    assert!(!castable(&mut t, P0, exiled[0]));
}

#[test]
fn vivien_exiled_card_cast_once_follows_timing_without_vivien() {
    cr!("400.7", "601.3", "307.1");
    ruling!(
        "Vivien, Champion of the Wilds",
        "Once you cast the creature card, it leaves exile. You can't cast it again if that card becomes exiled again."
    );
    ruling!(
        "Vivien, Champion of the Wilds",
        "You must follow any applicable timing permissions and restrictions if you cast the exiled card. This means that if Vivien leaves the battlefield, you can't cast the creature spell as though it had flash unless another effect allows you to do so."
    );
    supported("Vivien, Champion of the Wilds");
    let mut t = TestGame::new(2);
    let (vivien, bears) = vivien_exiles(&mut t, &["Grizzly Bears", "Shock", "Island"]);
    t.lands(P0, "Forest", 4);
    // With Vivien: as though it had flash, during the opponent's turn.
    t.set_step(P1, Step::Upkeep);
    assert!(castable(&mut t, P0, bears));
    // Without Vivien: only at sorcery speed.
    destroy(&mut t, vivien);
    assert!(!castable(&mut t, P0, bears));
    t.set_step(P0, Step::PrecombatMain);
    assert!(castable(&mut t, P0, bears));
    t.cast(P0, bears).go();
    t.resolve_all();
    let on_bf = t.g.current(bears);
    assert_eq!(t.zone(on_bf), Zone::Battlefield);
    // Exiled again: a new object that can't be cast.
    let again = crate::r_s05_common::move_to(&mut t, on_bf, Zone::Exile).unwrap();
    assert!(!castable(&mut t, P0, again));
}

#[test]
fn shield_of_the_righteous_follows_the_blocked_creature_to_its_new_controller() {
    cr!("502.3", "509.1");
    ruling!(
        "Shield of the Righteous",
        "Shield of the Righteous’s second ability doesn’t track the blocked creature’s controller. If that creature changes controllers before its old controller’s next untap step, this ability will prevent it from untapping during its new controller’s next untap step."
    );
    supported("Shield of the Righteous");
    let mut t = TestGame::new(2);
    let wall = t.battlefield(P0, "Hill Giant");
    attach_new(&mut t, P0, "Shield of the Righteous", wall);
    let bears = t.battlefield(P1, "Grizzly Bears");
    t.set_step(P1, Step::BeginningOfCombat);
    crate::r_s03_common::to_blockers(&mut t, &[(bears, Entity::Player(P0))], &[(wall, bears)]);
    t.resolve_all();
    give_control(&mut t, bears, P0);
    assert_eq!(t.obj_now(bears).controller, P0);
    assert!(is_tapped(&t, bears));
    // P0's next untap step: it doesn't untap.
    t.advance_to(P0, Step::Upkeep);
    assert!(is_tapped(&t, bears));
    // The one after does.
    t.advance_to(P1, Step::Upkeep);
    t.advance_to(P0, Step::Upkeep);
    assert!(!is_tapped(&t, bears));
}

#[test]
fn soul_strike_technique_manifest_is_a_plain_2_2_that_effects_can_change() {
    cr!("701.40a", "708.2a");
    ruling!(
        "Soul-Strike Technique",
        "The face-down permanent is a 2/2 creature with no name, mana cost, creature types, or abilities. It’s colorless and has a mana value of 0. Other effects that apply to the permanent can still grant or change any of these characteristics."
    );
    supported("Soul-Strike Technique");
    let mut t = TestGame::new(2);
    stack_library(&mut t, P0, &["Serra Angel"]);
    let bears = t.battlefield(P0, "Grizzly Bears");
    attach_new(&mut t, P0, "Soul-Strike Technique", bears);
    destroy(&mut t, bears);
    t.resolve_all();
    let fd: Vec<ObjectId> =
        t.g.permanents()
            .filter(|o| o.face_down)
            .map(|o| o.id)
            .collect();
    assert_eq!(fd.len(), 1);
    assert!(crate::r_s11_common::is_plain_face_down_2_2(&t, fd[0]));
    // Glorious Anthem: "Creatures you control get +1/+1."
    t.battlefield(P0, "Glorious Anthem");
    assert_eq!(t.pt(fd[0]), (3, 3));
}

#[test]
fn wrenn_emblem_follows_timing_and_costs() {
    cr!("305.2", "307.1", "601.2f");
    ruling!(
        "Wrenn and Realmbreaker",
        "The lands you play and spells you cast from your graveyard follow the usual timing restrictions, and you must pay any costs for spells you cast."
    );
    supported("Wrenn and Realmbreaker");
    let mut t = TestGame::new(2);
    let wrenn = t.battlefield(P0, "Wrenn and Realmbreaker");
    put_counters(&mut t, wrenn, "loyalty", 3);
    activate_containing(&mut t, P0, wrenn, "emblem").expect("−7");
    t.resolve_all();
    let bears = t.graveyard(P0, "Grizzly Bears");
    let f1 = t.graveyard(P0, "Forest");
    let f2 = t.graveyard(P0, "Forest");
    // No mana: it can't be cast.
    assert!(!castable(&mut t, P0, bears));
    t.lands(P0, "Forest", 2);
    assert!(castable(&mut t, P0, bears));
    // Not at instant speed, nor during the opponent's turn.
    t.set_step(P1, Step::PrecombatMain);
    assert!(!castable(&mut t, P0, bears));
    assert!(!can_play_land(&mut t, P0, f1));
    // One land per turn, as usual.
    t.set_step(P0, Step::PrecombatMain);
    t.play_land(P0, f1)
        .expect("plays a land from the graveyard");
    assert!(!can_play_land(&mut t, P0, f2));
}

#[test]
fn two_brave_the_sands_let_each_creature_block_three() {
    cr!("509.1b", "509.1c");
    ruling!(
        "Brave the Sands",
        "The last ability is cumulative. If you control an additional Brave the Sands, your creatures will each be able to block three creatures, and so on."
    );
    supported("Brave the Sands");
    let mut t = TestGame::new(2);
    let a: Vec<ObjectId> = (0..4).map(|_| t.battlefield(P0, "Grizzly Bears")).collect();
    let wall = t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Brave the Sands");
    crate::r_s01_common::attack_with(
        &mut t,
        &a.iter()
            .map(|x| (*x, Entity::Player(P1)))
            .collect::<Vec<_>>(),
    );
    let blocks = |n: usize| a[..n].iter().map(|x| (wall, *x)).collect::<Vec<_>>();
    let legal = crate::r_s21_common::legal_blocks;
    assert!(legal(&mut t, P1, &blocks(2)));
    assert!(!legal(&mut t, P1, &blocks(3)));
    t.battlefield(P1, "Brave the Sands");
    assert!(legal(&mut t, P1, &blocks(3)));
    assert!(!legal(&mut t, P1, &blocks(4)));
}

/// Valor's Reach Tag Team (the back face of Invasion of Kylem), compiled from its text.
fn tag_team_tokens(t: &mut TestGame) -> Vec<ObjectId> {
    let c = mtg_engine::card::card("Invasion of Kylem // Valor's Reach Tag Team");
    let text = c.faces[1].chars.rules_text.to_string();
    let def = crate::r_s01_common::custom_card(
        "Valor's Reach Tag Team",
        "Sorcery",
        "{2}{R}{W}",
        None,
        &text,
    );
    let spell = t.custom(P0, def, Zone::Hand(P0));
    t.lands(P0, "Plateau", 4);
    t.cast(P0, spell).go();
    t.resolve_all();
    let w = with_subtype(t, P0, "Warrior");
    for id in &w {
        // Not summoning sick: they've been under P0's control since the turn began.
        t.g.objects[id.0 as usize].summoning_sick = false;
    }
    w
}

#[test]
fn tag_team_tokens_trigger_attacking_different_players_and_on_a_nontoken_copy() {
    cr!("508.1", "603.2", "707.2", "506.2");
    ruling!(
        "Invasion of Kylem // Valor's Reach Tag Team",
        "The member of World's Greatest Valor's Reach Tag Team and the other creature token don't have to be attacking the same player, planeswalker, or battle for the ability to trigger."
    );
    ruling!(
        "Invasion of Kylem // Valor's Reach Tag Team",
        "The tokens' triggered ability also works if a nontoken creature somehow gains it. For example, if a nontoken creature becomes a copy of the token and then attacks alongside a creature token, the ability will trigger."
    );
    let mut t = TestGame::new(3);
    let w = tag_team_tokens(&mut t);
    assert_eq!(w.len(), 2);
    assert_eq!(t.pt(w[0]), (3, 2));
    crate::r_s01_common::attack_with(
        &mut t,
        &[(w[0], Entity::Player(P1)), (w[1], Entity::Player(P2))],
    );
    t.resolve_all();
    assert_eq!(t.counters(w[0], "+1/+1"), 1);
    assert_eq!(t.counters(w[1], "+1/+1"), 1);

    let mut t = TestGame::new(2);
    let w = tag_team_tokens(&mut t);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s17_common::become_copy(&mut t, bears, w[0]);
    assert!(!t.obj_now(bears).is_token());
    // The nontoken copy attacks alongside one creature token.
    crate::r_s01_common::attack_with(
        &mut t,
        &[(bears, Entity::Player(P1)), (w[1], Entity::Player(P1))],
    );
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    // The token's own ability needs another creature token: the copy isn't one.
    assert_eq!(t.counters(w[1], "+1/+1"), 0);
}

#[test]
fn ajani_goldmane_avatar_and_vigilance() {
    cr!("611.3a", "604.3", "611.2a");
    ruling!(
        "Ajani Goldmane",
        "The power and toughness of the Avatar created by the third ability will change as your life total changes."
    );
    ruling!(
        "Ajani Goldmane",
        "The vigilance granted to a creature by the second ability remains until the end of the turn even if the +1/+1 counter is removed."
    );
    supported("Ajani Goldmane");
    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani Goldmane");
    put_counters(&mut t, ajani, "loyalty", 2);
    activate_containing(&mut t, P0, ajani, "Avatar").expect("−6");
    t.resolve_all();
    let avatar = with_subtype(&t, P0, "Avatar")[0];
    assert_eq!(t.pt(avatar), (20, 20));
    t.g.gain_life(P0, 3);
    t.g.recompute();
    assert_eq!(t.pt(avatar), (23, 23));
    t.g.lose_life(P0, 10);
    t.g.recompute();
    assert_eq!(t.pt(avatar), (13, 13));

    let mut t = TestGame::new(2);
    let ajani = t.battlefield(P0, "Ajani Goldmane");
    let bears = t.battlefield(P0, "Grizzly Bears");
    activate_containing(&mut t, P0, ajani, "+1/+1 counter").expect("−1");
    t.resolve_all();
    assert_eq!(t.counters(bears, "+1/+1"), 1);
    assert!(vig(&t, bears));
    t.g.remove_counters(Entity::Object(t.g.current(bears)), "+1/+1", 1);
    t.g.recompute();
    assert_eq!(t.counters(bears, "+1/+1"), 0);
    assert!(vig(&t, bears));
    // Until end of turn.
    t.advance_to(P1, Step::Upkeep);
    assert!(!vig(&t, bears));
}

#[test]
fn three_blind_mice_copies_the_tokens_original_characteristics() {
    cr!("707.2", "111.3");
    ruling!(
        "Three Blind Mice",
        "The token copies the original characteristics of the target token as stated by the effect that created the token."
    );
    supported("Three Blind Mice");
    let mut t = TestGame::new(2);
    let saga = t.battlefield(P0, "Three Blind Mice");
    crate::r_s19_common::add_lore(&mut t, saga, 1);
    t.resolve_all();
    let mouse = with_subtype(&t, P0, "Mouse")[0];
    put_counters(&mut t, mouse, "+1/+1", 2);
    // Turned into a 4/4 Elemental by an effect.
    crate::r_s26_common::modify_until_eot(
        &mut t,
        mouse,
        vec![
            mtg_engine::ability::Modification::SetPT(
                Some(mtg_engine::ability::Value::c(4)),
                Some(mtg_engine::ability::Value::c(4)),
            ),
            mtg_engine::ability::Modification::AddSubtypes(vec!["Elemental".into()]),
        ],
    );
    assert_eq!(t.pt(mouse), (6, 6));
    t.answer_targets(P0, &[Entity::Object(mouse)]);
    crate::r_s19_common::add_lore(&mut t, saga, 1);
    t.resolve_all();
    let mice = with_subtype(&t, P0, "Mouse");
    assert_eq!(mice.len(), 2);
    let copy = *mice.iter().find(|m| **m != t.g.current(mouse)).unwrap();
    let o = t.obj_now(copy);
    assert_eq!(o.chars.name, "Mouse Token");
    assert_eq!(t.pt(copy), (1, 1));
    assert!(!o.chars.has_subtype("Elemental"));
    assert_eq!(o.chars.colors, ColorSet::single(Color::White));
    assert_eq!(t.counters(copy, "+1/+1"), 0);
}

#[test]
fn thranduils_company_is_cumulative_with_exploration() {
    cr!("305.2", "305.2a");
    ruling!(
        "Thranduil's Company",
        "Thranduil's Company's first ability is cumulative with other effects that let you play additional lands, such as the one from Exploration."
    );
    supported("Thranduil's Company");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Thranduil's Company");
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Exploration");
    let lands: Vec<ObjectId> = (0..4).map(|_| t.hand(P0, "Forest")).collect();
    for l in &lands[..3] {
        t.play_land(P0, *l).expect("a land drop");
        t.resolve_all();
    }
    assert!(!can_play_land(&mut t, P0, lands[3]));
}

#[test]
fn choose_a_creature_type_offers_single_existing_creature_types() {
    cr!("205.3m");
    ruling!(
        "And They Shall Know No Fear",
        "To choose a creature type, you must choose an existing creature type, such as Vampire or Knight."
    );
    ruling!(
        "Radiant Destiny",
        "To choose a creature type, you must choose an existing creature type, such as Vampire or Knight."
    );
    ruling!(
        "Rukarumel, Biologist",
        "To choose a creature type, you must choose an existing creature type, such as Vampire or Knight."
    );
    ruling!(
        "Titan of Littjara",
        "To choose a creature type, you must choose an existing creature type, such as Vampire or Knight."
    );
    ruling!(
        "Unclaimed Territory",
        "To choose a creature type, you must choose an existing creature type, such as Vampire or Knight."
    );
    for name in [
        "And They Shall Know No Fear",
        "Radiant Destiny",
        "Rukarumel, Biologist",
        "Titan of Littjara",
        "Unclaimed Territory",
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        let from = t.asked().len();
        if name == "And They Shall Know No Fear" {
            cast_new(&mut t, P0, name, &[]);
            t.resolve_all();
        } else {
            t.enter(P0, name);
            t.resolve_all();
        }
        let options: Vec<Vec<String>> = t.asked()[from..]
            .iter()
            .filter_map(|(p, d)| match d {
                Decision::ChooseOption {
                    prompt, options, ..
                } if *p == P0 && prompt.contains("creature type") => Some(options.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(options.len(), 1, "{name}: one creature type choice");
        let o = &options[0];
        for ok in ["Vampire", "Knight", "Sliver"] {
            assert!(o.iter().any(|x| x == ok), "{name}: {ok}");
        }
        for bad in [
            "Vampire Knight",
            "Artifact",
            "Creature",
            "Jace",
            "Vehicle",
            "Treasure",
        ] {
            assert!(!o.iter().any(|x| x == bad), "{name}: {bad}");
        }
    }
}

#[test]
fn natures_way_trample_is_only_for_combat_damage_and_needs_both_targets() {
    cr!("702.19b", "601.2c", "115.1");
    ruling!(
        "Nature's Way",
        "Trample only applies to the assignment of combat damage, not the damage Nature's Way has the creature deal."
    );
    ruling!(
        "Nature's Way",
        "You can't cast Nature's Way unless you choose both a creature you control and a creature you don't control as targets."
    );
    supported("Nature's Way");
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P0, "Hill Giant");
    let way = crate::r_s03_common::in_hand_with_mana(&mut t, P0, "Nature's Way");
    // No creature you don't control: it can't be cast.
    assert!(!castable(&mut t, P0, way));
    let bears = t.battlefield(P1, "Grizzly Bears");
    assert!(castable(&mut t, P0, way));
    t.cast(P0, way)
        .target(Entity::Object(giant))
        .target(Entity::Object(bears))
        .go();
    t.resolve_all();
    assert!(has_kw(&t, giant, KeywordKind::Trample));
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.life(P1), 20, "no damage trampled over");
}

#[test]
fn vihaan_treasures_keep_their_abilities_as_creatures() {
    cr!("613.1d", "613.1f", "702.10");
    ruling!(
        "Vihaan, Goldwaker",
        "Treasures you control will keep any abilities they previously had while they’re creatures."
    );
    supported("Vihaan, Goldwaker");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Vihaan, Goldwaker");
    let treasure = crate::r_s02_common::create_token(&mut t, P0, "Treasure");
    t.answer_yes(P0, true);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(crate::r_s04_common::is_creature(&t, treasure));
    assert_eq!(t.pt(treasure), (3, 3));
    assert!(t.obj_now(treasure).chars.has_subtype("Treasure"));
    assert!(t.obj_now(treasure).chars.has_subtype("Assassin"));
    // Its mana ability is still there, and (an outlaw with haste) it can be activated.
    activate_containing(&mut t, P0, treasure, "Sacrifice").expect("the Treasure's ability");
    assert!(!t.on_battlefield(treasure));
    assert_eq!(t.g.player(P0).mana_pool.total(), 1);
}

#[test]
fn makindi_patrol_vigilance_after_attacks_doesnt_untap() {
    cr!("702.20b", "508.1f");
    ruling!(
        "Makindi Patrol",
        "Vigilance matters only as attackers are declared. Giving a creature vigilance after that point won’t untap it, even if it’s attacking."
    );
    supported("Makindi Patrol");
    let mut t = TestGame::new(2);
    let bears = t.battlefield(P0, "Grizzly Bears");
    crate::r_s01_common::attack_with(&mut t, &[(bears, Entity::Player(P1))]);
    assert!(is_tapped(&t, bears));
    t.enter(P0, "Makindi Patrol");
    t.resolve_all();
    assert!(vig(&t, bears));
    assert!(is_tapped(&t, bears));
}

#[test]
fn extricator_of_sin_sacrifices_only_one_permanent() {
    cr!("608.2d", "701.21a");
    ruling!(
        "Extricator of Sin // Extricator of Flesh",
        "While resolving Extricator of Sin's first ability, you can't sacrifice multiple permanents to get multiple Eldrazi Horror tokens."
    );
    supported("Extricator of Sin // Extricator of Flesh");
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Ornithopter");
    let b = t.battlefield(P0, "Ornithopter");
    t.answer_yes(P0, true);
    t.answer_choose(P0, &[Entity::Object(a), Entity::Object(b)]);
    t.enter(P0, "Extricator of Sin // Extricator of Flesh");
    t.resolve_all();
    assert_eq!(t.named_on_battlefield("Ornithopter").len(), 1);
    assert_eq!(with_subtype(&t, P0, "Horror").len(), 1);
}

#[test]
fn fangorn_keeps_unspent_green_mana_only() {
    cr!("106.4");
    ruling!(
        "Fangorn, Tree Shepherd",
        "You can keep unspent green mana indefinitely while Fangorn, Tree Shepherd is on the battlefield."
    );
    supported("Fangorn, Tree Shepherd");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Fangorn, Tree Shepherd");
    let pool = |t: &TestGame| {
        let m = &t.g.player(P0).mana_pool;
        (m.count(ManaType::G), m.count(ManaType::R))
    };
    t.g.players[0].mana_pool.add_type(ManaType::G, 2);
    t.g.players[0].mana_pool.add_type(ManaType::R, 1);
    t.advance_to(P0, Step::BeginningOfCombat);
    assert_eq!(pool(&t), (2, 0));
    // Even a later turn.
    t.advance_to(P0, Step::Upkeep);
    assert_eq!(pool(&t), (2, 0));
    // And it can be spent then.
    t.set_step(P0, Step::PrecombatMain);
    let bears = t.hand(P0, "Grizzly Bears");
    t.cast(P0, bears).go();
    assert_eq!(pool(&t), (0, 0));
}

#[test]
fn distributing_counters_chooses_targets_and_division_on_casting() {
    cr!("601.2c", "601.2d", "115.7");
    ruling!(
        "Storm the Seedcore",
        "You choose how many targets Storm the Seedcore has and how the counters will be distributed as you cast the spell. Each target must receive at least one counter."
    );
    ruling!(
        "Undercity Upheaval",
        "You choose how many targets Undercity Upheaval has and how the counters will be distributed as you cast the spell. Each target must receive at least one counter."
    );
    // (card, creature cards in the graveyard, division)
    for (name, yard, split) in [
        ("Storm the Seedcore", 0, [3, 1]),
        ("Undercity Upheaval", 3, [2, 1]),
    ] {
        supported(name);
        let mut t = TestGame::new(2);
        for _ in 0..yard {
            t.graveyard(P0, "Grizzly Bears");
        }
        let a = t.battlefield(P0, "Grizzly Bears");
        let b = t.battlefield(P0, "Hill Giant");
        let spell = crate::r_s03_common::in_hand_with_mana(&mut t, P0, name);
        let from = t.asked().len();
        crate::r_s29_common::divide(&mut t, P0, &split);
        let id = t
            .cast(P0, spell)
            .targets(&[Entity::Object(a), Entity::Object(b)])
            .go();
        // Both were chosen while casting: the division was asked with each target getting
        // at least one.
        let divides: Vec<(u32, u32, usize)> = t.asked()[from..]
            .iter()
            .filter_map(|(_, d)| match d {
                Decision::Divide {
                    total,
                    min_each,
                    recipients,
                    ..
                } => Some((*total, *min_each, recipients.len())),
                _ => None,
            })
            .collect();
        assert_eq!(
            divides,
            vec![((split[0] + split[1]) as u32, 1, 2)],
            "{name}"
        );
        assert_eq!(t.zone(id), Zone::Stack);
        t.resolve_all();
        assert_eq!(t.counters(a, "+1/+1"), split[0] as u32, "{name}");
        assert_eq!(t.counters(b, "+1/+1"), split[1] as u32, "{name}");
        assert!(vig(&t, a) && vig(&t, b));
    }
}

#[test]
fn steel_seraph_target_on_stack_ability_on_resolution() {
    cr!("603.3d", "608.2d");
    ruling!(
        "Steel Seraph",
        "You choose the target creature as you put Steel Seraph's triggered ability on the stack. You choose which ability that creature gains as the ability resolves."
    );
    supported("Steel Seraph");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Steel Seraph");
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.advance_to(P0, Step::BeginningOfCombat);
    let from = t.asked().len();
    t.settle();
    assert_eq!(t.stack_len(), 1);
    let top = crate::r_s04_common::top_of_stack(&t);
    assert_eq!(
        crate::r_s25_common::targets_of(&t, top),
        vec![Entity::Object(bears)]
    );
    let options = |t: &TestGame, from: usize| {
        t.asked()[from..]
            .iter()
            .filter(|(_, d)| matches!(d, Decision::ChooseOption { .. }))
            .count()
    };
    assert_eq!(options(&t, from), 0, "the ability isn't chosen yet");
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.resolve_all();
    assert_eq!(options(&t, from), 1);
    assert!(vig(&t, bears));
}

#[test]
fn knightly_valor_needs_a_target_and_doesnt_enchant_its_token() {
    cr!("303.4a", "601.2c");
    ruling!(
        "Knightly Valor",
        "You need a creature for Knightly Valor to target as you cast it. There's no way to have it enter the battlefield attached to the Knight token it'll create."
    );
    supported("Knightly Valor");
    let mut t = TestGame::new(2);
    let aura = crate::r_s03_common::in_hand_with_mana(&mut t, P0, "Knightly Valor");
    assert!(!castable(&mut t, P0, aura));
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.cast(P0, aura).target(Entity::Object(bears)).go();
    t.resolve_all();
    let knight = with_subtype(&t, P0, "Knight")[0];
    assert!(vig(&t, knight));
    assert_eq!(t.pt(knight), (2, 2));
    assert_eq!(
        crate::r_s06_common::attached_to(&t, aura),
        Some(Entity::Object(bears))
    );
    assert_eq!(t.pt(bears), (4, 4));
}
