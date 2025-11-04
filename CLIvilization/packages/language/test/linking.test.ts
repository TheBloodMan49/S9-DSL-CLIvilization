import { LangiumDocument, EmptyFileSystem } from "langium";
import { parseHelper, clearDocuments } from "langium/test";
import { afterEach, beforeAll, describe, it, expect } from "vitest";
import { createClIvilizationServices } from "../src/clivilization-module.js";
import { Model } from "../src/index.js";


let services: ReturnType<typeof createClIvilizationServices>;
let parse:    ReturnType<typeof parseHelper<Model>>;
let document: LangiumDocument<Model> | undefined;

beforeAll(async () => {
    services = createClIvilizationServices(EmptyFileSystem);
    parse = parseHelper<Model>(services.ClIvilization);

    // activate the following if your linking test requires elements from a built-in library, for example
    // await services.shared.workspace.WorkspaceManager.initializeWorkspace([]);
});

afterEach(async () => {
    document && clearDocuments(services.shared, [ document ]);
});


describe('Linking tests', () => {

    it('parses a model with a building definition and a city that references it (by name)', async () => {
        const modelText = `
[buildings]
Farm {
    cost = 10
    build_time = 1
    slots = 1
    production = {
        type = ressource
        amount = 1
        time = 1
        cost = 0
    }
    prerequisites = []
}

[units]
Worker {
    attack = 1
}

[cities]
city1 {
    x = 1
    y = 2
    color = #ff0000
    starting_resources = 100
    player_type = PLAYER
    nb_slots_buildings = 2
    starting_buildings = [ { id_building = Farm level = 1 } ]
    nb_slots_units = 2
    starting_units = [ { id_units = Worker nb_units = 1 } ]
}
        `.trim();

        document = await parse(modelText);
        const model = document.parseResult.value;

        // No parse diagnostics
        expect(document.parseResult.parserErrors).toHaveLength(0);
        expect(document.parseResult.lexerErrors).toHaveLength(0);

        // Find building definition section and check name
        const buildingsSection = (model.sections as any[]).find(s => (s as any).buildings !== undefined);
        expect(buildingsSection).toBeDefined();
        const building = buildingsSection.buildings[0];
        expect(building.name).toBe('Farm');

        // Find cities section and check that the city's starting building references the same name
        const citiesSection = (model.sections as any[]).find(s => (s as any).cities !== undefined);
        expect(citiesSection).toBeDefined();
        const city = citiesSection.cities[0];
        const startingBuildings = city.buildings.elements;
        expect(startingBuildings).toHaveLength(1);
        expect(startingBuildings[0].id_building).toBe('Farm');
    });

});
