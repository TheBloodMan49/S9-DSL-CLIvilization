import type { Model } from 'clivilization-language';
import { createClIvilizationServices, CLIvilizationLanguageMetaData } from 'clivilization-language';
import chalk from 'chalk';
import { Command } from 'commander';
import { extractAstNode } from './util.js';
import {generateBinary, generateAST} from './generator.js';
import { NodeFileSystem } from 'langium/node';
import * as url from 'node:url';
import * as fs from 'node:fs/promises';
import * as path from 'node:path';
import {execSync} from "child_process";
import yoctoSpinner from "yocto-spinner";
const __dirname = url.fileURLToPath(new URL('.', import.meta.url));

const packagePath = path.resolve(__dirname, '..', 'package.json');
const packageContent = await fs.readFile(packagePath, 'utf-8');

export const generateAction = async (source: string, destination: string): Promise<void> => {
    const services = createClIvilizationServices(NodeFileSystem).ClIvilization;
    const model = await extractAstNode<Model>(source, services);

    try {
        execSync("cargo --version")
    } catch (e) {
        console.error(chalk.red('Unable to find the Rust toolchain!'))
        console.log(chalk.dim("Go to https://rustup.rs/ to setup the Rust toolchain (or use the package provided by your operating system)."))
        return;
    }

    const spinner = yoctoSpinner().start('Generating game executable');
    try {

        const generatedFilePath = await generateBinary(model, source, destination);
        spinner.success(`Executable generated successfully: ${generatedFilePath}`)
    } catch (e) {
        spinner.error("Generation failed")
        return;
    }
};

export const astAction = async (source: string, destination: string): Promise<void> => {
    const services = createClIvilizationServices(NodeFileSystem).ClIvilization;
    const model = await extractAstNode<Model>(source, services);

    const spinner = yoctoSpinner().start('Generating AST file');
    try {

        const generatedFilePath = await generateAST(model, source, destination);
        spinner.success(`AST file generated successfully: ${generatedFilePath}`)
    } catch (e) {
        spinner.error("Generation failed")
        return;
    }
};

export default function(): void {
    const program = new Command();

    program.version(JSON.parse(packageContent).version);

    // TODO: use Program API to declare the CLI
    const fileExtensions = CLIvilizationLanguageMetaData.fileExtensions.join(', ');
    program
        .command('generate')
        .argument('<file>', `source file (possible file extensions: ${fileExtensions})`)
        .argument('<destination>', 'destination file')
        .description('Generates binary for a provided source file.')
        .action(generateAction);

    program
        .command('ast')
        .argument('<file>', `source file (possible file extensions: ${fileExtensions})`)
        .argument('<destination>', 'destination file')
        .description('Generates AST for a provided source file.')
        .action(astAction);

    program.parse(process.argv);
}
