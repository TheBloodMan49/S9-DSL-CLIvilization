# TrustMeBro's CLIvilization

## Introduction

This project implements a Domain-Specific Language (DSL) for defining and managing game state in a Civilization-like game. 
The DSL allows users to define buildings, units, and cities with various attributes and relationships.

The project is built using langium, a language development framework that simplifies the creation of DSLs and Rust using `ratatui` for building terminal user interfaces.

## Generating the AST

To generate the Abstract Syntax Tree (AST) for the DSL, follow these steps:
1. Go to the project directory:

```bash
cd CLIvilization
```

2. Install the dependencies:

```bash
npm install
```

3. Generate the parser and AST:

```bash
npm run langium:generate
```

4. Build the project:

```bash
npm run build
```

You will find the generated AST files in the `package/language/src/generated` directory.

## VSCode Extension

To use the DSL in Visual Studio Code, you can install the provided extension using the [`vscode-CLIvilization-0.0.1.vsix`](vscode-CLIvilization-0.0.1.vsix) package (right-click and "Install extension VSIX").

To build a new version of the extension, run:

```bash
cd CLIvilization/package/extension
npm install
```

Then, compile the extension:

```bash
vsce package
```

The newly created `.vsix` file can be found in the same directory.

## MetaModel

The metamodel of the DSL is defined using a class diagram as shown below:

![Metamodel](model/metamodel.png)

The textual representation of the metamodel in PlantUML format can be found in the `model/metamodel.puml` file.


## Variants

The DSL supports different game variants, which can be specified in the game definition.
Each variant can modify the behavior of buildings, units, and other game mechanics.

You can find example variant definitions in the [`examples/`](examples/) directory.
You can also read the variability notes in [`docs/variability.md`](docs/variability.md).

## Validation

The DSL includes validation rules to ensure that the defined game state is consistent and adheres to the expected structure.
Validation checks include:
- Cross references with BuildingInstance and UnitInstance on id_building and id_unit
- Ensuring that every City, BuildingInstance, and UnitInstance has a unique identifier

Validation are in [`CLIvilization/packages/language/src/clivilization-validator.ts`](CLIvilization/packages/language/src/clivilization-validator.ts)

## Tests

Tests for the AST and validation are located in the [`CLIvilization/packages/language/test`](CLIvilization/packages/language/test) folder.
There are 3 test files:
- `parsing.test.ts` : tests for parsing the DSL and generating the AST
- `linking.test.ts` : tests for cross-references in the AST
- `validation.test.ts` : tests for validating the DSL definitions

You can run the tests using:

```bash
cd CLIvilization/
npm test
```
