//! "For each [players or objects], [instruction]" (CR 608.2c, 608.2f): the instruction is
//! performed once for each of them, with "that player", "its controller", "it", "that
//! creature" meaning the one it's performed for.

use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::*;

fn compiles(name: &str) {
    let def = card(name);
    assert!(
        def.unsupported_text().is_empty(),
        "{name} has unsupported text: {:?}",
        def.unsupported_text()
    );
}

/// The permanents named `name` that `p` controls.
fn controlled(t: &TestGame, p: PlayerId, name: &str) -> usize {
    t.named_on_battlefield(name)
        .into_iter()
        .filter(|o| t.obj(*o).controller == p)
        .count()
}

fn permanents(t: &TestGame, p: PlayerId) -> usize {
    t.g.permanents_controlled_by(p).len()
}

#[test]
fn march_of_souls_each_controller_gets_a_spirit_for_each_of_their_creatures() {
    cr!("608.2c", "608.2f", "608.2h");
    compiles("March of Souls");
    // "Destroy all creatures. They can't be regenerated. For each creature destroyed this
    // way, its controller creates a 1/1 white Spirit creature token with flying."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 5);
    let spell = t.hand(P0, "March of Souls");
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.named_on_battlefield("Grizzly Bears").is_empty());
    assert_eq!(controlled(&t, P0, "Spirit Token"), 1);
    assert_eq!(controlled(&t, P1, "Spirit Token"), 2);
}

#[test]
fn curse_of_the_swine_boars_for_the_exiled_creatures_controllers() {
    cr!("608.2h", "400.7");
    compiles("Curse of the Swine");
    // "Exile X target creatures. For each creature exiled this way, its controller creates a
    // 2/2 green Boar creature token."
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Grizzly Bears");
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Island", 4);
    let spell = t.hand(P0, "Curse of the Swine");
    t.cast(P0, spell)
        .x(2)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve();
    assert!(t.on_battlefield(mine));
    assert_eq!(controlled(&t, P1, "Boar Token"), 2);
    assert_eq!(controlled(&t, P0, "Boar Token"), 0);
}

#[test]
fn fade_away_each_creature_costs_its_controller_a_payment_or_a_sacrifice() {
    cr!("608.2f", "118.12a", "101.4");
    ruling!(
        "Fade Away",
        "all the sacrifices are done at the same time"
    );
    compiles("Fade Away");
    // "For each creature, its controller sacrifices a permanent of their choice unless they
    // pay {1}."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Grizzly Bears");
    t.battlefield(P1, "Grizzly Bears");
    t.battlefield(P1, "Hill Giant");
    t.battlefield(P1, "Forest");
    t.lands(P0, "Island", 3);
    let spell = t.hand(P0, "Fade Away");
    // P1 pays for one creature (with the Forest) and not for the other.
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(false));
    t.cast(P0, spell).go();
    t.resolve();
    // P0 sacrificed one of its four permanents; P1 one of its three.
    assert_eq!(permanents(&t, P0), 3);
    assert_eq!(permanents(&t, P1), 2);
}

#[test]
fn great_unclean_one_demons_for_opponents_with_less_life() {
    cr!("608.2c");
    compiles("Great Unclean One");
    // "At the beginning of your end step, each opponent loses 2 life. Then for each
    // opponent who has less life than you, create a 1/3 black Demon creature token named
    // Plaguebearer of Nurgle."
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Great Unclean One");
    t.g.player_mut(P2).life = 40;
    t.set_step(P0, Step::PostcombatMain);
    t.advance_to(P0, Step::End);
    t.resolve_all();
    assert_eq!(t.life(P1), 18);
    assert_eq!(t.life(P2), 38);
    // Only P1 has less life than P0.
    assert_eq!(t.named_on_battlefield("Plaguebearer of Nurgle").len(), 1);
}

#[test]
fn the_eternal_wanderer_each_player_keeps_one_chosen_creature() {
    cr!("115.10", "608.2c");
    ruling!(
        "The Eternal Wanderer",
        "none of the chosen creatures are targets of the ability"
    );
    compiles("The Eternal Wanderer");
    // "−4: For each player, choose a creature that player controls. Each player sacrifices
    // all creatures they control not chosen this way."
    let mut t = TestGame::new(2);
    let w = t.battlefield(P0, "The Eternal Wanderer");
    let have = t.counters(w, "loyalty");
    t.g.add_counters(Entity::Object(w), "loyalty", 6 - have, None);
    let keep0 = t.battlefield(P0, "Hill Giant");
    let lose0 = t.battlefield(P0, "Grizzly Bears");
    let lose1 = t.battlefield(P1, "Hill Giant");
    let keep1 = t.battlefield(P1, "Grizzly Bears");
    t.answer_choose(P0, &[Entity::Object(keep0)]);
    t.answer_choose(P0, &[Entity::Object(keep1)]);
    t.activate(P0, w, 2, &[]).unwrap();
    t.resolve();
    assert!(t.on_battlefield(keep0));
    assert!(t.on_battlefield(keep1));
    assert!(!t.on_battlefield(lose0));
    assert!(!t.on_battlefield(lose1));
}

#[test]
fn twinflame_copies_each_target_and_exiles_those_tokens() {
    cr!("707.2", "608.2c");
    compiles("Twinflame");
    // "Choose any number of target creatures you control. For each of them, create a token
    // that's a copy of that creature, except it has haste. Exile those tokens at the
    // beginning of the next end step."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P0, "Hill Giant");
    // {1}{R}, plus {2}{R} for the second target (strive).
    t.lands(P0, "Mountain", 5);
    let spell = t.hand(P0, "Twinflame");
    t.set_step(P0, Step::PrecombatMain);
    t.cast(P0, spell)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 2);
    assert_eq!(t.named_on_battlefield("Hill Giant").len(), 2);
    let copy = t
        .named_on_battlefield("Hill Giant")
        .into_iter()
        .find(|o| *o != b)
        .unwrap();
    assert!(t
        .obj(copy)
        .chars
        .has_keyword(mtg_engine::keywords::KeywordKind::Haste));
    t.advance_to(P0, Step::End);
    t.resolve_all();
    // The tokens are exiled; the originals stay.
    assert_eq!(t.named_on_battlefield("Grizzly Bears"), vec![a]);
    assert_eq!(t.named_on_battlefield("Hill Giant"), vec![b]);
}

#[test]
fn sword_point_diplomacy_an_opponent_pays_life_for_each_card() {
    cr!("118.12a", "608.2c");
    ruling!(
        "Sword-Point Diplomacy",
        "each opponent in turn order chooses whether to pay life for one card"
    );
    compiles("Sword-Point Diplomacy");
    // "Reveal the top three cards of your library. For each of those cards, put that card
    // into your hand unless any opponent pays 3 life. Then exile the rest."
    let mut t = TestGame::new(2);
    for _ in 0..3 {
        t.library_top(P0, "Grizzly Bears");
    }
    t.lands(P0, "Swamp", 3);
    let spell = t.hand(P0, "Sword-Point Diplomacy");
    let hand = t.hand_size(P0);
    // The opponent pays for the first card only.
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(false));
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(false));
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    // Two cards to hand (the spell left it), one exiled.
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    assert!(t.in_exile("Grizzly Bears"));
}

#[test]
fn bloom_tender_adds_one_mana_of_each_color_among_your_permanents() {
    cr!("105.4", "106.1");
    ruling!(
        "Bloom Tender",
        "Bloom Tender won't produce more than one mana of any particular color"
    );
    ruling!("Bloom Tender", "Bloom Tender can't produce colorless mana");
    compiles("Bloom Tender");
    // "{T}: For each color among permanents you control, add one mana of that color."
    let mut t = TestGame::new(2);
    let tender = t.battlefield(P0, "Bloom Tender");
    t.battlefield(P0, "Llanowar Elves");
    t.battlefield(P0, "Isamaru, Hound of Konda");
    t.battlefield(P0, "Ornithopter");
    t.activate(P0, tender, 0, &[]).unwrap();
    t.resolve_all();
    let pool = &t.g.player(P0).mana_pool;
    use mtg_engine::mana::ManaType;
    assert_eq!(pool.count(ManaType::G), 1);
    assert_eq!(pool.count(ManaType::W), 1);
    assert_eq!(pool.count(ManaType::C), 0);
    assert_eq!(pool.count(ManaType::U), 0);
}

#[test]
fn sokenzan_smelter_pays_mana_and_sacrifices_an_artifact() {
    cr!("118.12", "118.3");
    compiles("Sokenzan Smelter");
    // "At the beginning of combat on your turn, you may pay {1} and sacrifice an artifact.
    // If you do, create a 3/1 red Construct artifact creature token with haste."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sokenzan Smelter");
    let thopter = t.battlefield(P0, "Ornithopter");
    t.battlefield(P0, "Mountain");
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(!t.on_battlefield(thopter));
    assert_eq!(t.named_on_battlefield("Construct Token").len(), 1);
    // Without an artifact to sacrifice, the cost can't be paid.
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Sokenzan Smelter");
    t.battlefield(P0, "Mountain");
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.set_step(P0, Step::PrecombatMain);
    t.advance_to(P0, Step::BeginningOfCombat);
    t.resolve_all();
    assert!(t.named_on_battlefield("Construct Token").is_empty());
}

#[test]
fn wolf_of_devils_breach_deals_the_discarded_cards_mana_value() {
    cr!("118.12");
    ruling!(
        "Wolf of Devil's Breach",
        "You can’t choose to discard a card without also paying {1}{R}."
    );
    compiles("Wolf of Devil's Breach");
    // "Whenever this creature attacks, you may pay {1}{R} and discard a card. If you do,
    // this creature deals damage to target creature or planeswalker equal to the
    // discarded card's mana value."
    let mut t = TestGame::new(2);
    let wolf = t.battlefield(P0, "Wolf of Devil's Breach");
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Mountain", 2);
    let discard = t.hand(P0, "Hill Giant");
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer_choose(P0, &[Entity::Object(discard)]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.attack(&[(wolf, Entity::Player(P1))], &[]);
    assert!(t.in_graveyard(P0, "Hill Giant"));
    // Hill Giant (mana value 4) took 4 damage and died.
    assert!(!t.on_battlefield(giant));
}

#[test]
fn maulfist_revolutionary_gives_one_counter_of_each_kind() {
    cr!("122.1", "608.2h");
    ruling!(
        "Maulfist Revolutionary",
        "gives only one counter of each kind. It doesn’t double the number"
    );
    compiles("Maulfist Revolutionary");
    // "When this creature enters or dies, for each kind of counter on target permanent or
    // player, give that permanent or player another counter of that kind."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(Entity::Object(giant), "+1/+1", 2, None);
    t.g.add_counters(Entity::Object(giant), "charge", 1, None);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    t.enter(P0, "Maulfist Revolutionary");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.counters(giant, "+1/+1"), 3);
    assert_eq!(t.counters(giant, "charge"), 2);
    // A player: one more poison counter.
    let mut t = TestGame::new(2);
    t.g.add_counters(Entity::Player(P1), "poison", 2, None);
    t.answer_targets(P0, &[Entity::Player(P1)]);
    t.enter(P0, "Maulfist Revolutionary");
    t.g.flush_events();
    t.resolve_all();
    assert_eq!(t.g.player(P1).counters.get("poison").copied().unwrap_or(0), 3);
}

#[test]
fn quarry_hauler_chooses_separately_for_each_kind() {
    cr!("608.2h");
    ruling!(
        "Quarry Hauler",
        "You don’t have to make the same choice for each kind of counter."
    );
    compiles("Quarry Hauler");
    // "When this creature enters, for each kind of counter on target permanent, put another
    // counter of that kind on it or remove one from it."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.g.add_counters(Entity::Object(giant), "+1/+1", 2, None);
    t.g.add_counters(Entity::Object(giant), "charge", 2, None);
    t.answer_targets(P0, &[Entity::Object(giant)]);
    // Add for the first kind, remove for the second.
    t.answer(P0, DecisionKind::Option, Answer::Index(0));
    t.answer(P0, DecisionKind::Option, Answer::Index(1));
    t.enter(P0, "Quarry Hauler");
    t.g.flush_events();
    t.resolve_all();
    let (a, b) = (t.counters(giant, "+1/+1"), t.counters(giant, "charge"));
    assert_eq!(a + b, 4);
    assert!((a, b) == (3, 1) || (a, b) == (1, 3), "{a} {b}");
}

#[test]
fn refurbished_familiar_draws_for_each_opponent_who_cant_discard() {
    cr!("608.2c", "101.4");
    compiles("Refurbished Familiar");
    // "When this creature enters, each opponent discards a card. For each opponent who
    // can't, you draw a card."
    let mut t = TestGame::new(3);
    t.hand(P1, "Grizzly Bears");
    let hand = t.hand_size(P0);
    t.enter(P0, "Refurbished Familiar");
    t.g.flush_events();
    t.resolve_all();
    // P1 discarded; P2 had no card to discard.
    assert!(t.in_graveyard(P1, "Grizzly Bears"));
    assert_eq!(t.hand_size(P0), hand + 1);
}

#[test]
fn brace_for_impact_counters_for_each_damage_prevented() {
    cr!("615.5");
    compiles("Brace for Impact");
    // "Prevent all damage that would be dealt to target multicolored creature this turn.
    // For each 1 damage prevented this way, put a +1/+1 counter on that creature."
    let mut t = TestGame::new(2);
    let c = t.battlefield(P0, "Boros Recruit");
    t.lands(P0, "Plains", 5);
    let spell = t.hand(P0, "Brace for Impact");
    t.cast(P0, spell).target(c).go();
    t.resolve();
    t.lands(P1, "Mountain", 1);
    let bolt = t.hand(P1, "Lightning Bolt");
    t.cast(P1, bolt).target(c).go();
    t.resolve();
    assert!(t.on_battlefield(c));
    assert_eq!(t.counters(c, "+1/+1"), 3);
}

#[test]
fn inkshield_tokens_for_each_combat_damage_prevented() {
    cr!("615.5");
    compiles("Inkshield");
    // "Prevent all combat damage that would be dealt to you this turn. For each 1 damage
    // prevented this way, create a 2/1 white and black Inkling creature token with flying."
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Plains", 4);
    t.lands(P0, "Swamp", 1);
    t.set_step(P1, Step::BeginningOfCombat);
    let spell = t.hand(P0, "Inkshield");
    t.cast(P0, spell).go();
    t.resolve();
    t.attack(&[(giant, Entity::Player(P0))], &[]);
    assert_eq!(t.life(P0), 20);
    assert_eq!(controlled(&t, P0, "Inkling Token"), 3);
}

#[test]
fn bladecoil_serpent_draws_for_each_uu_spent() {
    cr!("601.2h");
    compiles("Bladecoil Serpent");
    // "When this creature enters, for each {U}{U} spent to cast it, draw a card."
    let mut t = TestGame::new(2);
    t.lands(P0, "Island", 5);
    t.lands(P0, "Wastes", 3);
    let serpent = t.hand(P0, "Bladecoil Serpent");
    let hand = t.hand_size(P0);
    t.cast(P0, serpent).x(2).go();
    t.resolve_all();
    // Five blue mana spent: two {U}{U}.
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
}

#[test]
fn all_suns_dawn_one_target_of_each_color() {
    cr!("105.1", "115.1");
    ruling!(
        "All Suns' Dawn",
        "You can target two blue-and-white multicolored cards by choosing one as the blue card and one as the white card."
    );
    compiles("All Suns' Dawn");
    // "For each color, return up to one target card of that color from your graveyard to
    // your hand. Exile All Suns' Dawn."
    let mut t = TestGame::new(2);
    let azorius = t.graveyard(P0, "Azorius Guildmage");
    let azorius2 = t.graveyard(P0, "Azorius Guildmage");
    let bears = t.graveyard(P0, "Grizzly Bears");
    t.lands(P0, "Forest", 5);
    let spell = t.hand(P0, "All Suns' Dawn");
    // White: a Guildmage; blue: the other one; green: the Bears (no black or red card to
    // choose).
    t.answer_targets(P0, &[Entity::Object(azorius)]);
    t.answer_targets(P0, &[Entity::Object(azorius2)]);
    t.answer_targets(P0, &[Entity::Object(bears)]);
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.in_hand(P0, "Grizzly Bears"));
    assert_eq!(
        t.g.player(P0)
            .hand
            .iter()
            .filter(|o| t.obj(**o).chars.name == "Azorius Guildmage")
            .count(),
        2
    );
    assert!(t.in_exile("All Suns' Dawn"));
}

#[test]
fn revival_experiment_returns_one_card_of_each_permanent_type() {
    cr!("110.4", "608.2c");
    ruling!(
        "Revival Experiment",
        "The cards you return are chosen as Revival Experiment resolves."
    );
    compiles("Revival Experiment");
    // "For each permanent type, return up to one card of that type from your graveyard to
    // the battlefield. You lose 3 life for each card returned this way. Exile Revival
    // Experiment."
    let mut t = TestGame::new(2);
    let bears = t.graveyard(P0, "Grizzly Bears");
    let forest = t.graveyard(P0, "Forest");
    t.lands(P0, "Swamp", 3);
    t.lands(P0, "Forest", 3);
    let spell = t.hand(P0, "Revival Experiment");
    t.answer_choose(P0, &[Entity::Object(bears)]);
    t.answer_choose(P0, &[Entity::Object(forest)]);
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.named_on_battlefield("Grizzly Bears").len(), 1);
    assert!(!t.in_graveyard(P0, "Grizzly Bears"));
    // The Bears and the Forest returned: 6 life.
    assert_eq!(t.life(P0), 14);
}

#[test]
fn moonlight_bargain_each_looked_at_card_costs_life_or_goes_to_the_graveyard() {
    cr!("118.12a", "608.2c");
    compiles("Moonlight Bargain");
    // "Look at the top five cards of your library. For each card, put that card into your
    // graveyard unless you pay 2 life. Then put the rest into your hand."
    let mut t = TestGame::new(2);
    for _ in 0..5 {
        t.library_top(P0, "Grizzly Bears");
    }
    t.lands(P0, "Swamp", 5);
    let spell = t.hand(P0, "Moonlight Bargain");
    let hand = t.hand_size(P0);
    // Pay for two cards; the other three go to the graveyard.
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(false));
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(false));
    t.answer(P0, DecisionKind::YesNo, Answer::Bool(false));
    t.cast(P0, spell).go();
    t.resolve();
    assert_eq!(t.life(P0), 16);
    assert_eq!(t.hand_size(P0), hand - 1 + 2);
    assert_eq!(
        t.g.player(P0)
            .graveyard
            .iter()
            .filter(|o| t.obj(**o).chars.name == "Grizzly Bears")
            .count(),
        3
    );
}

#[test]
fn killing_wave_pay_x_life_for_some_creatures_and_sacrifice_the_rest() {
    cr!("118.12a", "608.2f", "107.3a");
    ruling!(
        "Killing Wave",
        "A player may choose to pay life for some creatures and sacrifice the rest."
    );
    compiles("Killing Wave");
    // "For each creature, its controller sacrifices it unless they pay X life."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P1, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Swamp", 4);
    let spell = t.hand(P0, "Killing Wave");
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(false));
    t.cast(P0, spell).x(3).go();
    t.resolve();
    assert_eq!(t.life(P1), 17);
    assert_eq!([t.on_battlefield(a), t.on_battlefield(b)].iter().filter(|x| **x).count(), 1);
}

#[test]
fn doppelgang_x_copies_of_each_target() {
    cr!("707.2");
    compiles("Doppelgang");
    // "For each of X target permanents, create X tokens that are copies of that
    // permanent."
    let mut t = TestGame::new(2);
    let a = t.battlefield(P0, "Grizzly Bears");
    let b = t.battlefield(P1, "Hill Giant");
    t.lands(P0, "Forest", 4);
    t.lands(P0, "Island", 4);
    let spell = t.hand(P0, "Doppelgang");
    t.cast(P0, spell)
        .x(2)
        .targets(&[Entity::Object(a), Entity::Object(b)])
        .go();
    t.resolve();
    assert_eq!(controlled(&t, P0, "Grizzly Bears"), 3);
    assert_eq!(controlled(&t, P0, "Hill Giant"), 2);
}

#[test]
fn from_the_ashes_each_destroyed_lands_controller_may_search() {
    cr!("608.2c", "701.23");
    compiles("From the Ashes");
    // "Destroy all nonbasic lands. For each land destroyed this way, its controller may
    // search their library for a basic land card and put it onto the battlefield. Then
    // each player who searched their library this way shuffles."
    let mut t = TestGame::new(2);
    t.battlefield(P1, "Ancient Tomb");
    t.battlefield(P1, "Ancient Tomb");
    t.library_top(P1, "Forest");
    t.library_top(P1, "Island");
    t.lands(P0, "Mountain", 4);
    let spell = t.hand(P0, "From the Ashes");
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.cast(P0, spell).go();
    t.resolve();
    assert!(t.named_on_battlefield("Ancient Tomb").is_empty());
    assert_eq!(controlled(&t, P1, "Forest") + controlled(&t, P1, "Island"), 2);
}

#[test]
fn krenkos_buzzcrusher_destroys_one_nonbasic_land_of_each_player() {
    cr!("115.10");
    ruling!(
        "Krenko's Buzzcrusher",
        "None of the nonbasic lands are targets"
    );
    compiles("Krenko's Buzzcrusher");
    // "When this creature enters, for each player, destroy up to one nonbasic land that
    // player controls. For each land destroyed this way, its controller may search their
    // library for a basic land card, put it onto the battlefield tapped, then shuffle."
    let mut t = TestGame::new(2);
    let mine = t.battlefield(P0, "Ancient Tomb");
    let theirs = t.battlefield(P1, "Ancient Tomb");
    t.library_top(P1, "Forest");
    t.answer_choose(P0, &[]);
    t.answer_choose(P0, &[Entity::Object(theirs)]);
    t.answer(P1, DecisionKind::YesNo, Answer::Bool(true));
    t.enter(P0, "Krenko's Buzzcrusher");
    t.g.flush_events();
    t.resolve_all();
    assert!(t.on_battlefield(mine));
    assert!(!t.on_battlefield(theirs));
    assert_eq!(controlled(&t, P1, "Forest"), 1);
}

#[test]
fn mega_flare_damages_the_creature_chosen_for_each_opponent() {
    cr!("601.2c", "115.1");
    ruling!(
        "Mega Flare",
        "Use the greatest power among creatures you control as Mega Flare resolves"
    );
    compiles("Mega Flare");
    // "For each opponent, choose up to one target creature that player controls. Mega Flare
    // deals damage equal to the greatest power among creatures you control to each of the
    // chosen creatures."
    let mut t = TestGame::new(3);
    t.battlefield(P0, "Hill Giant");
    let a = t.battlefield(P1, "Hill Giant");
    let b = t.battlefield(P2, "Grizzly Bears");
    t.lands(P0, "Mountain", 3);
    let spell = t.hand(P0, "Mega Flare");
    t.answer_targets(P0, &[Entity::Object(a)]);
    t.answer_targets(P0, &[Entity::Object(b)]);
    t.cast(P0, spell).go();
    t.resolve();
    assert!(!t.on_battlefield(a));
    assert!(!t.on_battlefield(b));
}
