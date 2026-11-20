//! Item `rulings-residue-misc-1`, pass B — rulings about control, players leaving the game
//! and attacking (batch P065, "gains haste"): Alexios, Deimos of Kosmos changing hands each
//! upkeep, Rakshasa Debaser's reanimated creatures, defending players, Eumidian
//! Wastewaker, Tromokratis, Risona and Giant Slug.

use crate::r_p053_common::concede;
use crate::r_residue_misc_1_common::*;
use mtg_engine::ability::*;
use mtg_engine::decision::{Answer, Decision};
use mtg_engine::keywords::KeywordKind;
use mtg_engine::object::Zone;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

const ALEXIOS: &str = "Alexios, Deimos of Kosmos";

/// P0 declares no attackers in a fresh combat of P0's turn; the attackers that result.
fn attack_with_none(t: &mut TestGame) -> Vec<ObjectId> {
    t.g.combat = None;
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(P0, DecisionKind::Attackers, Answer::Attackers(vec![]));
    t.advance_to(P0, Step::DeclareAttackers);
    t.g.permanents()
        .filter(|o| t.g.is_attacking(o.id))
        .map(|o| o.id)
        .collect()
}

#[test]
fn alexios_attacks_only_if_able_and_without_paying_a_cost() {
    cr!("508.1d", "508.1h");
    ruling!(
        "Alexios, Deimos of Kosmos",
        "If Alexios can't attack for any reason (such as being tapped), then it doesn't attack. If there's a cost associated with having it attack, its controller isn't forced to pay that cost, so it doesn't have to attack in that case either."
    );
    // Untapped, no cost: it attacks though none was declared.
    let mut t = TestGame::new(2);
    let alexios = t.battlefield(P0, ALEXIOS);
    assert_eq!(attack_with_none(&mut t), vec![alexios]);
    // Tapped: it doesn't.
    let mut t = TestGame::new(2);
    let alexios = t.battlefield(P0, ALEXIOS);
    t.g.objects[alexios.0 as usize].tapped = true;
    assert!(attack_with_none(&mut t).is_empty());
    // Ghostly Prison's cost: it needn't be paid, though P0 could.
    let mut t = TestGame::new(2);
    t.battlefield(P0, ALEXIOS);
    t.battlefield(P1, "Ghostly Prison");
    t.lands(P0, "Plains", 2);
    assert!(attack_with_none(&mut t).is_empty());
}

#[test]
fn alexios_returns_to_the_most_recent_controller_still_in_the_game() {
    cr!("800.4a", "613.7", "611.2a");
    ruling!(
        "Alexios, Deimos of Kosmos",
        "If Alexios is controlled by a player other than its owner and its controller leaves the game, the effect giving that player control of Alexios ends. Alexios will return to the control of the player still in the game who most recently controlled it."
    );
    let mut t = TestGame::new(3);
    let alexios = t.battlefield(P0, ALEXIOS);
    t.set_step(P1, Step::Untap);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(controller(&mut t, alexios), P1);
    t.set_step(P2, Step::Untap);
    t.advance_to(P2, Step::Upkeep);
    t.resolve_all();
    assert_eq!(controller(&mut t, alexios), P2);
    concede(&mut t, P2);
    t.g.recompute();
    assert!(t.on_battlefield(alexios));
    assert_eq!(controller(&mut t, alexios), P1);
}

#[test]
fn alexios_leaves_with_its_owner() {
    cr!("800.4a");
    ruling!(
        "Alexios, Deimos of Kosmos",
        "If Alexios's owner leaves the game, Alexios leaves the game along with them."
    );
    let mut t = TestGame::new(3);
    let alexios = t.battlefield(P0, ALEXIOS);
    t.set_step(P1, Step::Untap);
    t.advance_to(P1, Step::Upkeep);
    t.resolve_all();
    assert_eq!(controller(&mut t, alexios), P1);
    concede(&mut t, P0);
    assert!(!t.on_battlefield(alexios));
    assert!(t.named_on_battlefield(ALEXIOS).is_empty());
}

#[test]
fn alexios_cant_be_sacrificed() {
    cr!("701.21a", "101.3");
    ruling!(
        "Alexios, Deimos of Kosmos",
        "If an effect instructs you to sacrifice a creature and you control any creatures other than Alexios, you must sacrifice one of those other creatures. You can't try to sacrifice Alexios."
    );
    // Diabolic Edict: "Target player sacrifices a creature."
    let mut t = TestGame::new(2);
    let alexios = t.battlefield(P1, ALEXIOS);
    let bears = t.battlefield(P1, "Grizzly Bears");
    let from = t.asked().len();
    cast_resolve(&mut t, P0, "Diabolic Edict", &[P1.into()]);
    assert!(t.on_battlefield(alexios));
    assert!(!t.on_battlefield(bears));
    for (_, d) in &t.asked()[from..] {
        if let Decision::ChooseEntities { candidates, .. } = d {
            assert!(!candidates.contains(&obj(alexios)));
        }
    }
    // Alone, it isn't sacrificed.
    let mut t = TestGame::new(2);
    let alexios = t.battlefield(P1, ALEXIOS);
    cast_resolve(&mut t, P0, "Diabolic Edict", &[P1.into()]);
    assert!(t.on_battlefield(alexios));
}

#[test]
fn attacking_a_planeswalker_its_controller_is_the_defending_player() {
    cr!("506.2", "508.5");
    ruling!(
        "Armix, Filigree Thrasher",
        "If a creature is attacking a planeswalker, that planeswalker's controller is the defending player."
    );
    let mut t = TestGame::new(3);
    let armix = t.battlefield(P0, "Armix, Filigree Thrasher");
    let walker = t.battlefield(P2, "Jace Beleren");
    let p1_bears = t.battlefield(P1, "Grizzly Bears");
    let p2_bears = t.battlefield(P2, "Grizzly Bears");
    t.hand(P0, "Lightning Bolt");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.attack(&[(armix, obj(walker))], &[]);
    let offered = crate::r_s02_common::target_candidates(&t, P0, from);
    assert!(!offered.is_empty());
    assert!(offered
        .iter()
        .all(|c| c.contains(&obj(p2_bears)) && !c.contains(&obj(p1_bears))));
}

#[test]
fn rakshasa_debasers_creatures_are_exiled_when_their_controller_leaves() {
    cr!("800.4a", "508.4");
    ruling!(
        "Rakshasa Debaser",
        "If you leave the game, all creatures owned by other players that were put onto the battlefield under your control are exiled."
    );
    ruling!(
        "Rakshasa Debaser",
        "The creatures put onto the battlefield by the triggered ability can't attack that combat, even if they have haste."
    );
    let mut t = TestGame::new(3);
    let debaser = t.battlefield(P0, "Rakshasa Debaser");
    // Raging Goblin has haste.
    let goblin = t.graveyard(P1, "Raging Goblin");
    t.answer_targets(P0, &[obj(goblin)]);
    t.attack(&[(debaser, P1.into())], &[]);
    let goblin = t.g.current(goblin);
    assert!(t.on_battlefield(goblin));
    assert_eq!(controller(&mut t, goblin), P0);
    assert!(!is_attacking(&t, goblin));
    assert_eq!(t.life(P1), 14, "only the Debaser dealt damage");
    concede(&mut t, P0);
    assert_eq!(t.zone(goblin), Zone::Exile);
}

/// Eumidian Wastewaker attacks P1; P0 answers its own "discard or sacrifice" choice with
/// `p0` (0: discard, 1: sacrifice) and the card or permanent `p0_pick`; P1 answers with
/// `p1` and `p1_pick` (`None`: not asked). Returns the decisions asked during the trigger.
fn wastewaker(
    t: &mut TestGame,
    p0: (usize, ObjectId),
    p1: Option<(usize, ObjectId)>,
) -> Vec<(PlayerId, Decision)> {
    let w = t.battlefield(P0, "Eumidian Wastewaker");
    t.answer(P0, DecisionKind::Option, Answer::Index(p0.0));
    t.answer_choose(P0, &[obj(p0.1)]);
    if let Some((i, pick)) = p1 {
        t.answer(P1, DecisionKind::Option, Answer::Index(i));
        t.answer_choose(P1, &[obj(pick)]);
    }
    let from = t.asked().len();
    t.attack(&[(w, P1.into())], &[]);
    t.asked()[from..]
        .iter()
        .filter(|(_, d)| {
            !matches!(
                d,
                Decision::Priority { .. } | Decision::DeclareAttackers { .. }
            )
        })
        .cloned()
        .collect()
}

#[test]
fn eumidian_wastewaker_you_choose_first_and_a_player_who_cant_discard_sacrifices() {
    cr!("608.2d", "101.4");
    ruling!(
        "Eumidian Wastewaker",
        "you choose to discard a card or sacrifice a permanent, and then defending player chooses to discard a card or sacrifice a permanent. If one of those players can’t discard a card (such as by having no cards in hand), that player must sacrifice a permanent."
    );
    let mut t = TestGame::new(2);
    let forest = t.hand(P0, "Forest");
    let mountain = t.battlefield(P1, "Mountain");
    let asked = wastewaker(&mut t, (0, forest), None);
    assert_eq!(t.zone(forest), Zone::Graveyard(P0));
    // P1 has no cards in hand: only sacrificing is offered, and P1 sacrifices.
    assert_eq!(t.zone(mountain), Zone::Graveyard(P1));
    assert_eq!(asked.first().map(|(p, _)| *p), Some(P0));
    let p1_options: Vec<Vec<String>> = asked
        .iter()
        .filter_map(|(p, d)| match d {
            Decision::ChooseOption { options, .. } if *p == P1 => Some(options.clone()),
            _ => None,
        })
        .collect();
    assert!(p1_options.iter().all(|o| o.len() == 1), "{p1_options:?}");
    // Two land cards put into graveyards: two cards drawn.
    assert_eq!(t.hand_size(P0), 2);
}

/// A Forest land token for `p`.
fn land_token(t: &mut TestGame, p: PlayerId) -> ObjectId {
    let before = t.g.battlefield.clone();
    let mut ctx = mtg_engine::eval::Ctx::new(None, p);
    t.g.exec(
        &Effect::CreateToken {
            spec: TokenSpec {
                name: "Forest".into(),
                colors: ColorSet::NONE,
                supertypes: vec![],
                card_types: vec![CardType::Land],
                subtypes: vec!["Forest".into()],
                power: None,
                toughness: None,
                abilities: vec![],
                scryfall_name: None,
                pt_values: None,
            },
            count: Value::c(1),
            controller: PlayerRef::You,
            tapped: false,
            attacking: false,
        },
        &mut ctx,
    );
    t.g.recompute();
    *t.g.battlefield
        .iter()
        .find(|id| !before.contains(id))
        .unwrap()
}

#[test]
fn eumidian_wastewaker_counts_only_land_cards_in_graveyards() {
    cr!("111.7", "404.1");
    ruling!(
        "Eumidian Wastewaker",
        "Only count the cards that are land cards in the graveyard. If a player sacrifices a land token, you won’t draw a card for it."
    );
    ruling!(
        "Eumidian Wastewaker",
        "if a player sacrifices Ashaya, Soul of the Wild (which has an ability that makes it a land on the battlefield), it is a creature card in the graveyard, not a land card, so you won’t draw a card for it."
    );
    // A land token (a copy of a Forest) and Ashaya, a Forest land on the battlefield.
    for ashaya in [false, true] {
        let mut t = TestGame::new(2);
        let bolt = t.hand(P0, "Lightning Bolt");
        let victim = if ashaya {
            t.battlefield(P1, "Ashaya, Soul of the Wild")
        } else {
            land_token(&mut t, P1)
        };
        t.g.recompute();
        assert!(t.obj_now(victim).is(CardType::Land));
        t.hand(P1, "Grizzly Bears");
        wastewaker(&mut t, (0, bolt), Some((1, victim)));
        assert!(!t.on_battlefield(victim), "ashaya {ashaya}");
        assert_eq!(t.hand_size(P0), 0, "ashaya {ashaya}: no land card");
    }
}

/// P0's Elemental token from a Rekindling Phoenix copy that died; at P0's next upkeep, the
/// candidates for its "return target card named Rekindling Phoenix" ability.
fn phoenix_token_targets(t: &mut TestGame) -> Vec<Vec<Entity>> {
    assert_eq!(tokens_of(t, P0, "Elemental"), 1);
    t.set_step(P0, Step::Untap);
    let from = t.asked().len();
    t.advance_to(P0, Step::Upkeep);
    t.settle();
    crate::r_s02_common::target_candidates(t, P0, from)
}

#[test]
fn rekindling_phoenix_copies_look_for_a_card_named_rekindling_phoenix() {
    cr!("707.2", "201.4", "115.1");
    ruling!(
        "Rekindling Phoenix",
        "the Elemental token's triggered ability will look for a card named Rekindling Phoenix, not one with the other card's name. This is true even if the card copying Rekindling Phoenix keeps its name while it's copying Rekindling Phoenix (such as Lazav, Dimir Mastermind does)."
    );
    // A Clone copying a Rekindling Phoenix dies.
    let mut t = TestGame::new(2);
    let theirs = t.battlefield(P1, "Rekindling Phoenix");
    t.answer_choose(P0, &[obj(theirs)]);
    let clone = t.enter(P0, "Clone");
    t.resolve_all();
    assert_eq!(name_now(&t, clone), "Rekindling Phoenix");
    destroy(&mut t, clone);
    t.resolve_all();
    let real = t.graveyard(P0, "Rekindling Phoenix");
    let clone_card = gy_named(&t, P0, "Clone")[0];
    let offered = phoenix_token_targets(&mut t);
    assert!(
        offered.iter().any(|c| c.contains(&obj(real))),
        "{offered:?}"
    );
    assert!(offered.iter().all(|c| !c.contains(&obj(clone_card))));
    // Lazav, Dimir Mastermind, a copy of a Phoenix named Lazav, dies.
    let mut t = TestGame::new(2);
    let lazav = t.battlefield(P0, "Lazav, Dimir Mastermind");
    let phoenix = t.battlefield(P1, "Rekindling Phoenix");
    t.answer_yes(P0, true);
    destroy(&mut t, phoenix);
    t.resolve_all();
    assert_eq!(t.pt(lazav), (4, 3));
    let tokens_before = tokens_of(&t, P1, "Elemental");
    assert_eq!(tokens_before, 1, "P1's own Phoenix died");
    destroy(&mut t, lazav);
    t.resolve_all();
    let real = t.graveyard(P0, "Rekindling Phoenix");
    let lazav_card = gy_named(&t, P0, "Lazav, Dimir Mastermind")[0];
    let offered = phoenix_token_targets(&mut t);
    assert!(
        offered.iter().any(|c| c.contains(&obj(real))),
        "{offered:?}"
    );
    assert!(offered.iter().all(|c| !c.contains(&obj(lazav_card))));
}

#[test]
fn bloodfeather_phoenix_needs_the_spell_to_deal_the_damage() {
    cr!("120.3", "603.2");
    ruling!(
        "Bloodfeather Phoenix",
        "a sorcery spell that says \"Target creature you control deals damage equal to its power to any target\" wouldn't cause Bloodfeather Phoenix's ability to trigger even if an opponent or battle was dealt damage, as it's the creature dealing the damage, not the spell."
    );
    ruling!(
        "Bloodfeather Phoenix",
        "You choose whether to pay {R} as the triggered ability is resolving."
    );
    // Soul's Fire: "Target creature you control deals damage equal to its power to any
    // target."
    let mut t = TestGame::new(2);
    let phoenix = t.graveyard(P0, "Bloodfeather Phoenix");
    let bears = t.battlefield(P0, "Grizzly Bears");
    let fire = t.hand(P0, "Soul's Fire");
    lands_for_cost(&mut t, P0, "Soul's Fire");
    t.cast(P0, fire).target(bears).target(P1).go();
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.zone(phoenix), Zone::Graveyard(P0));
    // Lightning Bolt deals the damage itself: P0 pays {R} as the trigger resolves.
    t.lands(P0, "Mountain", 1);
    cast_new(&mut t, P0, "Lightning Bolt", &[P1.into()]);
    t.resolve();
    t.settle();
    assert_eq!(t.stack_len(), 1, "the trigger");
    let from = t.asked().len();
    t.answer_yes(P0, true);
    t.resolve_all();
    assert!(was_asked(&t, from, |d| matches!(
        d,
        Decision::YesNo { .. } | Decision::OptionalCost { .. }
    )));
    let phoenix = t.g.current(phoenix);
    assert!(t.on_battlefield(phoenix));
    assert!(has(&mut t, phoenix, KeywordKind::Haste));
}

#[test]
fn kogla_and_yidaro_without_a_target_or_with_an_illegal_one() {
    cr!("608.2b", "115.10");
    ruling!(
        "Kogla and Yidaro",
        "You don't have to choose a target artifact or enchantment for the last ability. However, if you do, and that permanent is an illegal target at the time the ability tries to resolve, the ability won't resolve and none of its effects will happen."
    );
    // No target: it's shuffled into the library and P0 draws.
    let mut t = TestGame::new(2);
    let kogla = t.hand(P0, "Kogla and Yidaro");
    fill_pool(&mut t, P0, 2);
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, kogla, "Discard").unwrap();
    t.resolve_all();
    assert_eq!(t.zone(kogla), Zone::Library(P0));
    assert_eq!(t.hand_size(P0), hand, "discarded one, drew one");
    // A target that's gone: nothing happens.
    let mut t = TestGame::new(2);
    let kogla = t.hand(P0, "Kogla and Yidaro");
    let orni = t.battlefield(P1, "Bonesplitter");
    fill_pool(&mut t, P0, 2);
    t.answer_targets(P0, &[obj(orni)]);
    let hand = t.hand_size(P0);
    activate_containing(&mut t, P0, kogla, "Discard").unwrap();
    exile_card(&mut t, orni);
    t.resolve_all();
    assert_eq!(t.zone(kogla), Zone::Graveyard(P0));
    assert_eq!(t.hand_size(P0), hand - 1);
}

#[test]
fn mezzio_mugger_lands_follow_the_normal_timing() {
    cr!("305.2", "305.3", "601.3");
    ruling!(
        "Mezzio Mugger",
        "You must follow all normal timing rules for cards played this way. For example, you may play lands this way only during your main phase while no spells or abilities are on the stack and only if you haven't played a land yet this turn."
    );
    let mut t = TestGame::new(2);
    let mugger = t.battlefield(P0, "Mezzio Mugger");
    let land = t.library_top(P0, "Forest");
    t.library_top(P1, "Mountain");
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(mugger, P1.into())]),
    );
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    let land = t.g.current(land);
    assert_eq!(t.zone(land), Zone::Exile);
    // During combat: no.
    assert!(t.play_land(P0, land).is_err());
    t.advance_to(P0, Step::PostcombatMain);
    // In the main phase, with an empty stack: yes.
    assert!(t.play_land(P0, land).is_ok());
    // P1's exiled Mountain: not a second land this turn.
    let mountain =
        t.g.exile
            .iter()
            .copied()
            .find(|c| t.obj_now(*c).chars.name == "Mountain");
    assert!(t.play_land(P0, mountain.unwrap()).is_err());
}

#[test]
fn ziatoras_envoy_if_you_dont_refers_to_playing_the_card() {
    cr!("608.2c", "305.1");
    ruling!(
        "Ziatora's Envoy",
        "“If you don't” in the second ability refers to whether you played a land or cast a spell from the top of your library, not just whether or not you cast a spell."
    );
    for play in [true, false] {
        let mut t = TestGame::new(2);
        let envoy = t.battlefield(P0, "Ziatora's Envoy");
        let forest = t.library_top(P0, "Forest");
        // The card to play from among those that may be played (none: decline).
        let pick = if play { vec![obj(forest)] } else { vec![] };
        t.answer_choose(P0, &pick);
        t.attack(&[(envoy, P1.into())], &[]);
        t.resolve_all();
        let forest = t.g.current(forest);
        if play {
            assert!(t.on_battlefield(forest));
        } else {
            assert_eq!(t.zone(forest), Zone::Hand(P0));
        }
    }
}

#[test]
fn tromokratis_is_attacking_through_the_combat_damage_step() {
    cr!("506.4", "511.3");
    ruling!(
        "Tromokratis",
        "Tromokratis (like all attacking and blocking creatures) remains an attacking or blocking creature until the end of combat step is over."
    );
    let mut t = TestGame::new(2);
    let tromo = t.battlefield(P0, "Tromokratis");
    assert!(has(&mut t, tromo, KeywordKind::Hexproof));
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(tromo, P1.into())]),
    );
    t.advance_to(P0, Step::CombatDamage);
    assert_eq!(t.life(P1), 12);
    assert!(is_attacking(&t, tromo));
    assert!(!has(&mut t, tromo, KeywordKind::Hexproof));
    // An opponent's instant can target it now.
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.g.turn.priority = Some(P1);
    assert!(t.cast(P1, bolt).target(tromo).try_go().is_ok());
    t.advance_to(P0, Step::EndOfCombat);
    assert!(is_attacking(&t, tromo));
    t.advance_to(P0, Step::PostcombatMain);
    assert!(has(&mut t, tromo, KeywordKind::Hexproof));
}
