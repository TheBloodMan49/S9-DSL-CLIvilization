# CLIvilization – Métamodèle vs AST

Ce document compare le métamodèle avec l’AST TypeScript généré par Langium (`lang/generated/ast.ts`).  
La table suivante établit une correspondance entre les concepts du métamodèle et les nœuds/propriétés de l’AST, suivie de commentaires sur les arbitrages réalisés.

## Table de correspondance

| Métamodèle                 | AST TypeScript             | Commentaire / Arbitrage |
|----------------------------|----------------------------|------------------------|
| Model                      | Model                      | Directement mappé, `sections: Section[]`. |
| Section (interface)        | Section (type union)       | Fusion en type union AST (pas d’interface). |
| Cities                     | Cities                     | Propriété `cities: City[]`. |
| City                       | City                       | Propriétés renommées pour correspondre au style camelCase (`nbSlotsBuildings`, `startingResources`). |
| PlayerType (enum)          | PlayerType (type union)    | Enum TP1 remplacé par type union string (`'PLAYER' | 'AI'`). |
| BuildingInstanceArray      | BuildingInstanceArray      | Propriété `elements: BuildingInstance[]`. |
| BuildingInstance           | BuildingInstance           | Propriétés conservées (`id_building`, `level`). |
| UnitInstanceArray          | UnitInstanceArray          | Propriété `units: UnitInstance[]`. |
| UnitInstance               | UnitInstance               | Propriétés conservées (`id_units`, `nb_units`). |
| BuildingArray / UnitArray  | ValueArray                 | Fusion en un seul type générique `ValueArray` pour AST. |
| Game                       | Game                       | Propriétés renommées en camelCase (`mapX`, `mapY`, `currentTurn`, `uiColor`). |
| VictoryConditions          | VictoryConditions          | Directement mappé. |
| BuildingDefArray           | BuildingDefArray           | AST hérite de `Section` via superTypes. |
| BuildingDef                | BuildingDef                | Propriétés conservées et renommées (`buildTime`, `prodUnitId`). |
| Production                 | Production                 | Propriétés renommées (`prodType`, `prodUnitId`). |
| ProductionType (enum)      | ProductionType (type union)| Enum TP1 remplacé par type union string (`'unit' | 'ressource'`). |
| PrereqArray                | PrereqArray                | Directement mappé. |
| Prereq                     | Prereq                     | Directement mappé. |
| UnitDefArray               | UnitDefArray               | AST hérite de `Section` via superTypes. |
| UnitDef                    | UnitDef                    | Propriétés conservées (`name`, `attack`). |
| Value                      | Value                      | Mappé sur string; regex vérifie ID ou STRING. |
| COLOR                      | string                     | Directement stocké comme string (`#RRGGBB`). |

## Commentaires sur les arbitrages

- **Renommage** : les propriétés du métamodèle en snake_case ont été converties en camelCase pour respecter la convention TypeScript.  
- **Fusion / simplification** : `BuildingArray` et `UnitArray` ont été fusionnés en `ValueArray` dans l’AST, car ils servent tous deux de listes de références.
- **Enum → union string** : `PlayerType` et `ProductionType` sont représentés par des type unions dans l’AST pour simplifier la génération TypeScript.
- **Héritage / interface** : l’interface `Section` du métamodèle est traduite en union de types AST (`Section = BuildingDefArray | Cities | Game | UnitDefArray | VictoryConditions`).  
- **Références** : les références croisées (`id_building`, `id_units`, `prodUnitId`) sont conservées, mais AST ne les contraint pas directement ; la validation est prévue côté moteur.
- **Suppression / ajout** : aucun concept majeur n’a été supprimé, mais certains types utilitaires (`BuildingArray`, `UnitArray`) ont été consolidés pour simplifier l’AST.

