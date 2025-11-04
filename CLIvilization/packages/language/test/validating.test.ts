import { beforeAll, describe, expect, it } from "vitest";
import { createClIvilizationServices } from "../src/clivilization-module.js";
import { parseHelper } from "langium/test";
import { Model } from "../src/index.js";
import { EmptyFileSystem } from "langium";


let services: ReturnType<typeof createClIvilizationServices>;
let parse:    ReturnType<typeof parseHelper<Model>>;

beforeAll(async () => {
    services = createClIvilizationServices(EmptyFileSystem);
    const doParse = parseHelper<Model>(services.ClIvilization);
    parse = (input: string) => doParse(input, { validation: true });

    // activate the following if your linking test requires elements from a built-in library, for example
    // await services.shared.workspace.WorkspaceManager.initializeWorkspace([]);
});


describe('Validating', () => {

    it('no validation errors when starting building is defined in [buildings]', async () => {
        const input = `
[buildings]
house {
    cost = 10
    build_time = 2
    slots = 1
    production = { type = "none" id_units = 0 amount = 0 time = 0 cost = 0 }
    prerequisites = []
}

[cities]
city1 {
    x = 1
    y = 1
    color = #ffffff
    starting_resources = 100
    player_type = PLAYER
    civilization = "civ"
    nb_slots_buildings = 2
    starting_buildings = [ { id_building = house level = 1 } ]
    nb_slots_units = 0
    starting_units = []
}
        `;
        const res = await parse(input, { validation: true });
        expect(res).toBeDefined();
        expect(res!.diagnostics!.length).toBe(0);
    });

    it('reports an error when a starting building references a non-existing building', async () => {
        const input = `
[cities]
city1 {
    x = 1
    y = 1
    color = #ffffff
    starting_resources = 100
    player_type = PLAYER
    civilization = "civ"
    nb_slots_buildings = 2
    starting_buildings = [ { id_building = house level = 1 } ]
    nb_slots_units = 0
    starting_units = []
}
        `;
        const res = await parse(input);
        expect(res).toBeDefined();
        const diagnostics = res!.diagnostics;
        expect(diagnostics!.length).toBeGreaterThan(0);
        // ensure our custom validation message appears
        const hasMsg = diagnostics!.some(d => /not defined in \[buildings\] section/.test(d.message));
        expect(hasMsg).toBe(true);
    });

    it('reports error on duplicate city names', async () => {
        const input = `
[cities]
city1 {
    x = 1
    y = 1
    color = #ffffff
    starting_resources = 0
    player_type = PLAYER
    civilization = "civ"
    nb_slots_buildings = 0
    starting_buildings = []
    nb_slots_units = 0
    starting_units = []
}
city1 {
    x = 2
    y = 2
    color = #000000
    starting_resources = 0
    player_type = PLAYER
    civilization = "civ"
    nb_slots_buildings = 0
    starting_buildings = []
    nb_slots_units = 0
    starting_units = []
}
        `;
        const res = await parse(input);
        expect(res).toBeDefined();
        const diagnostics = res!.diagnostics || [];
        const hasDupCity = diagnostics.some(d => /Duplicate city name 'city1'/.test(d.message));
        expect(hasDupCity).toBe(true);
    });

    it('reports error on duplicate building definition names', async () => {
        const input = `
[buildings]
house {
    cost = 1
    build_time = 1
    slots = 1
    production = { type = "none" id_units = 0 amount = 0 time = 0 cost = 0 }
    prerequisites = []
}
house {
    cost = 2
    build_time = 2
    slots = 1
    production = { type = "none" id_units = 0 amount = 0 time = 0 cost = 0 }
    prerequisites = []
}
        `;
        const res = await parse(input);
        expect(res).toBeDefined();
        const diagnostics = res!.diagnostics || [];
        const hasDupBuilding = diagnostics.some(d => /Duplicate building name 'house'/.test(d.message));
        expect(hasDupBuilding).toBe(true);
    });

    it('reports error on duplicate unit definition names', async () => {
        const input = `
[units]
soldier {
    attack = 1
}
soldier {
    attack = 2
}
        `;
        const res = await parse(input);
        expect(res).toBeDefined();
        const diagnostics = res!.diagnostics || [];
        const hasDupUnit = diagnostics.some(d => /Duplicate unit name 'soldier'/.test(d.message));
        expect(hasDupUnit).toBe(true);
    });

    it('no validation errors when starting unit id matches a unit name (numeric name)', async () => {
        const input = `
[units]
aaa {
    attack = 1
}

[cities]
city1 {
    x = 1
    y = 1
    color = #ffffff
    starting_resources = 0
    player_type = PLAYER
    civilization = "civ"
    nb_slots_buildings = 0
    starting_buildings = []
    nb_slots_units = 1
    starting_units = [ { id_units = aaa } ]
}
        `;
        const res = await parse(input);
        expect(res).toBeDefined();
        const diagnostics = res!.diagnostics || [];
        const hasUnitError = diagnostics.some(d => /not defined in \[units\] section/.test(d.message));
        expect(hasUnitError).toBe(false);
    });

    it('reports an error when a starting unit references a non-existing unit', async () => {
        const input = `
[cities]
city1 {
    x = 1
    y = 1
    color = #ffffff
    starting_resources = 0
    player_type = PLAYER
    civilization = "civ"
    nb_slots_buildings = 0
    starting_buildings = []
    nb_slots_units = 1
    starting_units = [ { id_units = aaa } ]
}
        `;
        const res = await parse(input);
        expect(res).toBeDefined();
        const diagnostics = res!.diagnostics || [];
        const hasMsg = diagnostics.some(d => /not defined in \[units\] section/.test(d.message));
        expect(hasMsg).toBe(true);
    });

});
