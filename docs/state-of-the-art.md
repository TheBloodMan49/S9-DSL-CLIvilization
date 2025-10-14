# Mapping of Existing Systems – DSLs and Frameworks for **CLIvilization**

## Comparative Table

| Name | Category | Lang/Runtime | License | Game Families | Rules Expressivity | Variability | Interop | Maturity & Activity | URL |
|------|-----------|--------------|----------|----------------|--------------------|--------------|----------|---------------------|-----|
| **GDL/GDL-II/GDL-III** | DSL (Rules) | Prolog-like logic | Open (Academic) | Turn-based, board games, perfect/imperfect info | **State**: Complete (facts, relations)<br>**Randomness**: GDL-II (random)<br>**Hidden info**: GDL-III (sees/knows) | Compile-time (fixed rules) | Standard GGP protocol, text format | Mature, academic (2005+), active | http://ggp.stanford.edu/ |
| **OpenSpiel** | AI/RL Framework | C++/Python | Apache 2.0 | Board games, cards, auctions, coordination | **State**: Extensive-form games<br>**Randomness**: ChanceNode support<br>**Hidden info**: Information states, observations | Runtime (game parameters), extensible C++ API | Python bindings, standard RL APIs (Gym-like) | Very mature (DeepMind), active (2019+) | https://github.com/google-deepmind/open_spiel |
| **Ludii** | DSL (Rules) + UI | Java (own DSL) | GPL v3 | Board games (1000+ games), combinatorial | **State**: Spatial graphs, stacks, tracks<br>**Randomness**: Dice, random<br>**Hidden info**: Hidden info (limited) | Compile/runtime (rules with parameters, presets) | .lud format (text-based), export/import positions, AI API | Very active (2019+), academic + community | https://ludii.games/ |
| **PGX (Polygames)** | AI Framework | C++/Python | MIT | Board games, puzzles | **State**: Tensor-based observations<br>**Randomness**: Stochastic games<br>**Hidden info**: Partial observability | Runtime (game configs JSON) | C++ game interface, Python API, Torchscript | Active (Facebook/Meta AI, 2020+) | https://github.com/facebookincubator/polygames |
| **Tabletop Simulator** | UI Framework | Lua (scripting) | Proprietary | Tabletop games, cards, 3D boards | **State**: 3D physics objects<br>**Randomness**: Deck shuffle, dice<br>**Hidden info**: Hidden zones, hand | Runtime (Lua scripts, JSON objects) | Steam Workshop, JSON save format, Lua API | Very mature, very active (2015+) | https://api.tabletopsimulator.com/ |
| **BoardGameGeek XML API** | Format/Protocol | XML/JSON | Public API | Board game metadata | **State**: N/A (metadata only)<br>**Randomness**: N/A<br>**Hidden info**: N/A | N/A | REST API, XML/JSON formats | Mature, stable (2005+) | https://boardgamegeek.com/wiki/page/BGG_XML_API2 |
| **UCI Protocol** | Format/Protocol | Text-based | Public domain | Chess engines | **State**: Position (FEN), moves (algebraic)<br>**Randomness**: N/A<br>**Hidden info**: N/A | Runtime (options, variants) | Universal standard, text I/O | Very mature (1998+), reference | http://wbec-ridderkerk.nl/html/UCIProtocol.html |
| **Godot Engine** | UI Framework | GDScript/C# | MIT | 2D/3D games, flexible UI | **State**: Node-based scene tree<br>**Randomness**: RandomNumberGenerator<br>**Hidden info**: Custom logic | Runtime (scenes, scripts, resources) | GDScript, C# bindings, export formats | Very active, production-ready (2014+) | https://godotengine.org/ |
| **Unciv** | DSL (Rules) + UI | Kotlin/LibGDX | MPL 2.0 | 4X Civilization-like, turn-based strategy | **State**: Tiles, cities, units, techs<br>**Randomness**: Combat, barbarians<br>**Hidden info**: Fog of war, diplomacy | Runtime/Compile (JSON rulesets, mods via GitHub) | JSON mod format, save files, modding API | Very active (2018+), 460+ contributors | https://github.com/yairm210/Unciv |

---

## Analysis by Family

### DSLs and Rule Systems

#### Unciv

- Scope: Open-source clone of Civilization V with a JSON-based modding system  
- Strengths:
  - Structured JSON format for nations, units, buildings, technologies, social policies, promotions
  - “Uniques” system for special abilities of civilizations and units
  - Mods via GitHub, easy desktop editing
  - Built in Kotlin
- Expressivity: Complete for 4X (fog of war, combat, diplomacy, technology, culture, production)
- Integration with CLIvilization:
  - Direct JSON model: Structure of Nations.json, Units.json, Buildings.json, etc.
  - “Uniques” inspiration: Declarative system for special abilities
  - Export target: Generate Unciv mods from your DSL
  - Asset reuse: Compatible with the existing Unciv ecosystem

#### GDL (Game Description Language)

- Scope: Logical language for formally describing rules of turn-based games  
- Strengths:
  - Declarative Prolog-based syntax (facts, rules, relations)  
  - Extensions: GDL-II (randomness), GDL-III
  - Standard for General Game Playing since 2005  
- Expressivity: Full state representation (initial, terminal, legal, next), randomness (GDL-II), knowledge/belief (GDL-III)
- Integration with CLIvilization:
  - Inspiration for declarative syntax
  - Possible compilation target (export GDL for AI benchmarks)
  - Really verbose

#### Ludii

- Scope: DSL dedicated to combinatorial board games
- Strengths:
  - Highly concise domain-specific syntax
  - Supports spatial graphs, stacks, tracks, and connections  
  - Integrated UI + AI API (UCT, Alpha-Beta, minimax)  
- Expressivity: Made for board games, randomness (dice), limited hidden info
- Integration with CLIvilization:
  - Excellent model for spatial/grid rule DSLs  
  - Possible export target (.lud)
  - Inspiration for key concepts (sites, regions, components)

---

### AI/RL Frameworks and Solvers

#### OpenSpiel

- Scope: C++/Python framework for multi-agent reinforcement learning in games
- Strengths:
  - 80+ implemented games (extensive-form)
  - Algorithms: MCTS, CFR, minimax, alpha-beta, DQN, A3C, etc.
  - Supports perfect/imperfect info, simultaneous/sequential games
- Expressivity: Very complete (observations, chance nodes, information states)
- Integration with CLIvilization:
  - Natural target for advanced AI integration
  - C++ interface for registering custom games

#### PGX (Polygames)

- Scope: Meta/Facebook framework for self-play and RL research
- Strengths:
  - Optimized for tree search + neural networks
  - GPU support via PyTorch
  - Simple JSON-based game configurations
- Expressivity: Tensor-based observations, stochastic games, partial observability
- Integration with CLIvilization:
  - Requires C++ game interface implementation
  - Good for AI benchmarking  

---

### UI Engines and Frameworks

#### Tabletop Simulator

- Scope: 3D tabletop game simulator with Lua scripting
- Strengths:
  - Very flexible scripting system (events, objects, zones)
  - Steam Workshop with thousands of mods
  - Full support for cards, boards, dice
- Expressivity: 3D physics, Turing-complete scripting, networking
- Integration with CLIvilization:
  - Possible graphical mode target (export JSON + Lua script)
  - Inspiration for object/zone/scripting API
  - Requires DSL to TTS JSON conversion format

#### Godot Engine

- Scope: General-purpose open-source 2D/3D game engine
- Strengths:
  - Flexible scene tree (node-based)
  - GDScript (Python-like) or C#
  - Excellent for 2D tile-based games
- Expressivity: Complete for 2D/3D, networking, scripting
- Integration with CLIvilization:
  - Ideal target for rich graphical UI
  - GDScript accessible for DSL integration
  - Export to multiple platforms

---

### Formats and Protocols

#### UCI Protocol

- Scope: Text-based standard for communication between chess engines
- Strengths:
  - Simple textual format (position, bestmove, info)
  - Full interoperability within chess ecosystem
  - De facto standard since 1998
- Integration with CLIvilization:
  - Inspiration for text-based engine - UI protocol
  - Model for CLI/REPL interaction
  - Can be extended for turn-based strategy games

#### BoardGameGeek XML API

- Scope: Metadata API for board game database
- Strengths:
  - Rich metadata (mechanics, categories, stats)
  - Stable REST API
- Integration with CLIvilization:
  - Enrich game metadata
  - Interop with board game community
