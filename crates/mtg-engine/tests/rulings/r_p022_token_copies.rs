//! Rulings batch P022 — effects that create token copies of cards and permanents: the
//! copied "enters" abilities trigger and the "as enters" and "enters with" abilities apply
//! (CR 707.5, 603.6a, 614.1c), {X} is 0 (CR 107.3, 202.3e), the legend rule (CR 704.5j,
//! 603.3), tokens that leave cease to exist (CR 111.7, 111.8), copies of copies and tokens
//! (CR 707.3, 111.3), delayed triggers (CR 603.7) and copied permanent spells (CR 707.10,
//! 608.3f).

use crate::r_p022_common::*;
use mtg_engine::decision::Answer;
use mtg_engine::testing::*;
use mtg_engine::turn::Step;
use mtg_engine::types::*;
use mtg_engine::*;

/// Runs `make` (which gives P0 token copies of the card `card`, set up by `place`) twice:
/// with Elvish Visionary ("When this creature enters, draw a card": the token's copy
/// triggers) and Faithful Watchdog ("enters with three +1/+1 counters": the token gets
/// them). `make` returns the new tokens.
fn enters_abilities_work(
    place: impl Fn(&mut TestGame, &str) -> ObjectId,
    make: impl Fn(&mut TestGame, ObjectId) -> Vec<ObjectId>,
) {
    supported("Faithful Watchdog");
    let mut t = TestGame::new(2);
    let card = place(&mut t, "Elvish Visionary");
    let hand = t.hand_size(P0);
    let toks = make(&mut t, card);
    let tok = one(toks);
    assert_eq!(name_of(&t, tok), "Elvish Visionary");
    assert!(t.hand_size(P0) > hand, "the token's enters trigger drew: {}", t.dump_log());
    let mut t = TestGame::new(2);
    let card = place(&mut t, "Faithful Watchdog");
    let tok = one(make(&mut t, card));
    assert_eq!(name_of(&t, tok), "Faithful Watchdog");
    assert_eq!(t.counters(tok, counters::PLUS1), 3);
}

#[test]
fn token_copies_of_cards_have_working_enters_abilities() {
    cr!("707.5", "603.6a", "614.1c");
    ruling!("Grist, Voracious Larva // Grist, the Plague Swarm", "If a card copied by one of the tokens had any \"when [this creature] enters the battlefield\" abilities, the appropriate token also has those abilities and will trigger them when it's created.");
    ruling!("Altaïr Ibn-La'Ahad", "If a card copied by one of the tokens had any \"when [this permanent] enters the battlefield\" abilities, then the token also has those abilities and will trigger them when it's created.");
    ruling!("Ardyn, the Usurper", "If a card copied by the token had any \"when [this permanent] enters\" abilities, the token also has those abilities, and they'll trigger when it's created.");
    ruling!("Abyssal Harvester", "If a card copied by the token had any \"when this creature enters\" abilities, the appropriate token also has those abilities and will trigger them when it's created.");
    ruling!("Arcane Artisan", "If the card copied by the token had any “when [this permanent] enters the battlefield” abilities, then the token also has those abilities and will trigger them when it’s created.");
    ruling!("Cursecloth Wrappings", "If the card copied by the token had any “when [this permanent] enters” abilities, then the token also has those abilities and they’ll trigger when it enters.");
    supported("Ardyn, the Usurper");
    supported("Abyssal Harvester");
    supported("Arcane Artisan");
    supported("Cursecloth Wrappings");
    // Grist, the Plague Swarm's −6: "For each creature card in your graveyard, create a
    // token that's a copy of it, except it's a 1/1 black and green Insect."
    enters_abilities_work(
        |t, n| t.graveyard(P0, n),
        |t, _| {
            let grist = grist_planeswalker(t);
            let before = t.g.battlefield.clone();
            activate_containing(t, P0, grist, "For each creature card").expect("−6");
            t.resolve_all();
            new_tokens(t, P0, &before)
        },
    );
    // Altaïr: "Whenever Altaïr attacks, exile up to one target Assassin creature card from
    // your graveyard with a memory counter on it. Then for each creature card you own in
    // exile with a memory counter on it, create a tapped and attacking token that's a copy
    // of it."
    enters_abilities_work(
        |t, n| {
            let c = t.exile(P0, n);
            t.g.add_counters(Entity::Object(c), "memory", 1, None);
            c
        },
        altair_attacks,
    );
    // Ardyn's Starscourge: at the beginning of combat on P0's turn, exile up to one target
    // creature card from a graveyard; a token copy that's a 5/5 black Demon.
    enters_abilities_work(
        |t, n| t.graveyard(P1, n),
        |t, card| {
            t.battlefield(P0, "Ardyn, the Usurper");
            t.answer_targets(P0, &[obj(card)]);
            let before = t.g.battlefield.clone();
            t.advance_to(P0, Step::BeginningOfCombat);
            t.resolve_all();
            new_tokens(t, P0, &before)
        },
    );
    // Abyssal Harvester: "{T}: Exile target creature card from a graveyard that was put
    // there this turn. Create a token that's a copy of it, except it's a Nightmare".
    enters_abilities_work(
        |t, n| {
            let c = t.battlefield(P1, n);
            kill(t, c);
            t.g.current(c)
        },
        |t, card| {
            let h = t.battlefield(P0, "Abyssal Harvester");
            activate_for_tokens(t, P0, h, "Create a token", &[obj(card)])
        },
    );
    // Arcane Artisan: "{2}{U}, {T}: Target player draws a card, then exiles a card from
    // their hand. If a creature card is exiled this way, that player creates a token
    // that's a copy of that card."
    enters_abilities_work(
        |t, n| t.hand(P0, n),
        |t, card| {
            let a = t.battlefield(P0, "Arcane Artisan");
            lands_for(t, P0, "{2}{U}");
            t.answer_choose(P0, &[obj(card)]);
            activate_for_tokens(t, P0, a, "draws a card", &[Entity::Player(P0)])
        },
    );
    // Cursecloth Wrappings: "{T}: Target creature card in your graveyard gains embalm
    // until end of turn. The embalm cost is equal to its mana cost."
    enters_abilities_work(
        |t, n| t.graveyard(P0, n),
        |t, card| {
            let w = t.battlefield(P0, "Cursecloth Wrappings");
            t.answer_targets(P0, &[obj(card)]);
            activate_containing(t, P0, w, "gains embalm").expect("Wrappings");
            t.resolve_all();
            t.lands(P0, "Forest", 2);
            t.lands(P0, "Plains", 2);
            let before = t.g.battlefield.clone();
            let card = t.g.current(card);
            activate_containing(t, P0, card, "Embalm").expect("embalm");
            t.resolve_all();
            new_tokens(t, P0, &before)
        },
    );
}

/// P0's Grist, the Plague Swarm with seven loyalty (it stays after its −6).
fn grist_planeswalker(t: &mut TestGame) -> ObjectId {
    let g = t.battlefield(P0, "Grist, Voracious Larva // Grist, the Plague Swarm");
    mtg_engine::dfc::transform(&mut t.g, g);
    t.g.recompute();
    let g = t.g.current(g);
    t.g.add_counters(Entity::Object(g), counters::LOYALTY, 7, None);
    t.g.recompute();
    g
}

/// P0's Altaïr attacks P1 (targeting nothing); returns the tokens P0 got.
fn altair_attacks(t: &mut TestGame, _card: ObjectId) -> Vec<ObjectId> {
    let a = t.battlefield(P0, "Altaïr Ibn-La'Ahad");
    t.answer_targets(P0, &[]);
    t.set_step(P0, Step::BeginningOfCombat);
    t.answer(
        P0,
        DecisionKind::Attackers,
        Answer::Attackers(vec![(a, Entity::Player(P1))]),
    );
    let before = t.g.battlefield.clone();
    t.advance_to(P0, Step::DeclareAttackers);
    t.resolve_all();
    new_tokens(t, P0, &before)
}

#[test]
fn token_copies_of_exiled_or_graveyard_cards_have_x_0() {
    cr!("107.3", "202.3e", "707.2");
    ruling!("Altaïr Ibn-La'Ahad", "If an exiled card has {X} in its mana cost, X is 0.");
    ruling!("Grist, Voracious Larva // Grist, the Plague Swarm", "If any of the copied cards have {X} in their mana costs, {X} is 0.");
    // Altaïr: a Benevolent Hydra card in exile with a memory counter.
    let mut t = TestGame::new(2);
    let h = t.exile(P0, HYDRA);
    t.g.add_counters(Entity::Object(h), "memory", 1, None);
    let tok = one(altair_attacks(&mut t, h));
    hydra_with_x0(&mut t, tok);
    // Grist: a Benevolent Hydra card in the graveyard (the token is a 1/1).
    let mut t = TestGame::new(2);
    t.graveyard(P0, HYDRA);
    let grist = grist_planeswalker(&mut t);
    let before = t.g.battlefield.clone();
    activate_containing(&mut t, P0, grist, "For each creature card").expect("−6");
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    assert_eq!(mv_now(&mut t, tok), 2);
    assert_eq!(t.counters(tok, counters::PLUS1), 0);
}

#[test]
fn paleontologists_pick_axe_copy_has_x_0() {
    cr!("107.3", "202.3e", "707.2");
    ruling!("Paleontologist's Pick-Axe // Dinosaur Headdress", "If the chosen card has {X} in its mana cost, X is 0.");
    supported("Paleontologist's Pick-Axe // Dinosaur Headdress");
    let mut t = TestGame::new(2);
    let axe = t.battlefield(P0, "Paleontologist's Pick-Axe // Dinosaur Headdress");
    let hydra = t.graveyard(P0, HYDRA);
    let bears = t.battlefield(P0, "Grizzly Bears");
    t.lands(P0, "Wastes", 5);
    let craft = activated_index(&t, axe, "Craft");
    t.answer_choose(P0, &[obj(hydra)]);
    t.answer_targets(P0, &[obj(bears)]);
    t.activate(P0, axe, craft, &[]).expect("craft");
    t.resolve_all();
    assert_eq!(name_of(&t, bears), HYDRA);
    assert_eq!(mv_now(&mut t, bears), 2);
}

// --- The legend rule ------------------------------------------------------------------------

#[test]
fn inalla_token_copy_of_a_legend_triggers_before_the_legend_rule() {
    cr!("704.5j", "603.3", "603.6a", "603.10a");
    ruling!("Inalla, Archmage Ritualist", "If a copied creature is legendary, any triggered abilities it has will trigger if appropriate before the \"legend rule\" causes one of those legendary creatures to be put into its owner's graveyard. Those abilities will be put on the stack after the \"legend rule\" is applied.");
    supported("Inalla, Archmage Ritualist");
    supported("Cynette, Jelly Drover");
    // Cynette, Jelly Drover (a legendary Wizard): "When Cynette enters or dies, create a
    // 1/1 blue Jellyfish creature token with flying."
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Inalla, Archmage Ritualist");
    t.lands(P0, "Wastes", 1);
    t.answer_yes(P0, true);
    t.enter(P0, "Cynette, Jelly Drover");
    t.resolve_all();
    // The legend rule put one of the two into the graveyard.
    assert_eq!(t.named_on_battlefield("Cynette, Jelly Drover").len(), 1);
    // The original's enters trigger, the token's enters trigger (it triggered although
    // the token may already be gone) and the dies trigger of the one that was put into
    // the graveyard.
    assert_eq!(crate::r_s01_common::with_subtype(&t, P0, "Jellyfish").len(), 3, "{}", t.dump_log());
}

#[test]
fn dual_nature_token_of_a_legend_meets_the_legend_rule_at_once() {
    cr!("704.5j", "704.3");
    ruling!("Dual Nature", "If a legendary creature enters the battlefield, the token is put onto the battlefield and then the \"legend rule\" forces its controller to put either the original or the token into its owner's graveyard before any player can take any actions.");
    supported("Dual Nature");
    supported("Isamaru, Hound of Konda");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Dual Nature");
    let isamaru = t.enter(P0, "Isamaru, Hound of Konda");
    t.answer_choose(P0, &[obj(isamaru)]);
    t.settle();
    t.resolve();
    // The token was created and is gone before anyone gets priority again.
    assert_eq!(t.named_on_battlefield("Isamaru, Hound of Konda"), vec![isamaru]);
    assert_eq!(t.stack_len(), 0);
}

#[test]
fn sarkhan_copying_a_legendary_dragon_keeps_its_own_name() {
    cr!("704.5j", "707.9b");
    ruling!("Sarkhan, Soul Aflame", "If the Dragon that enters the battlefield is legendary and you have Sarkhan copy it, the two Dragons will have different names (unless the entering Dragon is somehow named Sarkhan, Soul Aflame). The \"legend rule\" won't apply.");
    supported("Sarkhan, Soul Aflame");
    supported("Rorix Bladewing");
    let mut t = TestGame::new(2);
    let sarkhan = t.battlefield(P0, "Sarkhan, Soul Aflame");
    t.answer_yes(P0, true);
    let rorix = t.enter(P0, "Rorix Bladewing");
    t.resolve_all();
    assert_eq!(name_of(&t, sarkhan), "Sarkhan, Soul Aflame");
    assert_eq!(t.pt(sarkhan), (6, 5));
    assert!(legendary(&t, sarkhan));
    assert!(t.on_battlefield(sarkhan) && t.on_battlefield(rorix));
}

// --- Copying copies and tokens ---------------------------------------------------------------

#[test]
fn miirym_copies_what_the_entering_dragon_copies() {
    cr!("707.3", "707.9b");
    ruling!("Miirym, Sentinel Wyrm", "If the Dragon that entered the battlefield is copying something else as the triggered ability resolves, the token will be a copy of whatever that creature is a copy of.");
    supported("Miirym, Sentinel Wyrm");
    supported("Shivan Dragon");
    let mut t = TestGame::new(2);
    t.battlefield(P0, "Miirym, Sentinel Wyrm");
    let dragon = t.battlefield(P1, "Shivan Dragon");
    let before = t.g.battlefield.clone();
    // P0's Clone enters as a copy of Shivan Dragon: a nontoken Dragon.
    clone_of(&mut t, P0, dragon);
    t.resolve_all();
    let tok = one(new_tokens(&t, P0, &before));
    assert_eq!(name_of(&t, tok), "Shivan Dragon");
    assert_eq!(t.pt(tok), (5, 5));
}

#[test]
fn march_of_progress_copies_what_a_copy_copies_and_a_tokens_original_values() {
    cr!("707.3", "707.2", "111.3");
    ruling!("March of Progress", "If the chosen artifact creature is copying something else, then the token enters the battlefield as whatever the chosen creature is copying.");
    ruling!("March of Progress", "If the chosen artifact creature is itself a token, the token March of Progress creates copies the original characteristics of that token as stated by the effect that created that token.");
    supported("March of Progress");
    supported("Phyrexian Metamorph");
    // Phyrexian Metamorph enters as a copy of Hill Giant (an artifact in addition).
    let mut t = TestGame::new(2);
    let giant = t.battlefield(P1, "Hill Giant");
    let m = enter_copying(&mut t, P0, "Phyrexian Metamorph", giant);
    assert_eq!(name_of(&t, m), "Hill Giant");
    let tok = one(cast_spell(&mut t, P0, "March of Progress", "{2}{U}", &[obj(m)], None, None));
    assert_eq!(name_of(&t, tok), "Hill Giant");
    assert!(is_artifact(&t, tok));
    assert_eq!(t.pt(tok), (3, 3));
    // A 1/1 blue Thopter artifact creature token, dressed up (tapped, counters, +3/+3,
    // green-blue): the copy is a plain untapped 1/1 blue Thopter.
    let mut t = TestGame::new(2);
    let mut spec = creature_spec("Thopter", Color::Blue, 1);
    spec.card_types = vec![CardType::Artifact, CardType::Creature];
    let thopter = token_of(&mut t, P0, spec);
    dress_up(&mut t, thopter);
    assert_eq!(t.pt(thopter), (6, 6));
    let tok = one(cast_spell(&mut t, P0, "March of Progress", "{2}{U}", &[obj(thopter)], None, None));
    assert_eq!(name_of(&t, tok), "Thopter");
    assert_eq!(t.pt(tok), (1, 1));
    assert_eq!(t.obj_now(tok).chars.colors, ColorSet::single(Color::Blue));
    assert!(fresh(&t, tok));
}
