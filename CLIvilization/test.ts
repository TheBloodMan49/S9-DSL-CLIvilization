import {EmptyFileSystem} from "langium"
import {parseHelper} from "langium/test"
import {createClIvilizationServices, Model} from "clivilization-language";
import * as fs from "node:fs";

const services = createClIvilizationServices(EmptyFileSystem)
const parse = parseHelper<Model>(services.ClIvilization);

fs.readFile("../examples/variant_1/game.civ", "utf-8", async (err, data) => {
    if (err) {
        console.error(err)
    } else {
        const document = await parse(data, {})
        console.log(JSON.stringify(document.parseResult.value, (key, value) => {
            if (key.startsWith('$')) return undefined; // Exclude Langium-internal properties
            return value;
        }, 2))
    }
})