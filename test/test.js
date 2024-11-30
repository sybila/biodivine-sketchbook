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

beforeEach(async function () {
  // Set timeout to 5 minutes to allow the program to build if needed
  this.timeout(300000);

  // Ensure the program has been built 
  // -- we skip this test for now, since it is faster to build it beforehand
  //spawnSync('cargo', ['tauri', 'build']);

  // Start Tauri driver
  tauriDriver = spawn(
    path.resolve(os.homedir(), '.cargo', 'bin', 'tauri-driver'),
    [],
    { stdio: 'ignore' }
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

afterEach(async function () {
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

/** Utility to simplify running query selector through shadow root of we-component elems. */
async function findInShadowRoot(element, selector, driver) {
  return await driver.executeScript(
    `
    let shadowRoot = arguments[0].shadowRoot;
    return shadowRoot ? shadowRoot.querySelector(arguments[1]) : null;
    `,
    element,
    selector
  );
}

/** Utility to open the example model from the signpost page. This is a common start
 * for multiple tests.
 */
async function openExampleModel(driver, rootComponent) {
    // find and click on button to open example model
    const initialScreenComponent = await findInShadowRoot(rootComponent, "initial-screen", driver);
    const loadExamplebutton = await findInShadowRoot(initialScreenComponent, '#load-example-button', driver);
    const loadButtonText = await loadExamplebutton.getText();
    expect(loadButtonText).to.match(/OPEN EXAMPLE/);
    await driver.executeScript("arguments[0].click();", loadExamplebutton);
    await sleep(500);
}

describe('Basic walkthrough test', () => {
  it('welcome message test', async () => {
    // Waiting for the app to fully initialize...
    await sleep(750);

    const rootComponent = await driver.findElement(By.css("root-component"));
    const initialScreenComponent = await findInShadowRoot(rootComponent, "initial-screen", driver);
    const heading = await findInShadowRoot(initialScreenComponent, "h2", driver);
    const headingText = await heading.getText();

    expect(headingText).to.match(/Welcome to SketchBook/);
  });
  it('Example inference walkthrough', async () => {
    // Waiting for the app to fully initialize...
    await sleep(750);

    // check we have one window opened
    const originalWindow = await driver.getWindowHandle();
    expect((await driver.getAllWindowHandles()).length).to.equal(1);
    const rootComponent = await driver.findElement(By.css("root-component"));

    await openExampleModel(driver, rootComponent)

    // find and click on button to open analysis tab
    const navBarComponent = await findInShadowRoot(rootComponent, "nav-bar", driver);
    const tabBarComponent = await findInShadowRoot(navBarComponent, "tab-bar", driver);
    const analysisButton = await findInShadowRoot(tabBarComponent, 'button:last-of-type', driver);
    const analysisButtonText = await analysisButton.getText();
    expect(analysisButtonText).to.match(/ANALYSIS/);
    await driver.executeScript("arguments[0].click();", analysisButton);

    await sleep(500);
    // find and  click on button to open inference window
    const contentPaneComponent = await findInShadowRoot(rootComponent, "#analysis", driver);
    const analysisTab = await findInShadowRoot(contentPaneComponent, "analysis-tab", driver);
    const inferenceButton = await findInShadowRoot(analysisTab, '#open-inference-button', driver);
    const inferenceButtonText = await inferenceButton.getText();
    expect(inferenceButtonText).to.match(/START INFERENCE WORKFLOW/);
    await driver.executeScript("arguments[0].click();", inferenceButton);

    // now lets wait a bit for the new window and then access it
    // https://www.selenium.dev/documentation/webdriver/interactions/windows/
    await driver.wait(
      async () => (await driver.getAllWindowHandles()).length === 2,
      2000
    );
    const windows = await driver.getAllWindowHandles();
    windows.forEach(async handle => {
      if (handle !== originalWindow) {
        await driver.switchTo().window(handle);
      }
    });
    
    // find and click button to run computation (only static inference, due to time)
    const analysisComponent = await driver.findElement(By.css("analysis-component"));
    const staticInferenceButton = await findInShadowRoot(analysisComponent, "#static-inference-button", driver);
    const staticInferenceButtonText = await staticInferenceButton.getText();
    expect(staticInferenceButtonText).to.match(/RUN STATIC INFERENCE/);
    await driver.executeScript("arguments[0].click();", staticInferenceButton);
    await sleep(2000);

    // check the results message (should contain correct number of admissible candidates)
    const resultsMessageElem = await findInShadowRoot(analysisComponent, ".overview-message", driver);
    const resultsMessageText = await resultsMessageElem.getText();
    expect(resultsMessageText).to.include("Number of satisfying candidates: 1296");
  });
  it('Example consistency check', async () => {
    // Waiting for the app to fully initialize...
    await sleep(750);
    const rootComponent = await driver.findElement(By.css("root-component"));
    await openExampleModel(driver, rootComponent)

    // find and click on button to open analysis tab
    const navBarComponent = await findInShadowRoot(rootComponent, "nav-bar", driver);
    const tabBarComponent = await findInShadowRoot(navBarComponent, "tab-bar", driver);
    const analysisButton = await findInShadowRoot(tabBarComponent, 'button:last-of-type', driver);
    const analysisButtonText = await analysisButton.getText();
    expect(analysisButtonText).to.match(/ANALYSIS/);
    await driver.executeScript("arguments[0].click();", analysisButton);

    await sleep(500);
    // find and  click on button to open inference window
    const contentPaneComponent = await findInShadowRoot(rootComponent, "#analysis", driver);
    const analysisTab = await findInShadowRoot(contentPaneComponent, "analysis-tab", driver);
    const consistencyCheckButton = await findInShadowRoot(analysisTab, '#consistency-check-button', driver);
    const consistencyCheckButtonText = await consistencyCheckButton.getText();
    expect(consistencyCheckButtonText).to.match(/RUN CONSISTENCY CHECK/);
    await driver.executeScript("arguments[0].click();", consistencyCheckButton);

    await sleep(500);
    // find the text area with the consistency check results
    const consistencyCheckArea = await findInShadowRoot(analysisTab, 'textarea', driver);
    const consistencyCheckAreaText = await consistencyCheckArea.getText();
    expect(consistencyCheckAreaText).to.match(/No issues with the sketch were discovered!/);
  });
});
