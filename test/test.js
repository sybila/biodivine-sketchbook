import os from 'os';
import path from 'path';
import { spawn, spawnSync } from 'child_process';
import { Builder, By, Capabilities } from 'selenium-webdriver';
import { expect } from 'chai';
import { fileURLToPath } from 'url';

// Create __dirname equivalent in ES modules
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Create the path to the expected application binary
const application = path.resolve(
  __dirname,
  '..',
  'src-tauri',
  'target',
  'release',
  'Biodivine Sketchbook.exe'
);

// Keep track of the WebDriver instance
let driver;

// Keep track of the Tauri driver process
let tauriDriver;

before(async function () {
  // Set timeout to 5 minutes to allow the program to build if needed
  this.timeout(300000);

  // Ensure the program has been built -- just build it beforehand for now
  //spawnSync('cargo', ['tauri', 'build']);

  // Start Tauri driver
  tauriDriver = spawn(
    path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
    [],
    { stdio: [null, process.stdout, process.stderr] }
  );

  const capabilities = new Capabilities();
  capabilities.set('tauri:options', {
    application: application,
    webviewOptions: {},
  });
  capabilities.setBrowserName('wry')

  // Start the WebDriver client
  driver = await new Builder()
    .withCapabilities(capabilities)
    .usingServer('http://localhost:4444/')
    .build();
});

after(async function () {
  // Stop the WebDriver session
  await driver.quit();

  // Kill the Tauri driver process
  if (tauriDriver) {
    tauriDriver.kill();
  }
});

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

describe('Basic test attempt', () => {
  it('should have welcome message', async () => {
    // Waiting for the app to fully initialize...
    await sleep(500);

    const rootComponent = await driver.findElement(By.css("root-component"));
    const initialScreenComponent = await driver.executeScript("return arguments[0].shadowRoot.querySelector('initial-screen');", rootComponent);
    const heading = await driver.executeScript("return arguments[0].shadowRoot.querySelector('h2');", initialScreenComponent);
    const headingText = await heading.getText();

    expect(headingText).to.match(/Welcome to SketchBook/);
  });
});
