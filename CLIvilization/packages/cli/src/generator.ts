import type { Model } from 'clivilization-language';
import * as fs from 'node:fs';
import { extractDestinationAndName } from './util.js';
import {exec} from "node:child_process";
import * as util from "node:util";

export async function generateOutput(model: Model, source: string, destination: string): Promise<string> {
    const data = extractDestinationAndName(destination);

    if (!fs.existsSync(data.destination)) {
        fs.mkdirSync(data.destination, { recursive: true });
    }

    // Set the environment variable with the JSON output
    process.env["CONFIG_BLOB"] = JSON.stringify(model, (key, value) => {
        if (key.startsWith('$')) return undefined; // Exclude Langium-internal properties
        return value;
    });

    // Build the executable
    await util.promisify(exec)(
        `cd $(git rev-parse --show-toplevel)/engine || exit 1;
        cargo build --release 2> /dev/null || exit 1;
        cp ./target/release/clivilization-engine ${data.destination}/${data.name} || exit 1`
    )

    return destination;
}
