# CLIvilization – Services Notes

## Légalité & coups
Le DSL de CLIvilization ne gère pas directement les règles ou les actions en jeu, mais définit toutes les données nécessaires pour qu’elles soient interprétées correctement au runtime. Il garantit la cohérence de la configuration initiale (position des cités, ressources, unités, bâtiments disponibles, conditions de victoire, etc.), ce qui assure que la partie démarre dans un état valide. Les règles de légalité (comme les actions possibles ou la fin de partie) sont déterminées par le moteur du jeu, en se basant sur ces paramètres définis dans le DSL.

## Complexité
Le DSL expose des paramètres influençant la complexité du jeu, comme la taille de la carte, le nombre de cités, les types d’unités ou de bâtiments, et les conditions de victoire. Ces éléments permettent d’ajuster la difficulté et la profondeur stratégique sans modifier le code du moteur. L’espace des possibles reste borné par les définitions contenues dans le fichier de configuration, ce qui permet de créer des variantes équilibrées et de mesurer leur impact sur la durée ou la richesse des parties.

## Mode texte
CLIvilization fonctionne entièrement en TUI (Text User Interface), avec une interface ASCII interactive. Le DSL fournit une notation textuelle stable et lisible pour décrire l’état initial du jeu, tandis que le joueur interagit ensuite via des commandes textuelles (`move`, `build`, `end_turn`, etc.). Cette approche rend la configuration, le chargement et la reprise de parties simples, reproductibles et facilement partageables entre utilisateurs.

## Graphique / skin
Le rendu du jeu s’appuie sur une représentation ASCII colorée : chaque cité, unité et zone de la carte est affichée avec des caractères et couleurs personnalisés. Le DSL permet de définir des attributs visuels comme les couleurs associées aux joueurs et à l’interface, ce qui ouvre la voie à des “skins” textuels et à une personnalisation complète de l’apparence du jeu sans changer le code. Le système reste extensible à d’autres modes d’affichage si nécessaire.

## IA basique
Une IA rudimentaire est prévue, reposant sur une heuristique simple exploitant les informations initialisées par le DSL (ressources, unités disponibles, positions, etc.). Le langage facilite la génération de configurations adaptées à des tests d’IA, en permettant de contrôler précisément les paramètres de départ. Cela simplifie la mise au point et l’évaluation de comportements automatiques cohérents.

## IA plus forte (optionnel)
Le format du DSL étant entièrement textuel et structuré, il peut servir de base à des IA plus avancées comme celles utilisant la recherche d’états (MCTS) ou l’apprentissage par renforcement. Rien n’est encore implémenté à ce niveau, mais la structure actuelle du langage permettrait d’extraire facilement les informations nécessaires pour explorer ces pistes à terme.

## Play with LLM
La représentation textuelle claire et complète du jeu rend possible une interaction directe avec un LLM. Les données de configuration ou d’état peuvent être exportées en texte, envoyées à un modèle de langage pour demander une stratégie, une prédiction ou une action conseillée. Cela ouvre la possibilité d’un mode de jeu assisté par IA générative ou d’un adversaire contrôlé par un LLM.
