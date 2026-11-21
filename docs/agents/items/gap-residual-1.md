# gap-residual-1: Residual engine gaps: several players' control changes and seats, granter references in granted abilities, terms on gained abilities, loop totals, piles in exile, spells cast "this way"

GOAL: Seven engine gaps that earlier work items reported as remaining, re-checked against the code at 06aa0f8a (code reading plus compiler probes and a throwaway engine test, see WORKING NOTES). They block about 65 paper cards, about 15 more need only compile work once these pieces exist, and three cards that compile today never work (Leonin Bola, Heartseeker, Razor Boomerang). One reported gap is already fixed and is dropped: ordered "starting with you" choices (see DROPPED). Two reported gaps turned out smaller than reported: the "repeat this process until" loop and face-down piles are mostly done, and their entries list only what is left. Two engine gaps found during the check are added: seat-relative players and directions (R2), and spells cast "this way" (R7, which includes Havengul Lich).

GAPS:

[R1] Control changes for several players at once (reported: "simultaneous token creation and control changes involving several controllers")
- Rules: CR 101.4, 101.4b, 608.2e, 608.2f (Blatant Thievery example: control of all the chosen permanents
  changes at the same time), 701.12a (if the whole exchange can't happen, none of it does), 701.12b.
- Engine now: simultaneous.rs plans only moves, exiles, sacrifices, discards, damage and searches
  (`plannable`, simultaneous.rs:551-579). `Effect::GainControl` and `ExchangeControl` fall through to
  `Plan::Exec`, so they run for one player at a time. `GainControl` recomputes at once (resolve.rs:835-879). A
  later player's "permanents controlled by [another player]" therefore already includes what earlier players
  took. Probe on the engine: a ForEachPlayer + GainControl "each player gains control of all creatures their
  opponents control" in a 2-player game left P1 controlling both creatures; a swap was expected. The module
  comment (simultaneous.rs:27-30) also makes an instruction whose choices are made by a player other than the
  iterated one (Inniaz: "of your choice") run choice-then-action for each player in turn. For control changes,
  the rulings require all the choices before any change. The compiler refuses control changes with "each ..."
  or "X and Y" subjects (control_change_grammar.rs:80), except two forms: regaining your own permanents
  (`p_each_player_regains`, :109, where the sets don't depend on control) and the two-player swap
  (`p_swap_all`, :376, which stores both sets first; Twist Allegiance compiles this way). `ExchangeControl`
  takes exactly two objects that are both on the battlefield (resolve.rs:881-921). So there is no exchange of
  two sets (Cultural Exchange) and none between a spell and a permanent (Perplexing Chimera, Sudden
  Substitution). Gaining control of a spell already works (Aethersnatch).
- Tokens (the other half of the report) are mostly fine. `CreateToken` for EachPlayer/EachOpponent creates
  them all in one instruction, and a ForEachPlayer body runs atomically as one event batch (simultaneous.rs
  `run`). One pitfall remains. A result an instruction stores by replacing a variable (`vars::CREATED`, `IT`)
  is kept per player, so a later instruction that names all of them sees only the last player's. Probe:
  ForEachPlayer{AsPlayer{CreateToken}} followed by Tap{Var(CREATED)} in a 3-player game tapped only P2's
  token. Before compiling "each player creates ... for each ... they ...; the tokens ...", collect such
  variables across frames, or use the single-instruction form.
- Cards (UNS): Aminatou, the Fateshifter (−6, also needs R2), Order of Succession (also R2), Inniaz, the Gale
  Force (also R2), Scrambleverse ("For each nonland permanent, choose a player at random. Then each player
  gains control of each permanent for which they were chosen."), Cultural Exchange, Perplexing Chimera, Sudden
  Substitution.
- Rulings (all UNCOVERED): Aminatou "each player takes control of their new permanents at the same time", "no matter which direction you choose" (two players: everything but Aminatou is exchanged); Order of Succession "All players will gain control of the creatures they chose simultaneously", "If the next player in the chosen direction controls no creatures"; Scrambleverse "All of the control-change effects happen at the same time", "including the permanent" (current controller may be chosen); Inniaz "you exchange control of two permanents of your choice with your opponent", "No player may take actions between the time you make the choices and the time control changes"; Perplexing Chimera "Neither Perplexing Chimera nor the spell changes zones"; Sudden Substitution "its new controller will control the permanent it becomes", "are both controlled by the same player, the exchange won".
- Approach: make `GainControl` plannable in simultaneous.rs. Each frame's objects are resolved in APNAP order
  with its choices, whoever makes them (each chooser sees the earlier choices, CR 101.4b, but no control has
  changed yet). Then perform_plans creates every control effect and recomputes once. Generalize
  `ExchangeControl` to two selections (all-or-nothing; "the same number" for Cultural Exchange) and allow a
  spell on the stack as either side, applying the same layer-2 effect that GainControl uses for spells.
  Scrambleverse = ForEach over the permanents (one random player each, choice_grammar_random.rs) + planned
  GainControl.
- Tests: tests/cr/r608_each_player_control.rs, cr!("608.2f", "101.4"): the probe AST above (2 players: swap; 3 players: a rotation once R2 exists); tests/cards/ for each card with its rulings; Perplexing Chimera/Sudden Substitution cr!("701.12a"); Cultural Exchange cr!("701.12a", "701.12b").
- Size: M

[R2] Seats: "the player to your left/right", "choose left or right", attacking only the nearest opponent in a direction (new; overlaps R1)
- Rules: CR 103.1 (turn order proceeds clockwise from the starting player), 101.4 (the next player in turn
  order is usually seated to the left), 102.3 (teammates), 508.1c (attack restrictions), 508.4 (creatures put
  onto the battlefield attacking aren't declared), 803.1a-b (the same seat notions for the game option).
- Engine now: `multiplayer::attack::neighbors` (multiplayer/attack.rs:11-22) returns a player's left and right
  neighbours, but only attack legality (the CR 803 option `GameConfig::attack_side`, Emperor, Alternating
  Teams) and Alternating Teams hand review (multiplayer/setup.rs:521) use it.
  No `PlayerRef` names a seat neighbour or a chosen direction (ability.rs:1061-1103). "Choose left or right."
  doesn't compile (probe), and `InTurnOrder` can only proceed in turn order (`TurnOrderStart::{You,
  NextOpponent}`, ability.rs:1049-1054). No restriction limits attacks to "the nearest opponent in the chosen
  direction".
- Cards (UNS): Grenzo's Rebuttal ("... controlled by the player to their left"), Hithlain Rope (2 abilities)
  and Bucknard's Everfull Purse ("The player to your right gains control of ~": texts owned by
  cards-game-rules, which needs this hook), Barroom Brawl ("the opponent to your left"), Pramikon, Sky Rampart
  (2 lines), Mystic Barrier (2 lines), Teyo, Geometric Tactician (−2), plus Aminatou, Order of Succession,
  Inniaz (R1). Draft-time seats (Paliano, the High City; Regicide) stay with cards-game-rules.
- Rulings (UNCOVERED): Hithlain Rope "the other player is the player to your right"; Barroom Brawl "the other player is the opponent to your left"; Inniaz "The leftmost player on each team gains control of a teammate" (Two-Headed Giant: seats ignore teams); Order of Succession "whether the next player is a teammate or an opponent"; Grenzo's Rebuttal "all chosen permanents are destroyed simultaneously"; Mystic Barrier "the opponent seated nearest you in both directions" (two Barriers with different directions both apply), "doesn’t affect creatures that enter the battlefield attacking"; Pramikon "disagree on which direction three or more players must attack" (then no one can attack); Teyo "disagree on which direction three or more players may attack".
- Approach: add `PlayerRef::Seat { of: Box<PlayerRef>, side: SeatSide }`, where `SeatSide::{Left, Right,
  Chosen}` and "nearest opponent in a direction" skips teammates (Mystic Barrier ruling). Store "choose left
  or right" as a linked word choice (`ChoiceKind::Word`, read back as `Choices::text`) for permanents ("the
  last chosen direction": the latest choice wins), or as a variable for a resolving spell. Give `InTurnOrder`
  an optional direction ("starting with you and proceeding in the chosen direction"). Add a
  `Restriction::AttackOnlyNearestOpponent { side }`, checked in `may_attack_player` (all such restrictions
  apply together; planeswalkers of that opponent are allowed, battles aren't by the wording). Don't apply it
  to creatures put onto the battlefield attacking. Keep seats separate from turn order: if "Reverse the game's
  turn order" (Aeon Engine, cards-game-rules) is implemented, `neighbors` must not follow the reversed order.
- Tests: 4-player TestGame tests for left/right/chosen direction: Grenzo's Rebuttal (choices in turn order, destroyed together), Hithlain Rope and Barroom Brawl two-player rulings, Order of Succession and Aminatou rotations, Pramikon/Mystic Barrier/Teyo cr!("508.1c", "508.4") with the rulings above.
- Size: M

[R3] Abilities granted with the granting object's name: costs, triggers, values and token creators (reported: "quoted abilities that name the granting card", Leonin Bola family)
- Rules: CR 201.5, 201.5a (the name means that specific source; Gutter Grime example: a token's granted
  ability refers only to the Gutter Grime that created it, also on a copy of the token), 201.5b, 113.7 (the
  granted ability's source is the object that has it), 701.3d (unattach), 111.3, 707.2.
- Engine now: granted_by.rs supports a quote only if every self-reference in it is the card's name
  (statics.rs:144-153; `quote_only_names_card`, :187). `refer_to_granter` (granted_by.rs:21-58) then rewrites
  only "Sacrifice ~" costs and `Sel::This` in an activated ability's effect, and returns None for every other
  kind of ability. Other cost parts keep `Sel::This`, which means the creature. "Unattach ~" compiles to
  `CostPart::Effect(Unattach{This})` (oracle/patterns/cost_parts.rs:378-399), and its payability check needs
  the creature itself to be attached (cost_effects.rs:41-44). As a result Leonin Bola, Heartseeker and Razor
  Boomerang compile, and the round-trip passes because "~" renders the same, but the granted ability can never
  be activated. Probe: `activate` returns Illegal("can't activate"), the target stays untapped and the
  Equipment stays attached. Trusty Boomerang, which only names the granter in its effect, works. The
  normalizer maps both "this creature" and the card's name to "~", so quotes that use both are refused.
  Tokens: nothing binds the creating object into a token's ability. `GameObject::created_by` (object.rs:455)
  is per object, isn't a copiable value, and is read only for linked exiles (`Sel::CreatorLinked`).
- Cards. Compile but broken: Leonin Bola, Heartseeker, Razor Boomerang, and Toralf's Hammer (back face of
  Toralf, God of Fury, which is UNS for its front face). UNS: Blinding Powder, Surestrike Trident, Shuriken
  ("Unattach ~"), Fishing Pole ("Tap ~"), Hankyu ("Remove all aim counters from ~"), The Dominion Bracelet
  ("Exile ~"), Archery Training (value: counters on the granter), Spare Dagger, Sunfire Torch, Trickster's
  Talisman, Dire Flail // Dire Blunderbuss, Animal Friend, Nettlevine Blight, Shifting Shadow, Grothama,
  All-Devouring, Torrent of Lava, Hellish Rebuke ("the player who cast ~"). Token creators: Gutter Grime,
  Saproling Burst (the data's printing is MTGO-only, so `unsupported --paper` skips it; it is a paper card).
  Same compile path, names that aren't self-references (they must stay names): Rekindling Phoenix, Sengir
  Nosferatu, Deathpact Angel ("card named ~"), Ozox, the Clattering King.
- Rulings (UNCOVERED): Leonin Bola and Heartseeker "moves off the creature it was equipping and remains on the battlefield"; Leonin Bola "chooses whether or not to activate the activated ability"; Razor Boomerang "Unattaching Razor Boomerang is a cost to activate the equipped creature", "is no longer on the battlefield by time the equipped creature", "The source of the damage is Razor Boomerang, not the equipped creature"; Surestrike Trident "moves off the creature it was equipping and remains on the battlefield"; Fishing Pole "refers only to the Fishing Pole granting that ability"; Archery Training "each granted ability counts the counters on the Archery Training that granted that ability"; Gutter Grime "each Ooze token remembers which one created it", "the power and toughness of each Ooze token it created will become 0"; Saproling Burst "Each token knows which Saproling Burst it came from"; Sunfire Torch "The source of the damage is the equipped creature, not Sunfire Torch"; Trickster's Talisman "will not be able to sacrifice Trickster's Talisman"; Shifting Shadow "leaves the battlefield after the triggered ability it grants has triggered".
- Approach: in quotes granted to other objects (statics.rs `granted_abilities_to`, and quotes on created
  tokens), compile the card's name to the granter (`Filter::Custom(granted_by::GRANTER)`) everywhere: costs,
  effects, values, conditions and trigger conditions. Keep "this creature/this permanent/this token" as
  `Sel::This`. To do that, distinguish the name from the other self-references before normalization folds both
  into "~", instead of refusing mixed quotes. Engine side: payability and payment for cost parts on the
  granter: Unattach (the granter must be attached), Tap (untapped granter), Exile, Remove counters ("the
  number of counters removed this way"); Sacrifice already works. Let `refer_to_granter` handle triggered and
  static abilities: `layers::acquired_ability` already binds any kind of ability (layers.rs:1931-1933). For
  token creators, bind the placeholder to the creating object when the token's characteristics are built
  (tokens.rs:58 `token_characteristics_in`), so the binding is part of the copiable values and copies keep it.
  A creator that has left the battlefield isn't found, so P/T read 0 (Gutter Grime ruling). Compile "card
  named [name]" in quotes as `Filter::Name`.
- Tests: extend tests/cards/grant_granter.rs. Leonin Bola (activate: the Bola is unattached and stays on the battlefield, the target is tapped, the ability is gone until it's re-equipped), Heartseeker, Razor Boomerang (it's the damage source; not returned if it left), Fishing Pole and Archery Training with two copies, Spare Dagger/Sunfire Torch (the creature is the damage source), Trickster's Talisman (can't sacrifice a Talisman you don't control), Gutter Grime (two Grimes; leaving the battlefield gives 0/0; a copy of the token refers to the same Grime). Cite cr!("201.5a", "113.7", "701.3d") and the rulings above.
- Size: M

[R4] Activation instructions on gained abilities: spending mana as though it were any color or type, "only once each turn" (reported: "spend mana as though ... scoped to abilities gained from an effect" and the Mairsil half of "Mairsil / Havengul Lich")
- Rules: CR 113.10a (an effect that adds an activated ability may include activation instructions, which
  become part of the added ability), 602.5b, 602.5c (a restriction applies to that ability as acquired from
  that object), 609.4b, 118.14 (mana of any type: colorless symbols too).
- Engine now: `StaticEffect::SpendAsAnyColor { applies_to: CostTarget::Abilities(Filter), types }`
  (ability.rs:3004-3009) applies to every activated ability of a matching source. `Game::any_color_mana(p,
  src, ability)` (casting.rs:2670-2698) isn't told which ability is being activated, and it covers colored
  symbols only. Paying {C} with any type exists for spells only (cost_rules.rs:527-549,
  `pay_with_any_mana(any_type)`). `Modification::AddAbilitiesOf { from, which }` (ability.rs:2039-2042;
  ability_grants.rs:125-140 `gained`, 174-190 `snapshot`) can't carry terms for the gained abilities, although
  `ActivatedAbility::max_per_turn` exists (ability.rs:190). Probes: "~ has all activated abilities of all
  creatures your opponents control." compiles. The same with "You may spend mana as though it were mana of any
  color to activate those abilities." does not. Mairsil's static alone compiles, but not with "You may
  activate each of those abilities only once each turn." "Mana of any type can be spent to activate ~'s
  abilities." doesn't compile either.
- Cards (UNS): Drana and Linvala; Scheming Fence (also needs "As ~ enters, you may choose a nonland permanent"
  and "the chosen permanent": compile, partly owned by cards-game-rules); Grell Philosopher (a resolving
  effect, until end of turn, blue mana only); Mairsil, the Pretender (its static; its ETB "exile ... from your
  hand or graveyard and put a cage counter on it" is compile-only: the same sentence with one zone compiles);
  Sharkey, Tyrant of the Shire (any type; text owned by cards-mana); compile-only: Nathan Drake, Treasure
  Hunter ("spells you don't own", "permanents you control but don't own").
- Rulings (UNCOVERED): Mairsil "Each may be activated once each turn", "most likely because its target became illegal before it resolved" (still used up), "it will have the abilities of only cards that player owns in exile with cage counters on them", "treat Mairsil's instance of that ability as though it referenced Mairsil by name instead"; Sharkey "The six types of mana are white, blue, black, red, green, and colorless", "Snow mana is not a type of mana"; Drana and Linvala "loses that ability because the opposing creature with that ability leaves the battlefield".
- Approach: give `AddAbilitiesOf` activation terms, e.g. `terms: GainedTerms { max_per_turn: Option<u32>,
  spend_as: Option<SpendAs { types: Vec<ManaType>, any_type: bool }> }` (serde default). Apply them in
  ability_grants `gained`/`snapshot` to each gained activated ability: lower `max_per_turn` to at most n, and
  set a new `ActivatedAbility::spend_as`. Pass the activated ability to `any_color_mana` so it consults that
  field. Give `SpendAsAnyColor` an `any_type` flag for Sharkey, so colored mana can pay {C}; snow costs are
  unchanged. Compile "You may spend [mana] as though it were mana of any color to activate those abilities"
  and "You may activate each of those abilities only once each turn" as follow-ups that modify the preceding
  AddAbilitiesOf.
- Tests: Mairsil cr!("113.10a", "602.5c") with the rulings (two caged cards with the same ability: two instances, each once per turn; a fizzled activation still counts; after a control change, only the new controller's caged cards). Drana and Linvala cr!("113.10a", "609.4b"): a gained ability can be paid with off-color mana, but an ability D&L has from elsewhere (an Equipment's) can't. Grell Philosopher (blue only, until end of turn). Sharkey cr!("118.14") (colored mana pays a gained land's {C}; the snow ruling).
- Size: S-M

[R5] Open-ended loops: per-player totals across passes, choices of cards that can't enter (reported: "repeat this process until ...", narrowed)
- Rules: CR 608.2c, 101.4b, 110.2a (a player told to put an object onto the battlefield controls it), 119.4,
  810.9 (Two-Headed Giant life payments), 303.4g (an Aura with nothing to enchant stays where it is).
- Engine now: the loop exists. `Effect::RepeatProcess`/`RepeatThisProcess` (repeat_process.rs) decides after
  each pass, and `Effect::InTurnOrder` (turn_order_choices.rs) runs each player's instruction in turn. Primal
  Surge, Ad Nauseam and Hoarder's Greed compile (tests/cr/r608_repeat_process.rs). Probe: Eureka built from
  existing AST worked: RepeatProcess{StoreValue(ANY,0); InTurnOrder{AsPlayer{Iterated, Seq[May{Move ...from
  hand to battlefield}, If PrevHappened StoreValue(ANY,1)]}}; If ANY>0 RepeatThisProcess}. Players were asked
  in order P0, P1, P0, P1, each card entered under its own player's control (without AsPlayer they enter under
  the caster's), and the loop stopped after a pass with no puts. Eureka and Hypergenesis are therefore
  compile-only. Two engine pieces are missing. (a) No per-player number survives across InTurnOrder iterations
  and passes: `Ctx::nums` is keyed by variable only (eval.rs:29), simultaneous.rs frames last for one
  ForEachPlayer, and join forces adds only to a total (kw/join_forces.rs:52-60). (b) A Move of a chosen card
  to the battlefield doesn't drop Auras with nothing to enchant. Only Dig does (library.rs:230-239,
  `attach::could_enter_unspecified`), so Thieves' Auction could keep choosing such an Aura until `MAX_PASSES`.
  Compile gaps: "Starting with you, each player may [instruction]" (iteration_grammar.rs:264 handles only
  "chooses", and :731 only join forces; probe: "Starting with you, each player may draw a card." is UNS);
  "Repeat this process until [condition]" (repeat_process_loop.rs:17 and repeat_process.rs FollowupPattern
  handle only the plain and "once" forms).
- Cards (UNS): Eureka (MTGO printing in the data, so `--paper` skips it; it is a paper card), Hypergenesis,
  Plague of Vermin, Thieves' Auction, Liege of the Hollows. Compile-only with these pieces: Tainted Pact,
  Struggle for Sanity, Timesifter, Kamahl's Summons, Mana Clash (owned by cards-dice-un).
- Rulings (UNCOVERED): Hypergenesis "The permanents enter the battlefield sequentially", "choose not to put a card onto the battlefield during one loop of the process", "Anything that triggers during the resolution of this will wait"; Eureka "choose not to put a card onto the battlefield. It doesn"; Plague of Vermin "the total amount of life each player paid this way is calculated", "counts as three separate instances of losing life", "the process ends when all N players in sequence", "The team's life total is adjusted in between"; Thieves' Auction "can still be chosen by you or another player later in the auction", "If there comes a point where only Auras and no legal permanents left", "You can’t pick a card that can’t legally be returned to the battlefield".
- Approach: add a per-player tally: `Effect::AddToPlayerTally { var, who: PlayerRef, value }` plus
  `Value::PlayerTally { var, who: PlayerRef }`, stored in Ctx and saved with it like `nums`. The "pay any
  amount of life/mana" customs (kw/pay_any_amount_of_life.rs, kw/join_forces.rs) record each player's payment
  in it. Plague of Vermin's tokens are then one ForEachPlayer/CreateToken instruction, so all enter at once.
  Apply `could_enter_unspecified` to cards chosen to be put onto the battlefield (CR 303.4g), and end Thieves'
  Auction's loop when no exiled card can be chosen. Then add the patterns.
- Tests: Hypergenesis/Eureka cr!("608.2c", "101.4b", "110.2a"): sequential entering (an earlier permanent's replacement or static applies to a later one), a player who declined may put a card on a later pass, triggers wait until the end. Plague of Vermin cr!("119.4"): separate life losses, per-player totals, tokens together; Two-Headed Giant cr!("810.9"). Thieves' Auction cr!("303.4g"): an unplaceable Aura stays exiled and the loop ends.
- Size: S-M

[R6] Piles of exiled cards, several face-down piles, three piles, random piles, ordered exiled piles (reported: "face-down piles", narrowed)
- Rules: CR 700.3, 700.3a-d, 406.3 (face-down exile; who may look), 406.3a, 406.4 (face-down exiled cards are
  kept in separate piles), 701.24a (shuffling a face-down pile).
- Engine now: piles.rs supports two piles (`PILE_A`/`PILE_B`, piles.rs:21-29) of objects left where they are,
  with one optional face-down pile (`FACE_DOWN`, used by `SeparateFaceDown` for cards left on top of the
  library). Choose shows the face-down pile as a count (piles.rs:127-160). There is one separator and one
  chooser (`one_player`, :45). Fact or Fiction, Liliana of the Veil, Riddles in the Dark, Curator of
  Destinies, Atris, Fortune's Favor and Ghastly Conscription compile (tests/cr/r700_piles.rs). Missing: piles
  formed as cards are exiled ("Exile the top four cards of your library in a face-down pile, then ... in a
  face-up pile"); two face-down piles and "turn a pile of your choice face up"; separating into two piles that
  are both face down to the chooser; three piles; a pile chosen at random; per-player separations whose pile
  is chosen by another player ("one of their opponents of their choice"); an exiled face-down pile kept in
  order and shuffled, which later abilities draw from ("the top card of the pile you exiled") or put back on
  top of the library.
- Cards (UNS): Abstract Performance, Hostile Negotiations, The Celestial Toymaker (first ability; the second
  is a manual candidate in manual-left.tsv), Phyrexian Portal (MTGO printing in the data; a paper card), Whims
  of the Fates (three piles, chosen at random), Make an Example, Bend or Break, Parallel Thoughts (2 lines),
  Mangara's Tome (2 lines), Triumph of Saint Katherine. Compile-only with the current engine: Sauron's Ransom
  (probe: the same text with "Target opponent looks ..." compiles, but "Choose an opponent. They look ..."
  doesn't), Become Anonymous and Expose the Culprit (a701_manifest_pile.rs:11 handles "... then manifest those
  cards" only; cloak exists in kwa/manifest.rs), Boneyard Parley, Brilliant Ultimatum.
- Rulings (UNCOVERED): Abstract Performance "You choose which opponent chooses the pile while resolving Abstract Performance", "look at the cards in the face-down pile until"; Hostile Negotiations "the second pile will be smaller than the first"; Whims of the Fates "each pile must have an equal chance of being chosen", "You may put Auras or Equipment into one pile and the creatures they were attached to into a different pile"; Make an Example "means the pile of their permanents that you chose"; Phyrexian Portal "One of the piles may have zero cards"; Parallel Thoughts "This allows you to skip draws"; Mangara's Tome "The ability is a replacement effect that replaces a draw".
- Approach: generalize piles.rs to a list of piles, each `(objects, face_down)`, kept in a variable family so
  ForEachPlayer frames keep per-player piles. Actions: SeparateInto { n, face_down per pile }, ExileAsPile {
  what, face_down }, TurnFaceUp { chooser }, Choose { chooser (incl. "an opponent of their choice"), at_random
  }. A face-down pile stays hidden from players who may not look at it (406.3; the observation layer). Ordered
  exiled piles: keep the linked exile order (`GameObject::linked` is ordered, object.rs:452), add a shuffle of
  a linked pile (701.24a) and a "top card of the pile" selection for the draw replacements.
- Tests: extend tests/cr/r700_piles.rs, cr!("700.3", "700.3a", "700.3d", "406.3", "406.4", "701.24a"), plus a card test per card with the rulings above.
- Size: M

[R7] Spells cast "this way": effects tied to the permission a spell was cast with (new; includes Havengul Lich from "Mairsil / Havengul Lich")
- Rules: CR 601.3, 400.7h (other parts of an effect that allows a card to be cast can find the spell it
  became), 400.7i (the same for a land played), 400.7a (an effect on a permanent spell continues to apply to
  the permanent it becomes), 603.7a-c (delayed triggers; "this turn"), 608.2h.
- Engine now: a cast records which permission it uses (`CastOption::permission`, casting.rs:35-40;
  permissions.rs). After the cast, `permissions::spell_cast_with` (permissions.rs:647-672) applies only
  `exile_instead`, and `CastInfo` (object.rs:261-303) keeps only `permission_cost_increase`. So nothing can
  tell later which permission, or which effect, a spell was cast with, and `PlayTerms` (ability.rs:3173-3241)
  has no effect for the spell cast. Probes: "{1}: You may cast target creature card in a graveyard this turn."
  compiles, but adding "When you cast it this turn, draw a card." doesn't. "You may cast creature spells with
  power 4 or greater from the top of your library." compiles; adding "If you cast a creature spell this way,
  it gains haste until end of turn." doesn't. "Exile the top card of your library. You may play that card this
  turn." compiles; adding "When you play a card this way, ..." doesn't.
- Cards (UNS): Havengul Lich, The Ruinous Powers, Fires of Mount Doom, Apple of Eden, Isu Relic, Share the
  Spoils, Galea, Kindler of Hope, Thundermane Dragon, Strago and Relm, Alaundo the Seer (inside the ability it
  grants). Related: Ogre Battlecaster ("When you cast that spell" after casting it during resolution: probably
  a reflexive trigger, compile-level).
- Rulings (UNCOVERED): Havengul Lich "creates a delayed triggered ability that triggers when you cast the creature card", "gains the activated abilities of the card as it existed in the graveyard while that card is still a spell on the stack", "cast the creature card that turn, Havengul Lich" (no cast, no abilities); Fires of Mount Doom "using that casting permission"; Share the Spoils "not one of each"; The Ruinous Powers "You must follow all normal timing rules when casting a spell or playing a land using the permission".
- Approach: record the permission on the spell. Add `CastInfo::permission: Option<PermissionRef { source,
  effect/link id }>`, set in `spell_cast_with`, and the same for a land played with a permission. Add a
  "cast/played this way" form of the cast trigger (`TriggerCond::CastSpell` + land played) that matches a
  permission given by the trigger's own source and link (statics: Galea, Share the Spoils) or by the effect
  that created the delayed trigger (Fires of Mount Doom, The Ruinous Powers, Apple of Eden, Havengul Lich;
  lasts this turn, 603.7b). For "If you cast a [spell] this way, it gains haste until end of turn", add an
  on-cast modification in `PlayTerms` applied to the spell; by 400.7a it carries over to the permanent.
  Havengul Lich: "that card" is the spell (400.7h). Gain the abilities from the card's last known information
  in the graveyard (`ability_grants::snapshot` on the old object).
- Tests: Havengul Lich cr!("400.7h", "603.7b") (the trigger resolves before the creature spell; casting the card some other way doesn't count; no cast that turn means no abilities). Fires of Mount Doom cr!("400.7i") (only a play with that permission deals the damage). Thundermane Dragon cr!("400.7a") (the creature keeps haste). Galea (only Equipment cast with Galea's permission gains the attach trigger).
- Size: M

ESTIMATED SIZE: R1 M, R2 M, R3 M, R4 S-M, R5 S-M, R6 M, R7 M: about L in total. The item splits cleanly into three workers if needed: players (R1, R2, R5), abilities and permissions (R3, R4, R7), piles (R6).

OVERLAPS: Most listed texts are also in the round-2 remainder shards (shard-1..4.tsv). This item owns the engine pieces and these cards' texts, so card items should leave them alone or coordinate. Pending round-1 items hold some of the texts: cards-game-rules (Hithlain Rope, Bucknard's Everfull Purse, Scheming Fence's "you may choose a nonland permanent", Aeon Engine's turn-order reversal, the draft seats of Paliano and Regicide), cards-mana (Sharkey), cards-dice-un (Mana Clash). The main files are shared with finished items: simultaneous.rs (gap-simultaneity), ability_grants.rs and granted_by.rs (gap-ability-grants), permissions.rs (gap-cast-permissions), piles.rs (roundtrip-clusters-2, cards-library-dig). New AST variants must be handled in oracle/render as well.

DROPPED (reported, but fixed in the engine):
- Ordered "starting with you" choices, where each player in turn order makes a choice knowing the earlier ones (CR 101.4b): `Effect::InTurnOrder` (ability.rs:3698-3703, turn_order_choices.rs), compiled by iteration_grammar.rs:264-370. Votes (kwa/vote.rs) and join forces (kw/join_forces.rs) do the same. 33 of the 46 legal cards with "starting with" text compile, tested in tests/cards/iteration_modal_turn_order.rs and tests/actions/a701_038_vote.rs. The other 13 fail for reasons covered above (R1/R2: Order of Succession, Grenzo's Rebuttal; R5: Eureka, Hypergenesis, Plague of Vermin, Thieves' Auction; R6: Whims of the Fates) or on vote-outcome phrasing that belongs to card items (Custodi Squire, Council Guardian, Split Decision, Selvala's Stampede, Trial of a Time Lord, Expropriate).
- The loop engine itself, and the face-down/face-up separation of library cards (see R5 and R6 for what remains).

DONE when every gap above is implemented per the CR, the listed cards compile and behave correctly in tests (and the three broken supported cards work), and the listed rulings are cited by tests that exercise them (or reported as remaining with a reason).

WORKING NOTES
- Read CLAUDE.md first; read the actual CR text in data/comprehensive-rules.txt for every rule cited here (it is newer than training data). File:line references are as of commit 06aa0f8a and may have shifted; search for the named functions and types.
- How this was checked. Compiler probes: copy data/ to a scratch directory with extra cards whose oracle text is the sentence in question, then run `mtg-tools unsupported --card ...` with `MTG_DATA_DIR` pointing at it. Engine probe: a throwaway tests/gap_probe.rs in a separate worktree, removed afterwards. It contained (a) Leonin Bola, Heartseeker, Razor Boomerang and Trusty Boomerang attached to Grizzly Bears with `t.activate(P0, bears, i, &[target])`; (b) a custom sorcery with `ForEachPlayer { EachPlayer, GainControl { what: All(creature ∧ ControlledByPlayer(Each(OpponentOf(Iterated)))), who: Iterated, duration: Permanent } }`; (c) the Eureka AST in R5; (d) ForEachPlayer{AsPlayer{CreateToken}} + Tap{Var(CREATED)}. (a), (b) and (d) make good failing-first regression tests.
- "UNS" means `cargo run --release -q -p mtg-tools -- unsupported --card "Full Name"` reports the line unsupported (use the full "A // B" name for split, flip and double-faced cards; the command silently skips cards that are legal in no format, and `--paper` also skips cards whose data printing isn't paper, such as Eureka, Saproling Burst and Phyrexian Portal). Do the engine work first, then add or extend oracle patterns (src/oracle/patterns/) so the listed cards compile, without letting the compiler accept text the engine doesn't implement faithfully.
- Prefer new modules and registry files (src/kw/, src/kwa/, src/oracle/patterns/, e.g. a seats.rs next to multiplayer/attack.rs) plus small hooks in core files. If you add an AST variant, handle it everywhere it's matched; no todo!/unimplemented! on reachable paths.
- Tests: use real cards through TestGame in crates/mtg-engine/tests/{cr,keywords,actions,rulings,cards}/; cite the rules each test exercises with cr!(...) and the Scryfall rulings listed here with ruling!("Card", "substring"). The ruling substrings above were checked against data/rulings.jsonl.gz and can be used as they are (some contain typographic apostrophes; copy them exactly). Check status with `cargo run --release -q -p mtg-tools -- rulings-coverage --text "substring"`. Every test must fail if the engine got the behaviour wrong. When no supported real card exercises a rule, a `CardDef::custom` is acceptable for the engine test, but also compile at least one listed real card.
- Report anything you could not finish as a remaining gap with its reason; do not approximate silently.
